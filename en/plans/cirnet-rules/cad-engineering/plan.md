# CIRNET CAD engineering implementation plan

Implement the [specification](../../../specs/cirnet-rules/cad-engineering/spec.md)
in `cirnet-rules`. Keep the referenced tool and product checkouts read-only.
Allocate temporary output in the ignored owner workspace.

## Steps and checks

1. Pin the owner and reference repositories. Read the tool commands and the
   Interface assets, selected dependency and loading contracts.
2. Add the bilingual CAD rule, links and optional Skill. Verify that the new
   Skill is indexed, packaged, installable on selection and outside defaults.
3. Write synthetic GLB fixtures, then implement the static intake. Check valid
   structure, active-scene component names, mesh reuse, pins, stale or unsafe
   paths, truncated output, unsupported features and exceeded limits.
4. Verify the same source on Windows and Ubuntu where available. Run the
   offline Rust build, documentation checker, affected verification and
   continuity fixtures, and Windows bundle and installer coverage.
5. Determine native CAD prerequisites locally. Exercise the approved pilot
   only if an appropriate preinstalled runtime is available.

## Intake evidence

| Source | Inspected revision |
| --- | --- |
| Rules master | `b10e75a378cca7196c3b27e346d283ddd6c6f42d` |
| text-to-cad main | `7687743d80305ba112666aee0a5e8c12af138458`, source version 0.11.15 |
| Interface master | `82dd99a62c899aa413d03856c41e252bf289ba22`, package 0.2.4, Three.js lock 0.171.0 |

The [operating rule](../../../rules/cad-engineering.md) records the observed
contract and what a native pilot must measure. Native CAD execution and
Interface adoption are separate acceptance steps.

## Verification record

| Check | Observation |
| --- | --- |
| Static gate fixture | `node --test tests/verification/cad-asset.test.mjs` exited 0. Covered embedded GLB, active-scene names, repeated mesh use, pins, path/file bounds, malformed fields, unsupported primitives and budgets. Explicit null fields that previously produced PASS now fail. |
| Interface reference assets | Read-only copies of the existing generator and scope GLBs passed the gate with their pinned owner digests and selected component names. Measured 18,038/16,926 triangles and 84/94 primitive instances. |
| Toolkit build | Offline Cargo build, tests, formatting and Clippy passed. The Windows link fixture was skipped. The documentation checker inspected all eight new Markdown files and their source hashes. |
| Skill distribution | Fallback and trigger metadata tests passed. The Windows bundle and install tests exercised default and optional CAD selection in fake homes. Reparse and junction fixtures were skipped by host permissions. |
| Native CAD pilot | `BLOCKED`. The checked machine has Python 3.14.4 and uv 0.11.9. The local source checkout provides neither an installed CAD executable nor the required build123d, OpenCASCADE or Blender runtime. No install or plugin activation ran. |
| Native visual and product acceptance | `NOT_RUN`. The pilot defines native geometry, snapshot, export and review work. Actual Interface loader, browser and interaction checks require separately authorized adoption. |

## Static intake hardening

The targeted review started at Rules `1d22a337fdf53ab30c6b15814f116569e43a8798`.
Use the existing GLB parser, artifact pins and selected-scene gate. Keep
text-to-cad and Interface as read-only reference checkouts.

- The regression suite reproduced false passes for out-of-range and reserved
  index values, non-finite POSITION and NORMAL components, invalid vertex
  declarations and inconsistent buffer layouts. The prior checker read
  declaration sizes and counts without decoding the values. Some simple
  invalid offsets were already rejected. The updated checker reads bounded
  data from the existing BIN chunk and validates each primitive's index
  range. Scans of a shared accessor are reused.
- The previous extension guard rejected any required extension. Fixtures
  confirm required and optional `KHR_materials_unlit` support and separate
  malformed declarations from unsupported extensions.
- Both owner-pinned Interface GLBs passed static intake. The generator
  exporter rewrites V coordinates after export and its delivered GLB retains
  two stale optional UV extrema. The static bound comparison covers POSITION
  and declared index extrema. UV layout and float values are checked.
  Preserve the delivered product bytes and leave texture correctness to
  the owner tests.
- The enclosure pilot remains `BLOCKED`. The current local installation
  check found no `cad` command or matching build123d/OpenCASCADE environment.
  Geometry inspection, snapshots, STEP/GLB export, visual review and new
  Interface runtime adoption remain `NOT_RUN`.

## Static material and texture checks

At `bbe06d4adb805d61cfe73a47d7ffbb70dca2a1e2`, the static gate accepted
broken texture, image, sampler and UV references. The focused regression
exited 1 before the fix. After the fix, the CAD suite exited 0. It also
checks image signatures against the declared PNG/JPEG type and keeps image
buffer views separate from geometry. It reports unsupported texture
extensions as `BLOCKED`. Both existing Interface GLBs passed with the
owner-pinned hashes. The signature check confirms the declared format at
the start of the embedded data. Image decoding remains an owner check.
