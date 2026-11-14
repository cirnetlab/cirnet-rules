<!-- doc-id: CNR-NATIVE-SMOKE-20261112-01-SPEC -->
<!-- revision: 1 -->
<a id="scope"></a>

# Codex Desktop documentation smoke test

Task `codex-native-smoke-20261112-01` awaits operator approval and native execution. The proposed target is the `cirnet-interface` worktree bound as `codex-e2e-20261112-01` at `f1ff70b9f22554c0680b92679e9e44a5544247ff`.

The Codex Desktop session uses the current task context and improves the two documentation indexes by identifying the reviewed Contracts inputs and the Vite development server. The existing guides describe both. The exact read, write and verification paths are `docs/en/index.md` and `docs/vi/index.md`.

| Path | Current line | Proposed line |
| --- | --- | --- |
| `docs/en/index.md` | `- [Getting started](guides/getting-started.md) covers dependency setup and Vite.` | `- [Getting started](guides/getting-started.md) covers installing reviewed Contracts artifacts and starting the Vite development server.` |
| `docs/vi/index.md` | `- [Bắt đầu](guides/getting-started.md) hướng dẫn cài gói phụ thuộc và chạy Vite.` | `- [Bắt đầu](guides/getting-started.md) hướng dẫn cài các gói Contracts đã được xem xét và khởi động máy chủ phát triển Vite.` |

- [ ] A real Codex Desktop call retrieves the task status and current context for the selected binding before editing.
- [ ] A current write grant passes for each exact path before the native editor changes the two lines. An assessment of a third path is blocked.
- [ ] The scoped diff contains exactly those changes relative to the task baseline. The controlled `interface-diff` check passes and native MCP checks the current scoped snapshot digest.
- [ ] The audit record links the task revision and verification receipt. Independent acceptance remains with the authorized reviewer.

The documentation indexes are the deliverable. There are no separate post-acceptance documentation targets for this task. Setup output belongs to the separate configuration task and is captured before this task starts.

[Plan](../../../plans/cirnet-interface/codex-native-smoke-20261112-01/plan.md)
