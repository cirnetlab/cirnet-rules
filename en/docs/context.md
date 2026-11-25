# Load context by task

| Layer | Load | Decision |
| --- | --- | --- |
| L0 | One common baseline and the target repo's applicable instructions | Check actual client scope, checkout and nested instructions |
| L1 | Client skill metadata | Choose the workflow whose trigger matches the task |
| L2 | Selected skill body, relevant references, owning source and tests | Expand only as dependencies or risk require |

Use [Skills](../skills/README.md) if the client does not list metadata. Open a
rule for its specific constraint. Search symbols and paths, then read enough
surrounding code and error output to understand the change. Decide per role:

- **Required** covers the baseline, applicable local and nested instructions,
  selected Skill bodies and decisive source or acceptance evidence.
- **Conditional** covers extra source, architecture, logs or tool probes when
  dependencies, risk, a first failure or an unresolved decision warrants them.
- **Already-loaded** covers material whose identity and applicable revision or
  digest is known and whose contents remain available. Reuse it within the role.
  Reload for a changed revision, a new scope or an identified evidence gap.

When a task affects a CIRNET architecture decision, use
[architecture selection](../policies/architecture.md) to find the applicable
sections and [ownership routing](../references/architecture-routing.md) to find
the repository owner. After installation, locate the policy in an approved
`cirnet-rules` checkout or a verified architecture artifact.

Open [Engineering examples](../skills/cirnet-implementation/references/engineering-examples.md)
when a comparable Before/After would clarify a change. The examples travel with
the implementation skill and stay out of startup context.

Agent context ordinarily comes from English. Vietnamese translations, old
plans, history and product maps serve review or a task that calls for them.
The root and English baseline are copies of one instruction. Skip a second copy
when its identity is verified. A Claude import
loads that baseline once. A managed adapter should never import the library.

## Model lifecycle

Select model and effort at runtime. Use [risk-based compatibility checks](model-compatibility.md)
when a change affects tool access, permissions or engineering behavior.

## Walkthroughs

| Task | Selective read | Expected judgment |
| --- | --- | --- |
| Fix a README typo | Applicable local instruction and the relevant paragraph | Handle local review requirements without loading the skill library |
| Document a bug fix | Search the authorized repo for the topic, issue and owning guide | Update the canonical guide and link dated evidence. Follow the [documentation lifecycle](../policies/documentation.md#documentation-lifecycle) |
| Change a shared HTTP response field | Ecosystem ownership, applicable domain contract, API compatibility and a direct consumer test | Inspect SDK/default/null behavior at the selected revision |
| Debug FPGA testbench timing | FPGA nested instruction, debugging skill, relevant RTL and testbench. Add Lab-First safety and FPGA architecture sections for changes at a Lab safety or RTL boundary | Report simulation results alongside the physical verification status |

## Text budgets

Aim for 500 whitespace-delimited words and 4,000 UTF-8 bytes in the baseline,
about 300 words in the fallback index, at most 240 characters in a skill
description, and 900 words or 250 lines per skill body. Keep safety rules clear.
The checker reports text statistics and advisory warnings for maintainers.
