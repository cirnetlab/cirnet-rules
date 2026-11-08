<!-- doc-id: TESTS -->
<!-- revision: 1 -->
<a id="verification"></a>

# Governance tests

Run cargo test with the lockfile. Rust tests cover translation digests,
machine identifiers, malformed input, candidate release gates and isolated
adapter operations. CI also executes CLI validation and checks that a missing
product workspace is blocked. Product tests are run in their own repositories.
[Evidence rule](../rules/testing.md)
