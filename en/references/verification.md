<!-- doc-id: VERIFY-LOCAL -->
<!-- revision: 2 -->
<a id="evidence"></a>

# Verification and audit

Run the core checks from the governance root using the pinned Rust toolchain.
Build both binaries and make `cnr` available on your path for the CLI checks.

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked --all-targets
cargo build --locked --bins
cnr governance validate --json
cnr governance check-translations --json
```

For a task, run the diff check against its recorded bindings. For coordinated
tasks, also check dependency evidence before verification. Inspect the task
audit for the recorded decision, receipts and current authority status.

```sh
cnr task diff-check --task ID --json
cnr task dependency check --task ID --json
cnr audit task --task ID --json
```

Follow the [operator workflow](cirnetrules-local.md) for verification,
review and acceptance. Record the governance and source revisions, command,
platform, exit code, result digest and any limits. Use `BLOCKED` when a
required input, grant or tool is unavailable.

The [candidate CI procedure](shared-task-coordination.md) consumes reviewed
portable evidence. Native agent instruction discovery and hook behavior
require a run in the chosen client. See the [adapter guidance](../adapters/README.md),
[testing rule](../rules/testing.md) and [coordination verification record](../specs/cirnet-rules/mcp-shared-coordination/verification.md).

The [product audit](product-audit.md) and [blocked release receipt](evidence/check-release-blocked.json)
record their original product inputs and decisions.
