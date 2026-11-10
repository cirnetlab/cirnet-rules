use super::{Pins, Project, read_optional, version};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Kind {
    Selector,
    Alternatives,
    Range,
    NpmRange,
}

#[derive(Serialize)]
pub(super) struct Requirement {
    pub source: String,
    pub value: String,
    pub kind: Kind,
}

impl Requirement {
    pub fn exact(&self) -> Option<String> {
        matches!(self.kind, Kind::Selector).then_some(())?;
        version::exact(&self.value)?;
        Some(self.value.trim_start_matches('v').to_owned())
    }

    pub fn matches(&self, version: version::Version) -> Option<bool> {
        match self.kind {
            Kind::Selector => version::selector(&self.value, version),
            Kind::Alternatives => {
                let mut unknown = false;
                for alternative in self.value.split_ascii_whitespace() {
                    match version::selector(alternative, version) {
                        Some(true) => return Some(true),
                        Some(false) => {}
                        None => unknown = true,
                    }
                }
                (!unknown).then_some(false)
            }
            Kind::Range => version::constraint(&self.value, version, false),
            Kind::NpmRange => version::constraint(&self.value, version, true),
        }
    }
}

#[derive(Default)]
pub(super) struct Inventory {
    pub requirements: BTreeMap<String, Vec<Requirement>>,
    pub files: BTreeMap<String, String>,
}

impl Inventory {
    pub fn read(project: &Project) -> Result<Self, String> {
        let mut inventory = Self::default();
        for file in [
            "rust-toolchain.toml",
            "rust-toolchain",
            "Cargo.toml",
            ".tool-versions",
            ".node-version",
            ".nvmrc",
            ".python-version",
            "package.json",
            "pyproject.toml",
            "CMakeLists.txt",
            "CMakePresets.json",
        ] {
            let Some(data) = read_optional(&project.root().join(file))? else {
                continue;
            };
            inventory.files.insert(
                file.into(),
                format!("{:x}", Sha256::digest(data.as_bytes())),
            );
            inventory.parse(file, &data)?;
        }
        Ok(inventory)
    }

