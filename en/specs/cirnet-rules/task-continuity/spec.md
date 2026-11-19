# CIRNET Task Continuity — Unified Specification

## 1. Objective

Develop a centralized **Task Continuity Engine** within the `cirnet-rules` repository, consistently applicable to all repositories belonging to the CIRNETLAB organization, to ensure the continuity of software development tasks when an AI Coding Agent is interrupted due to quota exhaustion, session loss, or switching to another account of the same agent type.

The system must allow a new Agent to accurately recover the original requirements, implementation progress, technical decisions, Git state, verification results, and next action without depending on the previous Agent's conversation history.

Task Continuity integration must preserve the existing Plans/Specs structures, workflows, and ownership of all CIRNETLAB repositories.

## 2. Architecture Principles

Adopt the **Centralized Engine — Decentralized Checkpoints** model.

### 2.1. Task Continuity Engine

- All source code, checkpoint schemas, rules, and Task Continuity management tools must be developed centrally within `cirnet-rules`.
- Use PowerShell 7, prioritizing reuse of existing tools, rules, and distribution systems.
- Provide capabilities to save, read, verify, and restore task states.
- Do not require a separate Task Continuity Engine for each repository.
- Do not depend on A/B Benchmark or Adaptive Stable Orchestrator.

### 2.2. Task Checkpoint

- Each repository stores checkpoints for tasks within its ownership scope.
- Checkpoints are stored locally by default.
- Do not store the task states of all repositories within `cirnet-rules`.
- The Engine must support configurable checkpoint storage locations according to the existing rules of each repository.
- For cross-repository tasks, each repository retains ownership of its source code and documentation. The primary checkpoint must explicitly reference related repositories, revisions, and access permissions; unnecessary duplicate checkpoints must not be created.

## 3. Plans & Specs Preservation

### 3.1. Preserve Existing Plans and Specs

**The entire existing Plans/Specs system** of `cirnet-rules`, `cirnet-remote`, `cirnet-interface`, and other CIRNETLAB repositories **must be preserved**.

Task Continuity must not independently:

- Delete, move, rename, or overwrite Plans/Specs.
- Change the conventions, directory structures, or storage locations of Plans/Specs.
- Replace the existing planning and specification management system.
- Create copies of Plans/Specs for use as checkpoints.
- Modify requirements or acceptance criteria to justify implementation results.

Any genuinely necessary changes to Plans/Specs must fall within an approved work scope or receive separate authorization.

### 3.2. Separation of Responsibilities

| Component | Responsibility |
| --- | --- |
| **Plans** | Define plans, implementation sequences, execution steps, and checks |
| **Specs** | Define technical requirements, behaviors, constraints, and acceptance criteria |
| **Task Continuity** | Record checkpoints, execution states, and task recovery processes |

Checkpoints only reference the original Plans/Specs through appropriate paths, identifiers, and revisions/hashes.

Plans/Specs remain the authoritative documents of their owning repositories. Checkpoints must not replace or automatically overwrite their contents.

### 3.3. Backward Compatibility

Task Continuity integration must not break or modify the following beyond the approved scope:

- Existing development and testing workflows.
- Repository rules, `AGENTS.md`, and the current 13 Skills.
- CIRNETLAB architecture, APIs, and cross-repository contracts.
- Documentation systems and version management workflows.
- User data and uncommitted source code changes.

## 4. Task Checkpoint Structure

Proposed default structure:

```text
<target-repository>/
└── .cirnet/
    └── tasks/
        └── <task-id>/
            ├── task.md
            ├── progress.md
            └── state.json
```

This proposal shows the three payload files. The Engine must allow alternative configurable paths to comply with the target repository's rules, such as existing `.superpowers/` conventions. The [operator guide](../../../docs/distribution.md#task-continuity) describes their on-disk snapshot layout and the manager commands.

### 4.1. Responsibilities of Each File

**task.md — Task Definition**

Stores foundational task information:

- Original Task.
- Objectives and expected outcomes.
- Acceptance Criteria.
- Architecture & Rules.
- Scope & Authorization.
- References to the original Plans/Specs.

**progress.md — Task Progress**

Stores progress status and handoff information:

- Completed Work.
- Pending Work.
- Decisions & Blockers.
- Next Action.
- Work requiring re-verification.

