use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// アプリ設定構造体
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppSettings {
    pub game_path: String,
    pub language: String,
}

// ゲーム情報構造体
#[derive(Serialize, Deserialize, Clone)]
pub struct GameInfo {
    pub found: bool,
    pub path: String,
    pub platform: String,
}

// MOD状態構造体
#[derive(Serialize, Deserialize, Clone)]
pub struct ModStatus {
    pub installed: bool,
    pub version: String,
    pub backup_exists: bool,
}

// デフォルト設定取得
pub fn default_settings() -> AppSettings {
    AppSettings {
        language: "en".to_string(),
        game_path: String::new(),
    }
}

// 設定格納ディレクトリ取得
pub fn app_config_dir() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        let base =
            std::env::var_os("APPDATA").ok_or_else(|| "APPDATA is not available".to_string())?;
        let dir = PathBuf::from(base).join("First Light Mod Manager");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let base = std::env::var_os("HOME").ok_or_else(|| "HOME is not available".to_string())?;
        let dir = PathBuf::from(base)
            .join(".config")
            .join("first-light-mod-manager");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        Ok(dir)
    }
}

// 設定ファイルパス取得
pub fn settings_path() -> Result<PathBuf, String> {
    Ok(app_config_dir()?.join("settings.json"))
}

// 設定ファイル読み込み
pub fn read_settings_file() -> AppSettings {
    let Ok(path) = settings_path() else {
        return default_settings();
    };
    let Ok(content) = fs::read_to_string(path) else {
        return default_settings();
    };
    let mut settings =
        serde_json::from_str::<AppSettings>(&content).unwrap_or_else(|_| default_settings());
    if settings.language != "en" && settings.language != "pt" {
        settings.language = "en".to_string();
    }
    settings
}

// 設定ファイル書き込み
pub fn write_settings_file(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path()?;
    let data = serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
}

// ゲームパス正規化
pub fn normalize_game_path(input: &str) -> Result<PathBuf, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("Game path is empty".to_string());
    }

    let mut path = PathBuf::from(trimmed);
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.eq_ignore_ascii_case("Runtime"))
        .unwrap_or(false)
    {
        path = path
            .parent()
            .ok_or_else(|| "Runtime folder has no parent game directory".to_string())?
            .to_path_buf();
    }

    let runtime = path.join("Runtime");
    let exe = path.join("Retail").join("007FirstLight.exe");
    if runtime.is_dir() && (runtime.join("chunk0.rpkg").is_file() || exe.is_file()) {
        return Ok(path);
    }

    Err("Selected folder is not a 007 First Light installation".to_string())
}
