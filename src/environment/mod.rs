//! Project-local environment selection, native pins and reproducible locks.

mod native;
#[cfg(test)]
mod tests;
mod version;

use crate::tooling::known_tool;
use native::Inventory;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{ErrorKind, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const CONFIG: &str = "cnr.toml";
const LOCK: &str = "cnr.lock";
const SCHEMA: u32 = 1;
const DEFAULT_PROFILE: &str = "default";
const FILE_LIMIT: u64 = 4 * 1024 * 1024;
static STAGE: AtomicU64 = AtomicU64::new(0);
type Pins = BTreeMap<String, String>;

pub struct Project {
    root: PathBuf,
}

pub struct ProjectLock(File);

impl Drop for ProjectLock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    schema: u32,
    profile: String,
    #[serde(default)]
    tools: Pins,
    #[serde(default)]
    profiles: BTreeMap<String, Pins>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema: SCHEMA,
            profile: DEFAULT_PROFILE.into(),
            tools: Pins::new(),
            profiles: BTreeMap::new(),
        }
    }
}

impl Config {
    fn validate(&self) -> Result<(), String> {
        if self.schema != SCHEMA {
            return Err("Unsupported CNR environment schema".into());
        }
        if self.profiles.len() > 64 || !valid_name(&self.profile) {
            return Err("Invalid CNR environment profiles".into());
        }
        if self.profile != DEFAULT_PROFILE && !self.profiles.contains_key(&self.profile) {
            return Err("The active environment profile does not exist".into());
        }
        for profile in self.profiles.keys() {
            if !valid_name(profile) {
                return Err("Invalid environment profile name".into());
            }
        }
        for (tool, version) in self.tools.iter().chain(self.profiles.values().flatten()) {
            validate_pin(tool, version)?;
        }
        Ok(())
    }

