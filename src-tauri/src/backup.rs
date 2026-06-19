use std::fs;
use std::path::Path;

// packagedefinition.txt のバックアップ
pub fn backup_packagedefinition_only(game_dir: &Path) -> Result<(), String> {
    let src = game_dir.join("Runtime").join("packagedefinition.txt");
    if !src.exists() {
        return Err("Runtime\\packagedefinition.txt was not found".to_string());
    }
    let backup_dir = game_dir.join(".flmm_backup");
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    fs::copy(&src, backup_dir.join("packagedefinition.txt")).map_err(|e| e.to_string())?;
    Ok(())
}

// packagedefinition.txt の復元
pub fn restore_packagedefinition(game_dir: &Path) -> Result<(), String> {
    let backup_file = game_dir.join(".flmm_backup").join("packagedefinition.txt");
    if !backup_file.exists() {
        return Err("Backup packagedefinition.txt was not found".to_string());
    }
    let dst = game_dir.join("Runtime").join("packagedefinition.txt");
    fs::copy(&backup_file, &dst).map_err(|e| e.to_string())?;
    Ok(())
}

// 旧形式バックアップの自動移行
pub fn migrate_legacy_backup(game_dir: &Path) -> Result<(), String> {
    let legacy = game_dir.join("Runtime_backup_original");
    let new_backup = game_dir.join(".flmm_backup");
    if !legacy.is_dir() || new_backup.exists() {
        return Ok(());
    }
    let legacy_pkg = legacy.join("packagedefinition.txt");
    if !legacy_pkg.exists() {
        return Ok(());
    }
    fs::create_dir_all(&new_backup).map_err(|e| e.to_string())?;
    fs::copy(&legacy_pkg, new_backup.join("packagedefinition.txt")).map_err(|e| e.to_string())?;
    fs::remove_dir_all(&legacy).map_err(|e| e.to_string())?;
    Ok(())
}
