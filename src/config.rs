use serde::Deserialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

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
    pub auto_update: bool,
    pub game_log_poll_interval_ms: u64,
    pub discord_update_interval_s: u64,
    // Overrides auto-detection when set to a non-empty path.
    pub game_folder: Option<String>,
    pub declined_shortcut: bool,
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
    /// Small hero icon.
    Icon,
    /// Full-body art uploaded as Discord app assets, keyed by hero key.
    FullBody,
}

impl HeroPortraitStyle {
    pub fn as_u8(self) -> u8 {
        match self {
            HeroPortraitStyle::Normal => 0,
            HeroPortraitStyle::Gloat => 1,
            HeroPortraitStyle::Critical => 2,
            HeroPortraitStyle::Icon => 3,
            HeroPortraitStyle::FullBody => 4,
        }
    }

    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => HeroPortraitStyle::Gloat,
            2 => HeroPortraitStyle::Critical,
            3 => HeroPortraitStyle::Icon,
            4 => HeroPortraitStyle::FullBody,
            _ => HeroPortraitStyle::Normal,
        }
    }

}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct PresenceConfig {
    pub show_hero_image: bool,
    pub show_match_timer: bool,
    pub show_statlocker_button: bool,
    pub hero_portrait_style: HeroPortraitStyle,
    pub details_with_hero: String,
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
            auto_update: false,
            game_log_poll_interval_ms: 500,
            discord_update_interval_s: 5,
            game_folder: None,
            declined_shortcut: false,
        }
    }
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self {
            show_hero_image: true,
            show_match_timer: true,
            show_statlocker_button: false,
            hero_portrait_style: HeroPortraitStyle::Normal,
            details_with_hero: "Playing as {hero}".to_string(),
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
            in_match: "{mode}".to_string(),
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
            fallback_large_image_tooltip: "Deadlock RPC".to_string(),
            corner_image: "deadlock_logo".to_string(),
            corner_image_tooltip: "Deadlock RPC".to_string(),
        }
    }
}

pub struct SharedBools {
    pub launch_game_on_start: AtomicBool,
    pub exit_when_game_closes: AtomicBool,
    pub auto_update: AtomicBool,
    pub show_hero_image: AtomicBool,
    pub show_match_timer: AtomicBool,
    pub show_statlocker_button: AtomicBool,
    pub hero_portrait_style: AtomicU8,
    // Set by the tray when a presence setting changes so the RPC loop wakes early.
    pub settings_dirty: AtomicBool,
}

impl SharedBools {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            launch_game_on_start: AtomicBool::new(cfg.general.launch_game_on_start),
            exit_when_game_closes: AtomicBool::new(cfg.general.exit_when_game_closes),
            auto_update: AtomicBool::new(cfg.general.auto_update),
            show_hero_image: AtomicBool::new(cfg.presence.show_hero_image),
            show_match_timer: AtomicBool::new(cfg.presence.show_match_timer),
            show_statlocker_button: AtomicBool::new(cfg.presence.show_statlocker_button),
            hero_portrait_style: AtomicU8::new(cfg.presence.hero_portrait_style.as_u8()),
            settings_dirty: AtomicBool::new(false),
        }
    }
}

// Reads config.toml, flips one boolean key in the given section, and writes it back.
pub fn set_config_bool(section: &str, key: &str, value: bool) {
    let path = config_path();
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            log::warn!("[config] Could not read config for update: {e}");
            return;
        }
    };
    let mut val: toml::Value = match toml::from_str(&text) {
        Ok(v) => v,
        Err(_) => return,
    };
    if let Some(toml::Value::Table(table)) = val.get_mut(section) {
        table.insert(key.to_string(), toml::Value::Boolean(value));
    }
    if let Ok(new_text) = toml::to_string_pretty(&val) {
        let _ = std::fs::write(&path, new_text);
    }
}

