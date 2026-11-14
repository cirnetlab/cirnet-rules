<!-- doc-id: CNR-NATIVE-SMOKE-20261112-01-PLAN -->
<!-- revision: 1 -->
<a id="implementation"></a>

# Codex Desktop documentation smoke test plan

Task `codex-native-smoke-20261112-01` awaits operator approval and native execution. [Specification and exact changes](../../../specs/cirnet-interface/codex-native-smoke-20261112-01/spec.md)

- [ ] Review the current `codex-e2e-20261112-01` binding and the completed configuration receipts. Approve only the two index paths and the `interface-diff` verification command. Start the task with a fresh baseline after the configuration task.
- [ ] In the Codex Desktop session opened at the bound worktree, call `cirnet_repositories_discover` and `cirnet_policy_lookup`. Read `cirnet_task_status` and use its revision for `cirnet_task_context`. Compare the context and task binding against CNR.
- [ ] Evaluate the write grant for `docs/en/index.md` and `docs/vi/index.md`. Evaluate a third path without editing it. When the two grants pass, apply the one-line replacements from the specification with the editor in that Desktop session.
- [ ] Compare the two changed files with the proposed diff and run `cnr task diff-check`. Use its current scoped digest in `cirnet_diff_check`. Record native calls and before and after digests without credentials.
- [ ] Record implementation and run `cnr task verify` with the approved `interface-diff` step. Read the task audit and leave the patch available for independent review.
