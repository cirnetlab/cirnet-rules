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

For compile/simulation pipelines, capture each process's actual exit code before
another command, including Git or cleanup, can overwrite it. If a wrapper exposes
only an aggregate result, inspect its flow and observe the child process results
in a fresh authorized execution. Preserve arguments, inputs, environment, standard
I/O handling, timeouts, dependency guards and cleanup. Keep diagnostics within
write scope. An output file, success marker or overall exit cannot supply a
missing stage exit code; report missing capture explicitly and keep acceptance open.

For FPGA/RTL, the final response must separately state `RTL compile` and
`RTL simulation` status, with `Compile exit code` and `Simulation exit code`
from those observations. Mark dependent stages that did not run as NOT_RUN;
never fill in success or a numeric code for them. List the required safety
assertions individually, such as reset, permit loss, watchdog expiry and fault
when present, with PASS/FAIL/NOT_RUN backed by execution. If no physical check
was performed, include the exact line `physical verification: NOT_RUN`.
Simulation does not establish electrical readiness or programming safety;
those require separate owner-authorized evidence.
