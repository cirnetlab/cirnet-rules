---
name: cirnet-engineering-workspace-ui
description: Use when designing, implementing, or reviewing an interactive editor or workbench for CIRNET circuits, CAD/3D, IDE, or Analyze.
---

# Build an engineering workspace UI

Read `en/rules/engineering-workspace-ui.md` from an approved `cirnet-rules`
checkout. Apply the owning repository's instructions, source and contracts
within the applicable `en/policies/architecture.md` constraints.

1. **Classify the task.** Identify the owner, user role, active artifact and
   primary tasks. Route ordinary login, account, LMS, scheduling and admin
   screens to the workflow appropriate to their own tasks.
2. **Map a real interaction.** Trace object → selection → mode → property or
   command → feedback → cancel, undo or save. Identify persistent document
   state, transient UI state, external operation status and failure paths.
   Define observable success for the affected task.
3. **Compose around the artifact.** Place the editor, viewport, plot or code
   surface first. Justify surrounding controls by the workflow they support.
   Reuse owner components, tokens and gestures. Keep panels compact, collapsible
   or resizable where supported and give the workspace a coherent CIRNET identity.
4. **Preserve ownership and engineering truth.** For CIRNET-Interface, inspect
   the current owner's `docs/en/architecture/workbench.md` and the affected
   components or controllers. Keep `SingleWorkspaceShell`, `CircuitGraph` and
   the Three.js projection in their established roles. Use graph semantics for
   connectivity and acknowledged owner state for operations and instruments.
   Scope any implementation to the authorized product and actions.
5. **Verify interaction.** Use the owner's functional tests and authorized
   native browser interactions when available. Exercise selection, edits,
   cancellation or undo, save, focus, resize, and unavailable or error states
   relevant to the change. Check keyboard and pointer paths, panel occlusion,
   small viewports, document/scene consistency and actual operation feedback
   as applicable.
   Report visual review, functional test results and observed browser
   interactions with their evidence. Mark unexecuted checks `NOT_RUN`.
