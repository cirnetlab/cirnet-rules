use super::{
    catalog,
    sessions::{self, Intake},
    storage::{AppError, Database},
};
use chrono::{Days, LocalResult, TimeZone, Utc};
use chrono_tz::Tz;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
enum Trigger {
    Manual,
    Daily {
        hour: u32,
        minute: u32,
        timezone: String,
    },
}

impl Trigger {
    fn next(&self, after: i64) -> Result<Option<i64>, AppError> {
        let Self::Daily {
            hour,
            minute,
            timezone,
        } = self
        else {
            return Ok(None);
        };
        let zone = timezone
            .parse::<Tz>()
            .map_err(|_| invalid("Select an IANA timezone"))?;
        let after = Utc
            .timestamp_opt(after, 0)
            .single()
            .ok_or_else(|| invalid("Unsupported schedule time"))?;
        let day = after.with_timezone(&zone).date_naive();
        for offset in 0..=370 {
            let local = day
                .checked_add_days(Days::new(offset))
                .and_then(|day| day.and_hms_opt(*hour, *minute, 0))
                .ok_or_else(|| invalid("Invalid scheduled time"))?;
            let candidate = match zone.from_local_datetime(&local) {
                LocalResult::None => continue, // Skip the nonexistent wall time.
                LocalResult::Single(time) => time,
                LocalResult::Ambiguous(a, b) => a.min(b), // One occurrence on a repeated day.
            };
            if candidate.timestamp() > after.timestamp() {
                return Ok(Some(candidate.timestamp()));
            }
        }
        Err(invalid("No schedulable occurrence in the next year"))
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Definition {
    name: String,
    binding_id: String,
    prompt: String,
    #[serde(default = "sessions::handoff")]
    profile: String,
    trigger: Trigger,
    #[serde(default)]
    enabled: bool,
    template_id: Option<String>,
    #[serde(default)]
    read_paths: Vec<String>,
}

#[derive(Deserialize)]
struct Create {
    key: String,
    #[serde(flatten)]
    definition: Definition,
}
#[derive(Deserialize)]
struct Update {
    id: String,
    revision: i64,
    #[serde(flatten)]
    definition: Definition,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Pause {
    id: String,
    revision: i64,
    enabled: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Run {
    id: String,
    key: String,
    #[serde(default)]
    confirm_inactive: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Preview {
    trigger: Trigger,
}

#[derive(Clone, Serialize)]
struct Template {
    id: &'static str,
    title: &'static str,
    description: &'static str,
    prompt: &'static str,
    mode: &'static str,
    revision: i64,
    status: &'static str,
    risk: &'static str,
    permissions: &'static [&'static str],
    verification_profile: &'static str,
}

fn templates() -> Vec<Template> {
    [
        (
            "scoped-review",
            "Scoped code review",
            "Review specific files and identify test gaps.",
            "Review the paths in the approved scope. Report risks and test gaps.",
            "WORK",
            "SCOPED",
        ),
        (
            "test-triage",
            "Test failure triage",
            "Use selected test evidence to propose a diagnosis.",
            "Inspect selected failure evidence and propose the smallest verification.",
            "WORK",
            "SCOPED",
        ),
        (
            "ui-change",
            "Scoped UI change",
            "Propose a limited UI change and tests.",
            "Propose a UI-only change. Flag API, auth, Run or Stop boundaries for review.",
            "WORK",
            "SCOPED",
        ),
        (
            "docs-preview",
            "Docs-sync preview",
            "Prepare an accepted behavior documentation proposal.",
            "Propose a documentation patch based on accepted evidence. Do not apply it.",
            "WORK",
            "SCOPED",
        ),
        (
            "security-review",
            "Read-only security review",
            "Prepare a security review for approved inputs.",
            "Assess security in the declared read scope. Escalate findings for human review.",
            "WORK",
            "REVIEW_REQUIRED",
        ),
        (
            "release-readiness",
            "Release readiness",
            "Prepare an explicit release-readiness request.",
            "Propose a release-readiness scope for the selected repository. Await approval.",
            "RELEASE",
            "REVIEW_REQUIRED",
        ),
    ]
    .into_iter()
    .map(|(id, title, description, prompt, mode, risk)| Template {
        id,
        title,
        description,
        prompt,
        mode,
        revision: 1,
        status: "UNVERIFIED",
        risk,
        permissions: &["PROPOSE"],
        verification_profile: "human-review-required",
    })
    .collect()
}

fn template(id: Option<&str>) -> Result<Option<Template>, AppError> {
    id.map(|id| {
        templates()
            .into_iter()
            .find(|row| row.id == id)
            .ok_or_else(|| invalid("Unknown workflow template"))
    })
    .transpose()
}
fn invalid(message: &'static str) -> AppError {
    AppError::new("INVALID_REQUEST", message)
}
fn missing() -> AppError {
    AppError::new("NOT_FOUND", "Automation record does not exist")
}
fn conflict(message: &'static str) -> AppError {
    AppError::new("CONFLICT", message)
}
fn decode<T: serde::de::DeserializeOwned>(value: &str) -> Result<T, AppError> {
    serde_json::from_str(value)
        .map_err(|_| AppError::new("INTEGRITY", "Stored automation data is invalid"))
}
fn encode<T: Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_string(value)
        .map_err(|_| AppError::new("INTEGRITY", "Could not encode automation data"))
}

#[derive(Clone, Deserialize, Serialize)]
struct Snapshot {
    name: String,
    prompt: String,
    #[serde(default)]
    read_paths: Vec<String>,
    profile: String,
    trigger: Trigger,
    template_id: Option<String>,
    template_revision: Option<i64>,
    mode: String,
    risk: String,
}

impl Snapshot {
    fn new(definition: &Definition) -> Result<Self, AppError> {
        let name = sessions::text(&definition.name, 160)?.to_owned();
        let prompt = sessions::text(&definition.prompt, 16_384)?.to_owned();
        sessions::profile(&definition.profile)?;
        if definition.read_paths.len() > 32
            || definition
                .read_paths
                .iter()
                .any(|path| !sessions::safe_path(path))
            || definition.read_paths.iter().collect::<BTreeSet<_>>().len()
                != definition.read_paths.len()
        {
            return Err(invalid("Use up to 32 unique safe relative read hints"));
        }
        let template = template(definition.template_id.as_deref())?;
        let mode = template.as_ref().map_or("WORK", |row| row.mode);
        if mode == "RELEASE" && !matches!(&definition.trigger, Trigger::Manual) {
            return Err(invalid(
                "Release-readiness proposals require an explicit manual run",
            ));
        }
        Ok(Self {
            name,
            prompt,
            read_paths: definition.read_paths.clone(),
            profile: definition.profile.clone(),
            trigger: definition.trigger.clone(),
            template_id: definition.template_id.clone(),
            template_revision: template.as_ref().map(|row| row.revision),
            mode: mode.into(),
            risk: template.as_ref().map_or("SCOPED", |row| row.risk).into(),
        })
    }
}

#[derive(Debug)]
struct Row {
    id: i64,
    name: String,
    binding: String,
    binding_revision: i64,
    profile: String,
    prompt: String,
    paths: String,
    template: Option<String>,
    template_revision: Option<i64>,
    revision: i64,
    enabled: bool,
    trigger: String,
    next_at: Option<i64>,
    mode: String,
    risk: String,
}
const DEF_COLS: &str = "id,name,binding_id,binding_revision,profile,prompt,paths_json,template_id,template_revision,revision,enabled,trigger_json,next_at,mode,risk";
fn row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Row> {
    Ok(Row {
        id: row.get(0)?,
        name: row.get(1)?,
        binding: row.get(2)?,
        binding_revision: row.get(3)?,
        profile: row.get(4)?,
        prompt: row.get(5)?,
        paths: row.get(6)?,
        template: row.get(7)?,
        template_revision: row.get(8)?,
        revision: row.get(9)?,
        enabled: row.get(10)?,
        trigger: row.get(11)?,
        next_at: row.get(12)?,
        mode: row.get(13)?,
        risk: row.get(14)?,
    })
}

fn load(conn: &Connection, id: i64) -> Result<Row, AppError> {
    conn.query_row(
        &format!("SELECT {DEF_COLS} FROM automation_definitions WHERE id=?1"),
        [id],
        row,
    )
    .optional()?
    .ok_or_else(missing)
}

fn snapshot(def: &Row) -> Result<Snapshot, AppError> {
    // Template semantics are deliberately fixed when a definition is saved.
    // Profile is handoff-only. A newer template never broadens it automatically.
    Ok(Snapshot {
        name: def.name.clone(),
        prompt: def.prompt.clone(),
        read_paths: decode(&def.paths)?,
        profile: def.profile.clone(),
        trigger: decode(&def.trigger)?,
        template_id: def.template.clone(),
        template_revision: def.template_revision,
        mode: def.mode.clone(),
        risk: def.risk.clone(),
    })
}

const RUN_COLS: &str = "id,automation_id,definition_revision,binding_id,binding_revision,kind,status,occurrence_at,attempt,conversation_id,task_id,outcome,created_at,updated_at";
fn run_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Value> {
    Ok(
        json!({"id":row.get::<_,i64>(0)?.to_string(),"automation_id":row.get::<_,i64>(1)?.to_string(),
        "definition_revision":row.get::<_,i64>(2)?,"binding_id":row.get::<_,String>(3)?,"binding_revision":row.get::<_,i64>(4)?,
        "kind":row.get::<_,String>(5)?,"status":row.get::<_,String>(6)?,"occurrence_at":row.get::<_,Option<i64>>(7)?,
        "attempt":row.get::<_,i64>(8)?,"conversation_id":row.get::<_,Option<i64>>(9)?.map(|n|n.to_string()),
        "task_id":row.get::<_,Option<i64>>(10)?.map(|n|n.to_string()),"outcome":row.get::<_,Option<String>>(11)?,
        "created_at":row.get::<_,i64>(12)?,"updated_at":row.get::<_,i64>(13)?}),
    )
}
fn run_view(conn: &Connection, id: i64) -> Result<Value, AppError> {
    conn.query_row(
        &format!("SELECT {RUN_COLS} FROM automation_runs WHERE id=?1"),
        [id],
        run_row,
    )
    .optional()?
    .ok_or_else(missing)
}
fn view(conn: &Connection, def: Row) -> Result<Value, AppError> {
    let last=conn.query_row(&format!("SELECT {RUN_COLS} FROM automation_runs WHERE automation_id=?1 ORDER BY id DESC LIMIT 1"),[def.id],run_row).optional()?;
    Ok(
        json!({"id":def.id.to_string(),"name":def.name,"binding_id":def.binding,"binding_revision":def.binding_revision,
        "profile":def.profile,"prompt":def.prompt,"read_paths":decode::<Value>(&def.paths)?,"template_id":def.template,
        "template_revision":def.template_revision,"revision":def.revision,"enabled":def.enabled,"trigger":decode::<Value>(&def.trigger)?,
        "next_at":def.next_at,"last_run":last,"misfire":"COALESCE_ONE","overlap":"SKIP_PENDING","retry":"NONE","grant":"PROPOSAL_ONLY"}),
    )
}

fn prior_create(conn: &Connection, key: &str, hash: &str) -> Result<Option<Value>, AppError> {
    let found = conn
        .query_row(
            "SELECT id,create_digest FROM automation_definitions WHERE create_key=?1",
            [key],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
        )
        .optional()?;
    match found {
        None => Ok(None),
        Some((id, previous)) if previous == hash => Ok(Some(view(conn, load(conn, id)?)?)),
        Some(_) => Err(conflict(
            "Idempotency key has already been used for another automation",
        )),
    }
}

fn create(db: &mut Database, input: Create) -> Result<Value, AppError> {
    sessions::request_key(&input.key)?;
    let snapshot = Snapshot::new(&input.definition)?;
    let hash = sessions::digest(&json!([
        snapshot,
        input.definition.binding_id,
        input.definition.enabled
    ]));
    if let Some(found) = prior_create(&db.conn, &input.key, &hash)? {
        return Ok(found);
    }
    let binding = catalog::resolve_binding(db, &input.definition.binding_id, None)?;
    let time = sessions::now();
    let next = snapshot.trigger.next(time)?;
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(found) = prior_create(&tx, &input.key, &hash)? {
        return Ok(found);
    }
    sessions::matches_binding(&tx, &binding)?;
    tx.execute("INSERT INTO automation_definitions(create_key,create_digest,name,binding_id,binding_revision,profile,prompt,paths_json,
        template_id,template_revision,mode,risk,revision,enabled,trigger_json,next_at,created_at,updated_at)
        VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,1,?13,?14,?15,?16,?16)",
        params![input.key,hash,snapshot.name,binding.id,binding.revision,snapshot.profile,snapshot.prompt,encode(&snapshot.read_paths)?,
            snapshot.template_id,snapshot.template_revision,snapshot.mode,snapshot.risk,input.definition.enabled,encode(&snapshot.trigger)?,next.filter(|_|input.definition.enabled),time])?;
    let result = view(&tx, load(&tx, tx.last_insert_rowid())?)?;
    tx.commit()?;
    Ok(result)
}

fn retire_pending(conn: &Connection, id: i64, time: i64, outcome: &str) -> Result<(), AppError> {
    let mut statement = conn.prepare(
        "UPDATE automation_runs SET status='CANCELLED',outcome=?2,updated_at=?3
        WHERE automation_id=?1 AND status='QUEUED' RETURNING id",
    )?;
    let retired = statement
        .query_map(params![id, outcome, time], |row| row.get::<_, i64>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(statement);
    for run in retired {
        sessions::event(
            conn,
            (None, None, Some(run), None),
            &format!("run-{run}"),
            "RUN_CANCELLED",
            &json!({"outcome":outcome}),
            time,
        )?;
    }
    Ok(())
}

fn update(db: &mut Database, input: Update) -> Result<Value, AppError> {
    let id = sessions::id(&input.id)?;
    let snapshot = Snapshot::new(&input.definition)?;
    let binding = catalog::resolve_binding(db, &input.definition.binding_id, None)?;
    let time = sessions::now();
    let next = snapshot.trigger.next(time)?;
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    sessions::matches_binding(&tx, &binding)?;
    if tx.execute("UPDATE automation_definitions SET name=?3,binding_id=?4,binding_revision=?5,profile=?6,prompt=?7,paths_json=?8,
        template_id=?9,template_revision=?10,mode=?11,risk=?12,enabled=?13,trigger_json=?14,next_at=?15,updated_at=?16,revision=revision+1 WHERE id=?1 AND revision=?2",
        params![id,input.revision,snapshot.name,binding.id,binding.revision,snapshot.profile,snapshot.prompt,encode(&snapshot.read_paths)?,
            snapshot.template_id,snapshot.template_revision,snapshot.mode,snapshot.risk,input.definition.enabled,encode(&snapshot.trigger)?,next.filter(|_|input.definition.enabled),time])?!=1{
        return Err(conflict("Automation revision is stale"));
    }
    retire_pending(&tx, id, time, "DEFINITION_CHANGED")?;
    let result = view(&tx, load(&tx, id)?)?;
    tx.commit()?;
    Ok(result)
}

fn pause(db: &mut Database, input: Pause) -> Result<Value, AppError> {
    let id = sessions::id(&input.id)?;
    let def = load(&db.conn, id)?;
    if def.revision != input.revision {
        return Err(conflict("Automation revision is stale"));
    }
    let time = sessions::now();
    let binding = if input.enabled {
        Some(catalog::resolve_binding(
            db,
            &def.binding,
            Some(def.binding_revision),
        )?)
    } else {
        None
    };
    let next = if input.enabled {
        decode::<Trigger>(&def.trigger)?.next(time)?
    } else {
        None
    };
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(binding) = binding.as_ref() {
        sessions::matches_binding(&tx, binding)?;
    }
    if tx.execute("UPDATE automation_definitions SET enabled=?3,next_at=?4,updated_at=?5,revision=revision+1 WHERE id=?1 AND revision=?2",
        params![id,input.revision,input.enabled,next,time])?!=1{return Err(conflict("Automation revision is stale"));}
    if !input.enabled {
        retire_pending(&tx, id, time, "DEFINITION_DISABLED")?;
    }
    let result = view(&tx, load(&tx, id)?)?;
    tx.commit()?;
    Ok(result)
}

fn prior_run(conn: &Connection, automation: i64, key: &str) -> Result<Option<Value>, AppError> {
    Ok(conn
        .query_row(
            &format!(
                "SELECT {RUN_COLS} FROM automation_runs WHERE automation_id=?1 AND request_key=?2"
            ),
            params![automation, key],
            run_row,
        )
        .optional()?)
}

fn active_run(conn: &Connection, id: i64) -> Result<bool, AppError> {
    Ok(conn.query_row("SELECT EXISTS(SELECT 1 FROM automation_runs WHERE automation_id=?1 AND status IN ('QUEUED','CLAIMED','WAITING_APPROVAL','RECOVERY_REQUIRED'))",
        [id],|row|row.get(0))?)
}

fn enqueue(
    conn: &Connection,
    def: &Row,
    kind: &str,
    key: Option<&str>,
    occurrence: Option<i64>,
    time: i64,
) -> Result<Option<i64>, AppError> {
    let written=conn.execute("INSERT INTO automation_runs(automation_id,definition_revision,binding_id,binding_revision,kind,request_key,
        occurrence_at,status,snapshot_json,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,'QUEUED',?8,?9,?9)
        ON CONFLICT(automation_id,occurrence_at) DO NOTHING",params![def.id,def.revision,def.binding,def.binding_revision,kind,key,occurrence,encode(&snapshot(def)?)?,time])?;
    if written == 0 {
        return Ok(None);
    }
    let id = conn.last_insert_rowid();
    let correlation = format!("run-{id}");
    sessions::event(
        conn,
        (None, None, Some(id), None),
        &correlation,
        "RUN_QUEUED",
        &json!({"kind":kind}),
        time,
    )?;
    Ok(Some(id))
}

fn manual(db: &mut Database, input: Run) -> Result<Value, AppError> {
    let id = sessions::id(&input.id)?;
    sessions::request_key(&input.key)?;
    if let Some(found) = prior_run(&db.conn, id, &input.key)? {
        return Ok(found);
    }
    let def = load(&db.conn, id)?;
    if !def.enabled && !input.confirm_inactive {
        return Err(AppError::new(
            "CONFIRMATION_REQUIRED",
            "Confirm a one-shot run of the inactive automation",
        ));
    }
    let binding = catalog::resolve_binding(db, &def.binding, Some(def.binding_revision))?;
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    if let Some(found) = prior_run(&tx, id, &input.key)? {
        return Ok(found);
    }
    if load(&tx, id)?.revision != def.revision {
        return Err(conflict("Automation changed before enqueue"));
    }
    sessions::matches_binding(&tx, &binding)?;
    if active_run(&tx, id)? {
        return Err(conflict(
            "An existing run must be resolved before another one starts",
        ));
    }
    let run = enqueue(&tx, &def, "MANUAL", Some(&input.key), None, sessions::now())?
        .ok_or_else(|| conflict("Manual run was not enqueued"))?;
    let result = run_view(&tx, run)?;
    tx.commit()?;
    Ok(result)
}

fn cancel(db: &mut Database, params: &Value) -> Result<Value, AppError> {
    let id = sessions::id(params["run_id"].as_str().unwrap_or(""))?;
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    let (conversation, task, status): (Option<i64>, Option<i64>, String) = tx
        .query_row(
            "SELECT conversation_id,task_id,status FROM automation_runs WHERE id=?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or_else(missing)?;
    if !matches!(
        status.as_str(),
        "QUEUED" | "CLAIMED" | "WAITING_APPROVAL" | "RECOVERY_REQUIRED"
    ) {
        return Err(conflict(
            "This run cannot be cancelled in its current state",
        ));
    }
    if let Some(task) = task {
        tx.execute(
            "UPDATE task_proposals SET state='CANCELLED' WHERE id=?1 AND state='WAITING_APPROVAL'",
            [task],
        )?;
    }
    let time = sessions::now();
    // CLAIMED is metadata inspection only. The worker cannot persist a
    // proposal after this transaction revokes the claim.
    tx.execute(
        "UPDATE automation_runs SET status='CANCELLED',outcome='OPERATOR_CANCELLED',updated_at=?2,
        lease_token=NULL,lease_expires_at=NULL WHERE id=?1",
        params![id, time],
    )?;
    sessions::event(
        &tx,
        (conversation, task, Some(id), None),
        &format!("run-{id}"),
        "RUN_CANCELLED",
        &json!({}),
        time,
    )?;
    let result = run_view(&tx, id)?;
    tx.commit()?;
    Ok(result)
}

fn list(db: &Database, params: &Value) -> Result<Value, AppError> {
    let query = params["query"].as_str().unwrap_or("").trim();
    if query.len() > 160 {
        return Err(invalid("Search is too long"));
    }
    let binding = params["binding_id"].as_str();
    let enabled = params["enabled"].as_bool();
    let sql=format!("SELECT {DEF_COLS} FROM automation_definitions WHERE id<?1 AND instr(lower(name),lower(?2))>0
        AND (?3 IS NULL OR binding_id=?3) AND (?4 IS NULL OR enabled=?4) ORDER BY id DESC LIMIT ?5");
    let mut statement = db.conn.prepare(&sql)?;
    let rows = statement
        .query_map(
            params![
                sessions::cursor(&params["before"], i64::MAX)?,
                query,
                binding,
                enabled,
                sessions::limit(&params["limit"])?.min(4)
            ],
            row,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let items = rows
        .into_iter()
        .map(|def| view(&db.conn, def))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({"next_before":items.last().map(|row|&row["id"]),"items":items}))
}

fn runs(db: &Database, params: &Value) -> Result<Value, AppError> {
    let id = sessions::id(params["id"].as_str().unwrap_or(""))?;
    load(&db.conn, id)?;
    let mut statement=db.conn.prepare(&format!("SELECT {RUN_COLS} FROM automation_runs WHERE automation_id=?1 AND id<?2 ORDER BY id DESC LIMIT ?3"))?;
    let items = statement
        .query_map(
            params![
                id,
                sessions::cursor(&params["before"], i64::MAX)?,
                sessions::limit(&params["limit"])?
            ],
            run_row,
        )?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(json!({"next_before":items.last().map(|row|&row["id"]),"items":items}))
}

fn preview(request: Preview) -> Result<Value, AppError> {
    let mut after = sessions::now();
    let mut occurrences = Vec::new();
    for _ in 0..4 {
        let Some(next) = request.trigger.next(after)? else {
            break;
        };
        occurrences.push(next);
        after = next;
    }
    Ok(
        json!({"next_at":occurrences.first(),"occurrences":occurrences,"misfire":"COALESCE_ONE","dst":"skip_missing_choose_earlier_repeated"}),
    )
}

pub fn dispatch(db: &mut Database, method: &str, params: Value) -> Result<Value, AppError> {
    match method {
        "automation.create" => create(db, sessions::parse(params)?),
        "automation.update" => update(db, sessions::parse(params)?),
        "automation.pause" => pause(db, sessions::parse(params)?),
        "automation.run" => manual(db, sessions::parse(params)?),
        "automation.cancel" => cancel(db, &params),
        "automation.list" => list(db, &params),
        "automation.runs" => runs(db, &params),
        "automation.events" => {
            let id = sessions::id(params["run_id"].as_str().unwrap_or(""))?;
            run_view(&db.conn, id)?;
            sessions::replay(&db.conn, None, Some(id), &params)
        }
        "automation.templates" => Ok(json!({"items":templates()})),
        "automation.preview" => preview(sessions::parse(params)?),
        _ => Err(AppError::new(
            "METHOD_NOT_FOUND",
            "Unsupported automation operation",
        )),
    }
}

fn schedule(db: &mut Database, time: i64) -> Result<usize, AppError> {
    if !db.conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM automation_definitions WHERE enabled=1 AND next_at<=?1)",
        [time],
        |row| row.get::<_, bool>(0),
    )? {
        return Ok(0);
    }
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    let mut statement=tx.prepare(&format!("SELECT {DEF_COLS} FROM automation_definitions WHERE enabled=1 AND next_at<=?1 ORDER BY next_at,id LIMIT 8"))?;
    let defs = statement
        .query_map([time], row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(statement);
    let mut count = 0;
    for def in defs {
        let next = decode::<Trigger>(&def.trigger)?.next(time)?;
        tx.execute(
            "UPDATE automation_definitions SET next_at=?2 WHERE id=?1",
            params![def.id, next],
        )?;
        if !active_run(&tx, def.id)?
            && enqueue(&tx, &def, "SCHEDULE", None, def.next_at, time)?.is_some()
        {
            count += 1;
        }
    }
    tx.commit()?;
    Ok(count)
}

struct Claim {
    id: i64,
    definition: i64,
    revision: i64,
    binding: String,
    binding_revision: i64,
    kind: String,
    snapshot: Snapshot,
    attempt: i64,
    token: String,
}

fn claim(db: &mut Database, time: i64) -> Result<Option<Claim>, AppError> {
    if !db.conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM automation_runs WHERE status='QUEUED' OR (status='CLAIMED' AND lease_expires_at<=?1))",
        [time],
        |row|row.get::<_,bool>(0),
    )? {
        return Ok(None);
    }
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    // Expired ownership is not permission to retry. This runtime can only
    // produce proposals, so it never automatically replays abandoned runs.
    let mut statement = tx.prepare("UPDATE automation_runs SET status='RECOVERY_REQUIRED',outcome='CLAIM_EXPIRED',updated_at=?1
        WHERE id IN (SELECT id FROM automation_runs WHERE status='CLAIMED' AND lease_expires_at<=?1
        ORDER BY lease_expires_at,id LIMIT 16) RETURNING id,attempt")?;
    let expired = statement
        .query_map([time], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(statement);
    for (run, attempt) in expired {
        sessions::event(
            &tx,
            (None, None, Some(run), Some(attempt)),
            &format!("run-{run}"),
            "RUN_RECOVERY_REQUIRED",
            &json!({"outcome":"CLAIM_EXPIRED"}),
            time,
        )?;
    }
    let found=tx.query_row("SELECT id,automation_id,definition_revision,binding_id,binding_revision,kind,snapshot_json,attempt FROM automation_runs
        WHERE status='QUEUED' ORDER BY id LIMIT 1",[],|row|Ok((row.get::<_,i64>(0)?,row.get::<_,i64>(1)?,row.get::<_,i64>(2)?,
        row.get::<_,String>(3)?,row.get::<_,i64>(4)?,row.get::<_,String>(5)?,row.get::<_,String>(6)?,row.get::<_,i64>(7)?))).optional()?;
    let Some((id, definition, revision, binding, binding_revision, kind, snapshot, attempt)) =
        found
    else {
        tx.commit()?;
        return Ok(None);
    };
    let snapshot = decode(&snapshot)?;
    let attempt = attempt
        .checked_add(1)
        .ok_or_else(|| AppError::new("INTEGRITY", "Run attempt overflow"))?;
    let token = format!("{id}-{attempt}-{}", std::process::id());
    if tx.execute("UPDATE automation_runs SET status='CLAIMED',lease_token=?2,lease_expires_at=?3,attempt=?4,updated_at=?5
        WHERE id=?1 AND status='QUEUED'",params![id,token,time.saturating_add(60),attempt,time])?!=1{return Err(conflict("Run was claimed by another worker"));}
    sessions::event(
        &tx,
        (None, None, Some(id), Some(attempt)),
        &format!("run-{id}"),
        "RUN_CLAIMED",
        &json!({}),
        time,
    )?;
    tx.commit()?;
    Ok(Some(Claim {
        id,
        definition,
        revision,
        binding,
        binding_revision,
        kind,
        snapshot,
        attempt,
        token,
    }))
}

fn finish(db: &mut Database, claim: Claim, time: i64) -> Result<(), AppError> {
    let binding = catalog::resolve_binding(db, &claim.binding, Some(claim.binding_revision));
    let prepared = binding.as_ref().ok().map(|binding| {
        let scope = BTreeMap::from([(binding.id.clone(), claim.snapshot.read_paths.clone())]);
        sessions::route(std::slice::from_ref(binding), &scope)
    });
    let time = time.max(sessions::now());
    let tx = db
        .conn
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    let owned:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM automation_runs WHERE id=?1 AND status='CLAIMED' AND lease_token=?2 AND lease_expires_at>?3)",
        params![claim.id,claim.token,time],|row|row.get(0))?;
    if !owned {
        return Ok(());
    }
    let def = load(&tx, claim.definition)?;
    let metadata_allowed =
        def.revision == claim.revision && (claim.kind == "MANUAL" || def.enabled);
    let mut intake = None;
    let mut outcome = if metadata_allowed {
        "BINDING_INVALID"
    } else {
        "DEFINITION_CHANGED"
    };
    if metadata_allowed {
        match binding {
            Ok(ref binding) if sessions::matches_binding(&tx, binding).is_ok() => match prepared {
                Some(Ok((route, risk))) => {
                    let risk = if claim.snapshot.risk == "REVIEW_REQUIRED" {
                        "REVIEW_REQUIRED"
                    } else {
                        risk
                    };
                    let proposal = Intake {
                        content: &claim.snapshot.prompt,
                        route: &route,
                        mode: &claim.snapshot.mode,
                        risk,
                        time,
                    };
                    intake = Some(sessions::from_automation(
                        &tx,
                        (claim.id, claim.attempt),
                        &claim.snapshot.name,
                        binding,
                        &proposal,
                    )?);
                    outcome = "APPROVAL_AND_EXECUTION_BACKEND_REQUIRED";
                }
                _ => outcome = "INVALID_SCOPE",
            },
            Err(ref error) => outcome = error.code,
            _ => {}
        }
    }
    let status = if intake.is_some() {
        "WAITING_APPROVAL"
    } else {
        "BLOCKED"
    };
    tx.execute("UPDATE automation_runs SET status=?3,outcome=?4,conversation_id=?5,task_id=?6,updated_at=?7,lease_token=NULL,lease_expires_at=NULL
        WHERE id=?1 AND lease_token=?2",params![claim.id,claim.token,status,outcome,intake.map(|item|item.0),intake.map(|item|item.1),time])?;
    sessions::event(
        &tx,
        (
            intake.map(|v| v.0),
            intake.map(|v| v.1),
            Some(claim.id),
            Some(claim.attempt),
        ),
        &format!("run-{}", claim.id),
        "RUN_OUTCOME",
        &json!({"status":status,"outcome":outcome}),
        time,
    )?;
    tx.commit()?;
    Ok(())
}

pub fn tick(db: &mut Database) -> Result<usize, AppError> {
    let mut count = schedule(db, sessions::now())?;
    for _ in 0..4 {
        let Some(run) = claim(db, sessions::now())? else {
            break;
        };
        finish(db, run, sessions::now())?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        root: std::path::PathBuf,
        data: std::path::PathBuf,
    }
    impl Fixture {
        fn new() -> (Self, Database, String) {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root =
                std::env::temp_dir().join(format!("cgacp-session-{}-{nonce}", std::process::id()));
            std::fs::create_dir(&root).unwrap();
            let checkout = root.join("đồ án bất kỳ");
            std::fs::create_dir(&checkout).unwrap();
            fn git(repo: &std::path::Path, args: &[&str]) {
                let output = std::process::Command::new("git")
                    .current_dir(repo)
                    .env_remove("GIT_DIR")
                    .env_remove("GIT_WORK_TREE")
                    .args(args)
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            git(&checkout, &["init", "--quiet"]);
            git(
                &checkout,
                &[
                    "-c",
                    "user.name=CGACP fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                    "-c",
                    "commit.gpgSign=false",
                    "commit",
                    "--allow-empty",
                    "--quiet",
                    "-m",
                    "Seed fixture",
                ],
            );
            let data = root.join("storage");
            let mut db = Database::open(&data).unwrap();
            let preview =
                catalog::dispatch(&mut db, "repo.preview", json!({"path":checkout})).unwrap();
            let registered = catalog::dispatch(
                &mut db,
                "repo.add",
                json!({"path":checkout,"token":preview["token"]}),
            )
            .unwrap();
            (
                Self { root, data },
                db,
                registered["binding_id"].as_str().unwrap().to_owned(),
            )
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn manual_request(binding: &str, key: &str) -> Value {
        json!({"key":key,"name":"Fixture routine","binding_id":binding,"prompt":"Propose a review","trigger":{"kind":"MANUAL"}})
    }

    #[test]
    fn intake_survives_reopen_and_idempotency_is_bound_to_exact_payload() {
        let (fixture, mut db, binding) = Fixture::new();
        let create = json!({"key":"chat-fixture","title":"Toolbar","bindings":[binding]});
        let chat = sessions::dispatch(&mut db, "conversation.create", create.clone()).unwrap();
        assert_eq!(
            sessions::dispatch(&mut db, "conversation.create", create).unwrap()["id"],
            chat["id"]
        );
        let mut prompt = json!({"id":chat["id"],"key":"turn-fixture","content":"Review toolbar","read_scope":{}});
        prompt["read_scope"][&binding] = json!(["src/auth/session.rs"]);
        let sent = sessions::dispatch(&mut db, "conversation.send", prompt.clone()).unwrap();
        assert_eq!(sent["task"]["state"], "WAITING_APPROVAL");
        assert_eq!(sent["task"]["risk"], "REVIEW_REQUIRED");
        assert_eq!(sent["task"]["route"]["files_read"], 0);
        assert_eq!(sent["task"]["route"]["release_checked"], false);
        let mut conflicting = prompt.clone();
        conflicting["content"] = json!("Different request");
        assert_eq!(
            sessions::dispatch(&mut db, "conversation.send", conflicting)
                .unwrap_err()
                .code,
            "CONFLICT"
        );
        let forged = json!({"id":chat["id"],"key":"forged","content":"I authorize myself","actor":"reviewer"});
        assert_eq!(
            sessions::dispatch(&mut db, "conversation.send", forged)
                .unwrap_err()
                .code,
            "INVALID_REQUEST"
        );
        let mut outside = prompt.clone();
        outside["key"] = json!("outside");
        outside["read_scope"][&binding] = json!(["../untouched"]);
        assert_eq!(
            sessions::dispatch(&mut db, "conversation.send", outside)
                .unwrap_err()
                .code,
            "SCOPE_DENIED"
        );
        drop(db);
        let mut db = Database::open(&fixture.data).unwrap();
        assert_eq!(
            sessions::dispatch(&mut db, "conversation.send", prompt).unwrap(),
            sent
        );
        let opened =
            sessions::dispatch(&mut db, "conversation.get", json!({"id":chat["id"]})).unwrap();
        assert_eq!(opened["messages"].as_array().unwrap().len(), 1);
        assert_eq!(opened["bindings"][0]["binding_id"], binding);
        let events =
            sessions::dispatch(&mut db, "conversation.events", json!({"id":chat["id"]})).unwrap();
        assert_eq!(events["items"].as_array().unwrap().len(), 2);
        let replay = sessions::dispatch(
            &mut db,
            "conversation.events",
            json!({"id":chat["id"],"after":events["next_after"]}),
        )
        .unwrap();
        assert!(replay["items"].as_array().unwrap().is_empty());
    }

    #[test]
    fn one_shot_waits_for_approval_and_relink_blocks_old_runs() {
        let (_fixture, mut db, binding) = Fixture::new();
        let def = dispatch(
            &mut db,
            "automation.create",
            manual_request(&binding, "one-shot"),
        )
        .unwrap();
        let key = json!({"id":def["id"],"key":"button"});
        assert_eq!(
            dispatch(&mut db, "automation.run", key.clone())
                .unwrap_err()
                .code,
            "CONFIRMATION_REQUIRED"
        );
        let confirmed = json!({"id":def["id"],"key":"button","confirm_inactive":true});
        let queued = dispatch(&mut db, "automation.run", confirmed.clone()).unwrap();
        assert_eq!(
            dispatch(&mut db, "automation.run", confirmed).unwrap()["id"],
            queued["id"]
        );
        assert_eq!(tick(&mut db).unwrap(), 1);
        let run = dispatch(&mut db, "automation.runs", json!({"id":def["id"]})).unwrap()["items"]
            [0]
        .clone();
        assert_eq!(run["status"], "WAITING_APPROVAL");
        assert_eq!(run["attempt"], 1);
        let events = dispatch(&mut db, "automation.events", json!({"run_id":run["id"]})).unwrap();
        let kinds: Vec<_> = events["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|event| event["type"].as_str())
            .collect();
        assert_eq!(
            kinds,
            ["RUN_QUEUED", "RUN_CLAIMED", "TURN_PROPOSED", "RUN_OUTCOME"]
        );
        let next = dispatch(
            &mut db,
            "automation.events",
            json!({"run_id":run["id"],"after":events["next_after"]}),
        )
        .unwrap();
        assert!(next["items"].as_array().unwrap().is_empty());
        let resumed = sessions::dispatch(
            &mut db,
            "conversation.get",
            json!({"id":run["conversation_id"]}),
        )
        .unwrap();
        assert_eq!(resumed["conversation"]["source"], "AUTOMATION");
        assert_eq!(resumed["tasks"][0]["id"], run["task_id"]);
        dispatch(&mut db, "automation.cancel", json!({"run_id":run["id"]})).unwrap();
        dispatch(
            &mut db,
            "automation.run",
            json!({"id":def["id"],"key":"button-next","confirm_inactive":true}),
        )
        .unwrap();
        db.conn
            .execute(
                "UPDATE bindings SET revision=revision+1 WHERE id=?1",
                [&binding],
            )
            .unwrap();
        assert_eq!(tick(&mut db).unwrap(), 1);
        let view = dispatch(&mut db, "automation.list", json!({})).unwrap();
        assert_eq!(view["items"][0]["enabled"], false);
        assert_eq!(view["items"][0]["last_run"]["status"], "BLOCKED");
        assert!(view["items"][0]["last_run"]["conversation_id"].is_null());
    }

    #[test]
    fn competing_scheduler_claims_once_and_preserves_abandoned_runs() {
        let (fixture, mut db, binding) = Fixture::new();
        let mut request = manual_request(&binding, "daily");
        request["trigger"] =
            json!({"kind":"DAILY","hour":12,"minute":0,"timezone":"Asia/Ho_Chi_Minh"});
        request["enabled"] = json!(true);
        let def = dispatch(&mut db, "automation.create", request).unwrap();
        let defid = sessions::id(def["id"].as_str().unwrap()).unwrap();
        let time = sessions::now();
        db.conn
            .execute(
                "UPDATE automation_definitions SET next_at=?2 WHERE id=?1",
                params![defid, time - 7 * 86_400],
            )
            .unwrap();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let start = barrier.clone();
        let data = fixture.data.clone();
        let other = std::thread::spawn(move || {
            let mut worker = Database::open(&data).unwrap();
            start.wait();
            schedule(&mut worker, time).unwrap()
        });
        barrier.wait();
        let scheduled = schedule(&mut db, time).unwrap() + other.join().unwrap();
        assert_eq!(scheduled, 1);
        assert!(load(&db.conn, defid).unwrap().next_at.unwrap() > time);
        let abandoned = claim(&mut db, time - 61).unwrap().unwrap();
        assert!(claim(&mut db, time).unwrap().is_none());
        let run = run_view(&db.conn, abandoned.id).unwrap();
        assert_eq!(run["status"], "RECOVERY_REQUIRED");
        assert_eq!(run["attempt"], 1);
        assert!(run["conversation_id"].is_null());
        assert_eq!(tick(&mut db).unwrap(), 0);
    }

    #[test]
    fn pause_between_claim_and_dispatch_blocks_that_occurrence() {
        let (_fixture, mut db, binding) = Fixture::new();
        let mut request = manual_request(&binding, "race-pause");
        request["enabled"] = json!(true);
        let def = dispatch(&mut db, "automation.create", request).unwrap();
        let enqueued = dispatch(
            &mut db,
            "automation.run",
            json!({"id":def["id"],"key":"race-run"}),
        )
        .unwrap();
        let time = sessions::now();
        let claimed = claim(&mut db, time).unwrap().unwrap();
        dispatch(
            &mut db,
            "automation.pause",
            json!({"id":def["id"],"revision":def["revision"],"enabled":false}),
        )
        .unwrap();
        finish(&mut db, claimed, time).unwrap();
        let run = run_view(
            &db.conn,
            sessions::id(enqueued["id"].as_str().unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(run["status"], "BLOCKED");
        assert_eq!(run["outcome"], "DEFINITION_CHANGED");
        assert!(run["task_id"].is_null());
    }

    fn utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> i64 {
        Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
            .single()
            .unwrap()
            .timestamp()
    }

    #[test]
    fn daily_schedule_skips_dst_gap_and_chooses_first_repeated_time() {
        let gap = Trigger::Daily {
            hour: 2,
            minute: 30,
            timezone: "America/New_York".into(),
        };
        assert_eq!(
            gap.next(utc(2026, 3, 8, 0, 0)).unwrap(),
            Some(utc(2026, 3, 9, 6, 30))
        );
        let repeat = Trigger::Daily {
            hour: 1,
            minute: 30,
            timezone: "America/New_York".into(),
        };
        let first = utc(2026, 11, 1, 5, 30);
        assert_eq!(repeat.next(utc(2026, 11, 1, 0, 0)).unwrap(), Some(first));
        assert_eq!(repeat.next(first).unwrap(), Some(utc(2026, 11, 2, 6, 30)));
    }

    #[test]
    fn versioned_templates_only_propose_and_never_enable_schedule() {
        assert_eq!(templates().len(), 6);
        assert!(templates().iter().all(|row| row.permissions == ["PROPOSE"]
            && row.status == "UNVERIFIED"
            && row.revision == 1));
    }
}