    fn effective(&self) -> Pins {
        let mut pins = self.tools.clone();
        if let Some(active) = self.profiles.get(&self.profile) {
            pins.extend(active.clone());
        }
        pins
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnvironmentLock {
    schema: u32,
    profile: String,
    versions: Pins,
    native_inputs: BTreeMap<String, String>,
}

struct Resolution {
    lock: EnvironmentLock,
    inventory: Inventory,
    unresolved: Vec<String>,
}

impl Resolution {
    fn exact(&self) -> Result<(), String> {
        if self.unresolved.is_empty() {
            return Ok(());
        }
        Err(format!(
            "Exact versions are required for {}. Review native pins and use cnr use TOOL@VERSION.",
            self.unresolved.join(", ")
        ))
    }
}

impl Project {
    pub fn open(path: &Path) -> Result<Self, String> {
        let path = std::path::absolute(path).map_err(|_| "Cannot resolve the project path")?;
        check_directory_chain(&path)?;
        let root = path
            .canonicalize()
            .map_err(|_| "Cannot resolve the project path")?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn lock(&self) -> Result<ProjectLock, String> {
        check_directory_chain(&self.root)?;
        let state = self.root.join(".cnr");
        match fs::create_dir(&state) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(_) => return Err("Cannot create the project state directory".into()),
        }
        check_directory_chain(&state)?;
        let path = state.join("environment.lock");
        ordinary_file(&path)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|_| "Cannot open the project environment lock")?;
        file.try_lock()
            .map_err(|_| "The project environment is busy")?;
        Ok(ProjectLock(file))
    }

    pub fn selected(&self) -> Result<Pins, String> {
        Ok(self
            .resolve(&self.config()?.unwrap_or_default())?
            .lock
            .versions)
    }

    pub fn selected_for_sync(&self) -> Result<Pins, String> {
        let resolution = self.resolve(&self.config()?.unwrap_or_default())?;
        resolution.exact()?;
        Ok(resolution.lock.versions)
    }

    pub fn selected_locked(&self) -> Result<Pins, String> {
        let resolution = self.resolve(&self.config()?.unwrap_or_default())?;
        resolution.exact()?;
        let lock = self
            .read_lock()?
            .ok_or("cnr.lock is missing; review the plan and run cnr env lock")?;
        if lock != resolution.lock {
            return Err(
                "cnr.lock differs from the current environment; review before refreshing it".into(),
            );
        }
        Ok(lock.versions)
    }

    pub fn references(&self, tool: &str, version: &str) -> Result<bool, String> {
        validate_pin(tool, version)?;
        let config = self.config()?.unwrap_or_default();
        let referenced = |pins: &Pins| pins.get(tool).is_some_and(|pin| pin == version);
        if referenced(&config.tools) || config.profiles.values().any(referenced) {
            return Ok(true);
        }
        if let Some(lock) = self.read_lock()?
            && referenced(&lock.versions)
        {
            return Ok(true);
        }
        let inventory = Inventory::read(self)?;
        Ok(inventory
            .requirements
            .get(tool)
            .is_some_and(|requirements| {
                let version = version::exact(version).expect("validated version");
                requirements
                    .iter()
                    .any(|item| item.matches(version) != Some(false))
            }))
    }

    fn config(&self) -> Result<Option<Config>, String> {
        read_optional(&self.root.join(CONFIG))?
            .map(|data| {
                let config: Config = toml::from_str(&data).map_err(|_| "Invalid cnr.toml")?;
                config.validate()?;
                Ok(config)
            })
            .transpose()
    }

    fn read_lock(&self) -> Result<Option<EnvironmentLock>, String> {
        read_optional(&self.root.join(LOCK))?
            .map(|data| {
                let lock: EnvironmentLock =
                    toml::from_str(&data).map_err(|_| "Invalid cnr.lock")?;
                if lock.schema != SCHEMA || !valid_name(&lock.profile) {
                    return Err("Unsupported cnr.lock schema or profile".into());
                }
                for (tool, version) in &lock.versions {
                    validate_pin(tool, version)?;
                }
                Ok(lock)
            })
            .transpose()
    }

    fn resolve(&self, config: &Config) -> Result<Resolution, String> {
        config.validate()?;
        let inventory = Inventory::read(self)?;
        let (versions, unresolved) = inventory.resolve(config.effective())?;
        let lock = EnvironmentLock {
            schema: SCHEMA,
            profile: config.profile.clone(),
            versions,
            native_inputs: inventory.files.clone(),
        };
        Ok(Resolution {
            lock,
            inventory,
            unresolved,
        })
    }

    fn write(&self, name: &str, value: &impl Serialize) -> Result<(), String> {
        check_directory_chain(&self.root)?;
        let path = self.root.join(name);
        ordinary_file(&path)?;
        let data = toml::to_string_pretty(value).map_err(|_| format!("Cannot serialize {name}"))?;
        if data.len() as u64 > FILE_LIMIT {
            return Err(format!("{name} exceeds its size limit"));
        }
        let stage = self.root.join(format!(
            ".cnr-stage-{}-{}",
            std::process::id(),
            STAGE.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stage)
            .map_err(|_| format!("Cannot stage {name}"))?;
        let result = file
            .write_all(data.as_bytes())
            .and_then(|_| file.sync_all());
        drop(file);
        let publish = || {
            result.map_err(|_| format!("Cannot sync staged {name}"))?;
            check_directory_chain(&self.root)?;
            ordinary_file(&path)?;
            fs::rename(&stage, &path).map_err(|_| format!("Cannot publish {name} atomically"))
        };
        if let Err(reason) = publish() {
            if check_directory_chain(&self.root).is_ok() {
                let _ = fs::remove_file(&stage);
            }
            return Err(reason);
        }
        Ok(())
    }
}

fn redirected(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return true;
        }
    }
    meta.file_type().is_symlink()
}

fn check_directory_chain(path: &Path) -> Result<(), String> {
    for part in path.ancestors() {
        let metadata =
            fs::symlink_metadata(part).map_err(|_| "Cannot inspect the project directory")?;
        if !metadata.is_dir() || redirected(&metadata) {
            return Err(
                "Project paths must use ordinary directories without links or reparse points"
                    .into(),
            );
        }
    }
    Ok(())
}

