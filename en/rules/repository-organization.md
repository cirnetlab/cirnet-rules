# CIRNETLAB Repository and Module Organization Standard

- **Status** — PROPOSED
- **Scope** — All CIRNETLAB repositories

Use this proposal to review ownership, feature cohesion and module
responsibility. Apply the current [architecture policy](../policies/architecture.md)
and the owning repository's instructions, contracts and conventions first.
Adopt structural changes through the owner's review and authorization.

## Ownership and cohesion

Use **Repository Owner → Domain → Feature → Module → File** as logical levels.
Choose physical directories where they make ownership, dependencies or
verification clearer. Give each relevant directory a discoverable
responsibility and an owning domain or module. Identify the owner and purpose
of modules, files, generated artifacts and evidence. Use existing source
structure, module boundaries, documentation or the owner's repository map
as evidence. Add a local README or ownership manifest only when it serves a
specific repository need. Justify new structure with an actual capability,
ownership boundary, test need or maintainability problem.

- A **domain** owns behavior, data and invariants within a bounded context.
  Use the [ownership routing table](../references/architecture-routing.md) for
  target ownership. Verify current authority from the actual writer, published
  contract or selected artifact and its consumers.
- A **feature** delivers a coherent capability or workflow. Use consistent
  feature names or ownership maps to help people find its source, tests and docs.
- A **module** has a cohesive responsibility and a testable boundary. Split
  it where responsibilities, dependencies, lifecycles or safety boundaries
  warrant separation. Keep small implementations cohesive.

## Placement and provenance

| Category | Use the owning repository's established location for |
| --- | --- |
| Source | Language-native production roots, grouped by domain or feature |
| Tests | Co-located checks or test roots such as `tests/`, `e2e/` or `native/tests/` |
| Documentation | Explanations, plans, specifications and evidence with applicable EN/VI correspondence |
| Scripts and configuration | Named build, test, migration or operational tools and their tool-native manifests or scoped configuration |
| Contracts | Versioned interfaces and compatibility fixtures published by the designated owner |
| Generated files | Declared output traceable to its generator, source inputs and revision |
| Assets and fixtures | Product or feature assets with traceable origin, license and expected use |
| Reports and evidence | Approved history for immutable evidence and allocated output for disposable logs |
| Migrations | Versioned changes and historical references that support controlled migration and rollback |

Create categories when work requires them. Use names that describe behavior or
responsibility and follow local language and framework conventions. Keep one
canonical document per purpose and preserve repository-specific task records,
including existing `.superpowers/plans/` and `.superpowers/specs/` conventions.
Follow the [documentation lifecycle](../policies/documentation.md#documentation-lifecycle)
and regenerate derived outputs through their authoritative producer.

## Search before creating

1. Establish the capability, owner and authorized scope.
2. Search paths, filenames, exported symbols, responsibilities, docs and
   tests within that scope. Inspect the current implementation and direct
   callers before choosing a placement.
3. Reuse or extend the authoritative implementation where it fits. Justify a
   separate module with a distinct responsibility and explain any overlap.
4. Verify a single authoritative owner for the capability. If ownership is
   unverified or contested, report the evidence gap and resolve it with the
   owner before changing or creating paths that depend on it.

Use global `utils/`, `common/`, `shared/` or `helpers/` only for a stable
shared responsibility with identifiable consumers and a testable boundary.
Keep feature-specific helpers with their domain even when several callers
use them. Prefer names such as `capture/`,
`validation/` or `routing/` for domain responsibilities. Preserve existing
shared directories with clear ownership and keep unrelated concerns in their
own modules.

## Language and repository conventions

- **TypeScript and Vue.** Preserve domain logic, application coordination,
  presentation, state, graphics and browser test responsibilities where they
  exist. For CIRNET-Interface, keep CircuitGraph document semantics,
  application operations, UI state and Three.js projection in their
  [established roles](engineering-workspace-ui.md#design-invariants).
- **Rust.** Follow existing Cargo workspaces and crates. Use domain modules
  where they clarify dependencies, keep private implementation details
  private, and justify workspace splits or additional processes with evidence.
  Maintain acyclic module dependencies.
- **C++.** Preserve CMake targets, worker entrypoints and ABI or protocol
  boundaries. Keep solver logic, worker transport, resource limits and
  integration adapters distinguishable as their responsibilities grow.
- **FPGA and RTL.** Organize by hardware block, interface and verification
  responsibility. Keep RTL, constraints, testbenches, synthesis configuration
  and generated programming output identifiable. Preserve module and top-level
  names, clock/reset interfaces, constraints and integration manifests unless
  an approved compatibility migration covers the change. Use simulation for
  RTL verification and owner-approved board evidence for physical acceptance.
- **Placeholder repositories.** Keep the applicable README, license and
  governance files. Add implementation, architecture folders and tests when
  an approved feature needs them.

## Cross-repository boundaries

Use documented, versioned interfaces or approved artifacts for cross-repository
reuse. Resolve the current producer, selected revision and direct consumers
through [ownership routing](../references/architecture-routing.md) and the
[cross-repository rule](cross-repo.md). Review these separations when relevant.

| Boundary | Responsibilities |
| --- | --- |
| Studio / Interface | Workspace composition and circuit editing |
| Learn / LMS | Learning presentation and academic behavior |
| Project / Format-Project / Contracts | Project lifecycle, portable serialization and canonical shared interfaces |
| Interface / Remote / Hardware | Lab presentation, operation authorization and station enforcement |
| Analyze / Numerics / Simulator | Analysis experience, reusable kernels and electronic simulation |
| IDE / Toolchains / Devices | Editing, build environments and device profiles and adapters |
| Rules / Product repositories | Shared agent guidance and owner-specific implementation and constraints |

Keep copied schemas, generated SDKs and consumer adapters traceable to their
authoritative owners.

## Safe restructuring and acceptance

Before a move, inventory source and destination owners, imports and exports,
public symbols and APIs, direct callers, package and crate boundaries, build
and code generation inputs, scripts, tests, fixtures, snapshots, coverage,
CI, packaging, deployment, releases, document links and EN/VI counterparts.
Check case, encoding and platform behavior. Identify generated assets,
historical evidence and rollback needs. Apply the smallest authorized change
and update references together. Preserve user edits, ignored and untracked
files, frozen releases, evidence and provenance. Obtain explicit authorization
for cross-repository moves, bulk cleanup or destructive history changes.

Verify each changed path's owner and purpose, one authoritative implementation,
imports, dependencies, build entrypoints, affected tests, fixtures, packaging
and CI references. Regenerate or validate affected outputs through their
producer. Check documentation and history links, public contracts and pinned
release compatibility or the approved migration, then review the final diff.
Report observed `PASS`, `FAIL`, `BLOCKED` and `NOT_RUN` results. Show the
ownership, duplication, dependency or workflow improvement with evidence.

Use the [repository structure Skill](../skills/cirnet-repository-structure/SKILL.md)
for the review and verification workflow.
