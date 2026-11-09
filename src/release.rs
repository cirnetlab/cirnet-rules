use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
    process::Command,
};

mod inventory;
#[cfg(test)]
mod tests;

const RELEASE_SCHEMA: &str = include_str!("../en/schemas/native-release.schema.json");
const MATRIX_SCHEMA: &str = include_str!("../en/schemas/compatibility-matrix.schema.json");
const EVIDENCE_SCHEMA: &str = include_str!("../en/schemas/release-evidence.schema.json");
const INPUT_LIMIT: u64 = 16 * 1024 * 1024;
const ARTIFACT_LIMIT: u64 = 4 * 1024 * 1024 * 1024;
fn components(registry: &Value) -> GateResult<&Map<String, Value>> {
    registry["release_policy"]["components"]
        .as_object()
        .filter(|items| !items.is_empty() && items.len() <= 64)
        .ok_or_else(|| failed("Release component profile is missing or empty"))
}

fn relative_input(path: &str) -> bool {
    !path.contains(['\\', ':'])
        && path
            .split('/')
            .all(|part| !part.is_empty() && !matches!(part, "." | ".."))
        && !path.is_empty()
}

fn authority<'a>(registry: &'a Value, kind: &str) -> GateResult<(&'a str, &'a str)> {
    let item = &registry["authority"][kind];
    let repo = item["repository"]
        .as_str()
        .filter(|repo| portable_filename(repo))
        .ok_or_else(|| failed("Release authority repository is absent"))?;
    let path = item["path"]
        .as_str()
        .filter(|path| relative_input(path))
        .ok_or_else(|| failed("Release authority input path is unsafe"))?;
    Ok((repo, path))
}

pub fn release_scope_matches(scope: &Value, registry: &Value) -> bool {
    components(registry).is_ok_and(|items| {
        items.keys().all(|repo| {
            inventory::release_inputs(repo, registry).is_ok_and(|inputs| {
                scope
                    .get(repo)
                    .and_then(Value::as_array)
                    .is_some_and(|granted| {
                        inputs
                            .iter()
                            .all(|path| granted.iter().any(|value| value == path))
                    })
            })
        })
    })
}
const DOMAINS: [&str; 12] = [
    "http",
    "realtime",
    "hardware_rpc",
    "simulator_ipc",
    "circuit_graph",
    "execution_result",
    "capability",
    "capture",
    "fpga_logical",
    "fpga_register_map",
    "data_storage",
    "release_manifest",
];

struct Gate<'a> {
    id: &'a str,
    repo: &'a str,
    kind: &'a str,
}

fn gates(registry: &Value) -> GateResult<Vec<Gate<'_>>> {
    const REQUIRED: &[(&str, &str)] = &[
        ("source_provenance", "review"),
        ("contract_conformance", "contract_test"),
        ("hardware_safe_stop", "native_integration"),
        ("simulator_worker", "protocol_test"),
        ("browser_native", "browser_e2e"),
        ("fpga_scaffold", "rtl_test"),
        ("clean_windows", "clean_install"),
        ("rollback", "rollback_test"),
        ("legacy_rejection", "native_integration"),
    ];
    let source = registry["release_policy"]["required_gates"]
        .as_object()
        .ok_or_else(|| failed("Release gate policy is missing"))?;
    if source.len() != REQUIRED.len() {
        return Err(failed(
            "Release gate policy does not preserve all mandatory gates",
        ));
    }
    REQUIRED
        .iter()
        .map(|&(id, kind)| {
            let row = source
                .get(id)
                .ok_or_else(|| failed("A mandatory release gate is missing"))?;
            let repo = row["repository"]
                .as_str()
                .filter(|repo| components(registry).is_ok_and(|items| items.contains_key(*repo)))
                .ok_or_else(|| failed("Release gate owner is not in the selected profile"))?;
            if row["evidence_type"] != kind {
                return Err(failed("Release gate evidence kind differs from policy"));
            }
            Ok(Gate { id, repo, kind })
        })
        .collect()
}

#[derive(Debug)]
enum GateError {
    Blocked(String),
    Failed(String),
}

type GateResult<T> = Result<T, GateError>;

fn blocked(message: impl Into<String>) -> GateError {
    GateError::Blocked(message.into())
}

fn failed(message: impl Into<String>) -> GateError {
    GateError::Failed(message.into())
}

