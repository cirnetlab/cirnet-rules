use super::super::storage::{self, AppError, Database};
use super::model::{Document, Kind, digest, encode};
use rusqlite::{Connection, OptionalExtension, Row, TransactionBehavior, params};
use serde::Serialize;
use serde_json::{Value, json};

const COLUMNS: &str = "id,kind,slug,title,description,scope,origin,source_name,revision,active_revision,draft_json IS NOT NULL,archived,deleted_at,updated_at,source_revision,source_digest";

#[derive(Serialize)]
pub struct Asset {
    pub id: String,
    pub kind: Kind,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub scope: String,
    pub origin: String,
    pub source_name: Option<String>,
    pub source_revision: Option<i64>,
    pub source_digest: Option<String>,
    pub revision: i64,
    pub active_revision: Option<i64>,
    pub draft: bool,
    pub archived: bool,
    pub deleted_at: Option<i64>,
    pub updated_at: i64,
}

impl Asset {
    fn from_row(row: &Row<'_>) -> rusqlite::Result<Self> {
        let kind = match row.get::<_, String>(1)?.as_str() {
            "rule" => Kind::Rule,
            "skill" => Kind::Skill,
            "profile" => Kind::Profile,
            "reference" => Kind::Reference,
            "policy" => Kind::Policy,
            "command" => Kind::Command,
            "hook" => Kind::Hook,
            _ => return Err(rusqlite::Error::InvalidQuery),
        };
        Ok(Self {
            id: row.get(0)?,
            kind,
            slug: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            scope: row.get(5)?,
            origin: row.get(6)?,
            source_name: row.get(7)?,
            revision: row.get(8)?,
            active_revision: row.get(9)?,
            draft: row.get(10)?,
            archived: row.get(11)?,
            deleted_at: row.get(12)?,
            updated_at: row.get(13)?,
            source_revision: row.get(14)?,
            source_digest: row.get(15)?,
        })
    }
}

#[derive(Serialize)]
pub struct Revision {
    pub revision: i64,
    pub digest: String,
    pub event: String,
    pub prior_revision: Option<i64>,
    pub created_at: i64,
}

#[derive(Serialize)]
pub struct ChangeEvent {
    pub event_id: i64,
    pub revision: i64,
    pub event: String,
    pub created_at: i64,
}

pub struct RevisionMetadata {
    pub title: String,
    pub slug: String,
    pub bytes: usize,
    pub digest: String,
}

pub struct Store<'a> {
    conn: &'a Connection,
}

