use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

use crate::crypto::{
    read_packagedefinition, write_packagedefinition,
};
use crate::backup::{
    backup_packagedefinition_only, restore_packagedefinition, migrate_legacy_backup,
};
use crate::settings::{
    normalize_game_path,
};

// MOD開始インデックス
pub const MOD_PATCH_START: u32 = 100;

// MOD情報構造体
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

// パッケージプレビュー構造体
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

// プレビュー用RPKG情報
#[derive(Serialize, Deserialize, Clone)]
pub struct PreviewRpkg {
    pub original_name: String,
    pub target_name: String,
    pub chunk: u32,
    pub requested_patch: u32,
    pub target_patch: u32,
    pub size: u64,
}

// 格納メタデータ構造体
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct StoredModMetadata {
    pub name: String,
    pub author: String,
    pub description: String,
    pub version: String,
    pub original_filename: String,
    pub installed_filename: String,
    pub source_package: String,
    pub installed_at: String,
}

// 保留中RPKG情報
pub struct PendingRpkg {
    pub original_name: String,
    pub data: Vec<u8>,
    pub size: u64,
    pub chunk: u32,
    pub requested_patch: u32,
}

// 割り当て済みRPKG情報
pub struct AssignedRpkg {
    pub pending: PendingRpkg,
    pub target_name: String,
    pub target_patch: u32,
}

// 改行コード検出
pub fn detect_line_ending(content: &str) -> &'static str {
    if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

