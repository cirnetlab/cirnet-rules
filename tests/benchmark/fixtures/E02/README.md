# Duration label

`formatDuration(seconds)` receives an integer from 0 through 3599 inclusive.
Return a five-character `MM:SS` label, with each group padded to two digits.
Examples: 0 → `00:00`, 7 → `00:07`, 83 → `01:23`, 3599 → `59:59`.
Callers validate the range; behavior outside it is unspecified.

Run the checks with `node --test`.