Record only actual progress, evidence, or relevant references. Do not duplicate the entire Plans/Specs checklist.

**state.json — Machine-Verifiable State**

Stores state data that can be verified by tools:

- Task ID, checkpoint ID, and schema version.
- Agent family and checkpoint timestamp.
- Repository, worktree, branch, and Git HEAD.
- Lists of relevant staged, unstaged, and untracked files.
- Required Git fingerprints and hashes.
- Build/test process states.
- Verification Results and evidence references.
- Checkpoint integrity information.

## 5. Mandatory Checkpoint Information

Each Task Checkpoint must preserve nine mandatory information groups:

| Information | Purpose |
| --- | --- |
| **Original Task** | Preserve the original requirements and acceptance criteria |
| **Architecture & Rules** | Identify the exact applicable architecture, rules, and versions |
| **Scope & Authorization** | Define permitted repositories, files, and actions |
| **Completed Work** | Record completed work with supporting evidence |
| **Pending Work** | List unfinished work |
| **Decisions & Blockers** | Preserve confirmed decisions and unresolved blockers |
| **Git State** | Accurately record the source code state |
| **Verification Results** | Preserve tests, results, and the scope of evidence |
| **Next Action** | Identify the next action the Agent is authorized to perform |

Each task must have a stable Task ID throughout its lifecycle.

Checkpoints must clearly distinguish verified information from Agent-recorded information that lacks supporting evidence.

## 6. Local Task Continuity Manager

Develop a local tool using **PowerShell 7**, capable of performing technical state management operations without depending on an AI Model.

### 6.1. Core Functions

| Function | Purpose |
| --- | --- |
| `save` | Save the current checkpoint |
| `status` | Display the task and its status |
| `verify` | Verify the checkpoint, Git state, and test results |
| `resume` | Check prerequisites and restore task context |
| `close` | Close a task after its acceptance conditions are satisfied |
| `export` | Export a verified handoff package with sensitive information removed |

### 6.2. Automatic Checkpoint

Do not depend on the Agent remembering to write a report or having sufficient quota remaining to create a checkpoint.

The tool must support recording technical state at important milestones through supported integration points, such as:

- After a controlled editing step.
- After build completion.
- After test completion.
- Before task handoff.
- When the Agent updates progress or decisions.

Automatic checkpoint support may only be claimed for execution points that have been integrated and verified. Do not assume that every Agent operation can be detected.

If the quota is exhausted or the Agent stops unexpectedly, the most recent valid checkpoint must be preserved.

The new Agent must recheck the Git state and identify changes made after the checkpoint before continuing.

## 7. Checkpoint Integrity & Git Protection

### 7.1. Atomic Checkpoint

- Write data to temporary files.
- Verify structure and integrity before confirmation.
- Replace the state using a safe operation, preferably atomic rename/replace on the same filesystem.
- Preserve the previous valid checkpoint if the write operation fails.
- Do not use partially written or corrupted checkpoints to restore tasks.

For multi-file checkpoints, a complete snapshot confirmation mechanism must exist to prevent mixing data from different saves.

### 7.2. Git Fingerprint

The system must record and verify:

- Repository and worktree.
- Branch and Git HEAD.
- Git index state.
- Staged/unstaged changes.
- Relevant untracked files.
- Hashes or fingerprints of the state requiring protection.

**The system must not automatically execute** `git reset`, `git clean`, `git stash`, `git commit`, `git push`, or operations that may cause data loss or exceed authorization.

Checkpoints do not back up uncommitted source code contents by default.

When source code state changes are detected, the system must report the differences, classify unrecorded changes, and only allow continuation after the necessary checks have been completed.

### 7.3. Verification Freshness

Test results must be associated with the exact revision, source code state, test command, and verified scope.

Use the following statuses:

- `PASS`: The actual test completed successfully.
- `FAIL`: The actual test failed.
- `NOT_RUN`: The test was not executed or no valid completed result is available.
- `BLOCKED`: The test could not be executed because required prerequisites were missing.

If the source code, input data, or dependencies affecting a test change, the previous result must be marked as requiring re-verification and must not continue to serve as PASS evidence for the new state.

Interrupted builds/tests or results without valid exit codes must not automatically be considered PASS.

