<!-- doc-id: GLOSSARY -->
<!-- revision: 1 -->
<a id="glossary"></a>

# Terms and translations

| Identifier | Meaning |
| --- | --- |
| `releaseTrain` | Exact system release target and selection. |
| `family` | CIRNET compatibility label and a string. It requires an exact approved combination. |
| `contractRevision` | Concrete source and digest for a contract. Independent of an API path suffix or package version. |
| Commit, tree, blob, SHA-256 | Git commit identifies history and a tree. Tree identifies the snapshot graph. Blob identifies a file object. SHA-256 here identifies artifact bytes. |
| `OBSERVED` / `VERIFIED` | Source observation / performed check with bounded evidence. |
| `TARGET` / `PROPOSED` | User-defined direction / design decision awaiting approval. |
| `UNKNOWN` / `BLOCKED` | Missing fact / gate that cannot proceed with available evidence. |

Review modality, versions and safety vocabulary in both languages. Preserve
identifiers such as schema keys, URLs, enum values, operation IDs and errors.
[Documentation rule](../rules/documentation.md)
