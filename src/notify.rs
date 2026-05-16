pub fn alert(body: &str) {
    platform::show(body);
}

#[cfg(not(windows))]
mod platform {
    pub fn show(body: &str) {
        let _ = notify_rust::Notification::new()
            .appname("Deadlock RPC")
            .summary("Deadlock RPC")
            .body(body)
            .show();
    }
}

#[cfg(windows)]
mod platform {
    pub fn show(body: &str) {
        let body = body.to_string();
        // Spawn so the caller is not blocked waiting for the user to dismiss the dialog.
        std::thread::spawn(move || {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use winapi::um::winuser::{MessageBoxW, MB_ICONWARNING, MB_OK};

            let to_wide = |s: &str| -> Vec<u16> {
                OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
            };

            unsafe {
                MessageBoxW(
                    std::ptr::null_mut(),
                    to_wide(&body).as_ptr(),
                    to_wide("Deadlock RPC").as_ptr(),
                    MB_ICONWARNING | MB_OK,
                );
            }
        });
    }
}
