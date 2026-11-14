---
name: debugging
description: Use when a failure, regression, flaky test or mismatch between expected and observed behavior needs diagnosis.
---

# Debugging

Work within the authorized scope and the target repository's instructions.

1. Capture expected and actual behavior, relevant revision, input, error and
   environment with sensitive values redacted. Reproduce in the approved test lane.
2. Trace the failing boundary. Compare with a working case and form one falsifiable
   hypothesis. Check evidence before changing implementation or timeouts.
3. Run the smallest safe experiment that distinguishes causes. If reproduction is
   blocked, report observations, missing inputs and the next useful experiment;
   label the cause as unconfirmed.
4. Once supported, fix the cause narrowly and add a regression check when practical.
   Preserve safety, cancellation, cleanup and compatibility checks.
5. Update affected English/Vietnamese documentation together under local policy;
   distinguish a confirmed cause from an unconfirmed hypothesis in both versions.
6. Rerun the reproduction and affected suite. Report the cause, change, commands,
   PASS/FAIL results and any NOT_RUN or BLOCKED checks. Follow local output allocation.

Example: Stop times out after a cancelled request. Trace cancellation and cleanup
acknowledgement before increasing the timeout; verify both completed cleanup and
failed cleanup behavior. A passing retry does not establish why the failure occurred.
