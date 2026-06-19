use std::fs;
use std::path::Path;

// XTEA鍵情報
pub const XTEA_KEYS: [u32; 4] = [0x71482CF0, 0x5FDC4B9F, 0x86CE569D, 0x0509FC1E];
pub const XTEA_DELTA: u32 = 0x61C88647;
pub const XTEA_SUM: u32 = 0xC6EF3720;
// 暗号化識別ヘッダー
pub const XTEA_HEADER: [u8; 16] = [
    0xB7, 0xE2, 0xEA, 0x00, 0x54, 0x5B, 0x6B, 0x87, 0x11, 0xBD, 0x6F, 0xE8, 0x4D, 0x6A, 0xD4, 0xBF,
];

// パッケージ定義構造体
#[derive(Debug)]
pub struct PackageDefinition {
    pub content: String,
    pub encrypted: bool,
}

// XTEAブロック復号
pub fn decrypt_block_xtea(a: &mut u32, b: &mut u32, keys: &[u32; 4]) {
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

// XTEAブロック暗号
pub fn encrypt_block_xtea(a: &mut u32, b: &mut u32, keys: &[u32; 4]) {
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

// バッファ復号
pub fn decrypt_buffer(data: &[u8]) -> Vec<u8> {
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

// バッファ暗号
pub fn encrypt_buffer(data: &[u8]) -> Vec<u8> {
    let mut padded = data.to_vec();
    while padded.len() % 8 != 0 {
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

// CRC32計算
pub fn crc32_ieee(data: &[u8]) -> u32 {
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

// Latin-1文字列デコード
pub fn decode_to_string(data: Vec<u8>) -> String {
    data.iter().map(|&b| b as char).collect()
}

// パッケージ定義ファイル読み込み
pub fn read_packagedefinition(runtime: &Path) -> Result<PackageDefinition, String> {
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

// パッケージ定義ファイル書き込み
pub fn write_packagedefinition(runtime: &Path, definition: &PackageDefinition) -> Result<(), String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    #[test]
    fn decode_to_string_is_latin1_fixed() {
        let ascii = "hello".as_bytes().to_vec();
        assert_eq!(decode_to_string(ascii), "hello");

        let latin1 = vec![0x68, 0x65, 0x6C, 0x6C, 0xE9];
        assert_eq!(decode_to_string(latin1), "hell\u{00e9}");
    }

    #[test]
    fn read_packagedefinition_strips_utf8_bom() {
        let game = TempDir::new("bom");
        let runtime = game.path.join("Runtime");
        fs::create_dir_all(&runtime).unwrap();

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

        let content = b"// copyright \xA9 test\r\n@partition name=super parent=none type=standard patchlevel=0\r\n".to_vec();
        fs::write(runtime.join("packagedefinition.txt"), &content).unwrap();

        let definition = read_packagedefinition(&runtime).unwrap();
        assert!(!definition.encrypted);
        assert!(definition.content.contains("©"));
        assert!(definition.content.contains("@partition"));
    }

    #[test]
    fn crc32_ieee_known_values() {
        let crc = crc32_ieee(b"123456789");
        assert_eq!(crc, 0xCBF43926);
    }
}
