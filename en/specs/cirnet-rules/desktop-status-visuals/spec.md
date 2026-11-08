<!-- doc-id: CGACP-DESKTOP-STATUS-VISUALS-SPEC -->
<!-- revision: 1 -->
<a id="acceptance"></a>

# Desktop status and visual polish

This task covers the Qt presentation layer. Use a private runtime and Git fixture for verification.

- [x] Local reports the connection state on click or keyboard activation and does not open a navigation menu.
- [x] Selecting a repository highlights the full row, including spare horizontal viewport space, while keeping its binding identity and table controls.
- [x] Refresh and related desktop icons use a consistent, legible vector style in both themes.
- [x] Toast glyphs and dismissal align with the message. Severity remains readable and wrapping, history and stack behavior are preserved.

The four regressions failed before the change and passed after it on Qt
6.11.2 for Windows. The repository fixture checks selection color and binding
identity after clicks in cells and trailing space. The icon tests inspect
both palettes and physical pixel sizes. The toast checks cover all four
severities and verify that a queued automation run is informational.
