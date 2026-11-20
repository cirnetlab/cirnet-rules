---
name: cirnet-work-prompt-authoring
description: Use when asked to write a Work Prompt or implementation handoff for Codex, Claude Code, Antigravity, or another agent to execute. Direct implementation requests use the applicable execution skill.
---

# Author a Work Prompt

Chat performs authorized read-only discovery and authors one scoped handoff.
Direct source changes use the applicable execution Skill. Work rechecks local
checkout, branch, instructions, permissions and evidence before implementation.

1. **Interpret.** Draft the goal and granted scope before discovery. Use the
   supplied task, including an embedded original request. Track its action,
   owner, file and symbol, transformation, outcome, scope, negative cases and
   permissions. Recheck the supplied request before declaring a missing goal.
   Repository access grants only reading.
2. **Locate Owner.** Identify the owner and write/read boundaries. Use
   `en/policies/architecture.md` and `en/references/architecture-routing.md`
   when ownership, consumers, contracts or a safety boundary need resolution.
   Read their governing sections from the approved toolkit, once per role.
3. **Select Authority/Skills.** Read the relevant English architecture sections
   and Rules. Inspect the 13 Skills' metadata, using `en/skills/README.md` when
   client metadata is unavailable. Read only Work Skill bodies whose triggers
   apply. List those exact names under Applicable Work Skills with their task
   responsibilities. Refer to shared workflows and carry decisive invariants.
4. **Gather Minimum Evidence.** Inspect accessible owner source, contracts,
   manifests and error output. Record claim, status, locator and impact or next
   check. Use `CONFIRMED` for observations, `INFERRED` for hypotheses with a
   resolving check, `NOT_INSPECTED` for unread inputs and `BLOCKED` for unmet
   prerequisites. Give verified repo/path/symbol/revision locators. For CI,
   include run URL/ID, commit SHA, job/step and first actionable error. Preserve
   historical results with their original inputs and date.
   Choose safe reads before running tools. Locate paths, structure, identifiers
   and allowed field names. Filter values before they reach tool output. Avoid
   dumping record stores, sensitive literals, credentials or environment
   contents. Keep values out of handoffs and evidence.
   A search for a field name may print its whole source line. Emit only the
   identifier or a value-filtered excerpt.
   For record data, prefer a verified bounded inspector returning structural
   metadata without evaluating source. Give Work its exact invocation and limits.
   Stop discovery once you can hand off safely. Follow the template's read
   recovery procedure and carry the applicable privacy controls into preflight.
5. **Draft Adaptive Prompt.** Open `references/chat-to-work-prompt.md` beside the
   installed Skill after bundle and installation verification, or use
   `en/templates/chat-to-work-prompt.md` from an approved checkout. Missing verified
   access yields `PROMPT_BLOCKED`. Choose `LIGHT`, `STANDARD` or `DEEP` by risk,
   complexity, dependencies and evidence. Consider all 12 semantic
   fields, grouping the applicable information into ordinary Markdown headings.

6. **Deduplicate.** Use Required, Conditional and Already-loaded context from
   `en/docs/context.md`. Reuse matching evidence. Keep facts, requirements and
   checks in one place with exact locators, applicable safety and permissions.
7. **Editorial Gate.** Review the draft for a usable goal, owner, scope, evidence,
   dependencies, negative cases, invariants and acceptance. Pair every criterion
   with a verified command/observation or ask Work to find the command in owner
   instructions. Compare goal, scope, constraints and acceptance to the
   supplied task.
   Keep supplied action, target and transformation explicit; a generic goal is
   insufficient when it loses the requested operation, even in LIGHT mode.
   Restore any lost operation or constraint before choosing how short to make
   the handoff. If the task is clear and authorized, use `NOT_INSPECTED` and Work
   preflight for unread source, unknown implementation details and test commands.
   Reserve `PROMPT_BLOCKED` for an indispensable missing decision, authority,
   unsafe operation or unresolved conflict that prevents a safe handoff.
   Use the offline handoff check in `en/docs/context.md` before delivery and at
   Work intake. Keep the approved effects and reporting requirements visible.
   Obtain the request and approval pin independently. Have the
   operator establish a preventive boundary before sensitive discovery.
   Execution results require Work's actual evidence.
8. **Hand Off.** Return `PROMPT_READY` and one complete copyable Work Prompt in
   the requested format. Exclude template instructions and unused placeholders.
   Report `PROMPT_BLOCKED` with the missing input, permission or decision when a
   safe handoff is unavailable. A separable task needs its own authorization.
   Include Work's permitted preflight and task-specific privacy, safety and read
   boundaries even in a blocked handoff, following the template's safe-inspection
   procedure. Blocking implementation does not protect later tool output.

For a continuing Codex or Claude Code task, reference the authorized Task ID,
checkpoint location and agent family. Ask Work to recover context and revalidate
Git, references and scope before editing. Have Work save before handoff. A
portable handoff requires separate disclosure approval.

## Access and authority

Model-agnostic handoff: specify required capabilities. Record evaluation
model and effort as runtime metadata (`en/docs/model-compatibility.md`).

Have Work establish applicable local/nested instructions, Rules, Skills and
owner sources once in its own role, reusing available matching revisions and
reopening changed or missing evidence before acting. If Work cannot read a
referenced source, include the essential trusted
facts, source identity/version, constraints and acceptance directly in the
handoff. Use that fallback only when reliable evidence establishes safe scope.
Mark unresolved portions `BLOCKED`, or the handoff `PROMPT_BLOCKED` if an
essential authority or decision is missing. Chat's lack of product-source access
can be `NOT_INSPECTED` with a Work preflight obligation when the owner and safe
scope are established. Keep template access distinct from Work source access.

Preserve applicable technology choices, `LAB_FIRST_PRIMARY`, exact pins,
immutable artifacts and provenance. `PARALLEL_ALLOWED` requires the architecture
policy's evidence. Independent Ecosystem work stays within its owners' bounds.
At physical Lab boundaries, distinguish Safety, Topology and Correctness, bind
to observed topology and preserve infrastructure-only routing and student
wiring. Use evidenced capabilities. Real E2E claims require the real selected
chain. Physical operation needs explicit authorization and owner safety evidence.

The handoff carries only granted permissions, including cross-repo writes,
commit/push/merge, publication and deployment. `PROMPT_READY` describes handoff
readiness. `IMPLEMENTATION_VERIFIED` requires fresh Work evidence. User-deferred
checks remain `NOT_RUN` with the reason, including `DEFERRED_BY_USER_PHASE_1`
where applicable.

<!-- bundle-resource: references/chat-to-work-prompt.md -->
