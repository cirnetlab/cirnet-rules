# Adaptive Stable integration plan

Scope: the CIRNET rules checkout and the supplied read-only overlay. The
[specification](../../../specs/cirnet-rules/adaptive-stable/spec.md) sets the
behavior and acceptance boundaries.

| Step | Check |
| --- | --- |
| Review the overlay and checkout | Compare anchor edits with current source and freeze the starting Git state. |
| Update the authoring Skill, template and guides | Review each EN/VI pair, verify exact source hashes and count 13 Skills. |
| Wire the Candidate-only smoke to the role runner | Preview and test the Author to Work path and its failure handling with an offline shim. |
| Review the diff and verification | Run the documentation, Cargo and offline regression checks. Verify historical inputs and evidence against the starting state. |

## Verification record

The integration started from a clean `master` at `fdfe0949`. The overlay
was reconciled manually so both context guides received the lifecycle section.
The offline smoke passed preview, byte-exact handoff, synthetic privacy guards,
objective failure and blocked-evidence checks. The source tests confirmed
13 Skills, one Work Prompt template pair and matching source hashes.

Cargo formatting, tests and Clippy, the changed-document selector, package and
install fixtures, and the offline behavioral, follow-up and A/B shim checks passed.
The architecture snapshots and 5,525 historical files under `.superpowers`
matched the starting checkout. RTL simulation lacked `iverilog` and `vvp`.
Windows skipped link cases that require symlink or reparse-point permissions.

Native smoke, independent behavioral review and the full native benchmark
remain `NOT_RUN`. No native usage was measured. Native execution requires
separate authorization, and compatibility requires independent review of
the resulting evidence.
