use std::path::Path;
use crate::settings::{
    read_settings_file, write_settings_file, normalize_game_path, AppSettings, GameInfo, ModStatus,
};
use crate::game_detect::{find_steam_game, find_epic_game};
use crate::mod_manager::{
    install_mod_internal, uninstall_mod_internal, list_mods_internal,
    toggle_mod_internal, delete_mod_internal, inspect_mod_file,
    ModInfo, ModPreview,
};

// 設定読み込み
#[tauri::command]
pub async fn load_settings() -> Result<AppSettings, String> {
    Ok(read_settings_file())
}

// 設定保存
#[tauri::command]
pub async fn save_settings(mut settings: AppSettings) -> Result<AppSettings, String> {
    if !settings.game_path.trim().is_empty() {
        settings.game_path = normalize_game_path(&settings.game_path)?
            .to_string_lossy()
            .to_string();
    }
    if settings.language != "en" && settings.language != "pt" {
        settings.language = "en".to_string();
    }
    write_settings_file(&settings)?;
    Ok(settings)
}

// MODファイル検査
#[tauri::command]
pub async fn inspect_mod(
    mod_path: String,
    game_path: Option<String>,
) -> Result<ModPreview, String> {
    let runtime = match game_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        Some(path) => Some(normalize_game_path(path)?.join("Runtime")),
        None => None,
    };
    inspect_mod_file(Path::new(&mod_path), runtime.as_deref())
}

// ゲームパス自動検出
#[tauri::command]
pub async fn detect_game() -> GameInfo {
    let settings = read_settings_file();
    if !settings.game_path.trim().is_empty() {
        if let Ok(path) = normalize_game_path(&settings.game_path) {
            return GameInfo {
                found: true,
                path: path.to_string_lossy().to_string(),
                platform: "manual".into(),
            };
        }
    }

    if let Some(path) = find_steam_game() {
        return GameInfo {
            found: true,
            path,
            platform: "steam".into(),
        };
    }
    if let Some(path) = find_epic_game() {
        return GameInfo {
            found: true,
            path,
            platform: "epic".into(),
        };
    }
    GameInfo {
        found: false,
        path: String::new(),
        platform: "unknown".into(),
    }
}

// MOD導入状態取得
#[tauri::command]
pub async fn get_mod_status(game_path: String) -> ModStatus {
    if game_path.is_empty() {
        return ModStatus {
            installed: false,
            version: String::new(),
            backup_exists: false,
        };
    }

    let Ok(game_dir) = normalize_game_path(&game_path) else {
        return ModStatus {
            installed: false,
            version: String::new(),
            backup_exists: false,
        };
    };

    let backup_exists = game_dir.join(".flmm_backup").exists()
        || game_dir.join("Runtime_backup_original").exists();
    let installed = list_mods_internal(&game_dir.to_string_lossy())
        .map(|mods| mods.iter().any(|item| item.active))
        .unwrap_or(false);

    ModStatus {
        installed,
        version: if installed {
            "managed".to_string()
        } else {
            String::new()
        },
        backup_exists,
    }
}

// MODインストール
#[tauri::command]
pub async fn install_mod(
    game_path: String,
    mod_path: String,
    lang: String,
) -> Result<String, String> {
    install_mod_internal(&game_path, &mod_path, &lang)
}

// MODアンインストール
#[tauri::command]
pub async fn uninstall_mod(game_path: String, lang: String) -> Result<String, String> {
    uninstall_mod_internal(&game_path, &lang)
}

// バックアップ削除
#[tauri::command]
pub async fn delete_backup(game_path: String) -> Result<(), String> {
    let game_dir = normalize_game_path(&game_path)?;
    let new_backup = game_dir.join(".flmm_backup");
    if new_backup.exists() {
        std::fs::remove_dir_all(new_backup).map_err(|e| e.to_string())?;
    }
    let legacy = game_dir.join("Runtime_backup_original");
    if legacy.exists() {
        std::fs::remove_dir_all(legacy).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ゲームフォルダ展開
#[tauri::command]
pub async fn open_game_folder(game_path: String) -> Result<(), String> {
    let game_dir = normalize_game_path(&game_path)?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(game_dir)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// MOD一覧取得
#[tauri::command]
pub async fn list_mods(game_path: String) -> Result<Vec<ModInfo>, String> {
    list_mods_internal(&game_path)
}

// MOD有効/無効トグル
#[tauri::command]
pub async fn toggle_mod(game_path: String, mod_id: String, active: bool) -> Result<(), String> {
    toggle_mod_internal(&game_path, &mod_id, active)
}

// MOD削除
#[tauri::command]
pub async fn delete_mod(game_path: String, mod_id: String) -> Result<(), String> {
    delete_mod_internal(&game_path, &mod_id)
}
