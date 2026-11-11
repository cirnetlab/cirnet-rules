use super::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AssetKind {
    Rule,
    Policy,
    Skill,
    Profile,
    Reference,
    Hook,
    Command,
}

impl AssetKind {
    pub(super) fn manual(self) -> bool {
        matches!(self, Self::Hook | Self::Command)
    }

    fn namespace(self, path: &str) -> bool {
        match self {
            Self::Rule => path.starts_with("en/rules/"),
            Self::Policy => path == "en/AGENTS.md" || path.starts_with("en/policies/"),
            Self::Skill => path.strip_prefix("en/skills/").is_some_and(|suffix| {
                suffix
                    .split_once('/')
                    .is_some_and(|(_, name)| name == "SKILL.md")
            }),
            Self::Profile => path
                .strip_prefix("en/profiles/")
                .is_some_and(|suffix| !suffix.contains('/')),
            Self::Reference => {
                path.starts_with("en/references/") || path.starts_with("en/routing/")
            }
            Self::Hook => path.starts_with("en/hooks/"),
            Self::Command => path.starts_with("en/commands/"),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum AssetStatus {
    Active,
    Draft,
    Archived,
    Retired,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Asset {
    pub id: String,
    pub kind: AssetKind,
    pub path: String,
    pub revision: u64,
    status: AssetStatus,
    pub depends_on: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    repositories: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    provenance: Option<Provenance>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Provenance {
    snapshot_sha256: String,
    legacy_asset_id: String,
    legacy_revision: u64,
}

impl Asset {
    pub(super) fn active(&self) -> bool {
        self.status == AssetStatus::Active
    }

    pub(super) fn assigned(&self, repository: &str) -> bool {
        self.repositories.is_empty() || self.repositories.iter().any(|repo| repo == repository)
    }

    fn validate(&self, registry: &Value) -> Result<(), String> {
        let valid_id = |id: &str| {
            id.len() <= 80
                && id.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
                && id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
        };
        if !valid_id(&self.id) || self.revision == 0 {
            return Err("Invalid authority asset ID or revision".into());
        }
        crate::tasks::relative(&self.path)?;
        if !self.kind.namespace(&self.path)
            || !self.path.ends_with(".md")
            || self.path.split('/').any(|part| {
                matches!(
                    part.to_ascii_lowercase().as_str(),
                    "history" | "archive" | "archived" | "legacy" | "retired" | "vi"
                ) || part.starts_with('.')
                    || !part
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
            })
        {
            return Err(format!(
                "Invalid English authority asset path for {}",
                self.id
            ));
        }
        if self.active()
            && matches!(
                registry["document_states"][&self.path]["state"].as_str(),
                Some("DRAFT" | "SUPERSEDED" | "ARCHIVED" | "RETIRED")
            )
        {
            return Err(format!(
                "Active asset {} conflicts with its document state",
                self.id
            ));
        }
        let mut assigned = BTreeSet::new();
        for repo in &self.repositories {
            if !assigned.insert(repo) || !eligible_repository(registry, repo) {
                return Err(format!("Invalid repository assignment for {}", self.id));
            }
        }
        if let Some(provenance) = &self.provenance
            && (provenance.legacy_revision == 0
                || !valid_id(&provenance.legacy_asset_id)
                || provenance.snapshot_sha256.len() != 64
                || !provenance
                    .snapshot_sha256
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()))
        {
            return Err(format!("Invalid migration provenance for {}", self.id));
        }
        Ok(())
    }
}

pub(super) fn eligible_repository(registry: &Value, repo: &str) -> bool {
    repo == "_workspace"
        || registry["repositories"].get(repo).is_some_and(|entry| {
            matches!(
                entry["classification"].as_str().unwrap_or("active"),
                "active" | "supporting"
            )
        })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Catalog {
    schema: u8,
    pub entries: Vec<Asset>,
}

impl Catalog {
    pub fn load(registry: &Value) -> Result<Self, String> {
        let catalog: Self = serde_json::from_value(registry["assets"].clone())
            .map_err(|_| "Invalid authority asset catalog")?;
        if catalog.schema != 1 || catalog.entries.is_empty() || catalog.entries.len() > 512 {
            return Err("Unsupported authority catalog schema or size".into());
        }
        let mut ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for asset in &catalog.entries {
            asset.validate(registry)?;
            if !ids.insert(asset.id.to_ascii_lowercase())
                || !paths.insert(asset.path.to_ascii_lowercase())
            {
                return Err(format!(
                    "Duplicate authority asset ID or path: {}",
                    asset.id
                ));
            }
        }
        let by_id = catalog.by_id();
        let mut complete = BTreeSet::new();
        for asset in &catalog.entries {
            visit(
                asset,
                &by_id,
                &mut BTreeSet::new(),
                &mut complete,
                &mut Vec::new(),
            )?;
        }
        catalog.validate_assignments(registry)?;
        Ok(catalog)
    }

    fn validate_assignments(&self, registry: &Value) -> Result<(), String> {
        let global = |path: &str, kind: Option<AssetKind>| {
            let asset = self.path(path)?;
            if !asset.active()
                || !asset.repositories.is_empty()
                || asset.kind.manual()
                || kind.is_some_and(|kind| asset.kind != kind)
            {
                return Err(format!("Invalid global authority assignment: {}", asset.id));
            }
            Ok(())
        };
        global(
            registry["control_plane"]["entrypoint"]
                .as_str()
                .unwrap_or("en/AGENTS.md"),
            Some(AssetKind::Policy),
        )?;
        let mut common = BTreeSet::new();
        for path in registry["context"]["common"]
            .as_array()
            .into_iter()
            .flatten()
        {
            let path = path.as_str().ok_or("Invalid common source path")?;
            if !common.insert(path) {
                return Err(format!("Duplicate common authority assignment: {path}"));
            }
            global(path, None)?;
        }
        for task in registry["context"]["tasks"]
            .as_object()
            .into_iter()
            .flat_map(|tasks| tasks.values())
        {
            for (key, kind) in [("profile", AssetKind::Profile), ("skill", AssetKind::Skill)] {
                global(
                    task[key]
                        .as_str()
                        .ok_or("Incomplete task authority assignment")?,
                    Some(kind),
                )?;
            }
        }
        for skill in registry["control_plane"]["skills"]
            .as_object()
            .into_iter()
            .flat_map(|skills| skills.values())
        {
            global(
                skill.as_str().ok_or("Invalid mode skill path")?,
                Some(AssetKind::Skill),
            )?;
        }
        for rule in registry["rules"].as_array().into_iter().flatten() {
            let path = rule["path"].as_str().ok_or("Missing registry rule path")?;
            global(path, None)?;
            let asset = self.path(path)?;
            if asset.kind != AssetKind::Policy
                && (asset.kind != AssetKind::Rule || rule["id"] != asset.id)
            {
                return Err(format!(
                    "Rule ID differs from the authority catalog: {path}"
                ));
            }
        }
        Ok(())
    }

    pub fn path(&self, path: &str) -> Result<&Asset, String> {
        self.entries
            .iter()
            .find(|asset| asset.path == path)
            .ok_or_else(|| format!("Unregistered authority source: {path}"))
    }

    pub fn by_id(&self) -> BTreeMap<&str, &Asset> {
        self.entries
            .iter()
            .map(|asset| (asset.id.as_str(), asset))
            .collect()
    }
}

pub(super) fn visit<'a>(
    asset: &'a Asset,
    catalog: &BTreeMap<&str, &'a Asset>,
    active: &mut BTreeSet<&'a str>,
    complete: &mut BTreeSet<&'a str>,
    ordered: &mut Vec<&'a Asset>,
) -> Result<(), String> {
    if complete.contains(asset.id.as_str()) {
        return Ok(());
    }
    if !active.insert(&asset.id) {
        return Err(format!("Authority dependency cycle at {}", asset.id));
    }
    let mut edges = BTreeSet::new();
    for dependency in &asset.depends_on {
        if !edges.insert(dependency) {
            return Err(format!("Duplicate authority dependency of {}", asset.id));
        }
        let dependency = *catalog.get(dependency.as_str()).ok_or_else(|| {
            format!(
                "Missing authority dependency for {}: {dependency}",
                asset.id
            )
        })?;
        if asset.active()
            && (!dependency.active()
                || (!dependency.repositories.is_empty()
                    && (asset.repositories.is_empty()
                        || asset
                            .repositories
                            .iter()
                            .any(|repo| !dependency.assigned(repo))))
                || (!asset.kind.manual() && dependency.kind.manual()))
        {
            return Err(format!(
                "Ineligible authority dependency: {} -> {}",
                asset.id, dependency.id
            ));
        }
        visit(dependency, catalog, active, complete, ordered)?;
    }
    active.remove(asset.id.as_str());
    complete.insert(&asset.id);
    ordered.push(asset);
    Ok(())
}
