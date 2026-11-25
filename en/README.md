# CIRNET Common Rules & Agent Skills Toolkit

Start with the [standalone baseline](AGENTS.md) and the target repository's
applicable instructions. Use English for agent work and Vietnamese for review.
Read the relevant owner source, rule and [Skill](skills/README.md) for the task.

| Need | Read |
| --- | --- |
| Choose context for the task | [Context loading](docs/context.md) |
| Select architecture, find the owner and check Lab-First priority | [Architecture policy](policies/architecture.md) · [Repository routing](references/architecture-routing.md) |
| Implement, debug, refactor and verify the scoped change | [Execution Skills](skills/README.md) · [Testing](rules/testing.md) |
| Check authorization and specific constraints | [Permissions](policies/permissions.md) · [Coding](rules/coding.md) · [Security](rules/security.md) · [Cross-repo](rules/cross-repo.md) |
| Compare versions, contracts and selected artifacts | [Compatibility evidence](docs/compatibility.md) |
| Identify file ownership and check a move | [Proposed organization standard](rules/repository-organization.md) · [Repository map template](templates/repository-map.md) |
| Set up a client, package or restore a local bundle | [Setup](docs/setup.md) · [Distribution](docs/distribution.md) |
| Maintain bilingual documents and run checks | [Documentation policy](policies/documentation.md) · [Checks](hooks/README.md) |
| Adapt local instructions or record task work | [Local AGENTS](templates/AGENTS.md) · [Plans](plans/README.md) · [Specs](specs/README.md) |

Current candidate changes are in the [changelog](docs/changelog.md).
Product evidence is dated in [references](references/README.md).
See [toolkit verification](docs/toolkit-verification.md) for execution results
and [earlier bilingual verification](docs/bilingual-verification.md) for history.
Use [focused compatibility checks](docs/model-compatibility.md) and the
[single-session Candidate benchmark](docs/behavioral-evaluation.md) when a model
or tool boundary changes. The [benchmark archive](../reports/work-prompt-benchmark-history.md)
records historical runs.

[License](../LICENSE) · [Third-party notices](../THIRD_PARTY_NOTICES.md)
