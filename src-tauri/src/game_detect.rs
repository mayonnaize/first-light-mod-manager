use std::fs;
use std::path::PathBuf;
use crate::settings::normalize_game_path;

// JSONフィールド値抽出
pub fn extract_json_field(json: &str, field: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    value[field].as_str().map(|text| text.to_string())
}

// Steamゲームパス自動検出
pub fn find_steam_game() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let steam_key = hklm
            .open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam")
            .or_else(|_| hklm.open_subkey("SOFTWARE\\Valve\\Steam"))
            .ok()?;

        let steam_path: String = steam_key.get_value("InstallPath").ok()?;
        for lib in get_steam_library_paths(&steam_path) {
            for candidate in [
                format!("{lib}\\steamapps\\common\\007 First Light"),
                format!("{lib}\\steamapps\\common\\007FirstLight"),
                format!("{lib}\\steamapps\\common\\Project 007"),
            ] {
                if normalize_game_path(&candidate).is_ok() {
                    return Some(candidate);
                }
            }
        }
        None
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

// Steamライブラリフォルダ一覧取得
pub fn get_steam_library_paths(steam_path: &str) -> Vec<String> {
    let mut paths = vec![steam_path.to_string()];
    let vdf = format!("{steam_path}\\steamapps\\libraryfolders.vdf");
    if let Ok(content) = fs::read_to_string(&vdf) {
        for line in content.lines() {
            let line = line.trim();
            if line.contains("\"path\"") {
                let parts: Vec<&str> = line.splitn(4, '"').collect();
                if parts.len() >= 4 {
                    let path = parts[3].replace("\\\\", "\\");
                    if !path.is_empty() {
                        paths.push(path);
                    }
                }
            }
        }
    }
    paths
}

// Epic Gamesゲームパス自動検出
pub fn find_epic_game() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let epic_key = hklm
            .open_subkey("SOFTWARE\\WOW6432Node\\Epic Games\\EpicGamesLauncher")
            .or_else(|_| hklm.open_subkey("SOFTWARE\\Epic Games\\EpicGamesLauncher"))
            .ok()?;

        let base: String = epic_key.get_value("AppDataPath").ok()?;
        let manifests = PathBuf::from(&base).parent()?.join("Manifests");
        for entry in fs::read_dir(manifests).ok()? {
            let entry = entry.ok()?;
            let content = fs::read_to_string(entry.path()).ok()?;
            if content.contains("007") || content.contains("Project007") {
                if let Some(location) = extract_json_field(&content, "InstallLocation") {
                    if normalize_game_path(&location).is_ok() {
                        return Some(location);
                    }
                }
            }
        }
        None
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}
