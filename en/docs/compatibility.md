# Compatibility evidence

Choose the domain before proposing a version. Toolkit, package/library, public
API, schema, protocol, generated SDK, firmware/board profile and system release
selection each have their own owner and version. Read the real selector in the
consumer and the source of truth in the producer. Some compatibility checks need
both writer/reader or sender/receiver directions.

| Boundary | Read this skill | Inspect |
| --- | --- | --- |
| Client-visible HTTP or public SDK | [API compatibility](../skills/cirnet-api-compatibility/SKILL.md) | Requests, responses, defaults, errors, auth and SDK behavior |
| Canonical data or wire format or ABI | [Contract change](../skills/cirnet-contract-change/SKILL.md) | Encoding, identity, lengths, old readers, generated bindings |
| Producer artifact and consumer pin | [Cross-repo compatibility](../skills/cirnet-cross-repo-compatibility/SKILL.md) | Exact revisions, selected input and required test lanes |
| Version proposal and user migration | [Release and changelog](../skills/cirnet-release-changelog/SKILL.md) | Owner version policy, breaking effects and recovery limits |

Classify source, wire, persisted data and behavior independently. Record evidence
in the [matrix](../templates/compatibility-matrix.md) at the owner-approved
location. A new optional field can still fail strict validation. Document any
data migration or irreversible step before recommending rollback. Release status
comes from the system owner and its selected artifacts.
