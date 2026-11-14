---
name: cirnet-contract-change
description: Use when evolving a canonical schema, serialization format, protocol, or ABI owned by a producer.
---

# Contract change

Use for producer-owned data or wire changes. Use API compatibility for HTTP operation semantics.

Minimum inputs: Canonical source and owner, current and proposed formats, selected readers/writers and codegen commands.

1. Compare encoding, bounds, field/enum identity, byte order, units and ABI layout where applicable.
2. Exercise old/new readers and writers, malformed data and saved fixtures in required directions. Generate bindings from the owner source.
3. Document versioning, deprecation, data migration and rollback limits. Coordinate owner-side changes within authorized scope.

Report contract revision, generated outputs, compared directions, results and missing evidence.

Example: A new enum value reaches an old strict reader. Test the actual fallback/rejection path before changing the producer version.
