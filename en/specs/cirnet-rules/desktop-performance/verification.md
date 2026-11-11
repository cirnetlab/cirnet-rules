<!-- doc-id: CIRNETRULES-DESKTOP-ASYNC-VERIFICATION -->
<!-- revision: 1 -->
<a id="evidence"></a>

# Desktop async verification

Baseline source is `ddc6892`. Use the same Release toolchain and display for each pair. The Windows 11 host used MSVC 2022, Qt 6.11.2, Rust and Cargo 1.99.1, an i5-12500H in Balanced power mode, a 1200 × 700 logical window and 125% DPI at 144 Hz. The Qt fixture selects from 24 conversations and returns a long line, more than 400 Unicode lines and a distinct final paint marker. Warm runs discard the first ten selections. First selection uses a new process. A separate runtime fixture delays the first Git command by one second before running real Git against a disposable checkout.

| Measure | Samples | Before | After |
| --- | ---: | ---: | ---: |
| First selection with 64 KiB line, ms | 1 per build | 146.37 | 24.84 |
| Warm first paint with 64 KiB line, p50/p95/p99/max ms | 50 per build | 148.78 / 187.67 / 206.86 / 206.86 | 39.28 / 61.66 / 69.74 / 69.74 |
| Warm first paint with 8 KiB line, p50/p95/p99/max ms | 50 per build | 33.86 / 43.96 / 54.48 / 54.48 | 41.01 / 57.63 / 74.83 / 74.83 |
| Sequential heartbeat with 64 KiB line, p50/p95/p99/max ms | 317 / 203 | 0.78 / 122.95 / 140.78 / 163.69 | 0 / 14.36 / 18.22 / 29.43 |
| Mixed heartbeat with 64 KiB line, p95/p99/max ms | 459 / 396 | 6.09 / 6.45 / 8.77 | 17.03 / 24.57 / 34.94 |
| Wire detail requests in 100 selections | 100 per build | 31 | 3 |
| Read round trip while Git waits, p50/p95 ms | 10 per build | 1111.85 / 1122.13 | 0.69 / 0.81 |
| Runtime queue wait / handler, p50 ms | 10 per build | 1111.09 / 0.50 | 0.08 / 0.41 |
| SQLite display query with 100,001 selected blockers, p50/p95/p99/max ms | 36 per query | 121.87 / 312.23 / 386.26 / 386.26 | 26.53 / 29.59 / 30.29 / 30.29 |

The slow in-order Qt fixture did not paint the final selection in the baseline drain. It painted it in 1.76 s after refactoring, while the intentionally serialized fixture still spent 2.3 s in its slow command. This fixture tests GUI responsiveness and coalescing. The real runtime test isolates the separate Git lane. The 8 KiB warm fixture regressed by 13.67 ms at p95. The mixed 64 KiB heartbeat exceeds the 16 ms p95 target by 1.03 ms. These results retain the original thresholds.

With 50 warm samples per run, the interleaved candidate / baseline / candidate sequence at 64 KiB returned first-paint p95 of 54.46 / 514.55 / 55.80 ms. Both candidate runs sent three detail requests and painted the final selection. Their mixed heartbeat p95 values were 17.46 and 18.11 ms. A separate run of the final candidate reached first-paint p95 of 473.78 ms and a maximum of 914.68 ms. It sent 69 detail requests, and mixed heartbeat p95 reached 64.21 ms. The warm GUI CPU time was comparable to the other candidate runs. The cause of that tail remains undetermined. The final 8 KiB rerun reached 50.17 ms p95, still above its 43.96 ms baseline.

An instrumented 64 KiB run measured the first 60 matched detail responses in each build. Parse p95 was 1.24 ms before and 1.42 ms after. Callback p95 was 469.37 ms before and 12.91 ms after, above the 4 ms apply target for the combined callback. Callback time includes model and synchronous document work. The paint probe measures actual transcript viewport paint separately. The query-only SQLite fixture used SQLite 3.50.4, 10,000 conversations, 50,005 messages and proposals, and 50,001 runs. A chat with six selected blockers remained near 0.03 ms p95 even with unrelated runs. Its baseline query plan uses indexed branches and scans a CTE scoped to the selected conversation. The heavy case pays for the exact count of its own blockers. A writer committed between metadata and watermark reads in the snapshot test. The following poll returned the new event.

At the 1,000-switch soak, GUI private bytes were 48,332,800 after warm-up and 48,275,456 at both 500 and 1,000 switches. QObject, QWidget and timer descendants stayed at 1096, 632 and 8. Process samples stayed at 12 threads and one child. Handles reached 1046 from 1044 and then plateaued. The GUI reconnect soak completed 100 cycles with 200 expected callbacks and zero child processes after drain. The Rust-only soak kept 84 handles and 13 threads across 1,000 alternating reads and cancels and also completed 100 reconnect/cancel cycles. Private bytes reached 2,920,448 from 2,887,680 and plateaued at sample 500. These are process resource samples. Live heap allocations, clean-host performance, and a full native GUI benchmark against a large real database are outside this evidence.

