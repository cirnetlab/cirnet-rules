use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{self, File, Metadata, OpenOptions},
    io::{ErrorKind, Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const RECORD_LIMIT: u64 = 16 * 1024;
const ENTRY_LIMIT: usize = 2048;
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Identity {
    kind: String,
    name: String,
    version: Option<String>,
}

impl Identity {
    pub(super) fn tool(name: &str, version: &str) -> Self {
        Self {
            kind: "tool".into(),
            name: name.into(),
            version: Some(version.into()),
        }
    }

    pub(super) fn venv(name: &str) -> Self {
        Self {
            kind: "venv".into(),
            name: name.into(),
            version: None,
        }
    }

    pub(super) fn cache() -> Self {
        Self {
            kind: "cache".into(),
            name: "uv".into(),
            version: None,
        }
    }

    fn relative(&self) -> PathBuf {
        match self.kind.as_str() {
            "tool" => Path::new(".cnr/tools")
                .join(&self.name)
                .join(self.version.as_deref().expect("tool version")),
            "venv" => Path::new(".cnr/venvs").join(&self.name),
            "cache" => PathBuf::from(".cnr/cache"),
            _ => unreachable!("locally constructed identity kind"),
        }
    }

    pub(super) fn path(&self, root: &Path) -> PathBuf {
        root.join(self.relative())
    }

    fn valid(&self) -> bool {
        match self.kind.as_str() {
            "tool" => {
                super::catalog::get(&self.name, None)
                    .is_ok_and(|tool| tool.id == self.name && tool.supported())
                    && self
                        .version
                        .as_deref()
                        .is_some_and(|version| super::exact_version(version).ok() == Some(version))
            }
            "venv" => super::venv_name(&self.name).is_ok() && self.version.is_none(),
            "cache" => *self == Self::cache(),
            _ => false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Intent {
    schema: u8,
    identity: Identity,
    python_version: Option<String>,
    python: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Receipt {
    schema: u8,
    identity: Identity,
    executable: PathBuf,
}

#[derive(Debug)]
pub(super) enum State {
    Absent,
    Interrupted,
    Ready(PathBuf),
    OwnedCache,
}

pub(super) fn ordinary(meta: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return false;
        }
    }
    !meta.file_type().is_symlink()
}

fn component(path: &Path) -> Result<(), String> {
    if path.components().all(
        |part| matches!(part, Component::Normal(name) if !name.to_string_lossy().contains(':')),
    ) {
        Ok(())
    } else {
        Err("Unsafe relative provider state path".into())
    }
}

fn make_dir(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new().mode(0o700).create(path)
    }
    #[cfg(not(unix))]
    {
        fs::create_dir(path)
    }
}

/// Walk each project-relative component without accepting links or junctions.
pub(super) fn directory(
    root: &Path,
    relative: &Path,
    create: bool,
) -> Result<Option<PathBuf>, String> {
    component(relative)?;
    let mut path = root.to_path_buf();
    for part in relative.components() {
        path.push(part.as_os_str());
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_dir() && ordinary(&meta) => {}
            Ok(_) => return Err("Provider state path is not an ordinary directory".into()),
            Err(error) if error.kind() == ErrorKind::NotFound && create => {
                match make_dir(&path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
                    Err(error) => {
                        return Err(format!(
                            "Could not create provider directory ({:?})",
                            error.kind()
                        ));
                    }
                }
                let meta =
                    fs::symlink_metadata(&path).map_err(|_| "Provider directory disappeared")?;
                if !meta.is_dir() || !ordinary(&meta) {
                    return Err("Provider directory was redirected".into());
                }
            }
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err("Could not inspect the provider state path".into()),
        }
    }
    Ok(Some(path))
}

fn file(path: &Path) -> Result<Option<File>, String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_file() && ordinary(&meta) && meta.len() <= RECORD_LIMIT => {
            File::open(path)
                .map(Some)
                .map_err(|_| "Could not open provider record".into())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        _ => Err("Provider record is not an ordinary bounded file".into()),
    }
}

fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
    let Some(input) = file(path)? else {
        return Ok(None);
    };
    let mut bytes = Vec::new();
    input
        .take(RECORD_LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "Could not read provider record")?;
    if bytes.len() as u64 > RECORD_LIMIT {
        return Err("Provider record exceeds the size limit".into());
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|_| "Malformed provider record".into())
}

fn stamp() -> String {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!(
        "{time:032x}-{:08x}-{:016x}",
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

fn write(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let stage = path.with_file_name(format!(".{}.pending", stamp()));
    let bytes = serde_json::to_vec(value).map_err(|_| "Could not serialize provider record")?;
    if bytes.len() as u64 > RECORD_LIMIT {
        return Err("Provider record exceeds the size limit".into());
    }
    let result = (|| {
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&stage)?;
        output.write_all(&bytes)?;
        output.sync_all()?;
        drop(output);
        fs::rename(&stage, path)
    })();
    if let Err(error) = result {
        let _ = fs::remove_file(&stage);
        return Err(format!(
            "Could not publish provider record ({:?})",
            error.kind()
        ));
    }
    Ok(())
}

fn intent(target: &Path, id: &Identity) -> Result<Intent, String> {
    let record: Intent = read(&target.join("intent.json"))?
        .ok_or("Unmanaged directory. CNR will not adopt or remove it.")?;
    if record.schema != 1 || record.identity != *id {
        return Err("Provider ownership record does not match the selected resource".into());
    }
    Ok(record)
}

/// Reserve with an intent already in place before the native provider can run.
pub(super) fn reserve(root: &Path, id: &Identity) -> Result<PathBuf, String> {
    reserve_with_python(root, id, None, None)
}

pub(super) fn reserve_with_python(
    root: &Path,
    id: &Identity,
    python_version: Option<&str>,
    python: Option<&Path>,
) -> Result<PathBuf, String> {
    let relative = id.relative();
    let parent = directory(root, relative.parent().expect("provider parent"), true)?
        .expect("provider parent created");
    let target = id.path(root);
    match fs::symlink_metadata(&target) {
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        _ => return Err("Provider resource already exists or cannot be inspected".into()),
    }
    let stage = parent.join(format!(".{}.reserve", stamp()));
    make_dir(&stage).map_err(|_| "Could not reserve a new provider directory")?;
    let record = Intent {
        schema: 1,
        identity: id.clone(),
        python_version: python_version.map(str::to_owned),
        python: python.map(Path::to_owned),
    };
    let result = write(&stage.join("intent.json"), &record).and_then(|()| {
        fs::rename(&stage, &target).map_err(|_| "Could not publish provider intent".into())
    });
    if result.is_err() {
        let _ = fs::remove_dir_all(&stage);
    }
    result?;
    Ok(target)
}

pub(super) fn checked_executable(target: &Path, path: &Path) -> Result<PathBuf, String> {
    let inside = path
        .canonicalize()
        .map_err(|_| "Managed executable is unavailable")?;
    if !inside.starts_with(target) || !inside.is_file() {
        return Err("Provider executable resolves outside its owned resource".into());
    }
    Ok(inside)
}

pub(super) fn publish(root: &Path, id: &Identity, path: &Path) -> Result<PathBuf, String> {
    let target = directory(root, &id.relative(), false)?.ok_or("Provider intent is missing")?;
    let owner = intent(&target, id)?;
    let executable = receipt_executable(&target, path, &owner)?;
    // Retain the virtual environment's own interpreter path, since CPython locates
    // pyvenv.cfg relative to the invoked path rather than its symlink target.
    let saved = path
        .strip_prefix(&target)
        .map_err(|_| "Provider executable is outside its receipt")?;
    component(saved)?;
    write(
        &target.join("installed.json"),
        &Receipt {
            schema: 1,
            identity: id.clone(),
            executable: saved.into(),
        },
    )?;
    Ok(if id.kind == "venv" {
        path.to_owned()
    } else {
        executable
    })
}

pub(super) fn state(root: &Path, id: &Identity) -> Result<State, String> {
    let Some(target) = directory(root, &id.relative(), false)? else {
        return Ok(State::Absent);
    };
    let owner = intent(&target, id)?;
    if id.kind == "cache" {
        return Ok(State::OwnedCache);
    }
    let Some(record): Option<Receipt> = read(&target.join("installed.json"))? else {
        return Ok(State::Interrupted);
    };
    if record.schema != 1 || record.identity != *id {
        return Err("Provider installation receipt does not match the selected resource".into());
    }
    component(&record.executable)?;
    let path = target.join(record.executable);
    let canonical = receipt_executable(&target, &path, &owner)?;
    Ok(State::Ready(if id.kind == "venv" {
        path
    } else {
        canonical
    }))
}

fn receipt_executable(target: &Path, path: &Path, owner: &Intent) -> Result<PathBuf, String> {
    if owner.identity.kind != "venv" {
        return checked_executable(target, path);
    }
    let relative = path
        .strip_prefix(target)
        .map_err(|_| "Invalid virtual environment interpreter")?;
    let expected = Path::new("env")
        .join(if cfg!(windows) { "Scripts" } else { "bin" })
        .join(super::providers::native_name("python"));
    if relative != expected {
        return Err("Unexpected virtual environment interpreter path".into());
    }
    directory(target, expected.parent().expect("venv bin"), false)?
        .ok_or("Virtual environment bin is missing")?;
    let executable = path
        .canonicalize()
        .map_err(|_| "Virtual environment interpreter is unavailable")?;
    let base = owner
        .python
        .as_deref()
        .ok_or("Virtual environment base interpreter is missing")?;
    if !base.is_file()
        || !executable.is_file()
        || (!executable.starts_with(target) && executable != base)
    {
        return Err("Virtual environment no longer resolves to its recorded interpreter".into());
    }
    Ok(executable)
}

pub(super) fn remove(root: &Path, id: &Identity) -> Result<bool, String> {
    let Some(target) = directory(root, &id.relative(), false)? else {
        return Ok(false);
    };
    intent(&target, id)?;
    let trash = directory(root, Path::new(".cnr/trash"), true)?.expect("provider trash created");
    let token = stamp();
    let retired = trash.join(&token);
    let journal = trash.join(format!("{token}.json"));
    // Keep the retirement receipt outside the resource so an interrupted
    // recursive cleanup cannot erase the proof needed to resume it.
    write(&journal, id)?;
    fs::rename(&target, &retired)
        .map_err(|_| "Could not retire the owned resource. It may still be in use.")?;
    // No archive paths are executed. std::fs::remove_dir_all does not follow symlinks.
    fs::remove_dir_all(&retired)
        .map_err(|_| "Resource retired; retry cleanup with cnr cache clean")?;
    fs::remove_file(&journal)
        .map_err(|_| "Resource removed; retry journal cleanup with cnr cache clean")?;
    Ok(true)
}

pub(super) fn owned(root: &Path, id: &Identity) -> Result<bool, String> {
    let Some(target) = directory(root, &id.relative(), false)? else {
        return Ok(false);
    };
    intent(&target, id)?;
    Ok(true)
}

pub(super) fn clean_trash(root: &Path) -> Result<usize, String> {
    let Some(trash) = directory(root, Path::new(".cnr/trash"), false)? else {
        return Ok(0);
    };
    let mut cleaned = 0;
    for journal in entries(&trash)? {
        let Some(name) = journal.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };
        if journal.extension().is_none_or(|ext| ext != "json")
            || name.len() != 58
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
            || !read::<Identity>(&journal)?.is_some_and(|identity| identity.valid())
        {
            continue;
        }
        let retired = trash.join(name);
        match fs::symlink_metadata(&retired) {
            Ok(meta) if meta.is_dir() && ordinary(&meta) => {
                fs::remove_dir_all(&retired)
                    .map_err(|_| "Retired provider resource is still in use")?;
                cleaned += 1;
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            _ => return Err("Retired provider path is not an ordinary directory".into()),
        }
        fs::remove_file(&journal).map_err(|_| "Could not remove the retired provider journal")?;
    }
    Ok(cleaned)
}

pub(super) fn entries(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir).map_err(|_| "Could not list provider state")? {
        if paths.len() == ENTRY_LIMIT {
            return Err("Provider directory exceeds the entry limit".into());
        }
        paths.push(
            entry
                .map_err(|_| "Could not read provider directory entry")?
                .path(),
        );
    }
    paths.sort();
    Ok(paths)
}

