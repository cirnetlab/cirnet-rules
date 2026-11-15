# Chat-to-Work Prompt template

Replace the placeholders with task facts and instructions for the Work Agent.
Keep all 12 headings. For small tasks, shorten entries and mark inapplicable
items explicitly. Use the same structure for Feature, Bugfix, Refactor,
Integration, FPGA, Performance, Documentation, Contract and Verification.
Return one prompt in chat by default.

## 01. Task Identity

`{{TASK_NAME}}` · `{{TASK_ID_IF_NEEDED}}` · `{{TASK_TYPE}}` · Owner `{{TARGET_REPOSITORY}}`

## 02. Objective

`{{USER_REQUEST}}` · `{{CURRENT_BEHAVIOR}}` · `{{EXPECTED_BEHAVIOR}}` · `{{EXPECTED_OUTPUT}}`

## 03. Repository Scope

`{{WRITABLE_REPOSITORY_AND_PATHS}}` · `{{READ_ONLY_REPOSITORIES}}` · `{{AUTHORIZED_ACTIONS}}`

Name writable paths or a bounded area for Work to confirm during preflight.
Record the permission for each affected repository and operation.

## 04. Architecture Authority

`{{ARCHITECTURE_REFERENCES}}` · `{{APPLICABLE_DECISIONS}}` · `{{RULES_AND_SKILLS}}`

Cite the applicable sections, owner and local instructions. Carry forward only
the decisions that constrain this task.

## 05. Source Evidence & Preflight

`{{SOURCE_EVIDENCE}}` · `{{WORK_PREFLIGHT}}`

Label evidence per claim. Use `CONFIRMED` for observed source or artifacts,
`INFERRED` for a hypothesis with a check to resolve it, `NOT_INSPECTED` for
unreviewed inputs and `BLOCKED` for an unmet prerequisite. Cite exact paths,
symbols, APIs, manifests, tests and revisions when verified. Have Work
recheck the checkout, scope and relevant evidence before choosing changes.

## 06. Functional Requirements

`{{REQUIREMENTS_AND_NEGATIVE_CASES}}`

State the required behavior, failure paths and behavior that must be preserved.

## 07. Technical Constraints

`{{TECHNICAL_CONSTRAINTS}}`

Record applicable runtime, security, safety, compatibility and performance
constraints. Identify owner contracts, exact pins and frozen artifacts.

## 08. Implementation Workflow

Inspect → Reproduce/Validate → Plan → Implement → Test → Review.

`{{TASK_STEPS}}`

## 09. Acceptance Criteria

`{{ACCEPTANCE_CRITERIA_AND_CHECKS}}`

Pair every observable outcome with a check and the evidence it should produce.

## 10. Build & Verification

`{{VERIFICATION_COMMANDS_AND_PREREQUISITES}}`

Give verified commands and environments, or ask Work to find them in the owner
instructions. A claim about real E2E requires the real chain and exact pins.
Report the scope of mocks or simulation. HIL needs separate authorization and
owner safety evidence.

## 11. Documentation & Deliverables

`{{DOCUMENTATION_AND_DELIVERABLES}}`

Identify affected EN/VI pairs, required artifacts and report destinations within
the authorized scope.

## 12. Final Report & Authorization

`{{REPORT_REQUIREMENTS}}` · `{{SEPARATE_AUTHORIZATIONS}}`

Ask for changed paths, actual commands, exit codes, `PASS`, `FAIL`, `NOT_RUN`
or `BLOCKED` per check, compatibility impact and limits. Record explicit
permission or prerequisites for Git changes, publication, release, deployment
and physical operation. The handoff grants no authority by itself.
