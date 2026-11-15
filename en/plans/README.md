# Plans

Plans describe how to execute a scoped change. For a nontrivial common-rules task,
store a small plan at `en/plans/cirnet-rules/<task>/plan.md` and its same-path
Vietnamese counterpart. Keep specifications under the corresponding specs tree.

Use the [plan template](../templates/plan.md) when it helps. Record scope, ordered
steps, meaningful checks and progress; link the specification instead of repeating it.
After implementation/build, update completed steps and observed outcomes in both
languages. Do not mark steps complete from intention or a successful unrelated test.

A typo needs no new plan unless local rules require one. Product tasks use their
owner's planning location, including existing `.superpowers` conventions; this
central directory does not replace those rules.

Task records: [initial rebuild](cirnet-rules/rules-common/plan.md) (historical),
[bilingual upgrade](cirnet-rules/bilingual-upgrade/plan.md),
[toolkit upgrade](cirnet-rules/toolkit-upgrade/plan.md),
[engineering behavior](cirnet-rules/engineering-behavior/plan.md),
[E01/E08 recovery](cirnet-rules/e01-e08-recovery/plan.md).
