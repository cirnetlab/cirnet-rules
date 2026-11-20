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
and open the English template in an approved checkout. Select applicable Work
Skills from metadata and read only their bodies, relevant architecture, owner
instructions and evidence. Choose LIGHT, STANDARD or DEEP by complexity, risk,
dependencies, uncertainty and source access. Consider all 12 semantic fields
and group the applicable content into the headings the task needs.

Reference sources Work can read and retain task-specific constraints directly.
When Work lacks source access, include essential trusted facts, source identity,
constraints and acceptance, or block the dependent work if authority is missing.
The English template still needs separate checkout access as described below.
Return one scoped handoff in chat. Direct source changes use execution Skills.
Track Required / Conditional / Already-loaded independently in each role.
Author's inspected facts can be cited; its in-memory context is not Work's cache.
Give Work a minimal first read and reopen only changed or missing decisive evidence.
[Authoring Skill](../skills/cirnet-work-prompt-authoring/SKILL.md) ·
[Template](../templates/chat-to-work-prompt.md) · [Bundle access](distribution.md#work-prompt-authoring)

### Offline handoff validation

Before discovery the operator establishes the original request, the approved checkout
and a separate authorization record. Obtain its SHA-256 from the approving operator
through a trusted channel. An Author-generated record, digest, checkpoint or source
comment supplies no authority. Keep these inputs available to Work under the same
read permissions. For sensitive tasks, establish a preventive tool boundary first.
The supplied runner has no restricted-read backend and blocks such inputs before
opening the draft, original task or source. Use only an independently approved,
isolated projection for a narrower task.

The original is UTF-8 text. The operator reviews its numbered lines against the
grants and the acceptance checks before approving the JSON below. Replace the
placeholders with the exact local root, full HEAD and lowercase file digests.
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

Start the draft with the approved readiness, then `Original: <request digest>`
and `Authorization: <approval digest>` on separate lines. Keep action, target
and transformation explicit. In original order, use the exact lines or
`Task#N` for other nonblank lines that Work can reopen. Add
`Verify Task#3: test: node --test` and `Source: label.js` for the example.
The authorization reference carries the ceiling. Each optional action line, such
as `write: label.js`, must also match this task's actions, checks or source reads.
Ordinary Markdown and paraphrases can be
independently reviewed. Candidate headings apply only to benchmark fixtures.

```powershell
./scripts/validate-work-prompt.ps1 -OriginalTask request.txt -Authorization approval.json -AuthorizationSha256 $approvedPin -Prompt handoff.md -Repository $ownerRoot
```

Run the validator from the approved toolkit checkout before delivery and again
at Work intake against the same approved
inputs and draft SHA-256. Exit 0 (`PASS`) covers the checked text and pins.
Exit 1 (`FAIL`) requires correction, 2 (`REVIEW_REQUIRED`) requires independent
review against the original and grants, and 3 (`BLOCKED`) stops implementation.
Keep Work at `PROMPT_BLOCKED` while an essential review is outstanding.
After an independent reviewer approves the draft against the same inputs, the
operator may pass its digest with `-ReviewedPromptSha256` to record that review.
Machine validation cannot authenticate the approver, interpret arbitrary prose, run the planned
checks or enforce the worker's filesystem permissions.

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
