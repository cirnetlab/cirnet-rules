use super::*;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::Sha512;

fn input<'a>(profile: &'a Value, field: &str) -> GateResult<&'a str> {
    profile[field]
        .as_str()
        .filter(|path| relative_input(path))
        .ok_or_else(|| failed(format!("Invalid release profile input {field}")))
}

fn utf8<'a>(bytes: &'a [u8], path: &str) -> GateResult<&'a str> {
    std::str::from_utf8(bytes)
        .map(|value| value.trim_start_matches('\u{feff}'))
        .map_err(|_| failed(format!("non-UTF-8 manifest: {path}")))
}

fn toml(bytes: &[u8], path: &str) -> GateResult<toml::Value> {
    toml::from_str(utf8(bytes, path)?)
        .map_err(|_| failed(format!("invalid or ambiguous TOML manifest: {path}")))
}

fn toml_text<'a>(value: &'a toml::Value, key: &[&str]) -> Option<&'a str> {
    key.iter()
        .try_fold(value, |value, field| value.get(*field))
        .and_then(toml::Value::as_str)
}

fn cmake_version(source: &str) -> Option<&str> {
    let mut found = None;
    for line in source.lines().map(str::trim) {
        if let Some(project) = line.strip_prefix("project(") {
            let mut tokens = project.split(')').next()?.split_ascii_whitespace();
            tokens.next()?;
            if tokens.any(|token| token == "VERSION") {
                if found.is_some() {
                    return None;
                }
                found = tokens.next();
            }
        }
    }
    found
}

fn primary(repo: &str, profile: &Value, bytes: &[u8]) -> GateResult<String> {
    let path = input(profile, "manifest")?;
    let value = match profile["format"].as_str().unwrap_or_default() {
        "cmake" => cmake_version(utf8(bytes, path)?)
            .unwrap_or_default()
            .to_owned(),
        "npm" => read_json(bytes, path)?["version"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        "text" => utf8(bytes, path)?.trim().to_owned(),
        "cargo" => toml_text(&toml(bytes, path)?, &["package", "version"])
            .unwrap_or_default()
            .to_owned(),
        _ => return Err(failed("Unsupported release producer format")),
    };
    if !version(&value) {
        return Err(failed(format!(
            "{repo} has no unambiguous product version in {path}"
        )));
    }
    Ok(value)
}

fn cargo_lock_version<'a>(lock: &'a toml::Value, name: &str) -> GateResult<&'a toml::Value> {
    let mut matches = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .filter(|package| toml_text(package, &["name"]) == Some(name));
    let package = matches
        .next()
        .ok_or_else(|| failed(format!("Cargo.lock does not select {name}")))?;
    if matches.next().is_some() {
        return Err(failed(format!(
            "Cargo.lock selects multiple identities for {name}"
        )));
    }
    Ok(package)
}

