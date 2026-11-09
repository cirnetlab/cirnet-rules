//! Local operator governance authoring. The runtime owns the authority and write transactions.
mod model;
mod sqlite;

use super::storage::{AppError, Database, decode};
use model::{
    Attachment, Diagnostic, Document, Format, HARD_DENIES, Kind, checked_text, digest, encode,
    translation, valid_asset_id,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlite::{Asset, Store};
use std::collections::{BTreeSet, VecDeque};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct List {
    kind: Option<Kind>,
    search: Option<String>,
    include_archived: Option<bool>,
    after: Option<String>,
    limit: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileList {
    after: Option<String>,
    limit: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Create {
    kind: Kind,
    slug: String,
    title: String,
    format: Option<Format>,
    source_name: Option<String>,
    mutation_key: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Expected {
    id: String,
    expected_revision: i64,
}

#[derive(Deserialize)]
struct Change<T> {
    id: String,
    expected_revision: i64,
    mutation_key: String,
    #[serde(flatten)]
    input: T,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rename {
    slug: String,
    title: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    title: Option<String>,
    description: Option<String>,
    en: String,
    vi: String,
    format: Format,
    files: Option<Vec<Attachment>>,
    dependencies: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Publish {
    review_token: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Rollback {
    target_revision: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RevisionGet {
    id: String,
    revision: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct History {
    id: String,
    limit: Option<u32>,
    before: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Assignment {
    binding_id: String,
    enabled: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProfileDefault {
    enabled: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Assigned {
    binding_id: Option<String>,
    limit: Option<u32>,
    after: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Effective {
    binding_id: Option<String>,
    profile_id: Option<String>,
    mode: Option<String>,
    budget_bytes: Option<usize>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Bundle {
    bundle_version: u8,
    kind: Kind,
    slug: String,
    title: String,
    revision: i64,
    digest: String,
    source_name: Option<String>,
    document: Document,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BundlePreview {
    bundle: Bundle,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleApply {
    bundle: Bundle,
    fingerprint: String,
    slug: Option<String>,
    mutation_key: String,
}

#[derive(Serialize)]
struct Check {
    valid: bool,
    translation: &'static str,
    diagnostics: Vec<Diagnostic>,
}

pub fn dispatch(db: &mut Database, method: &str, params: Value) -> Result<Value, AppError> {
    match method {
        "governance.asset.list" => list(&Store::new(db), decode(params)?),
        "governance.asset.get" => {
            let input: Identity = decode(params)?;
            Store::read(db, |store| detail(store, &find(store, &input.id)?))
        }
        "governance.asset.create" => {
            let input: Create = decode(params.clone())?;
            let key = input.mutation_key.clone();
            mutate(db, method, &params, &key, |store, at| {
                create(store, input, at)
            })
        }
        "governance.asset.rename" | "governance.asset.duplicate" => {
            change(db, method, params, |store, asset, input: Rename, at| {
                model::check_identity(&input.slug, &input.title)?;
                if method.ends_with(".rename") {
                    store.rename(asset, &input.slug, &input.title, at)?;
                    Ok(asset.id.clone())
                } else {
                    let doc = store
                        .draft(&asset.id)?
                        .or(store.active(asset)?)
                        .unwrap_or_else(|| Document::empty(asset.kind.default_format()));
                    let id = store.insert(
                        asset.kind,
                        &input.slug,
                        &input.title,
                        asset.source_name.as_deref(),
                        &doc,
                        at,
                    )?;
                    store.store_initial_relations(&id, &doc.dependencies)?;
                    Ok(id)
                }
            })
        }
        "governance.draft.save" => change(db, method, params, save),
        "governance.draft.discard" => change(db, method, params, |store, asset, _: Empty, at| {
            store.set_draft(asset, &asset.title, None, at)?;
            Ok(asset.id.clone())
        }),
        "governance.revision.validate" | "governance.revision.compare" => {
            Store::read(db, |store| {
                inspect_revision(store, decode(params)?, method.ends_with(".compare"))
            })
        }
        "governance.revision.get" => Store::read(db, |store| revision_get(store, decode(params)?)),
        "governance.change.history" => {
            Store::read(db, |store| change_history(store, decode(params)?))
        }
        "governance.revision.history" => history(&Store::new(db), decode(params)?),
        "governance.change.review" => change(db, method, params, |store, asset, _: Empty, at| {
            asset.kind.require_authenticated_review()?;
            let draft = require_draft(store, asset)?;
            require_valid(store, asset, &draft, store.active(asset)?.as_ref())?;
            store.prepare_review(asset, &digest(&encode(&draft)?), at)?;
            Ok(asset.id.clone())
        }),
        "governance.change.publish" => {
            change(db, method, params, |store, asset, input: Publish, at| {
                asset.kind.require_authenticated_review()?;
                let draft = require_draft(store, asset)?;
                let (token, expected) = store.review(asset, at)?;
                if token != input.review_token || digest(&encode(&draft)?) != expected {
                    return Err(AppError::new(
                        "PERMISSION_DENIED",
                        "Review the exact current draft before applying it.",
                    ));
                }
                require_valid(store, asset, &draft, store.active(asset)?.as_ref())?;
                store.activate(asset, &draft, at, None)?;
                Ok(asset.id.clone())
            })
        }
        "governance.change.rollback" => {
            change(db, method, params, |store, asset, input: Rollback, at| {
                asset.kind.require_authenticated_review()?;
                if asset.draft {
                    return Err(AppError::new(
                        "CONFLICT",
                        "Discard or publish the pending draft before rollback.",
                    ));
                }
                if asset.active_revision == Some(input.target_revision) {
                    return Err(AppError::new(
                        "INVALID_INPUT",
                        "Select an earlier published revision.",
                    ));
                }
                let document = store.revision(&asset.id, input.target_revision)?;
                require_valid(store, asset, &document, None)?;
                store.activate(asset, &document, at, Some(input.target_revision))?;
                Ok(asset.id.clone())
            })
        }
        "governance.asset.archive" | "governance.asset.restore" | "governance.asset.delete" => {
            change(db, method, params, |store, asset, _: Empty, at| {
                status(store, asset, method, at)
            })
        }
        "governance.assignment.list" => Store::read(db, |store| assigned(store, decode(params)?)),
        "governance.assignment.update" => {
            change(db, method, params, |store, asset, input: Assignment, at| {
                if input.enabled {
                    store.binding(&input.binding_id)?;
                    if asset.active_revision.is_none() {
                        return Err(AppError::new(
                            "INVALID_INPUT",
                            "Apply a revision before assigning this asset.",
                        ));
                    }
                }
                store.assign(asset, &input.binding_id, input.enabled, at)?;
                Ok(asset.id.clone())
            })
        }
        "governance.profile.list" => Store::read(db, |store| profiles(store, decode(params)?)),
        "governance.profile.default" => change(
            db,
            method,
            params,
            |store, asset, input: ProfileDefault, at| {
                if asset.kind != Kind::Profile || asset.active_revision.is_none() {
                    return Err(AppError::new(
                        "INVALID_INPUT",
                        "Apply the profile before making it a default.",
                    ));
                }
                store.set_default_profile(asset, input.enabled, at)?;
                Ok(asset.id.clone())
            },
        ),
        "governance.effective.preview" => {
            Store::read(db, |store| effective(store, decode(params)?))
        }
        "governance.asset.export" => Store::read(db, |store| export(store, decode(params)?)),
        "governance.asset.import.preview" => {
            let input: BundlePreview = decode(params)?;
            Store::read(db, |store| bundle_preview(store, &input.bundle))
        }
        "governance.asset.import.apply" => {
            let input: BundleApply = decode(params.clone())?;
            let key = input.mutation_key.clone();
            mutate(db, method, &params, &key, |store, at| {
                import_bundle(store, input, at)
            })
        }
        _ => Err(AppError::new(
            "METHOD_NOT_FOUND",
            "Unknown governance operation.",
        )),
    }
}

fn list(store: &Store<'_>, input: List) -> Result<Value, AppError> {
    let limit = page_size(input.limit)?;
    let after = cursor(input.after)?;
    let search = input.search.unwrap_or_default();
    checked_text(&search, 120, true)?;
    let mut items = store.assets(
        input.kind,
        &search,
        input.include_archived.unwrap_or(false),
        false,
        &after,
        limit + 1,
    )?;
    let next_after = if items.len() > limit as usize {
        items.pop();
        items.last().map(|asset| &asset.id)
    } else {
        None
    };
    Ok(json!({"items":items,"next_after":next_after}))
}

fn profiles(store: &Store<'_>, input: ProfileList) -> Result<Value, AppError> {
    let limit = page_size(input.limit)?;
    let after = cursor(input.after)?;
    let mut items = store.assets(Some(Kind::Profile), "", false, true, &after, limit + 1)?;
    for asset in &mut items {
        if let Some(revision) = asset.active_revision {
            let metadata = store.revision_metadata(&asset.id, revision)?;
            asset.title = metadata.title;
            asset.slug = metadata.slug;
        }
    }
    let next_after = if items.len() > limit as usize {
        items.pop();
        items.last().map(|asset| &asset.id)
    } else {
        None
    };
    Ok(json!({"items":items,"next_after":next_after,"default_profile_id":store.default_profile()?}))
}

fn cursor(after: Option<String>) -> Result<String, AppError> {
    let value = after.unwrap_or_default();
    if !value.is_empty() && !valid_asset_id(&value) {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Invalid governance page cursor.",
        ));
    }
    Ok(value)
}

fn page_size(limit: Option<u32>) -> Result<u32, AppError> {
    let limit = limit.unwrap_or(40);
    if !(1..=100).contains(&limit) {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Governance page size must be between 1 and 100.",
        ));
    }
    Ok(limit)
}

fn find(store: &Store<'_>, id: &str) -> Result<Asset, AppError> {
    if !valid_asset_id(id) {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Invalid governance asset ID.",
        ));
    }
    store.asset(id, false)
}

fn expected(asset: &Asset, value: i64) -> Result<(), AppError> {
    if asset.revision != value {
        return Err(
            AppError::new("CONFLICT", "The asset changed. Reload the latest revision.")
                .with_details(json!({"current_revision":asset.revision})),
        );
    }
    Ok(())
}

fn detail(store: &Store<'_>, asset: &Asset) -> Result<Value, AppError> {
    if asset.deleted_at.is_some() {
        return Ok(json!({"id":asset.id,"revision":asset.revision,"deleted":true}));
    }
    let active = store.active(asset)?;
    let draft = store.draft(&asset.id)?;
    let document = draft.as_ref().or(active.as_ref());
    let mut result = json!(asset);
    result["document"] = json!(document);
    result["translation"] = json!(
        document
            .map(|document| translation(document, active.as_ref()))
            .unwrap_or("MISSING")
    );
    result["default_profile"] = json!(
        asset.kind == Kind::Profile && store.default_profile()?.as_deref() == Some(&asset.id)
    );
    result["used_by"] = json!(store.reverse(&asset.id)?);
    result["assigned_bindings"] = json!(store.used_bindings(&asset.id)?);
    Ok(result)
}

fn create(store: &Store<'_>, input: Create, at: i64) -> Result<String, AppError> {
    model::check_identity(&input.slug, &input.title)?;
    if let Some(source) = &input.source_name {
        checked_source_name(source)?;
    }
    store.insert(
        input.kind,
        &input.slug,
        &input.title,
        input.source_name.as_deref(),
        &Document::empty(input.format.unwrap_or_else(|| input.kind.default_format())),
        at,
    )
}

fn save(store: &Store<'_>, asset: &Asset, input: Save, at: i64) -> Result<String, AppError> {
    let title = input.title.as_deref().unwrap_or(&asset.title);
    model::check_identity(&asset.slug, title)?;
    let original = store
        .draft(&asset.id)?
        .or(store.active(asset)?)
        .unwrap_or_else(|| Document::empty(asset.kind.default_format()));
    let document = Document {
        en: input.en,
        vi: input.vi,
        format: input.format,
        description: input.description.unwrap_or(original.description),
        files: input.files.unwrap_or(original.files),
        dependencies: input.dependencies.unwrap_or(original.dependencies),
    };
    document.check_bounds()?;
    for id in &document.dependencies {
        if id == &asset.id {
            return Err(AppError::new(
                "INVALID_INPUT",
                "An asset cannot reference itself.",
            ));
        }
        find(store, id)?;
    }
    store.set_draft(asset, title, Some(&document), at)?;
    Ok(asset.id.clone())
}

fn require_draft(store: &Store<'_>, asset: &Asset) -> Result<Document, AppError> {
    store
        .draft(&asset.id)?
        .ok_or_else(|| AppError::new("DRAFT_REQUIRED", "Save a draft first."))
}

fn check(
    store: &Store<'_>,
    asset: &Asset,
    document: &Document,
    active: Option<&Document>,
) -> Result<Check, AppError> {
    let mut diagnostics = model::validate(asset.kind, document, active);
    for id in &document.dependencies {
        match find(store, id) {
            Ok(asset) if asset.archived || asset.active_revision.is_none() => {
                diagnostics.push(Diagnostic::new(
                    "DEPENDENCY_INACTIVE",
                    "dependencies",
                    format!("Dependency {id} needs an active revision."),
                ))
            }
            Ok(_) if store.reaches(id, &asset.id)? => diagnostics.push(Diagnostic::new(
                "DEPENDENCY_CYCLE",
                "dependencies",
                format!("Dependency {id} would create a cycle."),
            )),
            Ok(_) => {}
            Err(issue) if issue.code == "NOT_FOUND" => diagnostics.push(Diagnostic::new(
                "DEPENDENCY_MISSING",
                "dependencies",
                format!("Dependency {id} no longer exists."),
            )),
            Err(issue) => return Err(issue),
        }
    }
    Ok(Check {
        valid: diagnostics.is_empty(),
        translation: translation(document, active),
        diagnostics,
    })
}

fn require_valid(
    store: &Store<'_>,
    asset: &Asset,
    document: &Document,
    active: Option<&Document>,
) -> Result<(), AppError> {
    let result = check(store, asset, document, active)?;
    if !result.valid {
        return Err(
            AppError::new("VALIDATION_FAILED", "The draft has blocking diagnostics.")
                .with_details(json!(result)),
        );
    }
    Ok(())
}

fn inspect_revision(store: &Store<'_>, input: Expected, compare: bool) -> Result<Value, AppError> {
    let asset = find(store, &input.id)?;
    expected(&asset, input.expected_revision)?;
    let active = store.active(&asset)?;
    let draft = store.draft(&asset.id)?;
    let document = draft
        .as_ref()
        .or(active.as_ref())
        .ok_or_else(|| AppError::new("DRAFT_REQUIRED", "Save a draft first."))?;
    let mut view = json!(check(store, &asset, document, active.as_ref())?);
    if compare {
        let previous = active.as_ref();
        let en_before = previous.map(|value| value.en.as_str()).unwrap_or_default();
        let vi_before = previous.map(|value| value.vi.as_str()).unwrap_or_default();
        let sizes = |name, before: &str, after: &str| {
            json!({
                "field":name, "before_bytes":before.len(),"after_bytes":after.len(),"changed":before!=after
            })
        };
        let before_files = previous
            .map(|value| value.files.as_slice())
            .unwrap_or_default();
        let before_deps = previous
            .map(|value| value.dependencies.as_slice())
            .unwrap_or_default();
        view["changes"] = json!([
            sizes("en", en_before, &document.en),
            sizes("vi", vi_before, &document.vi),
            sizes("files", &encode(before_files)?, &encode(&document.files)?),
            sizes(
                "dependencies",
                &encode(before_deps)?,
                &encode(&document.dependencies)?
            )
        ]);
        view["active"] = json!(active);
        view["draft"] = json!(draft);
        let exceeds_frame = encode(&view)?.len() > 230_000;
        if exceeds_frame {
            view["active"] = Value::Null;
            view["draft"] = Value::Null;
        }
        view["details_truncated"] = json!(exceeds_frame);
    }
    Ok(view)
}

fn revision_get(store: &Store<'_>, input: RevisionGet) -> Result<Value, AppError> {
    if !valid_asset_id(&input.id) || input.revision < 1 {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Select an exact published asset revision.",
        ));
    }
    let document = store.revision(&input.id, input.revision)?;
    let metadata = store.revision_metadata(&input.id, input.revision)?;
    Ok(
        json!({"id":input.id,"revision":input.revision,"digest":metadata.digest,
        "title":metadata.title,"slug":metadata.slug,"document":document}),
    )
}

fn change_history(store: &Store<'_>, input: History) -> Result<Value, AppError> {
    if !valid_asset_id(&input.id) {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Invalid governance asset ID.",
        ));
    }
    store.asset(&input.id, true)?;
    let limit = page_size(input.limit)?;
    let mut items = store.events(&input.id, input.before.unwrap_or(i64::MAX), limit + 1)?;
    let next_before = if items.len() > limit as usize {
        items.pop();
        items.last().map(|item| item.event_id)
    } else {
        None
    };
    Ok(json!({"items":items,"next_before":next_before}))
}

fn history(store: &Store<'_>, input: History) -> Result<Value, AppError> {
    if !valid_asset_id(&input.id) {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Invalid governance asset ID.",
        ));
    }
    store.asset(&input.id, true)?;
    let limit = page_size(input.limit)?;
    let mut items = store.history(&input.id, input.before.unwrap_or(i64::MAX), limit + 1)?;
    let next_before = if items.len() > limit as usize {
        items.pop();
        items.last().map(|item| item.revision)
    } else {
        None
    };
    Ok(json!({"items":items,"next_before":next_before}))
}

fn status(store: &Store<'_>, asset: &Asset, method: &str, at: i64) -> Result<String, AppError> {
    let archived = !method.ends_with(".restore");
    let deleted = method.ends_with(".delete");
    if deleted && !asset.archived {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Archive the asset before deleting its catalog entry.",
        ));
    }
    if archived && store.default_profile()?.as_deref() == Some(&asset.id) {
        return Err(AppError::new(
            "DEPENDENCY_IN_USE",
            "Clear the default profile before archiving it.",
        ));
    }
    if archived {
        let used_by = store.reverse(&asset.id)?;
        let bindings = store.used_bindings(&asset.id)?;
        if !used_by.is_empty() || !bindings.is_empty() {
            return Err(AppError::new(
                "DEPENDENCY_IN_USE",
                "Detach current consumers before removing the asset.",
            )
            .with_details(json!({"used_by":used_by,"assigned_bindings":bindings})));
        }
    }
    store.set_status(asset, archived, deleted, at)?;
    Ok(asset.id.clone())
}

fn assigned(store: &Store<'_>, input: Assigned) -> Result<Value, AppError> {
    let Some(binding) = input.binding_id else {
        return Ok(json!({"items":[],"next_after":null}));
    };
    let limit = page_size(input.limit)?;
    let after = cursor(input.after)?;
    let mut ids = store.assigned(&binding, &after, limit + 1)?;
    let next_after = if ids.len() > limit as usize {
        ids.pop();
        ids.last()
    } else {
        None
    };
    let items: Result<Vec<_>, _> = ids.iter().map(|id| find(store, id)).collect();
    Ok(json!({"items":items?,"next_after":next_after}))
}

fn effective(store: &Store<'_>, input: Effective) -> Result<Value, AppError> {
    let budget = input.budget_bytes.unwrap_or(32_768);
    if !(1..=220_000).contains(&budget) {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Set a preview budget between 1 and 220000 bytes.",
        ));
    }
    let mode = input
        .mode
        .unwrap_or_else(|| "WORK".into())
        .to_ascii_uppercase();
    if !matches!(
        mode.as_str(),
        "WORK" | "VERIFY" | "ACCEPT" | "RELEASE" | "AUDIT"
    ) {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Unknown governance preview mode.",
        ));
    }
    if let Some(binding) = &input.binding_id {
        store.binding(binding)?;
    }
    let bindings: Vec<_> = input.binding_id.as_deref().into_iter().collect();
    let default_profile = store.default_profile()?;
    let profile = input.profile_id.as_deref().or(default_profile.as_deref());
    resolve(store, profile, &bindings, &mode, budget)
}