fn report(check: &str, result: GateResult<Value>) -> Value {
    match result {
        Ok(value) => value,
        Err(GateError::Blocked(reason)) => {
            json!({"check": check, "status": "BLOCKED", "reason": reason})
        }
        Err(GateError::Failed(reason)) => {
            json!({"check": check, "status": "FAIL", "reason": reason})
        }
    }
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha(value: &Value, length: usize) -> bool {
    value.as_str().is_some_and(|value| hex(value, length))
}

fn same_keys(value: &Value, required: &[&str]) -> bool {
    value.as_object().is_some_and(|map| {
        map.len() == required.len() && required.iter().all(|key| map.contains_key(*key))
    })
}

fn version(value: &str) -> bool {
    let mut components = value.split('.');
    (0..3).all(|_| {
        components.next().is_some_and(|part| {
            !part.is_empty()
                && (part == "0" || !part.starts_with('0'))
                && part.bytes().all(|byte| byte.is_ascii_digit())
        })
    }) && components.next().is_none()
}

fn text(value: &Value) -> bool {
    value
        .as_str()
        .is_some_and(|value| !value.trim().is_empty() && value.len() <= 256)
}

fn portable_filename(value: &str) -> bool {
    if value.len() > 160
        || value.ends_with('.')
        || !value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
    {
        return false;
    }
    let stem = value
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        && !["COM", "LPT"].iter().any(|prefix| {
            stem.strip_prefix(prefix)
                .is_some_and(|suffix| suffix.len() == 1 && suffix.as_bytes()[0].is_ascii_digit())
        })
}

fn workspace(root: &Path) -> GateResult<PathBuf> {
    let canonical = root
        .canonicalize()
        .map_err(|_| blocked("product workspace is unavailable"))?;
    if !canonical.is_dir() {
        return Err(blocked("product workspace is not a directory"));
    }
    Ok(canonical)
}

fn safe_child(root: &Path, relative: &Path) -> GateResult<PathBuf> {
    if relative
        .components()
        .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(failed("unsafe relative evidence or source path"));
    }
    let requested = root.join(relative);
    let resolved = requested.canonicalize().map_err(|_| {
        blocked(format!(
            "missing or unreadable input {}",
            relative.display()
        ))
    })?;
    if resolved != requested {
        return Err(failed(format!(
            "redirected source or evidence path {}",
            relative.display()
        )));
    }
    Ok(resolved)
}

fn read_input(root: &Path, relative: &str) -> GateResult<Vec<u8>> {
    let path = safe_child(root, Path::new(relative))?;
    let file = File::open(path).map_err(|_| blocked(format!("cannot open {relative}")))?;
    let mut data = Vec::new();
    file.take(INPUT_LIMIT + 1)
        .read_to_end(&mut data)
        .map_err(|_| blocked(format!("cannot read {relative}")))?;
    if data.len() as u64 > INPUT_LIMIT {
        return Err(failed(format!("input exceeds read limit: {relative}")));
    }
    Ok(data)
}

fn read_json(data: &[u8], label: &str) -> GateResult<Value> {
    // serde_json's Value accepts duplicate map keys. Refuse ambiguity in release
    // inputs before validating the parsed structure.
    let normalized: serde_yaml::Value = serde_yaml::from_slice(data)
        .map_err(|_| failed(format!("invalid or ambiguous JSON: {label}")))?;
    let parsed: Value =
        serde_json::from_slice(data).map_err(|_| failed(format!("invalid JSON: {label}")))?;
    if serde_json::to_value(normalized).ok().as_ref() != Some(&parsed) {
        return Err(failed(format!("ambiguous JSON input: {label}")));
    }
    Ok(parsed)
}

fn check_schema(value: &Value, schema: &str, label: &str) -> GateResult<()> {
    let schema: Value =
        serde_json::from_str(schema).map_err(|_| failed("invalid embedded release schema"))?;
    let validator = jsonschema::validator_for(&schema)
        .map_err(|_| failed("cannot compile embedded release schema"))?;
    if !validator.is_valid(value) {
        return Err(failed(format!("{label} does not match its machine schema")));
    }
    Ok(())
}

fn checked_bytes(
    root: &Path,
    relative: &str,
    expected_sha: &str,
    size: Option<u64>,
) -> GateResult<()> {
    let path = safe_child(root, Path::new(relative))?;
    let mut file =
        File::open(path).map_err(|_| blocked("cannot read selected artifact or result"))?;
    let metadata = file
        .metadata()
        .map_err(|_| blocked("cannot inspect selected artifact or result"))?;
    if !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > ARTIFACT_LIMIT
        || size.is_some_and(|value| value != metadata.len())
    {
        return Err(failed(format!("wrong size or file kind: {relative}")));
    }
    let mut hasher = Sha256::new();
    std::io::copy(&mut file.by_ref().take(ARTIFACT_LIMIT + 1), &mut hasher)
        .map_err(|_| blocked("cannot hash selected artifact or result"))?;
    if format!("{:x}", hasher.finalize()) != expected_sha {
        return Err(failed(format!("SHA-256 mismatch: {relative}")));
    }
    Ok(())
}