pub fn set_config_string(section: &str, key: &str, value: &str) {
    let path = config_path();
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            log::warn!("[config] Could not read config for update: {e}");
            return;
        }
    };
    let mut val: toml::Value = match toml::from_str(&text) {
        Ok(v) => v,
        Err(_) => return,
    };
    if let Some(toml::Value::Table(table)) = val.get_mut(section) {
        table.insert(key.to_string(), toml::Value::String(value.to_string()));
    }
    if let Ok(new_text) = toml::to_string_pretty(&val) {
        let _ = std::fs::write(&path, new_text);
    }
}

// Builds the restored config text. Keeps the auto-managed game_folder and declined_shortcut
// values so a restore doesn't forget the detected install or re-show the shortcut prompt.
fn restored_config_text(current: Option<&toml::Value>) -> String {
    let general = current.and_then(|c| c.get("general"));
    let kept: Vec<(&str, toml::Value)> = [
        ("game_folder", toml::Value::String(String::new())),
        ("declined_shortcut", toml::Value::Boolean(false)),
    ]
    .into_iter()
    .filter_map(|(key, default)| {
        let val = general.and_then(|g| g.get(key))?;
        (val.type_str() == default.type_str() && *val != default).then(|| (key, val.clone()))
    })
    .collect();

    if kept.is_empty() {
        return DEFAULT_TOML.to_string();
    }
    let mut val: toml::Value = toml::from_str(DEFAULT_TOML).expect("default config is valid TOML");
    if let Some(toml::Value::Table(general)) = val.get_mut("general") {
        for (key, v) in kept {
            general.insert(key.to_string(), v);
        }
    }
    toml::to_string_pretty(&val).unwrap_or_else(|_| DEFAULT_TOML.to_string())
}

// Overwrites config.toml with the defaults and resets the live tray toggles to match.
pub fn restore_defaults(shared: &SharedBools) {
    let path = config_path();
    let current = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| toml::from_str::<toml::Value>(&t).ok());
    match std::fs::write(&path, restored_config_text(current.as_ref())) {
        Ok(_) => log::info!("[config] Restored config.toml to defaults"),
        Err(e) => log::warn!("[config] Could not restore config.toml: {e}"),
    }

    let cfg = Config::default();
    let fresh = SharedBools::from_config(&cfg);
    for (live, def) in [
        (&shared.launch_game_on_start, &fresh.launch_game_on_start),
        (&shared.exit_when_game_closes, &fresh.exit_when_game_closes),
        (&shared.auto_update, &fresh.auto_update),
        (&shared.show_hero_image, &fresh.show_hero_image),
        (&shared.show_match_timer, &fresh.show_match_timer),
        (&shared.show_statlocker_button, &fresh.show_statlocker_button),
    ] {
        live.store(def.load(Ordering::Relaxed), Ordering::Relaxed);
    }
    shared
        .hero_portrait_style
        .store(fresh.hero_portrait_style.load(Ordering::Relaxed), Ordering::Relaxed);
    shared.settings_dirty.store(true, Ordering::Relaxed);
}

pub fn apply_vars(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{key}}}"), value);
    }
    result
}

pub fn config_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("config.toml")))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

// Increment this when a migration is added that fixes existing key values.
const CURRENT_CONFIG_VERSION: i64 = 4;

// Recursively fills missing keys in `user` from `defaults`.
// Returns true if any key was added.
fn merge_defaults(user: &mut toml::Value, defaults: &toml::Value) -> bool {
    let (toml::Value::Table(user_table), toml::Value::Table(default_table)) = (user, defaults)
    else {
        return false;
    };
    let mut changed = false;
    for (key, default_val) in default_table {
        if let Some(user_val) = user_table.get_mut(key) {
            if merge_defaults(user_val, default_val) {
                changed = true;
            }
        } else {
            user_table.insert(key.clone(), default_val.clone());
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
                if update_config_file(&path, &text) {
                    // Re-read so this session uses the migrated values, not the pre-migration ones.
                    std::fs::read_to_string(&path)
                        .ok()
                        .and_then(|t| toml::from_str::<Config>(&t).ok())
                        .unwrap_or(cfg)
                } else {
                    cfg
                }
            }
            Err(e) => {
                log::warn!("[config] config.toml parse error: {e} — using defaults");
                Config::default()
            }
        },
    }
}

