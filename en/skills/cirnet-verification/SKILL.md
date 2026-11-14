---
name: cirnet-verification
description: Use when a completion, correctness or compatibility claim needs fresh evidence.
---

# Verification

Use to assess a claim. Use regression testing to design missing coverage.

Minimum inputs: Claim, exact revision/diff, expected result and local check prerequisites.

1. Map each claim to a check that can establish it, accounting for scope and side effects.
2. Run the checks and inspect exit codes, outputs and skips. Compare affected EN/VI instructions and the final diff.
3. Separate source review, unit/integration execution, client behavior and physical evidence.

List each command and PASS, FAIL, NOT_RUN or BLOCKED with its scope. Finish when claims match evidence.

Example: An RTL simulation passes. Report its exercised behavior and leave electrical readiness and programming unverified.
