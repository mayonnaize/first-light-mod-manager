use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ─── Estruturas de dados ───────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
pub struct GameInfo {
    pub found: bool,
    pub path: String,
    pub platform: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModStatus {
    pub installed: bool,
    pub version: String,
    pub backup_exists: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NexusRelease {
    pub version: String,
    pub url: String,
    pub has_update: bool,
}

// ─── detect_game ──────────────────────────────────────────────────────────

#[tauri::command]
pub fn detect_game() -> GameInfo {
    if let Some(path) = find_steam_game() {
        return GameInfo { found: true, path, platform: "steam".into() };
    }
    if let Some(path) = find_epic_game() {
        return GameInfo { found: true, path, platform: "epic".into() };
    }
    GameInfo { found: false, path: String::new(), platform: "unknown".into() }
}

fn find_steam_game() -> Option<String> {
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
        let libs = get_steam_library_paths(&steam_path);

        for lib in &libs {
            let candidates = [
                format!("{}\\steamapps\\common\\007 First Light", lib),
                format!("{}\\steamapps\\common\\007FirstLight", lib),
                format!("{}\\steamapps\\common\\Project 007", lib),
            ];
            for c in &candidates {
                if Path::new(c).exists() {
                    return Some(c.clone());
                }
            }
        }
        None
    }
    #[cfg(not(target_os = "windows"))]
    None
}

fn get_steam_library_paths(steam_path: &str) -> Vec<String> {
    let mut paths = vec![steam_path.to_string()];
    let vdf = format!("{}\\steamapps\\libraryfolders.vdf", steam_path);
    if let Ok(content) = fs::read_to_string(&vdf) {
        for line in content.lines() {
            let line = line.trim();
            if line.contains("\"path\"") {
                let parts: Vec<&str> = line.splitn(4, '"').collect();
                if parts.len() >= 4 {
                    let p = parts[3].replace("\\\\", "\\");
                    if !p.is_empty() { paths.push(p); }
                }
            }
        }
    }
    paths
}

fn find_epic_game() -> Option<String> {
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
        if !manifests.exists() { return None; }

        for entry in fs::read_dir(&manifests).ok()? {
            let entry = entry.ok()?;
            let content = fs::read_to_string(entry.path()).ok()?;
            if content.contains("007") || content.contains("Project007") {
                if let Some(loc) = extract_json_field(&content, "InstallLocation") {
                    if Path::new(&loc).exists() { return Some(loc); }
                }
            }
        }
        None
    }
    #[cfg(not(target_os = "windows"))]
    None
}

fn extract_json_field(json: &str, field: &str) -> Option<String> {
    let key = format!("\"{}\"", field);
    let pos = json.find(&key)?;
    let after = &json[pos + key.len()..];
    let colon = after.find(':')?;
    let value_start = after[colon + 1..].trim_start();
    if value_start.starts_with('"') {
        let end = value_start[1..].find('"')?;
        Some(value_start[1..=end].replace("\\\\", "\\").replace("\\/", "/"))
    } else {
        None
    }
}

// ─── get_mod_status ───────────────────────────────────────────────────────

#[tauri::command]
pub fn get_mod_status(game_path: String) -> ModStatus {
    if game_path.is_empty() {
        return ModStatus { installed: false, version: String::new(), backup_exists: false };
    }
    let backup = PathBuf::from(&game_path).join("Runtime_backup_original");
    
    // Check if there are any active mods
    let mut installed = false;
    let mut version = String::new();
    
    if let Ok(mods) = list_mods(game_path.clone()) {
        installed = mods.iter().any(|m| m.active);
        if installed {
            version = "0.1.0".to_string(); // Fallback representation
        }
    }
    
    ModStatus { installed, version, backup_exists: backup.exists() }
}

