---
name: cirnet-cross-repo-compatibility
description: Use when checking a producer revision or artifact against what a separate consumer repository selects.
---

# Cross-repository compatibility

Use to compare a producer and direct consumers. Use contract change when editing owned canonical formats.

Minimum inputs: Owner and consumer, selected revisions/artifact hashes, affected boundary and permitted repositories.

1. Read the owning manifest and consumer's actual dependency selector. Keep local source, candidate and system selection distinct.
2. Verify each required direction and direct consumer under its documented test lane. Record exact inputs and limits.
3. Describe necessary owner changes and coordinate permissions before editing any other repository.

Report producer artifact, consumer pin, direction, result, migration and unknowns for each checked pair.

Example: A UI pins an older SDK while a newer producer candidate exists. Check the pinned SDK and selected server separately from the candidate.
