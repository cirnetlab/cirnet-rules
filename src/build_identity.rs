use std::{env, fs, path::PathBuf};

fn main() {
    for input in ["application-identity.json", "Cargo.toml", "VERSION"] {
        println!("cargo:rerun-if-changed={input}");
    }
    let metadata: serde_json::Value =
        serde_json::from_str(&fs::read_to_string("application-identity.json").unwrap()).unwrap();
    let field = |key: &str| metadata[key].as_str().unwrap();
    let slug = field("product_id");
    let display = field("display_name");
    let organization = field("organization");
    let repository = field("repository");
    let protocol = metadata["ipc_protocol"]
        .as_u64()
        .filter(|&value| (1..=255).contains(&value))
        .unwrap();
    assert!(
        !slug.is_empty()
            && slug
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    );
    for value in [display, organization, repository] {
        assert!(
            !value.is_empty()
                && value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b" -".contains(&b))
        );
    }
    assert_eq!(slug, env::var("CARGO_PKG_NAME").unwrap());
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    assert_eq!(fs::read_to_string("VERSION").unwrap().trim(), version);
    let runtime = format!("{slug}-runtime");
    let cargo: toml::Value = toml::from_str(&fs::read_to_string("Cargo.toml").unwrap()).unwrap();
    assert!(
        cargo["bin"].as_array().unwrap().iter().any(|target| {
            target["name"].as_str() == Some(&runtime)
                && target["path"].as_str() == Some(&format!("src/bin/{runtime}.rs"))
        }),
        "Cargo runtime target must match application identity"
    );
    let values = [
        ("PRODUCT_ID", slug.to_owned()),
        ("DISPLAY_NAME", display.to_owned()),
        ("PRODUCT_SYMBOL", slug.to_ascii_uppercase()),
        ("ORGANIZATION", organization.to_owned()),
        ("API_NAMESPACE", format!("{slug}.")),
        ("DATABASE_FILENAME", format!("{slug}.sqlite3")),
        (
            "GOVERNANCE_ENV",
            format!("{}_GOVERNANCE", slug.to_ascii_uppercase()),
        ),
        (
            "TRACE_IPC_ENV",
            format!("{}_TRACE_IPC", slug.to_ascii_uppercase()),
        ),
        ("RELEASE_NAME", format!("{repository}-{version}")),
        ("VERSION", version),
    ];
    let mut code = format!("pub const IPC_PROTOCOL: u8 = {protocol};\n");
    for (key, value) in values {
        code.push_str(&format!("pub const {key}: &str = {value:?};\n"));
    }
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("identity.rs"),
        code,
    )
    .unwrap();
}