fn cargo_observations(
    repo: &str,
    profile: &Value,
    manifest: &[u8],
    lock: &[u8],
    registry: &Value,
) -> GateResult<Value> {
    let manifest = toml(manifest, "Cargo.toml")?;
    let lock = toml(lock, "Cargo.lock")?;
    let name = toml_text(&manifest, &["package", "name"])
        .ok_or_else(|| failed("Cargo package identity is absent"))?;
    let own_version = toml_text(&manifest, &["package", "version"])
        .ok_or_else(|| failed("Cargo package version is absent"))?;
    let locked = cargo_lock_version(&lock, name)?;
    if toml_text(locked, &["version"]) != Some(own_version) {
        return Err(failed(format!(
            "{repo} Cargo.toml and Cargo.lock disagree on package version"
        )));
    }
    let mut observation =
        json!({"package": name, "msrv": toml_text(&manifest, &["package", "rust-version"])});
    if let Some(dep) = profile["contracts_dependency"].as_str() {
        let contracts_owner = authority(registry, "contracts")?.0;
        let expected_remote = registry["repositories"][contracts_owner]["remote"]
            .as_str()
            .ok_or_else(|| failed("Selected Contracts Git provenance is absent"))?;
        let requirement = |section: &str| {
            let rev = toml_text(&manifest, &[section, dep, "rev"])?;
            let ver = toml_text(&manifest, &[section, dep, "version"])?.strip_prefix('=')?;
            let source = toml_text(&manifest, &[section, dep, "git"])?;
            (normalized_remote(source, expected_remote) && hex(rev, 40) && version(ver))
                .then_some((rev, ver))
        };
        let (rev, ver) = requirement("dependencies")
            .ok_or_else(|| failed(format!("{repo} Contracts is not pinned")))?;
        if requirement("build-dependencies") != Some((rev, ver)) {
            return Err(failed(format!(
                "{repo} runtime and build Contracts pins differ"
            )));
        }
        let locked = cargo_lock_version(&lock, dep)?;
        let source = toml_text(locked, &["source"]).unwrap_or_default();
        let (source, selected_rev) = source.rsplit_once('#').unwrap_or_default();
        let source = source
            .strip_prefix("git+")
            .and_then(|value| value.split('?').next())
            .unwrap_or_default();
        if !normalized_remote(source, expected_remote)
            || selected_rev != rev
            || toml_text(locked, &["version"]) != Some(ver)
        {
            return Err(failed(format!(
                "{repo} Contracts source pin and Cargo.lock disagree"
            )));
        }
        observation["contracts"] = json!({"package_version": ver, "source_commit": rev});
    }
    Ok(observation)
}

fn npm_observations(
    package: &[u8],
    lock: &[u8],
    contracts: &[u8],
    profile: &Value,
) -> GateResult<Value> {
    let package = read_json(package, "package.json")?;
    let lock = read_json(lock, "package-lock.json")?;
    let contracts = read_json(contracts, input(profile, "sdk_handoff")?)?;
    let dependency = profile["sdk_package"]
        .as_str()
        .filter(|value| !value.trim().is_empty() && value.len() <= 256)
        .ok_or_else(|| failed("SDK package identity is missing"))?;
    let locked = &lock["packages"][format!("node_modules/{dependency}")];
    let filename = contracts["npm"]["filename"]
        .as_str()
        .filter(|file| portable_filename(file))
        .ok_or_else(|| failed("Interface Contracts tarball name is missing"))?;
    let expected_path = format!("file:.artifacts/{filename}");
    if package["version"] != lock["version"]
        || package["version"] != lock["packages"][""]["version"]
        || package["dependencies"][dependency] != expected_path
        || lock["packages"][""]["dependencies"][dependency] != expected_path
        || locked["resolved"] != expected_path
        || locked["version"] != contracts["npm"]["version"]
        || contracts["contractsVersion"] != contracts["npm"]["version"]
        || !sha(&contracts["npm"]["sha256"], 64)
        || !locked["integrity"].as_str().is_some_and(valid_sdk_sri)
    {
        return Err(failed(
            "Interface package, npm lock and Contracts artifact identity disagree",
        ));
    }
    Ok(
        json!({"package_version": locked["version"], "sha256": contracts["npm"]["sha256"],
            "filename": filename, "integrity": locked["integrity"]}),
    )
}

fn valid_sdk_sri(value: &str) -> bool {
    value.strip_prefix("sha512-").is_some_and(|encoded| {
        encoded.len() == 88
            && STANDARD
                .decode(encoded)
                .is_ok_and(|bytes| bytes.len() == 64 && STANDARD.encode(bytes) == encoded)
    })
}

