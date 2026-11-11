use super::*;
use std::process::Command;

fn catalog_fixture() -> Value {
    json!({
        "repositories":{"cirnet-rules":{"classification":"supporting"},"product":{"classification":"active"},"old":{"classification":"archived"}},
        "control_plane":{"entrypoint":"en/AGENTS.md","skills":{"WORK":"en/skills/work/SKILL.md"}},
        "context":{"tasks":{"implementation":{"profile":"en/profiles/developer.md","skill":"en/skills/work/SKILL.md"}}},
        "assets":{"schema":1,"entries":[
            {"id":"AGENT-ENTRY","kind":"policy","path":"en/AGENTS.md","revision":2,"status":"ACTIVE","depends_on":["REF"]},
            {"id":"REF","kind":"reference","path":"en/references/work.md","revision":1,"status":"ACTIVE","depends_on":[]},
            {"id":"DRAFT","kind":"reference","path":"en/references/draft.md","revision":1,"status":"DRAFT","depends_on":[]},
            {"id":"WORK","kind":"skill","path":"en/skills/work/SKILL.md","revision":1,"status":"ACTIVE","depends_on":["REF"]},
            {"id":"ROLE-developer","kind":"profile","path":"en/profiles/developer.md","revision":1,"status":"ACTIVE","depends_on":[]},
            {"id":"HOOKS","kind":"hook","path":"en/hooks/README.md","revision":1,"status":"ACTIVE","depends_on":[]},
            {"id":"PRODUCT-REF","kind":"reference","path":"en/references/product.md","revision":1,"status":"ACTIVE","depends_on":[],"repositories":["product"]}
        ]}
    })
}

#[test]
fn catalog_rejects_cycles_aliases_and_ineligible_dependencies() {
    for (asset, field, value, message) in [
        (1, "depends_on", json!(["AGENT-ENTRY"]), "cycle"),
        (0, "depends_on", json!(["REF", "REF"]), "Duplicate"),
        (0, "depends_on", json!(["ABSENT"]), "Missing"),
        (0, "depends_on", json!(["DRAFT"]), "Ineligible"),
        (0, "depends_on", json!(["HOOKS"]), "Ineligible"),
        (0, "depends_on", json!(["PRODUCT-REF"]), "Ineligible"),
        (
            1,
            "path",
            json!("en/references/../rules/policy.md"),
            "Unsafe",
        ),
        (1, "path", json!("vi/references/work.md"), "English"),
        (1, "path", json!("en/references/CON.md"), "Unsafe"),
        (2, "id", json!("ref"), "Duplicate"),
        (2, "path", json!("en/references/WORK.md"), "Duplicate"),
        (6, "repositories", json!(["old"]), "assignment"),
    ] {
        let mut registry = catalog_fixture();
        registry["assets"]["entries"][asset][field] = value;
        let error = Catalog::load(&registry).unwrap_err();
        assert!(error.contains(message), "{error}");
    }
    let mut registry = catalog_fixture();
    registry["document_states"]["en/references/work.md"]["state"] = json!("RETIRED");
    assert!(Catalog::load(&registry).unwrap_err().contains("conflicts"));
    let mut registry = catalog_fixture();
    registry["context"]["common"] = json!(["en/AGENTS.md", "en/AGENTS.md"]);
    assert!(Catalog::load(&registry).unwrap_err().contains("Duplicate"));
}

struct Fixture(PathBuf);

impl Fixture {
    fn new(registry: &Value) -> Self {
        let root = crate::test_support::temporary_directory("cirnetrules-authority").unwrap();
        let f = Self(root);
        f.write("governance.yaml", &serde_json::to_vec(registry).unwrap());
        f.write(".gitattributes", b"* -text\n");
        for entry in registry["assets"]["entries"].as_array().unwrap() {
            f.write(entry["path"].as_str().unwrap(), b"authority fixture\r\n");
        }
        f.git(&["init", "-q"]);
        f.git(&["add", "--", "."]);
        f.git(&["commit", "-qm", "authority fixture"]);
        f
    }

