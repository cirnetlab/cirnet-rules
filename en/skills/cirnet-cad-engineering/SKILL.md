---
name: cirnet-cad-engineering
description: Use when specifying, authoring, inspecting, rendering, refining, or preparing CAD and 3D model artifacts for CIRNET.
---

# Engineer CAD artifacts

Read `en/rules/cad-engineering.md` from an approved `cirnet-rules` checkout.
For an installed Skill, use the checkout path and read access supplied by the
operator for the Rule and `scripts/verify-cad-asset.mjs`. If the Rule is
unavailable, report the CAD workflow as `BLOCKED` and identify the missing
source or access. For interactive workbench design, editing, selection, and
persistence, select `cirnet-engineering-workspace-ui`. When a task touches
both, use each Skill for its part.

1. **Understand and specify.** Confirm intent, references and the receiving
   owner's contract. Record units, measurable engineering constraints, visual
   preferences, unresolved decisions and acceptance checks. Label provisional
   dimensions and tolerances.
2. **Construct and inspect.** Verify the identity, version, command
   availability and prerequisites of the approved CAD capability. Use the
   supported text-to-cad CLI or MCP integration when available. Author editable
   parametric source in the approved workspace and inspect actual geometry,
   dimensions and assembly structure.
3. **Render, compare and refine.** Capture relevant engineering views and any
   requested presentation renders. Compare the images with the brief, record
   concrete defects, refine the affected features and rerun their checks.
4. **Verify, review and deliver.** For Interface asset preparation, run the
   checkout's preflight with Node and follow the owner's current contract.
   Keep electrical connectivity in `CircuitGraph` and preserve interaction
   anchors, transforms and procedural fallbacks. Record provenance and actual
   stage outcomes through `cirnet-verification` and use Task Continuity for
   long-running work. Report engineering checks, native execution and human
   visual approval with their own evidence. Mark unavailable or unexecuted
   checks `BLOCKED` or `NOT_RUN` as applicable.