// Applies pending migrations and fills missing keys in a single write pass.
// Returns true if the file was rewritten.
fn update_config_file(path: &std::path::Path, text: &str) -> bool {
    let Ok(mut val) = toml::from_str::<toml::Value>(text) else {
        return false;
    };
    let Ok(defaults) = toml::from_str::<toml::Value>(DEFAULT_TOML) else {
        return false;
    };

    let mut changed = apply_migrations(&mut val);

    if merge_defaults(&mut val, &defaults) {
        changed = true;
    }

    if !changed {
        return false;
    }

    match toml::to_string_pretty(&val) {
        Ok(new_text) => match std::fs::write(path, new_text) {
            Ok(_) => {
                log::info!("[config] config.toml updated");
                true
            }
            Err(e) => {
                log::warn!("[config] Could not update config.toml: {e}");
                false
            }
        },
        Err(e) => {
            log::warn!("[config] Could not serialize config: {e}");
            false
        }
    }
}

// Rewrites old default values in place based on config_version. Returns true if anything changed.
fn apply_migrations(val: &mut toml::Value) -> bool {
    let version = val
        .get("general")
        .and_then(|g| g.get("config_version"))
        .and_then(|v| v.as_integer())
        .unwrap_or(0);

    let mut changed = false;

    // Migration v1: replace the bullet character (U+2022) in loading_into_match
    // with a dash. The bullet caused rendering issues in some Discord clients.
    if version < 1 {
        if let Some(toml::Value::String(s)) = val
            .get_mut("presence")
            .and_then(|p| p.get_mut("status"))
            .and_then(|s| s.get_mut("loading_into_match"))
        {
            if s.contains('\u{2022}') {
                *s = s.replace('\u{2022}', "-");
                log::info!("[config] Migration v1: fixed bullet char in loading_into_match");
                changed = true;
            }
        }
        if let Some(toml::Value::Table(general)) = val.get_mut("general") {
            general.insert("config_version".to_string(), toml::Value::Integer(1));
            changed = true;
        }
    }

    // Migration v2: replace show_hero_gloat_portrait (bool) with hero_portrait_style (string enum).
    if version < 2 {
        if let Some(toml::Value::Table(presence)) = val.get_mut("presence") {
            let was_gloat = presence
                .get("show_hero_gloat_portrait")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            presence.remove("show_hero_gloat_portrait");
            if !presence.contains_key("hero_portrait_style") {
                let style = if was_gloat { "gloat" } else { "normal" };
                presence.insert(
                    "hero_portrait_style".to_string(),
                    toml::Value::String(style.to_string()),
                );
                log::info!(
                    "[config] Migration v2: converted show_hero_gloat_portrait to hero_portrait_style = \"{style}\""
                );
                changed = true;
            }
        }
        if let Some(toml::Value::Table(general)) = val.get_mut("general") {
            general.insert(
                "config_version".to_string(),
                toml::Value::Integer(CURRENT_CONFIG_VERSION),
            );
            changed = true;
        }
    }

    // Migration v3: the fallback logo tooltip now reads "Deadlock RPC". Only the untouched
    // default is updated so custom tooltips are kept.
    if version < 3 {
        if let Some(toml::Value::String(s)) = val
            .get_mut("images")
            .and_then(|i| i.get_mut("fallback_large_image_tooltip"))
        {
            if s == "Deadlock" {
                *s = "Deadlock RPC".to_string();
                log::info!("[config] Migration v3: set fallback_large_image_tooltip to \"Deadlock RPC\"");
                changed = true;
            }
        }
        if let Some(toml::Value::Table(general)) = val.get_mut("general") {
            general.insert("config_version".to_string(), toml::Value::Integer(3));
            changed = true;
        }
    }

    // Migration v4: trimmed status lines (issue #61). in_match drops its "In Match:" prefix, and
    // details_without_hero is removed because no phase shows it any more. Only untouched past
    // defaults of in_match are updated so custom strings are kept.
    if version < 4 {
        const OLD_IN_MATCH_DEFAULTS: [&str; 2] =
            ["In Match: {mode}", "{mode} \u{2022} Battling in {location}"];
        if let Some(toml::Value::String(s)) = val
            .get_mut("presence")
            .and_then(|p| p.get_mut("status"))
            .and_then(|s| s.get_mut("in_match"))
        {
            if OLD_IN_MATCH_DEFAULTS.contains(&s.as_str()) {
                log::info!("[config] Migration v4: set in_match from \"{s}\" to \"{{mode}}\"");
                *s = "{mode}".to_string();
                changed = true;
            }
        }
        if let Some(toml::Value::Table(presence)) = val.get_mut("presence") {
            if presence.remove("details_without_hero").is_some() {
                log::info!("[config] Migration v4: removed unused details_without_hero");
                changed = true;
            }
        }
        if let Some(toml::Value::Table(general)) = val.get_mut("general") {
            general.insert("config_version".to_string(), toml::Value::Integer(4));
            changed = true;
        }
    }

    changed
}

