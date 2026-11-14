---
name: implementation
description: Use when implementing a scoped feature or behavior change with existing source, interfaces and tests to reconcile.
---

# Implementation

Use the authorized scope and applicable repository instructions as inputs.
For a simple text edit, follow local rules directly; this workflow is optional.

1. Inspect status, affected source, direct callers, producer contracts and nearby
   tests. State the expected behavior and observable acceptance checks.
2. For nontrivial work, outline the smallest coherent change. Write a plan/spec
   only if local rules or the task require it, in the owner's required location.
3. Add meaningful behavioral coverage where warranted, then implement within the
   permitted files. Generate derived outputs through the canonical producer path.
   If a required producer change is outside scope, report that dependency.
4. Run affected checks using local tooling/output policy. Inspect failures and the
   final diff for unrelated edits, secrets and generated artifacts.
5. Report changed files, behavior, checks and limits. Use PASS for successful executed
   checks, FAIL for unsuccessful ones, NOT_RUN for unexecuted checks, and BLOCKED
   for a missing prerequisite. Do not imply release or native runtime acceptance.

Example: a consumer needs a new payload field. Check its pinned contract first;
implement a compatible consumer change if possible, otherwise identify the owner-side
change needed. Reading the producer repo does not authorize modifying it.
