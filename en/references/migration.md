<!-- doc-id: MIGRATION-MAP -->
<!-- revision: 1 -->
<a id="migration-map"></a>

# Old to new paths

The baseline governance checkout contained a README title, license and
tracked placeholders. This migration retains the license and Git history.

| Old path | New path or disposition |
| --- | --- |
| Root AGENTS and README | Thin bootstrap and bilingual navigation. |
| Root rules and profiles | Matching English and Vietnamese rules and profiles. |
| Root policies YAML placeholders | Shared policy prose and the single root governance registry. |
| Root schemas | Executable governance schemas under en/schemas, with one Vietnamese guide. |
| Root tools Python placeholders | Rust engine under en/tools, with a Vietnamese guide. |
| Root adapter, hook, test and reference placeholders | Functional directories under en and vi. |
| Root commands per-repo placeholders | Shared commands guides. |
| Root plans and specs | Six product plans/specs under en and vi. The CLI placeholder is ancillary and outside the baseline. |
| Root skills | Shared English workflows and Vietnamese counterparts. |

Discovery uses a selected English manifest. Historical sources remain accessible
from Git. Cross-links are checked in both languages. [Document rule](../rules/documentation.md)
