use crate::{authority, coordination, docs, policy, runner, tasks, workspace};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const MAX_FILE: u64 = 4 * 1024 * 1024;
const MAX_INPUTS: usize = 256;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Bundle {
    schema: u32,
    task: String,
    repository: String,
    base_commit: String,
    authority_commit: String,
    registry_sha256: String,
    authority_sha256: String,
    authority_sources: BTreeMap<String, String>,
    descriptor_sha256: String,
    approval_epoch: u64,
    submission_sha256: String,
    source_sha256: String,
    write_scope: BTreeSet<String>,
    verification_scope: BTreeSet<String>,
    changed: BTreeSet<String>,
    files: BTreeMap<String, FileEvidence>,
    commands: BTreeSet<String>,
    budget: BTreeMap<String, u64>,
    dependencies: Vec<Dependency>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct FileEvidence {
    sha256: Option<String>,
    executable: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    repository: String,
    path: String,
    sha256: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Dependency {
    task_id: String,
    revision: u64,
    approval_epoch: u64,
    source_sha256: String,
    evidence_sha256: String,
    outputs: Vec<Artifact>,
    selected_inputs: Vec<Artifact>,
}

fn hash(text: &str, length: usize) -> bool {
    text.len() == length
        && text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn git_id(text: &str) -> bool {
    hash(text, 40) || hash(text, 64)
}

fn value<'a>(record: &'a Value, key: &str) -> Result<&'a str, String> {
    record[key]
        .as_str()
        .ok_or_else(|| format!("Missing candidate evidence field {key}"))
}

fn strings(record: &Value) -> Result<BTreeSet<String>, String> {
    record
        .as_array()
        .ok_or("Invalid evidence list")?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or("Invalid evidence item".into())
        })
        .collect()
}

fn registry_digest(root: &Path) -> Result<String, String> {
    let registry = policy::read(&policy::safe_path(root, "governance.yaml")?)?;
    Ok(docs::sha256(registry.as_bytes()))
}

fn bounded_file(path: &Path) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(|_| "Evidence file is unavailable")?;
    if policy::is_link(&meta) || !meta.is_file() || meta.len() > MAX_FILE {
        return Err("Evidence input must be a bounded ordinary file".into());
    }
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(MAX_FILE + 1).read_to_end(&mut bytes))
        .map_err(|_| "Cannot read evidence bytes")?;
    if bytes.len() as u64 > MAX_FILE {
        return Err("Evidence input grew beyond its byte limit".into());
    }
    Ok(bytes)
}

fn cli_file(input: &Path) -> Result<PathBuf, String> {
    if !input.is_absolute() {
        return Err("Evidence files require an absolute path".into());
    }
    let name = input
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or("Invalid evidence filename")?;
    let parent = input.parent().ok_or("Missing evidence parent directory")?;
    let parent = workspace::workspace_path(parent, Some(parent))?;
    workspace::scoped_path(&parent, name)
}

fn load(path: &Path) -> Result<(Bundle, String), String> {
    let bytes = bounded_file(&cli_file(path)?)?;
    let bundle: Bundle =
        serde_json::from_slice(&bytes).map_err(|_| "Malformed portable evidence bundle")?;
    if bytes != serde_json::to_vec(&bundle).map_err(|_| "Cannot encode portable evidence")? {
        return Err("Bundle does not use the canonical evidence encoding".into());
    }
    validate(&bundle)?;
    Ok((bundle, docs::sha256(&bytes)))
}

