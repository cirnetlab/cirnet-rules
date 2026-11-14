# Common Rules Toolkit upgrade

Date: 2026-11-14. Baseline: `35e677b1264958839a7e0cb883c4477eb24da6df`.

Keep a standalone English baseline and load skills and supporting material on demand.
Ship 12 distinct namespaced skills with Vietnamese review copies, compatibility
and repository templates, and a versioned offline bundle. Use the existing Rust
checker and retain its bilingual, link, and entrypoint checks.

PowerShell 7 user-scope commands preview by default. Explicit apply uses a reviewed
local bundle, protects personal bytes, backs up affected paths, and allows conflict-aware
restore. Use fake homes and extracted copies for tests. Acceptance covers installer
and bundle failures, context walkthroughs, text measurements, the Rust suite, and
toolkit-only CI. Native loader observations are separate from file and fixture checks.

Authorized scope is the existing `rules-common` worktree. All product, release,
real client installation, and Git mutations are outside this task.
