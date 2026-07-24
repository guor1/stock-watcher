use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Application config persisted to disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// User's watchlist codes, e.g. ["sh600000", "sz000001"]
    pub watchlist: Vec<String>,
    /// Codes hidden from the main bar (indices or stocks)
    #[serde(default)]
    pub hidden_from_bar: Vec<String>,
    /// Boss key shortcut (default "Ctrl+`")
    pub boss_key: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            watchlist: vec![],
            hidden_from_bar: vec![],
            boss_key: "Ctrl+`".to_string(),
        }
    }
}

/// Get the config file path (in app data dir).
fn config_path(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_data_dir().expect("no app data dir");
    fs::create_dir_all(&dir).ok();
    dir.join("config.json")
}

/// Load config from disk. Returns default if not found.
pub fn load_config(app: &AppHandle) -> AppConfig {
    let path = config_path(app);
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

/// Save config to disk.
pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = config_path(app);
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}