fn git(repo: &Path, args: &[&str]) -> GateResult<Vec<u8>> {
    let caller = std::env::current_dir().unwrap_or_else(|_| repo.to_path_buf());
    let git = crate::runner::git_executable(repo, &caller).map_err(blocked)?;
    let mut command = Command::new(git);
    command
        .current_dir(repo)
        .arg("--no-replace-objects")
        .arg("--no-lazy-fetch")
        .arg("--no-optional-locks")
        .args(["-c", "core.fsmonitor=false"])
        .args(args);
    for (key, _) in std::env::vars_os() {
        if key
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("GIT_")
        {
            command.env_remove(key);
        }
    }
    command.env("GIT_OPTIONAL_LOCKS", "0");
    let output = command
        .output()
        .map_err(|_| blocked("Git is unavailable"))?;
    if !output.status.success() {
        let name = repo
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("checkout");
        return Err(blocked(format!(
            "Git {} could not resolve required local data in {name}",
            args[0]
        )));
    }
    if output.stdout.len() as u64 > INPUT_LIMIT {
        return Err(failed("Git output exceeds read limit"));
    }
    Ok(output.stdout)
}

fn git_text(repo: &Path, args: &[&str]) -> GateResult<String> {
    String::from_utf8(git(repo, args)?)
        .map(|value| value.trim().to_owned())
        .map_err(|_| failed("Git identity is not UTF-8"))
}

fn git_status(repo: &Path) -> GateResult<Vec<u8>> {
    let keys = git(repo, &["config", "--list", "--name-only", "--null"])?;
    let keys = std::str::from_utf8(&keys).map_err(|_| failed("Git configuration is not UTF-8"))?;
    let mut overrides = Vec::new();
    for key in keys.split('\0').collect::<BTreeSet<_>>() {
        if !key.starts_with("filter.") {
            continue;
        }
        match key.rsplit('.').next() {
            Some("clean" | "process" | "smudge") => overrides.push(format!("{key}=")),
            Some("required") => overrides.push(format!("{key}=false")),
            _ => {}
        }
    }
    let mut args = Vec::new();
    for setting in &overrides {
        args.extend(["-c", setting]);
    }
    // Submodule checkouts can have independent configuration and filters. A
    // future repo with gitlinks needs a separately reviewed drift policy.
    let index = git(repo, &["ls-files", "--stage", "-z"])?;
    if index
        .split(|byte| *byte == 0)
        .any(|entry| entry.starts_with(b"160000 "))
    {
        return Err(blocked(
            "drift verification for submodule worktrees is unreviewed",
        ));
    }
    args.extend([
        "status",
        "--porcelain=v1",
        "--untracked-files=normal",
        "--no-renames",
    ]);
    git(repo, &args)
}

pub(crate) fn worktree_is_clean(repo: &Path) -> Result<bool, String> {
    git_status(repo)
        .map(|status| status.is_empty())
        .map_err(|error| match error {
            GateError::Blocked(reason) | GateError::Failed(reason) => reason,
        })
}

fn normalized_remote(value: &str, expected: &str) -> bool {
    fn network_endpoint(address: &str) -> Option<(&str, &str)> {
        let address = address.trim();
        let (host, path) = if let Some(ssh) = address.strip_prefix("git@") {
            ssh.split_once(':')?
        } else {
            address
                .strip_prefix("https://")
                .or_else(|| address.strip_prefix("ssh://git@"))?
                .split_once('/')?
        };
        let path = path.strip_suffix('/').unwrap_or(path);
        let path = path.strip_suffix(".git").unwrap_or(path);
        if host.is_empty()
            || !host
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-".contains(&b))
            || !relative_input(path)
            || path.contains(['?', '#', '@'])
            || path.chars().any(char::is_whitespace)
        {
            return None;
        }
        Some((host, path))
    }
    match (network_endpoint(value), network_endpoint(expected)) {
        (Some(actual), Some(configured)) => actual == configured,
        (None, None) => !expected.trim().is_empty() && value.trim() == expected.trim(),
        _ => false,
    }
}

fn checkout(root: &Path, name: &str, registry: &Value) -> GateResult<PathBuf> {
    let repo = if let Some(governance) = registry["_cirnetrules_release_governance"].as_str() {
        crate::workspace::resolve(Path::new(governance), Some(root), name, registry)
            .map(|binding| binding.path)
            .map_err(blocked)?
    } else {
        safe_child(root, Path::new(name))?
    };
    let top = git_text(&repo, &["rev-parse", "--show-toplevel"])?;
    if Path::new(&top).canonicalize().ok().as_ref() != Some(&repo) {
        return Err(failed(format!("{name} is not the checkout root")));
    }
    if let Some(expected) = registry["repositories"][name]["remote"].as_str() {
        let actual = git_text(
            &repo,
            &[
                "config",
                "--local",
                "--no-includes",
                "--get",
                "remote.origin.url",
            ],
        )?;
        if !normalized_remote(&actual, expected) {
            return Err(failed(format!(
                "Git origin differs from configured provenance for {name}"
            )));
        }
    }
    Ok(repo)
}

fn baseline(registry: &Value) -> GateResult<(&str, &str)> {
    let train = registry["baseline"]["release_train"]
        .as_str()
        .filter(|value| version(value))
        .ok_or_else(|| failed("Invalid configured release train"))?;
    let family = registry["baseline"]["family"]
        .as_str()
        .filter(|value| value.split('.').count() == 2 && version(&format!("{value}.0")))
        .ok_or_else(|| failed("Invalid configured string contract family"))?;
    Ok((train, family))
}

