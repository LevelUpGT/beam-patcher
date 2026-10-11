// 📄 Dosya Yolu: /LevelUpGT/beam-patcher/beam-preview/src/tools/manifest_reader.rs
// 📌 Amac: Salt okunur yerel manifest JSON dosyalarini sinirli boyutta okumak
// 📌 Modul - Tool Rust
// Version: 1.0.0
// Aciklama: Dosya tipini, uzantisini ve sinirli bayt okumasini uygular
// Bagimli Oldugu Katman: Service

use std::io::Read;
use std::path::Path;

pub fn read_local_json(path: &str, max_bytes: usize) -> Result<Vec<u8>, String> {
    let file_path = Path::new(path);
    if file_path.extension().and_then(|v| v.to_str())
        .map(|ext| !ext.eq_ignore_ascii_case("json")).unwrap_or(true)
    {
        return Err("Yalnizca yerel JSON dosyalari acilabilir.".to_string());
    }
    let meta = std::fs::symlink_metadata(file_path)
        .map_err(|_| "Manifest dosyasi okunamadi.".to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink()
        || meta.len() == 0 || meta.len() > max_bytes as u64
    {
        return Err("Manifest dosya tipi veya boyutu gecersiz.".to_string());
    }
    let file = std::fs::File::open(file_path)
        .map_err(|_| "Manifest dosyasi acilamadi.".to_string())?;
    let mut bytes = Vec::new();
    file.take((max_bytes + 1) as u64).read_to_end(&mut bytes)
        .map_err(|_| "Manifest dosyasi okunamadi.".to_string())?;
    if bytes.is_empty() || bytes.len() > max_bytes {
        return Err("Manifest dosyasi boyutu degisti veya gecersiz.".to_string());
    }
    Ok(bytes)
}
