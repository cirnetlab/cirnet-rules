---
name: cirnet-regression-testing
description: Use when a specific failure or behavior change needs a focused test that detects regression.
---

# Regression testing

Use to build coverage for a known risk. Use debugging to establish an unknown cause.

Minimum inputs: Observed symptom, expected public behavior, fixture, supported test lane and output policy.

1. Choose a minimal representative reproduction. Assert the user-visible result and relevant adverse case.
2. Run against the faulty behavior where feasible and confirm the intended assertion fails. Repair implementation and rerun.
3. Keep fixtures deterministic and avoid unrelated snapshots. Verify the test runs in the relevant suite.

Report what the test protects and the observed failing/passing results or unavailable baseline.

Example: A parser accepts a truncated frame. Check that it rejects the input without writing partial output. An unrelated crash is insufficient evidence.
