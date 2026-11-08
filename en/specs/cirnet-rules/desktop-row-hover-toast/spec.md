<!-- doc-id: CGACP-DESKTOP-ROW-HOVER-TOAST-SPEC -->
<!-- revision: 1 -->
<a id="acceptance"></a>

# Desktop row hover and Conversation toasts

The desktop tables in scope are Repositories and Automation Run history.
Verify them with disposable data.

- [x] Hover paints the full populated row in both tables, including space after the last column where present. Alternating backgrounds cover the same width, and the empty area below rows stays plain.
- [x] Table selection, keyboard interaction, saved repository widths and run cancellation retain their existing behavior.
- [x] Toasts move to the upper right when entering Conversation and return to the lower right on other routes. The same card remains visible across transitions, subject to its normal expiry and available space.

Qt tests sample the row backgrounds in both themes, verify cancellation of
the selected run in disposable SQLite data, and track the same toast as the
operator switches pages and clicks the composer.
