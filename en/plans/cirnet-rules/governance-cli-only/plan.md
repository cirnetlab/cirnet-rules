<!-- doc-id: CNR-CLI-ONLY-PLAN -->
<!-- revision: 1 -->
<a id="implementation"></a>

# CLI-only implementation plan

The checkout and archive branches began at 52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c. The worktree had no staged, modified or untracked files. Ignored build outputs and the local zip remain outside this change. The current and two predecessor default database paths and SQLite sidecars were absent. Custom data locations are unknown.

| Action | Source and dependency boundary |
| --- | --- |
| Keep | Rust core, task store, Git bindings, context budgets, managed checks, docs and release gates, CNR tooling and terminal menu, compatibility CLI. |
| Extract | Checked legacy SQL and lossless snapshot plus reviewable mapping. Replace Desktop governance invariants with catalog and migration fixtures. |
| Replace | Dashboard authoring with the versioned Git authority; render selected sources for three Agent surfaces and use separate scoped sync. |
| Remove | Desktop source, private runtime and IPC, chat, scheduler, engine probes, Qt launch/package/CI. |

1. Inventory local state, check baseline tests and preserve migration SQL.
2. Implement and test SQLite snapshots, archives and migration proposals.
3. Remove the Desktop target and rewrite its active documentation.
4. Connect registry validation, shared resolution, freshness, CLI and receipt inspection.
5. Test exports and managed writes in fixture Git repositories and rerun the Windows checks.