fn resolve(
    store: &Store<'_>,
    profile: Option<&str>,
    bindings: &[&str],
    mode: &str,
    budget: usize,
) -> Result<Value, AppError> {
    let mut queue = VecDeque::new();
    if let Some(profile_id) = profile {
        let asset = find(store, profile_id)?;
        if asset.kind != Kind::Profile {
            return Err(AppError::new("INVALID_INPUT", "Select an agent profile."));
        }
        queue.push_back((profile_id.to_owned(), "selected_profile"));
    }
    for binding in bindings {
        let assignments = store.assigned(binding, "", 101)?;
        if assignments.len() > 100 {
            return Err(AppError::new(
                "DEPENDENCY_LIMIT",
                "The binding has too many assigned assets for one preview.",
            ));
        }
        queue.extend(assignments.into_iter().map(|id| (id, "binding_assignment")));
    }
    let mut visited = BTreeSet::new();
    let mut included = Vec::new();
    let mut excluded = Vec::new();
    let mut total = 0;
    let mut deferred = 0;
    while let Some((id, source)) = queue.pop_front() {
        if !visited.insert(id.clone()) {
            continue;
        }
        if visited.len() > 100 {
            return Err(AppError::new(
                "DEPENDENCY_LIMIT",
                "The preview dependency graph exceeds 100 assets.",
            ));
        }
        let asset = find(store, &id)?;
        let active = asset.active_revision.filter(|_| !asset.archived);
        let mut item = json!({"id":id,"title":asset.title,"kind":asset.kind,"revision":active,"source":source,"bytes":0});
        let Some(revision) = active else {
            item["reason"] = json!("inactive");
            excluded.push(item);
            continue;
        };
        let metadata = store.revision_metadata(&asset.id, revision)?;
        let bytes = metadata.bytes;
        item["title"] = json!(metadata.title);
        item["digest"] = json!(metadata.digest);
        item["bytes"] = json!(bytes);
        if matches!(asset.kind, Kind::Skill | Kind::Reference) {
            item["reason"] = json!("deferred");
            deferred += bytes;
        } else if total + bytes <= budget {
            item["reason"] = json!("within_budget");
            total += bytes;
        } else {
            item["reason"] = json!("budget_exceeded");
            excluded.push(item);
            continue;
        }
        included.push(item);
        queue.extend(
            store
                .active_relations(&asset.id)?
                .into_iter()
                .map(|id| (id, "dependency")),
        );
    }
    Ok(
        json!({"included":included,"excluded":excluded,"total_bytes":total,"budget_bytes":budget,
        "remaining_bytes":budget-total,"deferred_bytes":deferred,"hard_denies":HARD_DENIES,
        "language":"en","mode":mode,"profile_id":profile,"simulation":true,"runtime_enforcement":"unchanged"}),
    )
}

