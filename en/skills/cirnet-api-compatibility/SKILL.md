---
name: cirnet-api-compatibility
description: Use when changing an HTTP API or public SDK surface consumed outside the owning component.
---

# API compatibility

Use for client-visible API semantics. Use contract change for canonical wire/schema evolution.

Minimum inputs: Public operation or SDK method, version policy, current producer and selected consumer versions.

1. Compare request/response, optional/default/null, errors, auth, pagination, timeouts and SDK behavior.
2. Classify source, wire, data and behavioral effects. Try the actual selected consumer, including strict validators.
3. Propose a version or deprecation/migration only under owner policy. Check retry and rollback side effects.

Show changed operations, client directions, evidence and migration. Leave missing variants unknown.

Example: A response adds an optional field. Run a pinned client that checks additional properties before calling it compatible.
