use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub presence: PresenceConfig,
    pub images: ImagesConfig,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub launch_game_on_start: bool,
    pub exit_when_game_closes: bool,
    pub game_log_poll_interval_ms: u64,
    pub discord_update_interval_s: u64,
}

/// Which hero portrait art style to show in Discord.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HeroPortraitStyle {
    /// Standard hero card image.
    #[default]
    Normal,
    /// Gloat/celebration portrait (wider crop).
    Gloat,
    /// Critical/combat portrait.
    Critical,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct PresenceConfig {
    pub show_elapsed_timer: bool,
    pub show_hero_image: bool,
    pub show_statlocker_button: bool,
    pub hero_portrait_style: HeroPortraitStyle,
    pub details_with_hero: String,
    pub details_without_hero: String,
    pub status: StatusStrings,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct StatusStrings {
    pub game_not_running: String,
    pub in_main_menu: String,
    pub in_hideout: String,
    pub in_matchmaking: String,
    pub loading_into_match: String,
    pub in_match: String,
    pub match_location_label: String,
    pub post_match: String,
    pub spectating: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct ImagesConfig {
    pub fallback_large_image: String,
    pub fallback_large_image_tooltip: String,
    pub corner_image: String,
    pub corner_image_tooltip: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            launch_game_on_start: true,
            exit_when_game_closes: true,
            game_log_poll_interval_ms: 500,
            discord_update_interval_s: 5,
        }
    }
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self {
            show_elapsed_timer: true,
            show_hero_image: true,
            show_statlocker_button: false,
            hero_portrait_style: HeroPortraitStyle::Normal,
            details_with_hero: "Playing as {hero}".to_string(),
            details_without_hero: "{phase}".to_string(),
            status: StatusStrings::default(),
        }
    }
}

impl Default for StatusStrings {
    fn default() -> Self {
        Self {
            game_not_running: "Not Running".to_string(),
            in_main_menu: "Browsing the Main Menu".to_string(),
            in_hideout: "In the Hideout".to_string(),
            in_matchmaking: "Searching for a Match...".to_string(),
            loading_into_match: "{mode} - Loading into Match".to_string(),
            in_match: "In Match: {mode}".to_string(),
            match_location_label: "the Cursed Apple".to_string(),
            post_match: "Reviewing Match Results".to_string(),
            spectating: "Spectating a Match".to_string(),
        }
    }
}

impl Default for ImagesConfig {
    fn default() -> Self {
        Self {
            fallback_large_image: "deadlock_logo".to_string(),
            fallback_large_image_tooltip: "Deadlock".to_string(),
            corner_image: "deadlock_logo".to_string(),
            corner_image_tooltip: "Deadlock RPC".to_string(),
        }
    }
}

pub fn apply_vars(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{key}}}"), value);
    }
    result
}

fn config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("config.toml")))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

// Increment this when a migration is added that fixes existing key values.
const CURRENT_CONFIG_VERSION: i64 = 2;

// Recursively fills missing keys in `user` from `defaults`, preserving existing values and comments.
fn merge_defaults(user: &mut toml_edit::Table, defaults: &toml_edit::Table) -> bool {
    let mut changed = false;
    for (key, default_item) in defaults.iter() {
        if user.contains_key(key) {
            if let toml_edit::Item::Table(default_table) = default_item {
                if let Some(user_table) = user.get_mut(key).and_then(|i| i.as_table_mut()) {
                    if merge_defaults(user_table, default_table) {
                        changed = true;
                    }
                }
            }
        } else {
            user.insert(key, default_item.clone());
            log::info!("[config] Added missing key '{key}' with default value");
            changed = true;
        }
    }
    changed
}