pub(super) fn pin_conversation(
    conn: &Connection,
    id: i64,
    profile: Option<&str>,
    bindings: &[String],
) -> Result<(), AppError> {
    let store = Store::from_connection(conn);
    let default = if profile.is_none() {
        store.default_profile()?
    } else {
        None
    };
    let profile = profile.or(default.as_deref());
    if let Some(profile_id) = profile {
        let asset = find(&store, profile_id)?;
        if asset.kind != Kind::Profile || asset.archived || asset.active_revision.is_none() {
            return Err(AppError::new(
                "INVALID_INPUT",
                "Choose an active governance profile.",
            ));
        }
    }
    let bindings: Vec<_> = bindings.iter().map(String::as_str).collect();
    let manifest = session_manifest(resolve(&store, profile, &bindings, "WORK", 32_768)?)?;
    conn.execute(
        "INSERT INTO governance_session_snapshots VALUES(?,?)",
        params![id, encode(&manifest)?],
    )?;
    Ok(())
}

pub(super) fn pin_run(
    conn: &Connection,
    id: i64,
    binding: &str,
    mode: &str,
) -> Result<(), AppError> {
    let store = Store::from_connection(conn);
    let manifest = session_manifest(resolve(&store, None, &[binding], mode, 32_768)?)?;
    conn.execute(
        "INSERT INTO governance_run_snapshots VALUES(?,?)",
        params![id, encode(&manifest)?],
    )?;
    Ok(())
}

