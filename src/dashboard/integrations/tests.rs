use super::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    time::Duration,
};

fn fixture(adapter: &str, expected_path: &str, status: &str, body: &str) -> Value {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let expected_path = expected_path.to_owned();
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let server = thread::spawn(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let (mut stream, peer) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(error)
                    if error.kind() == io::ErrorKind::WouldBlock
                        && std::time::Instant::now() < deadline =>
                {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("loopback fixture could not accept the probe: {error}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        assert!(peer.ip().is_loopback());
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        let mut byte = [0u8; 1];
        while !request.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
            assert!(request.len() < 4096);
        }
        let request = String::from_utf8(request).unwrap();
        assert_eq!(
            request.lines().next().unwrap(),
            format!("GET {expected_path} HTTP/1.0")
        );
        let lowercase = request.to_ascii_lowercase();
        assert!(!lowercase.contains("authorization"));
        assert!(!lowercase.contains("cookie"));
        stream.write_all(response.as_bytes()).unwrap();
    });
    let result = Probe::parse(json!({"adapter":adapter,"port":port}))
        .unwrap()
        .run();
    server.join().unwrap();
    result
}

#[test]
fn openhands_pin_accepts_sdk_version_and_reports_no_authentication_evidence() {
    let result = fixture(
        "openhands",
        "/server_info",
        "200 OK",
        r#"{"sdk_version":"1.60.1"}"#,
    );
    assert_eq!(result["contract"], "MATCHED");
    assert_eq!(result["version"], "1.60.1");
    assert_eq!(result["authentication"], "NOT_VERIFIED");
    assert_eq!(result["native_execution"], "BLOCKED");
}

#[test]
fn opencode_v2_probe_checks_structure_and_exact_pin() {
    let result = fixture(
        "opencode",
        "/api/info",
        "200 OK",
        r#"{"version":"2.2.5","pid":10,"urls":["http://127.0.0.1"],"paths":{"tmp":"/tmp"},"capabilities":{"persistentPty":true}}"#,
    );
    assert_eq!(result["contract"], "MATCHED");
    assert!(result.get("urls").is_none());
    assert!(result.get("paths").is_none());
    let incomplete = fixture("opencode", "/api/info", "200 OK", r#"{"version":"2.2.5"}"#);
    assert_eq!(incomplete["contract"], "INVALID_RESPONSE");
    let v1 = fixture(
        "opencode",
        "/api/info",
        "200 OK",
        r#"{"version":"1.2.3","pid":1,"urls":[],"paths":{"tmp":"/tmp"}}"#,
    );
    assert_eq!(v1["contract"], "VERSION_MISMATCH");
}

#[test]
fn probe_rejects_extra_host_fields_and_distinguishes_authentication() {
    assert!(Probe::parse(json!({"adapter":"openhands","port":1234,"host":"example.com"})).is_err());
    assert!(Probe::parse(json!({"adapter":"openhands","port":0})).is_err());
    let result = fixture("openhands", "/server_info", "401 Unauthorized", "{}");
    assert_eq!(result["reachable"], true);
    assert_eq!(result["authentication"], "REQUIRED");
    assert_eq!(result["contract"], "AUTH_REQUIRED");
}

#[test]
fn dialect_preview_preserves_false_and_marks_privilege_fields_rejected() {
    let result = dispatch(
        "integration.import_preview",
        json!({
            "format":"opencode-v2-agent",
            "source":"{\"description\":\"Review\",\"disabled\":false,\"permissions\":[],\"request\":{},\"unknown\":0}"
        }),
    ).unwrap();
    assert_eq!(result["metadata"]["disabled"], false);
    assert_eq!(result["fields"][1]["field"], "disabled");
    assert!(
        result["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| { entry["field"] == "permissions" && entry["status"] == "REJECTED" })
    );
    assert!(
        result["fields"].as_array().unwrap().iter().any(|entry| {
            entry["field"] == "unknown" && entry["status"] == "RETAINED_AS_SOURCE"
        })
    );
}

#[test]
fn v1_and_v2_agent_fields_cannot_be_silently_combined() {
    for format in ["opencode-v1-agent", "opencode-v2-agent"] {
        let result = dispatch(
            "integration.import_preview",
            json!({"format":format,"source":r#"{"permission":"deny","permissions":[]}"#}),
        )
        .unwrap();
        assert!(
            result["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| { entry["code"] == "DIALECT_FIELD" })
        );
        assert_eq!(result["metadata"], json!({}));
    }
}

#[test]
fn openhands_skill_frontmatter_reports_opaque_extensions() {
    let result = dispatch(
        "integration.import_preview",
        json!({"format":"openhands-skill","source":"---\nname: review\ndescription: Review the change\nlicense: MIT\nmetadata:\n  id: local\n---\n# Review\n"}),
    ).unwrap();
    assert_eq!(result["metadata"]["name"], "review");
    assert_eq!(result["metadata"]["license"], "MIT");
    assert_eq!(result["diagnostics"], json!([]));
    assert!(
        result["fields"].as_array().unwrap().iter().any(|entry| {
            entry["field"] == "metadata" && entry["status"] == "RETAINED_AS_SOURCE"
        })
    );
}

#[test]
fn malformed_or_duplicate_fields_carry_a_location_and_no_metadata() {
    for source in ["{\n\"model\":\"a\",\n\"model\":\"b\"}", "{\n\"model\": [}"] {
        let result = dispatch(
            "integration.import_preview",
            json!({"format":"opencode-v1-agent","source":source}),
        )
        .unwrap();
        assert!(result["diagnostics"][0]["line"].as_u64().unwrap() >= 1);
        assert_eq!(result["metadata"], json!({}));
    }
}
