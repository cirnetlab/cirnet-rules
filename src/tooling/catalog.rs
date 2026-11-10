use super::providers;
use serde_json::{Value, json};
use std::path::Path;

pub(super) struct Tool {
    pub id: &'static str,
    pub kind: &'static str,
    pub label: &'static str,
    pub programs: &'static [&'static str],
    pub manager: Option<&'static str>,
    pub upstream: &'static str,
}

impl Tool {
    pub fn supported(&self) -> bool {
        self.manager.is_some() && cfg!(any(windows, target_os = "linux", target_os = "macos"))
    }

    pub fn evidence(&self, project: Option<&Path>, probe: bool) -> Value {
        let executable = if probe {
            providers::discover(self, project)
        } else {
            None
        };
        let platform = match self.id {
            "msvc" | "windows-sdk" if !cfg!(windows) => "UNSUPPORTED",
            _ => "SUPPORTED",
        };
        let state = if platform == "UNSUPPORTED" {
            "UNSUPPORTED"
        } else if !probe {
            "CATALOG"
        } else if executable.is_some() {
            "DISCOVERED"
        } else {
            "NOT_FOUND"
        };
        let manager_path = self
            .manager
            .and_then(|manager| providers::find(manager, project));
        json!({
            "tool": self.id,
            "name": self.label,
            "type": self.kind,
            "provider": self.manager.unwrap_or("native"),
            "upstream": self.upstream,
            "platform": std::env::consts::OS,
            "architecture": std::env::consts::ARCH,
            "status": state,
            "native_path": executable.as_ref().map(|found| &found.0),
            "native_version": executable.and_then(|found| found.1),
            "provider_path": manager_path,
            "capabilities": {
                "detect": platform == "SUPPORTED",
                "install_project": self.supported(),
                "update_project": self.supported(),
                "remove_owned_project": self.supported(),
                "offline_install": false,
                "mutate_user": false,
                "mutate_system": false
            }
        })
    }
}

macro_rules! tool {
    ($id:literal, $kind:literal, $label:literal, [$($program:literal),*], $manager:expr, $upstream:literal) => {
        Tool { id: $id, kind: $kind, label: $label, programs: &[$($program),*], manager: $manager, upstream: $upstream }
    };
}

pub(super) static TOOLS: &[Tool] = &[
    tool!(
        "rust",
        "toolchain",
        "Rust toolchain",
        ["rustc"],
        Some("rustup"),
        "https://static.rust-lang.org"
    ),
    tool!(
        "python",
        "runtime",
        "Python runtime",
        ["python3", "python"],
        Some("uv"),
        "https://github.com/astral-sh/python-build-standalone"
    ),
    tool!(
        "node",
        "runtime",
        "Node.js runtime",
        ["node"],
        Some("fnm"),
        "https://nodejs.org/dist"
    ),
    tool!(
        "rustup",
        "development-tool",
        "Rustup",
        ["rustup"],
        None,
        "https://rustup.rs"
    ),
    tool!(
        "uv",
        "package-manager",
        "uv",
        ["uv"],
        None,
        "https://docs.astral.sh/uv/"
    ),
    tool!(
        "fnm",
        "development-tool",
        "Fast Node Manager",
        ["fnm"],
        None,
        "https://github.com/Schniz/fnm"
    ),
    tool!(
        "cargo",
        "package-manager",
        "Cargo",
        ["cargo"],
        None,
        "https://doc.rust-lang.org/cargo/"
    ),
    tool!(
        "npm",
        "package-manager",
        "npm",
        ["npm"],
        None,
        "https://docs.npmjs.com/"
    ),
    tool!(
        "pnpm",
        "package-manager",
        "pnpm",
        ["pnpm"],
        None,
        "https://pnpm.io/"
    ),
    tool!(
        "yarn",
        "package-manager",
        "Yarn",
        ["yarn"],
        None,
        "https://yarnpkg.com/"
    ),
    tool!(
        "cmake",
        "development-tool",
        "CMake",
        ["cmake"],
        None,
        "https://cmake.org/"
    ),
    tool!(
        "ninja",
        "development-tool",
        "Ninja",
        ["ninja"],
        None,
        "https://ninja-build.org/"
    ),
    tool!(
        "qt",
        "sdk",
        "Qt SDK",
        ["qtpaths6", "qtpaths", "qmake6", "qmake"],
        None,
        "https://www.qt.io/"
    ),
    tool!(
        "msvc",
        "toolchain",
        "Microsoft Visual C++",
        ["cl"],
        None,
        "https://learn.microsoft.com/cpp/"
    ),
    tool!(
        "windows-sdk",
        "sdk",
        "Windows SDK",
        [],
        None,
        "https://developer.microsoft.com/windows/downloads/windows-sdk/"
    ),
    tool!(
        "git",
        "development-tool",
        "Git",
        ["git"],
        None,
        "https://git-scm.com/"
    ),
];

pub(super) fn kind_filter(kind: Option<&str>) -> Result<Option<&str>, String> {
    match kind {
        None => Ok(None),
        Some("runtime" | "toolchain" | "sdk" | "development-tool" | "package-manager") => Ok(kind),
        Some(_) => Err(
            "Unknown tool type. Use runtime, toolchain, sdk, development-tool or package-manager."
                .into(),
        ),
    }
}

pub(super) fn get(id: &str, kind: Option<&str>) -> Result<&'static Tool, String> {
    let id = match id {
        "nodejs" | "node.js" => "node",
        "rustc" => "rust",
        other => other,
    };
    TOOLS
        .iter()
        .find(|tool| tool.id == id && kind.is_none_or(|kind| tool.kind == kind))
        .ok_or_else(|| "Unknown tool for this type. Use cnr search to see matching tools.".into())
}

pub(super) fn matches(tool: &Tool, query: &str, kind: Option<&str>) -> bool {
    kind.is_none_or(|kind| tool.kind == kind)
        && [tool.id, tool.kind, tool.label]
            .iter()
            .any(|text| text.to_ascii_lowercase().contains(query))
}
