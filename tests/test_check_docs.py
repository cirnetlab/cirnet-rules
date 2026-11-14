"""Behavior tests; all scratch files stay below the repository's ignored output area."""

import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


TARGET = Path(__file__).resolve().parents[1]
OUTPUT = TARGET / ".superpowers" / "bilingual-tests"
SCRIPT = TARGET / "hooks" / "check_docs.py"


class CheckDocsTests(unittest.TestCase):
    def setUp(self):
        if not OUTPUT.resolve().is_relative_to(TARGET):
            raise RuntimeError("Fixture output escaped the repository")
        OUTPUT.mkdir(parents=True, exist_ok=True)
        self.scratch = Path(tempfile.mkdtemp(prefix="check-", dir=OUTPUT))
        self.root = self.scratch / "repo"
        self.root.mkdir()
        self.write("AGENTS.md", "# Agent rules\n")
        self.write("CLAUDE.md", "@AGENTS.md\n")
        self.write("README.md", "# Index\n\n[Guide](en/guide.md#first-heading)\n")
        self.pair("AGENTS.md", "# Agent rules\n")
        self.pair("guide.md", "# Guide\n\n## First heading\n")
        for name in ("implementation", "debugging", "verification"):
            self.pair(f"skills/{name}/SKILL.md",
                      f"---\nname: {name}\ndescription: Use when work needs {name}.\n---\n\n# Skill\n")

    def tearDown(self):
        resolved = self.scratch.resolve()
        if resolved.parent != OUTPUT.resolve() or not resolved.is_relative_to(TARGET):
            raise RuntimeError("Unsafe fixture cleanup path")
        shutil.rmtree(self.scratch)

    def write(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content.encode("utf-8") if isinstance(content, str) else content)

    def pair(self, name, source):
        data = source.encode("utf-8") if isinstance(source, str) else source
        self.write(f"en/{name}", data)
        self.write(f"vi/{name}", "# Bản dịch\n\n"
                   f"<!-- source: en/{name} -->\n"
                   f"<!-- source-sha256: {hashlib.sha256(data).hexdigest()} -->\n")

    def check(self, expected, *args):
        result = subprocess.run([sys.executable, "-B", str(SCRIPT), "--root", str(self.root), *args],
                                capture_output=True, text=True, encoding="utf-8")
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
        return result

    def test_valid_tree_is_read_only(self):
        before = {p.relative_to(self.root): p.read_bytes() for p in self.root.rglob("*") if p.is_file()}
        self.check(0)
        after = {p.relative_to(self.root): p.read_bytes() for p in self.root.rglob("*") if p.is_file()}
        self.assertEqual(after, before)

    def test_missing_translation_fails(self):
        (self.root / "vi/guide.md").unlink()
        self.check(1)

    def test_orphan_translation_fails(self):
        self.write("vi/orphan.md", "# Orphan\n")
        self.check(1)

    def test_stale_source_hash_fails(self):
        self.write("en/guide.md", "# Guide\n\n## First heading\n\nChanged.\n")
        self.check(1)

    def test_wrong_source_metadata_fails(self):
        path = self.root / "vi/guide.md"
        self.write("vi/guide.md", path.read_text(encoding="utf-8").replace("source: en/guide.md", "source: en/AGENTS.md"))
        self.check(1)

    def test_missing_or_nonterminal_metadata_fails(self):
        self.write("vi/guide.md", "# Missing stamps\n")
        self.check(1)
        self.pair("guide.md", "# Guide\n\n## First heading\n")
        path = self.root / "vi/guide.md"
        self.write("vi/guide.md", path.read_text(encoding="utf-8") + "After metadata.\n")
        self.check(1)

    def test_metadata_examples_do_not_override_terminal_pair(self):
        path = self.root / "vi/guide.md"
        self.write("vi/guide.md", "```md\n<!-- source: en/example.md -->\n"
                   "<!-- source-sha256: example -->\n```\n\n" + path.read_text(encoding="utf-8"))
        self.check(0)

    def test_broken_link_fails(self):
        self.write("README.md", "[Missing](en/missing.md)\n")
        self.check(1)

    def test_broken_fragment_fails(self):
        self.write("README.md", "[Missing](en/guide.md#missing)\n")
        self.check(1)

    def test_reference_links_and_encoded_fragment(self):
        self.pair("guide.md", "# Tiếng Việt\n\n## Same\n\n## Same\n")
        self.write("README.md", "[Read][guide]\n\n[guide]: en/guide.md#ti%E1%BA%BFng-vi%E1%BB%87t\n[Again](en/guide.md#same-1)\n")
        self.check(0)
        self.write("README.md", "[Read][guide]\n\n[guide]: en/missing.md\n")
        self.check(1)

    def test_missing_reference_definition_fails(self):
        self.write("README.md", "[Read][unknown]\n")
        self.check(1)

    def test_heading_inline_code_keeps_its_text_in_fragment(self):
        self.pair("guide.md", "# Use `command` here\n\nSetext heading\n--------------\n")
        self.write("README.md", "[Code heading](en/guide.md#use-command-here)\n[Setext](en/guide.md#setext-heading)\n")
        self.check(0)

    def test_frontmatter_is_not_a_heading(self):
        self.write("README.md", "[Metadata](en/skills/debugging/SKILL.md#description-use-when-work-needs-debugging)\n")
        self.check(1)

    def test_skill_frontmatter_requires_only_plain_name_and_description(self):
        name = "skills/debugging/SKILL.md"
        for source in ("# Missing metadata\n",
                       "---\nname: debugging\ndescription: Debug work.\nextra: value\n---\n",
                       "---\nname: wrong-name\ndescription: Debug work.\n---\n",
                       "---\nname: debugging\ndescription: [nested, values]\n---\n",
                       "---\nname: debugging\ndescription: - nested value\n---\n",
                       "---\nname: debugging\ndescription: true\n---\n",
                       "---\nname: debugging\ndescription: \n---\n"):
            with self.subTest(source=source):
                self.pair(name, source)
                self.check(1)

    def test_skill_directory_set_is_exact(self):
        self.pair("skills/extra/SKILL.md", "---\nname: extra\ndescription: Extra.\n---\n")
        self.check(1)

    def test_skill_file_is_required(self):
        (self.root / "en/skills/debugging/SKILL.md").unlink()
        (self.root / "vi/skills/debugging/SKILL.md").unlink()
        self.check(1)

    def test_root_agent_mirror_drift_fails(self):
        self.write("AGENTS.md", "# Drift\n")
        self.check(1)

    def test_adapter_is_exact(self):
        self.write("CLAUDE.md", "@en/AGENTS.md\n")
        self.check(1)

    def test_invalid_utf8_fails(self):
        self.write("README.md", b"# Invalid \xff\n")
        self.check(1)

    def test_crlf_and_trailing_whitespace_fail(self):
        for content in (b"# CRLF\r\n", b"# Trailing \n", b"# No final newline"):
            with self.subTest(content=content):
                self.write("README.md", content)
                self.check(1)

    def test_code_and_external_links_are_ignored(self):
        self.write("README.md", "# Examples\n\n`[ignored](missing.md)`\n\n"
                   "```md\n[ignored](missing.md)\n```\n\n"
                   "~~~md\n[ignored](missing.md)\n~~~\n\n"
                   "[Web](https://example.invalid/missing)\n[Email](mailto:someone@example.invalid)\n")
        self.check(0)

    def test_directory_and_non_markdown_targets_exist(self):
        self.write("asset.txt", "Asset\n")
        self.write("README.md", "[Directory](en/skills/)\n[Asset](asset.txt)\n")
        self.check(0)

    def test_traversal_outside_selected_root_fails(self):
        (self.scratch / "outside.md").write_bytes(b"# Exists\n")
        for target in ("../outside.md", "%2e%2e/outside.md", "..%5coutside.md"):
            with self.subTest(target=target):
                self.write("README.md", f"[Escape]({target})\n")
                self.check(1)

    def test_external_symlink_is_rejected(self):
        outside = self.scratch / "outside"
        outside.mkdir()
        (outside / "hidden.md").write_bytes(b"# Outside\n")
        try:
            os.symlink(outside, self.root / "en/escape", target_is_directory=True)
        except OSError as exc:
            self.skipTest(f"Symlink creation unavailable: {exc}")
        self.check(1)

    def test_excluded_trees_are_not_scanned(self):
        self.write(".superpowers/ignored.md", b"\xff")
        self.write(".git/ignored.md", b"\xff")
        self.check(0)

    def test_mixed_case_excluded_links_are_rejected(self):
        for directory in (".SUPERPOWERS", ".Git"):
            with self.subTest(directory=directory):
                self.write(f"{directory}/hidden.md", "# Hidden\n")
                self.write("README.md", f"[Excluded]({directory}/hidden.md)\n")
                self.check(1)

    def test_nested_mixed_case_excluded_trees_are_not_scanned(self):
        for language in ("en", "vi"):
            for directory in (".SUPERPOWERS", ".Git"):
                self.write(f"{language}/nested/{directory}/hidden.md", b"\xff")
        self.check(0)

    def test_invalid_root_exits_two(self):
        self.check(2, "--root", str(self.root / "does-not-exist"))


if __name__ == "__main__":
    unittest.main()
