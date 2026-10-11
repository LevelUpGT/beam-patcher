// 📄 Dosya Yolu: /LevelUpGT/beam-patcher/beam-preview/src/tools/package_reader.rs
// 📌 Amac: Kullanici tarafindan secilen yerel paket dosyasinin SHA-256 ozetini okumak
// 📌 Modul - Rust Tool
// Version: 1.0.0
// Aciklama: Akis tabanli SHA-256; symlink, boyut siniri ve degisen dosyayi reddeder
// Bagimli Oldugu Katman: Service

use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;
use crate::config::client_profiles;

const READ_CHUNK: usize = 64 * 1024;

pub struct PackageDigest {
    pub sha256: String,
    pub size: u64,
}

pub fn sha256_local(path: &str, expected_size: u64) -> Result<PackageDigest, String> {
    if expected_size == 0 || expected_size > client_profiles::MAX_ARCHIVE_BYTES {
        return Err("Beyan edilen paket boyutu gecersiz.".into());
    }
    let file_path = Path::new(path);
    let metadata = std::fs::symlink_metadata(file_path)
        .map_err(|_| "Yerel paket bulunamadi.".to_string())?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err("Yerel paket normal bir dosya olmali.".into());
    }
    if metadata.len() != expected_size {
        return Err("Yerel paket boyutu manifestle eslesmiyor.".into());
    }
    let mut file = std::fs::File::open(file_path)
        .map_err(|_| "Paket dosyasi acilamadi.".to_string())?;
    let opened = file.metadata().map_err(|_| "Paket bilgisi okunamadi.".to_string())?;
    if !opened.is_file() || opened.len() != expected_size {
        return Err("Paket dosyasi acilirken degisti.".into());
    }
    let mut hasher = Sha256::new();
    let mut size = 0u64;
    let mut buffer = [0u8; READ_CHUNK];
    loop {
        let count = file.read(&mut buffer)
            .map_err(|_| "Paket verisi okunamadi.".to_string())?;
        if count == 0 { break; }
        size = size.checked_add(count as u64)
            .ok_or("Paket boyutu siniri asildi.")?;
        if size > expected_size || size > client_profiles::MAX_ARCHIVE_BYTES {
            return Err("Yerel paket beklenen boyutu asti.".into());
        }
        hasher.update(&buffer[..count]);
    }
    if size != expected_size
        || file.metadata().map_err(|_| "Paket son boyutu okunamadi.".to_string())?.len() != size
    {
        return Err("Yerel paket okuma sirasinda degisti.".into());
    }
    Ok(PackageDigest { sha256: format!("{:x}", hasher.finalize()), size })
}