fn validate_registry(registry: &Value) -> GateResult<()> {
    baseline(registry)?;
    let (owner, _) = authority(registry, "release_set")?;
    let (contracts, _) = authority(registry, "contracts")?;
    let items = components(registry)?;
    if !matches!(
        registry["baseline"]["activation"].as_str(),
        Some("NOT_ACTIVATED" | "ACTIVATED")
    ) || !items.contains_key(owner)
        || !items.contains_key(contracts)
        || items
            .keys()
            .any(|name| !portable_filename(name) || registry["repositories"].get(name).is_none())
        || !registry["release_policy"]["matrix_path"]
            .as_str()
            .is_some_and(relative_input)
    {
        return Err(failed(
            "Release profile, provenance, or activation is invalid",
        ));
    }
    for name in items.keys() {
        inventory::release_inputs(name, registry)?;
    }
    Ok(())
}

pub fn validate_profile(registry: &Value) -> Result<(), String> {
    (|| {
        validate_registry(registry)?;
        gates(registry)?;
        Ok(())
    })()
    .map_err(|error| match error {
        GateError::Failed(reason) | GateError::Blocked(reason) => reason,
    })
}

pub fn check_drift(root: &Path, registry: &Value) -> Result<Value, String> {
    Ok(report(
        "check-drift",
        (|| {
            validate_registry(registry)?;
            let root = workspace(root)?;
            let mut changes = Vec::new();
            let mut observations = Map::new();
            for name in components(registry)?.keys().map(String::as_str) {
                let repo = checkout(&root, name, registry)?;
                let observed = &registry["repositories"][name]["snapshot"];
                if !sha(&observed["commit"], 40) || !sha(&observed["tree"], 40) {
                    return Err(blocked(format!(
                        "{name} has no complete reviewed Git snapshot"
                    )));
                }
                let commit = git_text(&repo, &["rev-parse", "HEAD"])?;
                let tree = git_text(&repo, &["rev-parse", "HEAD^{tree}"])?;
                let dirty = !git_status(&repo)?.is_empty();
                if observed["commit"] != commit || observed["tree"] != tree || dirty {
                    changes.push(name);
                }
                observations.insert(
                    name.to_owned(),
                    json!({"commit": commit, "tree": tree, "dirty": dirty}),
                );
            }
            Ok(
                json!({"check": "check-drift", "status": if changes.is_empty() {"PASS"} else {"FAIL"},
            "changed_repositories": changes, "observed": observations}),
            )
        })(),
    ))
}

pub fn check_versions(root: &Path, registry: &Value) -> Result<Value, String> {
    Ok(report("check-versions", inventory::check(root, registry)))
}

fn selection_digest(manifest: &Value) -> GateResult<String> {
    // The approved combination includes producer metadata, capabilities,
    // toolchains, the lockfile digests, and every reviewed contract identity.
    let mut selection = Map::new();
    for key in [
        "contracts",
        "family",
        "releaseTrain",
        "repositories",
        "rollback",
    ] {
        let value = manifest
            .get(key)
            .ok_or_else(|| failed("incomplete release selection"))?;
        selection.insert(key.to_owned(), value.clone());
    }
    canonical_digest(&Value::Object(selection))
}

fn canonical_digest(value: &Value) -> GateResult<String> {
    serde_json::to_vec(&sorted(value))
        .map(|bytes| digest(&bytes))
        .map_err(|_| failed("cannot encode canonical release JSON"))
}

fn sorted(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let entries = map.iter().collect::<std::collections::BTreeMap<_, _>>();
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key.clone(), sorted(value)))
                    .collect(),
            )
        }
        Value::Array(values) => Value::Array(values.iter().map(sorted).collect()),
        value => value.clone(),
    }
}

fn check_matrix(matrix: &Value, manifest: &Value, train: &str, family: &str) -> GateResult<String> {
    check_schema(matrix, MATRIX_SCHEMA, "compatibility matrix")?;
    if !same_keys(
        matrix,
        &[
            "schemaVersion",
            "releaseTrain",
            "family",
            "allowedSelections",
        ],
    ) || matrix["schemaVersion"] != 1
        || matrix["releaseTrain"] != train
        || matrix["family"] != family
        || manifest["releaseTrain"] != train
        || manifest["family"] != family
    {
        return Err(failed(
            "release train or compatibility family differs from the approved baseline",
        ));
    }
    let selection = selection_digest(manifest)?;
    let allowed = matrix["allowedSelections"]
        .as_array()
        .ok_or_else(|| failed("compatibility matrix has no allowlist"))?;
    let unique = allowed
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    if allowed.is_empty()
        || allowed.len() > 32
        || unique.len() != allowed.len()
        || !allowed.iter().all(|value| sha(value, 64))
        || !unique.contains(selection.as_str())
    {
        return Err(failed(
            "exact component and contract combination is absent from the reviewed compatibility matrix",
        ));
    }
    Ok(selection)
}

