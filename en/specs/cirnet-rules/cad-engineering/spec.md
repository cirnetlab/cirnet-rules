# CIRNET CAD engineering specification

## Scope and ownership

Give a CIRNET agent a repeatable way to specify, author, assess and hand off a
CAD artifact. CIRNET owns design requirements, evidence and acceptance. An
approved CAD tool owns parametric construction, kernel inspection and export.
Interface owns asset selection, rendering, runtime interactions and the
CircuitGraph. This change is confined to `cirnet-rules`. Read other checkouts
for their current contracts.

Use [the CAD engineering rule](../../../rules/cad-engineering.md) for the
operating procedure and the [plan](../../../plans/cirnet-rules/cad-engineering/plan.md)
for implementation evidence. The optional CAD Skill routes artifact work. The
engineering workspace UI Skill routes interaction design and testing.

## Acceptance criteria

1. A task specification records the use case, references, physical units,
   coordinate frame, required features, fit, materials, interaction datums,
   formats, budgets and unresolved choices. Requirements and preferences
   have distinct acceptance records.
2. An approved authoring workspace preserves editable parametric source and
   evidence with exact tool and input provenance. Construction, inspection,
   export and presentation review report separate outcomes. Missing tools
   leave the dependent stages `BLOCKED` or `NOT_RUN`.
3. The static intake CLI reads one independently pinned expectation and GLB
   from an explicitly selected local root. It reports measured SHA-256,
   bytes, active-scene mesh nodes, triangles and primitive instances.
   A mismatched artifact, malformed supported structure, missing or duplicate
   required name or exceeded reviewed budget fails within the intake domain.
   The gate checks actual finite vertex components and index values in active
   primitives, including bounds, declared POSITION extrema and buffer layout.
   Correct optional or required `KHR_materials_unlit` declarations are
   supported. Unavailable prerequisites, unsupported features and exceeded
   hard read or scan limits block the static intake.
4. A production adoption decision requires kernel and assembly evidence,
   review of rendered output and the owner's import, fallback and
   interaction checks. The static CLI covers its declared checks. It has no
   authority to adopt assets or change electrical connectivity.
5. The bilingual documentation pairs pass the toolkit checker. Skill selection,
   the existing installer defaults and opt-in distribution remain compatible.

Use the current text-to-cad CLI or independently approved MCP deployment
directly. A thin static verifier in Rules provides the artifact intake contract
without integrating Python or a CAD kernel into the Interface build.
