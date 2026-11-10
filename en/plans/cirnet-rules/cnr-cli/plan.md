<!-- doc-id: CNR-CLI-PLAN -->
<!-- revision: 1 -->
<a id="cnr-plan"></a>

# CNR implementation plan

[Acceptance](../../../specs/cirnet-rules/cnr-cli/spec.md)

1. Add fixture tests for headless entry, option policy, project initialization, and execution.
2. Move build and packaging helpers, preserve maintenance behavior in a shared core, and implement the CLI and terminal presentation.
3. Integrate project resolution, provider planning and owned operations with explicit version and grant checks.
4. Connect execution to the bounded process harness. Exercise compatibility, failure recovery and discovery using isolated fixtures.
5. Verify the affected CI commands and Desktop, synchronize bilingual documentation, and report limits against acceptance criteria.