// 行内パッチレベル置換
pub fn set_patchlevel_in_line(line: &str, level: u32) -> String {
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

// パーティションごとのパッチレベル取得
pub fn patchlevels_by_partition(runtime: &Path) -> Result<HashMap<u32, u32>, String> {
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

// パッチレベル適用
pub fn apply_patchlevels(content: &str, patchlevels: &HashMap<u32, u32>) -> Result<String, String> {
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

// パッケージ定義再生成
pub fn refresh_package_definition(runtime: &Path) -> Result<(), String> {
    let mut definition = read_packagedefinition(runtime)?;
    let patchlevels = patchlevels_by_partition(runtime)?;
    definition.content = apply_patchlevels(&definition.content, &patchlevels)?;
    write_packagedefinition(runtime, &definition)
}

// パッチファイル名パース
pub fn parse_patch_file_name(name: &str) -> Option<(u32, u32, bool)> {
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

// 開始パッチインデックス取得
pub fn target_patch_start(_chunk: u32) -> u32 {
    MOD_PATCH_START
}

// 使用済みパッチスロット収集
pub fn used_patch_slots(runtime: Option<&Path>) -> Result<HashSet<(u32, u32)>, String> {
    let mut used = HashSet::new();
    let Some(runtime) = runtime else {
        return Ok(used);
    };
    if !runtime.is_dir() {
        return Ok(used);
    }

    for entry in fs::read_dir(runtime).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some((chunk, patch, _)) = parse_patch_file_name(&name) {
            used.insert((chunk, patch));
        }
    }

    Ok(used)
}

// 対象RPKGスロット割り当て
pub fn assign_rpkg_targets(
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

// JSONからメタデータ構造体変換
pub fn metadata_from_json(content: &str) -> StoredModMetadata {
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

// メタデータファイル読み込み
pub fn read_stored_metadata(path: &Path) -> StoredModMetadata {
    let Ok(content) = fs::read_to_string(path) else {
        return StoredModMetadata::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

// UNIXタイムスタンプ文字列取得
pub fn now_unix_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

// RPKGファイルから保留中情報抽出
pub fn pending_from_rpkg_path(path: &Path) -> Result<PendingRpkg, String> {
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

// ZIPパッケージ読み込み
pub fn read_zip_package(
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

// MODファイル検査
pub fn inspect_mod_file(path: &Path, runtime: Option<&Path>) -> Result<ModPreview, String> {
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

// 現パッチレベル取得
pub fn current_patchlevels(runtime: &Path) -> HashMap<u32, u32> {
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

// メッセージローカライズ
pub fn localized(is_pt: bool, pt: &str, en: &str) -> String {
    if is_pt {
        pt.to_string()
    } else {
        en.to_string()
    }
}

// MODインストール処理
pub fn install_mod_internal(
    game_path: &str,
    mod_path: &str,
    lang: &str,
) -> Result<String, String> {
    let is_pt = lang == "pt";
    let game_dir = normalize_game_path(game_path)?;
    let runtime = game_dir.join("Runtime");
    let mod_file = PathBuf::from(mod_path);

    if !mod_file.exists() {
        return Err(localized(
            is_pt,
            &format!("Arquivo não encontrado: {mod_path}"),
            &format!("File not found: {mod_path}"),
        ));
    }

    migrate_legacy_backup(&game_dir)?;

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

// MODアンインストール処理
pub fn uninstall_mod_internal(game_path: &str, lang: &str) -> Result<String, String> {
    let is_pt = lang == "pt";
    let game_dir = normalize_game_path(game_path)?;
    let runtime = game_dir.join("Runtime");
    let new_backup = game_dir.join(".flmm_backup");
    let marker = game_dir.join(".flmm_installed");

    migrate_legacy_backup(&game_dir)?;

    if !new_backup.exists() {
        return Err(localized(
            is_pt,
            "Backup não encontrado. Não é possível desinstalar com segurança.",
            "Backup not found. Cannot safely uninstall.",
        ));
    }

    if runtime.is_dir() {
        for entry in fs::read_dir(&runtime).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let name = entry.file_name().to_string_lossy().to_string();
            let lower = name.to_ascii_lowercase();
            let is_mod_rpkg = parse_patch_file_name(&name)
                .map(|(_, patch, _)| patch >= MOD_PATCH_START)
                .unwrap_or(false);
            let is_mod_meta = lower.ends_with(".metadata.json");
            if is_mod_rpkg || is_mod_meta {
                fs::remove_file(entry.path()).map_err(|e| e.to_string())?;
            }
        }
    }

    restore_packagedefinition(&game_dir)?;

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

// MOD一覧取得
pub fn list_mods_internal(game_path: &str) -> Result<Vec<ModInfo>, String> {
    let game_dir = normalize_game_path(game_path)?;
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

// MOD有効/無効トグル
pub fn toggle_mod_internal(game_path: &str, mod_id: &str, active: bool) -> Result<(), String> {
    let game_dir = normalize_game_path(game_path)?;
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

// MOD個別削除
pub fn delete_mod_internal(game_path: &str, mod_id: &str) -> Result<(), String> {
    let game_dir = normalize_game_path(game_path)?;
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
    use crate::crypto::{write_packagedefinition, PackageDefinition};
    use std::io::Write;

    // テスト用一時ディレクトリ構造体
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
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
    fn official_patches_in_runtime_are_reserved_slots() {
        let game = TempDir::new("official_reserve");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

        fs::write(runtime.join("chunk0patch1.rpkg"), b"official").unwrap();

        let pending = PendingRpkg {
            original_name: "chunk0patch1.rpkg".to_string(),
            data: vec![1, 2, 3],
            size: 3,
            chunk: 0,
            requested_patch: 1,
        };

        let assigned = assign_rpkg_targets(vec![pending], Some(&runtime)).unwrap();

        assert_eq!(assigned[0].target_name, "chunk0patch100.rpkg");
    }

    #[test]
    fn official_patches_skip_occupied_slots() {
        let game = TempDir::new("official_skip");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

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

        assert_eq!(assigned[0].target_name, "chunk1patch100.rpkg");
    }

    #[test]
    fn parse_patch_file_name_active_rpkg() {
        let result = parse_patch_file_name("chunk0patch2.rpkg");
        assert_eq!(result, Some((0, 2, true)));
    }

    #[test]
    fn parse_patch_file_name_disabled_rpkg() {
        let result = parse_patch_file_name("chunk1patch100.rpkg.disabled");
        assert_eq!(result, Some((1, 100, false)));
    }

    #[test]
    fn parse_patch_file_name_case_insensitive() {
        let result = parse_patch_file_name("Chunk2Patch5.RPKG");
        assert_eq!(result, Some((2, 5, true)));
    }

    #[test]
    fn parse_patch_file_name_non_rpkg_returns_none() {
        assert_eq!(parse_patch_file_name("packagedefinition.txt"), None);
        assert_eq!(parse_patch_file_name("mod.json"), None);
        assert_eq!(parse_patch_file_name("chunk0.rpkg"), None);
    }

    #[test]
    fn parse_patch_file_name_large_numbers() {
        let result = parse_patch_file_name("chunk99patch9999.rpkg");
        assert_eq!(result, Some((99, 9999, true)));
    }

    #[test]
    fn set_patchlevel_in_line_replaces_existing_value() {
        let line = "@partition name=super parent=none type=standard patchlevel=0";
        let result = set_patchlevel_in_line(line, 100);
        assert!(result.contains("patchlevel=100"));
        assert!(!result.contains("patchlevel=0 ") && !result.ends_with("patchlevel=0"));
    }

    #[test]
    fn set_patchlevel_in_line_appends_when_missing() {
        let line = "@partition name=base parent=super type=standard";
        let result = set_patchlevel_in_line(line, 50);
        assert!(result.contains("patchlevel=50"));
    }

    #[test]
    fn set_patchlevel_in_line_preserves_surrounding_content() {
        let line = "@partition name=x patchlevel=5 extra=value";
        let result = set_patchlevel_in_line(line, 200);
        assert!(result.contains("extra=value"));
        assert!(result.contains("patchlevel=200"));
    }

    #[test]
    fn detect_line_ending_crlf() {
        assert_eq!(detect_line_ending("line1\r\nline2\r\n"), "\r\n");
    }

    #[test]
    fn detect_line_ending_lf_only() {
        assert_eq!(detect_line_ending("line1\nline2\n"), "\n");
    }

    #[test]
    fn detect_line_ending_no_newline() {
        assert_eq!(detect_line_ending("single line"), "\n");
    }

    #[test]
    fn apply_patchlevels_updates_correct_partition() {
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
        let content = "@partition name=boot patchlevel=0\n";
        let mut levels = HashMap::new();
        levels.insert(5, 100u32);
        let result = apply_patchlevels(content, &levels);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("chunk5"));
    }

    #[test]
    fn apply_patchlevels_preserves_crlf_trailing_newline() {
        let content = "@partition name=boot patchlevel=0\r\n";
        let levels = HashMap::new();
        let result = apply_patchlevels(content, &levels).unwrap();
        assert!(result.ends_with("\r\n"));
    }
}
