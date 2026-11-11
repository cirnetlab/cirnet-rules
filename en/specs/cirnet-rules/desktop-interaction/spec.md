<!-- doc-id: CGACP-DESKTOP-INTERACTION-SPEC -->
<!-- revision: 2 -->
<a id="acceptance"></a>

# Desktop interaction and Settings verification

- [x] Reproduce the reported symptoms with an isolated Windows Dashboard.
- [x] Verify the route, Sidebar, control, Settings and notification changes.
- [x] Record evidence and unverified conditions for UX-01 through UX-40.

## Baseline and reproduction

The Windows build at Git revision 37808be opened CGACP v1 and initialized a private SQLite fixture. Qt 6.11.2 used the Windows platform plugin. The baseline native capture measured a device pixel ratio of 1.25. The supplied recording decoded at 1918×1018 and 30 frames per second. The observations below refer to inspected frames, with temporary native captures as the independent comparison.

| Scope | Observed and reproduced | Implementation and verification |
| --- | --- | --- |
| Route and button states | At 00:05 both Automations and a conversation have highlighted surfaces. Native regressions reproduced retained mouse outline, active row after switching, and late refresh. | Active route now controls visual selection. Keyboard focus and pressed appearance use separate styling. |
| Sidebar | The supplied compact screenshot shows scattered controls. On the native baseline at the tall viewport, the group occupied logical y 160–741. | An explicit middle stretch keeps controls at the top and the footer at the bottom. |
| Settings and arrows | At 00:20 Settings uses a section dropdown. Native tests reproduced the missing category list and undersized arrow hit areas. | Settings uses a left category list and appearance previews. The shared Qt style draws and tests compound glyphs. |
| Repository and automation | Frames at 00:42, 01:04 and 02:03 show repository controls and an automation form. | The real fixture covers repository actions, review and one-shot history. The form and stepper were captured separately. |
| Toast | The recording at 01:22 and 02:28 shows a top-right toast. The native baseline newest card was 206 logical px above the client bottom. | Newest appears at the lower right with an 18 logical px margin unless an entry control requires more clearance. |

## UX-01 through UX-40