fn session_manifest(mut resolved: Value) -> Result<Value, AppError> {
    let selected = resolved["included"].take();
    let excluded = resolved["excluded"].take();
    let profile = resolved["profile_id"].take();
    let mode = resolved["mode"].take();
    let manifest = json!({"version":1,"profile_id":profile,"assets":selected,
        "excluded":excluded,"mode":mode,"language":"en","execution":"handoff"});
    let digest = digest(&encode(&manifest)?);
    let mut manifest = manifest;
    manifest["digest"] = json!(digest);
    Ok(manifest)
}

pub(super) fn copy_run_snapshot(
    conn: &Connection,
    run: i64,
    conversation: i64,
) -> Result<(), AppError> {
    conn.execute(
        "INSERT INTO governance_session_snapshots (conversation_id,manifest_json)
         SELECT ?1,manifest_json FROM governance_run_snapshots WHERE run_id=?2",
        params![conversation, run],
    )?;
    Ok(())
}

pub(super) fn conversation_snapshot(
    conn: &Connection,
    conversation: i64,
) -> Result<Option<Value>, AppError> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT manifest_json FROM governance_session_snapshots WHERE conversation_id=?",
            [conversation],
            |row| row.get(0),
        )
        .optional()?;
    raw.map(|raw| {
        serde_json::from_str(&raw)
            .map_err(|_| AppError::new("INTEGRITY", "Stored governance manifest is invalid."))
    })
    .transpose()
}

