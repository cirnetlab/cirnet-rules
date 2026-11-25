# Changes across ownership boundaries

Use for changes to repository ownership, an API, a shared model, a generated SDK
or an integration input. Start with [architecture selection](../policies/architecture.md)
and [ownership routing](../references/architecture-routing.md).

1. Select the architecture for the affected boundary. Verify the owner, producer
   and direct consumers from their current source and manifests. Use the target
   repository's instructions for implementation, commands and tests.
2. Compare the exact selected dependency with the producer revision being read.
   Trace schemas, serialization and fixtures to distinguish implementation
   details from canonical contract changes. Bring material architecture conflicts
   to the affected owners for review before changing that boundary.
3. If the task needs a producer change outside the permitted repo, describe the exact
   owner-side change and blocked consumer work. Continue independent scoped work;
   obtain explicit scope before editing another repository.
4. If several repos are authorized, validate each checkout and its local instructions
   independently. Keep each diff and verification result attributable to its owner.
5. Report compatibility and availability separately from deployment or release
   selection. A newer source version does not update a consumer's selected artifact.

Use the [dated repository map](../references/repository-map.md) for earlier
observations. Keep product versions, safety rules and release gates at their owners.
