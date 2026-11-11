<!-- doc-id: CNR-CLI-PLAN -->
<!-- revision: 3 -->
<a id="cnr-plan"></a>

# CNR implementation plan

[Acceptance](../../../specs/cirnet-rules/cnr-cli/spec.md)

1. Add fixture tests for headless entry, option policy, project initialization, and execution.
2. Move build and packaging helpers, preserve maintenance behavior in a shared core, and implement the CLI and terminal presentation.
3. Integrate project resolution, provider planning and owned operations with explicit version and grant checks.
4. Connect execution to the bounded process harness. Exercise compatibility, failure recovery and discovery using isolated fixtures.
5. Verify the affected CI commands, synchronize bilingual documentation, and report limits against acceptance criteria.

## Terminal refresh and source layout

1. Reproduce the full-screen erase during keyboard navigation in a Windows pseudoconsole.
2. Move the executable and tests under the Rust source tree and update Cargo, build identity and the Rust source policy together.
3. Separate menu state, layout and terminal output. Compose the wordmark and its offset shadow from half-block cells. Draw only changed rows. Show search, command input and optional help at the bottom when active.
4. Verify wide and compact screens, Vietnamese labels, command dispatch, resizing, `NO_COLOR`, unchanged rows and the headless CLI.
