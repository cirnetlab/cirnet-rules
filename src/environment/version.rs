pub(super) type Version = [u64; 3];

fn numeric(text: &str) -> Option<(Version, usize)> {
    let mut version = [0; 3];
    let mut count = 0;
    for part in text.strip_prefix('v').unwrap_or(text).split('.') {
        if count == version.len() || part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        if part.len() > 1 && part.starts_with('0') {
            return None;
        }
        version[count] = part.parse().ok()?;
        count += 1;
    }
    Some((version, count))
}

pub(super) fn exact(text: &str) -> Option<Version> {
    numeric(text)
        .filter(|(_, count)| *count == 3)
        .map(|(version, _)| version)
}

pub(super) fn selector(text: &str, version: Version) -> Option<bool> {
    let (wanted, count) = numeric(text)?;
    Some(version[..count] == wanted[..count])
}

pub(super) fn constraint(text: &str, version: Version, npm: bool) -> Option<bool> {
    let mut unknown = false;
    for clause in text.split("||") {
        match conjunction(clause, version, npm) {
            Some(true) => return Some(true),
            Some(false) => {}
            None => unknown = true,
        }
    }
    (!unknown).then_some(false)
}

fn conjunction(text: &str, version: Version, npm: bool) -> Option<bool> {
    let mut words = text
        .split(|ch: char| ch == ',' || ch.is_ascii_whitespace())
        .filter(|word| !word.is_empty());
    let first = words.next()?;
    let mut next = Some(first);
    let mut unknown = false;
    while let Some(word) = next {
        let operator = [">=", "<=", "==", "!=", "~=", ">", "<", "=", "^", "~"]
            .into_iter()
            .find(|op| word.starts_with(op))
            .unwrap_or("");
        let value = match &word[operator.len()..] {
            "" => words.next()?,
            value => value,
        };
        match compare(operator, value, version, npm) {
            Some(false) => return Some(false),
            Some(true) => {}
            None => unknown = true,
        }
        next = words.next();
    }
    (!unknown).then_some(true)
}

fn compare(op: &str, text: &str, version: Version, npm: bool) -> Option<bool> {
    let wildcard = matches!(text, "*" | "x" | "X");
    let prefix = text
        .strip_suffix(".*")
        .or_else(|| text.strip_suffix(".x"))
        .or_else(|| text.strip_suffix(".X"));
    if wildcard || prefix.is_some() {
        let same = if let Some(prefix) = prefix {
            selector(prefix, version)?
        } else {
            true
        };
        return match op {
            "" | "=" | "==" => Some(same),
            "!=" => Some(!same),
            _ => None,
        };
    }
    let (wanted, count) = numeric(text)?;
    if npm && count < 3 && matches!(op, ">" | "<=") {
        let mut upper = wanted;
        upper[count - 1] = upper[count - 1].checked_add(1)?;
        return Some(if op == ">" {
            version >= upper
        } else {
            version < upper
        });
    }
    Some(match op {
        "" | "=" => version[..count] == wanted[..count],
        "==" => version == wanted,
        "!=" => version != wanted,
        ">=" => version >= wanted,
        "<=" => version <= wanted,
        ">" => version > wanted,
        "<" => version < wanted,
        "^" | "~" | "~=" => {
            let component = match op {
                "^" => wanted
                    .iter()
                    .position(|part| *part != 0)
                    .unwrap_or(count - 1),
                "~" => (count > 1) as usize,
                "~=" if count >= 2 => count - 2,
                _ => return None,
            };
            let mut upper = wanted;
            upper[component] = upper[component].checked_add(1)?;
            upper[component + 1..].fill(0);
            version >= wanted && version < upper
        }
        _ => return None,
    })
}