fn validate(bundle: &Bundle) -> Result<(), String> {
    if bundle.schema != 1
        || bundle.task.is_empty()
        || bundle.approval_epoch == 0
        || !git_id(&bundle.base_commit)
        || !git_id(&bundle.authority_commit)
        || bundle.authority_sources.is_empty()
        || bundle.authority_sources.len() > 130
        || bundle.files.len() > MAX_INPUTS
        || bundle.commands.is_empty()
        || bundle.commands.len() > 32
        || bundle
            .budget
            .keys()
            .any(|key| !matches!(key.as_str(), "timeout_ms" | "log_bytes"))
        || bundle.dependencies.len() > 32
        || !bundle.changed.is_subset(&bundle.write_scope)
        || bundle
            .changed
            .iter()
            .any(|path| !bundle.files.contains_key(path))
        || bundle
            .verification_scope
            .iter()
            .any(|path| !bundle.files.contains_key(path))
        || [
            &bundle.registry_sha256,
            &bundle.authority_sha256,
            &bundle.descriptor_sha256,
            &bundle.submission_sha256,
            &bundle.source_sha256,
        ]
        .iter()
        .any(|digest| !hash(digest, 64))
    {
        return Err("Invalid portable evidence identities or scope".into());
    }
    for path in bundle.files.keys().chain(&bundle.write_scope) {
        tasks::relative(path)?;
    }
    if bundle.files.values().any(|file| {
        file.sha256.as_ref().is_some_and(|digest| !hash(digest, 64))
            || file.sha256.is_none() && file.executable
    }) {
        return Err("Invalid portable file digest".into());
    }
    for (path, digest) in &bundle.authority_sources {
        tasks::relative(path)?;
        if !path.starts_with("en/") || !hash(digest, 64) {
            return Err("Invalid portable authority source identity".into());
        }
    }
    for dependency in &bundle.dependencies {
        if dependency.task_id.is_empty()
            || dependency.revision == 0
            || dependency.approval_epoch == 0
            || !hash(&dependency.source_sha256, 64)
            || !hash(&dependency.evidence_sha256, 64)
            || dependency.outputs.len() > MAX_INPUTS
            || dependency.selected_inputs.len() > MAX_INPUTS
        {
            return Err("Invalid dependency evidence identity".into());
        }
        for artifact in dependency.outputs.iter().chain(&dependency.selected_inputs) {
            tasks::relative(&artifact.path)?;
            if artifact.repository.is_empty()
                || artifact
                    .sha256
                    .as_ref()
                    .is_some_and(|digest| !hash(digest, 64))
            {
                return Err("Invalid dependency artifact digest".into());
            }
        }
    }
    Ok(())
}

fn tree_entry(
    checkout: &Path,
    commit: &str,
    relative: &str,
) -> Result<Option<(bool, String)>, String> {
    let output = policy::git(
        checkout,
        &[
            "--literal-pathspecs",
            "ls-tree",
            "-z",
            "--full-tree",
            commit,
            "--",
            relative,
        ],
    )?;
    if output.is_empty() {
        return Ok(None);
    }
    let row = output.strip_suffix('\0').ok_or("Invalid Git tree entry")?;
    let (metadata, path) = row.split_once('\t').ok_or("Invalid Git tree entry")?;
    if path != relative || row.contains('\0') {
        return Err("Git tree entry path differs from the requested scoped path".into());
    }
    let mut fields = metadata.split(' ');
    let mode = match fields.next() {
        Some("100644") => false,
        Some("100755") => true,
        _ => {
            return Err(
                "Candidate scope contains a symlink, submodule or other non-file entry".into(),
            );
        }
    };
    if fields.next() != Some("blob") {
        return Err("Candidate scope does not refer to a Git blob".into());
    }
    let oid = fields
        .next()
        .filter(|value| git_id(value))
        .ok_or("Invalid Git blob identity")?;
    if fields.next().is_some() {
        return Err("Invalid Git tree metadata".into());
    }
    Ok(Some((mode, oid.into())))
}

fn index_mode(input: &Value, path: &Path) -> Result<bool, String> {
    let index_executable = if let Some(index) = input["index"].as_array() {
        let row = index
            .first()
            .and_then(Value::as_str)
            .ok_or("Malformed Git index entry")?;
        if index.len() != 1 || !row.ends_with(" 0") {
            return Err("Cannot seal an unmerged Git index".into());
        }
        match row.split(' ').next() {
            Some("100644") => false,
            Some("100755") => true,
            _ => return Err("Cannot seal a non-file Git index entry".into()),
        }
    } else {
        false
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = index_executable;
        Ok(fs::metadata(path)
            .map_err(|_| "Scoped file unavailable")?
            .permissions()
            .mode()
            & 0o111
            != 0)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(index_executable)
    }
}