fn ordinary_file(path: &Path) -> Result<bool, String> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("environment file");
    match fs::symlink_metadata(path) {
        Ok(metadata)
            if metadata.is_file() && !redirected(&metadata) && metadata.len() <= FILE_LIMIT =>
        {
            Ok(true)
        }
        Ok(_) => Err(format!(
            "{name} must be an ordinary file no larger than 4 MiB"
        )),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(_) => Err(format!("Cannot inspect {name}")),
    }
}

fn read_optional(path: &Path) -> Result<Option<String>, String> {
    if !ordinary_file(path)? {
        return Ok(None);
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("environment file");
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(FILE_LIMIT + 1).read_to_end(&mut bytes))
        .map_err(|_| format!("Cannot read {name}"))?;
    if bytes.len() as u64 > FILE_LIMIT {
        return Err(format!("{name} exceeds its size limit"));
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| format!("{name} is not UTF-8"))
}

fn valid_name(name: &str) -> bool {
    (1..=48).contains(&name.len())
        && name.as_bytes()[0].is_ascii_alphanumeric()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
}

fn validate_pin(tool: &str, value: &str) -> Result<(), String> {
    if !known_tool(tool) {
        return Err("Unknown CNR tool ID".into());
    }
    if value.starts_with('v') || value.len() > 48 || version::exact(value).is_none() {
        return Err("Use an exact X.Y.Z version".into());
    }
    Ok(())
}

fn config_value(config: &Config, key: &str) -> Result<Value, String> {
    let fields = key.split('.').collect::<Vec<_>>();
    match fields.as_slice() {
        ["profile"] => Ok(json!(config.profile)),
        ["tools", tool] => Ok(json!(config.tools.get(*tool))),
        ["profiles", name, tool] => Ok(json!(
            config.profiles.get(*name).and_then(|pins| pins.get(*tool))
        )),
        _ => Err("Use profile, tools.TOOL, or profiles.NAME.TOOL".into()),
    }
}

fn set_config(config: &mut Config, key: &str, value: &str) -> Result<(), String> {
    let fields = key.split('.').collect::<Vec<_>>();
    match fields.as_slice() {
        ["profile"] => config.profile = value.to_owned(),
        ["tools", tool] => {
            validate_pin(tool, value)?;
            config.tools.insert((*tool).into(), value.into());
        }
        ["profiles", name, tool] => {
            validate_pin(tool, value)?;
            if !valid_name(name) {
                return Err("Invalid environment profile name".into());
            }
            config
                .profiles
                .entry((*name).into())
                .or_default()
                .insert((*tool).into(), value.into());
        }
        _ => return Err("Use profile, tools.TOOL, or profiles.NAME.TOOL".into()),
    }
    Ok(())
}

fn unset_config(config: &mut Config, key: &str) -> Result<(), String> {
    let fields = key.split('.').collect::<Vec<_>>();
    match fields.as_slice() {
        ["tools", tool] if known_tool(tool) => {
            config.tools.remove(*tool);
        }
        ["profiles", name, tool] if valid_name(name) && known_tool(tool) => {
            if let Some(pins) = config.profiles.get_mut(*name) {
                pins.remove(*tool);
            }
        }
        _ => {
            return Err(
                "Use tools.TOOL or profiles.NAME.TOOL to remove a version selection".into(),
            );
        }
    }
    Ok(())
}

fn import(project: &Project, path: &str) -> Result<Config, String> {
    let path = std::path::absolute(project.root().join(path)).map_err(|_| "Invalid import path")?;
    check_directory_chain(path.parent().ok_or("Invalid import path")?)?;
    let input = read_optional(&path)?.ok_or("Environment import file does not exist")?;
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Export {
        Manifest(Config),
        Report { manifest: Config },
    }
    let (Export::Manifest(config) | Export::Report { manifest: config }) =
        serde_json::from_str(&input).map_err(|_| "Invalid CNR environment export")?;
    config.validate()?;
    Ok(config)
}