fn check_policy(registry: &Value) -> GateResult<&str> {
    let policy = &registry["release_policy"];
    let approved = policy["approved_matrix_sha256"]
        .as_str()
        .filter(|value| hex(value, 64))
        .ok_or_else(|| blocked("no approved exact compatibility matrix digest is recorded"))?;
    let domains = policy["required_domains"]
        .as_array()
        .ok_or_else(|| failed("release policy requires domain coverage"))?;
    let names = domains
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    if domains.len() != DOMAINS.len() || names != BTreeSet::from(DOMAINS) {
        return Err(failed(
            "release policy omits a mandatory contract domain or duplicates an identity",
        ));
    }
    gates(registry)?;
    let owners = policy["domain_owners"]
        .as_object()
        .ok_or_else(|| failed("Configured contract domain owners are missing"))?;
    if owners.len() != DOMAINS.len()
        || DOMAINS.iter().any(|name| {
            !owners
                .get(*name)
                .and_then(Value::as_str)
                .is_some_and(|repo| {
                    components(registry).is_ok_and(|items| items.contains_key(repo))
                })
        })
    {
        return Err(failed(
            "Configured contract domains do not have exact reviewed owners",
        ));
    }
    Ok(approved)
}

fn check_capabilities(capabilities: &Value) -> GateResult<()> {
    let map = capabilities
        .as_object()
        .filter(|map| !map.is_empty() && map.len() <= 32)
        .ok_or_else(|| failed("component has no capability scope"))?;
    for (name, capability) in map {
        if !portable_filename(name)
            || !same_keys(capability, &["implemented", "validated", "available"])
        {
            return Err(failed("invalid capability declaration"));
        }
        let flags = (
            capability["implemented"].as_bool(),
            capability["validated"].as_bool(),
            capability["available"].as_bool(),
        );
        match flags {
            (Some(implemented), Some(validated), Some(available))
                if (!available || validated) && (!validated || implemented) =>
            {
                if available
                    && ![
                        "simulation",
                        "contracts",
                        "local-rpc",
                        "web-workbench",
                        "capture",
                        "logical-scaffold",
                        "virtual-station",
                    ]
                    .contains(&name.as_str())
                {
                    return Err(failed(
                        "available capability is outside the reviewed software-only cutover vocabulary",
                    ));
                }
            }
            _ => {
                return Err(failed(
                    "an available capability must be implemented and validated",
                ));
            }
        }
    }
    Ok(())
}

