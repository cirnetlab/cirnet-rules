<!-- doc-id: CNR-MCP-COORDINATION-SPEC -->
<!-- revision: 1 -->
<a id="scope"></a>

# Shared task coordination and MCP

One Rust service reads Git governance and the existing local task store. CLI and MCP use the same task mutations. A coordinated task freezes its selected worktree and reserves it for a single writer. Lease generations fence stale workers. Committed task records link their receipts and audit events.

Reports preserve bounded worker assertions and independently observed source digests. Dependency and CI checks bind evidence to the exact accepted producer and candidate. Operator grants and independent acceptance remain governed by the existing lifecycle.

MCP starts with a fixed authority and workspace. Its read-only profile exposes bounded governance tools. Coordinated access requires a separately authorized local session. Client configuration is exported for review. Product checkout writes, native installation and release activation remain separate operations.