PASS records evidence for the stated check. NOT_RUN records an unexercised part without counting it as a pass. The screenshot evidence uses disposable data. See the [state guide](../../../references/desktop-interaction.md) and [Qt tests](https://github.com/cirnetlab/cirnet-rules/blob/52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c/desktop/tests/desktop_test.cpp).

| ID | Result | Evidence |
| --- | --- | --- |
| UX-01 | PASS | The dialog was cancelled by Escape. Automations stayed active and the fixture logged no conversation creation. |
| UX-02 | PASS | Mouse press/release/exit on the shared action role produced one click and no retained accent. Repository Refresh, Copy and Reset were exercised. |
| UX-03 | PASS | The primary fill passed a pixel check. Add Automation, Save and the confirmed one-shot were exercised with fixtures. |
| UX-04 | PASS | Tab and Shift+Tab moved focus in both directions and restored the accent focus indicator. |
| UX-05 | PASS | Chat and automation captures show separate entry surfaces. The draft and daily schedule fields remained interactive. |
| UX-06 | PASS | Opening a saved chat and switching to Automations cleared chat selection while keeping its draft. |
| UX-07 | PASS | Selecting a different conversation activated only its row. Automations became unchecked. |
| UX-08 | PASS | A deliberately delayed conversation list reply left Automations active and the history unselected. |
| UX-09 | PASS | The fake runtime delivered chat detail replies out of order. The latest chat and user route won. |
| UX-10 | PASS | Back restored the active workflow, draft and repository table state. Escape closed an open combo popup before leaving Settings. |
| UX-11 | PASS | The compact group kept its position at short and tall viewport sizes. Both footer controls remained inside their parent. |
| UX-12 | PASS | The panel glyph and borderless button were reviewed in native captures. Qt checked its hit size and collapse action. |
| UX-13 | PASS | Dragging and repeated toggles preserved expanded width. The real runtime restored sidebar preferences after reopening. |
| UX-14 | PASS | The compact rail test activated New Chat, Automations and Settings by keyboard. |
| UX-15 | PASS | The arrow raster and hit region passed in Dark and Light. Native Settings and repository captures show the actual drop-downs. |
| UX-16 | PASS | Mouse and keyboard changed the spinbox by its step. Bounds, suffix and unfocused wheel behavior were checked. |
| UX-17 | PASS | The control fixture exercised both arrows and disabled state. The automation dialog advanced its selected minute by keyboard. |
| UX-18 | PASS | The native Relink submenu opened by keyboard, aligned to its parent and returned to the parent menu on Escape. The selected binding stayed unchanged. |
| UX-19 | PASS | Dark and Light checkbox pixels retain a tick. Appearance tiles expose radio semantics and a persistent selected state. |
| UX-20 | PASS | Native glyph crops and geometry checks cover the reported effective display factors. The measured DPR is in each log. |
| UX-21 | PASS | The native Settings test verified a category list to the left of content and no section combo box. |
| UX-22 | PASS | Local search focused Reduced motion. The Chat and Automation repository links returned to their open forms. |
| UX-23 | PASS | Dark and Light tiles apply immediately and System survives reopening. A process-local Qt scheme change updated the shell and the open automation dialog while retaining its input. |
| UX-24 | PASS | A rejected fixture save showed unsaved state and retry. A real database reopened with the saved mode and reduced motion preference. |
| UX-25 | PASS | A native toast and Qt geometry assertions place the newest card at the client bottom right. |
| UX-26 | PASS | Qt checked upward ordering, dismissal, queue limits, deduplication and containment after resize. |
| UX-27 | PASS | Qt verified focus preservation, composer clearance and modal confirmation. A timed Windows fixture verified that expiry pauses for Qt hover state and keyboard focus, then resumes when both clear. |
| UX-28 | PASS | Quiet mode and duplicate events were exercised. The successful fixture startup issued data requests without a routine success toast. |
| UX-29 | PASS | An error remains in bounded history with its full text. The real one-shot still used an explicit confirmation and approval workflow. |
| UX-30 | PASS | Qt removed an active fade and created the next toast without animation when Reduced motion was selected. Persistence passed on reopen. |
| UX-31 | PASS | The native repository table retained interactive resizing, sort, selection and restored header settings. The bright table outline is absent in the capture. |
| UX-32 | PASS | The real temporary Git repository passed preview, binding, reopen and relink with a Unicode path. Qt's file URL handoff used the verified folder. |
| UX-33 | PASS | The real runtime created a fixture definition. Qt edited it, confirmed a run and toggled pause while keeping the same run and provenance. |
| UX-34 | PASS | The UI capture and fixture request list contain no workflow gallery. |
| UX-35 | PASS | About and runtime views display CGACP v1 and the runtime handoff and blocked capability values. |
| UX-36 | PASS | Vietnamese Qt text, the category list and Appearance at larger app text scale were checked in native runs. |
| UX-37 | PASS | Fixture request counts bounded startup and pure resize. Theme, collapse and navigation do not request a release or repository scan. |
| UX-38 | PASS | The diff covers the Desktop, presentation preferences, direct tests and bilingual documentation in the authorized checkout. |
| UX-39 | PASS | The executable smoke, native harness and Rust tests used private fixture databases and temporary repositories. |
| UX-40 | PASS | The evidence folder contains before and after native captures, regression results and environment limitations. |

## SOURCE_CHANGES

Qt changes cover the shell, conversation navigation, shared style, Settings and NotificationCenter. Rust and the IPC schema add the optional reduced motion presentation field to the existing record. Old rows load the default without a database migration.

## GUI_VERIFICATION

The native Windows harness exercises the Dashboard widgets using either deterministic IPC fixtures or the real Rust runtime with a temporary Git workspace. Effective DPI samples of 100%, 125%, 150% and 200% and logical window geometry are recorded in the logs. A separate test moves the window between the native 125% and 100% screens. CTest offscreen also exercises the runtime and the clipboard comparison. The standalone development executable opened with its private SQLite data in 1,454 ms on this host.

## RUNTIME_REGRESSION

The pinned Rust/Cargo 1.99.1 suite passed formatting, strict Clippy, locked build and 69 tests. Governance validation checked 72 EN/VI pairs. The preference test loads an old row, rejects a non-boolean and reopens the saved value.

## UNVERIFIED_ENVIRONMENTS

Linux GUI, a theme change through Windows Settings and external screen reader use were not exercised. The expiry test supplies Qt hover state directly because the sandbox could not deliver pointer movement consistently. The native clipboard comparison is excluded because the sandbox denied COM. The offscreen clipboard and intercepted folder URL checks are separate from Explorer and native clipboard operation.

## RELEASE_ACTIONS

CGACP v1 remains application 0.1.0. The change is in the local working tree. No release action was taken.