    fn add(&mut self, tool: &str, file: &str, value: &str, kind: Kind) -> Result<(), String> {
        let value = value.trim();
        if value.is_empty()
            || value.len() > 160
            || !value
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b" -.><=^~*,|+_/!".contains(&c))
        {
            return Err(format!("Invalid {tool} version requirement in {file}"));
        }
        self.requirements
            .entry(tool.to_owned())
            .or_default()
            .push(Requirement {
                source: file.to_owned(),
                value: value.to_owned(),
                kind,
            });
        Ok(())
    }

    fn parse(&mut self, file: &str, data: &str) -> Result<(), String> {
        match file {
            ".node-version" | ".nvmrc" | ".python-version" | "rust-toolchain"
                if !data.trim_start().starts_with('[') || file != "rust-toolchain" =>
            {
                let tool = match file {
                    ".python-version" => "python",
                    "rust-toolchain" => "rust",
                    _ => "node",
                };
                let lines: Vec<_> = data
                    .lines()
                    .map(|line| line.split('#').next().unwrap_or("").trim())
                    .filter(|line| !line.is_empty())
                    .collect();
                match lines.as_slice() {
                    [version] => self.add(tool, file, version, Kind::Selector)?,
                    versions if file == ".python-version" && !versions.is_empty() => {
                        self.add(tool, file, &versions.join(" "), Kind::Alternatives)?;
                    }
                    _ => return Err(format!("Expected a single version selector in {file}")),
                }
            }
            ".tool-versions" => {
                let mut seen = std::collections::BTreeSet::new();
                for line in data.lines() {
                    let mut words = line
                        .split('#')
                        .next()
                        .unwrap_or("")
                        .split_ascii_whitespace();
                    let Some(tool) = words
                        .next()
                        .map(|tool| if tool == "nodejs" { "node" } else { tool })
                    else {
                        continue;
                    };
                    if !super::known_tool(tool) {
                        continue;
                    }
                    if !seen.insert(tool) {
                        return Err(format!("Duplicate {tool} entry in {file}"));
                    }
                    let versions: Vec<_> = words.collect();
                    let kind = if versions.len() == 1 {
                        Kind::Selector
                    } else {
                        Kind::Alternatives
                    };
                    self.add(tool, file, &versions.join(" "), kind)?;
                }
            }
            "package.json" => {
                #[derive(Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct Manifest {
                    #[serde(default)]
                    engines: BTreeMap<String, String>,
                    package_manager: Option<String>,
                }
                let manifest: Manifest = serde_json::from_str(data)
                    .map_err(|error| format!("Invalid {file} at line {}", error.line()))?;
                for (tool, requirement) in manifest.engines {
                    if matches!(tool.as_str(), "node" | "npm" | "pnpm" | "yarn") {
                        self.add(&tool, file, &requirement, Kind::NpmRange)?;
                    }
                }
                if let Some((tool, value)) = manifest
                    .package_manager
                    .as_deref()
                    .and_then(|s| s.split_once('@'))
                    && matches!(tool, "npm" | "pnpm" | "yarn")
                {
                    // Corepack allows an integrity suffix after the version.
                    let value = value.split('+').next().unwrap_or(value);
                    self.add(tool, file, value, Kind::Selector)?;
                }
            }
            "CMakeLists.txt" => {
                // Read a literal minimum without evaluating CMake code or project hooks.
                let mut lines = data
                    .lines()
                    .map(|line| line.split('#').next().unwrap_or("").trim());
                while let Some(line) = lines.next() {
                    let Some((command, rest)) = line.split_once('(') else {
                        continue;
                    };
                    if !command
                        .trim_end()
                        .eq_ignore_ascii_case("cmake_minimum_required")
                    {
                        continue;
                    }
                    let mut args = String::new();
                    for part in std::iter::once(rest).chain(lines.by_ref()) {
                        let (tokens, complete) = part
                            .split_once(')')
                            .map_or((part, false), |(tokens, _)| (tokens, true));
                        if args.len() + tokens.len() > 512 {
                            return Err("CMake minimum declaration exceeds its size limit".into());
                        }
                        args.push(' ');
                        args.push_str(tokens);
                        if complete {
                            break;
                        }
                    }
                    let mut words = args.split_ascii_whitespace();
                    let value = words
                        .next()
                        .filter(|word| word.eq_ignore_ascii_case("VERSION"))
                        .and_then(|_| words.next())
                        .map(|value| value.trim_matches('"').split("...").next().unwrap_or(""));
                    if let Some(value) =
                        value.filter(|value| version::selector(value, [0; 3]).is_some())
                    {
                        self.add("cmake", file, &format!(">={value}"), Kind::Range)?;
                    } else {
                        self.add("cmake", file, "dynamic", Kind::Selector)?;
                    }
                }
            }
            "CMakePresets.json" => {
                #[derive(Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct Presets {
                    cmake_minimum_required: Option<Minimum>,
                }
                #[derive(Deserialize)]
                struct Minimum {
                    #[serde(default)]
                    major: u64,
                    #[serde(default)]
                    minor: u64,
                    #[serde(default)]
                    patch: u64,
                }
                let presets: Presets = serde_json::from_str(data)
                    .map_err(|error| format!("Invalid {file} at line {}", error.line()))?;
                if let Some(v) = presets.cmake_minimum_required {
                    self.add(
                        "cmake",
                        file,
                        &format!(">={}.{}.{}", v.major, v.minor, v.patch),
                        Kind::Range,
                    )?;
                }
            }
            _ => {
                let manifest: toml::Value =
                    toml::from_str(data).map_err(|_| format!("Invalid {file}"))?;
                let at = |keys: &[&str]| {
                    keys.iter()
                        .try_fold(&manifest, |value, key| value.get(*key))
                };
                match file {
                    "rust-toolchain" | "rust-toolchain.toml" => {
                        if at(&["toolchain", "path"]).is_some() {
                            return Err(format!(
                                "Path-based Rust toolchains in {file} require a manually managed environment"
                            ));
                        }
                        let channel = at(&["toolchain", "channel"])
                            .and_then(toml::Value::as_str)
                            .ok_or_else(|| format!("No Rust toolchain channel in {file}"))?;
                        self.add("rust", file, channel, Kind::Selector)?;
                    }
                    "Cargo.toml" => {
                        let version = at(&["package", "rust-version"])
                            .and_then(toml::Value::as_str)
                            .or_else(|| {
                                at(&["workspace", "package", "rust-version"])
                                    .and_then(toml::Value::as_str)
                            });
                        if let Some(version) = version {
                            self.add("rust", file, &format!(">={version}"), Kind::Range)?;
                        }
                    }
                    "pyproject.toml" => {
                        for requirement in [
                            at(&["project", "requires-python"]),
                            at(&["tool", "poetry", "dependencies", "python"]),
                        ]
                        .into_iter()
                        .flatten()
                        {
                            if let Some(requirement) = requirement.as_str().or_else(|| {
                                requirement.get("version").and_then(toml::Value::as_str)
                            }) {
                                self.add("python", file, requirement, Kind::Range)?;
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
        Ok(())
    }

    pub fn resolve(&self, mut selected: Pins) -> Result<(Pins, Vec<String>), String> {
        let mut unresolved = Vec::new();
        for (tool, requirements) in &self.requirements {
            for req in requirements {
                if let Some(exact) = req.exact() {
                    let selection = selected
                        .entry(tool.clone())
                        .or_insert_with(|| exact.clone());
                    if selection != &exact {
                        return Err(format!(
                            "Conflicting {tool} versions in CNR configuration or {}",
                            req.source
                        ));
                    }
                }
            }
            if let Some(version) = selected.get(tool).and_then(|value| version::exact(value)) {
                for req in requirements {
                    match req.matches(version) {
                        Some(true) => {}
                        Some(false) => {
                            return Err(format!(
                                "Selected {tool} version conflicts with {}",
                                req.source
                            ));
                        }
                        None => {
                            return Err(format!(
                                "Cannot verify the selected {tool} version against {}",
                                req.source
                            ));
                        }
                    }
                }
            } else {
                unresolved.push(tool.clone());
            }
        }
        Ok((selected, unresolved))
    }
}
