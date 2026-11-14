#!/usr/bin/env python3
"""Read-only, offline documentation check; run with Python 3.9+.

Checks root Markdown and en/vi trees only. Links support simple inline links,
images, reference definitions/uses, and ATX/Setext heading fragments (including
duplicate heading suffixes). External URLs, fenced code, inline code, and HTML
comments are ignored. Nested brackets/parentheses, multiline destinations, raw
HTML links/anchors, and renderer-specific anchor rules are outside this subset.
Symlinks and Windows reparse points are rejected; no link may leave the root or
enter .git/.superpowers. Skill YAML uses exactly two unquoted single-line plain
scalars: name and description. Exit codes: 0 PASS, 1 FAIL, 2 invalid invocation.
"""

import argparse
import hashlib
import os
from pathlib import Path
import re
import stat
import sys
from urllib.parse import unquote, urlsplit


SKILLS = {"implementation", "debugging", "verification"}
EXCLUDED = {".git", ".superpowers"}
STAMP = re.compile(r"<!-- source: (en/[^\n]+\.md) -->\n"
                   r"<!-- source-sha256: ([0-9a-f]{64}) -->\n?\Z")
DESTINATION = r'(<[^>\n]+>|[^\s)]+)'
INLINE = re.compile(r'!?\[[^\]\n]*\]\(\s*' + DESTINATION + r'(?:\s+"[^"\n]*"|\s+\x27[^\x27\n]*\x27)?\s*\)')
DEFINITION = re.compile(r'^ {0,3}\[([^\]\n]+)\]:\s*' + DESTINATION, re.M)
REFERENCE = re.compile(r'!?\[([^\]\n]+)\]\[([^\]\n]*)\]')


def prose(text, keep_inline_code=False):
    """Remove code/comments while preserving newlines for heading parsing."""
    lines, fence = [], None
    for line in text.splitlines(keepends=True):
        mark = re.match(r"^ {0,3}(`{3,}|~{3,})", line)
        if fence:
            if re.match(r"^ {0,3}" + re.escape(fence[0]) + "{" + str(len(fence)) + r",}\s*$", line):
                fence = None
            lines.append("\n")
        elif mark:
            fence = mark[1]
            lines.append("\n")
        else:
            lines.append(line)
    text = "".join(lines)
    text = re.sub(r"(`+)(?!`)(.*?)\1(?!`)",
                  lambda m: m[2] if keep_inline_code else "\n" * m[0].count("\n"), text, flags=re.S)
    return re.sub(r"<!--.*?-->", lambda m: "\n" * m[0].count("\n"), text, flags=re.S)


def anchors(text):
    result, counts = set(), {}
    text = re.sub(r"\A---\n.*?\n---(?:\n|\Z)", "", text, flags=re.S)
    lines = prose(text, keep_inline_code=True).splitlines()
    for index, line in enumerate(lines):
        heading = re.match(r"^ {0,3}#{1,6}\s+(.+?)\s*#*\s*$", line)
        title = heading[1] if heading else None
        if title is None and index + 1 < len(lines) and line.strip() and re.match(r"^ {0,3}(?:=+|-+)\s*$", lines[index + 1]):
            title = line.strip()
        if title is None:
            continue
        title = re.sub(r"!?\[([^\]]+)\]\([^)]*\)", r"\1", title)
        title = re.sub(r"<[^>]+>", "", title).lower()
        slug = re.sub(r"[^\w\-\s]", "", title, flags=re.UNICODE).replace(" ", "-")
        number = counts.get(slug, 0)
        counts[slug] = number + 1
        result.add(f"{slug}-{number}" if number else slug)
    return result


