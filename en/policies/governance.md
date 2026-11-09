<!-- doc-id: POLICY -->
<!-- revision: 2 -->
<a id="release"></a>

# Release, approvals and transitions

## REL-01 — Exact selection

The release train is `0.2.4`. The family is the string `"0.2"`.
A contract revision such as `0.2.0` has an exact source commit and digest.
Governance is independently versioned in `VERSION`. Lock toolchain and
external formats independently. See [terms](../references/glossary.md).

Remote alone owns `integration/release-set.json`. Keep existing published
tags and bytes immutable. Verify all six component identities and the
approved matrix digest, compatibility domains, exact artifacts and evidence
before activation. A matching label or a version at or above the baseline
does not grant compatibility. The [release schemas](../schemas/README.md)
describe proposed owner inputs. The current registry is `NOT_ACTIVATED`.

## Cutover and rollback

This one-time compatibility reset is a breaking transition. Following explicit
activation, decline pre-baseline CIRNET profiles and unlisted combinations
before side effects. Remove active legacy parsers, aliases and fallback paths
through reviewed changes. Retain historical tags and negative fixtures outside
active discovery. Authentication and authorization are independent of version
and digest. Binary contracts must define a bounded, explicit wire encoding.

After activation, compatible fixes may use patch releases. A wire, source or
behavior break needs an ADR, consumer review and a new family or revision
appropriate to the contract. Preserve OpenAPI dialect, JSON Schema dialect,
protobuf syntax, database migration sequence, compiler standards and board
identity as distinct domains.

Rollback only to a separately approved complete bundle with matching data.
If none exists, stop safely and use a forward fix or approved restore. Existing
data migration requires an explicit offline Rust tool, backup, dry-run and
verified output or an authorized fresh dataset.

## Authority and approval

Authors propose design as `DRAFT`. The owning maintainer records a real
approval reference before `APPROVED`. A replacement or retirement marks
`SUPERSEDED` or `RETIRED` and removes the source from active instructions.
Historical cutover records retain `PLANNED` and `RELEASED`. New local tasks
move `PROPOSED` → `READY` → `IN_PROGRESS` → `IMPLEMENTED` → `VERIFIED` →
`ACCEPTED` → `DOCS_SYNCED` → `CLOSED`. An unmet prerequisite yields
`BLOCKED`. An explicit documentation waiver can close an accepted task.
Release maintainers record publication separately. Verification binds checked
inputs to the governance and task revision. See the [CIRNETRULES workflow](../references/cirnetrules-local.md).

The initial exception list is closed. A future scoped exception requires
rule ID, owner, rationale, expiry and approval. An agent cannot waive its own
gate. Rules and hooks supplement filesystem permissions, CI and interlocks.
