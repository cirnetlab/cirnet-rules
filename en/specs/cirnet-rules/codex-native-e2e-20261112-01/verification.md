<!-- doc-id: CNR-CODEX-NATIVE-E2E-VERIFICATION -->
<!-- revision: 5 -->
<a id="evidence"></a>

# Codex Desktop native integration verification

Run `codex-native-e2e-20261112-01` verified a native Desktop documentation edit. Governance started from `e293c7b39b17b642b0423d9153dc0753627e74ea`. The separate Interface worktree is bound as `codex-e2e-20261112-01` at `f1ff70b9f22554c0680b92679e9e44a5544247ff`. The setup outputs form part of the smoke baseline. The two product index edits remain unstaged.

| Stage | Status | Evidence |
| --- | --- | --- |
| CP1: CNR, repository, builds | PASS | Windows CNR wizard displayed the exact binding and ran Check. It exited normally; unconfigured rows kept their warning and unobserved states. The final build validated governance and binding. |
| CP2: target and task preparation | PASS | CNR validated the named worktree and recorded `mrben`'s approval for separate setup and product scopes. The edit used revision 4 and approval epoch 2. |
| CP3: instruction and MCP previews | PASS | A managed sync preview lists 15 creates and one update to the managed block of `AGENTS.md`. The read-only MCP template validates and previews creating the project `.codex/config.toml` entry. |
| CP3: sync and registration apply | PASS | CNR applied 16 instruction outputs, preserved the existing `AGENTS.md` content outside the managed block, registered the project MCP entry, and read both back. The controlled `interface-diff` check passed. The setup receipt is retained and independent acceptance is pending. |
| CP3: CNR STDIO probe | PASS | The rebuilt executable launched its server and verified the read-only protocol including status and context for the actual smoke task. The result records `native_client: NOT_RUN`. |
| CP4: Desktop MCP calls and task context | PASS | The Local Interface session log records PASS for ten `cirnetrules` calls across all six read-only tools. The retained context results show revision 4, the expected binding and the two write paths. |
| CP4: Automatic project instruction and skill discovery | PASS | The Desktop session log contains the full matching target `AGENTS.md` and the target `implementation` skill metadata before the first user prompt. |
| CP4: Skill invocation | NOT_VERIFIED | The session read the full `SKILL.md` through a shell command. The first prompt also asked for skill evidence, so it cannot isolate implicit selection. Explicit invocation through the composer has yet to be tested. |
| CP5: Desktop edit, verification and audit | PASS | The approved host grant checks passed for both index files and blocked `AGENTS.md`. A Desktop file change event recorded one line replaced per file. Both files matched the expected bytes with CRLF preserved. The fresh CNR snapshot matched the digest in the native post-edit diff call. `interface-diff` passed and local audit confirmed the receipt at revision 6. |
| CP6: Rust regression and build gates | PASS | Format, check, Clippy with warnings denied, debug and release builds, and all targets passed: 266 tests passed, one external `uv`/Python test ignored. |

Regression coverage includes managed setup proposal creation without grants, documentation disposition conflicts, drift and trust denials, preservation of unrelated project settings, exact task context scopes, capability redaction, bounded response size, stale revisions and scoped diff digests. The shared task context still requires a current read grant. On Windows, isolated Git now reads the installer's built-in `core.autocrlf` value only when repository configuration does not supply it. Real content changes still show as dirty and scoped fingerprints use the original bytes.

The default Desktop shell returned an authority read error for CLI grant and diff checks. The same pinned binary completed those checks with per-command host approval. A separate sandbox test in the governance chat could open an Interface file but Git failed when changing to the Interface directory. Both Git roots resolved on the host. CNR reports an error while rebuilding authority as stale or unreadable. The underlying failing operation in the Interface shell remains unrecorded. The native audit call occurred before verification. The final CNR audit confirmed the controlled receipt and the smoke task remains `VERIFIED`, awaiting review.

The local checkpoint at `target/codex-native-e2e-20261112-01/checkpoint/resume.md` links the structured result, actual patch, sanitized Desktop traces, and controlled verification receipt. [Setup scope](../../cirnet-interface/codex-native-setup-20261112-01/spec.md) · [Smoke scope](../../cirnet-interface/codex-native-smoke-20261112-01/spec.md)
