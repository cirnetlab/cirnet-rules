---
name: verification
description: Use when assessing completion, a changed behavior or a review claim that needs fresh evidence.
---

# Verification

Verify the requested claim within authorized scope; local instructions define
required checks and output locations. Verification does not authorize publication.

1. Identify the exact diff/input and acceptance claim. Map each claim to an observable
   check, distinguishing static review, unit tests, integration and physical evidence.
2. Inspect command side effects and prerequisites. Use the repo's allocator and
   installed tools; protect existing fixtures, artifacts and prior evidence.
3. Run the relevant checks and read output plus exit codes. Identify skipped cases,
   pre-existing failures and changed failures. Do not replace a missing check with
   a different kind of evidence without stating the limitation.
4. Verify affected English/Vietnamese documents agree on behavior, commands,
   plan/spec progress, results and limits. Run the local documentation hook when
   available. Matching hashes prove freshness only; inspect translation meaning.
   A missing or stale required counterpart prevents a full completion claim.
5. Inspect the final diff and status for scope, accidental output and sensitive data.
6. Report check, command/context, result and limitation. PASS means executed and
   expected; FAIL means executed and unexpected; NOT_RUN means unexecuted; BLOCKED
   means a required prerequisite or permission is missing. Narrow claims accordingly.

Example: an RTL simulation passes but no board is connected. Report simulation PASS
and physical verification NOT_RUN; make no electrical-readiness claim and do not
program or energize hardware without separate authorization and owner evidence.
