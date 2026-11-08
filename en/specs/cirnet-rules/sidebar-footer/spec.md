<!-- doc-id: CGACP-SIDEBAR-FOOTER-SPEC -->
<!-- revision: 1 -->
<a id="acceptance"></a>

# Dashboard sidebar verification

- [x] Remove the standalone conversation paging button and verify that scrolling or the context menu still loads older conversations.
- [x] Verify that the slim Local and Settings footer stays usable with the sidebar expanded or collapsed.
- [x] Verify the Windows Qt build and synchronized operator documentation.

The paging fixture loaded 30, 30 and 5 rows in three requests. It used End for the second page and the list context menu for the third. An additional End on the short final page issued no request. The Qt 6.11.2 Windows build, native checks at 125%, CTest and governance validation passed.
