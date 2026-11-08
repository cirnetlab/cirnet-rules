use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    env,
    fs::{self, Metadata, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
};

const LIMIT: u64 = 1024 * 1024;
const CONTEXT: &str = "context.json";
const RECEIPT: &str = "receipt.json";

pub struct Request {
    pub action: String,
    pub agent: String,
    pub session_root: PathBuf,
    pub context: Value,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Context {
    governance_root: PathBuf,
    product_root: PathBuf,
    repository: String,
    task: String,
    risk: String,
    profile: String,
    plan_reference: String,
    audit_reference: String,
    governance_commit: String,
    governance_clean: bool,
    target_commit: String,
    documents: Vec<Document>,
    scope: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Document {
    path: String,
    sha256: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Receipt {
    schema_version: u8,
    agent: String,
    instructions_sha256: String,
    context_sha256: String,
}

struct Bundle {
    instructions: Vec<u8>,
    context: Vec<u8>,
    receipt: Vec<u8>,
}

impl Bundle {
    fn files(&self, entry: &'static str) -> [(&'static str, &[u8]); 3] {
        [
            (entry, &self.instructions),
            (CONTEXT, &self.context),
            (RECEIPT, &self.receipt),
        ]
    }

    fn backup(&self, root: &Path, agent: &str) -> PathBuf {
        root.join(format!(
            "cirnet-bootstrap-{agent}.backup-{}",
            sha(&self.receipt)
        ))
    }
}

pub fn run(request: Request) -> Result<Value, String> {
    let entry = match request.agent.as_str() {
        "codex" => "AGENTS.md",
        "claude" => "CLAUDE.md",
        "antigravity" => "GEMINI.md",
        _ => return Err("Unknown bootstrap agent".into()),
    };
    if !matches!(request.action.as_str(), "dry-run" | "install" | "uninstall") {
        return Err("Unknown bootstrap action".into());
    }
    let context = read_context(request.context)?;
    let root = directory(&request.session_root)?;
    let governance = directory(&context.governance_root)?;
    let product = directory(&context.product_root)?;
    external(&root, &governance, &product)?;
    let active = root.join(format!("cirnet-bootstrap-{}", request.agent));

    if request.action == "dry-run" {
        verify_sources(&governance, &context.documents)?;
        let mut preview = report("DRY_RUN", &active, entry, None);
        preview["manifest"] = serde_json::to_value(context).map_err(|_| "Invalid manifest")?;
        return Ok(preview);
    }

    let _lock = Lock::acquire(&root, &request.agent)?;
    if request.action == "uninstall" {
        let Some(installed) = existing(&active, entry, &request.agent)? else {
            return Ok(report("ABSENT", &active, entry, None));
        };
        let installed_context = decode_context(&installed.context)?;
        if installed_context.repository != context.repository
            || !same_path(&installed_context.governance_root, &context.governance_root)
            || !same_path(&installed_context.product_root, &context.product_root)
        {
            return Err("The active bundle belongs to a different checkout".into());
        }
        let backup = installed.backup(&root, &request.agent);
        move_bundle(
            &active,
            &backup,
            "The rollback destination already exists",
            "Unable to move the verified bundle to backup",
        )?;
        return Ok(report("ROLLED_BACK", &active, entry, Some(&backup)));
    }

    verify_sources(&governance, &context.documents)?;
    let bundle = render(&context, &request.agent)?;
    if let Some(installed) = existing(&active, entry, &request.agent)? {
        return if installed.receipt == bundle.receipt {
            Ok(report("UNCHANGED", &active, entry, None))
        } else {
            Err(
                "The active bundle differs. Back it up with uninstall before changing context."
                    .into(),
            )
        };
    }
    let backup = bundle.backup(&root, &request.agent);
    if let Some(saved) = existing(&backup, entry, &request.agent)? {
        if saved.receipt != bundle.receipt {
            return Err("Backup identity differs from the requested bundle".into());
        }
        move_bundle(
            &backup,
            &active,
            "The restore destination already exists",
            "Unable to restore the verified backup",
        )?;
        return Ok(report("RESTORED", &active, entry, None));
    }
    stage(&root, &active, entry, &bundle)?;
    Ok(report("INSTALLED", &active, entry, None))
}

fn report(state: &str, active: &Path, entry: &str, backup: Option<&Path>) -> Value {
    json!({
        "status": "PASS",
        "state": state,
        "mode": "isolated-manual-attachment",
        "directory": active,
        "files": [entry, CONTEXT, RECEIPT],
        "backup": backup,
        "harnessStarted": false,
    })
}

fn read_context(value: Value) -> Result<Context, String> {
    let context: Context = serde_json::from_value(value).map_err(|_| "Invalid context shape")?;
    let identifier = |value: &str| {
        !value.is_empty()
            && value.len() <= 80
            && value
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
    };
    if context.scope != "read-only"
        || !identifier(&context.repository)
        || !identifier(&context.task)
        || !identifier(&context.risk)
        || !identifier(&context.profile)
        || !english_path(&context.plan_reference)
        || !english_path(&context.audit_reference)
        || !hex(&context.governance_commit, 40)
        || !hex(&context.target_commit, 40)
        || context.documents.is_empty()
        || context.documents.len() > 128
        || !context
            .documents
            .iter()
            .any(|doc| doc.path == "en/AGENTS.md")
    {
        return Err("Expected bounded read-only English context with Git provenance".into());
    }
    let mut paths = BTreeSet::new();
    for doc in &context.documents {
        if !english_path(&doc.path) || !hex(&doc.sha256, 64) || !paths.insert(&doc.path) {
            return Err("Context contains an unsafe or non-English document".into());
        }
    }
    Ok(context)
}

fn english_path(path: &str) -> bool {
    path.starts_with("en/")
        && path.ends_with(".md")
        && path.len() <= 512
        && path.split('/').all(|part| {
            !part.is_empty()
                && !matches!(part, "." | "..")
                && part
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        })
}

fn decode_context(bytes: &[u8]) -> Result<Context, String> {
    read_context(serde_json::from_slice(bytes).map_err(|_| "Invalid saved context")?)
}

fn verify_sources(root: &Path, docs: &[Document]) -> Result<(), String> {
    for doc in docs {
        let path = root.join(&doc.path);
        ordinary_ancestors(&path)?;
        let data = read_file(&path)?;
        let source = std::str::from_utf8(&data).map_err(|_| "Invalid UTF-8 instruction source")?;
        if source.starts_with('\u{feff}')
            || sha(source.replace("\r\n", "\n").as_bytes()) != doc.sha256
        {
            return Err(format!("Instruction source drift: {}", doc.path));
        }
    }
    Ok(())
}

fn render(context: &Context, agent: &str) -> Result<Bundle, String> {
    let context_bytes = json_bytes(context)?;
    let mut instructions = format!(
        concat!(
            "# CIRNETLAB session bootstrap\n\n",
            "This session targets `{0}` and repository `{1}`. The task is `{2}` ",
            "at risk level `{3}` under profile `{4}`.\n\n",
            "This file describes a read-only audit. Read the adjacent `context.json` for the checkout ",
            "roots and full commit IDs. Read the listed English source documents in order. Stop ",
            "if a source is missing, unreadable or has changed. Treat translations and historical ",
            "sources as reference material. Host permissions govern access.\n\n",
            "The plan is `{5}` and the audit reference is `{6}`.\n\n",
        ),
        agent,
        context.repository,
        context.task,
        context.risk,
        context.profile,
        context.plan_reference,
        context.audit_reference,
    );
    for doc in &context.documents {
        instructions.push_str(&format!("- `{}` (SHA-256 `{}`)\n", doc.path, doc.sha256));
    }
    let instructions = instructions.into_bytes();
    let receipt = json_bytes(&Receipt {
        schema_version: 1,
        agent: agent.to_owned(),
        instructions_sha256: sha(&instructions),
        context_sha256: sha(&context_bytes),
    })?;
    Ok(Bundle {
        instructions,
        context: context_bytes,
        receipt,
    })
}

fn existing(path: &Path, entry: &str, agent: &str) -> Result<Option<Bundle>, String> {
    if !path_exists(path)? {
        return Ok(None);
    }
    directory(path)?;
    let mut count = 0;
    for item in fs::read_dir(path).map_err(failure("Unable to list adapter bundle"))? {
        let item = item.map_err(failure("Unable to inspect adapter bundle"))?;
        if ![entry, CONTEXT, RECEIPT]
            .iter()
            .any(|name| item.file_name() == *name)
        {
            return Err("Adapter bundle contains an unrecognized entry".into());
        }
        count += 1;
    }
    if count != 3 {
        return Err("Adapter bundle is incomplete".into());
    }
    let context = read_file(&path.join(CONTEXT))?;
    let receipt = read_file(&path.join(RECEIPT))?;
    let instructions = read_file(&path.join(entry))?;
    let recorded: Receipt =
        serde_json::from_slice(&receipt).map_err(|_| "Invalid saved receipt")?;
    if recorded.schema_version != 1
        || recorded.agent != agent
        || recorded.instructions_sha256 != sha(&instructions)
        || recorded.context_sha256 != sha(&context)
        || json_bytes(&recorded)? != receipt
    {
        return Err("Adapter bundle changed since installation".into());
    }
    Ok(Some(Bundle {
        instructions,
        context,
        receipt,
    }))
}

fn stage(root: &Path, active: &Path, entry: &'static str, bundle: &Bundle) -> Result<(), String> {
    let stage = root.join(format!(
        ".{}.stage-{}",
        active.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    ));
    ordinary_ancestors(root)?;
    #[cfg(unix)]
    let created = {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new().mode(0o700).create(&stage)
    };
    #[cfg(not(unix))]
    let created = fs::create_dir(&stage);
    created.map_err(failure("Unable to create adapter stage"))?;
    // Keep any partial stage for inspection. Its path may no longer identify
    // the directory created above, so it is unsafe to clean up by filename.
    for (name, bytes) in bundle.files(entry) {
        ordinary_ancestors(&stage)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(stage.join(name))
            .map_err(failure("Unable to create staged adapter file"))?;
        output
            .write_all(bytes)
            .and_then(|_| output.sync_all())
            .map_err(failure("Unable to persist staged adapter file"))?;
    }
    move_bundle(
        &stage,
        active,
        "Adapter destination changed during installation",
        "Unable to activate staged adapter bundle",
    )
}

fn move_bundle(
    source: &Path,
    destination: &Path,
    occupied: &'static str,
    error: &'static str,
) -> Result<(), String> {
    // Both paths are immediate children of the same session directory.
    ordinary_ancestors(source)?;
    if path_exists(destination)? {
        return Err(occupied.into());
    }
    fs::rename(source, destination).map_err(failure(error))
}

struct Lock(PathBuf);

impl Lock {
    fn acquire(root: &Path, agent: &str) -> Result<Self, String> {
        ordinary_ancestors(root)?;
        let path = root.join(format!(".cirnet-bootstrap-{agent}.lock"));
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(failure("Adapter session is busy or cannot be locked"))?;
        Ok(Self(path))
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        if ordinary_ancestors(&self.0).is_ok() {
            let _ = fs::remove_file(&self.0);
        }
    }
}

fn directory(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, Component::CurDir | Component::ParentDir))
    {
        return Err("Expected an absolute directory without traversal".into());
    }
    ordinary_ancestors(path)?;
    if !fs::metadata(path)
        .map_err(failure("Unable to inspect directory"))?
        .is_dir()
    {
        return Err("Expected a regular directory".into());
    }
    fs::canonicalize(path).map_err(failure("Unable to resolve directory"))
}