    fn git(&self, args: &[&str]) {
        let output = Command::new("git")
            .current_dir(&self.0)
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgSign=false",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn write(&self, path: &str, bytes: &[u8]) {
        let file = self.0.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, bytes).unwrap();
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn resolver_reports_order_scope_budget_and_real_byte_digests() {
    let registry = catalog_fixture();
    let f = Fixture::new(&registry);
    let get = || {
        resolve(
            &f.0,
            None,
            &registry,
            "cirnet-rules",
            None,
            Some("developer"),
        )
        .unwrap()
    };
    let original = get();
    assert!(!original.dirty);
    assert_eq!(original.profile.as_deref(), Some("ROLE-developer"));
    assert_eq!(
        original
            .included
            .iter()
            .map(|source| source.id.as_str())
            .collect::<Vec<_>>(),
        ["REF", "AGENT-ENTRY", "ROLE-developer", "WORK"]
    );
    assert_eq!(
        original.included[0].sha256,
        docs::sha256(b"authority fixture\r\n")
    );
    assert!(original.excluded.iter().any(|asset| asset.id == "DRAFT"));
    assert!(
        original
            .excluded
            .iter()
            .any(|asset| asset.id == "PRODUCT-REF")
    );
    assert!(
        original
            .deferred
            .iter()
            .any(|asset| asset.id == "HOOKS" && asset.reason == "manual procedure only")
    );
    assert_eq!(original.digest, get().digest);
    let catalog = Catalog::load(&registry).unwrap();
    let selection = select(
        &catalog,
        &registry,
        "cirnet-rules",
        "implementation",
        None,
        None,
        "normal",
    )
    .unwrap();
    assert_eq!(
        snapshot(&f.0, selection, "fixture", (1024, 3))
            .err()
            .unwrap(),
        FILE_LIMIT
    );
    f.write("en/references/work.md", b"authority fixture\n");
    assert!(
        confirm_sources(&f.0, original.included.iter())
            .unwrap_err()
            .contains("changed")
    );
    let changed = get();
    assert!(changed.dirty);
    assert_ne!(original.digest, changed.digest);
    assert_eq!(original.commit, changed.commit);
    f.git(&["rm", "--cached", "--", "en/references/work.md"]);
    let error = resolve(&f.0, None, &registry, "cirnet-rules", None, None).unwrap_err();
    assert!(error.contains("Git index"));
}

#[test]
fn export_preserves_manual_procedures_without_widening_task_context() {
    let mut registry = catalog_fixture();
    registry["assets"]["entries"][5]["depends_on"] = json!(["MANUAL-REF", "REF"]);
    registry["assets"]["entries"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "id":"MANUAL-REF","kind":"reference","path":"en/references/manual.md",
            "revision":1,"status":"ACTIVE","depends_on":[]
        }));
    let f = Fixture::new(&registry);
    let resolve_context = || resolve(&f.0, None, &registry, "cirnet-rules", None, None).unwrap();
    let resolve_export =
        || resolve_for_export(&f.0, None, &registry, "cirnet-rules", None, None).unwrap();
    let context = resolve_context();
    assert!(context.manual.is_empty());
    let export = resolve_export();
    assert_eq!(
        export
            .manual
            .iter()
            .map(|source| source.id.as_str())
            .collect::<Vec<_>>(),
        ["MANUAL-REF", "HOOKS"]
    );
    assert_eq!(
        context
            .included
            .iter()
            .map(|source| &source.sha256)
            .collect::<Vec<_>>(),
        export
            .included
            .iter()
            .map(|source| &source.sha256)
            .collect::<Vec<_>>()
    );
    f.write("en/hooks/README.md", b"Updated manual procedure\n");
    assert_eq!(context.digest, resolve_context().digest);
    assert_ne!(export.digest, resolve_export().digest);
}
