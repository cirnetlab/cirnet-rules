# Choosing and interpreting checks

Use this guide to decide what evidence a change needs. Result labels and reporting
requirements are defined in the standalone [baseline](../AGENTS.md).

| Change | Useful evidence, subject to local requirements |
| --- | --- |
| Text or links | Read rendered/source context, resolve links, inspect diff |
| Logic or bug | Focused behavioral regression plus affected local suite |
| API/serialization | Producer contract, compatibility fixtures and direct consumer checks |
| Authorization or cleanup | Denial, cancellation, timeout and failure behavior |
| Device-facing code | Software tests plus separately authorized owner-defined physical evidence |

Read test scripts before execution when their side effects are unclear. Use the
repo's allocator/output directory; determine whether a command downloads tools,
changes locks, starts services or needs fixtures before running it.
Use [verification evidence](../docs/distribution.md#verification-evidence)
when previewing owner checks, assessing required stages or bounding retries.

Record command, working directory, exit code, result and scope of evidence. Name
missing prerequisites. A skipped case is not covered by an overall green exit.
Separate pre-existing failures from new regressions without hiding either.
Broaden testing for affected dependencies or unresolved risks; do not repeatedly
run the same passing checks without a reason. A source review, mock, synthetic
fixture and installed runtime test establish different claims.

For compile/simulation or device-facing checks, retain each process exit code
before another shell command overwrites it. Inspect wrappers that expose only
an aggregate result; capture actual stage results without changing their inputs,
environment, process handling or safety gates. Output files and success messages
alone do not prove PASS. Keep dependent stages that did not execute as NOT_RUN.
Report compile and simulation statuses and observed exit codes separately, name
the assertions actually exercised, and state `physical verification: NOT_RUN`
when no physical check occurred. Missing capture remains an evidence gap; do not
invent a code or equate software simulation with physical verification. The
[verification skill](../skills/cirnet-verification/SKILL.md) defines the FPGA/RTL
report fields and preserves this distinction when installed independently.