pub fn dispatch(
    project: &Project,
    words: &[String],
    locked: bool,
    dry_run: bool,
) -> Result<Value, String> {
    let words = words.iter().map(String::as_str).collect::<Vec<_>>();
    let writes = matches!(
        words.as_slice(),
        ["init"]
            | ["use" | "pin", _]
            | ["config", "set", _, _]
            | ["config", "unset", _]
            | ["env", "lock"]
            | ["env", "import", _]
    );
    if locked {
        if writes {
            return Err("--locked cannot be combined with a project environment change".into());
        }
        project.selected_locked()?;
    }
    let _guard = (writes && !dry_run).then(|| project.lock()).transpose()?;
    let current = project.config()?;
    let mut config = current.clone().unwrap_or_default();
    let resolution = match words.as_slice() {
        ["init"] => {
            let resolution = project.resolve(&config)?;
            let changed = current.is_none();
            if changed && !dry_run {
                project.write(CONFIG, &config)?;
            }
            return Ok(
                json!({"status":"PASS","project":project.root(),"config":CONFIG,
                "changed":changed && !dry_run, "dry_run":dry_run, "versions":resolution.lock.versions,
                "unresolved":resolution.unresolved}),
            );
        }
        ["use" | "pin", selection] => {
            let (tool, version) = selection.split_once('@').ok_or("Use TOOL@VERSION")?;
            validate_pin(tool, version)?;
            if words[0] == "pin" {
                config.tools.insert(tool.into(), version.into());
                if let Some(active) = config.profiles.get_mut(&config.profile) {
                    active.remove(tool);
                }
            } else {
                config
                    .profiles
                    .entry(config.profile.clone())
                    .or_default()
                    .insert(tool.into(), version.into());
            }
            project.resolve(&config)?
        }
        ["config", "set", key, value] => {
            set_config(&mut config, key, value)?;
            project.resolve(&config)?
        }
        ["config", "unset", key] => {
            unset_config(&mut config, key)?;
            project.resolve(&config)?
        }
        ["config"] | ["config", "show"] => {
            return Ok(json!({"status":"PASS","project":project.root(),"manifest":config}));
        }
        ["config", "get", key] => {
            return Ok(json!({"status":"PASS","value":config_value(&config, key)?}));
        }
        ["env", "profiles"] => {
            let profiles: std::collections::BTreeSet<_> = config
                .profiles
                .keys()
                .map(String::as_str)
                .chain([DEFAULT_PROFILE])
                .collect();
            return Ok(json!({"status":"PASS","active":config.profile,"profiles":profiles}));
        }
        ["env", "plan"] => {
            let resolution = project.resolve(&config)?;
            let lock = match project.read_lock()? {
                None => "ABSENT",
                Some(lock) if lock == resolution.lock && resolution.unresolved.is_empty() => {
                    "CURRENT"
                }
                Some(_) => "STALE",
            };
            return Ok(
                json!({"status":"PASS","project":project.root(),"profile":config.profile,
                "versions":resolution.lock.versions,"unresolved":resolution.unresolved,"lock":lock,
                "native":resolution.inventory.requirements,"native_inputs":resolution.inventory.files}),
            );
        }
        ["env", "lock"] => {
            let resolution = project.resolve(&config)?;
            resolution.exact()?;
            let changed = project.read_lock()?.as_ref() != Some(&resolution.lock);
            if changed && !dry_run {
                project.write(LOCK, &resolution.lock)?;
            }
            return Ok(
                json!({"status":"PASS","project":project.root(),"lock":LOCK,"dry_run":dry_run,
                "changed":changed && !dry_run,"profile":config.profile,"versions":resolution.lock.versions}),
            );
        }
        ["env", "export"] => return Ok(json!({"status":"PASS","manifest":config})),
        ["env", "import", path] => {
            let imported = import(project, path)?;
            if current
                .as_ref()
                .is_some_and(|existing| existing != &imported && existing != &Config::default())
            {
                return Err("The project already has a different environment. Import into a new project or apply individual settings.".into());
            }
            config = imported;
            project.resolve(&config)?
        }
        _ => return Err("Unknown environment command or unexpected arguments".into()),
    };
    let changed = current
        .as_ref()
        .map_or(config != Config::default(), |existing| existing != &config);
    if changed && !dry_run {
        project.write(CONFIG, &config)?;
    }
    Ok(
        json!({"status":"PASS","project":project.root(),"profile":config.profile,
        "changed":changed && !dry_run,"dry_run":dry_run,"versions":resolution.lock.versions,
        "unresolved":resolution.unresolved}),
    )
}
