<!-- doc-id: CNR-TUI-READABILITY-PLAN -->
<!-- revision: 1 -->
<a id="implementation"></a>

# CNR check and editor implementation plan

1. CP1 records the checkout and reproduces the narrow terminal wrap and keyboard gaps.
2. CP2 renders typed findings in an issue-first list and an adaptive detail view. Test status fidelity, filters, search, copy redaction and resize.
3. CP3 shares a grapheme-aware editor across the command, search and setup fields. Test selection, undo, redo and paste limits.
4. CP4 maps input and output selections to terminal cells, adds the context menu and integrates the Windows clipboard. Test fallback and job cancellation.
5. CP5 verifies Windows Terminal behavior, builds the pinned binaries, runs the full suite and synchronizes operator documentation.

[Scope and layout decision](../../../specs/cirnet-rules/cnr-tui-readability/spec.md)
