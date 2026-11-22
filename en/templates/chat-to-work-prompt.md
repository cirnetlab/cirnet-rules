# Chat-to-Work Prompt template

## Authoring guidance for Chat

Use this single template for a scoped Work task. Chat performs read-only
discovery and authoring. Work owns local preflight, implementation, authorized
build/test and reporting. Render one complete handoff in ordinary Markdown,
using the user's requested format.

Draft the goal and granted scope from the supplied task before discovery,
including any embedded or quoted original request. Keep the requested action,
target and transformation explicit in the handoff. Recheck the supplied task
text before declaring the goal missing.

Consider all 12 semantic fields below. Group applicable information under the
headings the task needs. Omit inapplicable material and unused placeholders.
Keep these authoring instructions and the field checklist out of the final
handoff. Each fact, constraint and criterion belongs in one place.

Compare the draft goal, scope and acceptance against the user's requested
action, target, transformation, outcome, constraints, negative cases and
permissions. Restore any missing requirement before compressing. Keep this
comparison internal. Use autonomous preparation in
`en/docs/context.md#local-repository-discovery` before delivery. Include a
verified checkout path, or give Work bounded local resolution for the known owner. Derive
technical inputs from authorized reads and defer unknowns to Work preflight.
Regenerate after correcting mechanical issues. Keep owner, scoped effects,
acceptance, reporting and pending approvals visible. Have Work validate the
independently approved handoff and pass execution intake before acting. Include a
pinned baseline and require reconciliation for potentially mutating Work.

### Choose detail by task

Keep the handoff model-agnostic. State observable outcomes, permissions,
required tool capabilities and verified commands. Record explicitly requested
evaluation model and effort as runtime metadata. Work maps capabilities to its
available tools and marks steps `BLOCKED` when required capabilities are
unavailable. Preserve applicable safety checks and follow the
model compatibility procedure in `en/docs/model-compatibility.md` for runtime changes.

| Mode | Selection | Suggested presentation |
| --- | --- | --- |
| `LIGHT` | Small task with clear scope, low risk and few dependencies | About 5–6 concise heading groups |
| `STANDARD` | A feature, bugfix or refactor in one repo with straightforward dependencies | About 6–8 groups with explicit evidence, acceptance and verification |
| `DEEP` | Cross-repo or contract work, complex CI, FPGA/safety, security, user data, irreversible operations or limited source access | Up to 12 fields as separate sections, with extra sections where they resolve ambiguity |

Choose by complexity, risk, dependency, uncertainty and source accessibility.
User-request length does not determine mode. Heading ranges guide editing.
Keep all necessary evidence, negative cases and safety conditions even when
they require more detail. Use no hard token or word cap.

### Semantic completeness checklist

| Field | Task-specific content to consider |
| --- | --- |
| Task Identity | `{{TASK_NAME}}`, `{{TASK_ID_IF_NEEDED}}`, `{{TASK_TYPE}}`, owner `{{TARGET_REPOSITORY}}` |
| Objective | `{{USER_REQUEST}}`, `{{CURRENT_BEHAVIOR}}`, `{{EXPECTED_BEHAVIOR}}`, `{{EXPECTED_OUTPUT}}` |
| Repository Scope | `{{WRITABLE_REPOSITORY_AND_PATHS}}`, `{{READ_ONLY_REPOSITORIES}}`, `{{AUTHORIZED_ACTIONS}}` |
| Architecture Authority | `{{ARCHITECTURE_REFERENCES}}`, `{{APPLICABLE_DECISIONS}}`, `{{RULES_AND_SKILLS}}` |
| Source Evidence & Preflight | `{{SOURCE_EVIDENCE}}`, `{{WORK_PREFLIGHT}}` |
| Functional Requirements | `{{REQUIREMENTS_AND_NEGATIVE_CASES}}` |
| Technical Constraints | `{{TECHNICAL_CONSTRAINTS}}` |
| Implementation Workflow | `{{TASK_STEPS}}`, only the task-specific order, dependencies and decision points |
| Acceptance Criteria | `{{ACCEPTANCE_CRITERIA_AND_CHECKS}}` |
| Build & Verification | `{{VERIFICATION_COMMANDS_AND_PREREQUISITES}}` |
| Documentation & Deliverables | `{{DOCUMENTATION_AND_DELIVERABLES}}` |
| Final Report & Authorization | `{{REPORT_REQUIREMENTS}}`, `{{SEPARATE_AUTHORIZATIONS}}` |

### References, evidence and access

Name the approved checkout or trusted artifact and exact relative paths to the
applicable local instructions, architecture sections, Rules and Skills. Include
a verified revision when known. Under Applicable Work Skills, list only exact
names and their relevant responsibilities. Use metadata to select Skills, then
read applicable bodies. Reference shared workflows and include task-specific
decisions and decisive invariants directly.
Reuse already-loaded context when its identity is verified. Reopen source for
a changed revision or a gap that affects a decision. Stop discovery once the
objective, authority, scope and verification needs support a safe handoff.

