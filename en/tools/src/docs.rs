use crate::policy::{is_link, read, safe_path};
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn inventory(root: &Path, locale: &str) -> Result<BTreeSet<String>, String> {
    fn scan(
        dir: &Path,
        prefix: &str,
        locale: &str,
        files: &mut BTreeSet<String>,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 16 || files.len() > 1024 {
            return Err("Documentation inventory bound exceeded".into());
        }
        for entry in fs::read_dir(dir).map_err(|_| "Cannot list documentation directory")? {
            let entry = entry.map_err(|_| "Cannot inspect documentation entry")?;
            let meta = fs::symlink_metadata(entry.path())
                .map_err(|_| "Cannot read documentation metadata")?;
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| "Non-UTF-8 documentation path")?;
            let path = format!("{prefix}/{name}");
            if is_link(&meta) {
                return Err(format!("Link in documentation tree: {path}"));
            }
            if meta.is_dir() {
                if name.starts_with("cirnet-")
                    && !matches!(prefix, "en/plans" | "en/specs" | "vi/plans" | "vi/specs")
                {
                    return Err(format!(
                        "Repository-scoped folder outside plans/specs: {path}"
                    ));
                }
                scan(&entry.path(), &path, locale, files, depth + 1)?;
            } else if name.ends_with(".md") {
                files.insert(path);
            } else if locale == "vi" {
                return Err(format!("Vietnamese branch may contain only prose: {path}"));
            }
        }
        Ok(())
    }
    let mut files = BTreeSet::new();
    scan(&safe_path(root, locale)?, locale, locale, &mut files, 0)?;
    Ok(files)
}

fn marker<'a>(body: &'a str, key: &str) -> Result<&'a str, String> {
    let prefix = format!("<!-- {key}: ");
    let mut matches = body
        .lines()
        .filter_map(|line| line.strip_prefix(&prefix)?.strip_suffix(" -->"));
    let value = matches
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Missing {key} marker"))?;
    if matches.next().is_some() {
        return Err(format!("Duplicate {key} marker"));
    }
    Ok(value)
}

fn anchors(body: &str) -> BTreeSet<String> {
    body.lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("<a id=\"")?
                .strip_suffix("\"></a>")
        })
        .map(str::to_owned)
        .collect()
}

fn identifiers(body: &str) -> BTreeSet<String> {
    Parser::new(body)
        .filter_map(|event| match event {
            Event::Code(code)
                if code.len() >= 2
                    && code
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"/._-:@{}".contains(&b))
                    && (code
                        .bytes()
                        .any(|b| b.is_ascii_digit() || b"/._-:@{}".contains(&b))
                        || code.bytes().all(|b| b.is_ascii_uppercase())) =>
            {
                Some(code.into_string())
            }
            _ => None,
        })
        .collect()
}

fn prose_versions(body: &str) -> BTreeSet<String> {
    let mut versions = BTreeSet::new();
    for event in Parser::new(body) {
        let Event::Text(text) = event else { continue };
        for token in text.split(|c: char| !c.is_ascii_digit() && c != '.') {
            let token = token.trim_matches('.');
            let parts: Vec<_> = token.split('.').collect();
            if parts.len() == 3 && parts.iter().all(|part| !part.is_empty()) {
                versions.insert(token.to_owned());
            }
        }
    }
    versions
}

fn verify_pair(source: &str, translated: &str) -> Result<String, String> {
    let id = marker(source, "doc-id")?;
    let revision = marker(source, "revision")?;
    if !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        || !revision.parse::<u64>().is_ok_and(|r| r > 0)
        || marker(translated, "doc-id")? != id
        || marker(translated, "revision")? != revision
    {
        return Err("Document ID or revision differs".into());
    }
    let expected = sha256(source.as_bytes());
    if marker(translated, "source-sha256")? != expected {
        return Err(format!(
            "Translation source digest is stale, expected {expected}"
        ));
    }
    if marker(translated, "translation-status")? != "synchronized" {
        return Err("Translation has no synchronized status".into());
    }
    let critical = anchors(source);
    if critical.is_empty() || critical != anchors(translated) {
        return Err("Critical section anchors differ".into());
    }
    let en = identifiers(source);
    let vi = identifiers(translated);
    if en != vi {
        return Err(format!(
            "Machine identifiers differ: EN-only {:?}; VI-only {:?}",
            en.difference(&vi).collect::<Vec<_>>(),
            vi.difference(&en).collect::<Vec<_>>()
        ));
    }
    let en = prose_versions(source);
    let vi = prose_versions(translated);
    if en != vi {
        return Err(format!(
            "Version mentions differ: EN-only {:?}; VI-only {:?}",
            en.difference(&vi).collect::<Vec<_>>(),
            vi.difference(&en).collect::<Vec<_>>()
        ));
    }
    Ok(id.to_owned())
}

fn slug(title: &str) -> String {
    title
        .chars()
        .flat_map(char::to_lowercase)
        .filter_map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_') {
                Some(c)
            } else if c.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect()
}