class Checker:
    def __init__(self, root):
        self.root = root
        self.errors = []
        self.raw = {}
        self.text = {}

    def fail(self, path, message):
        self.errors.append(f"{path.relative_to(self.root).as_posix()}: {message}")

    def safe(self, path):
        """Validate every component before reading or descending into a path."""
        path = Path(os.path.abspath(path))
        try:
            parts = path.relative_to(self.root).parts
        except ValueError:
            raise ValueError("target leaves selected root") from None
        if EXCLUDED.intersection(part.casefold() for part in parts):
            raise ValueError("target enters an excluded directory")
        cursor = self.root
        for part in parts:
            cursor /= part
            try:
                info = cursor.lstat()
            except FileNotFoundError:
                break
            if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT:
                raise ValueError("symlink/reparse point is not allowed")
        return path

    def collect(self, directory):
        for path in sorted(directory.iterdir()):
            if path.name.casefold() in EXCLUDED:
                continue
            try:
                self.safe(path)
                if path.is_dir():
                    yield from self.collect(path)
                elif path.suffix.lower() == ".md":
                    yield path
            except (OSError, ValueError) as exc:
                self.fail(path, str(exc))

    def read(self, path):
        if path in self.text:
            return self.text[path]
        try:
            data = self.safe(path).read_bytes()
            self.raw[path] = data
            text = data.decode("utf-8")
            self.text[path] = text
            if b"\r" in data or (data and not data.endswith(b"\n")):
                self.fail(path, "use LF line endings and a final newline")
            if any(line.endswith((" ", "\t")) for line in text.splitlines()):
                self.fail(path, "trailing whitespace")
            return text
        except (OSError, UnicodeError, ValueError) as exc:
            self.fail(path, str(exc))
            return ""

    def link(self, source, destination):
        destination = destination.strip("<>")
        if destination.startswith("//") or re.match(r"^[A-Za-z][A-Za-z0-9+.-]*:", destination):
            return
        try:
            url = urlsplit(destination)
            name = unquote(url.path).replace("\\", "/")
            target = (self.root / name.lstrip("/") if name.startswith("/") else source.parent / name) if name else source
            target = self.safe(target)
            if not target.exists():
                raise ValueError("target does not exist")
            if url.fragment:
                if target.suffix.lower() != ".md" or not target.is_file():
                    raise ValueError("fragment target must be Markdown")
                if unquote(url.fragment) not in anchors(self.read(target)):
                    raise ValueError("heading fragment does not exist")
        except (OSError, ValueError) as exc:
            self.fail(source, f"invalid link {destination!r}: {exc}")

    def links(self, path, text):
        clean = prose(text)
        normalize = lambda value: " ".join(value.split()).casefold()
        definitions = {normalize(m[1]): m[2] for m in DEFINITION.finditer(clean)}
        for target in definitions.values():
            self.link(path, target)
        clean = DEFINITION.sub("", clean)
        for match in INLINE.finditer(clean):
            self.link(path, match[1])
        clean = INLINE.sub("", clean)
        for match in REFERENCE.finditer(clean):
            label = normalize(match[2] or match[1])
            if label not in definitions:
                self.fail(path, f"undefined link reference {label!r}")

    def translations(self, english, vietnamese):
        en = {p.relative_to(self.root / "en").as_posix(): p for p in english}
        vi = {p.relative_to(self.root / "vi").as_posix(): p for p in vietnamese}
        for name in sorted(en.keys() - vi.keys()):
            self.fail(en[name], "missing Vietnamese translation")
        for name in sorted(vi.keys() - en.keys()):
            self.fail(vi[name], "orphan Vietnamese translation")
        for name in sorted(en.keys() & vi.keys()):
            text = self.text.get(vi[name], "")
            stamp = STAMP.search(text)
            if not stamp:
                self.fail(vi[name], "end with source and source-sha256 metadata comments")
            elif stamp[1] != f"en/{name}":
                self.fail(vi[name], "source metadata does not match paired English path")
            elif en[name] in self.raw and stamp[2] != hashlib.sha256(self.raw[en[name]]).hexdigest():
                self.fail(vi[name], "source hash is stale")

    def skills(self):
        directory = self.root / "en/skills"
        try:
            self.safe(directory)
            found = set()
            for path in directory.iterdir():
                self.safe(path)
                if path.is_dir():
                    found.add(path.name)
            if found != SKILLS:
                self.fail(directory, "expected exactly implementation, debugging, verification directories")
        except (OSError, ValueError) as exc:
            self.fail(directory, str(exc))
        for name in sorted(SKILLS):
            path = directory / name / "SKILL.md"
            text = self.text.get(path, "")
            match = re.match(r"\A---\n(.*?)\n---(?:\n|\Z)", text, re.S)
            fields = {}
            valid = bool(match)
            for line in match[1].splitlines() if match else []:
                item = re.fullmatch(r"(name|description): ([^\n]+)", line)
                if not item or item[1] in fields:
                    valid = False
                    continue
                value = item[2]
                if (value[0] in "[]{}'\"&*!|>@`#%" or ": " in value or " #" in value
                        or re.match(r"[-?:]\s", value)
                        or value.lower() in {"true", "false", "yes", "no", "on", "off", "null", "~"}
                        or re.fullmatch(r"[-+]?\d+(?:\.\d+)?", value)):
                    valid = False
                fields[item[1]] = value
            if not valid or set(fields) != {"name", "description"} or fields.get("name") != name:
                self.fail(path, "require plain name/description frontmatter matching the skill directory")

    def run(self):
        roots = sorted(p for p in self.root.iterdir() if p.suffix.lower() == ".md")
        trees = {}
        for language in ("en", "vi"):
            path = self.root / language
            try:
                self.safe(path)
                trees[language] = list(self.collect(path))
            except (OSError, ValueError) as exc:
                self.fail(path, str(exc))
                trees[language] = []
        for path in roots + trees["en"] + trees["vi"]:
            self.read(path)
        self.translations(trees["en"], trees["vi"])
        for relative in ("AGENTS.md", "en/AGENTS.md", "CLAUDE.md"):
            if self.root / relative not in self.raw:
                self.fail(self.root / relative, "required readable Markdown file is missing")
        if self.raw.get(self.root / "AGENTS.md") != self.raw.get(self.root / "en/AGENTS.md"):
            self.fail(self.root / "AGENTS.md", "must exactly mirror en/AGENTS.md")
        if self.raw.get(self.root / "CLAUDE.md") != b"@AGENTS.md\n":
            self.fail(self.root / "CLAUDE.md", "must contain exactly @AGENTS.md and LF")
        self.skills()
        for path, text in list(self.text.items()):
            self.links(path, text)
        return self.errors


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        root = args.root.resolve(strict=True)
        if not root.is_dir():
            raise ValueError("root must be a directory")
    except (OSError, ValueError) as exc:
        parser.error(str(exc))
    try:
        errors = Checker(root).run()
    except OSError as exc:
        print(f"FAIL: {exc}")
        return 1
    for error in errors:
        print(f"FAIL: {error}")
    if not errors:
        print("PASS: bilingual documentation checks")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
