//! Validated desktop presentation preferences. Nothing in this store grants authority.
use super::storage::{AppError, Database, Empty, decode};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Theme {
    Dark,
    Light,
    System,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Density {
    Comfortable,
    Compact,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum NotificationLevel {
    All,
    Important,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum RepositoryStatus {
    All,
    Active,
    Missing,
    Inaccessible,
    Rebind,
    Archived,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RepositoryLayout {
    widths: [u16; 5],
    order: [u8; 5],
    sort_column: i8,
    sort_descending: bool,
    show_archived: bool,
    status: RepositoryStatus,
    selected_id: Option<String>,
    #[serde(default)]
    scroll: u32,
    #[serde(default)]
    horizontal_scroll: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    theme: Theme,
    density: Density,
    text_scale: u16,
    sidebar_width: u16,
    sidebar_collapsed: bool,
    repository_layout: Option<RepositoryLayout>,
    notification_level: NotificationLevel,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            density: Density::Comfortable,
            text_scale: 100,
            sidebar_width: 260,
            sidebar_collapsed: false,
            repository_layout: None,
            notification_level: NotificationLevel::All,
        }
    }
}

impl Settings {
    fn validate(&mut self) -> Result<(), AppError> {
        if !(85..=150).contains(&self.text_scale) || !(210..=360).contains(&self.sidebar_width) {
            return Err(invalid());
        }
        if let Some(layout) = &mut self.repository_layout {
            let mut order = layout.order;
            order.sort_unstable();
            if order != [0, 1, 2, 3, 4]
                || layout
                    .widths
                    .iter()
                    .any(|width| !(48..=4096).contains(width))
                || !(-1..=4).contains(&layout.sort_column)
                || layout.scroll > 1_000_000
                || layout.horizontal_scroll > 1_000_000
            {
                return Err(invalid());
            }
            if layout.selected_id.as_deref() == Some("") {
                layout.selected_id = None;
            }
            if let Some(id) = &layout.selected_id {
                super::catalog::validate_id(id, "binding").map_err(|_| invalid())?;
            }
        }
        Ok(())
    }
}

fn invalid() -> AppError {
    AppError::new(
        "INVALID_INPUT",
        "Desktop preferences are outside the supported bounds.",
    )
}

fn read(db: &Database) -> Result<Value, AppError> {
    let saved: Option<(i32, String)> = db
        .conn
        .query_row(
            "SELECT revision, settings FROM desktop_preferences WHERE singleton=1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    let Some((revision, raw)) = saved else {
        return Ok(json!({"revision":0,"settings":Settings::default()}));
    };
    let corrupt = || AppError::new("INTEGRITY", "Saved desktop preferences are invalid.");
    let mut settings: Settings = serde_json::from_str(&raw).map_err(|_| corrupt())?;
    settings.validate().map_err(|_| corrupt())?;
    Ok(json!({"revision":revision,"settings":settings}))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    expected_revision: i32,
    settings: Settings,
}

fn save(db: &Database, mut input: Save) -> Result<Value, AppError> {
    input.settings.validate()?;
    let revision = input
        .expected_revision
        .checked_add(1)
        .filter(|revision| *revision > 0)
        .ok_or_else(invalid)?;
    let payload = serde_json::to_string(&input.settings)
        .map_err(|_| AppError::new("STORAGE_UNAVAILABLE", "Cannot encode desktop preferences."))?;
    let changed = if input.expected_revision == 0 {
        db.conn.execute(
            "INSERT INTO desktop_preferences (singleton, revision, settings)
             VALUES (1, ?1, ?2) ON CONFLICT(singleton) DO NOTHING",
            params![revision, payload],
        )?
    } else {
        db.conn.execute(
            "UPDATE desktop_preferences SET revision=?1, settings=?2
             WHERE singleton=1 AND revision=?3",
            params![revision, payload, input.expected_revision],
        )?
    };
    if changed == 0 {
        return Err(AppError::new(
            "CONFLICT",
            "Desktop preferences changed in another window. Reload them and retry.",
        ));
    }
    Ok(json!({"revision":revision,"settings":input.settings}))
}

pub fn dispatch(db: &Database, method: &str, params: Value) -> Result<Value, AppError> {
    match method {
        "preferences.get" => {
            let _: Empty = decode(params)?;
            read(db)
        }
        "preferences.set" => save(db, decode(params)?),
        _ => Err(AppError::new(
            "UNKNOWN_METHOD",
            "Unknown desktop preferences operation.",
        )),
    }
}
