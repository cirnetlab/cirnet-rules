---
name: cirnet-regression-testing
description: Use when a specific failure or behavior change needs a focused test that detects regression.
---

# Regression testing

Use to build coverage for a known risk. Use debugging to establish an unknown cause.

Minimum inputs: Observed symptom, expected public behavior, fixture, supported test lane and output policy.

1. Choose a minimal deterministic reproduction. Assert the observable behavior
   the defect violates, such as the boundary result or exact ordering.
2. Run the new test against the faulty code when feasible. Check that the specific
   assertion fails with the reported symptom. Apply the focused fix and rerun the
   same test and affected suite.
3. Keep the assertion, timeout and real boundary under test. Reject edits to
   expectations, skipped tests or mocks that hide the defect. Verify the suite
   actually ran the new check.

Report what the test protects and the observed failure and pass. Identify any
unavailable run and the reason.

A parser accepts a truncated frame. Assert rejection and absence of partial
output. Confirm the failure comes from the parser behavior under test.
