use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path, process::Command};

fn retired_name(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        concat!("open", "rules"),
        concat!("open-", "rules"),
        concat!("open ", "rules"),
        concat!("cg", "acp"),
    ]
    .iter()
    .any(|pattern| lower.contains(pattern))
}

#[test]
fn identity_inventory_requires_exact_retired_name_exceptions() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let listed = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-co", "--exclude-standard", "-z"])
        .output()
        .expect("list checked-in and pending source paths");
    assert!(listed.status.success());
    let allowlist = "identity-legacy-allowlist.json";
    let mut actual = BTreeMap::<(String, String), u64>::new();
    for name in String::from_utf8(listed.stdout)
        .unwrap()
        .split('\0')
        .filter(|p| !p.is_empty())
    {
        if name == allowlist || !root.join(name).is_file() {
            continue;
        }
        if retired_name(name) {
            actual.insert((name.into(), "filename".into()), 1);
        }
        let Ok(content) = fs::read_to_string(root.join(name)) else {
            continue;
        };
        for line in content.lines().filter(|line| retired_name(line)) {
            let digest = format!("{:x}", Sha256::digest(line.as_bytes()));
            *actual.entry((name.into(), digest)).or_default() += 1;
        }
    }
    let registry: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join(allowlist)).unwrap()).unwrap();
    assert_eq!(registry["version"], 1);
    let mut expected = BTreeMap::new();
    for entry in registry["exceptions"].as_array().unwrap() {
        let path = entry["path"].as_str().unwrap();
        assert!(
            entry["reason"]
                .as_str()
                .is_some_and(|reason| !reason.is_empty())
        );
        for (digest, count) in entry["matches"].as_object().unwrap() {
            assert!(
                expected
                    .insert(
                        (path.to_owned(), digest.to_owned()),
                        count.as_u64().unwrap()
                    )
                    .is_none()
            );
        }
    }
    let changes = actual
        .iter()
        .filter(|(key, count)| expected.get(*key) != Some(*count))
        .chain(
            expected
                .iter()
                .filter(|(key, _)| !actual.contains_key(*key)),
        )
        .collect::<Vec<_>>();
    assert!(
        changes.is_empty(),
        "Unreviewed or obsolete retired identity exceptions: {changes:?}"
    );
}
