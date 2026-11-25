---
name: cirnet-candidate-evaluation
description: Use when planning, running, reviewing, or reporting a CIRNET Single Candidate behavioral evaluation.
---

# Evaluate a single CIRNET Candidate

Use Single Candidate Evaluation to run each selected engineering scenario directly. Use product-owner checks for product acceptance.

1. **Select by risk.** Identify the affected behavior, tool or trust boundary.
   Select the smallest set of catalog scenarios that exercises it. Record the
   selection and rationale. A full E01–E08 rerun requires a separate rationale
   that explains the coverage needed across all eight cases.
2. **Freeze and preview.** Use the existing benchmark runner and a fresh owned
   session. Verify the source revision, frozen Rules, Skills, evaluator and
   original fixture hashes. Keep historical reports, original E01–E08
   evidence and archived A/B and Adaptive Stable Orchestrator material intact.
   For native calls, obtain separate authorization and the exact scenario call
   cap. Review the frozen scope and fixtures before native execution.
3. **Grade the evidence.** Check execution, objective results, scope audit,
   privacy and the sealed evidence. A shim result covers the exercised
   harness. `OBJECTIVE_PASS_REVIEW_REQUIRED` leaves behavioral review pending.
   An independent reviewer assesses actual native evidence against the frozen
   criteria and indexed artifacts. Use the direct run’s output and tool trace as evidence. Claim native behavioral PASS only when the final quality
   verdict records PASS for the criteria in scope.
4. **Report the scope.** Record scenarios, revision, hashes, runtime settings
   and evidence locators. Use `PASS` for supported criteria, `FAIL` for observed
   failures, `BLOCKED` for unmet prerequisites and `NOT_RUN` for unexecuted
   checks. State which results cover the harness, objective checks or native
   behavior, and identify unverified criteria.

Use Single Candidate for current evaluation and retain A/B tooling in the
historical archive. Route a demonstrated runner compatibility defect through
separate authorization before changing the runner.

Use the applicable revision in the approved checkout.
Single Candidate procedure (`en/docs/behavioral-evaluation.md`) ·
Risk-based compatibility (`en/docs/model-compatibility.md`)
