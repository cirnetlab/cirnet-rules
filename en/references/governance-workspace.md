<!-- doc-id: CIRNETRULES-GOVERNANCE-WORKSPACE -->
<!-- revision: 2 -->
<a id="workspace"></a>

# Governance workspace

> Historical record for the [archived source](https://github.com/cirnetlab/cirnet-rules/blob/52c163e638578bc8f8afe03b3d7bf7e2c0d21b8c/en/references/governance-workspace.md). Use the [current CLI workflow](cirnetrules-local.md) for operations.

Open **Settings → Configuration**. The Library uses one paged catalog for skills, rules, profiles, references, policies, commands and hooks. Filter by kind or search by name. Select a row to load its current draft, if any, or its published revision. The Agents & Runtime page retains diagnostics and the current execution limits.

## Draft, review and publication

1. Choose **New asset**, enter a title and a stable slug, and choose the asset type. For a skill, enter the canonical Markdown with `name` and `description` in its frontmatter. Use **Import file** to preview a UTF-8 source and create a local draft with its filename as provenance.
2. Edit **EN · Canonical** and **VI · Translation**. Use **Files → Add file** for an asset-relative text file or **Attach text** for a selected local file. Use the Dependency IDs field for stable asset relationships. Use Ctrl+F to find and replace in the current editor and Ctrl+S to save a draft.
3. Save or wait for autosave, then choose **Validate**. Inspect the diagnostic and translation status. When an English revision changes, review the Vietnamese text for the same revision. Markdown preview disables HTML and resource loading.
4. Use **View diff** to inspect the changes, then **Review** and **Apply**. Apply requires the reviewed draft digest and current revision. Commands, hooks and policies can be drafted. Publication that would require authenticated review or an execution boundary remains blocked.
5. Use **History** to inspect immutable published revisions. Rolling back an eligible snapshot creates a new revision. **Asset actions** contains rename, duplicate, archive, restore, logical removal and export. Archive or removal fails when an active consumer remains. Reload the saved revision when another editor has changed the same asset.

Use **Assignments** to attach a published asset to a registered repository binding. In **Effective context**, choose the binding and mode. Selecting a profile in Library adds that profile to the preview. The preview shows English input size, deferred skills and references, origin, revision, and excluded entries. It is a simulation. Runtime scope and hard denials still apply.

## Authority and boundaries

The local authoring service stores editable documents, drafts, virtual attachment paths, immutable published revisions and audit state in one SQLite authority through the Dashboard DAL. Publication changes the pointer and writes its audit record in one SQL transaction. The application has no writable binding to checkout governance files in this increment. Import treats a selected file as input data, and export is a reviewed snapshot. Files in a repository and instructions imported from other tools carry no operator authority.

The stdio runtime validates the protocol before dispatch. Mutations use a request identity and an expected revision. A retry with identical inputs reuses its request identity. The local desktop channel identifies the operator boundary at the level of the current OS account. SQLite integrity and revision digests do not authenticate a different human reviewer or isolate other processes running as that account.

The Integrations panel supports a read-only loopback probe and dialect inspection. Capability receipts describe the checked surface and its limits. See [adapter capabilities and source evidence](adapter-capabilities.md).
