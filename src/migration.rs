use serde_json::Value;
use std::{collections::BTreeMap, path::Path};

mod archive;
mod documents;
mod files;
mod proposal;

#[cfg(test)]
mod tests;

pub(crate) fn run(
    root: &Path,
    action: &str,
    args: &BTreeMap<String, String>,
) -> Result<Value, String> {
    let allowed = match action {
        "inspect" => &["source"][..],
        "export" => &["source", "destination"],
        "validate" => &["archive"],
        "preview" => &["archive", "additional-archive", "workspace", "preview-out"],
        "apply" => &[
            "archive",
            "additional-archive",
            "workspace",
            "expected-digest",
        ],
        "rollback" => &["expected-digest"],
        _ => return Err("Unknown legacy migration action".into()),
    };
    if let Some(key) = args.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(format!("Unexpected legacy migration option --{key}"));
    }
    let value = |key| {
        args.get(key)
            .map(String::as_str)
            .ok_or_else(|| format!("Missing --{key}"))
    };
    let path = |key| value(key).map(Path::new);
    match action {
        "inspect" => archive::inspect(path("source")?),
        "export" => archive::export(root, path("source")?, path("destination")?),
        "validate" => archive::open(path("archive")?).map(|(_, report)| report),
        "preview" | "apply" => {
            let mut paths = vec![path("archive")?];
            if let Some(additional) = args.get("additional-archive") {
                paths.push(Path::new(additional));
            }
            let workspace = args.get("workspace").map(Path::new);
            if action == "preview" {
                proposal::preview(
                    root,
                    &paths,
                    workspace,
                    args.get("preview-out").map(Path::new),
                )
            } else {
                proposal::apply(root, &paths, workspace, value("expected-digest")?)
            }
        }
        "rollback" => proposal::rollback(root, value("expected-digest")?),
        _ => unreachable!("Legacy migration action was checked"),
    }
}

pub(crate) fn audit(root: &Path, digest: &str) -> Result<Value, String> {
    proposal::audit(root, digest)
}