fn check_components(
    manifest: &Value,
    registry: &Value,
    train: &str,
    family: &str,
) -> GateResult<()> {
    let repositories = &manifest["repositories"];
    let configured = components(registry)?;
    let expected: Vec<_> = configured.keys().map(String::as_str).collect();
    let (owner, _) = authority(registry, "release_set")?;
    if !same_keys(repositories, &expected) {
        return Err(failed(
            "Native release must select the exact configured repository set",
        ));
    }
    for name in components(registry)?.keys().map(String::as_str) {
        let entry = &repositories[name];
        if !same_keys(
            entry,
            &[
                "version",
                "tag",
                "commit",
                "target",
                "toolchain",
                "lockSha256",
                "capabilities",
                "artifacts",
            ],
        ) || entry["version"] != train
            || entry["tag"] != format!("v{train}")
            || !sha(&entry["commit"], 40)
            || !sha(&entry["lockSha256"], 64)
            || !entry["target"].as_str().is_some_and(portable_filename)
            || !entry["toolchain"].as_object().is_some_and(|map| {
                !map.is_empty()
                    && map.len() <= 12
                    && map
                        .iter()
                        .all(|(key, value)| portable_filename(key) && text(value))
            })
        {
            return Err(failed(format!(
                "{name} has a mixed train, missing provenance or malformed metadata"
            )));
        }
        let component = &configured[name];
        let target = if component["target"] == "release-owner" {
            repositories[owner]["target"].clone()
        } else {
            component["target"].clone()
        };
        if entry["target"] != target
            || (name == owner
                && !target
                    .as_str()
                    .unwrap_or_default()
                    .ends_with("-windows-msvc"))
        {
            return Err(failed(format!(
                "{name} target is outside the selected Windows MSVC / static / source scope"
            )));
        }
        check_capabilities(&entry["capabilities"])?;
        let artifacts = entry["artifacts"]
            .as_array()
            .filter(|array| !array.is_empty() && array.len() <= 16)
            .ok_or_else(|| failed(format!("{name} has no selected artifact")))?;
        let mut names = BTreeSet::new();
        let mut includes_required_kind = false;
        for artifact in artifacts {
            let filename = artifact["filename"].as_str().unwrap_or_default();
            let kind = artifact["kind"].as_str().unwrap_or_default();
            let versioned_name = filename
                .strip_prefix(&format!("{name}-{train}"))
                .is_some_and(|tail| {
                    tail.starts_with('-') || [".zip", ".tar.gz", ".tgz", ".exe"].contains(&tail)
                });
            if !same_keys(artifact, &["filename", "kind", "sha256", "size"])
                || !portable_filename(filename)
                || !versioned_name
                || !names.insert(filename.to_ascii_lowercase())
                || !sha(&artifact["sha256"], 64)
                || !artifact["size"]
                    .as_u64()
                    .is_some_and(|size| size > 0 && size <= ARTIFACT_LIMIT)
                || !matches!(
                    kind,
                    "contract-package"
                        | "native-executable"
                        | "web-static"
                        | "rtl-source"
                        | "source"
                        | "npm-sdk"
                )
            {
                return Err(failed(format!(
                    "invalid artifact name, bytes or kind for {name}"
                )));
            }
            includes_required_kind |= component["kind"] == kind;
        }
        if !includes_required_kind {
            return Err(failed(format!(
                "required native artifact kind is missing for {name}"
            )));
        }
        if component["require_sdk"] == true
            && !artifacts
                .iter()
                .any(|artifact| artifact["kind"] == "npm-sdk")
        {
            return Err(failed(
                "Contracts must select the Interface npm SDK artifact",
            ));
        }
    }
    let contracts = &manifest["contracts"];
    if !same_keys(contracts, &DOMAINS) {
        return Err(failed(
            "native release must explicitly map every CIRNET contract domain",
        ));
    }
    for name in DOMAINS {
        let contract = &contracts[name];
        let owner = registry["release_policy"]["domain_owners"][name]
            .as_str()
            .ok_or_else(|| failed("Contract owner is absent from the selected profile"))?;
        if !same_keys(
            contract,
            &[
                "family",
                "revision",
                "filename",
                "sha256",
                "repository",
                "commit",
                "encoding",
            ],
        ) || contract["family"] != family
            || !contract["revision"].as_str().is_some_and(version)
            || !contract["filename"].as_str().is_some_and(portable_filename)
            || !sha(&contract["sha256"], 64)
            || !text(&contract["encoding"])
            || contract["repository"] != owner
            || contract["commit"] != repositories[owner]["commit"]
        {
            return Err(failed(format!(
                "contract {name} lacks an exact family, revision, owner or encoding"
            )));
        }
    }
    if !same_keys(
        &manifest["rollback"],
        &[
            "automaticStartReplay",
            "matchingDatabaseBackupRequired",
            "matchingCaptureBackupRequired",
            "preserveQuarantine",
            "automaticDowngrade",
        ],
    ) || manifest["rollback"]["automaticStartReplay"] != false
        || manifest["rollback"]["matchingDatabaseBackupRequired"] != true
        || manifest["rollback"]["matchingCaptureBackupRequired"] != true
        || manifest["rollback"]["preserveQuarantine"] != true
        || manifest["rollback"]["automaticDowngrade"] != false
    {
        return Err(failed(
            "rollback must preserve matched data and quarantine and disable automatic replay or downgrade",
        ));
    }
    Ok(())
}

fn unreviewed_python(path: &str) -> bool {
    let file = path.rsplit('/').next().unwrap_or_default();
    file.ends_with(".py")
        && file != "conftest.py"
        && !file.starts_with("test_")
        && !path.starts_with("docs/")
        && !path.split('/').any(|part| {
            matches!(
                part,
                "test"
                    | "tests"
                    | "consumer-tests"
                    | "testdata"
                    | "fixtures"
                    | "benches"
                    | "benchmarks"
                    | "benchmark"
            )
        })
}

fn check_evidence(
    evidence: &Value,
    gate: &Gate<'_>,
    source: &str,
    governance: &str,
    selection: &str,
) -> GateResult<()> {
    check_schema(evidence, EVIDENCE_SCHEMA, "gate evidence")?;
    if !same_keys(
        evidence,
        &[
            "schemaVersion",
            "gate",
            "repository",
            "sourceCommit",
            "governanceCommit",
            "selectionSha256",
            "command",
            "toolVersions",
            "platform",
            "recordedAt",
            "exitCode",
            "result",
            "evidenceType",
            "resultFile",
            "resultSha256",
            "limitations",
        ],
    ) || evidence["schemaVersion"] != 1
        || evidence["gate"] != gate.id
        || evidence["repository"] != gate.repo
        || evidence["sourceCommit"] != source
        || evidence["governanceCommit"] != governance
        || evidence["selectionSha256"] != selection
        || evidence["evidenceType"] != gate.kind
        || evidence["exitCode"] != 0
        || evidence["result"] != "PASS"
        || !evidence["resultFile"]
            .as_str()
            .is_some_and(portable_filename)
        || !sha(&evidence["resultSha256"], 64)
        || !text(&evidence["platform"])
        || !text(&evidence["limitations"])
        || !evidence["command"]
            .as_array()
            .is_some_and(|args| !args.is_empty() && args.len() <= 64 && args.iter().all(text))
        || !evidence["toolVersions"]
            .as_object()
            .is_some_and(|map| !map.is_empty() && map.len() <= 24 && map.values().all(text))
        || !evidence["recordedAt"].as_str().is_some_and(timestamp)
    {
        return Err(failed(format!(
            "gate {} evidence does not match exact selection, source, governance or verification kind",
            gate.id
        )));
    }
    Ok(())
}