fn export(
    root: &Path,
    registry: &Value,
    work: Option<&Path>,
    id: &str,
    output: &Path,
) -> Result<Value, String> {
    let task = tasks::load_current(root, id)?;
    tasks::authorize(&task, "read")?;
    let descriptor = &task["descriptor"];
    let repo = value(descriptor, "primary_repo")?;
    if repo == "_workspace"
        || descriptor["required_children"]
            .as_array()
            .is_some_and(|items| !items.is_empty())
        || descriptor["mode"] != "WORK"
    {
        return Err("Portable candidate evidence currently requires a leaf WORK task".into());
    }
    let submission = &task["coordination"]["submission"];
    let snapshot = runner::snapshot(root, &task, registry, work)?;
    coordination::check_evidence(root, work, registry, &task, &snapshot)?;
    let source_sha256 = value(submission, "source_sha256")?;
    if snapshot["status"] != "PASS" || snapshot["sha256"] != source_sha256 {
        return Err("Candidate must match a current submitted task snapshot".into());
    }
    let rows = snapshot["repositories"]
        .as_array()
        .ok_or("No captured repositories")?;
    if rows.iter().any(|row| {
        row["repo"] != repo
            && row["changed_paths"]
                .as_array()
                .is_none_or(|paths| !paths.is_empty())
    }) {
        return Err("Portable candidates cannot carry writes for multiple repositories".into());
    }
    let row = rows
        .iter()
        .find(|row| row["repo"] == repo)
        .ok_or("Primary worktree has no captured inputs")?;
    let binding = workspace::resolve_task(root, work, repo, registry, &task)?;
    let mut files = BTreeMap::new();
    for (relative, fingerprint) in row["inputs"]
        .as_object()
        .ok_or("No captured candidate file evidence")?
    {
        let path = workspace::scoped_path(&binding.path, relative)?;
        let sha256 = fingerprint["sha256"].as_str().map(str::to_owned);
        let executable = sha256.is_some() && index_mode(fingerprint, &path)?;
        files.insert(relative.clone(), FileEvidence { sha256, executable });
    }
    let (authority_commit, clean) = policy::checkout(root)?;
    if !clean {
        return Err("Portable evidence requires an approved clean authority checkout".into());
    }
    let dependencies = task["coordination"]["dependencies"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let dependencies = serde_json::from_value(Value::Array(dependencies))
        .map_err(|_| "Cannot encode portable dependency evidence")?;
    let selected = authority::task_snapshot(
        root,
        registry,
        descriptor,
        (
            workspace::configuration(registry, "context_bytes", 32_768, 2 * 1024 * 1024)?,
            workspace::configuration(registry, "context_files", 32, 128)?,
        ),
    )?;
    if selected.digest != task["policy_sha256"] {
        return Err("Selected authority sources changed during evidence export".into());
    }
    let mut authority_sources: BTreeMap<_, _> = selected
        .included
        .into_iter()
        .map(|source| (source.path, source.sha256))
        .collect();
    for field in ["plan", "spec"] {
        let relative = value(descriptor, field)?;
        let bytes = bounded_file(&policy::safe_path(root, relative)?)?;
        authority_sources.insert(relative.into(), docs::sha256(&bytes));
    }
    let bundle = Bundle {
        schema: 1,
        task: id.into(),
        repository: repo.into(),
        base_commit: value(row, "commit")?.into(),
        authority_commit,
        registry_sha256: registry_digest(root)?,
        authority_sha256: value(&task, "policy_sha256")?.into(),
        authority_sources,
        descriptor_sha256: value(&task, "descriptor_sha256")?.into(),
        approval_epoch: task["approval_epoch"]
            .as_u64()
            .ok_or("Invalid approval epoch")?,
        submission_sha256: docs::sha256(
            &serde_json::to_vec(submission).map_err(|_| "Cannot encode submission")?,
        ),
        source_sha256: source_sha256.into(),
        write_scope: strings(&descriptor["write_scope"][repo])?,
        verification_scope: descriptor["verification_scope"]
            .get(repo)
            .map(strings)
            .transpose()?
            .unwrap_or_default(),
        changed: strings(&row["changed_paths"])?,
        files,
        commands: strings(&descriptor["commands"])?
            .into_iter()
            .filter(|id| registry["control_plane"]["commands"][id]["mode"] == "VERIFY")
            .collect(),
        budget: ["timeout_ms", "log_bytes"]
            .into_iter()
            .filter_map(|key| {
                descriptor["budget"][key]
                    .as_u64()
                    .map(|value| (key.into(), value))
            })
            .collect(),
        dependencies,
    };
    validate(&bundle)?;
    let current = tasks::load_current(root, id)?;
    let observed = runner::snapshot(root, &current, registry, work)?;
    coordination::check_evidence(root, work, registry, &current, &observed)?;
    if observed["status"] != "PASS"
        || observed["sha256"] != source_sha256
        || observed["violations"] != snapshot["violations"]
        || current["revision"] != task["revision"]
        || current["coordination"]["submission"] != *submission
        || policy::checkout(root)? != (bundle.authority_commit.clone(), true)
    {
        return Err(
            "Candidate inputs or authority changed while preparing portable evidence".into(),
        );
    }
    let bytes = serde_json::to_vec(&bundle).map_err(|_| "Cannot encode portable evidence")?;
    if bytes.len() as u64 > MAX_FILE {
        return Err("Portable evidence exceeds its byte limit".into());
    }
    let path = cli_file(output)?;
    if path.starts_with(root) || path.starts_with(&binding.path) {
        return Err(
            "Portable evidence output must be outside the authority and candidate checkouts".into(),
        );
    }
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| "Portable evidence destination already exists or is not writable")?;
    out.write_all(&bytes)
        .and_then(|_| out.sync_all())
        .map_err(|_| "Cannot persist portable evidence")?;
    Ok(
        json!({"status":"PASS","state":"EXPORTED","schema":1,"task":id,"bundle_sha256":docs::sha256(&bytes)}),
    )
}

