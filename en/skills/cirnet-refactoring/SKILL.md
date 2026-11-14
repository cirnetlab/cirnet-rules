---
name: cirnet-refactoring
description: Use when changing internal structure while preserving observable behavior and supported inputs.
---

# Refactoring

Use for behavior-preserving restructuring. Treat a new behavior as a separate implementation change.

Minimum inputs: Current behavior, callers, boundaries, covered invariants and permitted files.

1. Identify observable inputs, outputs, state changes, errors and ordering.
2. Capture focused checks for those invariants before editing. Move in small steps
   and update affected references together. Use existing structure where it fits.
3. Tie each hunk to the refactor or its verification. Remove imports or helpers
   made unused by the refactor after checking callers, generated references and
   dynamic use. Leave pre-existing dead code and unrelated formatting in place.
4. Compare before/after behavior with the same inputs and assertions. Keep public
   names, persisted formats and fixture meaning stable where promised.

Report structural changes, protected behavior and observed checks. Surface any behavior difference explicitly.

When extracting a parser helper, compare invalid-input errors and offsets on the
same existing fixtures.
