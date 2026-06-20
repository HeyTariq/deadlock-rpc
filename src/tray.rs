// Load the icon PNG from the assets directory, returning raw RGBA bytes.
// Checks next to the executable first (release), then the current working
// directory (development via `cargo run`).
fn load_rgba() -> Option<(Vec<u8>, u32, u32)> {
    let candidates = [
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("assets").join("icon.png"))),
        Some(std::path::PathBuf::from("assets/icon.png")),
    ];

    let path = candidates.into_iter().flatten().find(|p| p.exists())?;

    let img = image::open(path).ok()?.into_rgba8();
    let (w, h) = img.dimensions();
    Some((img.into_raw(), w, h))
}

// Spawns the system tray icon and blocks forever.
// The only way out is the user clicking Quit, which calls process::exit.
pub fn run(shared: std::sync::Arc<crate::config::SharedBools>) {
    #[cfg(target_os = "linux")]
    linux::run(shared);

    #[cfg(not(target_os = "linux"))]
    windows::run(shared);
}

#[cfg(target_os = "linux")]
mod linux {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;

    struct DeadlockTray {
        argb: Vec<u8>,
        icon_w: i32,
        icon_h: i32,
        shared: Arc<crate::config::SharedBools>,
    }

    impl ksni::Tray for DeadlockTray {
        fn id(&self) -> String {
            "deadlock-rpc".to_string()
        }

        fn title(&self) -> String {
            "Deadlock RPC".to_string()
        }

        fn icon_name(&self) -> String {
            "applications-games".to_string()
        }

