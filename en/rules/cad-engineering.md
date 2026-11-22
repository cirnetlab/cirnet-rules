# CIRNET CAD engineering

Use this rule for authoring, inspecting, rendering or preparing an owned CAD
artifact. CIRNET owns the design, evidence requirements and acceptance.
Use [engineering workspace UI](engineering-workspace-ui.md) for behavior of
the editor around an artifact. Use the [architecture routing](../references/architecture-routing.md)
and check the product team's current source before adopting an export.

## Specify the design

Use the existing [task specification](../templates/spec.md). Record the
following design inputs before constructing the model.

| Record | Design inputs |
| --- | --- |
| Intent and constraints | Use case, reference images and their approval, mandatory features, preferences, assumptions and open choices |
| Geometry and fit | Units, dimensions, proportions, tolerances, datum and axis directions, part hierarchy, assembly poses, clearance, symmetry, profiles, curvature, fillets, vents and openings |
| Look | Material and finish by named surface, colors, PBR values, detailing, decals and reference views |
| Interaction | Stable part names, connector/selection datums, local pivots and motion axes, owner endpoint IDs where applicable |
| Delivery | Editable parameter source, target formats, target runtime, revision lookup, bytes/geometry/GPU budgets and measurable acceptance by stage |

Give every required dimension and feature an observation method. Identify
which requirements are mandatory and which preferences need visual review.
Record provisional values and obtain approval for unresolved choices that
affect fit, behavior or safety. Compare each revision to the original intent.

## Build and verify the artifact

1. Understand and specify, then construct editable parametric parts in the
   authorized workspace. Name durable features, references and assembly parts.
2. Inspect the actual kernel result. Record construction and Boolean failures,
   empty solids, kernel validity, feature dimensions and assembly clearance or
   overlap for each required pose. Match the specification's dimensions and
   units. Add targeted native assertions when the tool summary lacks coverage.
3. Render and compare. Take engineering snapshots in relevant orthographic
   front, rear, both sides, top and bottom views, with isometric and detail views
   for fit, shape and occlusion. Use consistent viewpoints and scale for
   comparisons. Review visible seams, curvature and tessellation at the
   intended viewing distance.
