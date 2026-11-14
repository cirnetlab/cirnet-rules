# Changes across ownership boundaries

Use when a task changes an API, shared model, generated SDK or integration input.
Start from the [baseline](../AGENTS.md); the
[repository map](../references/repository-map.md) is a dated navigation aid.

1. Identify the producer and direct consumers from their current source and manifests.
   Compare the consumer's selected dependency with the producer revision being read.
2. Distinguish implementation details from canonical contract changes. Trace schema,
   serialization and compatibility fixtures before choosing where to edit.
3. If the task needs a producer change outside the permitted repo, describe the exact
   owner-side change and blocked consumer work. Continue independent scoped work;
   obtain explicit scope before editing another repository.
4. If several repos are authorized, validate each checkout and its local instructions
   independently. Keep each diff and verification result attributable to its owner.
5. Report compatibility and availability separately from deployment or release
   selection. A newer source version does not update a consumer's selected artifact.

For CIRNET, Contracts owns canonical schemas and SDK; Remote owns system release
selection. Confirm current ownership in those repositories. Do not copy their
versions, product safety rules or release gates into the common baseline.
