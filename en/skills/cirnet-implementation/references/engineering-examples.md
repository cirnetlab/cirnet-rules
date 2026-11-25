# Engineering examples

These are synthetic fixtures with invented identifiers and contracts. The
checks describe what to run and what to expect in the fixtures. Product claims
require evidence from the relevant CIRNET system. Read the example that matches
the task.

## API and SDK compatibility

**Request** — Show a short group label in a consumer list. The selected SDK
returns a documented group code. Only the consumer repository is in scope.

**Before** — Add `groupLabel?: string` to a generated DTO and fall back to an
empty label. The producer still emits only the code. A hand edit disappears
at regeneration, and a new optional wire field can break consumers that reject
unknown properties.

**After** — Check the consumer pin and compare it with the producer owner's
schema. If fixed labels are acceptable, render the known codes with the
consumer's existing label formatter. Keep the SDK, wire behavior and unknown-code
fallback. If labels must come from the server, identify the producer migration
needed for the new field, including compatibility checks for strict decoders
and a compatible release for the consumer to select.

**Check** — Decode fixtures for the pinned SDK and render known and unknown
codes. Check that the diff touches only label rendering and its associated tests.

**Limit** — This covers consumer behavior with the selected SDK. A producer
release and compatibility with other consumers need separate evidence.

## Vue and TypeScript display fix

**Request** — A graph panel shows an em dash for an available-neighbor count of
zero. Show `0` and keep the dash for missing data.

**Before** — Rebuild the graph store and rename the count prop to address the
display issue. The render expression `count || '—'` treats zero as absent.

**After** — Change the template interpolation to `count ?? '—'`. Keep the
prop type, emitted events, graph state and styles.

**Check** — A focused component test renders `0`, an ordinary positive count
and `undefined`. The zero assertion fails before the edit. Check the
existing selection-event test and inspect the diff for the single template
expression and focused test.

**Limit** — This covers the mounted component and exercised events. Browser
end-to-end behavior and layout require their own checks.

## FPGA testbench sampling

**Request** — A synthetic RTL scoreboard flags an output mismatch immediately
after a rising edge. The registered outputs update at each rising edge
and stay stable between those updates.

**Before** — Disable the watchdog or hold the protection permit high to make the
test pass. Neither change resolves the scoreboard reading before the
nonblocking assignments settle, and both defeat a safety scenario.

**After** — Confirm the mismatch against the waveform. In this synchronous
fixture, sample the scoreboard on the following falling edge and compare
against the expectation for the preceding rising edge. Preserve DUT timing,
watchdog, permit checks, safe-state logic and the existing safety assertions.

**Check** — Compare the normal update waveform with the expected cycle. Run the
reset, permit loss and watchdog expiry cases with their original assertions and
confirm that the outputs enter the specified safe state.

**Limit** — RTL simulation covers the exercised model and timing. Board
operation, electrical readiness and programming require separate owner
authorization and evidence.

## Regression at an interval boundary

**Request** — A report for the half-open range `[start, end)` must include the
start tick and exclude the end tick.

**Before** — The filter uses `at > start && at <= end`. An assertion about the
row count passes because each implementation returns two rows from this input.

**After** — Assert the exact IDs from the public filter before editing it.
Then change the predicate to `at >= start && at < end`.

```typescript
const rows = [
  { id: 'at-start', at: 10 },
  { id: 'inside', at: 12 },
  { id: 'at-end', at: 15 },
]

expect(reportRows(rows, 10, 15).map(row => row.id)).toEqual(['at-start', 'inside'])
```

**Check** — The original predicate returns `['inside', 'at-end']` and fails
this assertion. The corrected predicate returns `['at-start', 'inside']` and
passes the same assertion. Run the affected filter suite to check nearby cases.

**Limit** — This fixture uses integer ticks. Timestamp parsing and time zone
conversion require separate coverage.