fn check_sdk_bytes(root: &Path, path: &str, identity: &Value) -> GateResult<()> {
    let path = safe_child(root, Path::new(path))?;
    let file = File::open(path).map_err(|_| blocked("cannot read the selected npm SDK"))?;
    let metadata = file
        .metadata()
        .map_err(|_| blocked("cannot inspect the selected npm SDK"))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > ARTIFACT_LIMIT {
        return Err(failed("invalid npm SDK file kind or size"));
    }
    let mut input = file.take(ARTIFACT_LIMIT + 1);
    let mut sha256 = Sha256::new();
    let mut sha512 = Sha512::new();
    let mut buffer = [0; 64 * 1024];
    let mut size = 0;
    loop {
        let count = input
            .read(&mut buffer)
            .map_err(|_| blocked("cannot hash the selected npm SDK"))?;
        if count == 0 {
            break;
        }
        size += count as u64;
        sha256.update(&buffer[..count]);
        sha512.update(&buffer[..count]);
    }
    if size != metadata.len()
        || identity["sha256"] != format!("{:x}", sha256.finalize())
        || identity["integrity"] != format!("sha512-{}", STANDARD.encode(sha512.finalize()))
    {
        return Err(failed(
            "npm SDK bytes do not match both the artifact SHA-256 and npm lock SHA-512 integrity",
        ));
    }
    Ok(())
}

fn scan_python(tracked: &[u8]) -> GateResult<Vec<&str>> {
    let files = std::str::from_utf8(tracked).map_err(|_| failed("tracked paths are not UTF-8"))?;
    Ok(files
        .split('\0')
        .filter(|path| unreviewed_python(path))
        .collect())
}

pub(super) fn release_inputs<'a>(repo: &str, registry: &'a Value) -> GateResult<Vec<&'a str>> {
    let profile = components(registry)?
        .get(repo)
        .ok_or_else(|| failed("Unknown release producer"))?;
    let mut paths = vec![input(profile, "manifest")?, input(profile, "lock")?];
    for path in profile["legacy"]
        .as_object()
        .into_iter()
        .flat_map(|values| values.keys())
    {
        if !relative_input(path) {
            return Err(failed("Unsafe historical input path"));
        }
        paths.push(path);
    }
    let (owner, path) = authority(registry, "release_set")?;
    if repo == owner {
        paths.extend([path, input(&registry["release_policy"], "matrix_path")?]);
    }
    if repo == authority(registry, "contracts")?.0 {
        paths.push(input(&registry["release_policy"], "openapi_handoff")?);
    }
    if profile.get("sdk_handoff").is_some() {
        paths.push(input(profile, "sdk_handoff")?);
    }
    Ok(paths)
}

fn legacy_indicators(
    root: &Path,
    profile: &Value,
    commit: Option<&str>,
) -> GateResult<Vec<String>> {
    let mut indicators = Vec::new();
    for (path, token) in profile["legacy"].as_object().into_iter().flatten() {
        let token = token
            .as_str()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| failed("Invalid legacy rule"))?;
        let content = match commit {
            Some(revision) => {
                if git(root, &["cat-file", "-e", &format!("{revision}:{path}")]).is_err() {
                    continue;
                }
                git(root, &["cat-file", "blob", &format!("{revision}:{path}")])?
            }
            None => {
                if !root.join(path).is_file() {
                    continue;
                }
                read_input(root, path)?
            }
        };
        if utf8(&content, path)?.contains(token) {
            indicators.push(path.to_owned());
        }
    }
    Ok(indicators)
}