fn observe(root: &Path, bundle: &Bundle, checkout: &Path, commit: &str) -> Result<(), String> {
    let changed = policy::git(
        checkout,
        &[
            "diff",
            "--name-only",
            "--no-renames",
            "--no-ext-diff",
            "--no-textconv",
            "-z",
            &bundle.base_commit,
            commit,
            "--",
        ],
    )?;
    let actual: BTreeSet<_> = changed.split_terminator('\0').map(str::to_owned).collect();
    if actual != bundle.changed {
        return Err("Candidate changed paths differ from the reviewed source snapshot".into());
    }
    for (relative, expected) in &bundle.files {
        let entry = tree_entry(checkout, commit, relative)?;
        let file = workspace::scoped_path(checkout, relative)?;
        let actual = if let Some((executable, oid)) = entry {
            let blob = runner::committed_blob(root, checkout, &oid, MAX_FILE as usize)?;
            let digest = docs::sha256(&blob);
            if docs::sha256(&bounded_file(&file)?) != digest {
                return Err(format!(
                    "Checkout bytes differ from the committed blob for {relative}"
                ));
            }
            FileEvidence {
                sha256: Some(digest),
                executable,
            }
        } else {
            if file.symlink_metadata().is_ok() {
                return Err("Candidate contains an untracked file in its sealed scope".into());
            }
            FileEvidence {
                sha256: None,
                executable: false,
            }
        };
        if actual != *expected {
            return Err(format!("Candidate content or mode is stale for {relative}"));
        }
    }
    Ok(())
}