impl<'a> Store<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { conn: &db.conn }
    }

    pub fn from_connection(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn read<T>(
        db: &Database,
        f: impl FnOnce(&Store<'_>) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let tx = db.conn.unchecked_transaction()?;
        let result = f(&Store::from_connection(&tx))?;
        tx.commit()?;
        Ok(result)
    }

    pub fn write<T>(
        db: &mut Database,
        f: impl FnOnce(&Store<'_>, i64) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let tx = db
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let result = f(&Store { conn: &tx }, storage::now()?)?;
        tx.commit()?;
        Ok(result)
    }

    pub fn random_id(&self, prefix: &str) -> Result<String, AppError> {
        let suffix: String =
            self.conn
                .query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))?;
        Ok(format!("{prefix}-{suffix}"))
    }

    pub fn asset(&self, id: &str, include_deleted: bool) -> Result<Asset, AppError> {
        Ok(self.conn.query_row(
            &format!(
                "SELECT {COLUMNS} FROM governance_assets WHERE id=? AND (? OR deleted_at IS NULL)"
            ),
            params![id, include_deleted],
            Asset::from_row,
        )?)
    }

    pub fn assets(
        &self,
        kind: Option<Kind>,
        search: &str,
        archived: bool,
        published: bool,
        after: &str,
        limit: u32,
    ) -> Result<Vec<Asset>, AppError> {
        let mut query = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM governance_assets WHERE id>?1 AND deleted_at IS NULL
              AND (?2 IS NULL OR kind=?2) AND (?3 OR NOT archived)
              AND (instr(lower(title),lower(?4))>0 OR instr(lower(slug),lower(?4))>0)
              AND (NOT ?6 OR active_revision IS NOT NULL)
             ORDER BY id LIMIT ?5"
        ))?;
        Ok(query
            .query_map(
                params![
                    after,
                    kind.map(Kind::name),
                    archived,
                    search,
                    limit,
                    published
                ],
                Asset::from_row,
            )?
            .collect::<Result<_, _>>()?)
    }

    pub fn insert(
        &self,
        kind: Kind,
        slug: &str,
        title: &str,
        source: Option<&str>,
        document: &Document,
        at: i64,
    ) -> Result<String, AppError> {
        let id = self.random_id("asset")?;
        let origin = if source.is_some() {
            "local_import"
        } else {
            "local"
        };
        self.conn.execute(
            "INSERT INTO governance_assets(id,kind,slug,title,description,origin,source_name,revision,draft_json,created_at,updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,1,?8,?9,?9)",
            params![id, kind.name(), slug, title, document.description, origin, source, encode(document)?, at],
        )?;
        self.event(&id, 1, "CREATED", at)?;
        Ok(id)
    }

    pub fn draft(&self, id: &str) -> Result<Option<Document>, AppError> {
        let serialized: Option<String> = self.conn.query_row(
            "SELECT draft_json FROM governance_assets WHERE id=?",
            [id],
            |row| row.get(0),
        )?;
        serialized.map(|text| decode_document(&text)).transpose()
    }

    pub fn revision(&self, id: &str, revision: i64) -> Result<Document, AppError> {
        let (serialized, expected): (String, String) = self.conn.query_row(
            "SELECT document_json,digest FROM governance_revisions WHERE asset_id=? AND revision=?",
            params![id, revision],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        if digest(&serialized) != expected {
            return Err(AppError::new(
                "INTEGRITY",
                "Published governance revision failed digest verification.",
            ));
        }
        decode_document(&serialized)
    }

    pub fn active(&self, asset: &Asset) -> Result<Option<Document>, AppError> {
        asset
            .active_revision
            .map(|revision| self.revision(&asset.id, revision))
            .transpose()
    }

    pub fn set_draft(
        &self,
        asset: &Asset,
        title: &str,
        document: Option<&Document>,
        at: i64,
    ) -> Result<(), AppError> {
        let data = document.map(encode).transpose()?;
        let description = match document {
            Some(value) => value.description.clone(),
            None => self
                .active(asset)?
                .map(|value| value.description)
                .unwrap_or_default(),
        };
        self.conn.execute(
            "UPDATE governance_assets SET revision=revision+1,title=?,draft_json=?,description=?,updated_at=? WHERE id=?",
            params![title, data, description, at, asset.id],
        )?;
        self.set_relations(
            &asset.id,
            "draft",
            document
                .map(|value| value.dependencies.as_slice())
                .unwrap_or_default(),
        )?;
        self.event(
            &asset.id,
            asset.revision + 1,
            if document.is_some() {
                "DRAFT_SAVED"
            } else {
                "DRAFT_DISCARDED"
            },
            at,
        )?;
        Ok(())
    }

    pub fn rename(&self, asset: &Asset, slug: &str, title: &str, at: i64) -> Result<(), AppError> {
        self.conn.execute(
            "UPDATE governance_assets SET revision=revision+1,slug=?,title=?,updated_at=? WHERE id=?",
            params![slug,title,at,asset.id],
        )?;
        self.event(&asset.id, asset.revision + 1, "RENAMED", at)
    }

    pub fn set_status(
        &self,
        asset: &Asset,
        archived: bool,
        deleted: bool,
        at: i64,
    ) -> Result<(), AppError> {
        self.conn.execute(
            "UPDATE governance_assets SET revision=revision+1,archived=?,deleted_at=?,updated_at=? WHERE id=?",
            params![archived, deleted.then_some(at), at, asset.id],
        )?;
        if deleted {
            self.set_relations(&asset.id, "active", &[])?;
            self.set_relations(&asset.id, "draft", &[])?;
        }
        self.event(
            &asset.id,
            asset.revision + 1,
            if deleted {
                "DELETED"
            } else if archived {
                "ARCHIVED"
            } else {
                "RESTORED"
            },
            at,
        )
    }

    pub fn history(&self, id: &str, before: i64, limit: u32) -> Result<Vec<Revision>, AppError> {
        let mut query = self.conn.prepare(
            "SELECT revision,digest,event,prior_revision,created_at FROM governance_revisions
             WHERE asset_id=? AND revision<? ORDER BY revision DESC LIMIT ?",
        )?;
        Ok(query
            .query_map(params![id, before, limit], |row| {
                Ok(Revision {
                    revision: row.get(0)?,
                    digest: row.get(1)?,
                    event: row.get(2)?,
                    prior_revision: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?
            .collect::<Result<_, _>>()?)
    }

    pub fn events(&self, id: &str, before: i64, limit: u32) -> Result<Vec<ChangeEvent>, AppError> {
        let mut query = self.conn.prepare(
            "SELECT id,asset_revision,event,created_at FROM governance_events
             WHERE asset_id=? AND id<? ORDER BY id DESC LIMIT ?",
        )?;
        Ok(query
            .query_map(params![id, before, limit], |row| {
                Ok(ChangeEvent {
                    event_id: row.get(0)?,
                    revision: row.get(1)?,
                    event: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<_, _>>()?)
    }

    pub fn reverse(&self, id: &str) -> Result<Vec<Value>, AppError> {
        let mut query = self.conn.prepare(
            "SELECT DISTINCT a.id,a.title,a.kind FROM governance_dependencies d JOIN governance_assets a ON a.id=d.asset_id
             WHERE d.target_id=? AND a.deleted_at IS NULL ORDER BY a.id LIMIT 64"
        )?;
        Ok(query.query_map([id], |row| {
            Ok(json!({"id":row.get::<_, String>(0)?,"title":row.get::<_, String>(1)?,"kind":row.get::<_, String>(2)?}))
        })?.collect::<Result<_, _>>()?)
    }

    pub fn reaches(&self, start: &str, target: &str) -> Result<bool, AppError> {
        let (count, found): (i64, bool) = self.conn.query_row(
            "WITH RECURSIVE reached(id) AS (
                SELECT ?1 UNION SELECT d.target_id FROM governance_dependencies d
                JOIN reached r ON d.asset_id=r.id WHERE d.layer='active' LIMIT 513
             ) SELECT count(*),max(id=?2) FROM reached",
            params![start, target],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        if count == 513 {
            return Err(AppError::new(
                "DEPENDENCY_LIMIT",
                "The dependency graph exceeds the preview limit.",
            ));
        }
        Ok(found)
    }

    fn set_relations(
        &self,
        id: &str,
        layer: &str,
        dependencies: &[String],
    ) -> Result<(), AppError> {
        self.conn.execute(
            "DELETE FROM governance_dependencies WHERE asset_id=? AND layer=?",
            params![id, layer],
        )?;
        let mut query = self.conn.prepare(
            "INSERT INTO governance_dependencies(asset_id,target_id,layer) VALUES(?,?,?)",
        )?;
        for target in dependencies {
            query.execute(params![id, target, layer])?;
        }
        Ok(())
    }

    pub fn store_initial_relations(
        &self,
        id: &str,
        dependencies: &[String],
    ) -> Result<(), AppError> {
        self.set_relations(id, "draft", dependencies)
    }

    pub fn revision_metadata(&self, id: &str, revision: i64) -> Result<RevisionMetadata, AppError> {
        Ok(self.conn.query_row(
            "SELECT title,slug,canonical_bytes,digest FROM governance_revisions WHERE asset_id=? AND revision=?",
            params![id, revision],
            |row| Ok(RevisionMetadata { title:row.get(0)?,slug:row.get(1)?,bytes:row.get(2)?,digest:row.get(3)? }),
        )?)
    }

    pub fn set_origin(&self, id: &str, revision: i64, digest: &str) -> Result<(), AppError> {
        self.conn.execute(
            "UPDATE governance_assets SET source_revision=?,source_digest=? WHERE id=?",
            params![revision, digest, id],
        )?;
        Ok(())
    }
    pub fn default_profile(&self) -> Result<Option<String>, AppError> {
        Ok(self
            .conn
            .query_row(
                "SELECT asset_id FROM governance_defaults WHERE singleton=1",
                [],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn set_default_profile(
        &self,
        asset: &Asset,
        enabled: bool,
        at: i64,
    ) -> Result<(), AppError> {
        if enabled {
            self.conn.execute(
                "INSERT INTO governance_defaults VALUES(1,?) ON CONFLICT(singleton) DO UPDATE SET asset_id=excluded.asset_id",
                [&asset.id],
            )?;
        } else {
            self.conn.execute(
                "DELETE FROM governance_defaults WHERE asset_id=?",
                [&asset.id],
            )?;
        }
        self.conn.execute(
            "UPDATE governance_assets SET revision=revision+1,updated_at=? WHERE id=?",
            params![at, asset.id],
        )?;
        self.event(
            &asset.id,
            asset.revision + 1,
            if enabled {
                "DEFAULT_PROFILE_SET"
            } else {
                "DEFAULT_PROFILE_CLEARED"
            },
            at,
        )
    }
    pub fn active_relations(&self, id: &str) -> Result<Vec<String>, AppError> {
        let mut query = self.conn.prepare(
            "SELECT target_id FROM governance_dependencies WHERE asset_id=? AND layer='active' ORDER BY target_id",
        )?;
        Ok(query
            .query_map([id], |row| row.get(0))?
            .collect::<Result<_, _>>()?)
    }

    pub fn prepare_review(&self, asset: &Asset, digest: &str, at: i64) -> Result<(), AppError> {
        match self.review(asset, at) {
            Ok((_, original)) if original == digest => return Ok(()),
            Ok(_) => return Err(AppError::new("CONFLICT", "The reviewed digest changed.")),
            Err(error) if error.code == "PERMISSION_DENIED" => {}
            Err(error) => return Err(error),
        }
        let token = self.random_id("review")?;
        self.conn.execute(
            "INSERT INTO governance_reviews(asset_id,expected_revision,digest,token,expires_at) VALUES(?,?,?,?,?)
             ON CONFLICT(asset_id) DO UPDATE SET expected_revision=excluded.expected_revision,digest=excluded.digest,
                token=excluded.token,expires_at=excluded.expires_at",
            params![asset.id,asset.revision,digest,token,at+600],
        )?;
        self.event(&asset.id, asset.revision, "REVIEWED_LOCAL_OPERATOR", at)
    }

    pub fn review(&self, asset: &Asset, at: i64) -> Result<(String, String), AppError> {
        self.conn.query_row(
            "SELECT token,digest FROM governance_reviews WHERE asset_id=? AND expected_revision=? AND expires_at>=?",
            params![asset.id,asset.revision,at], |row| Ok((row.get(0)?,row.get(1)?)),
        ).optional()?.ok_or_else(|| AppError::new("PERMISSION_DENIED", "Review the exact current draft before applying it."))
    }

    pub fn activate(
        &self,
        asset: &Asset,
        document: &Document,
        at: i64,
        rollback: Option<i64>,
    ) -> Result<(), AppError> {
        let serialized = encode(document)?;
        let revision = asset.revision + 1;
        let event = if rollback.is_some() {
            "ROLLBACK"
        } else {
            "PUBLISHED"
        };
        self.conn.execute(
            "INSERT INTO governance_revisions(asset_id,revision,digest,document_json,title,slug,canonical_bytes,event,prior_revision,created_at)
             VALUES(?,?,?,?,?,?,?,?,?,?)",
            params![asset.id,revision,digest(&serialized),serialized,asset.title,asset.slug,document.en.len(),event,rollback.or(asset.active_revision),at],
        )?;
        self.conn.execute(
            "UPDATE governance_assets SET active_revision=?1,revision=?1,description=?2,draft_json=NULL,updated_at=?3 WHERE id=?4",
            params![revision,document.description,at,asset.id],
        )?;
        self.set_relations(&asset.id, "active", &document.dependencies)?;
        self.set_relations(&asset.id, "draft", &[])?;
        self.conn.execute(
            "DELETE FROM governance_reviews WHERE asset_id=?",
            [&asset.id],
        )?;
        self.event(&asset.id, revision, event, at)
    }

    pub fn used_bindings(&self, id: &str) -> Result<Vec<String>, AppError> {
        let mut query = self.conn.prepare("SELECT binding_id FROM governance_assignments WHERE asset_id=? ORDER BY binding_id LIMIT 64")?;
        Ok(query
            .query_map([id], |row| row.get(0))?
            .collect::<Result<_, _>>()?)
    }

    pub fn binding(&self, id: &str) -> Result<(), AppError> {
        self.conn
            .query_row(
                "SELECT 1 FROM bindings b JOIN repositories r ON r.id=b.repo_id
             WHERE b.id=? AND b.status='ACTIVE' AND NOT r.archived
                AND b.machine_id=(SELECT value FROM catalog_settings WHERE key='machine_id')",
                [id],
                |_| Ok(()),
            )
            .optional()?
            .ok_or_else(|| {
                AppError::new(
                    "INVALID_BINDING",
                    "Select an available registered worktree.",
                )
            })
    }

    pub fn assigned(
        &self,
        binding: &str,
        after: &str,
        limit: u32,
    ) -> Result<Vec<String>, AppError> {
        let mut query = self.conn.prepare(
            "SELECT asset_id FROM governance_assignments WHERE binding_id=?1 AND asset_id>?2 ORDER BY asset_id LIMIT ?3",
        )?;
        Ok(query
            .query_map(params![binding, after, limit], |row| row.get(0))?
            .collect::<Result<_, _>>()?)
    }

    pub fn assign(
        &self,
        asset: &Asset,
        binding: &str,
        enabled: bool,
        at: i64,
    ) -> Result<(), AppError> {
        if enabled {
            self.conn.execute(
                "INSERT INTO governance_assignments(binding_id,asset_id,created_at) VALUES(?,?,?) ON CONFLICT DO NOTHING",
                params![binding,asset.id,at],
            )?;
        } else {
            self.conn.execute(
                "DELETE FROM governance_assignments WHERE binding_id=? AND asset_id=?",
                params![binding, asset.id],
            )?;
        }
        self.conn.execute(
            "UPDATE governance_assets SET revision=revision+1,updated_at=? WHERE id=?",
            params![at, asset.id],
        )?;
        self.event(
            &asset.id,
            asset.revision + 1,
            if enabled { "ASSIGNED" } else { "UNASSIGNED" },
            at,
        )
    }

    fn event(&self, id: &str, revision: i64, event: &str, at: i64) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO governance_events(asset_id,asset_revision,event,created_at) VALUES(?,?,?,?)",
            params![id,revision,event,at],
        )?;
        Ok(())
    }

    pub fn replay(&self, key: &str, fingerprint: &str) -> Result<Option<Asset>, AppError> {
        let receipt: Option<(String, String, i64)> = self
            .conn
            .query_row(
                "SELECT fingerprint,asset_id,result_revision FROM governance_mutations WHERE key=?",
                [key],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()?;
        receipt
            .map(|(original, id, revision)| {
                if original != fingerprint {
                    return Err(AppError::new(
                        "CONFLICT",
                        "The mutation key already identifies different input.",
                    ));
                }
                let current = self.asset(&id, true)?;
                if revision != current.revision {
                    return Err(AppError::new(
                        "IDEMPOTENCY_REPLAY_STALE",
                        "The mutation already completed. Reload current state.",
                    ));
                }
                Ok(current)
            })
            .transpose()
    }

    pub fn record(
        &self,
        key: &str,
        fingerprint: &str,
        asset: &Asset,
        at: i64,
    ) -> Result<(), AppError> {
        self.conn.execute(
            "INSERT INTO governance_mutations(key,fingerprint,asset_id,result_revision,created_at) VALUES(?,?,?,?,?)",
            params![key,fingerprint,asset.id,asset.revision,at],
        )?;
        Ok(())
    }
}

fn decode_document(serialized: &str) -> Result<Document, AppError> {
    serde_json::from_str(serialized)
        .map_err(|_| AppError::new("INTEGRITY", "Stored governance document is invalid."))
}
