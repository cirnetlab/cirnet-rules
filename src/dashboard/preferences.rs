//! Validated desktop presentation preferences. Nothing in this store grants authority.
use super::storage::{AppError, Database, Empty, decode};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Deserializer, Serialize};
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

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum Accent {
    #[default]
    Cirnet,
    Cobalt,
    Green,
    Yellow,
    Pink,
    Orange,
    Purple,
    White,
    Custom,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum UiFont {
    #[default]
    System,
    Monospace,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Language {
    En,
    Vi,
    #[serde(rename = "zh_CN")]
    ZhCn,
}

fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    // A missing field uses the legacy value. A supplied field must contain a value of its own.
    T::deserialize(deserializer).map(Some)
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
struct Appearance {
    accent: Accent,
    ui_font: UiFont,
    content_scale: u16,
    code_scale: u16,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    interface_px: Option<u16>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    conversation_px: Option<u16>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    code_px: Option<u16>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    density_px: Option<u16>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    background: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    foreground: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    custom_accent: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    font_family: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present"
    )]
    language: Option<Language>,
}

impl Appearance {
    fn validate(&self) -> Result<(), AppError> {
        let invalid_scale = [self.content_scale, self.code_scale]
            .into_iter()
            .any(|scale| scale != 0 && !(85..=150).contains(&scale));
        let invalid_size = [self.interface_px, self.conversation_px, self.code_px]
            .into_iter()
            .flatten()
            .any(|size| !(10..=26).contains(&size));
        let invalid_density = self.density_px.is_some_and(|px| !(2..=14).contains(&px));
        let invalid_color = [
            self.background.as_deref(),
            self.foreground.as_deref(),
            self.custom_accent.as_deref(),
        ]
        .into_iter()
        .flatten()
        .any(|hex| {
            hex.len() != 7
                || !hex.starts_with('#')
                || !hex.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
        });
        let missing_custom = self.accent == Accent::Custom && self.custom_accent.is_none();
        let invalid_font = self.font_family.as_deref().is_some_and(|font| {
            font.is_empty()
                || font.trim() != font
                || font.chars().count() > 128
                || font.chars().any(char::is_control)
        });
        if invalid_scale
            || invalid_size
            || invalid_density
            || invalid_color
            || missing_custom
            || invalid_font
        {
            return Err(invalid());
        }
        Ok(())
    }
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
    #[serde(default)]
    appearance: Appearance,
    sidebar_width: u16,
    sidebar_collapsed: bool,
    #[serde(default)]
    reduced_motion: bool,
    repository_layout: Option<RepositoryLayout>,
    notification_level: NotificationLevel,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            density: Density::Comfortable,
            text_scale: 100,
            appearance: Appearance::default(),
            sidebar_width: 260,
            sidebar_collapsed: false,
            reduced_motion: false,
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
        self.appearance.validate()?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appearance_defaults_are_compatible_and_checked() {
        let mut legacy = serde_json::to_value(Settings::default()).unwrap();
        legacy.as_object_mut().unwrap().remove("appearance");
        let mut settings: Settings = serde_json::from_value(legacy).unwrap();
        assert_eq!(settings.appearance.content_scale, 0);
        assert_eq!(settings.appearance.code_scale, 0);
        settings.appearance.content_scale = 151;
        assert!(settings.validate().is_err());
        settings.appearance.content_scale = 115;
        settings.appearance.code_scale = 90;
        assert!(settings.validate().is_ok());
    }

    fn preferences_schema() -> jsonschema::Validator {
        let schema: Value =
            serde_json::from_str(include_str!("../../en/schemas/dashboard-ipc.schema.json"))
                .unwrap();
        jsonschema::validator_for(&json!({
            "$schema": schema["$schema"],
            "$defs": schema["$defs"],
            "$ref": "#/$defs/desktopSettings"
        }))
        .unwrap()
    }

    #[test]
    fn personalized_appearance_round_trips_with_legacy_fields() {
        let fixture =
            crate::test_support::temporary_directory("openrules-personalized-appearance").unwrap();
        let mut settings = serde_json::to_value(Settings::default()).unwrap();
        settings["appearance"] = json!({
            "accent": "custom",
            "custom_accent": "#246AC0",
            "background": "#161819",
            "foreground": "#FAFAFA",
            "ui_font": "system",
            "font_family": "Noto Sans CJK SC",
            "interface_px": 20,
            "conversation_px": 18,
            "code_px": 16,
            "density_px": 9,
            "language": "zh_CN",
            "content_scale": 0,
            "code_scale": 0
        });
        let db = Database::open(&fixture).unwrap();
        let reply = dispatch(
            &db,
            "preferences.set",
            json!({"expected_revision":0,"settings":settings}),
        )
        .unwrap();
        assert_eq!(reply["settings"], settings);
        assert!(preferences_schema().is_valid(&settings));
        drop(db);

        let reopened = Database::open(&fixture).unwrap();
        assert_eq!(read(&reopened).unwrap(), reply);
        drop(reopened);
        std::fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn personalized_appearance_bounds_agree_with_the_wire_schema() {
        let schema = preferences_schema();
        let fixture =
            crate::test_support::temporary_directory("openrules-invalid-appearance").unwrap();
        let db = Database::open(&fixture).unwrap();
        let baseline = serde_json::to_value(Settings::default()).unwrap();
        assert!(schema.is_valid(&baseline));
        for (key, value) in [
            ("accent", json!("custom")),
            ("accent", json!("unknown")),
            ("interface_px", json!(9)),
            ("conversation_px", json!(27)),
            ("code_px", json!("16")),
            ("density_px", json!(1)),
            ("language", json!("zh")),
            ("font_family", json!("")),
            ("font_family", json!(" Trailing")),
            ("font_family", json!("A\u{0085}B")),
            ("font_family", json!("a".repeat(129))),
            ("background", json!("#112233ff")),
            ("foreground", json!("#gg0011")),
            ("custom_accent", json!("#001122\n")),
            ("interface_px", Value::Null),
        ] {
            let mut settings = baseline.clone();
            settings["appearance"][key] = value;
            assert!(!schema.is_valid(&settings), "schema accepted {key}");
            let error = dispatch(
                &db,
                "preferences.set",
                json!({"expected_revision":0,"settings":settings}),
            )
            .unwrap_err();
            assert_eq!(error.code, "INVALID_INPUT", "{key}");
        }
        assert_eq!(read(&db).unwrap()["revision"], 0);
        drop(db);
        std::fs::remove_dir_all(fixture).unwrap();
    }

    #[test]
    fn appearance_survives_database_reopen() {
        let fixture = crate::test_support::temporary_directory("openrules-appearance").unwrap();
        let settings = Settings {
            appearance: Appearance {
                accent: Accent::Cobalt,
                ui_font: UiFont::Monospace,
                content_scale: 125,
                code_scale: 90,
                ..Appearance::default()
            },
            ..Settings::default()
        };
        {
            let db = Database::open(&fixture).unwrap();
            save(
                &db,
                Save {
                    expected_revision: 0,
                    settings,
                },
            )
            .unwrap();
        }
        let db = Database::open(&fixture).unwrap();
        let reply = read(&db).unwrap();
        assert_eq!(reply["revision"], 1);
        assert_eq!(
            reply["settings"]["appearance"],
            json!({"accent":"cobalt","ui_font":"monospace","content_scale":125,"code_scale":90})
        );
        drop(db);
        std::fs::remove_dir_all(fixture).unwrap();
    }
}