pub(super) fn references_python(root: &Path, version: &str) -> Result<bool, String> {
    let Some(venvs) = directory(root, Path::new(".cnr/venvs"), false)? else {
        return Ok(false);
    };
    for target in entries(&venvs)? {
        let Some(name) = target.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if super::venv_name(name).is_err() {
            continue;
        }
        let id = Identity::venv(name);
        let Some(target) = directory(root, &id.relative(), false)? else {
            continue;
        };
        if intent(&target, &id)?.python_version.as_deref() == Some(version) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn exclusive(root: &Path) -> Result<File, String> {
    let state = directory(root, Path::new(".cnr"), true)?.expect("provider state created");
    let path = state.join("operations.lock");
    if let Ok(meta) = fs::symlink_metadata(&path)
        && (!meta.is_file() || !ordinary(&meta))
    {
        return Err("Unsafe provider lock file".into());
    }
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|_| "Could not open provider lock")?;
    lock.try_lock()
        .map_err(|_| "Another CNR provider operation or managed job is active")?;
    Ok(lock)
}

pub(super) fn shared(root: &Path) -> Result<Option<File>, String> {
    let Some(state) = directory(root, Path::new(".cnr"), false)? else {
        return Ok(None);
    };
    let Some(lock) = file(&state.join("operations.lock"))? else {
        return Ok(None);
    };
    lock.try_lock_shared()
        .map_err(|_| "A CNR provider operation is active")?;
    Ok(Some(lock))
}

pub(super) fn audit(
    root: &Path,
    id: Option<&Identity>,
    action: &str,
    status: &str,
) -> Result<(), String> {
    let dir = directory(root, Path::new(".cnr/audit"), true)?.expect("audit directory created");
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    write(
        &dir.join(format!("{}.json", stamp())),
        &json!({
            "schema": 1,
            "action": action,
            "resource": id,
            "status": status,
            "timestamp_ms": timestamp_ms
        }),
    )
}

pub(super) fn audit_list(root: &Path) -> Result<Vec<Value>, String> {
    let Some(dir) = directory(root, Path::new(".cnr/audit"), false)? else {
        return Ok(Vec::new());
    };
    let mut records = Vec::new();
    for entry in entries(&dir)?
        .into_iter()
        .rev()
        .filter(|entry| entry.extension().is_some_and(|ext| ext == "json"))
        .take(100)
    {
        if let Some(event) = read::<AuditRecord>(&entry)? {
            if event.schema != 1
                || !matches!(
                    event.action.as_str(),
                    "install"
                        | "update"
                        | "remove"
                        | "venv.create"
                        | "venv.remove"
                        | "venv.run"
                        | "cache.clean"
                )
                || !matches!(
                    event.status.as_str(),
                    "STARTED" | "PASS" | "FAIL" | "BLOCKED" | "RECOVERY_REQUIRED"
                )
                || event
                    .resource
                    .as_ref()
                    .is_some_and(|identity| !identity.valid())
            {
                return Err("Invalid provider audit event".into());
            }
            records.push(serde_json::to_value(event).map_err(|_| "Could not encode audit event")?);
        }
    }
    Ok(records)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuditRecord {
    schema: u8,
    action: String,
    resource: Option<Identity>,
    status: String,
    timestamp_ms: u64,
}
