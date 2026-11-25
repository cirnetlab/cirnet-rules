---
name: cirnet-documentation-sync
description: Use when creating or changing documentation, including instructions, examples, navigation, or translations.
---

# Documentation synchronization

Use the repository's documentation policy for the affected scope.

1. Identify the purpose. Search the authorized repository by filename, title,
   heading, topic, issue ID, related text and incoming links. Update the
   canonical document in place. Create only for a distinct purpose or required
   evidence, and record why it belongs there.
2. For overlap, propose the canonical file, material to retain and links to
   update. Consolidate within scope and preserve separate plans and specs,
   dated results, compatibility versions, EN/VI pairs, fixtures and templates.
3. Update English first and review the paired Vietnamese for meaning, commands,
   parameters, versions, test outcomes, compatibility, provenance and safety
   limits before refreshing its source hash.
4. Run the applicable checker and `git diff --check`. Add `--new-file PATH`
   to `check-docs` for each new Markdown path it supports. When no suitable
   checker covers a path, review purpose, overlap, links and translation
   freshness manually within scope. Review duplicate suggestions and report
   changed pairs, decisions and verification limits.

For architecture snapshots, follow `en/policies/architecture.md`. Preserve the
supplied source text and confirm the EN/VI paths and digests in
`docs/architecture-sources.tsv`. Review divergent copies and retain their history
when recommending replacement links.