fn timestamp(value: &str) -> bool {
    let b = value.as_bytes();
    let separators = [4, 7, 10, 13, 16, 19];
    if b.len() != 20
        || [b[4], b[7], b[10], b[13], b[16], b[19]] != *b"--T::Z"
        || b.iter()
            .enumerate()
            .any(|(i, byte)| !separators.contains(&i) && !byte.is_ascii_digit())
    {
        return false;
    }
    let number = |index: usize| (b[index] - b'0') * 10 + b[index + 1] - b'0';
    (1..=12).contains(&number(5))
        && (1..=31).contains(&number(8))
        && number(11) <= 23
        && number(14) <= 59
        && number(17) <= 59
}

fn evidence_approval<'a>(registry: &'a Value, index_sha256: &str) -> GateResult<Option<&'a str>> {
    let approval = &registry["release_policy"]["approved_evidence"];
    if approval.is_null() {
        return Ok(None);
    }
    if !same_keys(approval, &["governance_commit", "index_sha256"])
        || !sha(&approval["governance_commit"], 40)
        || !sha(&approval["index_sha256"], 64)
    {
        return Err(failed("malformed reviewed release evidence approval"));
    }
    if registry["_approved_evidence_verified"] != true {
        return Err(blocked("the reviewed evidence policy commit is unverified"));
    }
    if approval["index_sha256"] != index_sha256 {
        return Err(failed(
            "release evidence differs from the reviewed index digest",
        ));
    }
    Ok(approval["governance_commit"].as_str())
}

fn check_native(
    root: &Path,
    remote: &Path,
    registry: &Value,
    manifest: &Value,
) -> GateResult<Value> {
    let (train, family) = baseline(registry)?;
    if !same_keys(
        manifest,
        &[
            "schemaVersion",
            "releaseTrain",
            "family",
            "distribution",
            "repositories",
            "contracts",
            "rollback",
            "compatibilityMatrixSha256",
            "evidence",
        ],
    ) || manifest["releaseTrain"] != train
        || manifest["family"] != family
        || !matches!(
            manifest["distribution"].as_str(),
            Some("candidate" | "published")
        )
    {
        return Err(failed(
            "native manifest is malformed or outside the configured baseline",
        ));
    }
    let gates = gates(registry)?;
    if !same_keys(
        &manifest["evidence"],
        &gates.iter().map(|gate| gate.id).collect::<Vec<_>>(),
    ) {
        return Err(blocked(
            "native release is missing one or more required acceptance gates",
        ));
    }
    check_schema(manifest, RELEASE_SCHEMA, "native release selection")?;
    check_components(manifest, registry, train, family)?;
    if registry["_governance_origin_verified"] != true || registry["_governance_clean"] != true {
        return Err(blocked(
            "native release verification requires a clean governance checkout of verified origin",
        ));
    }
    let approved = check_policy(registry)?;
    if manifest["compatibilityMatrixSha256"] != approved {
        return Err(failed(
            "release set does not select the approved compatibility matrix",
        ));
    }
    let matrix_path = registry["release_policy"]["matrix_path"]
        .as_str()
        .ok_or_else(|| failed("No matrix path configured"))?;
    let matrix = read_input(remote, matrix_path)?;
    if digest(&matrix) != approved {
        return Err(failed(
            "compatibility matrix bytes differ from the approved digest",
        ));
    }
    let selection = check_matrix(&read_json(&matrix, matrix_path)?, manifest, train, family)?;
    let current_governance = registry["_governance_commit"]
        .as_str()
        .filter(|value| hex(value, 40))
        .ok_or_else(|| blocked("verification requires an actual governance Git commit"))?;
    let evidence_index_sha256 = canonical_digest(&manifest["evidence"])?;
    let evidence_governance = evidence_approval(registry, &evidence_index_sha256)?;
    let governance = evidence_governance.unwrap_or(current_governance);
    let evidence_root = safe_child(
        root,
        Path::new(&format!("cirnet-release-evidence/{selection}")),
    )?;
    let published = manifest["distribution"] == "published";
    for name in components(registry)?.keys().map(String::as_str) {
        let repo = checkout(root, name, registry)?;
        let entry = &manifest["repositories"][name];
        inventory::check_producer(
            &repo,
            name,
            entry,
            registry,
            &manifest["repositories"][authority(registry, "contracts")?.0],
            published,
            &evidence_root,
        )?;
        for artifact in entry["artifacts"].as_array().unwrap() {
            let filename = artifact["filename"].as_str().unwrap();
            checked_bytes(
                &evidence_root,
                &format!("artifacts/{name}/{filename}"),
                artifact["sha256"].as_str().unwrap(),
                artifact["size"].as_u64(),
            )?;
        }
    }
    for domain in DOMAINS {
        let contract = &manifest["contracts"][domain];
        checked_bytes(
            &evidence_root,
            &format!(
                "contracts/{domain}/{}",
                contract["filename"].as_str().unwrap()
            ),
            contract["sha256"].as_str().unwrap(),
            None,
        )?;
    }
    for gate in &gates {
        let reference = &manifest["evidence"][gate.id];
        if !same_keys(reference, &["filename", "sha256"])
            || reference["filename"] != format!("{}.json", gate.id)
            || !sha(&reference["sha256"], 64)
        {
            return Err(failed(format!(
                "unsafe or malformed evidence reference for {}",
                gate.id
            )));
        }
        let path = format!("gates/{}.json", gate.id);
        let bytes = read_input(&evidence_root, &path)?;
        if reference["sha256"] != digest(&bytes) {
            return Err(failed(format!("metadata digest mismatch for {}", gate.id)));
        }
        let evidence = read_json(&bytes, &path)?;
        check_evidence(
            &evidence,
            gate,
            manifest["repositories"][gate.repo]["commit"]
                .as_str()
                .unwrap(),
            governance,
            &selection,
        )?;
        checked_bytes(
            &evidence_root,
            &format!("results/{}", evidence["resultFile"].as_str().unwrap()),
            evidence["resultSha256"].as_str().unwrap(),
            None,
        )?;
    }
    if evidence_governance.is_none() {
        return Ok(
            json!({"check":"check-release", "status":"BLOCKED", "integrity_validation":"PASS",
            "reason":"the exact gate evidence index lacks governance approval", "selection_sha256":selection,
            "evidence_index_sha256":evidence_index_sha256}),
        );
    }
    if registry["baseline"]["activation"] != "ACTIVATED" || !published {
        return Ok(
            json!({"check": "check-release", "status": "BLOCKED", "candidate_validation": "PASS",
            "reason": "reviewed candidate is verified locally but baseline activation or publication is absent",
            "selection_sha256": selection, "evidence_index_sha256":evidence_index_sha256}),
        );
    }
    Ok(
        json!({"check": "check-release", "status": "PASS", "activation": "ACTIVATED",
        "release_train": train, "family": family, "selection_sha256": selection, "evidence_index_sha256":evidence_index_sha256,
        "scope": "local artifact, Git tag and software evidence verification"}),
    )
}

