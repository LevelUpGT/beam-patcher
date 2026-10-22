// Dosya Yolu: REPO_ROOT/beam-formats/src/bin/verify_map_beam.rs
// Amac: Python BEAM map candidate files validated with the actual Rust reader
// Modul - Rust offline QA executable
// Version: 0.1.0-candidate
// Aciklama: Read-only SHA256 and source bytes gate, NO GRF, client or network writes
// Bagimli Oldugu Katman: beam-formats::beam::BeamArchive | RagnarokClient maps/manifest.json

use beam_formats::beam::BeamArchive;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

type VerifyResult<T> = Result<T, Box<dyn std::error::Error>>;

fn fail(reason: &str) -> Box<dyn std::error::Error> {
    reason.to_string().into()
}

fn valid_group(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.bytes().next().is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && s.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_' || c == b'-')
}
fn valid_map(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 11
        && s.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'@' | b'-'))
}
fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
fn regular_file(path: &Path, max_size: u64) -> VerifyResult<Vec<u8>> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > max_size {
        return Err(fail("SOURCE_FILE_UNSAFE_OR_OVERSIZED"));
    }
    Ok(fs::read(path)?)
}

fn verify(archive_path: &Path, maps_root: &Path, catalog: &Path) -> VerifyResult<usize> {
    let archive_meta = fs::symlink_metadata(archive_path)?;
    if !archive_meta.is_file() || archive_meta.file_type().is_symlink()
        || !(64..=536_870_912).contains(&archive_meta.len())
    {
        return Err(fail("BEAM_ARCHIVE_SIZE_OR_LINK_INVALID"));
    }
    // Bound the untrusted archive's entry count before its Rust reader allocates.
    let mut file = fs::File::open(archive_path)?;
    let mut header = [0u8; 12];
    file.read_exact(&mut header)?;
    if &header[..4] != b"BEAM" || u32::from_le_bytes(header[4..8].try_into()?) != 1
        || !(1..=150).contains(&u32::from_le_bytes(header[8..12].try_into()?))
    {
        return Err(fail("BEAM_HEADER_OR_COUNT_INVALID"));
    }
    file.seek(SeekFrom::Start(0))?;

    let manifest: Value = serde_json::from_slice(&regular_file(catalog, 131_072)?)?;
    if manifest.get("schema").and_then(Value::as_str) != Some("levelupgt.addon-map-assets.v2") {
        return Err(fail("MAP_CATALOG_SCHEMA_INVALID"));
    }
    let maps = manifest
        .get("maps")
        .and_then(Value::as_array)
        .ok_or_else(|| fail("MAP_CATALOG_MISSING_MAPS"))?;
    if maps.is_empty() || maps.len() > 50 {
        return Err(fail("MAP_CATALOG_MAP_COUNT_INVALID"));
    }
    if u32::from_le_bytes(header[8..12].try_into()?) as usize != maps.len() * 3 {
        return Err(fail("BEAM_MAP_ENTRY_COUNT_MISMATCH"));
    }
    let entries = BeamArchive::open(archive_path)?;
    if entries.version != 1 {
        return Err(fail("BEAM_ARCHIVE_VERSION_MISMATCH"));
    }
    let mut expected = BTreeSet::new();
    let mut used_map_names = BTreeSet::new();
    let mut total: u64 = 0;
    for row in maps {
        let group = row.get("group").and_then(Value::as_str).ok_or_else(|| fail("GROUP_MISSING"))?;
        let map = row.get("name").and_then(Value::as_str).ok_or_else(|| fail("MAP_NAME_MISSING"))?;
        if !valid_group(group) || !valid_map(map)
            || !used_map_names.insert(map.to_ascii_lowercase())
        {
            return Err(fail("GROUP_OR_MAP_NAME_INVALID_OR_DUPLICATE"));
        }
        let group_dir = maps_root.join("data").join(group);
        let dir_meta = fs::symlink_metadata(&group_dir)?;
        if !dir_meta.is_dir() || dir_meta.file_type().is_symlink() {
            return Err(fail("MAP_GROUP_DIRECTORY_UNSAFE"));
        }
        for extension in ["gat", "gnd", "rsw"] {
            let filename = format!("{map}.{extension}");
            let archive_name = format!("data\\{filename}");
            let source_path = group_dir.join(&filename);
            let expected_hash = row
                .get(format!("{extension}_sha256"))
                .and_then(Value::as_str)
                .ok_or_else(|| fail("SOURCE_SHA_MISSING"))?;
            if expected_hash.len() != 64 || !expected_hash.bytes().all(|c| c.is_ascii_hexdigit()) {
                return Err(fail("SOURCE_SHA_INVALID"));
            }
            let source = regular_file(&source_path, 32 * 1024 * 1024)?;
            total += source.len() as u64;
            if total > 512 * 1024 * 1024
                || !sha256_hex(&source).eq_ignore_ascii_case(expected_hash)
            {
                return Err(fail("SOURCE_HASH_OR_SIZE_INVALID"));
            }
            if !expected.insert(archive_name.clone()) {
                return Err(fail("BEAM_DUPLICATE_DESTINATION"));
            }
            let entry = entries.get_entry(&archive_name)
                .ok_or_else(|| fail("BEAM_MISSING_EXPECTED_MAP_FILE"))?;
            if entry.grf_path.is_some() {
                return Err(fail("BEAM_UNEXPECTED_GRF_PATH_ALIAS"));
            }
            if entry.uncompressed_size as usize != source.len() {
                return Err(fail("BEAM_SOURCE_SIZE_MISMATCH"));
            }
            let extracted = entries.extract_file(&archive_name)?;
            if extracted != source {
                return Err(fail("BEAM_RUST_READER_SOURCE_MISMATCH"));
            }
        }
    }
    let actual: BTreeSet<String> = entries.list_files().into_iter().map(str::to_string).collect();
    if actual != expected {
        return Err(fail("BEAM_EXTRA_OR_MISSING_ENTRIES"));
    }
    Ok(maps.len())
}

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 4 {
        eprintln!("USAGE: verify_map_beam <custom-maps.beam> <client-maps-root> <maps/manifest.json>");
        std::process::exit(2);
    }
    let archive = PathBuf::from(&args[1]);
    let maps = PathBuf::from(&args[2]);
    let manifest = PathBuf::from(&args[3]);
    match verify(&archive, &maps, &manifest) {
        Ok(count) => println!(
            "BEAM_RUST_READBACK_OK maps={count} files={} offline=true",
            count * 3
        ),
        Err(err) => {
            eprintln!("BEAM_RUST_READBACK_REJECTED: {err}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_map_and_group() {
        assert!(valid_group("overlookwaterdungeon"));
        assert!(valid_map("iz_dun00t"));
        assert!(!valid_group("../other"));
        assert!(!valid_map("this_is_way_too_long"));
        assert!(!valid_map("bad/path"));
    }
}
