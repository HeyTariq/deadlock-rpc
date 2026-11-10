#[cfg(windows)]
fn messagebox(body: &str, icon: u32) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let body_owned = body.to_string();
    std::thread::spawn(move || {
        let title: Vec<u16> = OsStr::new("Deadlock RPC")
            .encode_wide()
            .chain(Some(0))
            .collect();
        let text: Vec<u16> = OsStr::new(&body_owned)
            .encode_wide()
            .chain(Some(0))
            .collect();
        unsafe {
            winapi::um::winuser::MessageBoxW(
                std::ptr::null_mut(),
                text.as_ptr(),
                title.as_ptr(),
                winapi::um::winuser::MB_OK | icon | winapi::um::winuser::MB_TOPMOST,
            );
        }
    });
}

#[cfg(windows)]
pub fn alert(body: &str) {
    messagebox(body, winapi::um::winuser::MB_ICONINFORMATION);
}

#[cfg(windows)]
pub fn warn_alert(body: &str) {
    messagebox(body, winapi::um::winuser::MB_ICONWARNING);
}

#[cfg(not(windows))]
pub fn alert(body: &str) {
    let _ = notify_rust::Notification::new()
        .appname("Deadlock RPC")
        .summary("Deadlock RPC")
        .body(body)
        .show();
}

#[cfg(not(windows))]
pub fn warn_alert(body: &str) {
    let _ = notify_rust::Notification::new()
        .appname("Deadlock RPC")
        .summary("Deadlock RPC")
        .body(body)
        .urgency(notify_rust::Urgency::Critical)
        .show();
}

// Blocking warning dialog with a confirm button and Cancel. Returns true only if the user confirms.
#[cfg(windows)]
pub fn confirm(body: &str, _ok_label: &str) -> bool {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winuser::{MessageBoxW, IDYES, MB_DEFBUTTON2, MB_ICONWARNING, MB_YESNO};

    let title: Vec<u16> = OsStr::new("Deadlock RPC").encode_wide().chain(Some(0)).collect();
    let text: Vec<u16> = OsStr::new(body).encode_wide().chain(Some(0)).collect();
    let result = unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2,
        )
    };
    result == IDYES
}

// Tries zenity (GTK/GNOME), then kdialog (KDE). Returns false if neither is available.
#[cfg(not(windows))]
pub fn confirm(body: &str, ok_label: &str) -> bool {
    let zenity = std::process::Command::new("zenity")
        .args(["--question", "--icon=dialog-warning", "--title=Deadlock RPC", "--cancel-label=Cancel"])
        .arg(format!("--ok-label={ok_label}"))
        .arg(format!("--text={body}"))
        .status();
    if let Ok(status) = zenity {
        return status.success();
    }

    let kdialog = std::process::Command::new("kdialog")
        .args(["--title", "Deadlock RPC", "--warningyesno", body])
        .args(["--yes-label", ok_label, "--no-label", "Cancel"])
        .status();
    match kdialog {
        Ok(status) => status.success(),
        Err(_) => {
            log::warn!("[notify] No dialog tool found (zenity or kdialog), so the action was cancelled");
            false
        }
    }
}
