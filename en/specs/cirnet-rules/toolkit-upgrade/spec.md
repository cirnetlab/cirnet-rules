# Common Rules Toolkit upgrade

Baseline on 2026-11-14 was `35e677b1264958839a7e0cb883c4477eb24da6df`.

- Deliver a standalone English baseline, on-demand context, 12 namespaced
  skills with VI review copies, compatibility and repo templates, and a
  versioned offline bundle. Retain Rust bilingual, link and entrypoint checks.
- Default PowerShell 7 user commands to preview. Apply a reviewed local bundle
  explicitly, preserve personal bytes, back up affected paths and support
  conflict-aware restore.
- Verify failure paths in fake homes and extracted bundles, context walkthroughs,
  text size, Rust tests and toolkit CI. Record native loading separately.

Writes are limited to `rules-common`. Product work, releases, real client
installation and Git mutations are outside scope.

## Markdown hygiene

Extend the offline, read-only Rust checker using the scanned Markdown inventory.
Separate byte matches, conservative normalization and scored similarity within
comparable document roles and languages. Suggest a canonical candidate and flag
unique content and incoming links for review. Protect deliberate duplication and
evidence. Review explicitly supplied new paths without Git or network access.

Use existing rules and skills for search, update, consolidation and verification.
Accept the change when focused checker tests, EN/VI docs and the checkout pass.
Behavioral cases and their evidence are outside this verification.

Hardening at `66d2c8a` adds a standalone baseline reminder and requires
reviewing each new Markdown path when the checker is available. Exact or
conservatively normalized non-exempt copies fail when either path is declared
new. Similarity remains advisory. CI delegates committed Markdown additions to
the offline checker using a verified local Git base and literal arguments.
Without a usable base, report the limited scan. Verify this path with isolated
Git fixtures and preserve native evidence.

## Architecture compliance

Preserve the Ecosystem, Lab-First and FPGA source pairs under `docs/` with
revision and hashes. Use the [policy](../../../policies/architecture.md) and
[owner routing](../../../references/architecture-routing.md) on demand.
Default to `LAB_FIRST_PRIMARY`; allow parallel prioritization only with
recorded evidence from a pinned vertical slice. Record VD-01–VD-10 separately.

Extend the existing offline check and CI to reject missing or mismatched
snapshots, stale architecture links and invalid routing. Warn about divergent
copies without editing them. Confirm package and installer behavior in isolated
fixtures. Product behavior, hardware operation, real client configuration and
historical E01–E08 evidence are outside this change.