        fn icon_pixmap(&self) -> Vec<ksni::Icon> {
            if self.argb.is_empty() {
                return vec![];
            }
            vec![ksni::Icon {
                width: self.icon_w,
                height: self.icon_h,
                data: self.argb.clone(),
            }]
        }

        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                title: "Deadlock RPC".to_string(),
                icon_name: String::new(),
                icon_pixmap: vec![],
                description: String::new(),
            }
        }

        fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
            use ksni::menu::*;
            vec![
                SubMenu {
                    label: "Settings   ".to_string(),
                    submenu: vec![
                        CheckmarkItem {
                            label: "Launch Game on Start".to_string(),
                            checked: self.shared.launch_game_on_start.load(Ordering::Relaxed),
                            activate: Box::new(|tray: &mut Self| {
                                let old = tray.shared.launch_game_on_start.fetch_xor(true, Ordering::Relaxed);
                                crate::config::set_config_bool("general", "launch_game_on_start", !old);
                            }),
                            ..Default::default()
                        }
                        .into(),
                        CheckmarkItem {
                            label: "Exit When Game Closes".to_string(),
                            checked: self.shared.exit_when_game_closes.load(Ordering::Relaxed),
                            activate: Box::new(|tray: &mut Self| {
                                let old = tray.shared.exit_when_game_closes.fetch_xor(true, Ordering::Relaxed);
                                crate::config::set_config_bool("general", "exit_when_game_closes", !old);
                            }),
                            ..Default::default()
                        }
                        .into(),
                        ksni::MenuItem::Separator,
                        CheckmarkItem {
                            label: "Show Hero Image".to_string(),
                            checked: self.shared.show_hero_image.load(Ordering::Relaxed),
                            activate: Box::new(|tray: &mut Self| {
                                let old = tray.shared.show_hero_image.fetch_xor(true, Ordering::Relaxed);
                                crate::config::set_config_bool("presence", "show_hero_image", !old);
                                tray.shared.settings_dirty.store(true, Ordering::Relaxed);
                            }),
                            ..Default::default()
                        }
                        .into(),
                        CheckmarkItem {
                            label: "Show Statlocker Button".to_string(),
                            checked: self.shared.show_statlocker_button.load(Ordering::Relaxed),
                            activate: Box::new(|tray: &mut Self| {
                                let old = tray.shared.show_statlocker_button.fetch_xor(true, Ordering::Relaxed);
                                crate::config::set_config_bool("presence", "show_statlocker_button", !old);
                                tray.shared.settings_dirty.store(true, Ordering::Relaxed);
                            }),
                            ..Default::default()
                        }
                        .into(),
                        SubMenu {
                            label: "Hero Portrait Style".to_string(),
                            submenu: {
                                use crate::config::HeroPortraitStyle;
                                let style = HeroPortraitStyle::from_u8(
                                    self.shared.hero_portrait_style.load(Ordering::Relaxed)
                                );
                                vec![
                                    CheckmarkItem {
                                        label: "Normal".to_string(),
                                        checked: style == HeroPortraitStyle::Normal,
                                        activate: Box::new(|tray: &mut Self| {
                                            tray.shared.hero_portrait_style.store(
                                                crate::config::HeroPortraitStyle::Normal.as_u8(),
                                                Ordering::Relaxed,
                                            );
                                            crate::config::set_config_string(
                                                "presence", "hero_portrait_style", "normal",
                                            );
                                            tray.shared.settings_dirty.store(true, Ordering::Relaxed);
                                        }),
                                        ..Default::default()
                                    }
                                    .into(),
                                    CheckmarkItem {
                                        label: "Gloat".to_string(),
                                        checked: style == HeroPortraitStyle::Gloat,
                                        activate: Box::new(|tray: &mut Self| {
                                            tray.shared.hero_portrait_style.store(
                                                crate::config::HeroPortraitStyle::Gloat.as_u8(),
                                                Ordering::Relaxed,
                                            );
                                            crate::config::set_config_string(
                                                "presence", "hero_portrait_style", "gloat",
                                            );
                                            tray.shared.settings_dirty.store(true, Ordering::Relaxed);
                                        }),
                                        ..Default::default()
                                    }
                                    .into(),
                                    CheckmarkItem {
                                        label: "Critical".to_string(),
                                        checked: style == HeroPortraitStyle::Critical,
                                        activate: Box::new(|tray: &mut Self| {
                                            tray.shared.hero_portrait_style.store(
                                                crate::config::HeroPortraitStyle::Critical.as_u8(),
                                                Ordering::Relaxed,
                                            );
                                            crate::config::set_config_string(
                                                "presence", "hero_portrait_style", "critical",
                                            );
                                            tray.shared.settings_dirty.store(true, Ordering::Relaxed);
                                        }),
                                        ..Default::default()
                                    }
                                    .into(),
                                ]
                            },
                            ..Default::default()
                        }
                        .into(),
                        ksni::MenuItem::Separator,
                        StandardItem {
                            label: "Open Config File".to_string(),
                            activate: Box::new(|_| {
                                let path = crate::config::config_path();
                                let _ = std::process::Command::new("xdg-open").arg(path).spawn();
                            }),
                            ..Default::default()
                        }
                        .into(),
                        StandardItem {
                            label: "Open Log File".to_string(),
                            activate: Box::new(|_| {
                                if let Some(path) = crate::logger::log_path() {
                                    let _ = std::process::Command::new("xdg-open").arg(path).spawn();
                                }
                            }),
                            ..Default::default()
                        }
                        .into(),
                    ],
                    ..Default::default()
                }
                .into(),
                ksni::MenuItem::Separator,
                StandardItem {
                    label: "Check for Updates".to_string(),
                    activate: Box::new(|_| {
                        std::thread::spawn(crate::updater::check_from_tray);
                    }),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: "Latest Changes".to_string(),
                    activate: Box::new(|_| {
                        let _ = std::process::Command::new("xdg-open")
                            .arg("https://github.com/HeyTariq/deadlock-rpc/releases/latest")
                            .spawn();
                    }),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: "Source Code".to_string(),
                    activate: Box::new(|_| {
                        let _ = std::process::Command::new("xdg-open")
                            .arg("https://github.com/HeyTariq/deadlock-rpc")
                            .spawn();
                    }),
                    ..Default::default()
                }
                .into(),
                ksni::MenuItem::Separator,
                StandardItem {
                    label: "Quit".to_string(),
                    activate: Box::new(|_| std::process::exit(0)),
                    ..Default::default()
                }
                .into(),
            ]
        }
    }

    fn build_tray(shared: Arc<crate::config::SharedBools>) -> DeadlockTray {
        let (argb, icon_w, icon_h) = match super::load_rgba() {
            Some((rgba, w, h)) => {
                // SNI requires ARGB32 in network (big-endian) byte order.
                // image crate gives RGBA, so reorder each pixel: [A, R, G, B].
                let argb: Vec<u8> = rgba
                    .chunks_exact(4)
                    .flat_map(|p| [p[3], p[0], p[1], p[2]])
                    .collect();
                (argb, w as i32, h as i32)
            }
            None => (vec![], 0, 0),
        };
        DeadlockTray { argb, icon_w, icon_h, shared }
    }

    pub fn run(shared: Arc<crate::config::SharedBools>) {
        ksni::TrayService::new(build_tray(shared)).spawn();

        // Keep the main thread alive. The ksni daemon thread owns the icon;
        // Quit exits the process directly.
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod windows {
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;
    use tray_icon::{Icon, TrayIconBuilder};

    fn load_icon() -> Icon {
        if let Some((rgba, w, h)) = super::load_rgba() {
            if let Ok(icon) = Icon::from_rgba(rgba, w, h) {
                return icon;
            }
        }
        // Fallback: small blue square
        let size = 32u32;
        let rgba: Vec<u8> = (0..(size * size))
            .flat_map(|_| [40u8, 120u8, 200u8, 255u8])
            .collect();
        Icon::from_rgba(rgba, size, size).expect("fallback icon failed")
    }

    pub fn run(shared: Arc<crate::config::SharedBools>) {
        use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
        use crate::config::HeroPortraitStyle;

        let launch_item = CheckMenuItem::new(
            "Launch Game on Start",
            true,
            shared.launch_game_on_start.load(Ordering::Relaxed),
            None,
        );
        let exit_item = CheckMenuItem::new(
            "Exit When Game Closes",
            true,
            shared.exit_when_game_closes.load(Ordering::Relaxed),
            None,
        );
        let hero_item = CheckMenuItem::new(
            "Show Hero Image",
            true,
            shared.show_hero_image.load(Ordering::Relaxed),
            None,
        );
        let statlocker_item = CheckMenuItem::new(
            "Show Statlocker Button",
            true,
            shared.show_statlocker_button.load(Ordering::Relaxed),
            None,
        );

        let current_style = HeroPortraitStyle::from_u8(
            shared.hero_portrait_style.load(Ordering::Relaxed)
        );
        let portrait_normal_item = CheckMenuItem::new(
            "Normal",
            true,
            current_style == HeroPortraitStyle::Normal,
            None,
        );
        let portrait_gloat_item = CheckMenuItem::new(
            "Gloat",
            true,
            current_style == HeroPortraitStyle::Gloat,
            None,
        );
        let portrait_critical_item = CheckMenuItem::new(
            "Critical",
            true,
            current_style == HeroPortraitStyle::Critical,
            None,
        );

        let portrait_menu = Submenu::new("Hero Portrait Style", true);
        portrait_menu
            .append_items(&[
                &portrait_normal_item,
                &portrait_gloat_item,
                &portrait_critical_item,
            ])
            .unwrap();

        let open_config_item = MenuItem::new("Open Config File", true, None);
        let open_log_item = MenuItem::new("Open Log File", true, None);
        let check_updates_item = MenuItem::new("Check for Updates", true, None);
        let latest_changes_item = MenuItem::new("Latest Changes", true, None);
        let source_code_item = MenuItem::new("Source Code", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        let settings_menu = Submenu::new("Settings", true);
        settings_menu
            .append_items(&[
                &launch_item,
                &exit_item,
                &PredefinedMenuItem::separator(),
                &hero_item,
                &statlocker_item,
                &portrait_menu,
                &PredefinedMenuItem::separator(),
                &open_config_item,
                &open_log_item,
            ])
            .unwrap();

        let launch_id = launch_item.id().clone();
        let exit_id = exit_item.id().clone();
        let hero_id = hero_item.id().clone();
        let statlocker_id = statlocker_item.id().clone();
        let portrait_normal_id = portrait_normal_item.id().clone();
        let portrait_gloat_id = portrait_gloat_item.id().clone();
        let portrait_critical_id = portrait_critical_item.id().clone();
        let open_config_id = open_config_item.id().clone();
        let open_log_id = open_log_item.id().clone();
        let check_updates_id = check_updates_item.id().clone();
        let latest_changes_id = latest_changes_item.id().clone();
        let source_code_id = source_code_item.id().clone();
        let quit_id = quit_item.id().clone();

        let menu = Menu::new();
        menu.append(&settings_menu).unwrap();
        menu.append(&PredefinedMenuItem::separator()).unwrap();
        menu.append(&check_updates_item).unwrap();
        menu.append(&latest_changes_item).unwrap();
        menu.append(&source_code_item).unwrap();
        menu.append(&PredefinedMenuItem::separator()).unwrap();
        menu.append(&quit_item).unwrap();

        let _tray = TrayIconBuilder::new()
            .with_tooltip("Deadlock RPC")
            .with_icon(load_icon())
            .with_menu(Box::new(menu))
            .build()
            .expect("Failed to create tray icon");

        // Windows requires a Win32 message pump for the tray context menu to
        // appear. Without PeekMessage/DispatchMessage the hidden tray window
        // never processes WM_RBUTTONUP and the menu is never shown.
        unsafe {
            use winapi::um::winuser::{DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE};
            let mut msg: MSG = std::mem::zeroed();
            loop {
                while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                while let Ok(event) = MenuEvent::receiver().try_recv() {
                    if event.id == open_config_id {
                        let path = crate::config::config_path();
                        if let Some(p) = path.to_str() {
                            let _ = std::process::Command::new("cmd")
                                .args(["/c", "start", "", p])
                                .spawn();
                        }
                    } else if event.id == open_log_id {
                        if let Some(path) = crate::logger::log_path() {
                            if let Some(p) = path.to_str() {
                                let _ = std::process::Command::new("cmd")
                                    .args(["/c", "start", "", p])
                                    .spawn();
                            }
                        }
                    } else if event.id == check_updates_id {
                        std::thread::spawn(crate::updater::check_from_tray);
                    } else if event.id == latest_changes_id {
                        let _ = std::process::Command::new("cmd")
                            .args(["/c", "start", "", "https://github.com/HeyTariq/deadlock-rpc/releases/latest"])
                            .spawn();
                    } else if event.id == source_code_id {
                        let _ = std::process::Command::new("cmd")
                            .args(["/c", "start", "", "https://github.com/HeyTariq/deadlock-rpc"])
                            .spawn();
                    } else if event.id == quit_id {
                        std::process::exit(0);
                    } else if event.id == launch_id {
                        let new_val = !shared.launch_game_on_start.fetch_xor(true, Ordering::Relaxed);
                        launch_item.set_checked(new_val);
                        crate::config::set_config_bool("general", "launch_game_on_start", new_val);
                    } else if event.id == exit_id {
                        let new_val = !shared.exit_when_game_closes.fetch_xor(true, Ordering::Relaxed);
                        exit_item.set_checked(new_val);
                        crate::config::set_config_bool("general", "exit_when_game_closes", new_val);
                    } else if event.id == hero_id {
                        let new_val = !shared.show_hero_image.fetch_xor(true, Ordering::Relaxed);
                        hero_item.set_checked(new_val);
                        crate::config::set_config_bool("presence", "show_hero_image", new_val);
                        shared.settings_dirty.store(true, Ordering::Relaxed);
                    } else if event.id == statlocker_id {
                        let new_val = !shared.show_statlocker_button.fetch_xor(true, Ordering::Relaxed);
                        statlocker_item.set_checked(new_val);
                        crate::config::set_config_bool("presence", "show_statlocker_button", new_val);
                        shared.settings_dirty.store(true, Ordering::Relaxed);
                    } else if event.id == portrait_normal_id {
                        shared.hero_portrait_style.store(HeroPortraitStyle::Normal.as_u8(), Ordering::Relaxed);
                        portrait_normal_item.set_checked(true);
                        portrait_gloat_item.set_checked(false);
                        portrait_critical_item.set_checked(false);
                        crate::config::set_config_string("presence", "hero_portrait_style", "normal");
                        shared.settings_dirty.store(true, Ordering::Relaxed);
                    } else if event.id == portrait_gloat_id {
                        shared.hero_portrait_style.store(HeroPortraitStyle::Gloat.as_u8(), Ordering::Relaxed);
                        portrait_normal_item.set_checked(false);
                        portrait_gloat_item.set_checked(true);
                        portrait_critical_item.set_checked(false);
                        crate::config::set_config_string("presence", "hero_portrait_style", "gloat");
                        shared.settings_dirty.store(true, Ordering::Relaxed);
                    } else if event.id == portrait_critical_id {
                        shared.hero_portrait_style.store(HeroPortraitStyle::Critical.as_u8(), Ordering::Relaxed);
                        portrait_normal_item.set_checked(false);
                        portrait_gloat_item.set_checked(false);
                        portrait_critical_item.set_checked(true);
                        crate::config::set_config_string("presence", "hero_portrait_style", "critical");
                        shared.settings_dirty.store(true, Ordering::Relaxed);
                    }
                }
                thread::sleep(Duration::from_millis(50));
            }
        }
    }
}
