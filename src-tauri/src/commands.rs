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

    // Runtime 内の既存パッチを収集
    for entry in fs::read_dir(runtime).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some((chunk, patch, _)) = parse_patch_file_name(&name) {
            used.insert((chunk, patch));
        }
    }

    // バックアップ内の公式パッチを予約スロットとして追加
    let backup_runtime = runtime
        .parent()
        .map(|game| game.join("Runtime_backup_original"));
    if let Some(ref backup) = backup_runtime {
        if backup.is_dir() {
            for entry in fs::read_dir(backup).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some((chunk, patch, _)) = parse_patch_file_name(&name) {
                    used.insert((chunk, patch));
                }
            }
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

    let backup = game_dir.join("Runtime_backup_original");
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
        backup_exists: backup.exists(),
    }
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
    let backup = game_dir.join("Runtime_backup_original");
    let mod_file = PathBuf::from(&mod_path);

    if !mod_file.exists() {
        return Err(localized(
            is_pt,
            &format!("Arquivo não encontrado: {mod_path}"),
            &format!("File not found: {mod_path}"),
        ));
    }

    if !backup.exists() {
        copy_dir_all(&runtime, &backup).map_err(|e| e.to_string())?;
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
                "File must be .rpkg or .zip",
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

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !src.exists() {
        return Ok(());
    }
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

#[tauri::command]
pub async fn uninstall_mod(game_path: String, lang: String) -> Result<String, String> {
    let is_pt = lang == "pt";
    let game_dir = normalize_game_path(&game_path)?;
    let runtime = game_dir.join("Runtime");
    let backup = game_dir.join("Runtime_backup_original");
    let marker = game_dir.join(".flmm_installed");

    if !backup.exists() {
        return Err(localized(
            is_pt,
            "Backup não encontrado. Não é possível desinstalar com segurança.",
            "Backup not found. Cannot safely uninstall.",
        ));
    }

    if runtime.exists() {
        fs::remove_dir_all(&runtime).map_err(|e| e.to_string())?;
    }
    copy_dir_all(&backup, &runtime).map_err(|e| e.to_string())?;
    fs::remove_dir_all(&backup).map_err(|e| e.to_string())?;
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
    let backup = game_dir.join("Runtime_backup_original");
    if backup.exists() {
        fs::remove_dir_all(backup).map_err(|e| e.to_string())?;
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

    fn unique_temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("flmm_{name}_{suffix}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_packagedefinition() -> String {
        "// --- Chunk Boot + PlayGo\r\n@partition name=super parent=none type=standard patchlevel=0\r\n[assembly:/_glacier/ini/globalresources.ini].pc_resourceidx\r\n// --- Chunk Rest of missions\r\n@partition name=base parent=super type=standard patchlevel=0\r\n".to_string()
    }

    fn write_encrypted_packagedefinition(runtime: &Path, content: &str) {
        let definition = PackageDefinition {
            content: content.to_string(),
            encrypted: true,
        };
        write_packagedefinition(runtime, &definition).unwrap();
    }

    #[test]
    fn refresh_package_definition_sets_patchlevel_and_preserves_crlf() {
        let game = unique_temp_dir("pkg");
        let runtime = game.join("Runtime");
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

        fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn inactive_mods_lower_patchlevel_to_zero() {
        let game = unique_temp_dir("toggle");
        let runtime = game.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();
        write_encrypted_packagedefinition(&runtime, &sample_packagedefinition());
        fs::write(runtime.join("chunk0patch2.rpkg.disabled"), b"dummy").unwrap();

        refresh_package_definition(&runtime).unwrap();
        let definition = read_packagedefinition(&runtime).unwrap();

        assert!(definition
            .content
            .contains("@partition name=super parent=none type=standard patchlevel=0"));
        fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn assigns_mod_to_patch100_slot() {
        let runtime = unique_temp_dir("assign");
        let pending = PendingRpkg {
            original_name: "chunk0patch1.rpkg".to_string(),
            data: vec![1, 2, 3],
            size: 3,
            chunk: 0,
            requested_patch: 1,
        };

        let assigned = assign_rpkg_targets(vec![pending], Some(&runtime)).unwrap();

        assert_eq!(assigned[0].target_name, "chunk0patch100.rpkg");
        fs::remove_dir_all(runtime).unwrap();
    }

    #[test]
    fn inspect_zip_reports_ignored_packagedefinition() {
        let root = unique_temp_dir("zip");
        let zip_path = root.join("mod.zip");
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

        let preview = inspect_mod_file(&zip_path, Some(&root)).unwrap();

        assert!(preview.installable);
        assert!(preview.has_packagedefinition);
        assert!(preview.has_metadata);
        assert_eq!(preview.rpkg_files[0].target_name, "chunk0patch100.rpkg");
        assert!(preview
            .warnings
            .iter()
            .any(|warning| warning.contains("ignored")));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn decode_to_string_is_latin1_fixed() {
        // Latin-1 固定: 各バイトをそのまま Unicode コードポイントとして扱う
        let ascii = "hello".as_bytes().to_vec();
        assert_eq!(decode_to_string(ascii), "hello");

        // 0xE9 = Latin-1 で 'é'
        let latin1 = vec![0x68, 0x65, 0x6C, 0x6C, 0xE9];
        assert_eq!(decode_to_string(latin1), "hell\u{00e9}");
    }

    #[test]
    fn read_packagedefinition_strips_utf8_bom() {
        let game = unique_temp_dir("bom");
        let runtime = game.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        // BOM + ASCII テキスト
        let mut content = vec![0xEF, 0xBB, 0xBF];
        content.extend_from_slice(b"@partition name=super parent=none type=standard patchlevel=0\r\n");
        fs::write(runtime.join("packagedefinition.txt"), &content).unwrap();

        let definition = read_packagedefinition(&runtime).unwrap();
        assert!(!definition.encrypted);
        assert!(definition.content.starts_with("@partition"));

        fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn read_packagedefinition_handles_latin1_content() {
        let game = unique_temp_dir("latin1");
        let runtime = game.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        // 0xA9 (©), 0xE9 (é) — Latin-1 で有効、UTF-8 では不正
        let content = b"// copyright \xA9 test\r\n@partition name=super parent=none type=standard patchlevel=0\r\n".to_vec();
        fs::write(runtime.join("packagedefinition.txt"), &content).unwrap();

        let definition = read_packagedefinition(&runtime).unwrap();
        assert!(!definition.encrypted);
        assert!(definition.content.contains("©"));
        assert!(definition.content.contains("@partition"));

        fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn backup_patches_are_reserved_slots() {
        let game = unique_temp_dir("backup_reserve");
        let runtime = game.join("Runtime");
        let backup = game.join("Runtime_backup_original");
        fs::create_dir_all(&runtime).unwrap();
        fs::create_dir_all(&backup).unwrap();

        // バックアップに公式パッチ chunk0patch1 が存在
        fs::write(backup.join("chunk0patch1.rpkg"), b"official").unwrap();

        let pending = PendingRpkg {
            original_name: "chunk0patch1.rpkg".to_string(),
            data: vec![1, 2, 3],
            size: 3,
            chunk: 0,
            requested_patch: 1,
        };

        let assigned = assign_rpkg_targets(vec![pending], Some(&runtime)).unwrap();

        // バックアップに公式パッチ chunk0patch1 が存在しても MOD_PATCH_START=100 から開始
        assert_eq!(assigned[0].target_name, "chunk0patch100.rpkg");
        fs::remove_dir_all(game).unwrap();
    }

    #[test]
    fn backup_patches_skip_occupied_slots() {
        let game = unique_temp_dir("backup_skip");
        let runtime = game.join("Runtime");
        let backup = game.join("Runtime_backup_original");
        fs::create_dir_all(&runtime).unwrap();
        fs::create_dir_all(&backup).unwrap();

        // バックアップに chunk1patch1, Runtime に chunk1patch2 が存在
        fs::write(backup.join("chunk1patch1.rpkg"), b"official").unwrap();
        fs::write(runtime.join("chunk1patch2.rpkg"), b"existing_mod").unwrap();

        let pending = PendingRpkg {
            original_name: "chunk1patch1.rpkg".to_string(),
            data: vec![4, 5, 6],
            size: 3,
            chunk: 1,
            requested_patch: 1,
        };

        let assigned = assign_rpkg_targets(vec![pending], Some(&runtime)).unwrap();

        // patch100 以降を割り当て (patch1 公式・patch2 既存MODは無関係)
        assert_eq!(assigned[0].target_name, "chunk1patch100.rpkg");
        fs::remove_dir_all(game).unwrap();
    }
}