#[tauri::command]
pub fn install_mod(game_path: String, mod_path: String, lang: String) -> Result<String, String> {
    let is_pt = lang == "pt";
    if game_path.is_empty() { 
        return Err(if is_pt { "Caminho do jogo não informado." } else { "Game path not provided." }.into()); 
    }
    if mod_path.is_empty()  { 
        return Err(if is_pt { "Caminho do mod não informado." } else { "Mod path not provided." }.into()); 
    }

    let game_dir   = PathBuf::from(&game_path);
    let runtime    = game_dir.join("Runtime");
    let backup     = game_dir.join("Runtime_backup_original");
    let mod_file   = PathBuf::from(&mod_path);

    if !mod_file.exists() {
        return Err(if is_pt { format!("Arquivo não encontrado: {}", mod_path) } else { format!("File not found: {}", mod_path) });
    }

    let ext = mod_file.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext != "rpkg" && ext != "zip" {
        return Err(if is_pt { "O arquivo deve ser .rpkg ou .zip" } else { "File must be .rpkg or .zip" }.into());
    }

    // Cria Runtime se não existir
    if !runtime.exists() {
        fs::create_dir_all(&runtime).map_err(|e| {
            if is_pt { format!("Erro ao criar pasta Runtime: {}", e) } else { format!("Error creating Runtime directory: {}", e) }
        })?;
    }

    // Backup (só se ainda não existe)
    if !backup.exists() {
        copy_dir_all(&runtime, &backup).map_err(|e| {
            if is_pt { format!("Erro no backup: {}", e) } else { format!("Error creating backup: {}", e) }
        })?;
    }

    if ext == "rpkg" {
        let dest = runtime.join(mod_file.file_name().unwrap());
        fs::copy(&mod_file, &dest).map_err(|e| {
            if is_pt { format!("Erro ao copiar RPKG: {}", e) } else { format!("Error copying RPKG: {}", e) }
        })?;
        update_package_definition(&runtime, mod_file.file_name().unwrap().to_str().unwrap())?;
    } else {
        extract_rpkg_from_zip(&mod_file, &runtime)?;
    }

    let version = "0.1.0";
    fs::write(game_dir.join(".flmm_installed"), version)
        .map_err(|e| {
            if is_pt { format!("Erro ao gravar versão: {}", e) } else { format!("Error writing version: {}", e) }
        })?;

    Ok(if is_pt { "Mod instalado com sucesso!" } else { "Mod installed successfully!" }.into())
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() { return Ok(()); }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

fn update_package_definition(runtime: &Path, rpkg_name: &str) -> Result<(), String> {
    let pkg_def = runtime.join("packagedefinition.txt");
    let mut content = if pkg_def.exists() {
        fs::read_to_string(&pkg_def).map_err(|e| e.to_string())?
    } else {
        String::new()
    };

    let chunk = rpkg_name.trim_end_matches(".rpkg");
    let entry = format!("@include {}", chunk);
    if !content.contains(&entry) {
        if !content.is_empty() && !content.ends_with('\n') { content.push('\n'); }
        content.push_str(&entry);
        content.push('\n');
        fs::write(&pkg_def, &content).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn extract_rpkg_from_zip(zip_path: &Path, dest: &Path) -> Result<(), String> {
    use std::io::Read;
    let file = fs::File::open(zip_path).map_err(|e| format!("Error opening ZIP: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP archive: {}", e))?;

    // First search for mod.json anywhere inside the zip
    let mut metadata_content: Option<String> = None;
    for i in 0..archive.len() {
        if let Ok(mut zf) = archive.by_index(i) {
            if zf.name().ends_with("mod.json") {
                let mut buf = String::new();
                if zf.read_to_string(&mut buf).is_ok() {
                    metadata_content = Some(buf);
                    break;
                }
            }
        }
    }

    let mut rpkg_files = Vec::new();

    for i in 0..archive.len() {
        let mut zf = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = zf.name().to_string();
        if name.ends_with(".rpkg") {
            let fname = Path::new(&name).file_name().unwrap();
            let out = dest.join(fname);
            let mut buf = Vec::new();
            zf.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            fs::write(&out, &buf).map_err(|e| e.to_string())?;
            update_package_definition(dest, fname.to_str().unwrap())?;
            rpkg_files.push(fname.to_str().unwrap().to_string());
        }
    }

    // Write companion metadata files
    if let Some(ref meta) = metadata_content {
        for rpkg in &rpkg_files {
            let id = rpkg.trim_end_matches(".rpkg");
            let meta_path = dest.join(format!("{}.metadata.json", id));
            let _ = fs::write(meta_path, meta);
        }
    }

    Ok(())
}

#[tauri::command]
pub fn uninstall_mod(game_path: String, lang: String) -> Result<String, String> {
    let is_pt = lang == "pt";
    if game_path.is_empty() { 
        return Err(if is_pt { "Caminho do jogo não informado." } else { "Game path not provided." }.into()); 
    }

    let game_dir = PathBuf::from(&game_path);
    let runtime  = game_dir.join("Runtime");
    let backup   = game_dir.join("Runtime_backup_original");
    let marker   = game_dir.join(".flmm_installed");

    if !backup.exists() {
        return Err(if is_pt { "Backup não encontrado. Não é possível desinstalar com segurança." } else { "Backup not found. Cannot safely uninstall." }.into());
    }

    if runtime.exists() {
        fs::remove_dir_all(&runtime).map_err(|e| {
            if is_pt { format!("Erro ao remover pasta Runtime: {}", e) } else { format!("Error removing Runtime directory: {}", e) }
        })?;
    }

    copy_dir_all(&backup, &runtime).map_err(|e| {
        if is_pt { format!("Erro ao restaurar backup: {}", e) } else { format!("Error restoring backup: {}", e) }
    })?;
    fs::remove_dir_all(&backup).map_err(|e| {
        if is_pt { format!("Erro ao remover backup: {}", e) } else { format!("Error removing backup directory: {}", e) }
    })?;
    let _ = fs::remove_file(&marker);

    Ok(if is_pt { "Mods desinstalados! Arquivos originais restaurados." } else { "Mods uninstalled! Original game files restored." }.into())
}

// ─── check_updates ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn check_updates(current_version: String, mod_id: String, api_key: Option<String>) -> Result<NexusRelease, String> {
    if mod_id == "0" || mod_id.is_empty() {
        return Ok(NexusRelease {
            version: "0.1.0".into(),
            url: "https://www.nexusmods.com/007firstlight".into(),
            has_update: false,
        });
    }

    let url = format!("https://api.nexusmods.com/v1/games/007firstlight/mods/{}.json", mod_id);
    let client = reqwest::Client::new();

    let mut builder = client
        .get(&url)
        .header("User-Agent", "First-Light-Mod-Manager/0.1.0");

    if let Some(ref key) = api_key {
        if !key.trim().is_empty() {
            builder = builder.header("apikey", key.trim());
        }
    }

    let resp = builder
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Ok(NexusRelease {
            version: "0.1.0".into(),
            url: format!("https://www.nexusmods.com/007firstlight/mods/{}", mod_id),
            has_update: false,
        });
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let latest = json["version"].as_str().unwrap_or("0.1.0").to_string();
    let has_update = !current_version.is_empty() && latest != current_version;

    Ok(NexusRelease {
        version: latest,
        url: format!("https://www.nexusmods.com/007firstlight/mods/{}", mod_id),
        has_update,
    })
}

// ─── open_game_folder ────────────────────────────────────────────────────

#[tauri::command]
pub fn open_game_folder(game_path: String) -> Result<(), String> {
    if game_path.is_empty() { return Err("No game directory configured.".into()); }
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer")
        .arg(&game_path)
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

// ─── Mod Toggle List implementation ──────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone)]
pub struct ModInfo {
    pub id: String,
    pub filename: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub version: String,
    pub active: bool,
}

fn remove_package_definition(runtime: &Path, rpkg_name: &str) -> Result<(), String> {
    let pkg_def = runtime.join("packagedefinition.txt");
    if !pkg_def.exists() { return Ok(()); }
    let content = fs::read_to_string(&pkg_def).map_err(|e| e.to_string())?;

    let chunk = rpkg_name.trim_end_matches(".rpkg");
    let entry = format!("@include {}", chunk);

    let lines: Vec<&str> = content.lines()
        .filter(|line| line.trim() != entry)
        .collect();

    let mut new_content = lines.join("\n");
    if !new_content.is_empty() {
        new_content.push('\n');
    }
    fs::write(&pkg_def, &new_content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_mods(game_path: String) -> Result<Vec<ModInfo>, String> {
    if game_path.is_empty() {
        return Ok(Vec::new());
    }

    let runtime = PathBuf::from(&game_path).join("Runtime");
    let backup = PathBuf::from(&game_path).join("Runtime_backup_original");

    if !runtime.exists() {
        return Ok(Vec::new());
    }

    let pkg_def = runtime.join("packagedefinition.txt");
    let includes_content = if pkg_def.exists() {
        fs::read_to_string(&pkg_def).unwrap_or_default()
    } else {
        String::new()
    };

    let mut mods = Vec::new();

    if let Ok(entries) = fs::read_dir(&runtime) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rpkg") {
                    let filename = path.file_name().unwrap().to_str().unwrap().to_string();
                    let id = filename.trim_end_matches(".rpkg").to_string();

                    if backup.exists() {
                        let original_file = backup.join(&filename);
                        if original_file.exists() {
                            continue;
                        }
                    } else {
                        continue;
                    }

                    let meta_file = runtime.join(format!("{}.metadata.json", id));
                    let mut name = id.clone();
                    let mut author = String::new();
                    let mut description = String::new();
                    let mut version = String::new();

                    if meta_file.exists() {
                        if let Ok(content) = fs::read_to_string(&meta_file) {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                                name = json["name"].as_str().unwrap_or(&id).to_string();
                                author = json["author"].as_str().unwrap_or("").to_string();
                                description = json["description"].as_str().unwrap_or("").to_string();
                                version = json["version"].as_str().unwrap_or("").to_string();
                            }
                        }
                    }

                    let entry_str = format!("@include {}", id);
                    let active = includes_content.lines().any(|line| line.trim() == entry_str);

                    mods.push(ModInfo {
                        id,
                        filename,
                        name,
                        author,
                        description,
                        version,
                        active,
                    });
                }
            }
        }
    }

    Ok(mods)
}

#[tauri::command]
pub fn toggle_mod(game_path: String, mod_id: String, active: bool) -> Result<(), String> {
    if game_path.is_empty() || mod_id.is_empty() {
        return Err("Invalid parameters.".into());
    }

    let runtime = PathBuf::from(&game_path).join("Runtime");
    if active {
        update_package_definition(&runtime, &format!("{}.rpkg", mod_id))?;
    } else {
        remove_package_definition(&runtime, &format!("{}.rpkg", mod_id))?;
    }

    Ok(())
}

#[tauri::command]
pub fn delete_mod(game_path: String, mod_id: String) -> Result<(), String> {
    if game_path.is_empty() || mod_id.is_empty() {
        return Err("Invalid parameters.".into());
    }

    let runtime = PathBuf::from(&game_path).join("Runtime");
    let mod_file = runtime.join(format!("{}.rpkg", mod_id));
    let meta_file = runtime.join(format!("{}.metadata.json", mod_id));

    let _ = remove_package_definition(&runtime, &format!("{}.rpkg", mod_id));

    if mod_file.exists() {
        fs::remove_file(mod_file).map_err(|e| e.to_string())?;
    }
    if meta_file.exists() {
        let _ = fs::remove_file(meta_file);
    }

    Ok(())
}