fn checks(
    root: &Path,
    bundle: &Bundle,
    registry: &Value,
    checkout: &Path,
    commit: &str,
) -> Result<Value, String> {
    let mut results = Vec::new();
    let mut observed_inputs = BTreeMap::<String, String>::new();
    let approved = json!({
        "commands":bundle.commands,
        "verification_scope":BTreeMap::from([(&bundle.repository, &bundle.verification_scope)]),
        "budget":bundle.budget
    });
    for id in &bundle.commands {
        let step = match runner::evidence_step_plan(&approved, registry, id) {
            Ok(step) if step["repository"] == bundle.repository => step,
            _ => {
                return Ok(
                    json!({"status":"BLOCKED","reason":"Required check does not match the trusted runner profile","check":id}),
                );
            }
        };
        let paths: Vec<_> = step["paths"]
            .as_array()
            .ok_or("Missing required verification paths")?
            .iter()
            .map(|path| path.as_str().ok_or("Invalid verification path"))
            .collect::<Result<_, _>>()?;
        let timeout =
            Duration::from_millis(step["timeout_ms"].as_u64().ok_or("Invalid check timeout")?);
        let max_output = step["log_bytes"]
            .as_u64()
            .ok_or("Invalid check output limit")? as usize;
        let started = Instant::now();
        let passed = match (step["program"].as_str(), &step["argv"]) {
            (Some("git"), argv) if *argv == json!(["diff", "--check"]) => {
                runner::committed_diff_check(
                    root,
                    checkout,
                    (&bundle.base_commit, commit),
                    &paths,
                    timeout,
                    max_output,
                )?
            }
            (Some("git"), argv) if *argv == json!(["rev-parse", "HEAD"]) => Some(
                policy::git(checkout, &["rev-parse", "--verify", "HEAD^{commit}"])?.trim()
                    == commit,
            ),
            (Some("check-json" | "check-utf8"), _) => {
                let mut passed = true;
                let mut checked_inputs = BTreeMap::new();
                for relative in paths {
                    let input = bounded_file(&workspace::scoped_path(checkout, relative)?)?;
                    let digest = docs::sha256(&input);
                    if bundle.files[relative].sha256.as_deref() != Some(&digest) {
                        return Err("Required check input changed before execution".into());
                    }
                    passed &= if step["program"] == "check-json" {
                        serde_json::from_slice::<Value>(&input).is_ok()
                    } else {
                        std::str::from_utf8(&input).is_ok()
                    };
                    checked_inputs.insert(relative.into(), digest);
                    if started.elapsed() >= timeout {
                        break;
                    }
                }
                let bytes = serde_json::to_vec(&checked_inputs)
                    .map_err(|_| "Cannot encode verification inputs")?
                    .len();
                if started.elapsed() >= timeout || bytes > max_output {
                    None
                } else {
                    observed_inputs.extend(checked_inputs);
                    Some(passed)
                }
            }
            _ => None,
        };
        let status = match passed {
            Some(true) => "PASS",
            Some(false) => "FAIL",
            None => "BLOCKED",
        };
        results.push(json!({"id":id,"status":status,"exit_code":passed.map(|value| i32::from(!value)),"producer":"cirnetrules-ci-controlled"}));
        if status != "PASS" {
            return Ok(json!({"status":status,"checks":results}));
        }
    }
    for dependency in &bundle.dependencies {
        for input in &dependency.selected_inputs {
            if input.repository != bundle.repository
                || input.sha256.is_none()
                || observed_inputs.get(&input.path) != input.sha256.as_ref()
                || !dependency
                    .outputs
                    .iter()
                    .any(|output| output.sha256 == input.sha256)
            {
                return Ok(
                    json!({"status":"BLOCKED","reason":"Producer output was not observed by a required consumer check","dependency":dependency.task_id}),
                );
            }
        }
    }
    Ok(json!({"status":"PASS","checks":results,"observed_inputs":observed_inputs}))
}