fn export(store: &Store<'_>, input: Identity) -> Result<Value, AppError> {
    let asset = find(store, &input.id)?;
    let revision = asset
        .active_revision
        .ok_or_else(|| AppError::new("NOT_FOUND", "There is no published revision to export."))?;
    let document = store.revision(&asset.id, revision)?;
    if document.sensitive() {
        return Err(AppError::new(
            "SENSITIVE_INPUT",
            "Remove credentials before exporting this governance revision.",
        ));
    }
    let metadata = store.revision_metadata(&asset.id, revision)?;
    Ok(json!(Bundle {
        bundle_version: 1,
        kind: asset.kind,
        slug: metadata.slug,
        title: metadata.title,
        revision,
        digest: metadata.digest,
        source_name: asset.source_name,
        document,
    }))
}

fn checked_source_name(source: &str) -> Result<(), AppError> {
    checked_text(source, 160, false)?;
    if source.contains(['/', '\\', ':']) || super::sessions::sensitive_text(source) {
        return Err(AppError::new(
            "INVALID_PATH",
            "Import provenance takes a source filename, not a path.",
        ));
    }
    Ok(())
}

fn validate_bundle(bundle: &Bundle) -> Result<String, AppError> {
    if bundle.bundle_version != 1 || bundle.revision < 1 {
        return Err(AppError::new(
            "CAPABILITY_UNAVAILABLE",
            "This governance bundle version is unavailable.",
        ));
    }
    model::check_identity(&bundle.slug, &bundle.title)?;
    if let Some(source) = &bundle.source_name {
        checked_source_name(source)?;
    }
    bundle.document.check_bounds()?;
    if digest(&encode(&bundle.document)?) != bundle.digest {
        return Err(AppError::new(
            "INTEGRITY",
            "The document digest does not match the manifest.",
        ));
    }
    Ok(digest(&encode(bundle)?))
}

