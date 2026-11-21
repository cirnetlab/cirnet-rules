# Engineering workspace UI principles

Apply this rule to interactive engineering editors and workbenches. Choose
forms, tables or dashboards where they suit transactional tasks such as
login, accounts, LMS, scheduling and administration.

## Design invariants

1. **Primary work surface.** Give the active circuit, schematic, model, code,
   plot or experimental data the main work area. In the CIRNET-Interface circuit
   editor, keep the interactive 3D viewport as the main editing surface.
2. **Domain ownership.** Follow the product owner's instructions, source and
   contracts within the applicable [CIRNET architecture](../policies/architecture.md).
   In CIRNET-Interface, `SingleWorkspaceShell` hosts the workspace, `CircuitGraph`
   commands own electrical connectivity, document edits and history, and the
   Three.js scene projects the graph. Derive electrical connections only from
   graph semantics. Camera motion, visual crossings and panel state are
   presentation details.
3. **Contextual controls.** Organize properties and commands around the object,
   selection, mode and active task. Justify each major region by the work it
   supports. Add trees, inspectors, toolbars, task panels, history and gizmos
   where they help. Prefer compact or adaptive panels that keep the primary
   surface and critical feedback accessible.
4. **Reversible edits.** Preserve supported selection, multi-selection,
   cancellation, confirmation, undo/redo and unsaved-change handling. Show
   active modes and previews separately from committed document state.
5. **Engineering feedback.** Distinguish requested, pending, accepted, applied,
   failed, unavailable and measured states as applicable. Use the owner's
   contracts and operation evidence for capabilities, readiness, results and
   Run/Stop acknowledgements. Keep hardware authorization with Remote and
   Hardware for Lab operations. Show measurements with their units, ranges,
   provenance and uncertainty where relevant.
6. **Visual identity.** Use density, typography, hierarchy, contrast and
   iconography to make technical information legible and interactions precise.
   Choose visual elements and themes by task benefit. Preserve CIRNET's own
   visual identity. Draw on general interaction patterns from other tools.
7. **Usability and performance.** Preserve keyboard and pointer access,
   visible focus, discoverable commands, meaningful labels, resize behavior,
   graphics lifecycle and loading, error and unsupported states.

## Acceptance boundary

Exercise at least one representative task on the identified primary surface.
Select an object, enter the relevant mode, and use a contextual command or
property control. Verify that previews reflect the pending edit, cancellation
keeps the prior document, and commit and undo/redo keep the document and
scene consistent, where supported. For a save, verify the acknowledged
revision and resulting persisted state through the owner's supported checks.

Exercise relevant keyboard and pointer paths and verify focus can return to
the work surface. With panels open, verify the surface stays usable or show
how the user regains access. Exercise relevant failure and unavailable states
and verify that the feedback matches the actual outcome. Use observed
interactions to establish task success and screenshots to review appearance.
Use authorized native browser evidence when available. Preserve the owner's
functional checks and safety boundaries.

Use the [engineering workspace UI Skill](../skills/cirnet-engineering-workspace-ui/SKILL.md)
for the implementation and review workflow.
