use crate::{lifecycle::EXIT_REQUESTED, window_controls::activate_window};
use tauri::{
    Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};
use tracing::warn;

const TRAY_MENU_QUIT_ID: &str = "tray.quit_cardinal";

pub fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let handle = app.handle();

    let quit_item = MenuItemBuilder::with_id(TRAY_MENU_QUIT_ID, "Quit Cardinal")
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;

    let menu = MenuBuilder::new(app).item(&quit_item).build()?;

    let mut tray_builder = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Cardinal")
        .on_menu_event(|app, event| {
            if event.id.as_ref() == TRAY_MENU_QUIT_ID {
                EXIT_REQUESTED.store(true, std::sync::atomic::Ordering::Relaxed);
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, tray_event| {
            if let TrayIconEvent::Click { button, .. } = tray_event {
                if matches!(button, MouseButton::Left) {
                    let app_handle = tray.app_handle();
                    if let Some(window) = app_handle.get_webview_window("main") {
                        activate_window(&window);
                    } else {
                        warn!("Tray click requested but main window is unavailable");
                    }
                }
            }
        });

    let tray_icon_owned = handle
        .default_window_icon()
        .cloned()
        .map(|icon| icon.to_owned());
    if let Some(icon) = tray_icon_owned {
        tray_builder = tray_builder.icon(icon);
    }

    let tray_icon = tray_builder.build(app)?;
    let _ = Box::leak(Box::new(tray_icon));
    Ok(())
}