Compress evidence using Claim / Status / Locator / Impact or Next check.
Use `CONFIRMED`, `INFERRED`, `NOT_INSPECTED` and `BLOCKED` per claim.
For CI, record run URL/ID, commit SHA, job/step and first actionable error.
For source, record repo/path/symbol/revision actually inspected. Identify unknown
locators or commands for Work to discover. Keep historical results bound to
their original inputs. Link accessible logs and source rather than copying
whole files. Include a short excerpt only when necessary to recover the evidence.
Choose safe source slices before invoking a tool. Inspect structure and allowed
field names first. Filter sensitive literals before output and exclude them from
the prompt and evidence. Mark inspection `BLOCKED` when safe access is absent.

On a failed read, reuse confirmed evidence first. Only retry with a verified
alternative when a fact is essential to a safe handoff and the alternative
stays within the same read scope and output limits. Honor access and
shell-policy denials. When the supplied operation, owner and safe scope are
clear, record unread sources or unverified commands as `NOT_INSPECTED`. Give
a scoped Work preflight with the locator, permitted check and stop condition.
Only block the handoff when an essential decision, authority, conflict or
safety condition prevents a safe handoff. Keep blocked reads and unrun checks
explicit.

Carry the task-specific safe-inspection boundary into the final Work text,
including `PROMPT_BLOCKED`. When export authority is missing, Work may confirm
local instructions, revision and scope and ask for the missing decisions.
Reuse the inspected projection and field-name facts with their locators; do
not require another record-store read merely to reconfirm the block. If an
independently necessary read is authorized, emit identifiers or value-filtered
projection code only. Remove sensitive literals in memory before any source
excerpt, matched search line, error or log reaches output. Unfiltered file output
and value-bearing search matches are unsafe. If filtering cannot be made
safe, keep that read blocked and request a safer access path.

When a verified bounded inspector is available, give Work its exact relative
path and metadata/projection invocation, source digest and output limitations.
Prefer its allowlisted identifiers over hand-written value-bearing searches.
An unrestricted shell can still disclose data; a later trace sanitizer or audit
cannot undo model exposure. Keep authorization questions and that limit explicit.

Have Work reconcile references with its authorized current source and reuse
matching evidence. For references Work cannot reopen, include essential trusted
facts, source identity/version, constraints and acceptance. Use this fallback
when reliable evidence establishes safe scope. Mark steps that lack essential
authority or evidence `BLOCKED`. The approved English template itself must be
accessible to Chat as described in the authoring Skill.

Carry the evidence needed for transient failures, difficult reproduction,
security, safety or compatibility. Preserve architecture authority,
`LAB_FIRST_PRIMARY` and any evidenced priority decision, per-repo ownership,
compatibility, exact pins and immutable artifacts. At physical Lab boundaries,
keep Safety, Topology and Correctness distinct, observed bindings and
infrastructure-only routing intact. Physical operation needs explicit
authorization and owner safety evidence.

## Generated handoff organization

For LIGHT or STANDARD, these groups can carry the relevant fields.

- **Goal & Expected Behavior** covers task identity, current and desired behavior.
- **Owner & Permissions** covers each writable area, read-only dependencies and
  granted actions. Reading a repo establishes no write permission.
- **References & Evidence** covers authority, Applicable Work Skills, locators,
  evidence labels and Work preflight.
- **Requirements & Constraints** covers functional and failure behavior,
  compatibility, security, data and safety invariants, plus task-specific order.
- **Acceptance & Verification** pairs every outcome with a verified command,
  test or observation, or directs Work to find the command in owner instructions.
  State environment/prerequisites and expected evidence.
  Group criteria covered by the same check, retaining every original assertion.
  Do not request the same unchanged test twice under focused/full-suite labels;
  repeat when source, inputs, scope or missing evidence warrants it.
  When filtering verification output, retain verified safe machine-readable
  stage markers, numeric child exit codes and assertion identities. Redact
  sensitive payloads without relabelling their output protocol.
- **Deliverables & Reporting** covers affected docs, EN/VI pairs where required,
  artifacts, destinations, changed paths, actual commands/exit codes and per-check
  `PASS` / `FAIL` / `NOT_RUN` / `BLOCKED`, compatibility impact and limits.

For DEEP, split Root Cause, Repository-by-Repository Plan, Contract Compatibility,
Safety Gates or Build/Test Matrix when needed. A root-cause heading retains
hypothesis labels until evidence establishes the cause. Multi-repo work names
each owner's write/read boundaries and acceptance separately.

Refer Work to applicable Skills/local instructions for the shared execution
workflow. Preserve negative cases, authorization checks, cleanup and Stop.
Require the real selected chain for real E2E claims and label mock/simulation
scope. Keep any user-deferred checks explicit with their reason.

State granted permissions and any separate authorization needed for Git changes,
publication, release, deployment and physical operation. Return `PROMPT_READY`
for an actionable handoff or `PROMPT_BLOCKED` for missing essential authority.
The handoff conveys the user's granted scope. Work supplies fresh evidence for
`IMPLEMENTATION_VERIFIED`.

Before handing off, review completeness, accessibility and repetition.
Expand missing critical detail or block the dependent task. Return only the
finished task-specific handoff and readiness status, without template directions
or a repeated generic Inspect → Plan → Implement → Test → Report sequence.