4. Refine only observed defects. Preserve the parameter source and first
   failure, rerun affected assertions and renew stale evidence. Use the
   [correction workflow](../docs/distribution.md#bound-correction) to stop
   when evidence, authorization or progress is insufficient.
5. Verify each export, then review and deliver. Keep command and working
   directory, source revision and input SHA-256, exact tool identity/version,
   per-stage exit code, assertion counts, bounded report locators and
   artifact hashes. Use [Task Continuity](../docs/distribution.md#task-continuity)
   for checkpoints and [claim verification](../docs/distribution.md#assess-required-stages)
   for current scoped evidence. A failed operation stays `FAIL`. A missing
   prerequisite is `BLOCKED`. An unexecuted or skipped check retains its
   own status and leaves a required check incomplete.

### Presentation review

Compare actual presentation renders with the approved reference. Review
silhouette, proportion, consistent bevels and surface finish, detailing,
material response, lighting, shadows, reflections, framing and legibility
at intended display scales. Record view, setting, defect, affected requirement
and the reviewer's decision. Use matched-view image comparisons to find
regressions. A human reviews aesthetic acceptance. Capture engineering
snapshots and presentation renders as separate artifacts.

## Static GLB intake

Use the checkout's dependency-free Node gate before submitting an embedded
static GLB candidate. The operator supplies an independently reviewed
expectation and pin. It contains the artifact path, artifact SHA-256,
maximum bytes, maximum triangles, maximum primitive instances and any
required node names. Use owner-provided budgets. The gate counts active-scene
uses of a mesh separately. Primitive instances are a draw-cost proxy.

```json
{
  "schemaVersion": 1,
  "path": "out/enclosure.glb",
  "sha256": "<independently reviewed artifact SHA-256>",
  "budgets": { "bytes": 1500000, "triangles": 20000, "primitiveInstances": 120 },
  "requiredNodes": ["enclosure_base", "enclosure_lid", "connector_usb"]
}
```

The limits above are fixture examples. From the approved Rules checkout,
with Node installed and an independently retained lowercase expectation pin:

```powershell
node ./scripts/verify-cad-asset.mjs --root $approvedAuthoringRoot --expectation intake.json --sha256 $expectationPin
```

Both file paths are literal and relative to the authoring root. Use an
owner-controlled workspace after the export finishes. The gate caps
input at 32 KiB for the expectation and 32 MiB for the GLB. It reads without
launching an authoring tool or fetching files. The supported profile is GLB
2.0 with embedded buffer data, PNG/JPEG images when present, and static triangle
primitives. It accepts correctly declared `KHR_materials_unlit` in optional
or required form. Unsupported extensions and scan limits yield `BLOCKED`.
The gate checks the container, selected scene tree, node name types and
transform declaration shapes. Invalid camera references in the selected scene
yield `FAIL`.
A camera referenced by that scene is outside the static profile and yields
`BLOCKED` when its reference is valid. For active primitives the gate checks
attribute counts, buffer ranges, stride and alignment, finite float values,
decoded index bounds and reserved index values. It compares POSITION bounds
and declared index bounds with decoded values. For active materials it
verifies core glTF factor types, lengths and ranges, alpha declarations,
the double-sided flag, normal scale and occlusion strength. It also verifies
material texture bindings, sampler references, required UV sets and declared
PNG/JPEG signatures. It reserves image buffer views for image data. Distinct
geometry scans have a cumulative 32 MiB budget.
Each required name must occur once in the selected scene. The JSON report
uses `PASS`/0, `FAIL`/1 or `BLOCKED`/3.

Use the owner's checks for mesh surface quality, normal direction and length,
world dimensions, transform behavior, image decoding and material appearance.
Optional texture-coordinate extrema are outside the static bounds comparison.
Measure memory and draw calls in the owner runtime. Keep native CAD,
visual approval and product evidence separately. An independently reviewed
SHA-256 pins bytes. Record who produced the file and how the owner established
that provenance.

[CAD engineering specification](../specs/cirnet-rules/cad-engineering/spec.md)

## text-to-cad execution

The inspected source is `earthtojake/text-to-cad` at `7687743d80305ba112666aee0a5e8c12af138458`, release `0.11.15`. The Codex plugin manifest loads its Skills and `codex.mcp.json`, whose stdio command is `uvx --no-config --managed-python --python 3.14 cad@0.11.15 mcp` with a 300-second startup timeout. MCP serves the local viewer. Codex's MCP contract delegates inspection, snapshots, and export to the CLI. Starting the stock plugin can fetch the runtime and provision a global tool. Use an already authorized installation for viewer interaction and leave plugin activation to the owner.

The command examples apply after verifying the absolute executable path and `cad --version` against the approved release. Preflight a matching Python environment carrying `cad[build123d]` and working build123d/OpenCASCADE imports. The engine can install a missing kernel base or script environment on demand, including a network retry. With installs out of scope, require a pre-provisioned runtime. Prefer its Python for the model script, which the CLI reference supports. Use the documented `cad run <script.py>` only when its matching environment is ready and its resolution behavior is authorized.

Run in an approved isolated authoring root, with engine, state, cache and output paths beneath it. Set `CAD_ENGINE_DIR`, `CAD_STATE_DIR`, `CAD_CACHE_DIR`, `CAD_UPDATE_CHECK=0` and `CAD_TELEMETRY=0` in the process environment. The daemon and its Python execute with the launching account's access. These directories route state and outputs. Use enforceable write and network bounds for the daemon and author only reviewed local scripts. Keep bearer tokens and authenticated viewer URLs out of evidence shared with others. Stop only the isolated server you started.

```powershell
& $Cad --version
& $Cad server start
& $CadPython -B ./src/enclosure.py
& $Cad model inspect $Id --json
& $Cad model interference $Id --json
& $Cad model snapshot $Id ./out/iso.png --camera iso --json
& $Cad model export $Id ./out/enclosure.step --json
& $Cad model export $Id ./out/enclosure.glb --tolerance 0.05 --angle 5 --json
& $Cad glb inspect ./out/enclosure.glb --json
& $Cad server stop --json
```

Create `out` under the approved root. `$Cad` and `$CadPython` are the verified executable paths, and `$Id` comes from the current build. The script explicitly calls a zero-argument `@model` returning a build123d shape and labels assembly children. Inspect exact bounds, volumes, and required references. Add targeted solid validity, clearance and feature assertions on native shapes. Check interference per required pose and retain unmeasured pairs. Inspect snapshot and export warnings for unmeshed faces, even on exit zero. The GLB inspection lists nodes, meshes and animations. Validate required names, transforms and product budgets in the exported artifact. The exporter applies `(x,z,-y) × 0.001` at the root, taking CAD millimetres and +Z up to GLB metres and +Y up. Keep child transforms and independently verify world scale and front. Optional presentation renders load the GLB using `cad.blender.load` with already provisioned `bpy 5.2.3` on Python 3.13. Capture actual presentation images separately from OCCT snapshots.

### Synthetic enclosure pilot

Provisional fixture in CAD millimetres with a footprint-centred origin on the base plane. Model an 80 × 50 × 21 enclosure with an 18 mm base, 3 mm lid, 2 mm shell walls, R3 outside corners and a 12 × 6 front opening centred at CAD `(0,-25,10)`. Use `enclosure_base`, `enclosure_lid` and `connector_usb` labels. Assert valid solids, required labels, stated dimensions and radii within ±0.05 mm, a through-opening, at least 0.5 mm clearance around the connector in the aperture, and zero unintended overlap in the assembled pose. Capture front, back, left, right, top, bottom, iso and an interior detail with the lid hidden. Check exported node names, a 0.080 × 0.021 × 0.050 m world envelope, +Z as the GLB front, and source and artifact SHA-256 identity. This is a planned native fixture. Record blocked or skipped operations with the reason when the runtime is unavailable.

In a separate fixture run, omit the connector datum and check that the named
node requirement fails. Confirm exported nodes directly on the GLB. At the
inspected revision, the GLB writer can share sibling nodes with the same label
and omits empty paths. It emits vertex positions and normals and supports
material parameters. Textured deliverables need an owner-reviewed pipeline
for UVs and images. See the tool's `packages/cad/src/cad/engine/export/glb.py`.

## Interface asset handoff

Pin the Interface revision and identify the target asset profile before export.
The reviewed preview contract is at Interface
`82dd99a62c899aa413d03856c41e252bf289ba22` (Three.js 0.171.0). The workbench
constructs both instruments with procedural factories. The standalone previews
load the owned GLBs and report when they use a procedural fallback. Preserve
both paths, placement, selection, transforms, persistence and Undo/Redo.
(`docs/en/guides/instrument-models.md:5–29`,
`src/three/ComponentModel.ts:346–349`, `src/preview/instrumentPreview.ts:57–75`.)

Bench previews use Y up, +Z front, a center-bottom footprint origin and an
identity asset root. Supports and rear enclosure contact the Y=0 plane. The
generator and scope visual catalog sizes are `[6,3.6,2.8]` and `[6,3.8,2.8]`
in editor world units. The `visual_chassis` pitches −10° about local X while
the root retains editor yaw about Y. For physical packages, use the canonical
footprint frame with its lead-contact-plane origin and scale `s = 0.13 / 2.54`
world units per millimeter, so a 2.54 mm pitch measures 0.13 world units.
Subtract the specified source origin before applying the measured axis mapping
and scale. For example, a measured CAD frame with +X right, +Z up and −Y front
maps a source-relative point to `s × (x, z, −y)` when targeting that physical
scale. Apply that axis conversion once. For a text-to-cad GLB already in the
approved Y-up frame and in meters, use the approved datum and `1000 × s`
world units per meter for that physical package. Verify basis landmarks,
dimensions and anchor positions after import.
For bench visuals, derive scale from the target layout and record the mapping
explicitly. (`src/data/partCatalog.ts:7–8`, `src/three/PackageModel.ts:6–17`,
`src/electrical/modulePose.ts:3–19`, `src/three/InstrumentChassis.ts:5–31`.)

Preserve stable node names and local pivots. The independent `screen` uses an
unlit material. Centered knobs rotate about local Z and button press axes are
local Z. Connector groups include `ch1_output`, `ch2_output`, `sync_counter`
for the generator and `ch1_bnc`, `ch2_bnc`, `trig_out_bnc` for the scope.
Retain the approved historical `net_circuit_*` roots for delivered bytes.
Use core PBR surfaces and owner-supported extensions. Both tracked GLBs embed
three PNGs and use `KHR_materials_unlit`. The imported CanvasTexture uses
sRGB and `flipY = false`. Map any functional anchor to an existing canonical
endpoint ID. The bench BNCs are visual only. Contracts owns catalog identifiers.
Interface graph commands own connectivity and history, and its scene projects
resolved endpoints for routing and picking. (`docs/en/architecture/namespace.md:62`,
`src/preview/instrumentPreview.ts:35–47`, `src/electrical/endpoints.ts:15–38`,
`src/three/SceneManager.ts:78–90`, `docs/en/architecture/workbench.md:5–17`.)

Record asset ID, producer revision, source and tool provenance, SHA-256 and
measured costs. The preview provider enforces the descriptor digest and a
2,000,000-byte ceiling per owned GLB. Delivered-asset tests require under
1,500,000 bytes. Procedural factory tests require fewer than 100 meshes per
instrument and fewer than 20,000 generator or 25,000 scope triangles. Actual
delivered figures are 84 meshes, 18,038 triangles and 1,460,816 bytes for the
generator, and 94 meshes, 16,926 triangles and 1,219,028 bytes for the scope.
Use an owner-approved budget for other profiles and any draw-call or memory
limits. (`src/data/instrumentAssets.ts:3–12`, `src/services/assets/gltfAssets.ts:47–104`,
`tests/function-generator.test.ts:75–96`, `tests/oscilloscope.test.ts:54–60,128–133`.)

Submit the static candidate check and measured handoff evidence to Interface.
Owner acceptance includes `GLTFLoader` round trips, preview and fallback
behavior, browser rendering and the affected workspace and graph regressions.
(`tests/gltf-assets.test.ts`, `tests/instrument-chassis.test.ts`,
`tests/function-generator.test.ts:91–136`, `tests/oscilloscope.test.ts:128–173`,
`e2e/assets.spec.ts`.)
