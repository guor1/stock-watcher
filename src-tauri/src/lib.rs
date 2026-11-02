mod quote;
mod store;

use std::str::FromStr;
use store::{AppConfig, save_config as store_save, load_config};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

/// Fetch quotes for the given codes from Tencent Finance.
#[tauri::command]
async fn fetch_quotes(codes: Vec<String>) -> Result<Vec<quote::Quote>, String> {
    quote::fetch_quotes(codes).await
}

/// Fetch whole-market turnover (今日实时成交额 / 较昨日变动) from 同花顺.
#[tauri::command]
async fn fetch_market_turnover() -> Result<quote::MarketTurnover, String> {
    quote::fetch_market_turnover().await
}

/// Return the persisted config.
#[tauri::command]
fn get_config(app: AppHandle) -> AppConfig {
    load_config(&app)
}

/// Persist config and notify the main window to refresh.
#[tauri::command]
fn save_config(app: AppHandle, config: AppConfig) -> Result<(), String> {
    store_save(&app, &config)?;
    let _ = app.emit("config-updated", ());
    Ok(())
}

/// Open (or focus) the settings window.
#[tauri::command]
fn open_settings(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("settings") {
        let _ = win.unminimize();
        win.show().map_err(|e| e.to_string())?;
        let _ = win.set_focus();
        Ok(())
    } else {
        Err("settings window not found".into())
    }
}

/// Hide the settings window (keeps it alive for reopening).
#[tauri::command]
fn close_settings(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("settings") {
        win.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Bring a window back to a normal, visible, focused state.
///
/// `show()` alone is not enough on Windows: `IsWindowVisible` (what
/// `is_visible()` reads) stays `true` for a minimized window, and
/// `ShowWindow(SW_SHOW)` shows a still-minimized window without restoring it.
fn reveal_window(app: &AppHandle, label: &str) {
    if let Some(win) = app.get_webview_window(label) {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
    }
}

/// Toggle main window visibility (used by the boss key and the tray icon).
fn toggle_main(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let minimized = win.is_minimized().unwrap_or(false);
        match win.is_visible() {
            // A minimized window still reports as visible — treat it as hidden.
            Ok(true) if !minimized => {
                let _ = win.hide();
            }
            _ => reveal_window(app, "main"),
        }
    }
}

/// Parse a boss-key string like "Ctrl+`" into a Shortcut, with a safe fallback.
fn parse_boss_key(s: &str) -> Shortcut {
    if let Ok(sc) = Shortcut::from_str(s) {
        return sc;
    }
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyK)
}

/// Build system tray icon with right-click menu.
#[cfg(desktop)]
fn build_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    use tauri::tray::TrayIconBuilder;

    let show = MenuItemBuilder::with_id("show", "显示/隐藏").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "设置").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&settings)
        .separator()
        .item(&quit)
        .build()?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("no default window icon")?;

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("盯盘小助手")
        .menu(&menu)
        // Left click toggles the bar; the menu stays on right click only.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "show" => toggle_main(app),
                "settings" => reveal_window(app, "settings"),
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_main(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            // Intercept settings window close → hide instead of destroy
            if let Some(settings_win) = app.get_webview_window("settings") {
                let win_clone = settings_win.clone();
                settings_win.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            // System tray icon (like WeChat's notification-area icon)
            #[cfg(desktop)]
            build_tray(app.handle())?;

            let handle = app.handle();
            let config = load_config(handle);
            let boss = parse_boss_key(&config.boss_key);
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                if let Err(e) = app.global_shortcut().register(boss) {
                    eprintln!("Failed to register boss key: {}", e);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            fetch_quotes,
            fetch_market_turnover,
            get_config,
            save_config,
            open_settings,
            close_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
