<!-- doc-id: CNR-CLI-SPEC -->
<!-- revision: 3 -->
<a id="cnr-acceptance"></a>

# CNR implementation acceptance

- [x] A separate binary shares the existing governance core. Maintenance command envelopes retain compatibility.
- [x] The terminal menu is usable with keyboard controls and supplies plain help outside a TTY. The CLI validates option scopes, returns clean JSON and supports completion.
- [x] Projects can initialize, select profiles and exact tool versions, lock native inputs, plan and synchronize environments, and export or import configuration.
- [x] A single provider inventory drives discovery and installation plans. Mutations require authority, stay within project ownership, support dry runs, and leave recoverable records.
- [x] Build, run, test, exec and package commands apply environment and execution policy. Verify bounded output, timeouts, cancellation, version conflicts and Windows argument safety.
- [x] Verify Rust, governance, translations and CLI build compatibility. Report platform and provider cases without fixture or native evidence.

## Terminal acceptance

- The wide terminal shows a Claude-orange `CIRNET RULES` wordmark with a common baseline, consistent letter size and a dark offset shadow. Narrow or short terminals use a compact title.
- Arrow navigation keeps the screen in place and redraws changed rows. A resize recomputes the layout. Returning from a command restores terminal input and output.
- The default view has command labels. The bottom is reserved for search, command input and optional help when active.
- `cnr`, `--no-banner`, `--lang vi`, `NO_COLOR` and the headless CLI preserve their roles. The command parser, permission checks and MCP entry points use the existing core.
