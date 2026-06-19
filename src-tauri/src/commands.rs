use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppSettings {
    pub game_path: String,
    pub language: String,
    pub nexus_api_key: String,
    pub nexus_mod_id: String,
    pub auto_check_updates: bool,
}

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

#[derive(Serialize, Deserialize, Clone)]
pub struct ModInfo {
    pub id: String,
    pub filename: String,
    pub original_filename: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub version: String,
    pub active: bool,
    pub chunk: u32,
    pub patch: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModPreview {
    pub file_name: String,
    pub package_type: String,
    pub installable: bool,
    pub rpkg_files: Vec<PreviewRpkg>,
    pub has_packagedefinition: bool,
    pub has_metadata: bool,
    pub warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PreviewRpkg {
    pub original_name: String,
    pub target_name: String,
    pub chunk: u32,
    pub requested_patch: u32,
    pub target_patch: u32,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct StoredModMetadata {
    name: String,
    author: String,
    description: String,
    version: String,
    original_filename: String,
    installed_filename: String,
    source_package: String,
    installed_at: String,
}

#[derive(Clone)]
struct PendingRpkg {
    original_name: String,
    data: Vec<u8>,
    size: u64,
    chunk: u32,
    requested_patch: u32,
}

#[derive(Clone)]
struct AssignedRpkg {
    pending: PendingRpkg,
    target_name: String,
    target_patch: u32,
}

#[derive(Debug)]
struct PackageDefinition {
    content: String,
    encrypted: bool,
}

const XTEA_KEYS: [u32; 4] = [0x71482CF0, 0x5FDC4B9F, 0x86CE569D, 0x0509FC1E];
const XTEA_DELTA: u32 = 0x61C88647;
const XTEA_SUM: u32 = 0xC6EF3720;
// 007 First Light の packagedefinition.txt 先頭16バイト (暗号化識別ヘッダー)
const XTEA_HEADER: [u8; 16] = [
    0xB7, 0xE2, 0xEA, 0x00, 0x54, 0x5B, 0x6B, 0x87, 0x11, 0xBD, 0x6F, 0xE8, 0x4D, 0x6A, 0xD4, 0xBF,
];
// MODパッチ番号の開始値 (公式パッチと衝突しないよう大きく離す)
const MOD_PATCH_START: u32 = 100;

fn default_settings() -> AppSettings {
    AppSettings {
        language: "en".to_string(),
        nexus_mod_id: "0".to_string(),
        auto_check_updates: true,
        ..AppSettings::default()
    }
}

fn app_config_dir() -> Result<PathBuf, String> {
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

fn settings_path() -> Result<PathBuf, String> {
    Ok(app_config_dir()?.join("settings.json"))
}

fn read_settings_file() -> AppSettings {
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
    if settings.nexus_mod_id.trim().is_empty() {
        settings.nexus_mod_id = "0".to_string();
    }
    settings
}

fn write_settings_file(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path()?;
    let data = serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
}

fn normalize_game_path(input: &str) -> Result<PathBuf, String> {
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

#[tauri::command]
pub async fn load_settings() -> Result<AppSettings, String> {
    Ok(read_settings_file())
}

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
    if settings.nexus_mod_id.trim().is_empty() {
        settings.nexus_mod_id = "0".to_string();
    }
    write_settings_file(&settings)?;
    Ok(settings)
}

fn decrypt_block_xtea(a: &mut u32, b: &mut u32, keys: &[u32; 4]) {
    let mut sum: u32 = XTEA_SUM;
    for _ in 0..32 {
        *b = b.wrapping_sub(
            ((*a << 4) ^ (*a >> 5)).wrapping_add(*a)
                ^ sum.wrapping_add(keys[((sum >> 11) & 3) as usize]),
        );
        sum = sum.wrapping_add(XTEA_DELTA);
        *a = a.wrapping_sub(
            ((*b << 4) ^ (*b >> 5)).wrapping_add(*b) ^ sum.wrapping_add(keys[(sum & 3) as usize]),
        );
    }
}

fn encrypt_block_xtea(a: &mut u32, b: &mut u32, keys: &[u32; 4]) {
    let mut sum: u32 = 0;
    for _ in 0..32 {
        *a = a.wrapping_add(
            ((*b << 4) ^ (*b >> 5)).wrapping_add(*b) ^ sum.wrapping_add(keys[(sum & 3) as usize]),
        );
        sum = sum.wrapping_sub(XTEA_DELTA);
        *b = b.wrapping_add(
            ((*a << 4) ^ (*a >> 5)).wrapping_add(*a)
                ^ sum.wrapping_add(keys[((sum >> 11) & 3) as usize]),
        );
    }
}

fn decrypt_buffer(data: &[u8]) -> Vec<u8> {
    let mut decrypted = Vec::with_capacity(data.len());
    for block in data.chunks_exact(8) {
        let mut a = u32::from_le_bytes([block[0], block[1], block[2], block[3]]);
        let mut b = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);
        decrypt_block_xtea(&mut a, &mut b, &XTEA_KEYS);
        decrypted.extend_from_slice(&a.to_le_bytes());
        decrypted.extend_from_slice(&b.to_le_bytes());
    }
    while decrypted.last() == Some(&0) {
        decrypted.pop();
    }
    decrypted
}

fn encrypt_buffer(data: &[u8]) -> Vec<u8> {
    let mut padded = data.to_vec();
    while !padded.len().is_multiple_of(8) {
        padded.push(0);
    }

    let mut encrypted = Vec::with_capacity(padded.len());
    for block in padded.chunks_exact(8) {
        let mut a = u32::from_le_bytes([block[0], block[1], block[2], block[3]]);
        let mut b = u32::from_le_bytes([block[4], block[5], block[6], block[7]]);
        encrypt_block_xtea(&mut a, &mut b, &XTEA_KEYS);
        encrypted.extend_from_slice(&a.to_le_bytes());
        encrypted.extend_from_slice(&b.to_le_bytes());
    }
    encrypted
}

fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB88320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

// バイト列を文字列にデコード (Latin-1 / ISO 8859-1 固定)
fn decode_to_string(data: Vec<u8>) -> String {
    data.iter().map(|&b| b as char).collect()
}

fn read_packagedefinition(runtime: &Path) -> Result<PackageDefinition, String> {
    let pkg_def = runtime.join("packagedefinition.txt");
    if !pkg_def.exists() {
        return Err("Runtime\\packagedefinition.txt was not found".to_string());
    }

    let raw = fs::read(&pkg_def).map_err(|e| e.to_string())?;

    // UTF-8 BOM 除去
    let data = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        raw[3..].to_vec()
    } else {
        raw
    };

    if data.len() >= 20 && data[..16] == XTEA_HEADER {
        let decrypted = decrypt_buffer(&data[20..]);
        let expected_crc = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let actual_crc = crc32_ieee(&decrypted);
        if expected_crc != actual_crc {
            return Err("packagedefinition.txt checksum mismatch".to_string());
        }
        let content = decode_to_string(decrypted);
        Ok(PackageDefinition {
            content,
            encrypted: true,
        })
    } else {
        let content = decode_to_string(data);
        Ok(PackageDefinition {
            content,
            encrypted: false,
        })
    }
}

fn write_packagedefinition(runtime: &Path, definition: &PackageDefinition) -> Result<(), String> {
    let pkg_def = runtime.join("packagedefinition.txt");
    let content_bytes = definition.content.as_bytes();
    if definition.encrypted {
        let encrypted = encrypt_buffer(content_bytes);
        let crc = crc32_ieee(content_bytes);
        let mut file_data = Vec::with_capacity(20 + encrypted.len());
        file_data.extend_from_slice(&XTEA_HEADER);
        file_data.extend_from_slice(&crc.to_le_bytes());
        file_data.extend_from_slice(&encrypted);
        fs::write(pkg_def, file_data).map_err(|e| e.to_string())
    } else {
        fs::write(pkg_def, content_bytes).map_err(|e| e.to_string())
    }
}

fn detect_line_ending(content: &str) -> &'static str {
    if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

fn set_patchlevel_in_line(line: &str, level: u32) -> String {
    let Some(start) = line.find("patchlevel=") else {
        return format!("{line} patchlevel={level}");
    };
    let value_start = start + "patchlevel=".len();
    let value_end = line[value_start..]
        .find(char::is_whitespace)
        .map(|offset| value_start + offset)
        .unwrap_or(line.len());
    format!("{}{}{}", &line[..value_start], level, &line[value_end..])
}

fn patchlevels_by_partition(runtime: &Path) -> Result<HashMap<u32, u32>, String> {
    let mut max_patch_by_chunk: HashMap<u32, u32> = HashMap::new();
    if !runtime.is_dir() {
        return Ok(max_patch_by_chunk);
    }

    for entry in fs::read_dir(runtime).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some((chunk, patch, active)) = parse_patch_file_name(&name) {
            if active {
                max_patch_by_chunk
                    .entry(chunk)
                    .and_modify(|value| *value = (*value).max(patch))
                    .or_insert(patch);
            }
        }
    }

    for value in max_patch_by_chunk.values_mut() {
        if *value > 0 {
            *value = (*value).max(MOD_PATCH_START);
        }
    }
    Ok(max_patch_by_chunk)
}

fn apply_patchlevels(content: &str, patchlevels: &HashMap<u32, u32>) -> Result<String, String> {
    let line_ending = detect_line_ending(content);
    let had_trailing_newline = content.ends_with('\n');
    let mut partition_index = 0_u32;
    let mut touched = HashSet::new();
    let mut lines = Vec::new();

    for line in content.lines() {
        if line.trim_start().starts_with("@partition") {
            let level = patchlevels.get(&partition_index).copied().unwrap_or(0);
            lines.push(set_patchlevel_in_line(line, level));
            touched.insert(partition_index);
            partition_index += 1;
        } else {
            lines.push(line.to_string());
        }
    }

    for chunk in patchlevels.keys() {
        if !touched.contains(chunk) {
            return Err(format!(
                "packagedefinition.txt has no partition for chunk{chunk}"
            ));
        }
    }

    let mut next = lines.join(line_ending);
    if had_trailing_newline {
        next.push_str(line_ending);
    }
    Ok(next)
}

fn refresh_package_definition(runtime: &Path) -> Result<(), String> {
    let mut definition = read_packagedefinition(runtime)?;
    let patchlevels = patchlevels_by_partition(runtime)?;
    definition.content = apply_patchlevels(&definition.content, &patchlevels)?;
    write_packagedefinition(runtime, &definition)
}

fn parse_patch_file_name(name: &str) -> Option<(u32, u32, bool)> {
    let lower = name.to_ascii_lowercase();
    let (stem, active) = if let Some(stem) = lower.strip_suffix(".rpkg") {
        (stem, true)
    } else if let Some(stem) = lower.strip_suffix(".rpkg.disabled") {
        (stem, false)
    } else {
        return None;
    };

    let rest = stem.strip_prefix("chunk")?;
    let patch_pos = rest.find("patch")?;
    let chunk = rest[..patch_pos].parse::<u32>().ok()?;
    let patch = rest[patch_pos + "patch".len()..].parse::<u32>().ok()?;
    Some((chunk, patch, active))
}

fn target_patch_start(_chunk: u32) -> u32 {
    MOD_PATCH_START
}

fn used_patch_slots(runtime: Option<&Path>) -> Result<HashSet<(u32, u32)>, String> {
    let mut used = HashSet::new();
    let Some(runtime) = runtime else {
        return Ok(used);
    };
    if !runtime.is_dir() {
        return Ok(used);
    }

    // Runtime 内の既存パッチを収集 (公式パッチも Mod パッチも含む)
    for entry in fs::read_dir(runtime).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some((chunk, patch, _)) = parse_patch_file_name(&name) {
            used.insert((chunk, patch));
        }
    }

