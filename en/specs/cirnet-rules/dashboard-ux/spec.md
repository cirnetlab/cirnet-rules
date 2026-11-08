<!-- doc-id: CGACP-DASHBOARD-UX-SPEC -->
<!-- revision: 1 -->
<a id="acceptance"></a>

# Dashboard UX redesign acceptance

CGACP v1 retains its Qt desktop, Rust runtime and SQLite storage. The shell, settings and notification history are local operator surfaces. Repository binding authority, automation provenance and independent approval remain governed by the runtime.

- [x] Record baseline and compare the implementation with the supplied symptoms.
- [x] Verify the implemented shell, repository, conversation, automation and preference paths with the fixtures below.
- [x] Record evidence and limitations for UX-01 through UX-34 in both languages.

## Baseline and design decisions

The baseline was Git revision 63f4b109 on Windows 11 build 26300 with Qt 6.11.2 and effective scale 125%. Native fixture captures reproduced the stacked brand, fixed rail, bottom status strip, gallery, missing checkbox tick and dark rectangles in About. Fixed table resize modes and unformatted extended paths were confirmed from source. A valid extended path does not indicate a registration failure.

Use a native splitter and a single Settings surface. Palette tokens and a Qt checkbox primitive keep widget states consistent. A revisioned SQLite preference record stores only presentation, while the in-memory toast history stays within the session. The repository catalog and operational path remain under runtime authority.

## Acceptance evidence

PASS covers the checks listed here. PARTIAL identifies a scenario with some coverage and an explicit remaining test. Data in captures is disposable. Native capture uses the Windows Qt platform and the actual Dashboard widgets. Fixture data and real Rust runtime integration are indicated in the artifact names and logs.

| ID | Result | Evidence |
| --- | --- | --- |
| UX-01 | PASS | Native geometry and captures place owl, CGACP and CIRNETLAB on one row. |
| UX-02 | PASS | Mouse drag changes splitter width and reflows the active card grid. |
| UX-03 | PASS | Collapse keeps icon actions available. SQLite restores the collapsed state and expanded width. |
| UX-04 | PASS | The footer exposes Local and Settings with real handshake state. |
| UX-05 | PASS | Repositories and About are Settings sections. Back returns to the active conversation or form. |
| UX-06 | PARTIAL | Dark and Light repaint, System persists on reopen, and a rejected save stays visible. A live OS theme change was not exercised. |
| UX-07 | PASS | Native before and after captures show continuous backgrounds in About, cards, the repository table and confirmation dialog. |
| UX-08 | PASS | Rendered pixels show a tick in both themes, including focus and disabled. Label click and Space toggle the control. |
| UX-09 | PASS | Qt mouse drag and double-click fit resize the path column. Refresh preserves width. |
| UX-10 | PASS | A reopened Qt client restores header width, order, sort, horizontal scroll and binding selection. Reset keeps the archived binding. |
| UX-11 | PASS | The tooltip and offscreen clipboard keep the complete canonical path. A Qt file URL handler records the health-checked folder target. |
| UX-12 | PASS | Windows displays ordinary drive paths separately. Real SQLite/IPC retains extended canonical roots and binding identity. |
| UX-13 | PARTIAL | Qt file registration and Rust fixtures cover Unicode, spaces, linked worktrees, detached HEAD, reopen and relink. The Windows shell picker was not automated. |
| UX-14 | PARTIAL | Rust fixture distinguishes missing paths, non-Git folders and blocked UNC/device paths. Inaccessible Git metadata was not reproduced. |
| UX-15 | PASS | Rename preserves IDs. Relink preserves the binding and increments revision. Runtime tests reject stale automation bindings. |
| UX-16 | PASS | A 41-row fixture checks selection by ID after paging, sort, filter and refresh. Real Qt actions verify rename and archive against the saved binding. |
| UX-17 | PASS | Archiving preserves files and exposes the archived row through Show archived. |
| UX-18 | PASS | Native capture and geometry checks confirm that no feedback strip occupies content space. |
| UX-19 | PARTIAL | Queue, dismissal, positioning, resizing and focus preservation are covered by Qt checks. Hover pause was not timed independently. |
| UX-20 | PASS | Fixture checks bounded history, deduplication, quiet mode and error visibility. Run confirmation remains a dialog. |
| UX-21 | PASS | Captures contain no gallery and the fixture records no gallery-only runtime request. |
| UX-22 | PASS | Real runtime creates a template-derived record, opens it for edit and verifies the same provenance and run history. |
| UX-23 | PASS | The empty state and Add Automation are visible with an empty fixture. |
| UX-24 | PASS | Qt checks two columns at wide viewport and one at narrow viewport while retaining card identity. Native DPI captures show visible actions. |
| UX-25 | PARTIAL | Rust verifies run idempotency and stale permission gates. Real Qt confirms a one-shot and sees its run ID. A native Run now error/retry was not injected. |
| UX-26 | PARTIAL | One Sidebar history and shared New Chat are exercised. Runtime tests cover persistence, but every rename/archive/paging permutation was not driven in Qt. |
| UX-27 | PASS | The Qt fixture keeps drafts across conversation switches, Settings, theme changes and reconnect. |
| UX-28 | PASS | Chat and Automation preserve filled forms and selected bindings through Manage repositories. A paged fixture includes the last-page binding. |
| UX-29 | PARTIAL | Space, Escape, clickable controls, focus and accessible names have automated checks. An external screen reader was not tested. |
| UX-30 | PARTIAL | Resize keeps stable cards and issues no catalog, source or release requests. Startup was measured with a fixture. No production-scale benchmark was run. |
| UX-31 | PASS | A v3 database upgrades to v4 through a verified backup. Windows and Linux migration fixtures keep binding, conversation, task, run and provenance IDs. |
| UX-32 | PASS | Settings shows handoff, isolated execution blocked and offline states. A real one-shot still follows the approval path. |
| UX-33 | PASS | Qt translation loads. Native captures cover Vietnamese and target window sizes at four effective display scales. |
| UX-34 | PASS | The handover separates native captures, Qt fixtures, Rust integration and unavailable checks. Release remains unpublished. |

## Verification environment

Windows Qt/MSVC build, CTest offscreen and native Qt suites use temporary data. The physical targets are 1366×768 and 1920×1080, allowing rounding by Qt. Effective 100%, 150% and 200% use per-process scaling. The host is at 125%. A physical mixed-DPI monitor move, system screen reader and native Linux UI were not run. The Windows clipboard comparison is offscreen because the sandbox can deny COM. The folder-open test intercepts the Qt URL instead of starting Explorer.

The Rust suite uses the pinned Rust/Cargo 1.99.1 and locked dependencies. The Linux container covers the migration test. Existing product repositories and their data are outside this verification.

The Windows CMake build, CTest, native Qt suites, Cargo formatting, Clippy, locked build, test and governance validator returned exit 0. Windows Rust ran 68 tests. CTest ran one suite. The native 125% run logged 11 Qt Test passes including initialization and cleanup. The other scales ran the layout, empty and Vietnamese scenarios. The standalone application also opened a native window and initialized a private SQLite fixture. The validator checked 67 EN/VI document pairs.

The measured body text and canvas contrast is 15.03:1 in Dark and 15.07:1 in Light. Control outline contrast on the surface is 3.36:1 and 4.49:1 respectively. These measurements cover the stated color pairs.
