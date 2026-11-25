---
name: cirnet-code-review
description: Use when asked to review a diff or design for actionable correctness, regression or security risks.
---

# Code review

Use to assess a change. Use implementation for a fix that has already been agreed.
Edit only if the review request includes fixing findings.

Minimum inputs: Diff or proposal, intended behavior, affected callers and verification evidence.

1. Read the change and upstream contracts. Trace realistic normal and failure
   paths. Map each changed area to a requirement, observed defect or check.
2. Check data handling, concurrency and compatibility where relevant. Flag
   speculative abstractions, unrelated edits and missing evidence that affect
   the task. Check whether test assertions would catch the reported failure.
3. Prioritize findings by user impact. Separate proven issues from questions.
   Point to the smallest relevant file range and state the condition that
   exposes the issue.

Return actionable findings first, then test gaps and limits. If none are found,
say what was reviewed. Keep fixes within the authorized scope and record other
issues as findings.

A retry interceptor also retries non-idempotent writes. Show when duplicate
effects can occur and identify the affected call site.
