---
name: cirnet-debugging
description: Use when a failure, regression or flaky result needs a cause established before a fix.
---

# Debugging

Use for an unexplained symptom. Use implementation when the change and cause are already established.

Minimum inputs: Expected and actual results, revision, failing input, environment and redacted error.

1. Reproduce under local test rules and compare with a working case. Record the
   exact input, observed symptom and first failing boundary.
2. Keep observations separate from hypotheses. Test one explanation with the
   smallest experiment that distinguishes it from plausible alternatives. Record
   what would confirm or rule it out.
3. Apply a narrow fix after evidence supports the cause. Preserve cleanup,
   cancellation and compatibility, then rerun the reproduction and affected checks.

Report evidence, confirmed or unconfirmed cause, fix and rerun results. Say what
prevented reproduction when blocked. A repeat needs a specific diagnostic reason.

A Stop request times out after cancellation. Trace cleanup acknowledgment before
changing the timeout. Treat a passing retry as one observation.