## Verification gates

- Rust formatting, Clippy with warnings denied, and the full Release build passed.
- The Rust suite passed 152 tests. The opt-in offline `uv` fixture passed separately.
- The Qt CTest preset passed. The native suite reported 53 passes, zero failures
  and two opt-in skips. The performance and reconnect workloads have separate
  native logs. The toast regression verifies focus in its host window across
  expiry.
- Governance validated 106 bilingual document pairs. The staged runtime checksum
  matched the Release binary and its pin. The `cnr` help and version commands ran.
- A deliberately unread stdout and stderr reproduced a stalled shutdown before the
  diagnostics fix. The final Release runtime exited in 2.04 s in the raw
  reproduction. The receipt, cancellation, and overload exercises passed.

## Reproduction

Build in the local compiler environment with the existing Qt installation and pinned Rust.

```powershell
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
$qtKit = Read-Host 'Qt MSVC kit directory'
$runtime = (Resolve-Path .\target\release\cirnetrules-runtime.exe).Path
cmake --preset desktop "-DCMAKE_PREFIX_PATH=$qtKit" "-DCIRNETRULES_RUNTIME_EXECUTABLE=$runtime"
cmake --build --preset desktop
ctest --preset desktop
.\target\release\cirnet-governance.exe validate
.\target\release\cnr.exe --help
.\target\release\cnr.exe --version
```

Add the kit's `bin` directory to the current process path before starting
`build/desktop/desktop/cirnetrules-desktop.exe`. Use an absolute `--data-dir`
for a disposable Dashboard instance.

Run `asyncConversationPerformance` from the staged Qt Test executable with
`QT_QPA_PLATFORM=windows`, `CIRNETRULES_TEST_PRIMARY_SCREEN=1` and
`CIRNETRULES_ASYNC_BENCH=1`. Set `CIRNETRULES_ASYNC_BENCH_TYPICAL=1` for the
8 KiB variant and `CIRNETRULES_ASYNC_BENCH_SOAK=1` for 1,000 switches.
Run `asyncRuntimeReconnectSoak` with `CIRNETRULES_ASYNC_BENCH_RECONNECT=1` for process
lifecycle sampling. A Qt Test text log captures the measurements.
Run `python src/tests/runtime_performance.py` with a Release runtime path,
`--iterations 10` and `--trace` followed by an output file. Run the same command
against the saved baseline binary. The candidate also supports `--exercise`
for slow consumer, cancellation and receipt checks and `--soak` for resource
sampling. Use a disposable data directory for all integration tests.

Set `CIRNETRULES_TRACE_IPC=1` for a process to capture request-ID timings in
local Qt diagnostic output or runtime stderr. Each client trace has a 256 KiB
budget. The runtime admits diagnostics into 64 slots of at most 512 bytes each
and limits request tracing to 2,048 records. The runtime can close a stalled
stdout while the diagnostic sink is waiting on stderr. Output includes timings,
known method names, queue counts and frame sizes. The trace excludes payloads,
paths and environment values. Unset the variable to turn tracing off. Compare
durations within each process and correlate by ID.

## Ownership and rollback

`runtime_client.cpp` owns IPC framing, expiry, coalesced intent slots and
process cleanup. `conversations.cpp` owns visit generations, transcript bounds
and widget updates. `src/bin/runtime/dispatcher.rs` owns lane admission, cancellation
and response delivery. `src/dashboard/storage.rs` owns connection creation and
progress-hook cleanup. `src/dashboard/sessions.rs` and `src/dashboard/sessions/sqlite.rs`
own the snapshot and bounded page queries. The Git subprocess uses the existing
contained process runner and shares a deadline across commands.

The frame ceiling is 256 KiB. The UI retains at most 32 wire callbacks or
credits, two queued view intents and a transcript window of at most 48 messages,
3,500 blocks and 768 KiB of retained UTF-16 strings. A message preview caps a
line at 1,024 UTF-16 units. The full-message viewer serves the saved text for
selection and copy. Drafts retain their separate
limits. Each runtime lane has one active and eight queued requests. Response
capacity reserves 32 ordinary slots, four rejection slots and eight control
slots. The writer retains its receipt when a response is lost and the pipe has
a two-second stall limit. File operations without cooperative hooks can run
until process containment or their own deadline.

To roll back the source, close Desktop, restore the previous source revision
and rebuild and stage its matching runtime. Keep the existing data directory
and any reviewed backups. This change requires no database migration. A runtime
without `read_cancellation` still accepts the existing v3 request envelope.