pub fn check_release(root: &Path, registry: &Value) -> Result<Value, String> {
    Ok(report(
        "check-release",
        (|| {
            validate_registry(registry)?;
            let (train, family) = baseline(registry)?;
            let root = workspace(root)?;
            let (owner, release_path) = authority(registry, "release_set")?;
            let remote = checkout(&root, owner, registry)?;
            let bytes = read_input(&remote, release_path)?;
            let manifest = read_json(&bytes, release_path)?;
            if manifest["schemaVersion"] == 1 {
                let selected = manifest["releaseVersion"]
                    .as_str()
                    .filter(|v| version(v))
                    .ok_or_else(|| failed("historical manifest has no valid release identity"))?;
                if !registry["release_policy"]["historical_system_releases"]
                    .as_array()
                    .is_some_and(|versions| versions.iter().any(|value| value == selected))
                {
                    return Err(failed(
                        "unreviewed v1 release manifest cannot activate the baseline",
                    ));
                }
                return Ok(
                    json!({"check": "check-release", "status": "BLOCKED", "observed_release": selected,
                "manifest_sha256": digest(&bytes), "target_release_train": train, "target_family": family,
                "activation": registry["baseline"]["activation"], "authority": format!("{owner}/{release_path}"),
                "reason": "the selected historical release is not a verified native cutover manifest"}),
                );
            }
            if manifest["schemaVersion"] != 2 {
                return Err(failed("unknown release-set schema version"));
            }
            if manifest["distribution"] == "published" {
                if git(
                    &remote,
                    &["cat-file", "blob", &format!("HEAD:{release_path}")],
                )? != bytes
                {
                    return Err(failed(
                        "published release selection differs from the committed owner manifest",
                    ));
                }
                if registry["baseline"]["activation"] == "ACTIVATED" {
                    check_publication(registry, &bytes)?;
                }
            }
            check_native(&root, &remote, registry, &manifest)
        })(),
    ))
}

fn check_publication(registry: &Value, manifest: &[u8]) -> GateResult<()> {
    let approved = registry["release_policy"]["approved_publication_sha256"]
        .as_str()
        .filter(|value| hex(value, 64))
        .ok_or_else(|| blocked("no reviewed publication manifest digest is recorded"))?;
    if digest(manifest) != approved {
        return Err(failed(
            "published owner manifest differs from the reviewed publication digest",
        ));
    }
    Ok(())
}