fn bundle_preview(store: &Store<'_>, bundle: &Bundle) -> Result<Value, AppError> {
    let fingerprint = validate_bundle(bundle)?;
    let missing: Vec<_> = bundle
        .document
        .dependencies
        .iter()
        .filter_map(|id| match find(store, id) {
            Ok(_) => None,
            Err(error) => Some((id, error)),
        })
        .map(|(id, error)| {
            if error.code == "NOT_FOUND" {
                Ok(id)
            } else {
                Err(error)
            }
        })
        .collect::<Result<_, _>>()?;
    Ok(
        json!({"fingerprint":fingerprint,"kind":bundle.kind,"slug":bundle.slug,
        "title":bundle.title,"source_name":bundle.source_name,"source_revision":bundle.revision,
        "digest":bundle.digest,"missing_dependencies":missing,"read_only_preview":true,
        "paths":bundle.document.files.iter().map(|file| &file.path).collect::<Vec<_>>(),
        "diagnostics":model::validate(bundle.kind,&bundle.document,None),"activates":false}),
    )
}

fn import_bundle(store: &Store<'_>, input: BundleApply, at: i64) -> Result<String, AppError> {
    if validate_bundle(&input.bundle)? != input.fingerprint {
        return Err(AppError::new(
            "CONFLICT",
            "The bundle changed. Preview the current bytes before importing.",
        ));
    }
    let Bundle {
        kind,
        slug,
        title,
        source_name,
        revision,
        digest,
        document,
        ..
    } = input.bundle;
    let slug = input.slug.as_deref().unwrap_or(&slug);
    model::check_identity(slug, &title)?;
    for dependency in &document.dependencies {
        find(store, dependency)?;
    }
    let id = store.insert(
        kind,
        slug,
        &title,
        Some(source_name.as_deref().unwrap_or("CIRNETRULES bundle")),
        &document,
        at,
    )?;
    store.store_initial_relations(&id, &document.dependencies)?;
    store.set_origin(&id, revision, &digest)?;
    Ok(id)
}

