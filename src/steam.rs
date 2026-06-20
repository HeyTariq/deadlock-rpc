use std::path::PathBuf;
use std::sync::OnceLock;

const DEADLOCK_APP_ID: &str = "1422450";
const CONSOLE_LOG_SUFFIX: &str = "game/citadel/console.log";

pub fn find_console_log(game_folder_override: Option<&str>) -> PathBuf {
    // User-configured path takes priority over auto-detection.
    if let Some(folder) = game_folder_override.filter(|s| !s.is_empty()) {
        let game_dir = std::path::Path::new(folder);
        let ends_with_deadlock = game_dir
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.eq_ignore_ascii_case("Deadlock"))
            .unwrap_or(false);
        if !ends_with_deadlock {
            log::warn!(
                "[steam] game_folder does not end with \"Deadlock\": {}",
                game_dir.display()
            );
        }
        let path = game_dir.join(CONSOLE_LOG_SUFFIX);
        log::info!("[steam] Using configured game_folder: {}", path.display());
        return path;
    }

    match try_find_console_log() {
        Some(path) => {
            // Strip the console.log suffix to get the Deadlock root folder.
            let game_root = path
                .ancestors()
                .nth(3) // strip game/citadel/console.log
                .map(|p| p.to_string_lossy().into_owned());
            if let Some(root) = game_root {
                log::info!("[steam] Auto-detected game folder saved to config: {root}");
                crate::config::set_config_string("general", "game_folder", &root);
            }
            path
        }
        None => {
            let fallback = default_fallback().join(CONSOLE_LOG_SUFFIX);
            log::warn!(
                "[steam] Deadlock not found in Steam library. Using fallback path: {}. \
                If Deadlock is installed in a custom Steam library, set game_folder in config.toml.",
                fallback.display()
            );
            crate::notify::warn_alert(
                "Deadlock could not be found in your Steam library.\n\
                Rich presence may not update. Set game_folder in config.toml \
                to your Deadlock install folder.",
            );
            fallback
        }
    }
}

static VDF_PATTERNS: OnceLock<(regex::Regex, regex::Regex)> = OnceLock::new();

fn try_find_console_log() -> Option<PathBuf> {
    let vdf_locations = steam_vdf_locations();

    let (path_re, dir_re) = VDF_PATTERNS.get_or_init(|| (
        regex::Regex::new(r#""path"\s+"([^"]+)""#).unwrap(),
        regex::Regex::new(r#""installdir"\s+"([^"]+)""#).unwrap(),
    ));

    for vdf_path in &vdf_locations {
        let content = match std::fs::read_to_string(vdf_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for cap in path_re.captures_iter(&content) {
            let lib = PathBuf::from(&cap[1]);
            let manifest = lib.join(format!("steamapps/appmanifest_{DEADLOCK_APP_ID}.acf"));
            let mtext = match std::fs::read_to_string(&manifest) {
                Ok(t) => t,
                Err(_) => continue,
            };
            if let Some(m) = dir_re.captures(&mtext) {
                let game_path = lib.join("steamapps/common").join(&m[1]);
                if game_path.exists() {
                    return Some(game_path.join(CONSOLE_LOG_SUFFIX));
                }
            }
        }
    }

    // Hardcoded fallbacks
    hardcoded_fallbacks()
        .into_iter()
        .find(|p| p.parent().is_some_and(|d| d.exists()))
}


#[cfg(unix)]
pub fn steam_exe_path() -> Option<PathBuf> {
    // Check every directory in $PATH for a `steam` binary.
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            let candidate = PathBuf::from(dir).join("steam");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    // Fall back to known installation locations not always on $PATH.
    let home = dirs::home_dir().unwrap_or_default();
    let candidates = [
        home.join(".local/share/Steam/steam.sh"),
        home.join(".steam/steam.sh"),
        PathBuf::from("/usr/bin/steam"),
        PathBuf::from("/usr/local/bin/steam"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

#[cfg(unix)]
fn steam_vdf_locations() -> Vec<PathBuf> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return vec![],
    };
    vec![
        home.join(".steam/steam/steamapps/libraryfolders.vdf"),
        home.join(".local/share/Steam/steamapps/libraryfolders.vdf"),
    ]
}

#[cfg(unix)]
fn hardcoded_fallbacks() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    vec![
        home.join(".steam/steam/steamapps/common/Deadlock").join(CONSOLE_LOG_SUFFIX),
        home.join(".local/share/Steam/steamapps/common/Deadlock").join(CONSOLE_LOG_SUFFIX),
    ]
}

#[cfg(unix)]
fn default_fallback() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".local/share/Steam/steamapps/common/Deadlock")
}

#[cfg(windows)]
pub fn steam_exe_path() -> Option<PathBuf> {
    steam_root().map(|r| r.join("steam.exe"))
}

#[cfg(windows)]
fn steam_root() -> Option<PathBuf> {
    // Try registry first
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    if let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam") {
        if let Ok(path) = key.get_value::<String, _>("SteamPath") {
            let p = PathBuf::from(path);
            if p.exists() {
                return Some(p);
            }
        }
    }
    // Common install locations
    let candidates = [
        r"C:\Program Files (x86)\Steam",
        r"C:\Program Files\Steam",
    ];
    candidates.iter().map(PathBuf::from).find(|p| p.exists())
}

#[cfg(windows)]
fn steam_vdf_locations() -> Vec<PathBuf> {
    match steam_root() {
        Some(root) => vec![root.join("steamapps\\libraryfolders.vdf")],
        None => vec![],
    }
}

#[cfg(windows)]
fn hardcoded_fallbacks() -> Vec<PathBuf> {
    match steam_root() {
        Some(root) => vec![
            root.join("steamapps\\common\\Deadlock").join(CONSOLE_LOG_SUFFIX),
        ],
        None => vec![
            PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Deadlock")
                .join(CONSOLE_LOG_SUFFIX),
        ],
    }
}

#[cfg(windows)]
fn default_fallback() -> PathBuf {
    steam_root()
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)\Steam"))
        .join("steamapps\\common\\Deadlock")
}
