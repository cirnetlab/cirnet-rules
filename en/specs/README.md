# Specifications

A spec captures intended behavior, ownership boundaries, non-goals and acceptance.
Use `en/specs/cirnet-rules/<task>/spec.md` with a matching Vietnamese document
for nontrivial common-rules changes. Start from the [spec template](../templates/spec.md)
when needed; do not create a spec solely for a tiny edit unless local policy requires it.

Keep requirements separate from verification results. If implementation reveals a
design change, update the English spec and its translation, explain the decision
and check affected acceptance. Do not silently rewrite an old requirement to make
a failing result appear compliant.

Historical records retain their original date and scope. Current requirements
supersede earlier design choices explicitly; a historical plan is not fresh authority.
Product repositories keep their own spec directories and acceptance owners.

Task records: [initial rebuild](cirnet-rules/rules-common/spec.md) (historical),
[bilingual upgrade](cirnet-rules/bilingual-upgrade/spec.md),
[toolkit upgrade](cirnet-rules/toolkit-upgrade/spec.md),
[engineering behavior](cirnet-rules/engineering-behavior/spec.md),
[Task Continuity](cirnet-rules/task-continuity/spec.md).