fn change<T: serde::de::DeserializeOwned>(
    db: &mut Database,
    method: &str,
    params: Value,
    operation: impl FnOnce(&Store<'_>, &Asset, T, i64) -> Result<String, AppError>,
) -> Result<Value, AppError> {
    let input: Change<T> = decode(params.clone())?;
    mutate(db, method, &params, &input.mutation_key, |store, at| {
        let asset = find(store, &input.id)?;
        expected(&asset, input.expected_revision)?;
        if asset.archived
            && !matches!(
                method,
                "governance.asset.delete" | "governance.asset.restore"
            )
        {
            return Err(AppError::new(
                "ARCHIVED",
                "Restore the asset before editing.",
            ));
        }
        operation(store, &asset, input.input, at)
    })
}

fn mutate(
    db: &mut Database,
    method: &str,
    params: &Value,
    key: &str,
    operation: impl FnOnce(&Store<'_>, i64) -> Result<String, AppError>,
) -> Result<Value, AppError> {
    if key.is_empty()
        || key.len() > 80
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
    {
        return Err(AppError::new(
            "INVALID_INPUT",
            "Invalid governance mutation key.",
        ));
    }
    let fingerprint = digest(&encode(&(method, params))?);
    Store::write(db, |store, at| {
        let asset = if let Some(previous) = store.replay(key, &fingerprint)? {
            previous
        } else {
            let id = operation(store, at)?;
            let current = store.asset(&id, true)?;
            store.record(key, &fingerprint, &current, at)?;
            current
        };
        if method == "governance.change.review" {
            let (token, digest) = store.review(&asset, at)?;
            return Ok(
                json!({"id":asset.id,"revision":asset.revision,"review_token":token,"digest":digest}),
            );
        }
        detail(store, &asset)
    })
}
