<!-- doc-id: CIRNETRULES-DESKTOP-RELEASE -->
<!-- revision: 1 -->
<a id="candidate"></a>

# Dashboard application candidate

| Identity | Candidate |
| --- | --- |
| Display name | CIRNET RULES |
| Product ID | `cirnetrules` |
| Application version | 0.1.0 |
| Release title | `cirnet-rules-0.1.0` |
| Proposed tag | `v0.1.0` |
| Publication gate | `RELEASE_BLOCKED` |

This candidate adds native Qt repository selection, persistent Git worktree
bindings, durable conversation and scoped intake, SQLite backup and reviewed
legacy archive, and a backend automation scheduler. Catalog registration
does not grant execution or file writes. A relink invalidates the previous
binding revision. The package targets Windows x64 MSVC.

The candidate archive uses the product name in its filename. The release
identifier in the manifest still uses the repository name.

The archive manifest records the base source revision, fingerprint of the
working sources, installed file digests and archive SHA-256. Keep the manifest
and checksum next to the zip. Verify both before testing. Extract the archive
into a private directory and run the Dashboard under bin. Use a disposable
data directory and the [operator workflow](dashboard.md) for verification.

The release gate still requires native provider execution, verified isolation,
authenticated independent review, a clean Windows installation and review
of redistribution requirements. Do not publish the local candidate as the
release. Existing source histories and product release authorities are
preserved. See [architecture and limits](dashboard-architecture.md).
