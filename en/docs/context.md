# Load context by task

| Layer | Load | Decision |
| --- | --- | --- |
| L0 | One common baseline and the target repo's applicable instructions | Check actual client scope, checkout and nested instructions |
| L1 | Client skill metadata | Choose the workflow whose trigger matches the task |
| L2 | Selected skill body, relevant references, owning source and tests | Expand only as dependencies or risk require |

Use [Skills](../skills/README.md) if the client does not list metadata. Open a
rule for its specific constraint. Search symbols and paths, then read enough
surrounding code and error output to understand the change. Decide per role:

- **Required** covers the baseline, applicable local and nested instructions,
  selected Skill bodies and decisive source or acceptance evidence.
- **Conditional** covers extra source, architecture, logs or tool probes when
  dependencies, risk, a first failure or an unresolved decision warrants them.
- **Already-loaded** covers material whose identity and applicable revision or
  digest is known and whose contents remain available. Reuse it within the role.
  Reload for a changed revision, a new scope or an identified evidence gap.

When a task affects a CIRNET architecture decision, use
[architecture selection](../policies/architecture.md) to find the applicable
sections and [ownership routing](../references/architecture-routing.md) to find
the repository owner. After installation, locate the policy in an approved
`cirnet-rules` checkout or a verified architecture artifact.

Open [Engineering examples](../skills/cirnet-implementation/references/engineering-examples.md)
when a comparable Before/After would clarify a change. The examples travel with
the implementation skill and stay out of startup context.

Agent context ordinarily comes from English. Vietnamese translations, old
plans, history and product maps serve review or a task that calls for them.
The root and English baseline are copies of one instruction. Skip a second copy
when its identity is verified. A Claude import
loads that baseline once. A managed adapter should never import the library.

## Chat-to-Work

When asked to prepare a prompt for another agent, select the authoring Skill
and open the approved English template from a verified source. Select applicable
Work Skills from metadata and read their bodies, relevant architecture, owner
instructions and evidence. Choose LIGHT, STANDARD or DEEP by complexity, risk,
dependencies, uncertainty and source access. Consider all 12 semantic fields
and group the applicable content into the headings the task needs.