fn gate(
    root: &Path,
    registry: &Value,
    bundle: &Bundle,
    digest: &str,
    expected: &str,
    authority_commit: &str,
    checkout: &Path,
) -> Result<Value, String> {
    let (authority, clean) = policy::checkout(root)?;
    if !hash(expected, 64) || expected != digest {
        return Ok(
            json!({"status":"BLOCKED","reason":"Bundle does not match the separately reviewed digest"}),
        );
    }
    if !clean
        || !git_id(authority_commit)
        || authority != authority_commit
        || bundle.authority_commit != authority
        || registry_digest(root)? != bundle.registry_sha256
        || !policy::governance_origin_matches(root)?
    {
        return Ok(
            json!({"status":"BLOCKED","reason":"Trusted authority commit or registry bytes do not match"}),
        );
    }
    for (relative, expected) in &bundle.authority_sources {
        let path = policy::safe_path(root, relative)?;
        if docs::sha256(&bounded_file(&path)?) != *expected
            || policy::git(
                root,
                &[
                    "--literal-pathspecs",
                    "ls-files",
                    "--error-unmatch",
                    "--",
                    relative,
                ],
            )
            .is_err()
        {
            return Ok(
                json!({"status":"BLOCKED","reason":"Reviewed authority source bytes or tracking changed"}),
            );
        }
    }
    if registry["repositories"].get(&bundle.repository).is_none() {
        return Err("Bundle repository is not registered".into());
    }
    let checkout = workspace::workspace_path(root, Some(checkout))?;
    if checkout == root {
        return Err("Candidate and trusted governance checkouts must be separate".into());
    }
    let origin = policy::git(
        &checkout,
        &[
            "config",
            "--local",
            "--no-includes",
            "--get-all",
            "remote.origin.url",
        ],
    )?;
    let expected_origin = format!("https://github.com/cirnetlab/{}", bundle.repository);
    if origin.trim().trim_end_matches(".git") != expected_origin {
        return Err("Candidate origin does not match the reviewed repository".into());
    }
    let actual_root = policy::git(&checkout, &["rev-parse", "--show-toplevel"])?;
    if Path::new(actual_root.trim()).canonicalize().ok().as_ref() != Some(&checkout) {
        return Err("Candidate must be the exact checkout root".into());
    }
    let commit = policy::git(&checkout, &["rev-parse", "--verify", "HEAD^{commit}"])?
        .trim()
        .to_owned();
    if !git_id(&commit)
        || !runner::guarded_git_clean(root, &checkout)?
        || policy::git(
            &checkout,
            &["merge-base", "--is-ancestor", &bundle.base_commit, &commit],
        )
        .is_err()
    {
        return Ok(
            json!({"status":"BLOCKED","reason":"Candidate must be clean and descend from the sealed base"}),
        );
    }
    observe(root, bundle, &checkout, &commit)?;
    let mut report = checks(root, bundle, registry, &checkout, &commit)?;
    observe(root, bundle, &checkout, &commit)?;
    if policy::git(&checkout, &["rev-parse", "--verify", "HEAD^{commit}"])?.trim() != commit
        || !runner::guarded_git_clean(root, &checkout)?
        || policy::checkout(root)? != (authority.clone(), true)
    {
        return Err("Authority or candidate changed during verification".into());
    }
    report["schema"] = json!(1);
    report["candidate_commit"] = json!(commit);
    report["base_commit"] = json!(bundle.base_commit);
    report["authority_commit"] = json!(authority);
    report["task"] = json!(bundle.task);
    report["bundle_sha256"] = json!(digest);
    report["source_sha256"] = json!(bundle.source_sha256);
    report["trust"] = json!("operator-pinned approval digest and governance commit");
    report["evidence_limits"] = json!(
        "Controlled software checks only. No authenticated acceptance, release or hardware evidence."
    );
    report["receipt_sha256"] = json!(docs::sha256(
        &serde_json::to_vec(&report).map_err(|_| "Cannot encode gate receipt")?
    ));
    Ok(report)
}

pub(crate) fn run(
    root: &Path,
    registry: &Value,
    action: &str,
    args: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let allowed: &[&str] = match action {
        "evidence-export" => &["task", "output", "workspace"],
        "evidence-inspect" => &["bundle"],
        "verify" => &["bundle", "target", "expected-digest", "authority-commit"],
        _ => return Err("Unknown portable evidence action".into()),
    };
    if args.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err("Unexpected portable evidence option".into());
    }
    let required = |key| {
        args.get(key)
            .map(String::as_str)
            .ok_or_else(|| format!("Missing --{key}"))
    };
    let path = |key| required(key).map(Path::new);
    if action == "evidence-export" {
        return export(
            root,
            registry,
            args.get("workspace").map(Path::new),
            required("task")?,
            path("output")?,
        );
    }
    let (bundle, digest) = load(path("bundle")?)?;
    if action == "evidence-inspect" {
        return Ok(
            json!({"status":"PASS","state":"INTEGRITY_CHECKED","bundle_sha256":digest,"trust":"UNTRUSTED","bundle":bundle}),
        );
    }
    gate(
        root,
        registry,
        &bundle,
        &digest,
        required("expected-digest")?,
        required("authority-commit")?,
        path("target")?,
    )
}

#[cfg(test)]
#[path = "ci/tests.rs"]
mod tests;
