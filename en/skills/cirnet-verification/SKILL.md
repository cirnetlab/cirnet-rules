---
name: cirnet-verification
description: Use when a completion, correctness or compatibility claim needs fresh evidence.
---

# Verification

Use to assess a claim. Use regression testing to design missing coverage.

Minimum inputs: Claim, exact revision/diff, expected result and local check prerequisites.

1. Map each claim and acceptance criterion to a check that can establish it.
   Confirm the revision, prerequisites and authorized side effects.
2. Run available checks and inspect their assertions, exit codes, outputs and
   skips. Use observed results to assess completion claims. Compare affected
   EN/VI instructions and review the final diff.
3. Distinguish source review, unit and integration execution, native client
   behavior and physical evidence. A skipped case is uncovered even when the
   command exits successfully.

List each command and PASS, FAIL, NOT_RUN or BLOCKED with its scope. For gaps,
state the missing prerequisite or unresolved check.

When RTL simulation passes, report the behavior exercised. Electrical readiness
and programming require separate owner-authorized evidence.
