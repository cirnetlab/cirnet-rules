# CIRNET Common Agent Rules & Skills

## Start here

1. Use the standalone [baseline](AGENTS.md); the root `AGENTS.md` is its identical
   distribution copy. Claude's root adapter imports that copy.
2. Follow [manual setup](docs/setup.md) to review and install English guidance or
   explicitly read the checkout. Preserve personal preferences and product instructions.
3. After implementing/building, update both affected documentation versions under
   the [documentation policy](policies/documentation.md) and run the local check.

## Select only relevant context

| Directory | Purpose |
| --- | --- |
| [hooks](hooks/README.md) | Preflight, after-change and completion checks; optional local checker |
| [plans](plans/README.md) | Small execution plans and central task records |
| [specs](specs/README.md) | Intended behavior, boundaries and acceptance |
| [documentation policy](policies/documentation.md) | English authority and paired updates |
| [permissions policy](policies/permissions.md) | Authorized actions and repository boundaries |
| [coding](rules/coding.md), [security](rules/security.md), [testing](rules/testing.md), [cross-repo](rules/cross-repo.md) | Focused implementation guidance |
| [implementation](skills/implementation/SKILL.md), [debugging](skills/debugging/SKILL.md), [verification](skills/verification/SKILL.md) | Three optional English skills |
| [references](references/README.md) | Dated source evidence; not product authority |
| [local instructions](templates/AGENTS.md), [plan](templates/plan.md), [spec](templates/spec.md) | Adaptable templates, never blind overwrites |

Build/test commands, versions, contracts, release selection, board configuration
and local planning conventions remain owned by each product. This repository
contains no orchestration service, task database, mandatory CLI or approval engine.
These instructions are standalone Markdown. The optional read-only checker uses
only the Rust standard library and requires an installed Rust toolchain to build.

## Maintenance and evidence

Author English, translate Vietnamese, review meaning, refresh translation metadata,
then check both. Do not load both language trees as default agent context.
See [current bilingual verification](docs/bilingual-verification.md) and
[historical rebuild evidence](docs/rebuild-verification.md).
The [bilingual upgrade plan](plans/cirnet-rules/bilingual-upgrade/plan.md) and
[spec](specs/cirnet-rules/bilingual-upgrade/spec.md) record this change.

The old Governance application is preserved on
[governance-cli-only](https://github.com/cirnetlab/cirnet-rules/tree/governance-cli-only).
It is not part of the common-rules runtime.
[License](../LICENSE) · [Third-party notices](../THIRD_PARTY_NOTICES.md).
