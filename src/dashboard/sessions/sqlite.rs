//! SQLite implementation of the conversation lifecycle repository and unit of work.
use super::super::automations::{ACTIVE_RUN, retire_definition};
use super::{
    AppError, digest, event,
    lifecycle::{
        AutomationLink, Blocker, ConversationFilter, ConversationPage, ConversationSummary,
        Inspection, LifecycleRepository, LifecycleUnitOfWork, Preview, Receipt, Target,
    },
};
use rusqlite::{Connection, OptionalExtension, Row, TransactionBehavior, params};
use serde_json::json;

pub(super) struct Repository<'a>(pub &'a mut Connection);
struct UnitOfWork<'a>(&'a Connection);

const CONVERSATION_COLUMNS: &str = "c.id,c.title,c.state,c.profile,c.source,c.revision,c.created_at,c.updated_at,c.provider_thread";
const FILTER: &str = "c.state=?1 AND (?2='' OR instr(lower(c.title),lower(?2))>0)
    AND (?3 IS NULL OR EXISTS(SELECT 1 FROM conversation_bindings b
        WHERE b.conversation_id=c.id AND b.binding_id=?3))
    AND (?4 IS NULL OR c.profile=?4) AND (?5 IS NULL OR c.source=?5)
    AND (?6 IS NULL OR c.updated_at<=?6) AND (?7 IS NULL OR c.updated_at>=?7)";

fn row_summary(row: &Row<'_>) -> rusqlite::Result<ConversationSummary> {
    Ok(ConversationSummary {
        id: row.get::<_, i64>(0)?.to_string(),
        title: row.get(1)?,
        state: row.get(2)?,
        profile: row.get(3)?,
        source: row.get(4)?,
        revision: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
        provider_thread: row.get(8)?,
    })
}

fn summary(conn: &Connection, id: i64) -> Result<ConversationSummary, AppError> {
    Ok(conn.query_row(
        &format!("SELECT {CONVERSATION_COLUMNS} FROM conversations c WHERE c.id=?1"),
        [id],
        row_summary,
    )?)
}

impl LifecycleRepository for Repository<'_> {
    fn page(&self, filter: ConversationFilter) -> Result<ConversationPage, AppError> {
        let limit = filter.page_size()?;
        let before = filter.cursor()?;
        let tx = self.0.unchecked_transaction()?;
        let (mut items, total) = {
            let mut statement = tx.prepare(&format!(
                "SELECT {CONVERSATION_COLUMNS} FROM conversations c
                WHERE {FILTER} AND c.id<?8 ORDER BY c.id DESC LIMIT ?9"
            ))?;
            let query = filter.query.trim();
            let items = statement
                .query_map(
                    params![
                        filter.state,
                        query,
                        filter.binding_id,
                        filter.profile,
                        filter.source,
                        filter.updated_before,
                        filter.updated_after,
                        before,
                        limit + 1
                    ],
                    row_summary,
                )?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            let total = if filter.state == "ARCHIVED" || filter.include_total {
                Some(tx.query_row(
                    &format!("SELECT count(*) FROM conversations c WHERE {FILTER}"),
                    params![
                        filter.state,
                        query,
                        filter.binding_id,
                        filter.profile,
                        filter.source,
                        filter.updated_before,
                        filter.updated_after
                    ],
                    |row| row.get(0),
                )?)
            } else {
                None
            };
            (items, total)
        };
        tx.commit()?;
        let more = items.len() as i64 > limit;
        if more {
            items.pop();
        }
        let next_before = more
            .then(|| items.last().map(|item| item.id.clone()))
            .flatten();
        Ok(ConversationPage {
            items,
            next_before,
            total,
        })
    }

    fn transaction<T>(
        &mut self,
        execute: impl FnOnce(&dyn LifecycleUnitOfWork) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = execute(&UnitOfWork(&tx))?;
        tx.commit()?;
        Ok(result)
    }
}

