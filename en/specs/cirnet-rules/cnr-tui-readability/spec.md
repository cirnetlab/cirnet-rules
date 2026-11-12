<!-- doc-id: CNR-TUI-READABILITY-SPEC -->
<!-- revision: 1 -->
<a id="scope"></a>

# CNR check presentation and terminal input

CNR uses typed findings for navigation and keeps the existing check report and exit contracts. The operator sees severity, reasons and next actions at narrow widths. Evidence and related actions are available from the selected finding.

## Layout decision

A fixed table serves short values at a stable width. Long reasons and paths make the table difficult to scan on a narrow terminal. A grouped list keeps each reason with its finding and makes counts visible before expansion. A master-detail layout keeps the selected finding visible while displaying evidence where space permits. CNR combines the grouped list and the master-detail layout. Narrow terminals switch between the list and one finding; wide terminals can display both.

The shared editor supports grapheme selection, replacement and bounded undo. Clipboard access follows an explicit operator action. Output selection is read-only, and the application excludes sensitive or raw report data from clipboard actions. Paste changes the draft and requires a separate Enter event to submit it.

The work uses the current CNR TUI and its terminal lifecycle. Host-managed key and mouse events remain subject to the terminal settings. This scope covers presentation and input. Governance, MCP, permissions, verification and audit retain their existing contracts.