    Ok(used)
}

fn assign_rpkg_targets(
    pending: Vec<PendingRpkg>,
    runtime: Option<&Path>,
) -> Result<Vec<AssignedRpkg>, String> {
    let mut used = used_patch_slots(runtime)?;
    let mut assigned = Vec::new();

    for item in pending {
        let start = target_patch_start(item.chunk);
        let mut patch = item.requested_patch.max(start);
        while used.contains(&(item.chunk, patch)) {
            patch += 1;
        }
        used.insert((item.chunk, patch));
        assigned.push(AssignedRpkg {
            target_name: format!("chunk{}patch{}.rpkg", item.chunk, patch),
            target_patch: patch,
            pending: item,
        });
    }

    Ok(assigned)
}

fn metadata_from_json(content: &str) -> StoredModMetadata {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return StoredModMetadata::default();
    };

    StoredModMetadata {
        name: value["name"].as_str().unwrap_or_default().to_string(),
        author: value["author"].as_str().unwrap_or_default().to_string(),
        description: value["description"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        version: value["version"].as_str().unwrap_or_default().to_string(),
        ..StoredModMetadata::default()
    }
}

fn read_stored_metadata(path: &Path) -> StoredModMetadata {
    let Ok(content) = fs::read_to_string(path) else {
        return StoredModMetadata::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

fn now_unix_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn pending_from_rpkg_path(path: &Path) -> Result<PendingRpkg, String> {
    let original_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "RPKG path has no file name".to_string())?
        .to_string();
    let (chunk, requested_patch, _) = parse_patch_file_name(&original_name)
        .ok_or_else(|| format!("{original_name} is not named like chunk0patch2.rpkg"))?;
    let data = fs::read(path).map_err(|e| e.to_string())?;
    Ok(PendingRpkg {
        size: data.len() as u64,
        data,
        original_name,
        chunk,
        requested_patch,
    })
}

fn read_zip_package(
    path: &Path,
) -> Result<(Vec<PendingRpkg>, StoredModMetadata, bool, bool), String> {
    let file = fs::File::open(path).map_err(|e| format!("Error opening ZIP: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Invalid ZIP archive: {e}"))?;
    let mut pending = Vec::new();
    let mut metadata = StoredModMetadata::default();
    let mut has_packagedefinition = false;
    let mut has_metadata = false;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| e.to_string())?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().replace('\\', "/");
        let file_name = Path::new(&name)
            .file_name()
            .and_then(|part| part.to_str())
            .unwrap_or_default()
            .to_string();
        let lower = file_name.to_ascii_lowercase();

        if lower == "packagedefinition.txt" {
            has_packagedefinition = true;
            continue;
        }
        if lower == "mod.json" || lower.ends_with(".metadata.json") {
            let mut content = String::new();
            entry
                .read_to_string(&mut content)
                .map_err(|e| e.to_string())?;
            metadata = metadata_from_json(&content);
            has_metadata = true;
            continue;
        }
        if lower.ends_with(".rpkg") {
            let (chunk, requested_patch, _) = parse_patch_file_name(&file_name)
                .ok_or_else(|| format!("{file_name} is not named like chunk0patch2.rpkg"))?;
            let mut data = Vec::new();
            entry.read_to_end(&mut data).map_err(|e| e.to_string())?;
            pending.push(PendingRpkg {
                original_name: file_name,
                size: data.len() as u64,
                data,
                chunk,
                requested_patch,
            });
        }
    }

    Ok((pending, metadata, has_packagedefinition, has_metadata))
}