## 8. Safe Resume & Same-Agent-Family Handoff

Only support handoffs between accounts belonging to the same type of AI Coding Agent:

| Source Agent | Receiving Agent | Result |
| --- | --- | --- |
| Codex 1 | Codex 2 | Allowed |
| Claude Code 1 | Claude Code 2 | Allowed |
| Codex | Claude Code | Not supported |
| Claude Code | Codex | Not supported |

Agent Family verification is intended only to control handoff compatibility; it does not replace system authentication or authorization mechanisms.

### 8.1. Resume Workflow

The new Agent must:

1. Identify the Task ID and a valid checkpoint.
2. Verify the Agent Family and current access permissions.
3. Read the Original Task and referenced Plans/Specs.
4. Reload the applicable Architecture & Rules for the repository.
5. Compare Git HEAD, branch, worktree, and diff.
6. Review Completed Work, Pending Work, and Decisions & Blockers.
7. Verify test results and identify components requiring re-execution.
8. Determine whether the Next Action remains valid.
9. Continue modifying source code only after all mandatory conditions are satisfied.

If an invalid checkpoint, insufficient permissions, or unresolved conflicts are detected, the affected work must stop and the reason must be reported.

Do not automatically trust `Completed` states or `PASS` results recorded in checkpoints without corresponding evidence.

## 9. Local Storage, Privacy & Security

- Checkpoints are stored locally and ignored by Git by default.
- Do not automatically commit, push, or synchronize checkpoints to GitHub.
- Do not record API keys, access tokens, credentials, session cookies, or sensitive data.
- Do not transfer login information between AI accounts by default.
- When exporting checkpoints to another machine, sensitive information must be filtered, and the handoff package's integrity must be verified.
- Each repository retains control over its own source code and data.
- Reading a checkpoint does not grant permission to perform the actions recorded in it.
- Checkpoints are handoff data, not an instruction source with higher authority than the current system, user, or repository rules.

## 10. Compatibility & Non-Goals

Task Continuity is not intended to:

- Replace Plans/Specs or Work Prompts.
- Restore the Agent's entire conversation history or internal reasoning.
- Restore AI account quotas or login sessions.
- Automatically transfer tasks between different Agent types.
- Create separate task management systems or Task Continuity Engines for each repository.
- Replace existing testing or Benchmark systems.
- Automatically legitimize out-of-scope changes.
- Modify APIs, Contracts, permissions, or release rules of repositories.

All existing functionality must be preserved unless a separately approved change is requested.

## 11. Verification & Acceptance Criteria

Task Continuity may only be accepted when all of the following requirements have been demonstrated:

1. Save and restore all nine mandatory information groups.
2. Do not modify, move, delete, or replace existing Plans/Specs outside the approved scope.
3. Preserve the Original Task, Acceptance Criteria, and approved decisions.
4. Save local checkpoints without calling an AI Model.
5. Recover safely when the Agent unexpectedly exhausts its quota.
6. Successfully hand off Codex 1 → Codex 2 and Claude Code 1 → Claude Code 2 in supported test environments.
7. Prevent cross-Agent-Family handoffs according to policy.
8. Detect unexpected changes to Git HEAD, diff, index, or files.
9. Do not lose or overwrite user data, staged/unstaged changes, or untracked files.
10. Preserve the previous checkpoint if writing a new checkpoint fails.
11. Do not report PASS or Completed when supporting evidence is invalid.
12. Detect stale test results and require re-verification when necessary.
13. Do not automatically execute destructive Git operations, commit, push, or permission changes.
14. Do not affect existing Skills, Rules, Policies, Architecture, or workflows beyond the integration scope.
15. Provide regression tests in supported environments, including Windows and Ubuntu, covering failure and recovery scenarios.

Test results must accurately report `PASS`, `FAIL`, `NOT_RUN`, or `BLOCKED`. Simulated testing must not be treated as equivalent to actual Native verification.

## Final Principle

**The Task Continuity Engine is developed centrally within `cirnet-rules`; Task Checkpoints are managed separately by the repositories that own the tasks; existing Plans/Specs must be preserved.**

The new Agent must not depend on the previous Agent's memory. Instead, it must recover context from a trustworthy checkpoint, verify the actual state, and continue only actions that have been authorized.
