# Load context by task

| Layer | Load | Decision |
| --- | --- | --- |
| L0 | One common baseline and the target repo's applicable instructions | Check actual client scope, checkout and nested instructions |
| L1 | Client skill metadata | Choose the workflow whose trigger matches the task |
| L2 | Selected skill body, relevant references, owning source and tests | Expand only as dependencies or risk require |

Use [Skills](../skills/README.md) if the client does not list metadata. Open a
rule for its specific constraint. Search symbols and paths, then read enough
surrounding code and error output to understand the change. Reload material when
revision, scope or missing context warrants it.

Open [Engineering examples](../skills/cirnet-implementation/references/engineering-examples.md)
when a comparable Before/After would clarify a change. The examples travel with
the implementation skill and stay out of startup context.

Agent context ordinarily comes from English. Vietnamese translations, old
plans, history and product maps serve review or a task that calls for them.
The root and English baseline are copies of one instruction. A Claude import
loads that baseline once. A managed adapter should never import the library.

## Walkthroughs

| Task | Selective read | Expected judgment |
| --- | --- | --- |
| Fix a README typo | Applicable local instruction and the relevant paragraph | Handle local review requirements without loading the skill library |
| Change an HTTP response field | API compatibility, producer contract and a direct consumer test | Inspect SDK/default/null behavior without loading FPGA references |
| Debug FPGA testbench timing | FPGA nested instruction, debugging skill, relevant RTL and testbench | Report simulation results with physical readiness unverified |

These are documentation walkthroughs. The separate
[behavioral evaluation](behavioral-evaluation.md) uses synthetic projects and
records native Codex observations when prerequisites permit.

## Text budgets

Aim for 500 whitespace-delimited words and 4,000 UTF-8 bytes in the baseline,
about 300 words in the fallback index, at most 240 characters in a skill
description, and 900 words or 250 lines per skill body. Keep safety rules clear.
The checker reports text statistics and advisory warnings. Token counts depend
on what the client loads and its tokenizer.
