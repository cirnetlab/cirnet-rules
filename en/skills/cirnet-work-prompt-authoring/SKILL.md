---
name: cirnet-work-prompt-authoring
description: Use when asked to write a Work Prompt or implementation handoff for Codex, Claude Code, Antigravity, or another agent to execute. Direct implementation requests use the applicable execution skill.
---

# Author a Work Prompt

In the Chat role, do authorized read-only discovery and author the handoff.
Use the applicable implementation, debugging or verification skill when
asked to do the work directly. The Work Agent rechecks local checkout, branch,
permissions and evidence, then plans, edits, tests and reports within scope.

1. Read the request and identify the intended outcome, allowed repository
   paths, read-only dependencies and authorized operations.
2. Open `en/policies/architecture.md` and `en/references/architecture-routing.md`
   from the approved toolkit. Identify the owner and select the applicable
   English Ecosystem, Lab-First or FPGA sections and relevant Rules and Skills.
   Carry forward the applicable technology and `LAB_FIRST_PRIMARY` constraints.
   Independent Ecosystem work can proceed within its boundaries. Any switch to
   `PARALLEL_ALLOWED` needs the policy's evidence.
3. Inspect the owner's source, contracts, tests and manifests when accessible.
   Record observed facts with their scope and mark unverified claims. If Chat
   lacks source access but the owner and safe scope are clear, use
   `NOT_INSPECTED` and make confirmation part of Work preflight. Treat earlier
   test results as evidence for their original inputs.
4. Read `en/templates/chat-to-work-prompt.md` from an approved `cirnet-rules`
   checkout and fill it with task-specific instructions. The installed bundle
   supplies this Skill and requires access to that checkout for the template.
   If the checkout or template is inaccessible, report `PROMPT_BLOCKED` and
   the access needed.

Before returning a prompt, check the owner and architecture, the boundary with
other repositories, the evidence labels and observable acceptance criteria.
Preserve contract compatibility, exact pins and frozen artifacts. At physical
Lab boundaries, keep Safety separate from Correctness, bind to observed topology
and reject automatic repair of student wiring. Use evidenced capabilities and
results. Tie real E2E claims to real boundaries. Carry only granted permissions,
especially for cross-repository edits, publication, deployment and physical
operation.

Return `PROMPT_READY` and one copyable Work Prompt in chat when the scoped
handoff is ready. Work must still provide evidence for
`IMPLEMENTATION_VERIFIED`. Return `PROMPT_BLOCKED` with the missing input,
permission or conflicting decision when it prevents a safe handoff. Include a
Work Prompt for a separable task only if that task is independently authorized.
