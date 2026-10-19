// File Path: beam-formats/examples/verify_local_beam.rs
// Purpose: native Rust read-only BEAM v1 verification for private/local Renewal test candidates
// Module: Rust executable with synthetic fixture regression tests
// Version: 0.1.1-candidate
// Description: never calls Grf::save, writes only disposable synthetic BEAM files in tests
// Dependency Layer: beam-formats::beam::BeamArchive

use beam_formats::beam::BeamArchive;
use std::error::Error;
use std::path::{Path, PathBuf};

const MAX_FILE: u32 = 32 * 1024 * 1024;
const MAX_ENTRY_COUNT: usize = 40;

fn inspect(path: &Path) -> Result<(usize, u64), Box<dyn Error>> {
    if path.extension().and_then(|v| v.to_str()) != Some("beam") {
        return Err("Only local .beam candidates can be validated".into());
    }
    let archive = BeamArchive::open(path)?;
    if archive.version != 1 {
        return Err("Unexpected BEAM archive version".into());
    }
    let names = archive.list_files();
    if names.is_empty() || names.len() > MAX_ENTRY_COUNT {
        return Err("Unexpected GRF source entry count".into());
    }
    let mut count = 0usize;
    let mut uncompressed_bytes = 0u64;
    for logical in names {
        let entry = archive.get_entry(logical).ok_or("Missing BEAM entry")?;
        if !logical.is_ascii() || !logical.starts_with("data/") {
            return Err("Non-ASCII or non-data BEAM entry refused".into());
        }
        if logical.split('/').any(|p| p == ".." || p.is_empty()) {
            return Err("Unsafe logical path".into());
        }
        let target = entry.grf_path.as_deref().ok_or("GRF destination alias is required")?;
        if !target.is_ascii()
            || !target.starts_with("data\\")
            || target != logical.replace('/', "\\")
        {
            return Err("BEAM alias does not match exact Windows GRF destination".into());
        }
        let leaf = logical.rsplit('/').next().unwrap_or("").to_ascii_lowercase();
        if ["data.ini", "clientinfo.xml", "sclientinfo.xml", "sclientinfo2.xml"]
            .contains(&leaf.as_str())
        {
            return Err("Protected client configuration file in GRF candidate".into());
        }
        if entry.uncompressed_size > MAX_FILE {
            return Err("Oversized candidate GRF source".into());
        }
        if !archive.verify_file(logical)? {
            return Err("BEAM readback MD5 verification failed".into());
        }
        count += 1;
        uncompressed_bytes += u64::from(entry.uncompressed_size);
    }
    Ok((count, uncompressed_bytes))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut argv = std::env::args_os();
    let _binary = argv.next();
    let path: PathBuf = argv
        .next()
        .ok_or("Usage: cargo run -p beam-formats --example verify_local_beam -- <private-local.beam>")?
        .into();
    if argv.next().is_some() {
        return Err("Exactly one local .beam file path is required".into());
    }
    let (count, uncompressed_bytes) = inspect(&path)?;
    println!(
        "BEAM_V1_NATIVE_READBACK_OK entries={count} original_bytes={uncompressed_bytes} write_operations=0"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn temp_file() -> PathBuf {
        let index = NEXT.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "beam_local_readback_{}_{}.beam",
            std::process::id(), index
        ))
    }

    #[test]
    fn valid_grf_destination_alias_is_verified_using_real_beam_reader() {
        let path = temp_file();
        let mut archive = BeamArchive::new();
        archive
            .add_file_with_grf_path(
                "data/luafiles514/lua files/skillinfoz/skilldescript.lub",
                "data\\luafiles514\\lua files\\skillinfoz\\skilldescript.lub",
                b"synthetic renewal skill payload",
            )
            .unwrap();
        archive.save(&path).unwrap();
        assert_eq!(inspect(&path).unwrap(), (1, 31));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn blocked_server_configuration_never_counts_as_grf_candidate() {
        let path = temp_file();
        let mut archive = BeamArchive::new();
        archive
            .add_file_with_grf_path(
                "data/clientinfo.xml",
                "data\\clientinfo.xml",
                b"<server>private</server>",
            )
            .unwrap();
        archive.save(&path).unwrap();
        assert!(inspect(&path).is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn malformed_destination_alias_is_rejected_even_when_md5_is_valid() {
        let path = temp_file();
        let mut archive = BeamArchive::new();
        archive
            .add_file_with_grf_path(
                "data/skillinfo.lub",
                "data\\different-skillinfo.lub",
                b"synthetic test",
            )
            .unwrap();
        archive.save(&path).unwrap();
        assert!(inspect(&path).is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn loose_systemen_does_not_become_grf_asset() {
        let path = temp_file();
        let mut archive = BeamArchive::new();
        archive
            .add_file_with_grf_path("SystemEN/Towninfo.lub", "SystemEN\\Towninfo.lub", b"test")
            .unwrap();
        archive.save(&path).unwrap();
        assert!(inspect(&path).is_err());
        let _ = std::fs::remove_file(path);
    }
}