const TARGET_TASKS: &str = "SELECT id FROM task_proposals WHERE conversation_id=?1
    AND (?2 IS NULL OR message_id=?2)";
const TARGET_RUNS: &str = "((?2 IS NULL AND conversation_id=?1) OR task_id IN
    (SELECT id FROM target_tasks))";

pub(super) fn conversation_blockers(
    conn: &Connection,
    conversation: i64,
) -> Result<(i64, Vec<Blocker>), AppError> {
    // Direct runs and runs linked by task are disjoint here. Visit cross-links once,
    // and count the large direct sets without sorting all of their detail rows.
    let mut statement = conn.prepare_cached(&format!(
        "WITH linked_runs AS MATERIALIZED (
            SELECT a.id,a.status,a.attempt,a.lease_expires_at
            FROM task_proposals p CROSS JOIN automation_runs a ON a.task_id=p.id
            WHERE p.conversation_id=?1 AND a.conversation_id IS NOT ?1 AND {ACTIVE_RUN}
        ), direct_runs AS (
            SELECT id,status,attempt,lease_expires_at FROM automation_runs
            WHERE conversation_id=?1 AND {ACTIVE_RUN} ORDER BY id LIMIT 64
        ), run_page AS (
            SELECT 'run' AS kind,id,status AS state,attempt,lease_expires_at AS lease FROM direct_runs
            UNION ALL
            SELECT 'run',id,status,attempt,lease_expires_at FROM linked_runs ORDER BY id LIMIT 64
        ), task_page AS (
            SELECT 'task' AS kind,id,state,NULL AS attempt,NULL AS lease FROM task_proposals
            WHERE conversation_id=?1 AND state!='CANCELLED' ORDER BY id LIMIT 64
        ), page AS (
            SELECT kind,id,state,attempt,lease FROM run_page
            UNION ALL SELECT kind,id,state,attempt,lease FROM task_page ORDER BY kind,id LIMIT 64
        )
        SELECT kind,id,state,attempt,lease,
            (SELECT count(*) FROM task_proposals WHERE conversation_id=?1 AND state!='CANCELLED')
            + (SELECT count(*) FROM automation_runs WHERE conversation_id=?1 AND {ACTIVE_RUN})
            + (SELECT count(*) FROM linked_runs)
        FROM page ORDER BY kind,id"
    ))?;
    collect_blockers(&mut statement, [conversation])
}

fn blockers_in_scope(
    conn: &Connection,
    target: Target,
    automation: Option<i64>,
) -> Result<(i64, Vec<Blocker>), AppError> {
    let mut statement = conn.prepare(&format!(
        "WITH target_tasks AS ({TARGET_TASKS}), blocked AS (
            SELECT 'task' AS kind,id,state,NULL AS attempt,NULL AS lease FROM task_proposals
                WHERE id IN (SELECT id FROM target_tasks) AND state!='CANCELLED'
            UNION ALL
            SELECT 'run',id,status,attempt,lease_expires_at FROM automation_runs
                WHERE ({TARGET_RUNS} OR automation_id=?3) AND {ACTIVE_RUN}
        ) SELECT kind,id,state,attempt,lease,count(*) OVER () FROM blocked
            ORDER BY kind,id LIMIT 64"
    ))?;
    collect_blockers(
        &mut statement,
        params![target.conversation, target.message, automation],
    )
}

