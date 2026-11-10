use super::{Options, Selection, resolve};
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub(super) struct Plan {
    pub engine: &'static str,
    pub steps: Vec<Step>,
}

pub(super) struct Step {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub env: Vec<(&'static str, OsString)>,
    pub path_tools: Vec<PathBuf>,
    visible_args: usize,
}

impl Step {
    fn new(program: PathBuf, args: impl IntoIterator<Item = impl Into<OsString>>) -> Self {
        let args: Vec<_> = args.into_iter().map(Into::into).collect();
        Self {
            path_tools: vec![program.clone()],
            program,
            visible_args: args.len(),
            args,
            env: Vec::new(),
        }
    }

    fn arg(&mut self, value: impl Into<OsString>) {
        self.args.push(value.into());
        self.visible_args += 1;
    }

    fn tail(&mut self, args: &[String]) {
        self.args.extend(args.iter().map(OsString::from));
    }
}

impl Plan {
    pub(super) fn preview(&self, root: &Path, options: &Options) -> Value {
        let steps: Vec<_> = self
            .steps
            .iter()
            .map(|step| {
                json!({
                    "executable":step.program, "arguments":step.args.iter().take(step.visible_args)
                        .map(|arg| arg.to_string_lossy()).collect::<Vec<_>>(),
                    "additional_arguments":step.args.len() - step.visible_args,
                    "environment_keys":step.env.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
                })
            })
            .collect();
        json!({
            "engine":self.engine,"project":root,"scope":"project","steps":steps,
            "timeout_ms":options.timeout_ms,"offline":options.offline,"locked":options.locked,
            "execution":"Local operator process. Project hooks and scripts have host permissions.",
            "offline_scope":"Dependency resolution options of the selected native tool",
        })
    }
}

const ENGINES: [(&str, &str); 4] = [
    ("rust", "Cargo.toml"),
    ("node", "package.json"),
    ("python", "pyproject.toml"),
    ("cmake", "CMakeLists.txt"),
];

fn manifest(root: &Path, name: &str) -> bool {
    root.join(name)
        .symlink_metadata()
        .is_ok_and(|meta| meta.is_file() && !redirected(&meta))
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

fn engine(root: &Path, first: Option<&str>, package: bool) -> Result<&'static str, String> {
    let mut candidates = ENGINES
        .into_iter()
        .filter(|(engine, _)| !package || *engine != "cmake")
        .filter(|(_, name)| manifest(root, name));
    if let Some(first) = first {
        return candidates
            .find(|(engine, _)| *engine == first)
            .map(|(engine, _)| engine)
            .ok_or_else(|| {
                "Requested engine is unsupported or its project manifest is missing".into()
            });
    }
    let (name, _) = candidates
        .next()
        .ok_or("No supported native project manifest was found")?;
    if candidates.next().is_some() {
        return Err("Multiple native project manifests match. Select rust, node, python or cmake explicitly".into());
    }
    Ok(name)
}

pub(super) fn create(
    root: &Path,
    words: &[String],
    selected: &Selection,
    options: &Options,
) -> Result<Plan, String> {
    let (action, words) = words
        .split_first()
        .ok_or("A project operation is required")?;
    if action == "exec" {
        if options.offline || options.locked {
            return Err(
                "An arbitrary executable cannot enforce offline or locked dependency resolution"
                    .into(),
            );
        }
        let [separator, program, args @ ..] = words else {
            return Err("Use exec -- EXECUTABLE [ARG ...]".into());
        };
        if separator != "--" {
            return Err("Use exec -- EXECUTABLE [ARG ...]".into());
        }
        let mut step = Step::new(resolve::explicit(root, program)?, [] as [&str; 0]);
        step.tail(args);
        return Ok(Plan {
            engine: "exec",
            steps: vec![step],
        });
    }
    if action == "package" {
        let (engine, words) = if let Some(name) = words
            .first()
            .filter(|value| ENGINES.iter().any(|(engine, _)| engine == value))
        {
            (engine(root, Some(name), true)?, &words[1..])
        } else {
            (engine(root, None, true)?, words)
        };
        let steps = vec![package(root, engine, words, selected, options)?];
        return Ok(Plan { engine, steps });
    }
    if !matches!(action.as_str(), "build" | "run" | "test") {
        return Err("Unsupported execution action".into());
    }
    let separator = words
        .iter()
        .position(|arg| arg == "--")
        .unwrap_or(words.len());
    let extra = words.get(separator + 1..).unwrap_or_default();
    let engine = match &words[..separator] {
        [] => engine(root, None, false)?,
        [engine_name] => engine(root, Some(engine_name), false)?,
        _ => return Err("Select at most one engine before --".into()),
    };
    require_pin(selected, engine, options)?;
    let steps = match engine {
        "rust" => vec![cargo_action(root, action, extra, selected, options)?],
        "node" => vec![node_action(root, action, extra, selected, options)?],
        "python" => vec![python_action(root, action, extra, selected, options)?],
        "cmake" => cmake_action(root, action, extra, selected, options)?,
        _ => unreachable!("engine comes from the manifest table"),
    };
    Ok(Plan { engine, steps })
}

fn require_pin(selected: &Selection, tool: &str, options: &Options) -> Result<(), String> {
    if options.locked && !selected.contains_key(tool) {
        return Err(format!(
            "--locked requires an exact selected {tool} version"
        ));
    }
    Ok(())
}

fn cargo(
    root: &Path,
    action: &str,
    selected: &Selection,
    options: &Options,
) -> Result<Step, String> {
    if options.locked && !manifest(root, "Cargo.lock") {
        return Err("--locked requires an existing Cargo.lock".into());
    }
    let rustc = resolve::selected(root, selected, "rust")?;
    let cargo = resolve::sibling(&rustc, "cargo").or_else(|reason| {
        if selected.contains_key("rust") {
            Err(reason)
        } else {
            resolve::discover(root, "cargo")
        }
    })?;
    let mut step = Step::new(cargo, [action]);
    step.env.push(("RUSTC", rustc.into_os_string()));
    if options.locked {
        step.arg("--locked");
    }
    if options.offline {
        step.arg("--offline");
        step.env.push(("CARGO_NET_OFFLINE", "true".into()));
    }
    Ok(step)
}

fn cargo_action(
    root: &Path,
    action: &str,
    args: &[String],
    selected: &Selection,
    options: &Options,
) -> Result<Step, String> {
    let mut step = cargo(root, action, selected, options)?;
    if !args.is_empty() {
        if action == "run" {
            step.arg("--");
        } else if options.offline || options.locked {
            return Err(
                "Extra Cargo build/test arguments are unavailable in locked or offline mode".into(),
            );
        }
        step.tail(args);
    }
    Ok(step)
}

fn npm(root: &Path, selected: &Selection, options: &Options) -> Result<Step, String> {
    if selected.contains_key("pnpm")
        || selected.contains_key("yarn")
        || ["pnpm-lock.yaml", "yarn.lock"]
            .iter()
            .any(|name| root.join(name).symlink_metadata().is_ok())
    {
        return Err(
            "This project uses a different package manager. The Node package adapter supports npm"
                .into(),
        );
    }
    let node = resolve::selected(root, selected, "node")?;
    let script = resolve::provider_path(&resolve::npm_cli(
        root,
        &node,
        selected.contains_key("node"),
    )?);
    if let Some(version) = selected.get("npm") {
        resolve::verify_version(&node, std::slice::from_ref(&script), version, root)?;
    }
    let mut step = Step::new(node, [script]);
    step.arg("--global=false");
    step.arg("--prefix");
    step.arg(resolve::provider_path(root));
    if options.offline {
        step.arg("--offline");
        step.env.push(("npm_config_offline", "true".into()));
    }
    Ok(step)
}

fn node_action(
    root: &Path,
    action: &str,
    args: &[String],
    selected: &Selection,
    options: &Options,
) -> Result<Step, String> {
    if options.locked {
        return Err("A Node script cannot verify installed dependencies against a lockfile. Use package node sync --locked first".into());
    }
    if action == "run" && !args.is_empty() {
        if options.offline {
            return Err(
                "Direct Node execution cannot enforce offline dependency resolution".into(),
            );
        }
        let node = resolve::selected(root, selected, "node")?;
        let file = local_file(root, &args[0])?;
        let mut step = Step::new(node, [file.as_os_str()]);
        step.tail(&args[1..]);
        return Ok(step);
    }
    if !args.is_empty() {
        return Err(
            "npm scripts use the project's shell; extra argv is available only for direct Node run"
                .into(),
        );
    }
    let mut step = npm(root, selected, options)?;
    step.arg("run");
    step.arg(if action == "run" { "start" } else { action });
    Ok(step)
}

fn uv(root: &Path, selected: &Selection, options: &Options, args: &[&str]) -> Result<Step, String> {
    let python = resolve::selected(root, selected, "python")?;
    let mut step = Step::new(
        resolve::selected(root, selected, "uv")?,
        args.iter().copied(),
    );
    step.arg("--no-python-downloads");
    if options.offline {
        step.arg("--offline");
    }
    step.arg("--python");
    step.arg(python.as_os_str());
    step.path_tools.push(python);
    Ok(step)
}

fn python_action(
    root: &Path,
    action: &str,
    args: &[String],
    selected: &Selection,
    options: &Options,
) -> Result<Step, String> {
    if options.locked {
        return Err("Python build and interpreter execution do not enforce uv.lock. Use package python sync --locked first".into());
    }
    if action == "build" {
        if !args.is_empty() {
            return Err("Python build does not accept extra arguments".into());
        }
        return uv(root, selected, options, &["build"]);
    }
    if options.offline {
        return Err("Direct Python execution cannot enforce offline dependency resolution".into());
    }
    let python = resolve::selected(root, selected, "python")?;
    if action == "test" {
        let mut step = Step::new(python, ["-B", "-m", "pytest"]);
        step.tail(args);
        return Ok(step);
    }
    let (script, args) = args
        .split_first()
        .ok_or("Specify a project script using run python -- SCRIPT [ARG ...]")?;
    let file = local_file(root, script)?;
    let mut step = Step::new(python, [OsString::from("-B"), file.into_os_string()]);
    step.tail(args);
    Ok(step)
}

fn local_file(root: &Path, argument: &str) -> Result<PathBuf, String> {
    let path = Path::new(argument);
    if !resolve::plain_relative(path) {
        return Err("Use a file path relative to the selected project".into());
    }
    let path = root
        .join(path)
        .canonicalize()
        .map_err(|_| "Project file is unavailable")?;
    if !path.starts_with(root) || !path.is_file() {
        return Err("Project file resolves outside the selected project or is not a file".into());
    }
    Ok(path)
}

fn cmake_action(
    root: &Path,
    action: &str,
    args: &[String],
    selected: &Selection,
    options: &Options,
) -> Result<Vec<Step>, String> {
    if options.locked || options.offline {
        return Err(
            "Generic CMake projects cannot enforce a locked or offline dependency policy".into(),
        );
    }
    let mut build = root.to_owned();
    for part in [".cnr", "build", "cmake"] {
        build.push(part);
        match fs::symlink_metadata(&build) {
            Ok(meta) if meta.is_dir() && !redirected(&meta) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            _ => {
                return Err(
                    "CMake build path crosses a redirected or invalid project directory".into(),
                );
            }
        }
    }
    if action == "run" {
        let (name, args) = args
            .split_first()
            .ok_or("Specify a built program using run cmake -- PATH [ARG ...]")?;
        let file = local_file(&build, name)?;
        let mut step = Step::new(resolve::executable(&file)?, [] as [&str; 0]);
        step.tail(args);
        return Ok(vec![step]);
    }
    if !args.is_empty() {
        return Err(
            "CMake configure/build/test uses its managed directory without extra arguments".into(),
        );
    }
    let cmake = resolve::selected(root, selected, "cmake")?;
    let native_build = cmake_path(&build);
    if action == "test" {
        if !manifest(&build, "CMakeCache.txt") {
            return Err("Configure and build the CMake project before testing".into());
        }
        let ctest = resolve::sibling(&cmake, "ctest")?;
        return Ok(vec![Step::new(
            ctest,
            ["--test-dir", &native_build, "--output-on-failure"],
        )]);
    }
    let ninja = resolve::selected(root, selected, "ninja")?;
    let generator = format!("-DCMAKE_MAKE_PROGRAM={}", cmake_path(&ninja));
    let source = cmake_path(root);
    let mut configure = Step::new(
        cmake.clone(),
        [
            "-S",
            &source,
            "-B",
            &native_build,
            "-G",
            "Ninja",
            &generator,
        ],
    );
    configure.path_tools.push(ninja.clone());
    let mut compile = Step::new(cmake, ["--build", &native_build]);
    compile.path_tools.push(ninja);
    Ok(vec![configure, compile])
}

fn cmake_path(path: &Path) -> String {
    let raw = path.to_string_lossy();
    #[cfg(windows)]
    {
        if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
            return format!("//{}", rest.replace('\\', "/"));
        }
        raw.strip_prefix(r"\\?\").unwrap_or(&raw).replace('\\', "/")
    }
    #[cfg(not(windows))]
    raw.into_owned()
}

fn package(
    root: &Path,
    engine: &str,
    words: &[String],
    selected: &Selection,
    options: &Options,
) -> Result<Step, String> {
    let (action, args) = words
        .split_first()
        .ok_or("Use package [rust|node|python] list|add|remove|sync|outdated")?;
    let action = action.as_str();
    let mutation = matches!(action, "add" | "remove");
    if mutation && options.locked {
        return Err(
            "Package changes require permission to update native manifests and lockfiles".into(),
        );
    }
    if !matches!(action, "list" | "sync" | "outdated" | "add" | "remove") {
        return Err("Unsupported package operation".into());
    }
    if mutation {
        let [spec] = args else {
            return Err("Specify one registry package name or version".into());
        };
        if !valid_package(spec, engine) {
            return Err("Only a registry package specifier is allowed. Paths, URLs and native switches are unavailable".into());
        }
    } else if !args.is_empty() {
        return Err("This package operation does not accept additional arguments".into());
    }
    require_pin(selected, engine, options)?;
    let mut step = match engine {
        "rust" => {
            let native = match action {
                "list" => "tree",
                "sync" => "fetch",
                "outdated" => return Err("Cargo has no built-in outdated operation".into()),
                _ => action,
            };
            check_local_target(root, "Cargo.lock", false)?;
            let mut step = cargo(root, native, selected, options)?;
            if action == "list" && !options.locked {
                if !manifest(root, "Cargo.lock") {
                    return Err("Package listing needs an existing Cargo.lock".into());
                }
                step.arg("--locked");
            }
            step
        }
        "node" => {
            if options.locked && action != "sync" {
                return Err("npm lock enforcement is available for package node sync".into());
            }
            require_pin(selected, "npm", options)?;
            check_local_target(root, "node_modules", true)?;
            for file in ["package-lock.json", "npm-shrinkwrap.json"] {
                check_local_target(root, file, false)?;
            }
            let mut step = npm(root, selected, options)?;
            if action == "sync"
                && !(manifest(root, "package-lock.json") || manifest(root, "npm-shrinkwrap.json"))
            {
                return Err("npm ci requires an existing npm lockfile".into());
            }
            step.arg(match action {
                "list" => "ls",
                "sync" => "ci",
                "add" => "install",
                "remove" => "uninstall",
                _ => "outdated",
            });
            step.arg("--ignore-scripts");
            step.arg("--no-audit");
            if action == "list" {
                step.arg("--depth=0");
            }
            step
        }
        "python" => {
            check_local_target(root, "uv.lock", false)?;
            let locked = options.locked || matches!(action, "list" | "outdated");
            if locked && !manifest(root, "uv.lock") {
                return Err("This operation requires an existing uv.lock".into());
            }
            require_pin(selected, "uv", options)?;
            let native = match action {
                "list" | "outdated" => "tree",
                _ => action,
            };
            let mut step = uv(root, selected, options, &[native])?;
            if locked {
                step.arg("--locked");
            }
            if mutation {
                step.arg("--no-sync");
            }
            if action == "outdated" {
                step.arg("--outdated");
            }
            if action == "sync" {
                check_local_target(root, ".venv", true)?;
                step.env.push((
                    "UV_PROJECT_ENVIRONMENT",
                    root.join(".venv").into_os_string(),
                ));
            }
            step
        }
        _ => unreachable!("package excludes CMake"),
    };
    step.tail(args);
    Ok(step)
}

fn check_local_target(root: &Path, name: &str, directory: bool) -> Result<(), String> {
    match root.join(name).symlink_metadata() {
        Ok(meta)
            if !redirected(&meta)
                && if directory {
                    meta.is_dir()
                } else {
                    meta.is_file()
                } =>
        {
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        _ => Err(format!(
            "Refusing a redirected or invalid project target {name}"
        )),
    }
}

fn valid_package(spec: &str, engine: &str) -> bool {
    if spec.is_empty() || spec.len() > 256 || spec.starts_with(['-', '.']) || spec.contains("..") {
        return false;
    }
    let scoped = engine == "node" && spec.starts_with('@');
    let bytes = if scoped { &spec[1..] } else { spec };
    if bytes.contains('/') && (!scoped || bytes.matches('/').count() != 1) {
        return false;
    }
    bytes
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphanumeric)
        && bytes
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"@/._-+~^*<>=!,[]".contains(&b))
}