pub(super) fn check(root: &Path, registry: &Value) -> GateResult<Value> {
    validate_registry(registry)?;
    let (train, family) = baseline(registry)?;
    let root = workspace(root)?;
    let mut observations = Map::new();
    let mut pending = Vec::new();
    for (name, profile) in components(registry)? {
        let repo = checkout(&root, name, registry)?;
        let manifest_path = input(profile, "manifest")?;
        let lock_path = input(profile, "lock")?;
        let manifest = read_input(&repo, manifest_path)?;
        let lock = read_input(&repo, lock_path)?;
        let product_version = primary(name, profile, &manifest)?;
        let mut observation = json!({
            "product_version": product_version,
            "manifest": manifest_path,
            "lock_or_build_input": lock_path,
            "lock_or_build_input_sha256": digest(&lock),
            "checkout_commit": git_text(&repo, &["rev-parse", "HEAD"])?,
        });
        if product_version != train {
            pending.push(format!(
                "{name}: observed product {product_version} differs from target train {train}"
            ));
        }
        match profile["format"].as_str().unwrap_or_default() {
            "npm" if profile.get("sdk_handoff").is_some() => {
                let contracts = read_input(&repo, input(profile, "sdk_handoff")?)?;
                let mut identity = npm_observations(&manifest, &lock, &contracts, profile)?;
                if identity["package_version"] != train {
                    pending.push(format!(
                        "{name}: selected SDK differs from the target Contracts product train"
                    ));
                }
                let sdk_path = format!(".artifacts/{}", identity["filename"].as_str().unwrap());
                identity["local_bytes"] = if repo.join(&sdk_path).exists() {
                    check_sdk_bytes(&repo, &sdk_path, &identity)?;
                    json!("VERIFIED")
                } else {
                    pending.push(format!(
                        "{name}: selected Contracts SDK bytes are unavailable locally"
                    ));
                    json!("NOT_AVAILABLE")
                };
                observation["contracts"] = identity;
            }
            "cmake" => {
                observation["native_language"] = json!("C++");
            }
            "text" => {
                observation["toolchain"] = read_json(&lock, lock_path)?["tag"].clone();
            }
            "cargo" => {
                let mut cargo = cargo_observations(name, profile, &manifest, &lock, registry)?;
                cargo["toolchain"] = json!(toml_text(
                    &toml(
                        &read_input(&repo, "rust-toolchain.toml")?,
                        "rust-toolchain.toml"
                    )?,
                    &["toolchain", "channel"]
                ));
                if profile.get("contracts_dependency").is_some()
                    && cargo["contracts"]["package_version"] != train
                {
                    pending.push(format!("{name}: locked Contracts crate differs from the target Contracts product train"));
                }
                observation["cargo"] = cargo;
            }
            _ => {}
        }
        if repo.join("pyproject.toml").is_file() {
            let config = toml(&read_input(&repo, "pyproject.toml")?, "pyproject.toml")?;
            observation["python_distribution"] = json!(toml_text(&config, &["project", "version"]));
        }
        let tracked = git(&repo, &["ls-files", "-z", "--", "*.py"])?;
        let python = scan_python(&tracked)?;
        let legacy = legacy_indicators(&repo, profile, None)?;
        if !python.is_empty() || !legacy.is_empty() {
            pending.push(format!(
                "{name}: Python or an observed legacy lane still requires classification or removal"
            ));
        }
        observation["python_outside_test_paths"] = json!({"count": python.len(), "sample": python.into_iter().take(12).collect::<Vec<_>>()});
        observation["legacy_indicators"] = json!(legacy);
        observations.insert(name.to_owned(), observation);
    }
    let (owner, release_path) = authority(registry, "release_set")?;
    let remote = checkout(&root, owner, registry)?;
    let release = read_json(&read_input(&remote, release_path)?, release_path)?;
    let selected = match release["schemaVersion"].as_u64() {
        Some(1) => {
            let value = release["releaseVersion"]
                .as_str()
                .filter(|v| version(v))
                .ok_or_else(|| failed("historical release identity is absent"))?;
            pending.push(format!("Remote still selects historical release {value}"));
            value
        }
        Some(2) => {
            if release["releaseTrain"] != train || release["family"] != family {
                return Err(failed(
                    "Remote release-set does not select the reviewed train and family",
                ));
            }
            train
        }
        _ => return Err(failed("unknown release manifest schema")),
    };
    let (contracts_id, _) = authority(registry, "contracts")?;
    let contracts = checkout(&root, contracts_id, registry)?;
    let openapi_path = input(&registry["release_policy"], "openapi_handoff")?;
    let openapi = read_json(&read_input(&contracts, openapi_path)?, openapi_path)?;
    if !text(&openapi["openapi"]) || !text(&openapi["info"]["version"]) {
        return Err(failed(
            "Contracts OpenAPI dialect or document version is missing",
        ));
    }
    Ok(
        json!({"check": "check-versions", "status": if pending.is_empty() {"PASS"} else {"BLOCKED"},
        "observed": observations, "selected_system_release": selected, "target_release_train": train, "target_family": family,
        "openapi_dialect": openapi["openapi"], "openapi_document_version": openapi["info"]["version"],
        "reason": if pending.is_empty() {"version and known source checks passed; use check-release for cutover"}
            else {"observed source or selected release requires cutover work"}, "pending": pending}),
    )
}

