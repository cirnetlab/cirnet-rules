<!-- doc-id: CIRNETRULES-ROUTING -->
<!-- revision: 3 -->
<a id="routing"></a>

# Local workspace and repository routing

Run `cnr workspace discover` to propose checkout candidates from bounded
workspace metadata. Review the registered repository ID and bind the chosen
checkout with `cnr workspace bind --repo REPO --path WORKTREE --actor OPERATOR`.
The binding records the Git root, admin and common directories and anchor
commit. Replacing it requires the previous identity. Use
`cnr workspace health --repo REPO` to check it.

For parallel work on the same repository, use the
[named worktree workflow](../references/shared-task-coordination.md).
Choose the binding in the task descriptor before approval. Verify the
binding when handing off the task. A different binding requires a task
revision and renewed approval.

Use exact files in read and write scopes. Module cards and direct
dependencies describe impact. The resolver returns product files and task
records as data and loads instructions from the English governance entries.
When onboarding a repository, review its ownership, classification and
typed relations in the [registry](../../governance.yaml) first.

`cnr task context --task ID --session TOKEN` resolves the selected entrypoint,
spec, plan, skill and product sources. Reuse the token to request deltas.
Use a fresh token for a new or compacted session. The resolver checks dirty,
missing and untracked selected paths and records the input and cache counts.
See the [CLI reference](../references/cnr.md#execution) for cache maintenance.