fn ordinary_ancestors(path: &Path) -> Result<(), String> {
    for parent in path.ancestors().filter(|p| !p.as_os_str().is_empty()) {
        let metadata = fs::symlink_metadata(parent).map_err(failure("Unable to inspect path"))?;
        if link(&metadata) {
            return Err("Adapter paths may not cross a symlink or reparse point".into());
        }
    }
    Ok(())
}

fn external(root: &Path, governance: &Path, product: &Path) -> Result<(), String> {
    if [governance, product]
        .iter()
        .any(|checkout| inside(root, checkout) || inside(checkout, root))
    {
        return Err(
            "Adapter session must be disjoint from product and governance checkouts".into(),
        );
    }
    for parent in root.ancestors() {
        if path_exists(&parent.join(".git"))?
            || (path_exists(&parent.join("HEAD"))? && path_exists(&parent.join("objects"))?)
        {
            return Err("Adapter session must be outside Git checkouts".into());
        }
    }
    let names: Vec<_> = root
        .iter()
        .map(|part| part.to_string_lossy().to_ascii_lowercase())
        .collect();
    for (i, name) in names.iter().enumerate() {
        if matches!(
            name.as_str(),
            ".claude" | ".gemini" | ".agents" | ".antigravity" | ".config"
        ) || (name == "antigravity" && names[..i].iter().any(|part| part == "appdata"))
            || (name == ".codex" && names.get(i + 1).is_none_or(|next| next != "visualizations"))
        {
            return Err("Adapter session may not be a native agent configuration directory".into());
        }
    }
    for key in ["CODEX_HOME", "CLAUDE_CONFIG_DIR", "XDG_CONFIG_HOME"] {
        if let Some(path) = env::var_os(key).and_then(|p| fs::canonicalize(p).ok())
            && inside(root, &path)
            && !(key == "CODEX_HOME" && inside(root, &path.join("visualizations")))
        {
            return Err("Adapter session may not overlap a configured agent directory".into());
        }
    }
    Ok(())
}

