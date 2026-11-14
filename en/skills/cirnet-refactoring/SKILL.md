---
name: cirnet-refactoring
description: Use when changing internal structure while preserving observable behavior and supported inputs.
---

# Refactoring

Use for behavior-preserving restructuring. Treat a new behavior as a separate implementation change.

Minimum inputs: Current behavior, callers, boundaries, covered invariants and permitted files.

1. Identify observable inputs, outputs, state changes, errors and ordering.
2. Choose focused checks that protect those invariants. Move in small steps and update references together.
3. Compare before/after behavior. Keep public names, persisted formats and fixture meaning stable where promised.

Report structural changes, protected behavior and observed checks. Surface any behavior difference explicitly.

Example: Extract a parser helper while preserving invalid-input errors and offsets. Update callers and compare existing fixtures.
