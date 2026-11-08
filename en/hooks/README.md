<!-- doc-id: HOOKS -->
<!-- revision: 1 -->
<a id="manual"></a>

# Validator hooks

Use the same CLI at preflight and before handoff. The CI workflow invokes
validation and release-readiness checks independently. This directory is a
portable procedure, not automatic hook discovery. An external adapter must
check the repo scope before invoking governance and leave unrelated projects
untouched. Use dry-run before changing any installation. [Adapter limitations](../adapters/README.md)
