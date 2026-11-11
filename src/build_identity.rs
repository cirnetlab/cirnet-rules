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
    let cli = field("cli");
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
    assert_eq!(cli, "cnr");
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    assert_eq!(fs::read_to_string("VERSION").unwrap().trim(), version);
    let cargo: toml::Value = toml::from_str(&fs::read_to_string("Cargo.toml").unwrap()).unwrap();
    assert!(
        cargo["bin"].as_array().unwrap().iter().any(|target| {
            target["name"].as_str() == Some(cli)
                && target["path"].as_str() == Some("cli/src/main.rs")
        }),
        "Cargo CLI target must match application identity"
    );
    let values = [
        ("PRODUCT_ID", slug.to_owned()),
        ("DISPLAY_NAME", display.to_owned()),
        ("PRODUCT_SYMBOL", slug.to_ascii_uppercase()),
        ("ORGANIZATION", organization.to_owned()),
        (
            "GOVERNANCE_ENV",
            format!("{}_GOVERNANCE", slug.to_ascii_uppercase()),
        ),
        ("RELEASE_NAME", format!("{repository}-{version}")),
        ("VERSION", version),
    ];
    let mut code = String::new();
    for (key, value) in values {
        code.push_str(&format!("pub const {key}: &str = {value:?};\n"));
    }
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("identity.rs"),
        code,
    )
    .unwrap();
}
