<!-- doc-id: CNR-SHARED-COORDINATION -->
<!-- revision: 1 -->
<a id="coordination"></a>

# Shared task coordination

Confirm the default repo binding and register a distinct Git worktree from
the same common directory. Set the optional descriptor `bindings` map
before approving the task. The task freezes the selected binding. Rebinding
uses explicit task revision and renewed approval.

```sh
cnr workspace binding add --repo REPO --binding NAME --path WORKTREE_RELATIVE --actor OPERATOR --json
cnr workspace binding validate --repo REPO --binding NAME --json
cnr task status --task ID --json
cnr task session-prepare --task ID --revision REVISION --actor APPROVER --request-id REQUEST --json
cnr task claim --task ID --revision REVISION --session-file SESSION_FILE --request-id REQUEST --json
cnr task renew --task ID --generation GENERATION --session-file SESSION_FILE --request-id REQUEST --json
cnr task report --task ID --revision REVISION --generation GENERATION --session-file SESSION_FILE --request-id REQUEST --report REPORT_JSON --json
cnr task dependency check --task ID --json
cnr audit task --task ID --json
```

Supply `--workspace` consistently when the workspace differs from the
default. Read the new task revision and lease generation after a mutation.
Each write request uses a separate business request ID. Reuse the ID only
with the original arguments to recover the committed response. The default
lease lasts 900 seconds, subject to the authorization expiry. An expired
writer can leave a live external process. Recovery requires operator review
and explicit confirmation that the external writer stopped. Use
`cnr task recover --stopped true` with the task revision, generation,
authorization actor and request ID.

Continue with the existing `cnr task verify`, review and acceptance gates.
A worker report supplies bounded assertions. CIRNETRULES seals actual input
bytes, changes, binding and dependency evidence before verification.
Local actor and session labels are assertions. A valid receipt digest does
not establish independent identity.

To upgrade an existing task record, stop older binaries and back up the
ignored task directory. Run `cnr task coordination migrate --confirm preview`
with the task ID, revision and authorization actor. Repeat with the reviewed
digest. Resume through ordinary revision and grant review as required.
Keep the original backup for recovery. Older binaries cannot be forced
to honor the new task fields.

## Portable CI gate

```sh
cnr task evidence export --task ID --output ABSOLUTE_BUNDLE_FILE --json
cnr task evidence inspect --bundle ABSOLUTE_BUNDLE_FILE --json
cnr ci verify --governance-root TRUSTED_ROOT --authority-commit TRUSTED_COMMIT --bundle ABSOLUTE_BUNDLE_FILE --expected-digest APPROVED_DIGEST --target CANDIDATE_CHECKOUT --json
```

Export requires a current sealed leaf task. The candidate is committed by the
operator on the approved base and CI observes the exact changed paths and
scoped file bytes. Supply the approved bundle digest and trusted authority
commit from a separate reviewed channel. Inspection only checks integrity.
The gate uses controlled Git and format checks from trusted policy and
checks consumed artifact digests when required. Product build, authenticated
review and hardware gates need their own evidence.

The reusable workflow `.github/workflows/ci-task-gate.yml` takes immutable
authority, candidate and evidence commits. Run it from a trusted caller
and supply the approved digest independently of PR input. Candidate scripts
are outside the workflow's execution surface. The gate re-reads raw Git
blobs and requires checkout bytes to match. If a checkout filter changes
line endings or content, prepare the candidate with the reviewed bytes
and repeat the check.

The gate receipt is the JSON result of `cnr ci verify`. Retain it in the
trusted runner's logs or artifact store. The portable bundle and receipts
carry digests and logical paths. Keep the private source task store
and live session files local.