// Load config from `config.toml` next to the executable.
//
// - If the file does not exist: write a fully-documented default and return defaults.
// - If the file is malformed: log a warning and return defaults without overwriting.
// - If the file is a valid partial config: unset fields fall back to their defaults,
//   and any missing keys are written back to disk with their default values.
//   Migrations for changed default values are also applied automatically.
pub fn load() -> Config {
    let path = config_path();

    if !path.exists() {
        match std::fs::write(&path, DEFAULT_TOML) {
            Ok(_) => log::info!("[config] Created default config.toml at {}", path.display()),
            Err(e) => log::warn!("[config] Could not write default config.toml: {e}"),
        }
        return Config::default();
    }

    match std::fs::read_to_string(&path) {
        Err(e) => {
            log::warn!("[config] Could not read config.toml: {e} — using defaults");
            Config::default()
        }
        Ok(text) => match toml::from_str::<Config>(&text) {
            Ok(cfg) => {
                update_config_file(&path, &text);
                cfg
            }
            Err(e) => {
                log::warn!("[config] config.toml parse error: {e} — using defaults");
                Config::default()
            }
        },
    }
}

// Applies pending migrations and fills missing keys in a single write pass.
// Uses toml_edit to preserve comments and formatting in the user's file.
fn update_config_file(path: &std::path::Path, text: &str) {
    let Ok(mut doc) = text.parse::<toml_edit::DocumentMut>() else {
        return;
    };
    let Ok(defaults_doc) = DEFAULT_TOML.parse::<toml_edit::DocumentMut>() else {
        return;
    };

    let version = doc
        .get("general")
        .and_then(|i| i.as_table())
        .and_then(|t| t.get("config_version"))
        .and_then(|i| i.as_integer())
        .unwrap_or(0);

    let mut changed = false;

    // Migration v1: replace the bullet character (U+2022) in loading_into_match
    // with a dash. The bullet caused rendering issues in some Discord clients.
    if version < 1 {
        if let Some(status) = doc
            .get_mut("presence")
            .and_then(|i| i.as_table_mut())
            .and_then(|t| t.get_mut("status"))
            .and_then(|i| i.as_table_mut())
        {
            if let Some(item) = status.get_mut("loading_into_match") {
                let cur = item.as_str().unwrap_or("").to_string();
                if cur.contains('\u{2022}') {
                    *item = toml_edit::value(cur.replace('\u{2022}', "-"));
                    log::info!("[config] Migration v1: fixed bullet char in loading_into_match");
                    changed = true;
                }
            }
        }
        if let Some(general) = doc.get_mut("general").and_then(|i| i.as_table_mut()) {
            general["config_version"] = toml_edit::value(1_i64);
            changed = true;
        }
    }

    // Migration v2: replace show_hero_gloat_portrait (bool) with hero_portrait_style (string enum).
    if version < 2 {
        if let Some(presence) = doc.get_mut("presence").and_then(|i| i.as_table_mut()) {
            let was_gloat = presence
                .get("show_hero_gloat_portrait")
                .and_then(|i| i.as_bool())
                .unwrap_or(false);
            presence.remove("show_hero_gloat_portrait");
            if !presence.contains_key("hero_portrait_style") {
                let style = if was_gloat { "gloat" } else { "normal" };
                presence.insert("hero_portrait_style", toml_edit::value(style));
                log::info!(
                    "[config] Migration v2: converted show_hero_gloat_portrait to hero_portrait_style = \"{style}\""
                );
                changed = true;
            }
        }
        if let Some(general) = doc.get_mut("general").and_then(|i| i.as_table_mut()) {
            general["config_version"] = toml_edit::value(CURRENT_CONFIG_VERSION);
            changed = true;
        }
    }

    if merge_defaults(doc.as_table_mut(), defaults_doc.as_table()) {
        changed = true;
    }

    if !changed {
        return;
    }

    match std::fs::write(path, doc.to_string()) {
        Ok(_) => log::info!("[config] config.toml updated"),
        Err(e) => log::warn!("[config] Could not update config.toml: {e}"),
    }
}

pub const DEFAULT_TOML: &str = include_str!("default_config.toml");