pub(super) fn check_producer(
    repo: &Path,
    name: &str,
    entry: &Value,
    registry: &Value,
    contracts: &Value,
    published: bool,
    evidence_root: &Path,
) -> GateResult<()> {
    let commit = entry["commit"].as_str().unwrap();
    let train = entry["version"].as_str().unwrap();
    if git_text(repo, &["cat-file", "-t", commit])? != "commit" {
        return Err(failed(format!(
            "{name} source identity is not a Git commit"
        )));
    }
    if published
        && git_text(
            repo,
            &[
                "rev-parse",
                "--verify",
                &format!("refs/tags/v{train}^{{commit}}"),
            ],
        )? != commit
    {
        return Err(failed(format!(
            "{name} local published tag resolves to a different commit"
        )));
    }
    let source = |path: &str| {
        git(repo, &["cat-file", "blob", &format!("{commit}:{path}")]).map_err(|error| match error {
            GateError::Blocked(reason) => blocked(format!("{reason}: {path}")),
            other => other,
        })
    };
    let profile = &components(registry)?[name];
    let manifest = source(input(profile, "manifest")?)?;
    let lock = source(input(profile, "lock")?)?;
    if primary(name, profile, &manifest)? != train || entry["lockSha256"] != digest(&lock) {
        return Err(failed(format!(
            "{name} selected source has a different product version or build lock"
        )));
    }
    let locked_contracts = match profile["format"].as_str().unwrap_or_default() {
        "npm" if profile.get("sdk_handoff").is_some() => npm_observations(
            &manifest,
            &lock,
            &source(input(profile, "sdk_handoff")?)?,
            profile,
        )?,
        "cargo" => {
            cargo_observations(name, profile, &manifest, &lock, registry)?["contracts"].clone()
        }
        _ => Value::Null,
    };
    if !locked_contracts.is_null() && locked_contracts["package_version"] != contracts["version"] {
        return Err(failed(format!(
            "{name} selected source still pins a different first-party Contracts package"
        )));
    }
    if profile.get("contracts_dependency").is_some()
        && locked_contracts["source_commit"] != contracts["commit"]
    {
        return Err(failed(format!(
            "{name} does not pin the exact selected Contracts source commit"
        )));
    }
    if profile.get("sdk_handoff").is_some() {
        if !contracts["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|artifact| {
                artifact["kind"] == "npm-sdk"
                    && artifact["sha256"] == locked_contracts["sha256"]
                    && artifact["filename"] == locked_contracts["filename"]
            })
        {
            return Err(failed(
                "Interface does not consume the exact selected Contracts SDK bytes",
            ));
        }
        check_sdk_bytes(
            evidence_root,
            &format!(
                "artifacts/{}/{}",
                authority(registry, "contracts")?.0,
                locked_contracts["filename"].as_str().unwrap()
            ),
            &locked_contracts,
        )?;
    }
    let paths = git(repo, &["ls-tree", "-r", "-z", "--name-only", commit])?;
    if !scan_python(&paths)?.is_empty()
        || !legacy_indicators(repo, profile, Some(commit))?.is_empty()
    {
        return Err(failed(format!(
            "{name} selected source still has unreviewed Python or an observed legacy lane"
        )));
    }
    Ok(())
}