Reference sources Work can read and retain task-specific constraints directly.
When Work lacks source access, include essential trusted facts, source identity,
constraints and acceptance, or block the dependent work if authority is missing.
Return one scoped handoff in chat. Direct source changes use execution Skills.
Track Required / Conditional / Already-loaded independently in each role.
Author's inspected facts can be cited; its in-memory context is not Work's cache.
Give Work a minimal first read and reopen only changed or missing decisive evidence.
[Authoring Skill](../skills/cirnet-work-prompt-authoring/SKILL.md) ·
[Template](../templates/chat-to-work-prompt.md) · [Bundle access](distribution.md#work-prompt-authoring)

### Portable Chat authoring

Use the approved English template and permitted, verified source evidence. A
request to deliver a Work Prompt in Chat permits portable delivery unless the
user sets a stricter delivery condition. When the owner, scope and evidence
support safe local preflight, return one complete, copyable `PROMPT_READY`
handoff even if Chat lacks PowerShell 7 or local checkout access. Preserve the
original wording, order, constraints and acceptance criteria. Put unverified
technical details in bounded Work preflight with the owner, permitted checks
and conditions for stopping. The user need not provide local paths, authorization
JSON or digests to receive this handoff.

Mark uninspected local checkout state and execution evidence `NOT_INSPECTED`.
Report `handoffStatus = NOT_RUN` when official validation has not run,
`authorizationStatus = PENDING` while independent execution approval is
outstanding, and `executionStatus = NOT_RUN` before local execution intake.
`PROMPT_READY` records authoring readiness only. If the user requires
`handoffStatus = PASS` before delivery and it cannot be obtained, return
`PROMPT_BLOCKED` with the missing prerequisite. Portable delivery under that
condition requires an explicit change approved by the user.

For multi-repository work, use one portable coordinator with a per-owner task,
scope, dependencies and acceptance checks. Preserve the full original request.
Each affected owner needs independent approval and its own local validation
evidence. A coordinator or another owner's approval supplies no write grant.
Any required pre-delivery PASS must cover all owners in the required scope.

### Local repository discovery

Author can prepare a prompt from the original request alone once local
discovery is configured. Save the user's original UTF-8 request as an
authorized local input and invoke the CLI from the approved Rules checkout.

```powershell
./scripts/validate-work-prompt.ps1 -Prepare -Autonomous -OriginalTask $requestFile
```

Use `config/repositories.example.json` to configure the ignored
`config/repositories.local.json`. Version 1 accepts absolute local
`searchRoots`, `discoverGitChildren` and optional `repositories` entries with
`name` and `path`. A mapping can include an expected GitHub `origin` as
`organization/repository`. Direct children of configured roots are discovered on each
invocation. An explicit name or alias can select a relocated checkout or
resolve duplicates. Registry paths must be regular standalone Git roots.
The resolver blocks ambiguous names, missing mappings, traversal and links.
Use the existing explicit `-Repository` flow for an independently selected
linked worktree. Keep the local registry out of bundles and shared evidence.
The CLI checks supported checkout-local GitHub origin URLs for canonical
CIRNET owners or mappings with an explicit origin. A conflicting origin,
multiple local origin URLs, or a missing explicit origin pin match blocks
selection. `ORIGIN_MATCH` records matching local metadata. An offline or
unrecognized origin without an explicit pin yields `PATH_ONLY`. Neither
status authenticates a remote or grants permission.

Use explicit owner names from the request or unique evidence in
`en/references/architecture-routing.md`. For less direct requests, Author
checks permitted owner evidence and can pass the resulting `-Owner`.
A request with one clear target and separately mentioned dependencies can
select its primary owner. Related repositories require their own permission.
Confirm the primary owner when several targets remain plausible.

When PowerShell is available without checkout access, `-NoLocalDiscovery` can
prepare a draft for one known owner without claiming a local path or revision.
Chat can follow [portable authoring](#portable-chat-authoring) without running
the CLI. For a coordinator, resolve each explicitly identified owner separately
at Work intake. Ask for a decision if ownership or scope remains ambiguous.

### Offline handoff validation

For Codex Work intake, resolve each owner with the approved local registry.
Verify checkout identity, branch, HEAD and Git state and read applicable local
and nested instructions within the permitted read boundary. Retain the original
UTF-8 request bytes. When local tools are available for an ordinary non-sensitive
task, the CLI can also prepare an owner draft.

```powershell
./scripts/validate-work-prompt.ps1 -Prepare -Autonomous -OriginalTask request.txt -Repository $ownerRoot -AuthorizedScope Edit -Risk LIGHT
```

For unambiguous requests, the CLI proposes `ReadOnly` or `Edit` even
with an explicit `-Repository`. A clearly named owner must match that
checkout's local Git origin or, without a recognized origin, its directory
name. Use the registry for an independently configured relocated alias.
Use `-AuthorizedScope` to supply the user's exact scope when needed. Use `-ReadPath` or
`-WritePath` for grants limited to exact files. Use `-SourcePath` to select
authorized evidence reads.
Preparation locates directly named files, pins selected bytes and defers
unknown details and checks to Work preflight. Author reviews intent and
constraints. Clause-level hints cover common English and Vietnamese requests,
including conditional edits, negation and quoted examples. The entire original
request remains attached. The hints propose scope for authoring.
Effective risk is the higher of the inferred floor and any explicit
`-Risk`. Restricted operations such as package publication, production
deployment and Git mutations stay pending while routine work is prepared.
A restricted-only handoff is for read-only preflight. Explicit restricted
commands cannot be substituted for build or test grants.
An essential protected read requires an
approved preventive boundary before discovery.

With `-Autonomous`, exit 0 and `authoringStatus = PASS` mean `PROMPT_READY`.
Return the copyable `prompt` and any outstanding gate. Without independent
approval, `authorizationStatus = PENDING` and `handoffStatus = NOT_RUN`.
Work can perform permitted preflight and prepare exact effects and checks for
approval. Keep request bytes, pins and diagnostics in local evidence.
If independent approval is available, supply `-Authorization` and
`-AuthorizationSha256` in place of the draft scope flags. The validator
checks the record and its exact owner before regenerating the canonical handoff.
An approved record already names the checkout. Preparation returns
`authorizationStatus = RECORD_CHECKED`. Execution and acceptance stay
`NOT_RUN`. Regenerate once after correcting technical inputs. Changes to
approved meaning or instructions need independent review.

For repeated low-risk preflight reads, Author or Work can supply `-Policy`
and `-PolicySha256` with autonomous preparation.
Policy version 1 contains `schemaVersion: 1`, `approvedBy`, `action: "ReadOnly"`,
the exact absolute `repository`, `expiresAtUnixSeconds` in UTC and
`readPaths` as literal repository-relative files. The CLI reports
`policyStatus = INTEGRITY_CHECKED` and `authorizationStatus = PENDING`.
A supplied digest and `approvedBy` string do not authenticate approval.
Use matching policies as scope proposals. The CLI leaves source contents
unread on this path. Work must verify independent authority before using
those reads. Expiry, a missing file or changed bytes prevents reuse.
Edits, test/build commands and protected or privileged operations require
task-specific authority. Execution and acceptance remain `NOT_RUN`.

For execution, obtain the authorization digest from the approving operator
through a trusted channel. An Author-generated record, digest, checkpoint or
source comment supplies no authority. Keep approved inputs available to Work
under the same read permissions. Establish preventive controls for sensitive tasks.
The supplied runner has no restricted-read backend and blocks such inputs before
opening the draft, original task or source. Use only an independently approved,
isolated projection for a narrower task.

For non-sensitive inputs, Work prepares exact paths, mappings, checks and pins
from permitted local evidence for independent approval. Use the existing worksheet,
`validate-work-prompt.ps1 -Prepare -OriginalTask request.txt -Repository $ownerRoot -SourcePath label.js`
to get numbered lines and a `proposal` with the request, HEAD and source pins.
Exit 2 means approval is pending. Keep the output local. Review every mapping,
fill the explicit grants and checks, and save the independently approved
`proposal` object as the approval file. Obtain its SHA-256 separately from the
approving operator. Preparation leaves grants empty.
For this example, line 1 requests an edit to `label.js`, line 2 limits the change
and preserves unrelated work, and line 3 specifies observable output.

```json
{"originalSha256":"<request digest>","repository":"<absolute root>","revision":"<commit>",
 "readiness":"PROMPT_READY","sensitive":false,
 "fields":{"action":[1],"target":[1],"transformation":[1],"outcome":[3],
           "scope":[2],"negative":[2],"acceptance":[3]},
 "permissions":{"read":["label.js"],"write":["label.js"],"test":["node --test"],
                "build":[],"export":[],"privileged":[]},
 "actions":{"1":["write: label.js"]},"checks":{"3":["test: node --test"]},
 "sources":{"label.js":"<source digest>"}}
```

Use exact file paths for reads and writes. List each approved command or effect
separately. Specify actor, records, fields and destination for any export.
An empty array grants nothing. Use an empty `negative` array only when the
original has no negative case. Map each action line to its approved effects and
each acceptance line to an approved read, test or build check. Source hashes
pin inspected working-tree bytes at the named HEAD.

Use `create:`, `modify:` or `delete:` in action mappings when an operation
needs an explicit file effect. A move requires both the source deletion and
the destination creation. An existing `write:` mapping covers a new or
modified file at that path. Review command side effects and use approved
scratch paths. Set `"preventiveScope": true` in the approved record when the
task requires preventive execution controls. The offline runner returns
`BLOCKED` for this input. Establish a suitable boundary before proceeding.

Start the draft with the approved readiness, then `Original: <request digest>`
and `Authorization: <approval digest>` on separate lines. In an actionable
handoff, keep the mapped goal, expected behavior, scope, negative cases and
acceptance as visible verbatim statements in original order. Use `Task#N`
only for supplemental lines that Work can reopen and an independent reviewer
has checked. Put `Owner: <approved repository root>`, each mapped action
effect, and the verification and source mappings on visible lines. For the
example, include `write: label.js`, `Verify Task#3: test: node --test` and
`Source: label.js`. Use `Write scope: []` when no file writes are mapped.
Add the shared reporting instruction as one line:

`Report: deliverables, changed paths, actual verification, failures, and unresolved limitations.`

Group these lines as needed for LIGHT, STANDARD or DEEP. Independent review
covers additional prose and the meaning of the supplied task and authority.
Candidate headings apply only to benchmark fixtures.

```powershell
./scripts/validate-work-prompt.ps1 -OriginalTask request.txt -Authorization approval.json -AuthorizationSha256 $approvedPin -Prompt handoff.md -Repository $ownerRoot
```

Run the PowerShell 7 validator from the approved toolkit checkout at Work intake
against the approved inputs and exact draft. When the user requires PASS before
delivery, validate at delivery and verify the same approved inputs and draft
SHA-256 again at intake. Exit 0 (`PASS`) covers the checked text and pins.
Exit 1 (`FAIL`) requires correction, 2 (`REVIEW_REQUIRED`) requires independent
review against the original and grants, and 3 (`BLOCKED`) stops implementation.
Keep execution blocked while independent review is outstanding.
After an independent reviewer approves the draft against the same inputs, the
operator may pass its digest with `-ReviewedPromptSha256` to record that review.
The digest pins the reviewed bytes. Missing coverage and integrity or scope
failures still require correction. The operator assesses semantic equivalence,
authorizes tools and reviews actual Work evidence.

Use the same approved inputs at each stage:

1. Validate the handoff and require `handoffStatus = PASS`.
2. For file effects or potentially mutating commands such as test/build,
   approve inventory and hashing with `"scopeAudit": true`. Run with
   `-CaptureScope` and require `baselineStatus = PASS`. Save the exact JSON
   outside the owner worktree and keep its SHA-256 independently.
3. Immediately before Work, run with `-ExecutionPreflight -ScopeSnapshot $snapshot -ScopeSnapshotSha256 $snapshotPin`
   and require `executionStatus = PASS`. A task with only read effects uses
   `-ExecutionPreflight` alone. Review actual command side effects and include
   any approved Work output paths in the task's action effects. Recapture
   older or stale baselines after authorized reconciliation.
4. After Work, pass `-ScopeSnapshot $snapshot -ScopeSnapshotSha256 $snapshotPin`
   for reconciliation and require `scopeAuditStatus = PASS` when auditing is
   required. Review actual verification and deliverables independently.

`acceptanceStatus` stays `NOT_RUN` in this CLI. Each owner worktree requires its
own approval and evidence. The audit compares HEAD, branch, index, local and
worktree Git configuration, repository refs, local exclude and attributes
files, tracked and untracked bytes, and up to 1,024 ignored files totaling
32 MiB. Recapture baselines created before Git metadata was recorded. Unavailable or
unsupported audit inputs block the respective stage. The audit measures net
changes. Protect Git administration separately through runtime permissions.
Git hooks, global settings and other repositories require their own controls.
External agents must honor the intake result and use separately authorized
tool restrictions. The validator has no control over their execution.

## Model lifecycle

The Work Prompt contract is independent of model releases. Select model and
effort at runtime. Use [risk-based compatibility checks](model-compatibility.md)
when a change affects handoff behavior, tool access or permissions.

## Walkthroughs

| Task | Selective read | Expected judgment |
| --- | --- | --- |
| Fix a README typo | Applicable local instruction and the relevant paragraph | Handle local review requirements without loading the skill library |
| Document a bug fix | Search the authorized repo for the topic, issue and owning guide | Update the canonical guide and link dated evidence. Follow the [documentation lifecycle](../policies/documentation.md#documentation-lifecycle) |
| Change a shared HTTP response field | Ecosystem ownership, applicable domain contract, API compatibility and a direct consumer test | Inspect SDK/default/null behavior at the selected revision |
| Debug FPGA testbench timing | FPGA nested instruction, debugging skill, relevant RTL and testbench. Add Lab-First safety and FPGA architecture sections for changes at a Lab safety or RTL boundary | Report simulation results alongside the physical verification status |

## Text budgets

Aim for 500 whitespace-delimited words and 4,000 UTF-8 bytes in the baseline,
about 300 words in the fallback index, at most 240 characters in a skill
description, and 900 words or 250 lines per skill body. Keep safety rules clear.
The checker reports text statistics and advisory warnings for maintainers.