pub const DEFAULT_TOML: &str = include_str!("default_config.toml");

#[cfg(test)]
mod tests {
    use super::*;

    fn migrate(text: &str) -> toml::Value {
        let mut val: toml::Value = toml::from_str(text).unwrap();
        apply_migrations(&mut val);
        val
    }

    fn in_match(val: &toml::Value) -> &str {
        val["presence"]["status"]["in_match"].as_str().unwrap()
    }

    #[test]
    fn v4_updates_every_past_in_match_default() {
        for (version, old) in [(3, "In Match: {mode}"), (0, "{mode} \u{2022} Battling in {location}")] {
            let val = migrate(&format!(
                "[general]\nconfig_version = {version}\n[presence]\ndetails_without_hero = \"{{phase}}\"\n[presence.status]\nin_match = \"{old}\"\n"
            ));
            assert_eq!(in_match(&val), "{mode}", "from {old:?}");
            assert!(val["presence"].get("details_without_hero").is_none());
            assert_eq!(val["general"]["config_version"].as_integer(), Some(4));
        }
    }

    #[test]
    fn v4_keeps_custom_in_match() {
        let val = migrate(
            "[general]\nconfig_version = 3\n[presence.status]\nin_match = \"Fighting: {mode}\"\n",
        );
        assert_eq!(in_match(&val), "Fighting: {mode}");
    }

    #[test]
    fn v4_leaves_current_configs_alone() {
        let mut val: toml::Value = toml::from_str(DEFAULT_TOML).unwrap();
        assert!(!apply_migrations(&mut val));
    }

    #[test]
    fn default_toml_matches_struct_defaults() {
        let parsed: Config = toml::from_str(DEFAULT_TOML).unwrap();
        let s = StatusStrings::default();
        assert_eq!(parsed.presence.status.in_match, s.in_match);
        assert_eq!(
            toml::from_str::<toml::Value>(DEFAULT_TOML).unwrap()["general"]["config_version"].as_integer(),
            Some(CURRENT_CONFIG_VERSION)
        );
    }

    #[test]
    fn restore_returns_defaults_verbatim_when_nothing_to_keep() {
        assert_eq!(restored_config_text(None), DEFAULT_TOML);
        let custom: toml::Value = toml::from_str(
            "[general]\nauto_update = true\n[presence.status]\nin_match = \"Fighting\"\n",
        )
        .unwrap();
        assert_eq!(restored_config_text(Some(&custom)), DEFAULT_TOML);
    }

    #[test]
    fn restore_keeps_managed_keys() {
        let custom: toml::Value = toml::from_str(
            "[general]\nauto_update = true\ngame_folder = \"/games/Deadlock\"\ndeclined_shortcut = true\n[presence.status]\nin_match = \"Fighting\"\n",
        )
        .unwrap();
        let restored: Config = toml::from_str(&restored_config_text(Some(&custom))).unwrap();
        assert_eq!(restored.general.game_folder.as_deref(), Some("/games/Deadlock"));
        assert!(restored.general.declined_shortcut);
        assert!(!restored.general.auto_update);
        assert_eq!(restored.presence.status.in_match, StatusStrings::default().in_match);
    }
}
