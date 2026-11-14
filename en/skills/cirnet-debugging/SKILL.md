---
name: cirnet-debugging
description: Use when a failure, regression or flaky result needs a cause established before a fix.
---

# Debugging

Use for an unexplained symptom. Use implementation when the change and cause are already established.

Minimum inputs: Expected and actual results, revision, failing input, environment and redacted error.

1. Reproduce under local test rules and compare with a working case.
2. Trace the first failing boundary, form one falsifiable hypothesis and run the smallest experiment that distinguishes causes.
3. Apply a narrow fix after evidence supports it. Protect cleanup, cancellation and compatibility, then rerun reproduction and affected checks.

Report evidence, confirmed or unconfirmed cause, fix and rerun results. Say what prevented reproduction when blocked.

Example: A Stop request times out after cancellation. Trace cleanup acknowledgment before changing timeout. One passing retry does not establish the cause.