fn collect_blockers(
    statement: &mut rusqlite::Statement<'_>,
    params: impl rusqlite::Params,
) -> Result<(i64, Vec<Blocker>), AppError> {
    let rows = statement
        .query_map(params, |row| {
            Ok((
                row.get::<_, i64>(5)?,
                Blocker {
                    kind: if row.get_ref(0)? == rusqlite::types::ValueRef::Text(b"task") {
                        "task"
                    } else {
                        "run"
                    },
                    id: row.get::<_, i64>(1)?.to_string(),
                    state: row.get(2)?,
                    attempt: row.get(3)?,
                    lease_expires_at: row.get(4)?,
                },
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok((
        rows.first().map_or(0, |row| row.0),
        rows.into_iter().map(|row| row.1).collect(),
    ))
}

impl LifecycleUnitOfWork for UnitOfWork<'_> {
    fn transition(
        &self,
        id: i64,
        revision: i64,
        state: &str,
        time: i64,
    ) -> Result<ConversationSummary, AppError> {
        let existing = summary(self.0, id)?;
        if existing.revision != revision {
            return Err(AppError::new("CONFLICT", "Conversation revision is stale"));
        }
        if existing.state == state {
            return Ok(existing);
        }
        self.0.execute(
            "UPDATE conversations SET state=?2,revision=revision+1,updated_at=?3 WHERE id=?1",
            params![id, state, time],
        )?;
        event(
            self.0,
            (Some(id), None, None, None),
            &id.to_string(),
            if state == "ARCHIVED" {
                "CONVERSATION_ARCHIVED"
            } else {
                "CONVERSATION_UNARCHIVED"
            },
            &json!({}),
            time,
        )?;
        summary(self.0, id)
    }

    fn inspect(&self, target: Target) -> Result<Inspection, AppError> {
        let row = summary(self.0, target.conversation)?;
        let mismatch: bool = self.0.query_row(
            "SELECT EXISTS(SELECT 1 FROM automation_runs r JOIN task_proposals p ON p.id=r.task_id
                WHERE (r.conversation_id=?1 OR p.conversation_id=?1)
                    AND r.conversation_id IS NOT NULL AND r.conversation_id!=p.conversation_id)",
            [target.conversation],
            |row| row.get(0),
        )?;
        if mismatch {
            return Err(AppError::new(
                "INTEGRITY",
                "A related automation run has inconsistent ownership",
            ));
        }
        let (message_count, task_count, run_count) = self.0.query_row(
            &format!(
                "WITH target_tasks AS ({TARGET_TASKS}) SELECT
                (SELECT count(*) FROM messages WHERE conversation_id=?1 AND (?2 IS NULL OR id=?2)),
                (SELECT count(*) FROM target_tasks),
                (SELECT count(*) FROM automation_runs WHERE {TARGET_RUNS})"
            ),
            params![target.conversation, target.message],
            |row| Ok((row.get::<_, i64>(0)?, row.get(1)?, row.get(2)?)),
        )?;
        if target.message.is_some() && message_count != 1 {
            return Err(AppError::new(
                "NOT_FOUND",
                "Message does not exist in this conversation",
            ));
        }
        // A generated conversation has both the originating run FK and the
        // reserved key written by the runtime when it materializes that run.
        let automation = if target.message.is_none() {
            self.0
                .query_row(
                    "SELECT a.id,a.name,a.revision,a.enabled FROM conversations c
                 JOIN automation_runs r ON r.conversation_id=c.id AND c.create_key=('auto-'||r.id)
                 JOIN automation_definitions a ON a.id=r.automation_id
                 WHERE c.id=?1 AND c.source='AUTOMATION' AND a.deleted_at IS NULL",
                    [target.conversation],
                    |row| {
                        Ok(AutomationLink {
                            id: row.get(0)?,
                            name: row.get(1)?,
                            revision: row.get(2)?,
                            enabled: row.get(3)?,
                        })
                    },
                )
                .optional()?
        } else {
            None
        };
        let (blocker_count, blockers) =
            blockers_in_scope(self.0, target, automation.as_ref().map(|link| link.id))?;
        Ok(Inspection {
            revision: row.revision,
            title: row.title,
            message_count,
            task_count,
            run_count,
            automation,
            blocker_count,
            blockers,
        })
    }

    fn issue_preview(&self, preview: &Preview, now: i64) -> Result<String, AppError> {
        self.0.execute(
            "DELETE FROM session_delete_previews WHERE expires_at<=?1",
            [now],
        )?;
        self.0.execute("DELETE FROM session_delete_previews WHERE token IN
            (SELECT token FROM session_delete_previews ORDER BY expires_at DESC,rowid DESC LIMIT -1 OFFSET 127)", [])?;
        Ok(self.0.query_row(
            "INSERT INTO session_delete_previews(token,conversation_id,message_id,revision,scope_digest,expires_at)
             VALUES(lower(hex(randomblob(32))),?1,?2,?3,?4,?5) RETURNING token",
            params![preview.target.conversation, preview.target.message, preview.revision,
                preview.scope_digest, preview.expires_at], |row| row.get(0))?)
    }

    fn read_preview(&self, token: &str) -> Result<Option<Preview>, AppError> {
        Ok(self
            .0
            .query_row(
                "SELECT conversation_id,message_id,revision,scope_digest,expires_at
            FROM session_delete_previews WHERE token=?1",
                [token],
                |row| {
                    Ok(Preview {
                        target: Target {
                            conversation: row.get(0)?,
                            message: row.get(1)?,
                        },
                        revision: row.get(2)?,
                        scope_digest: row.get(3)?,
                        expires_at: row.get(4)?,
                    })
                },
            )
            .optional()?)
    }

    fn read_receipt(&self, key_digest: &str) -> Result<Option<Receipt>, AppError> {
        Ok(self
            .0
            .query_row(
                "SELECT conversation_id,message_id,revision,token_digest,deleted_at
            FROM session_delete_receipts WHERE key_digest=?1",
                [key_digest],
                |row| {
                    Ok(Receipt {
                        target: Target {
                            conversation: row.get(0)?,
                            message: row.get(1)?,
                        },
                        revision: row.get(2)?,
                        token_digest: row.get(3)?,
                        deleted_at: row.get(4)?,
                    })
                },
            )
            .optional()?)
    }

    fn record_receipt(&self, key_digest: &str, receipt: &Receipt) -> Result<(), AppError> {
        self.0.execute(
            "INSERT INTO session_delete_receipts VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                key_digest,
                receipt.target.conversation,
                receipt.target.message,
                receipt.revision,
                receipt.token_digest,
                receipt.deleted_at
            ],
        )?;
        Ok(())
    }

    fn purge(
        &self,
        target: Target,
        automation: Option<&AutomationLink>,
        time: i64,
    ) -> Result<(), AppError> {
        if let Some(link) = automation {
            retire_definition(self.0, link.id, link.revision, time)?;
        }
        let values = params![target.conversation, target.message];
        // Terminal run history remains addressable. Remove linked prompt-derived
        // snapshots and detach the deleted records before removing their rows.
        self.0.execute(
            &format!(
                "WITH target_tasks AS ({TARGET_TASKS}) UPDATE automation_runs
            SET conversation_id=CASE WHEN ?2 IS NULL THEN NULL ELSE conversation_id END,
                task_id=NULL,snapshot_json='{{}}' WHERE {TARGET_RUNS}"
            ),
            values,
        )?;
        self.0.execute(&format!("WITH target_tasks AS ({TARGET_TASKS}) UPDATE session_events
            SET conversation_id=CASE WHEN ?2 IS NULL THEN NULL ELSE conversation_id END,
                task_id=NULL,correlation='run-'||run_id,
                payload_json=CASE WHEN event_type IN ('RUN_OUTCOME','RUN_CANCELLED','RUN_RECOVERY_REQUIRED')
                    THEN payload_json ELSE '{{}}' END
            WHERE run_id IS NOT NULL AND {TARGET_RUNS}"), values)?;
        self.0.execute(
            &format!(
                "WITH target_tasks AS ({TARGET_TASKS}) DELETE FROM session_events
            WHERE run_id IS NULL AND {TARGET_RUNS}"
            ),
            values,
        )?;
        self.0.execute(
            &format!("DELETE FROM task_proposals WHERE id IN ({TARGET_TASKS})"),
            values,
        )?;
        if let Some(message) = target.message {
            let key: String = self.0.query_row(
                "SELECT request_key FROM messages WHERE id=?1 AND conversation_id=?2",
                params![message, target.conversation],
                |row| row.get(0),
            )?;
            self.0.execute(
                "INSERT INTO session_retired_message_keys VALUES (?1,?2)",
                params![target.conversation, digest(&json!(key))],
            )?;
            self.0.execute(
                "DELETE FROM messages WHERE id=?1 AND conversation_id=?2",
                params![message, target.conversation],
            )?;
            self.0.execute(
                "UPDATE conversations SET revision=revision+1,updated_at=?2 WHERE id=?1",
                params![target.conversation, time],
            )?;
            event(
                self.0,
                (Some(target.conversation), None, None, None),
                &message.to_string(),
                "MESSAGE_DELETED",
                &json!({"message_id":message.to_string()}),
                time,
            )?;
        } else {
            let key: String = self.0.query_row(
                "SELECT create_key FROM conversations WHERE id=?1",
                [target.conversation],
                |row| row.get(0),
            )?;
            self.0.execute(
                "INSERT INTO session_retired_conversation_keys VALUES (?1)",
                [digest(&json!(key))],
            )?;
            self.0.execute(
                "DELETE FROM messages WHERE conversation_id=?1",
                [target.conversation],
            )?;
            self.0.execute(
                "DELETE FROM conversation_bindings WHERE conversation_id=?1",
                [target.conversation],
            )?;
            self.0.execute(
                "DELETE FROM conversations WHERE id=?1",
                [target.conversation],
            )?;
        }
        Ok(())
    }
}

pub(super) enum Sequence {
    Conversation,
    Message,
    Proposal,
    Event,
}

pub(super) fn next_id(conn: &Connection, sequence: Sequence) -> Result<i64, AppError> {
    let (table, column) = match sequence {
        Sequence::Conversation => ("conversations", "id"),
        Sequence::Message => ("messages", "id"),
        Sequence::Proposal => ("task_proposals", "id"),
        Sequence::Event => ("session_events", "sequence"),
    };
    Ok(conn.query_row(
        &format!(
            "UPDATE session_id_sequences SET last_id =
        max(last_id,(SELECT COALESCE(max({column}),0) FROM {table}))+1
        WHERE name=?1 RETURNING last_id"
        ),
        [table],
        |row| row.get(0),
    )?)
}

pub(super) fn next_ordinal(conn: &Connection, conversation: i64) -> Result<i64, AppError> {
    Ok(conn.query_row("INSERT INTO session_message_ordinals
        SELECT ?1,coalesce(max(ordinal),0)+1 FROM messages WHERE conversation_id=?1
        ON CONFLICT(conversation_id) DO UPDATE SET last_ordinal=max(excluded.last_ordinal,last_ordinal+1)
        RETURNING last_ordinal", [conversation], |row| row.get(0))?)
}

pub(super) fn retired_key(
    conn: &Connection,
    conversation: Option<i64>,
    key: &str,
) -> Result<(), AppError> {
    let hash = digest(&json!(key));
    let retired: bool = if let Some(conversation) = conversation {
        conn.query_row("SELECT EXISTS(SELECT 1 FROM session_retired_message_keys WHERE conversation_id=?1 AND key_digest=?2)",
            params![conversation, hash], |row| row.get(0))?
    } else {
        conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM session_retired_conversation_keys WHERE key_digest=?1)",
            [hash],
            |row| row.get(0),
        )?
    };
    if retired {
        return Err(AppError::new(
            "DELETED",
            "This request belongs to a permanently deleted record",
        ));
    }
    Ok(())
}
