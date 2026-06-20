use log::{info, warn};
#[cfg(not(debug_assertions))]
use std::path::PathBuf;

const DEADLOCK_APP_ID: &str = "1422450";

pub fn launch_deadlock() {
    info!("[launcher] Launching Deadlock with -condebug...");
    match launch_via_steam() {
        Ok(_) => info!("[launcher] Steam launch initiated."),
        Err(e) => {
            warn!("[launcher] Failed to launch Deadlock: {e}");
            crate::notify::warn_alert(
                "Failed to launch Deadlock.\n\
                Make sure Steam is installed and running, then launch the game manually.",
            );
        }
    }
}

#[cfg(unix)]
fn launch_via_steam() -> std::io::Result<()> {
    let steam = crate::steam::steam_exe_path()
        .unwrap_or_else(|| std::path::PathBuf::from("steam"));
    info!("[launcher] Using Steam executable: {}", steam.display());
    std::process::Command::new(steam)
        .args(["-applaunch", DEADLOCK_APP_ID, "-condebug"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(windows)]
fn launch_via_steam() -> std::io::Result<()> {
    let steam_exe = crate::steam::steam_exe_path()
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Program Files (x86)\Steam\steam.exe"));
    std::process::Command::new(steam_exe)
        .args(["-applaunch", DEADLOCK_APP_ID, "-condebug"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| ())
}

#[cfg(not(debug_assertions))]
pub fn install_shortcut() {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            warn!("[install] Could not determine executable path: {e}");
            return;
        }
    };

    let dest = match shortcut_path() {
        Some(p) => p,
        None => {
            warn!("[install] Could not determine shortcut path");
            return;
        }
    };

    if dest.exists() && shortcut_current(&dest, &exe) {
        info!("[install] Shortcut is current, skipping");
        return;
    }

    if !prompt_shortcut(&dest) {
        info!("[install] User declined shortcut creation");
        return;
    }

    match install_platform_shortcut(&exe, &dest) {
        Ok(()) => info!("[install] Shortcut created: {}", dest.display()),
        Err(e) => warn!("[install] Failed to create shortcut: {e}"),
    }
}

#[cfg(all(unix, not(debug_assertions)))]
fn shortcut_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".local/share/applications/deadlock-rpc.desktop"))
}

#[cfg(all(windows, not(debug_assertions)))]
fn shortcut_path() -> Option<PathBuf> {
    let userprofile = std::env::var("USERPROFILE").ok()?;
    Some(PathBuf::from(userprofile).join("Desktop").join("Deadlock RPC.lnk"))
}

#[cfg(all(unix, not(debug_assertions)))]
fn prompt_shortcut(dest: &std::path::Path) -> bool {
    let text = format!(
        "Would you like to add Deadlock RPC to your applications menu?\n\
         This will create a launcher shortcut at:\n{}",
        dest.display()
    );

    let zenity = std::process::Command::new("zenity")
        .args(["--question", "--title=Deadlock RPC", "--ok-label=Yes", "--cancel-label=No"])
        .arg(format!("--text={text}"))
        .status();

    if let Ok(status) = zenity {
        return status.success();
    }

    let kdialog = std::process::Command::new("kdialog")
        .args(["--title", "Deadlock RPC", "--yesno", &text])
        .status();

    match kdialog.ok().and_then(|s| s.code()) {
        Some(0) => true,
        _ => false,
    }
}

#[cfg(all(windows, not(debug_assertions)))]
fn prompt_shortcut(dest: &std::path::Path) -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winuser::{MessageBoxW, IDYES, MB_ICONQUESTION, MB_YESNO};

    let message = format!(
        "Would you like to create a shortcut on your Desktop?\n\nThis will add:\n{}",
        dest.display()
    );
    let message_wide: Vec<u16> = OsStr::new(&message)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let caption_wide: Vec<u16> = OsStr::new("Deadlock RPC")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let result = unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            message_wide.as_ptr(),
            caption_wide.as_ptr(),
            MB_YESNO | MB_ICONQUESTION,
        )
    };

    result == IDYES
}

#[cfg(all(unix, not(debug_assertions)))]
fn shortcut_current(dest: &std::path::Path, exe: &std::path::Path) -> bool {
    let Ok(content) = std::fs::read_to_string(dest) else { return false };
    let expected = format!("Exec={}", exe.display());
    content.lines().any(|line| line == expected)
}

#[cfg(all(windows, not(debug_assertions)))]
fn shortcut_current(dest: &std::path::Path, exe: &std::path::Path) -> bool {
    let script = format!(
        r#"(New-Object -COM WScript.Shell).CreateShortcut('{}').TargetPath"#,
        dest.display()
    );
    let Ok(output) = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }
    let target = String::from_utf8_lossy(&output.stdout).trim().to_string();
    std::path::Path::new(&target) == exe
}

#[cfg(all(unix, not(debug_assertions)))]
const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

#[cfg(all(unix, not(debug_assertions)))]
fn install_xdg_icon() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let icon_dir = PathBuf::from(home)
        .join(".local/share/icons/hicolor/256x256/apps");
    std::fs::create_dir_all(&icon_dir).ok()?;
    let icon_path = icon_dir.join("deadlock-rpc.png");
    std::fs::write(&icon_path, ICON_PNG).ok()?;
    Some(icon_path)
}

#[cfg(all(unix, not(debug_assertions)))]
fn install_platform_shortcut(exe: &std::path::Path, dest: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    install_xdg_icon();

    let desktop = format!(
        "[Desktop Entry]\n\
         Version=1.0\n\
         Name=Deadlock RPC\n\
         Comment=Discord Rich Presence for Deadlock\n\
         Exec={exe}\n\
         Icon=deadlock-rpc\n\
         Terminal=false\n\
         Type=Application\n\
         Categories=Game;\n",
        exe = exe.display()
    );

    std::fs::write(dest, desktop)?;

    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755))?;

    Ok(())
}

#[cfg(all(windows, not(debug_assertions)))]
fn icon_path(exe: &std::path::Path, filename: &str) -> Option<PathBuf> {
    let p = exe.parent()?.join("assets").join(filename);
    if p.exists() { Some(p) } else { None }
}

#[cfg(all(windows, not(debug_assertions)))]
fn install_platform_shortcut(exe: &std::path::Path, dest: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let icon_part = icon_path(exe, "icon.ico")
        .map(|p| format!("$s.IconLocation='{}';", p.display()))
        .unwrap_or_default();

    let script = format!(
        r#"$s=(New-Object -COM WScript.Shell).CreateShortcut('{lnk}');$s.TargetPath='{exe}';{icon_part}$s.Save()"#,
        lnk = dest.display(),
        exe = exe.display(),
    );

    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .status()?;

    if !status.success() {
        return Err("PowerShell shortcut creation failed".into());
    }

    Ok(())
}
