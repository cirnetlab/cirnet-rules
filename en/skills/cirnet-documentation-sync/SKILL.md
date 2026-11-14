---
name: cirnet-documentation-sync
description: Use when a change affects instructions, navigation, examples, or paired English and Vietnamese documentation.
---

# Documentation synchronization

Use for affected docs. Leave unrelated history and translations out of the task.

Minimum inputs: Affected files, canonical language, actual change/test evidence and local document policy.

1. Find affected nodes from the diff and their navigation links. Update the canonical source first.
2. Update the corresponding translation after checking commands, versions, limits and meaning. Refresh the source hash after review.
3. Validate links and examples using the applicable checker. Preserve published history and distinguish old runs from current ones.

Report changed pairs, resolved navigation and verification. A matching hash establishes freshness of the reviewed source.

Example: An install flag changes. Update setup examples and their translation, then test the shown command against a fixture before marking it verified.