fn link_anchors(body: &str) -> BTreeSet<String> {
    let mut anchors = anchors(body);
    let mut heading = None;
    for event in Parser::new(body) {
        match event {
            Event::Start(Tag::Heading { .. }) => heading = Some(String::new()),
            Event::Text(value) | Event::Code(value) if heading.is_some() => {
                heading.as_mut().unwrap().push_str(&value);
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(text) = heading.take() {
                    anchors.insert(slug(&text));
                }
            }
            _ => {}
        }
    }
    anchors
}

fn links(root: &Path, path: &str, body: &str) -> Result<(), String> {
    for event in Parser::new(body) {
        let dest = match event {
            Event::Start(Tag::Link { dest_url, .. } | Tag::Image { dest_url, .. }) => dest_url,
            _ => continue,
        };
        if ["https://", "http://", "mailto:"]
            .iter()
            .any(|scheme| dest.starts_with(scheme))
        {
            continue;
        }
        if dest.contains(['%', '\\', ':', '?']) || dest.starts_with('/') {
            return Err(format!("Unsafe link in {path}: {dest}"));
        }
        let (file, fragment) = dest.split_once('#').unwrap_or((&dest, ""));
        let mut rel = PathBuf::from(path);
        if !file.is_empty() {
            rel.pop();
            for component in Path::new(file).components() {
                match component {
                    Component::Normal(value) => rel.push(value),
                    Component::CurDir => {}
                    Component::ParentDir if rel.pop() => {}
                    _ => return Err(format!("Link leaves governance checkout in {path}: {dest}")),
                }
            }
        }
        let rel = rel.to_string_lossy().replace('\\', "/");
        let target =
            safe_path(root, &rel).map_err(|_| format!("Broken local link in {path}: {dest}"))?;
        if !fragment.is_empty()
            && target.extension().is_some_and(|ext| ext == "md")
            && !link_anchors(&read(&target)?).contains(fragment)
        {
            return Err(format!("Broken anchor in {path}: {dest}"));
        }
    }
    Ok(())
}

pub fn verify(root: &Path) -> Result<usize, String> {
    let english = inventory(root, "en")?;
    let vietnamese = inventory(root, "vi")?;
    let expected: BTreeSet<_> = english
        .iter()
        .map(|path| path.replacen("en/", "vi/", 1))
        .collect();
    if english.is_empty() || expected != vietnamese {
        return Err(format!(
            "Bilingual inventory differs. Missing {:?}, orphan {:?}",
            expected.difference(&vietnamese).collect::<Vec<_>>(),
            vietnamese.difference(&expected).collect::<Vec<_>>()
        ));
    }
    let mut ids = BTreeSet::new();
    for path in &english {
        let vi = path.replacen("en/", "vi/", 1);
        let source = read(&safe_path(root, path)?)?;
        let translated = read(&safe_path(root, &vi)?)?;
        let id = verify_pair(&source, &translated).map_err(|error| format!("{path}: {error}"))?;
        if !ids.insert(id.clone()) {
            return Err(format!("Duplicate document ID: {id}"));
        }
        links(root, path, &source)?;
        links(root, &vi, &translated)?;
    }
    for path in ["README.md", "AGENTS.md"] {
        links(root, path, &read(&safe_path(root, path)?)?)?;
    }
    Ok(english.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> (String, String) {
        let en = "<!-- doc-id: EXAMPLE -->\n<!-- revision: 1 -->\n<a id=\"critical\"></a>\nUse 0.2.4 and `0.2`.\n";
        let vi = format!(
            "<!-- doc-id: EXAMPLE -->\n<!-- revision: 1 -->\n<!-- source-sha256: {} -->\n<!-- translation-status: synchronized -->\n<a id=\"critical\"></a>\nDùng 0.2.4 và `0.2`.\n",
            sha256(en.as_bytes())
        );
        (en.to_owned(), vi)
    }

    #[test]
    fn matching_translation_and_stale_revision() {
        let (en, vi) = pair();
        assert!(verify_pair(&en, &vi).is_ok());
        assert!(
            verify_pair(&(en + "new text\n"), &vi)
                .unwrap_err()
                .contains("stale")
        );
    }

    #[test]
    fn a_different_version_in_translated_prose_fails() {
        let (en, vi) = pair();
        let vi = vi.replace("0.2.4", "0.2.5");
        assert!(
            verify_pair(&en, &vi)
                .unwrap_err()
                .contains("Version mentions differ")
        );
    }

    #[test]
    fn critical_anchor_and_identifier_cannot_be_translated_away() {
        let (en, vi) = pair();
        assert!(
            verify_pair(&en, &vi.replace("critical", "critical-vi"))
                .unwrap_err()
                .contains("anchors")
        );
        assert!(
            verify_pair(&en, &vi.replace("0.2", "0.3"))
                .unwrap_err()
                .contains("identifiers")
        );
    }
}