fn inside(path: &Path, root: &Path) -> bool {
    let mut parts = path.components();
    root.components().all(|expected| {
        parts.next().is_some_and(|actual| {
            if cfg!(windows) {
                actual
                    .as_os_str()
                    .eq_ignore_ascii_case(expected.as_os_str())
            } else {
                actual == expected
            }
        })
    })
}

fn same_path(left: &Path, right: &Path) -> bool {
    inside(left, right) && inside(right, left)
}

fn read_file(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::symlink_metadata(path).map_err(failure("Unable to inspect adapter file"))?;
    if link(&metadata) || !metadata.is_file() || metadata.len() > LIMIT {
        return Err("Expected a bounded regular adapter file".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(failure("Unable to open adapter file"))?
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(failure("Unable to read adapter file"))?;
    if bytes.len() as u64 > LIMIT {
        return Err("Adapter file exceeded its size bound".into());
    }
    Ok(bytes)
}

fn path_exists(path: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(failure("Unable to inspect adapter path")(error)),
    }
}

fn link(metadata: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        metadata.file_type().is_symlink()
    }
}

fn failure(message: &'static str) -> impl FnOnce(io::Error) -> String {
    move |error| format!("{message}: {}", error.kind())
}

fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn json_bytes(value: &impl Serialize) -> Result<Vec<u8>, String> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|_| "Unable to serialize adapter file")?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn test_parent() -> PathBuf {
        let path = env::var_os("CIRNET_ADAPTER_TEST_TMP")
            .map(PathBuf::from)
            .unwrap_or_else(env::temp_dir);
        fs::canonicalize(path).unwrap()
    }

    struct Fixture {
        root: PathBuf,
        governance: PathBuf,
        product: PathBuf,
        session: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let parent = test_parent();
            let root = loop {
                let candidate = parent.join(format!(
                    "cirnet-adapter-test-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
                match fs::create_dir(&candidate) {
                    Ok(()) => break candidate,
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => panic!("Unable to create isolated adapter test: {error}"),
                }
            };
            let governance = root.join("governance");
            let product = root.join("product");
            let session = root.join("external session");
            fs::create_dir(&governance).unwrap();
            fs::create_dir(&product).unwrap();
            fs::create_dir(&session).unwrap();
            fs::create_dir(governance.join("en")).unwrap();
            fs::create_dir(governance.join("vi")).unwrap();
            fs::write(governance.join("en/AGENTS.md"), b"English source\r\n").unwrap();
            fs::write(governance.join("vi/AGENTS.md"), b"Vietnamese reference\n").unwrap();
            Self {
                root,
                governance,
                product,
                session,
            }
        }

        fn request(&self, action: &str, agent: &str) -> Request {
            Request {
                action: action.into(),
                agent: agent.into(),
                session_root: self.session.clone(),
                context: json!({
                    "governanceRoot": self.governance,
                    "productRoot": self.product,
                    "repository": "cirnet-remote",
                    "task": "audit",
                    "risk": "high",
                    "profile": "rust-service",
                    "planReference": "en/plans/cirnet-remote/roadmap.md",
                    "auditReference": "en/specs/cirnet-remote/audit.md",
                    "governanceCommit": "1".repeat(40),
                    "governanceClean": true,
                    "targetCommit": "2".repeat(40),
                    "documents": [{ "path": "en/AGENTS.md", "sha256": sha(b"English source\n") }],
                    "scope": "read-only",
                }),
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            if let Ok(resolved) = fs::canonicalize(&self.root) {
                let parent = test_parent();
                assert_eq!(resolved.parent(), Some(parent.as_path()));
                assert!(
                    resolved
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .starts_with("cirnet-adapter-test-")
                );
                let _ = fs::remove_dir_all(resolved);
            }
        }
    }

    #[test]
    fn all_surfaces_are_idempotent_and_reversible_in_an_external_session() {
        let fixture = Fixture::new();
        for (agent, entry) in [
            ("codex", "AGENTS.md"),
            ("claude", "CLAUDE.md"),
            ("antigravity", "GEMINI.md"),
        ] {
            let dry = run(fixture.request("dry-run", agent)).unwrap();
            let active: PathBuf = serde_json::from_value(dry["directory"].clone()).unwrap();
            assert!(!active.exists());
            assert_eq!(dry["status"], "PASS");
            assert_eq!(dry["state"], "DRY_RUN");
            assert_eq!(dry["manifest"]["profile"], "rust-service");
            assert_eq!(dry["harnessStarted"], false);
            assert_eq!(
                run(fixture.request("install", agent)).unwrap()["state"],
                "INSTALLED"
            );
            let instructions = fs::read_to_string(active.join(entry)).unwrap();
            assert!(instructions.contains("en/AGENTS.md"));
            assert!(!instructions.contains("vi/"));
            assert_eq!(
                run(fixture.request("install", agent)).unwrap()["state"],
                "UNCHANGED"
            );
            let removed = run(fixture.request("uninstall", agent)).unwrap();
            let backup: PathBuf = serde_json::from_value(removed["backup"].clone()).unwrap();
            assert!(!active.exists());
            assert_eq!(removed["state"], "ROLLED_BACK");
            assert_eq!(
                fs::read_to_string(backup.join(entry)).unwrap(),
                instructions
            );
            assert_eq!(
                run(fixture.request("uninstall", agent)).unwrap()["state"],
                "ABSENT"
            );
            assert_eq!(
                run(fixture.request("install", agent)).unwrap()["state"],
                "RESTORED"
            );
            assert_eq!(
                fs::read_to_string(active.join(entry)).unwrap(),
                instructions
            );
        }
        assert_eq!(fs::read_dir(&fixture.product).unwrap().count(), 0);
    }

    #[test]
    fn mismatched_context_and_config_scopes_are_rejected_without_writing() {
        let fixture = Fixture::new();
        let mut request = fixture.request("install", "codex");
        request.context["documents"][0]["path"] = json!("vi/AGENTS.md");
        assert!(run(request).unwrap_err().contains("English"));
        let mut request = fixture.request("install", "codex");
        request.context["apiKey"] = json!("never-report-this");
        let error = run(request).unwrap_err();
        assert!(!error.contains("never-report-this"));
        let mut request = fixture.request("install", "codex");
        request.session_root = fixture.product.clone();
        assert!(run(request).unwrap_err().contains("disjoint"));
        let git_root = fixture.root.join("another-project");
        fs::create_dir(&git_root).unwrap();
        fs::write(git_root.join(".git"), "fixture worktree pointer").unwrap();
        let mut request = fixture.request("install", "codex");
        request.session_root = git_root;
        assert!(run(request).unwrap_err().contains("Git"));
        let profile = fixture.root.join(".claude");
        fs::create_dir(&profile).unwrap();
        let mut request = fixture.request("install", "claude");
        request.session_root = profile;
        assert!(run(request).unwrap_err().contains("configuration"));
        assert_eq!(fs::read_dir(&fixture.session).unwrap().count(), 0);
    }

    #[test]
    fn failed_activation_preserves_the_stage_and_existing_destination() {
        let fixture = Fixture::new();
        let context = read_context(fixture.request("install", "codex").context).unwrap();
        let bundle = render(&context, "codex").unwrap();
        let active = fixture.session.join("cirnet-bootstrap-codex");
        fs::create_dir(&active).unwrap();
        fs::write(active.join("AGENTS.md"), "existing instructions").unwrap();

        assert!(stage(&fixture.session, &active, "AGENTS.md", &bundle).is_err());
        assert_eq!(
            fs::read_to_string(active.join("AGENTS.md")).unwrap(),
            "existing instructions"
        );
        let staged: Vec<_> = fs::read_dir(&fixture.session)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path != &active)
            .collect();
        assert_eq!(staged.len(), 1);
        for (name, bytes) in bundle.files("AGENTS.md") {
            assert_eq!(fs::read(staged[0].join(name)).unwrap(), bytes);
        }
    }

    #[cfg(unix)]
    #[test]
    fn lock_cleanup_refuses_a_redirected_session() {
        use std::os::unix::fs::symlink;

        let fixture = Fixture::new();
        let lock = Lock::acquire(&fixture.session, "codex").unwrap();
        fs::rename(&fixture.session, fixture.root.join("old session")).unwrap();
        symlink(&fixture.product, &fixture.session).unwrap();
        let outside_lock = fixture.product.join(".cirnet-bootstrap-codex.lock");
        fs::write(&outside_lock, "unrelated lock").unwrap();
        drop(lock);
        assert_eq!(fs::read_to_string(outside_lock).unwrap(), "unrelated lock");
    }

    #[test]
    fn stale_sources_and_modified_bundles_fail_closed() {
        let fixture = Fixture::new();
        let request = fixture.request("dry-run", "antigravity");
        let preview = run(request).unwrap();
        let active: PathBuf = serde_json::from_value(preview["directory"].clone()).unwrap();
        fs::write(fixture.governance.join("en/AGENTS.md"), "changed").unwrap();
        assert!(
            run(fixture.request("install", "antigravity"))
                .unwrap_err()
                .contains("source drift")
        );
        assert!(!active.exists());
        fs::write(fixture.governance.join("en/AGENTS.md"), "English source\n").unwrap();
        run(fixture.request("install", "antigravity")).unwrap();
        let entry = active.join("GEMINI.md");
        fs::write(&entry, "custom data").unwrap();
        assert!(run(fixture.request("uninstall", "antigravity")).is_err());
        assert_eq!(fs::read_to_string(entry).unwrap(), "custom data");
    }
}