fn inspect_mod_file(path: &Path, runtime: Option<&Path>) -> Result<ModPreview, String> {
    if !path.is_file() {
        return Err(format!("File not found: {}", path.display()));
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("mod")
        .to_string();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let (pending, has_packagedefinition, has_metadata) = match extension.as_str() {
        "rpkg" => (vec![pending_from_rpkg_path(path)?], false, false),
        "zip" => {
            let (items, _, has_pkg, has_meta) = read_zip_package(path)?;
            (items, has_pkg, has_meta)
        }
        _ => return Err("File must be .rpkg or .zip".to_string()),
    };

    let assigned = assign_rpkg_targets(pending, runtime)?;
    let mut warnings = Vec::new();
    if has_packagedefinition {
        warnings.push(
            "Included packagedefinition.txt will be ignored and regenerated safely.".to_string(),
        );
    }

    let rpkg_files = assigned
        .into_iter()
        .map(|item| {
            if item.pending.original_name != item.target_name {
                warnings.push(format!(
                    "{} will be installed as {} to avoid reserved or occupied patch slots.",
                    item.pending.original_name, item.target_name
                ));
            }
            PreviewRpkg {
                original_name: item.pending.original_name,
                target_name: item.target_name,
                chunk: item.pending.chunk,
                requested_patch: item.pending.requested_patch,
                target_patch: item.target_patch,
                size: item.pending.size,
            }
        })
        .collect::<Vec<_>>();

    Ok(ModPreview {
        file_name,
        package_type: extension,
        installable: !rpkg_files.is_empty(),
        rpkg_files,
        has_packagedefinition,
        has_metadata,
        warnings,
    })
}

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

fn get_steam_library_paths(steam_path: &str) -> Vec<String> {
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

fn extract_json_field(json: &str, field: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(json).ok()?;
    value[field].as_str().map(|text| text.to_string())
}

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

    // 新形式バックアップの存在確認 (旧形式も互換チェック)
    let backup_exists = game_dir.join(".flmm_backup").exists()
        || game_dir.join("Runtime_backup_original").exists();
    let installed = list_mods(game_dir.to_string_lossy().to_string())
        .await
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

// packagedefinition.txt のみを .flmm_backup/ にバックアップ
fn backup_packagedefinition_only(game_dir: &Path) -> Result<(), String> {
    let src = game_dir.join("Runtime").join("packagedefinition.txt");
    if !src.exists() {
        return Err("Runtime\\packagedefinition.txt was not found".to_string());
    }
    let backup_dir = game_dir.join(".flmm_backup");
    fs::create_dir_all(&backup_dir).map_err(|e| e.to_string())?;
    fs::copy(&src, backup_dir.join("packagedefinition.txt")).map_err(|e| e.to_string())?;
    Ok(())
}

// バックアップから packagedefinition.txt を Runtime/ に復元
fn restore_packagedefinition(game_dir: &Path) -> Result<(), String> {
    let backup_file = game_dir.join(".flmm_backup").join("packagedefinition.txt");
    if !backup_file.exists() {
        return Err("Backup packagedefinition.txt was not found".to_string());
    }
    let dst = game_dir.join("Runtime").join("packagedefinition.txt");
    fs::copy(&backup_file, &dst).map_err(|e| e.to_string())?;
    Ok(())
}

// 旧形式 Runtime_backup_original/ → 新形式 .flmm_backup/ へ自動移行
fn migrate_legacy_backup(game_dir: &Path) -> Result<(), String> {
    let legacy = game_dir.join("Runtime_backup_original");
    let new_backup = game_dir.join(".flmm_backup");
    if !legacy.is_dir() || new_backup.exists() {
        return Ok(());
    }
    let legacy_pkg = legacy.join("packagedefinition.txt");
    if !legacy_pkg.exists() {
        // packagedefinition.txt がない旧バックアップは移行不可のため無視
        return Ok(());
    }
    fs::create_dir_all(&new_backup).map_err(|e| e.to_string())?;
    fs::copy(&legacy_pkg, new_backup.join("packagedefinition.txt")).map_err(|e| e.to_string())?;
    fs::remove_dir_all(&legacy).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn install_mod(
    game_path: String,
    mod_path: String,
    lang: String,
) -> Result<String, String> {
    let is_pt = lang == "pt";
    let game_dir = normalize_game_path(&game_path)?;
    let runtime = game_dir.join("Runtime");
    let mod_file = PathBuf::from(&mod_path);

    if !mod_file.exists() {
        return Err(localized(
            is_pt,
            &format!("Arquivo não encontrado: {mod_path}"),
            &format!("File not found: {mod_path}"),
        ));
    }

    // 旧形式バックアップを新形式へ移行してから処理
    migrate_legacy_backup(&game_dir).map_err(|e| e.to_string())?;

    // 初回インストール時のみ packagedefinition.txt をバックアップ
    let new_backup = game_dir.join(".flmm_backup");
    if !new_backup.exists() {
        backup_packagedefinition_only(&game_dir)?;
    }

    let extension = mod_file
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let source_package = mod_file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("mod")
        .to_string();

    let (pending, base_metadata) = match extension.as_str() {
        "rpkg" => (
            vec![pending_from_rpkg_path(&mod_file)?],
            StoredModMetadata::default(),
        ),
        "zip" => {
            let (items, metadata, _, _) = read_zip_package(&mod_file)?;
            (items, metadata)
        }
        _ => {
            return Err(localized(
                is_pt,
                "O arquivo deve ser .rpkg ou .zip",
                "File must be .rpkg ou .zip",
            ));
        }
    };

    if pending.is_empty() {
        return Err(localized(
            is_pt,
            "Nenhum RPKG válido encontrado no pacote",
            "No valid RPKG files were found in the package",
        ));
    }

    let assigned = assign_rpkg_targets(pending, Some(&runtime))?;
    for item in assigned {
        let target_path = runtime.join(&item.target_name);
        fs::write(&target_path, &item.pending.data).map_err(|e| e.to_string())?;

        let fallback_name = item
            .pending
            .original_name
            .trim_end_matches(".rpkg")
            .to_string();
        let metadata = StoredModMetadata {
            name: if base_metadata.name.is_empty() {
                fallback_name
            } else {
                base_metadata.name.clone()
            },
            author: base_metadata.author.clone(),
            description: base_metadata.description.clone(),
            version: base_metadata.version.clone(),
            original_filename: item.pending.original_name,
            installed_filename: item.target_name.clone(),
            source_package: source_package.clone(),
            installed_at: now_unix_string(),
        };
        let metadata_path = runtime.join(format!(
            "{}.metadata.json",
            item.target_name.trim_end_matches(".rpkg")
        ));
        let metadata_json = serde_json::to_vec_pretty(&metadata).map_err(|e| e.to_string())?;
        fs::write(metadata_path, metadata_json).map_err(|e| e.to_string())?;
    }

    refresh_package_definition(&runtime)?;
    fs::write(game_dir.join(".flmm_installed"), "0.2.0").map_err(|e| e.to_string())?;

    Ok(localized(
        is_pt,
        "Mod instalado com sucesso!",
        "Mod installed successfully!",
    ))
}

fn localized(is_pt: bool, pt: &str, en: &str) -> String {
    if is_pt {
        pt.to_string()
    } else {
        en.to_string()
    }
}

#[tauri::command]
pub async fn uninstall_mod(game_path: String, lang: String) -> Result<String, String> {
    let is_pt = lang == "pt";
    let game_dir = normalize_game_path(&game_path)?;
    let runtime = game_dir.join("Runtime");
    let new_backup = game_dir.join(".flmm_backup");
    let marker = game_dir.join(".flmm_installed");

    // 旧形式バックアップを新形式へ移行してから処理
    migrate_legacy_backup(&game_dir).map_err(|e| e.to_string())?;

    if !new_backup.exists() {
        return Err(localized(
            is_pt,
            "Backup não encontrado. Não é possível desinstalar com segurança.",
            "Backup not found. Cannot safely uninstall.",
        ));
    }

    // Mod が追加したファイルのみ削除 (元の rpkg は触らない)
    if runtime.is_dir() {
        for entry in fs::read_dir(&runtime).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().to_string();
            let lower = name.to_ascii_lowercase();
            // MOD パッチ番号以上のファイルと metadata を削除
            let is_mod_rpkg = parse_patch_file_name(&name)
                .map(|(_, patch, _)| patch >= MOD_PATCH_START)
                .unwrap_or(false);
            let is_mod_meta = lower.ends_with(".metadata.json");
            if is_mod_rpkg || is_mod_meta {
                fs::remove_file(entry.path()).map_err(|e| e.to_string())?;
            }
        }
    }

    // packagedefinition.txt をオリジナルに復元
    restore_packagedefinition(&game_dir)?;

    // バックアップと管理ファイルを削除
    fs::remove_dir_all(&new_backup).map_err(|e| e.to_string())?;
    if marker.exists() {
        fs::remove_file(marker).map_err(|e| e.to_string())?;
    }

    Ok(localized(
        is_pt,
        "Mods desinstalados. Arquivos originais restaurados.",
        "Mods uninstalled. Original game files restored.",
    ))
}

#[tauri::command]
pub async fn delete_backup(game_path: String) -> Result<(), String> {
    let game_dir = normalize_game_path(&game_path)?;
    // 新形式バックアップを削除
    let new_backup = game_dir.join(".flmm_backup");
    if new_backup.exists() {
        fs::remove_dir_all(new_backup).map_err(|e| e.to_string())?;
    }
    // 旧形式バックアップが残存していれば合わせて削除
    let legacy = game_dir.join("Runtime_backup_original");
    if legacy.exists() {
        fs::remove_dir_all(legacy).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn check_updates(
    current_version: String,
    mod_id: String,
    api_key: Option<String>,
) -> Result<NexusRelease, String> {
    if mod_id == "0" || mod_id.is_empty() {
        return Ok(NexusRelease {
            version: "0.2.0".into(),
            url: "https://www.nexusmods.com/007firstlight".into(),
            has_update: false,
        });
    }

    let url = format!("https://api.nexusmods.com/v1/games/007firstlight/mods/{mod_id}.json");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let mut builder = client
        .get(&url)
        .header("User-Agent", "First-Light-Mod-Manager/0.2.0");
    if let Some(key) = api_key.filter(|key| !key.trim().is_empty()) {
        builder = builder.header("apikey", key.trim());
    }

    let resp = builder.send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Ok(NexusRelease {
            version: "0.2.0".into(),
            url: format!("https://www.nexusmods.com/007firstlight/mods/{mod_id}"),
            has_update: false,
        });
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let latest = json["version"].as_str().unwrap_or("0.2.0").to_string();
    let has_update = !current_version.is_empty() && latest != current_version;
    Ok(NexusRelease {
        version: latest,
        url: format!("https://www.nexusmods.com/007firstlight/mods/{mod_id}"),
        has_update,
    })
}

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

fn current_patchlevels(runtime: &Path) -> HashMap<u32, u32> {
    let Ok(definition) = read_packagedefinition(runtime) else {
        return HashMap::new();
    };
    let mut levels = HashMap::new();
    let mut index = 0_u32;
    for line in definition.content.lines() {
        if line.trim_start().starts_with("@partition") {
            if let Some(start) = line.find("patchlevel=") {
                let value_start = start + "patchlevel=".len();
                let value = line[value_start..]
                    .split_whitespace()
                    .next()
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or(0);
                levels.insert(index, value);
            }
            index += 1;
        }
    }
    levels
}

#[tauri::command]
pub async fn list_mods(game_path: String) -> Result<Vec<ModInfo>, String> {
    let game_dir = normalize_game_path(&game_path)?;
    let runtime = game_dir.join("Runtime");
    if !runtime.exists() {
        return Ok(Vec::new());
    }

    let patchlevels = current_patchlevels(&runtime);
    let mut mods = Vec::new();

    for entry in fs::read_dir(&runtime).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        let Some((chunk, patch, file_active)) = parse_patch_file_name(&file_name) else {
            continue;
        };

        let id = file_name
            .trim_end_matches(".disabled")
            .trim_end_matches(".rpkg")
            .to_string();
        let metadata = read_stored_metadata(&runtime.join(format!("{id}.metadata.json")));
        let level = patchlevels.get(&chunk).copied().unwrap_or(0);
        let active = file_active && level >= patch;

        mods.push(ModInfo {
            id: id.clone(),
            filename: format!("{id}.rpkg"),
            original_filename: if metadata.original_filename.is_empty() {
                format!("{id}.rpkg")
            } else {
                metadata.original_filename
            },
            name: if metadata.name.is_empty() {
                id.clone()
            } else {
                metadata.name
            },
            author: metadata.author,
            description: metadata.description,
            version: metadata.version,
            active,
            chunk,
            patch,
        });
    }

    mods.sort_by_key(|item| (item.chunk, item.patch));
    Ok(mods)
}

#[tauri::command]
pub async fn toggle_mod(game_path: String, mod_id: String, active: bool) -> Result<(), String> {
    let game_dir = normalize_game_path(&game_path)?;
    let runtime = game_dir.join("Runtime");
    let active_path = runtime.join(format!("{mod_id}.rpkg"));
    let disabled_path = runtime.join(format!("{mod_id}.rpkg.disabled"));

    if active {
        if disabled_path.exists() {
            if active_path.exists() {
                return Err(format!("{} already exists", active_path.display()));
            }
            fs::rename(&disabled_path, &active_path).map_err(|e| e.to_string())?;
        } else if !active_path.exists() {
            return Err(format!("{mod_id}.rpkg was not found"));
        }
    } else if active_path.exists() {
        fs::rename(&active_path, &disabled_path).map_err(|e| e.to_string())?;
    }

    refresh_package_definition(&runtime)
}

#[tauri::command]
pub async fn delete_mod(game_path: String, mod_id: String) -> Result<(), String> {
    let game_dir = normalize_game_path(&game_path)?;
    let runtime = game_dir.join("Runtime");
    for path in [
        runtime.join(format!("{mod_id}.rpkg")),
        runtime.join(format!("{mod_id}.rpkg.disabled")),
        runtime.join(format!("{mod_id}.metadata.json")),
    ] {
        if path.exists() {
            fs::remove_file(path).map_err(|e| e.to_string())?;
        }
    }
    refresh_package_definition(&runtime)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    // テスト用一時ディレクトリ自動削除構造体
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        // 一時ディレクトリ作成
        fn new(name: &str) -> Self {
            let suffix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!("flmm_{name}_{suffix}"));
            fs::create_dir_all(&path).unwrap();
            TempDir { path }
        }
    }

    impl Drop for TempDir {
        // インスタンス破棄時の一時ディレクトリ再帰削除
        fn drop(&mut self) {
            if self.path.exists() {
                let _ = fs::remove_dir_all(&self.path);
            }
        }
    }

    // ダミーパッケージ定義テキスト生成
    fn sample_packagedefinition() -> String {
        "// --- Chunk Boot + PlayGo\r\n@partition name=super parent=none type=standard patchlevel=0\r\n[assembly:/_glacier/ini/globalresources.ini].pc_resourceidx\r\n// --- Chunk Rest of missions\r\n@partition name=base parent=super type=standard patchlevel=0\r\n".to_string()
    }

    // 暗号化パッケージ定義書き込み
    fn write_encrypted_packagedefinition(runtime: &Path, content: &str) {
        let definition = PackageDefinition {
            content: content.to_string(),
            encrypted: true,
        };
        write_packagedefinition(runtime, &definition).unwrap();
    }

    #[test]
    fn refresh_package_definition_sets_patchlevel_and_preserves_crlf() {
        let game = TempDir::new("pkg");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();
        write_encrypted_packagedefinition(&runtime, &sample_packagedefinition());
        fs::write(runtime.join("chunk0patch2.rpkg"), b"dummy").unwrap();

        refresh_package_definition(&runtime).unwrap();
        let definition = read_packagedefinition(&runtime).unwrap();

        assert!(definition.encrypted);
        assert!(definition
            .content
            .contains("@partition name=super parent=none type=standard patchlevel=100"));
        assert!(definition
            .content
            .contains("@partition name=base parent=super type=standard patchlevel=0"));
        assert!(
            !definition
                .content
                .as_bytes()
                .windows(1)
                .any(|window| window == b"\n")
                || definition.content.contains("\r\n")
        );
    }

    #[test]
    fn inactive_mods_lower_patchlevel_to_zero() {
        let game = TempDir::new("toggle");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();
        write_encrypted_packagedefinition(&runtime, &sample_packagedefinition());
        fs::write(runtime.join("chunk0patch2.rpkg.disabled"), b"dummy").unwrap();

        refresh_package_definition(&runtime).unwrap();
        let definition = read_packagedefinition(&runtime).unwrap();

        assert!(definition
            .content
            .contains("@partition name=super parent=none type=standard patchlevel=0"));
    }

    #[test]
    fn assigns_mod_to_patch100_slot() {
        let runtime = TempDir::new("assign");
        let pending = PendingRpkg {
            original_name: "chunk0patch1.rpkg".to_string(),
            data: vec![1, 2, 3],
            size: 3,
            chunk: 0,
            requested_patch: 1,
        };

        let assigned = assign_rpkg_targets(vec![pending], Some(&runtime.path)).unwrap();

        assert_eq!(assigned[0].target_name, "chunk0patch100.rpkg");
    }

    #[test]
    fn inspect_zip_reports_ignored_packagedefinition() {
        let root = TempDir::new("zip");
        let zip_path = root.path.join("mod.zip");
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("chunk0patch1.rpkg", options).unwrap();
        zip.write_all(b"rpkg").unwrap();
        zip.start_file("packagedefinition.txt", options).unwrap();
        zip.write_all(b"ignored").unwrap();
        zip.start_file("mod.json", options).unwrap();
        zip.write_all(br#"{"name":"Preview Mod","version":"1.2"}"#)
            .unwrap();
        zip.finish().unwrap();

        let preview = inspect_mod_file(&zip_path, Some(&root.path)).unwrap();

        assert!(preview.installable);
        assert!(preview.has_packagedefinition);
        assert!(preview.has_metadata);
        assert_eq!(preview.rpkg_files[0].target_name, "chunk0patch100.rpkg");
        assert!(preview
            .warnings
            .iter()
            .any(|warning| warning.contains("ignored")));
    }

    #[test]
    fn decode_to_string_is_latin1_fixed() {
        // ASCIIデコード動作検証
        let ascii = "hello".as_bytes().to_vec();
        assert_eq!(decode_to_string(ascii), "hello");

        // Latin-1拡張文字デコード動作検証
        let latin1 = vec![0x68, 0x65, 0x6C, 0x6C, 0xE9];
        assert_eq!(decode_to_string(latin1), "hell\u{00e9}");
    }

    #[test]
    fn read_packagedefinition_strips_utf8_bom() {
        let game = TempDir::new("bom");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        // UTF-8 BOM付きテキストファイル書き込み
        let mut content = vec![0xEF, 0xBB, 0xBF];
        content.extend_from_slice(b"@partition name=super parent=none type=standard patchlevel=0\r\n");
        fs::write(runtime.join("packagedefinition.txt"), &content).unwrap();

        let definition = read_packagedefinition(&runtime).unwrap();
        assert!(!definition.encrypted);
        assert!(definition.content.starts_with("@partition"));
    }

    #[test]
    fn read_packagedefinition_handles_latin1_content() {
        let game = TempDir::new("latin1");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        // Latin-1エンコーディングデータ書き込み
        let content = b"// copyright \xA9 test\r\n@partition name=super parent=none type=standard patchlevel=0\r\n".to_vec();
        fs::write(runtime.join("packagedefinition.txt"), &content).unwrap();

        let definition = read_packagedefinition(&runtime).unwrap();
        assert!(!definition.encrypted);
        assert!(definition.content.contains("©"));
        assert!(definition.content.contains("@partition"));
    }

    #[test]
    fn official_patches_in_runtime_are_reserved_slots() {
        // Runtime 内の公式パッチがスロット予約に含まれることを確認
        let game = TempDir::new("official_reserve");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        // Runtime に公式パッチを配置
        fs::write(runtime.join("chunk0patch1.rpkg"), b"official").unwrap();

        let pending = PendingRpkg {
            original_name: "chunk0patch1.rpkg".to_string(),
            data: vec![1, 2, 3],
            size: 3,
            chunk: 0,
            requested_patch: 1,
        };

        let assigned = assign_rpkg_targets(vec![pending], Some(&runtime)).unwrap();

        // MOD開始インデックス割り当て検証
        assert_eq!(assigned[0].target_name, "chunk0patch100.rpkg");
    }

    #[test]
    fn official_patches_skip_occupied_slots() {
        // Runtime 内の既存ファイルが衝突回避されることを確認
        let game = TempDir::new("official_skip");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        // Runtime に公式パッチと既存 Mod を配置
        fs::write(runtime.join("chunk1patch1.rpkg"), b"official").unwrap();
        fs::write(runtime.join("chunk1patch2.rpkg"), b"existing_mod").unwrap();

        let pending = PendingRpkg {
            original_name: "chunk1patch1.rpkg".to_string(),
            data: vec![4, 5, 6],
            size: 3,
            chunk: 1,
            requested_patch: 1,
        };

        let assigned = assign_rpkg_targets(vec![pending], Some(&runtime)).unwrap();

        // 開始インデックス以降の割り当て検証
        assert_eq!(assigned[0].target_name, "chunk1patch100.rpkg");
    }

    // ─── parse_patch_file_name ────────────────────────────────────────

    #[test]
    fn parse_patch_file_name_active_rpkg() {
        // 有効パッチ名パース
        let result = parse_patch_file_name("chunk0patch2.rpkg");
        assert_eq!(result, Some((0, 2, true)));
    }

    #[test]
    fn parse_patch_file_name_disabled_rpkg() {
        // 無効化パッチ名パース
        let result = parse_patch_file_name("chunk1patch100.rpkg.disabled");
        assert_eq!(result, Some((1, 100, false)));
    }

    #[test]
    fn parse_patch_file_name_case_insensitive() {
        // 大文字小文字混在パッチ名パース
        let result = parse_patch_file_name("Chunk2Patch5.RPKG");
        assert_eq!(result, Some((2, 5, true)));
    }

    #[test]
    fn parse_patch_file_name_non_rpkg_returns_none() {
        // 非対象ファイル名のパース除外
        assert_eq!(parse_patch_file_name("packagedefinition.txt"), None);
        assert_eq!(parse_patch_file_name("mod.json"), None);
        assert_eq!(parse_patch_file_name("chunk0.rpkg"), None);
    }

    #[test]
    fn parse_patch_file_name_large_numbers() {
        // 大きな値のパッチ名パース
        let result = parse_patch_file_name("chunk99patch9999.rpkg");
        assert_eq!(result, Some((99, 9999, true)));
    }

    // ─── set_patchlevel_in_line ──────────────────────────────────────

    #[test]
    fn set_patchlevel_in_line_replaces_existing_value() {
        // 既存値の置換
        let line = "@partition name=super parent=none type=standard patchlevel=0";
        let result = set_patchlevel_in_line(line, 100);
        assert!(result.contains("patchlevel=100"));
        assert!(!result.contains("patchlevel=0 ") && !result.ends_with("patchlevel=0"));
    }

    #[test]
    fn set_patchlevel_in_line_appends_when_missing() {
        // キー未存在時の追記
        let line = "@partition name=base parent=super type=standard";
        let result = set_patchlevel_in_line(line, 50);
        assert!(result.contains("patchlevel=50"));
    }

    #[test]
    fn set_patchlevel_in_line_preserves_surrounding_content() {
        // 周辺パラメーター保持
        let line = "@partition name=x patchlevel=5 extra=value";
        let result = set_patchlevel_in_line(line, 200);
        assert!(result.contains("extra=value"));
        assert!(result.contains("patchlevel=200"));
    }

    // ─── detect_line_ending ──────────────────────────────────────────

    #[test]
    fn detect_line_ending_crlf() {
        // CRLF改行検出
        assert_eq!(detect_line_ending("line1\r\nline2\r\n"), "\r\n");
    }

    #[test]
    fn detect_line_ending_lf_only() {
        // LF改行検出
        assert_eq!(detect_line_ending("line1\nline2\n"), "\n");
    }

    #[test]
    fn detect_line_ending_no_newline() {
        // 改行なし時のデフォルトLF返却
        assert_eq!(detect_line_ending("single line"), "\n");
    }

    // ─── apply_patchlevels ───────────────────────────────────────────

    #[test]
    fn apply_patchlevels_updates_correct_partition() {
        // 特定パーティションパッチ更新
        let content = "@partition name=boot patchlevel=0\n@partition name=main patchlevel=0\n";
        let mut levels = HashMap::new();
        levels.insert(1, 100u32);
        let result = apply_patchlevels(content, &levels).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines[0].contains("patchlevel=0"));
        assert!(lines[1].contains("patchlevel=100"));
    }

    #[test]
    fn apply_patchlevels_error_on_missing_partition() {
        // パーティション未存在時エラー
        let content = "@partition name=boot patchlevel=0\n";
        let mut levels = HashMap::new();
        levels.insert(5, 100u32);
        let result = apply_patchlevels(content, &levels);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("chunk5"));
    }

    #[test]
    fn apply_patchlevels_preserves_crlf_trailing_newline() {
        // 末尾改行コード維持
        let content = "@partition name=boot patchlevel=0\r\n";
        let levels = HashMap::new();
        let result = apply_patchlevels(content, &levels).unwrap();
        assert!(result.ends_with("\r\n"));
    }

    // ─── crc32_ieee ──────────────────────────────────────────────────

    #[test]
    fn crc32_ieee_known_values() {
        // IEEE標準チェックサム検証
        let crc = crc32_ieee(b"123456789");
        assert_eq!(crc, 0xCBF43926);
    }

    #[test]
    fn crc32_ieee_empty_input() {
        // 空バッファ時のゼロ値
        assert_eq!(crc32_ieee(b""), 0x00000000);
    }

    #[test]
    fn crc32_ieee_single_byte() {
        // 単一バイトハッシュ
        let crc = crc32_ieee(&[0x00]);
        assert_eq!(crc, 0xD202EF8D);
    }

    // ─── encrypt / decrypt 往復テスト ─────────────────────────────────

    #[test]
    fn encrypt_decrypt_roundtrip_ascii() {
        // 暗号化/復号往復一致検証
        let original = b"Hello, World!!! Test data here.";
        let encrypted = encrypt_buffer(original);
        let decrypted = decrypt_buffer(&encrypted);
        assert_eq!(decrypted, original);
    }

    #[test]
    fn encrypt_decrypt_roundtrip_with_padding() {
        // パディングブロック往復一致検証
        let original = b"abc";
        let encrypted = encrypt_buffer(original);
        let decrypted = decrypt_buffer(&encrypted);
        assert_eq!(decrypted, original);
    }

    #[test]
    fn encrypt_buffer_output_is_multiple_of_8() {
        // ブロック境界出力サイズ検証
        for len in 1..=20usize {
            let data: Vec<u8> = (0..len as u8).collect();
            let enc = encrypt_buffer(&data);
            assert_eq!(enc.len() % 8, 0, "len={len} encrypted to {} bytes", enc.len());
        }
    }

    #[test]
    fn encrypt_decrypt_roundtrip_packagedefinition_content() {
        // 構造定義データ暗号往復
        let content = "@partition name=super parent=none type=standard patchlevel=100\r\n\
                       @partition name=base parent=super type=standard patchlevel=0\r\n";
        let original = content.as_bytes();
        let encrypted = encrypt_buffer(original);
        let decrypted = decrypt_buffer(&encrypted);
        assert_eq!(decrypted, original);
    }

    // ─── XTEA ブロック暗号単体 ───────────────────────────────────────

    #[test]
    fn xtea_block_encrypt_decrypt_roundtrip() {
        // ブロック単位暗号化往復
        let original_a: u32 = 0xDEADBEEF;
        let original_b: u32 = 0xCAFEBABE;
        let mut a = original_a;
        let mut b = original_b;
        encrypt_block_xtea(&mut a, &mut b, &XTEA_KEYS);
        assert_ne!(a, original_a);
        decrypt_block_xtea(&mut a, &mut b, &XTEA_KEYS);
        assert_eq!(a, original_a);
        assert_eq!(b, original_b);
    }

    #[test]
    fn xtea_block_zero_block() {
        // ゼロブロック暗号化往復
        let mut a = 0u32;
        let mut b = 0u32;
        encrypt_block_xtea(&mut a, &mut b, &XTEA_KEYS);
        assert_ne!((a, b), (0, 0));
        decrypt_block_xtea(&mut a, &mut b, &XTEA_KEYS);
        assert_eq!((a, b), (0, 0));
    }

    // ─── normalize_game_path ────────────────────────────────────────

    #[test]
    fn normalize_game_path_accepts_game_dir_with_runtime_and_exe() {
        let game = TempDir::new("norm_game");
        let runtime = game.path.join("Runtime");
        let retail = game.path.join("Retail");
        fs::create_dir_all(&runtime).unwrap();
        fs::create_dir_all(&retail).unwrap();
        fs::write(runtime.join("chunk0.rpkg"), b"dummy").unwrap();

        let result = normalize_game_path(game.path.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn normalize_game_path_accepts_runtime_subdirectory() {
        let game = TempDir::new("norm_runtime");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();
        fs::write(runtime.join("chunk0.rpkg"), b"dummy").unwrap();

        let result = normalize_game_path(runtime.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), game.path);
    }

    #[test]
    fn normalize_game_path_rejects_empty_string() {
        // 空パスの除外
        let result = normalize_game_path("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn normalize_game_path_rejects_invalid_dir() {
        // 実在しないパスの除外
        let result = normalize_game_path("C:\\NotExistingDirectory\\FakeGame");
        assert!(result.is_err());
    }

    // ─── patchlevels_by_partition ────────────────────────────────────

    #[test]
    fn patchlevels_by_partition_ignores_disabled_mods() {
        let runtime = TempDir::new("pbl_disabled");
        fs::write(runtime.path.join("chunk0patch100.rpkg"), b"active").unwrap();
        fs::write(runtime.path.join("chunk0patch101.rpkg.disabled"), b"disabled").unwrap();

        let result = patchlevels_by_partition(&runtime.path).unwrap();
        assert_eq!(result.get(&0), Some(&100));
    }

    #[test]
    fn patchlevels_by_partition_picks_max_active_patch() {
        let runtime = TempDir::new("pbl_max");
        fs::write(runtime.path.join("chunk0patch100.rpkg"), b"a").unwrap();
        fs::write(runtime.path.join("chunk0patch105.rpkg"), b"b").unwrap();
        fs::write(runtime.path.join("chunk0patch102.rpkg"), b"c").unwrap();

        let result = patchlevels_by_partition(&runtime.path).unwrap();
        assert_eq!(result.get(&0), Some(&105));
    }

    #[test]
    fn patchlevels_by_partition_empty_runtime() {
        let runtime = TempDir::new("pbl_empty");
        let result = patchlevels_by_partition(&runtime.path).unwrap();
        assert!(result.is_empty());
    }

    // ─── metadata_from_json ─────────────────────────────────────────

    #[test]
    fn metadata_from_json_parses_all_fields() {
        // JSONメタデータパース
        let json = r#"{"name":"Test Mod","author":"Tester","description":"A test mod","version":"2.0.0"}"#;
        let meta = metadata_from_json(json);
        assert_eq!(meta.name, "Test Mod");
        assert_eq!(meta.author, "Tester");
        assert_eq!(meta.description, "A test mod");
        assert_eq!(meta.version, "2.0.0");
    }

    #[test]
    fn metadata_from_json_partial_fields() {
        // 部分的なJSONメタデータパース
        let json = r#"{"name":"Only Name"}"#;
        let meta = metadata_from_json(json);
        assert_eq!(meta.name, "Only Name");
        assert_eq!(meta.author, "");
        assert_eq!(meta.version, "");
    }

    #[test]
    fn metadata_from_json_invalid_json_returns_default() {
        // 不正JSONパース失敗時のデフォルトフォールバック
        let meta = metadata_from_json("{not valid json}");
        assert_eq!(meta.name, "");
        assert_eq!(meta.author, "");
    }

    #[test]
    fn metadata_from_json_empty_string_returns_default() {
        // 空文字パース失敗時のデフォルトフォールバック
        let meta = metadata_from_json("");
        assert_eq!(meta.name, "");
    }

    // ─── extract_json_field ─────────────────────────────────────────

    #[test]
    fn extract_json_field_returns_existing_string() {
        // 特定キーの値取得
        let json = r#"{"InstallLocation":"C:\\Games\\007 First Light","AppId":"007"}"#;
        let result = extract_json_field(json, "InstallLocation");
        assert_eq!(result, Some("C:\\Games\\007 First Light".to_string()));
    }

    #[test]
    fn extract_json_field_returns_none_for_missing_key() {
        // 未存在キーのNone取得
        let json = r#"{"AppId":"007"}"#;
        let result = extract_json_field(json, "InstallLocation");
        assert_eq!(result, None);
    }

    #[test]
    fn extract_json_field_returns_none_for_non_string_value() {
        // 非文字列型キーのNone取得
        let json = r#"{"count":42}"#;
        let result = extract_json_field(json, "count");
        assert_eq!(result, None);
    }

    #[test]
    fn extract_json_field_invalid_json_returns_none() {
        // 不正JSONパース失敗
        let result = extract_json_field("not json at all", "field");
        assert_eq!(result, None);
    }

    // ─── localized ──────────────────────────────────────────────────

    #[test]
    fn localized_pt_returns_portuguese() {
        // ポルトガル語選択
        let result = localized(true, "Instalado com sucesso!", "Installed successfully!");
        assert_eq!(result, "Instalado com sucesso!");
    }

    #[test]
    fn localized_en_returns_english() {
        // 英語選択
        let result = localized(false, "Instalado com sucesso!", "Installed successfully!");
        assert_eq!(result, "Installed successfully!");
    }

    // ─── get_steam_library_paths ─────────────────────────────────────

    #[test]
    fn get_steam_library_paths_includes_steam_root() {
        // Steamインストール親ディレクトリの取得確認
        let paths = get_steam_library_paths("C:\\Steam");
        assert!(!paths.is_empty());
        assert_eq!(paths[0], "C:\\Steam");
    }

    #[test]
    fn get_steam_library_paths_parses_vdf_path_field() {
        let dir = TempDir::new("steam_vdf");
        let steam_path = dir.path.to_str().unwrap().to_string();
        let steamapps = dir.path.join("steamapps");
        fs::create_dir_all(&steamapps).unwrap();

        let vdf_content = "\"LibraryFolders\"\n{\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"D:\\\\SteamLibrary\"\n\t}\n}\n";
        fs::write(steamapps.join("libraryfolders.vdf"), vdf_content).unwrap();

        let paths = get_steam_library_paths(&steam_path);
        assert!(paths.len() >= 2);
        let found = paths.iter().any(|p| p.contains("SteamLibrary"));
        assert!(found, "VDF path was not extracted: {:?}", paths);
    }

    // ─── write_packagedefinition / read_packagedefinition 往復 ─────

    #[test]
    fn write_read_packagedefinition_plaintext_roundtrip() {
        let game = TempDir::new("rw_plain");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        let content = "@partition name=boot patchlevel=0\n@partition name=main patchlevel=0\n";
        let definition = PackageDefinition {
            content: content.to_string(),
            encrypted: false,
        };
        write_packagedefinition(&runtime, &definition).unwrap();
        let read_back = read_packagedefinition(&runtime).unwrap();

        assert!(!read_back.encrypted);
        assert_eq!(read_back.content, content);
    }

    #[test]
    fn write_read_packagedefinition_encrypted_roundtrip() {
        let game = TempDir::new("rw_enc");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        let content = "@partition name=boot patchlevel=100\r\n@partition name=main patchlevel=0\r\n";
        let definition = PackageDefinition {
            content: content.to_string(),
            encrypted: true,
        };
        write_packagedefinition(&runtime, &definition).unwrap();
        let read_back = read_packagedefinition(&runtime).unwrap();

        assert!(read_back.encrypted);
        assert_eq!(read_back.content, content);
    }

    #[test]
    fn read_packagedefinition_detects_checksum_mismatch() {
        let game = TempDir::new("crc_mismatch");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        let content = "test content here";
        let definition = PackageDefinition {
            content: content.to_string(),
            encrypted: true,
        };
        write_packagedefinition(&runtime, &definition).unwrap();

        let pkg_path = runtime.join("packagedefinition.txt");
        let mut raw = fs::read(&pkg_path).unwrap();
        raw[16] ^= 0xFF;
        fs::write(&pkg_path, &raw).unwrap();

        let result = read_packagedefinition(&runtime);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("checksum"));
    }

    // ─── assign_rpkg_targets (複数 pending) ─────────────────────────

    #[test]
    fn assign_rpkg_targets_multiple_chunks_no_collision() {
        let runtime = TempDir::new("multi_chunk");
        let pending = vec![
            PendingRpkg {
                original_name: "chunk0patch1.rpkg".to_string(),
                data: vec![1],
                size: 1,
                chunk: 0,
                requested_patch: 1,
            },
            PendingRpkg {
                original_name: "chunk1patch1.rpkg".to_string(),
                data: vec![2],
                size: 1,
                chunk: 1,
                requested_patch: 1,
            },
        ];

        let assigned = assign_rpkg_targets(pending, Some(&runtime.path)).unwrap();
        assert_eq!(assigned.len(), 2);
        assert_eq!(assigned[0].target_patch, 100);
        assert_eq!(assigned[1].target_patch, 100);
    }

    #[test]
    fn assign_rpkg_targets_same_chunk_sequential_slots() {
        let runtime = TempDir::new("same_chunk_seq");
        let pending = vec![
            PendingRpkg {
                original_name: "chunk0patch1.rpkg".to_string(),
                data: vec![1],
                size: 1,
                chunk: 0,
                requested_patch: 1,
            },
            PendingRpkg {
                original_name: "chunk0patch2.rpkg".to_string(),
                data: vec![2],
                size: 1,
                chunk: 0,
                requested_patch: 2,
            },
        ];

        let assigned = assign_rpkg_targets(pending, Some(&runtime.path)).unwrap();
        assert_eq!(assigned[0].target_patch, 100);
        assert_eq!(assigned[1].target_patch, 101);
    }

    #[test]
    fn assign_rpkg_targets_skips_occupied_slots_sequentially() {
        let runtime = TempDir::new("skip_seq");
        fs::write(runtime.path.join("chunk0patch100.rpkg"), b"a").unwrap();
        fs::write(runtime.path.join("chunk0patch101.rpkg"), b"b").unwrap();

        let pending = PendingRpkg {
            original_name: "chunk0patch1.rpkg".to_string(),
            data: vec![3],
            size: 1,
            chunk: 0,
            requested_patch: 1,
        };

        let assigned = assign_rpkg_targets(vec![pending], Some(&runtime.path)).unwrap();
        assert_eq!(assigned[0].target_patch, 102);
    }

    // ─── used_patch_slots ────────────────────────────────────────────

    #[test]
    fn used_patch_slots_with_no_runtime_returns_empty() {
        let slots = used_patch_slots(None).unwrap();
        assert!(slots.is_empty());
    }

    #[test]
    fn used_patch_slots_includes_active_and_disabled_rpkg() {
        let runtime = TempDir::new("slots_mixed");
        fs::write(runtime.path.join("chunk0patch100.rpkg"), b"a").unwrap();
        fs::write(runtime.path.join("chunk1patch101.rpkg.disabled"), b"b").unwrap();

        let slots = used_patch_slots(Some(&runtime.path)).unwrap();
        assert!(slots.contains(&(0, 100)));
        assert!(slots.contains(&(1, 101)));
    }

    #[test]
    fn used_patch_slots_runtime_official_patches_are_reserved() {
        // Runtime 内の公式パッチがスロット予約に含まれることを確認
        let game = TempDir::new("slots_official");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        fs::write(runtime.join("chunk0patch1.rpkg"), b"official").unwrap();

        let slots = used_patch_slots(Some(&runtime)).unwrap();
        assert!(slots.contains(&(0, 1)));
    }

    // ─── target_patch_start ─────────────────────────────────────────

    #[test]
    fn target_patch_start_always_returns_mod_patch_start() {
        // MOD開始インデックス初期値取得
        assert_eq!(target_patch_start(0), 100);
        assert_eq!(target_patch_start(1), 100);
        assert_eq!(target_patch_start(99), 100);
    }

    // ─── current_patchlevels ─────────────────────────────────────────

    #[test]
    fn current_patchlevels_empty_runtime_returns_empty() {
        let runtime = TempDir::new("cpatch_empty");
        let result = current_patchlevels(&runtime.path);
        assert!(result.is_empty());
    }

    #[test]
    fn current_patchlevels_parses_packagedefinition_patchlevels() {
        let runtime = TempDir::new("cpatch_vals");

        let pkg_def_content = "\
@partition0 patchlevel=103
@partition1 patchlevel=100
";
        fs::write(runtime.path.join("packagedefinition.txt"), pkg_def_content).unwrap();

        let result = current_patchlevels(&runtime.path);

        assert_eq!(result.get(&0), Some(&103));
        assert_eq!(result.get(&1), Some(&100));
    }

    #[tokio::test]
    async fn test_delete_backup_new_format() {
        // 新形式 .flmm_backup/ の削除確認
        let game = TempDir::new("del_backup_new");
        let runtime = game.path.join("Runtime");
        let backup = game.path.join(".flmm_backup");

        fs::create_dir_all(&runtime).unwrap();
        fs::create_dir_all(&backup).unwrap();
        fs::write(runtime.join("chunk0.rpkg"), b"").unwrap();
        fs::write(backup.join("packagedefinition.txt"), b"pkg").unwrap();

        let result = delete_backup(game.path.to_string_lossy().to_string()).await;
        assert!(result.is_ok());
        assert!(!backup.exists());
    }

    #[tokio::test]
    async fn test_delete_backup_legacy_format() {
        // 旧形式 Runtime_backup_original/ も削除されることを確認
        let game = TempDir::new("del_backup_legacy");
        let runtime = game.path.join("Runtime");
        let legacy = game.path.join("Runtime_backup_original");

        fs::create_dir_all(&runtime).unwrap();
        fs::create_dir_all(&legacy).unwrap();
        fs::write(runtime.join("chunk0.rpkg"), b"").unwrap();

        let result = delete_backup(game.path.to_string_lossy().to_string()).await;
        assert!(result.is_ok());
        assert!(!legacy.exists());
    }

    #[tokio::test]
    async fn test_list_mods() {
        let game = TempDir::new("list_mods");
        let runtime = game.path.join("Runtime");

        fs::create_dir_all(&runtime).unwrap();
        fs::write(runtime.join("chunk0.rpkg"), b"").unwrap();

        let pkg_def_content = "@partition0 patchlevel=100\n";
        fs::write(runtime.join("packagedefinition.txt"), pkg_def_content).unwrap();
        fs::write(runtime.join("chunk0patch100.rpkg"), b"data").unwrap();

        let metadata_content = r#"{"name":"Test Mod Name","author":"Author Name","description":"Desc","version":"1.0","original_filename":"chunk0patch100.rpkg","installed_filename":"chunk0patch100.rpkg","source_package":"","installed_at":""}"#;
        fs::write(runtime.join("chunk0patch100.metadata.json"), metadata_content).unwrap();

        let mods = list_mods(game.path.to_string_lossy().to_string()).await.unwrap();
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].name, "Test Mod Name");
        assert_eq!(mods[0].author, "Author Name");
        assert!(mods[0].active);
    }

    #[tokio::test]
    async fn test_toggle_mod_and_delete_mod() {
        let game = TempDir::new("toggle_and_del");
        let runtime = game.path.join("Runtime");

        fs::create_dir_all(&runtime).unwrap();
        fs::write(runtime.join("chunk0.rpkg"), b"").unwrap();

        let pkg_def_content = "@partition0 patchlevel=0\n";
        fs::write(runtime.join("packagedefinition.txt"), pkg_def_content).unwrap();

        let active_path = runtime.join("chunk0patch100.rpkg");
        fs::write(&active_path, b"data").unwrap();

        let result = toggle_mod(game.path.to_string_lossy().to_string(), "chunk0patch100".to_string(), false).await;
        assert!(result.is_ok());
        assert!(!active_path.exists());
        let disabled_path = runtime.join("chunk0patch100.rpkg.disabled");
        assert!(disabled_path.exists());

        let result = toggle_mod(game.path.to_string_lossy().to_string(), "chunk0patch100".to_string(), true).await;
        assert!(result.is_ok());
        assert!(active_path.exists());
        assert!(!disabled_path.exists());

        let result = delete_mod(game.path.to_string_lossy().to_string(), "chunk0patch100".to_string()).await;
        assert!(result.is_ok());
        assert!(!active_path.exists());
    }

    #[tokio::test]
    async fn test_open_game_folder_invalid_path() {
        let result = open_game_folder("invalid_path_xyz".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_and_save_settings() {
        let temp_dir = TempDir::new("settings_test");

        // テスト用環境変数の設定
        #[cfg(target_os = "windows")]
        std::env::set_var("APPDATA", &temp_dir.path);
        #[cfg(not(target_os = "windows"))]
        std::env::set_var("HOME", &temp_dir.path);

        // デフォルト設定ロード検証
        let loaded = load_settings().await.unwrap();
        assert_eq!(loaded.language, "en");
        assert_eq!(loaded.nexus_mod_id, "0");
        assert!(loaded.auto_check_updates);

        // 設定の保存検証
        let to_save = AppSettings {
            game_path: "".to_string(),
            language: "pt".to_string(),
            nexus_api_key: "my_api_key".to_string(),
            nexus_mod_id: "999".to_string(),
            auto_check_updates: false,
        };
        let saved = save_settings(to_save).await.unwrap();
        assert_eq!(saved.language, "pt");
        assert_eq!(saved.nexus_api_key, "my_api_key");
        assert_eq!(saved.nexus_mod_id, "999");
        assert!(!saved.auto_check_updates);

        // 設定のロード再検証
        let loaded2 = load_settings().await.unwrap();
        assert_eq!(loaded2.language, "pt");
        assert_eq!(loaded2.nexus_api_key, "my_api_key");
        assert_eq!(loaded2.nexus_mod_id, "999");
        assert!(!loaded2.auto_check_updates);
    }

    // ─── backup_packagedefinition_only ──────────────────────────────

    #[test]
    fn backup_packagedefinition_only_creates_flmm_backup() {
        // .flmm_backup/packagedefinition.txt が生成されることを確認
        let game = TempDir::new("bkp_only");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();
        fs::write(runtime.join("packagedefinition.txt"), b"original").unwrap();

        backup_packagedefinition_only(&game.path).unwrap();

        let backed_up = game.path.join(".flmm_backup").join("packagedefinition.txt");
        assert!(backed_up.exists());
        assert_eq!(fs::read(&backed_up).unwrap(), b"original");
    }

    #[test]
    fn backup_packagedefinition_only_fails_without_packagedefinition() {
        // packagedefinition.txt が存在しない場合はエラー
        let game = TempDir::new("bkp_missing");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        let result = backup_packagedefinition_only(&game.path);
        assert!(result.is_err());
    }

    // ─── restore_packagedefinition ──────────────────────────────────

    #[test]
    fn restore_packagedefinition_overwrites_runtime_file() {
        // バックアップから Runtime/packagedefinition.txt を復元
        let game = TempDir::new("restore_pkg");
        let runtime = game.path.join("Runtime");
        let backup_dir = game.path.join(".flmm_backup");
        fs::create_dir_all(&runtime).unwrap();
        fs::create_dir_all(&backup_dir).unwrap();

        fs::write(backup_dir.join("packagedefinition.txt"), b"backup_content").unwrap();
        fs::write(runtime.join("packagedefinition.txt"), b"modified_by_mod").unwrap();

        restore_packagedefinition(&game.path).unwrap();

        let restored = fs::read(runtime.join("packagedefinition.txt")).unwrap();
        assert_eq!(restored, b"backup_content");
    }

    #[test]
    fn restore_packagedefinition_fails_without_backup() {
        // バックアップが存在しない場合はエラー
        let game = TempDir::new("restore_missing");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        let result = restore_packagedefinition(&game.path);
        assert!(result.is_err());
    }

    // ─── migrate_legacy_backup ──────────────────────────────────────

    #[test]
    fn migrate_legacy_backup_moves_packagedefinition_and_removes_old() {
        // 旧形式を新形式へ移行し、旧ディレクトリを削除
        let game = TempDir::new("migrate");
        let legacy = game.path.join("Runtime_backup_original");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("packagedefinition.txt"), b"legacy_pkg").unwrap();
        fs::write(legacy.join("chunk0patch1.rpkg"), b"official").unwrap();

        migrate_legacy_backup(&game.path).unwrap();

        let new_pkg = game.path.join(".flmm_backup").join("packagedefinition.txt");
        assert!(new_pkg.exists());
        assert_eq!(fs::read(&new_pkg).unwrap(), b"legacy_pkg");
        assert!(!legacy.exists());
    }

    #[test]
    fn migrate_legacy_backup_skips_if_new_backup_exists() {
        // 新形式が既に存在する場合は移行しない
        let game = TempDir::new("migrate_skip");
        let legacy = game.path.join("Runtime_backup_original");
        let new_backup = game.path.join(".flmm_backup");
        fs::create_dir_all(&legacy).unwrap();
        fs::create_dir_all(&new_backup).unwrap();
        fs::write(legacy.join("packagedefinition.txt"), b"old").unwrap();
        fs::write(new_backup.join("packagedefinition.txt"), b"new").unwrap();

        migrate_legacy_backup(&game.path).unwrap();

        assert!(legacy.exists());
        assert_eq!(
            fs::read(new_backup.join("packagedefinition.txt")).unwrap(),
            b"new"
        );
    }

    #[test]
    fn migrate_legacy_backup_skips_if_no_legacy() {
        // 旧形式が存在しない場合は何もしない
        let game = TempDir::new("migrate_noop");
        let result = migrate_legacy_backup(&game.path);
        assert!(result.is_ok());
    }

    #[test]
    fn migrate_legacy_backup_skips_if_no_packagedefinition_in_legacy() {
        // 旧形式に packagedefinition.txt がない場合は移行しない
        let game = TempDir::new("migrate_no_pkg");
        let legacy = game.path.join("Runtime_backup_original");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("chunk0patch1.rpkg"), b"official").unwrap();

        migrate_legacy_backup(&game.path).unwrap();

        assert!(legacy.exists());
        assert!(!game.path.join(".flmm_backup").exists());
    }

}
