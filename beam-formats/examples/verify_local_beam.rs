// File Path: beam-formats/examples/verify_local_beam.rs
// Purpose: Read-only native Rust BeamArchive v1 check for private/local Renewal GRF test candidate
// Module: Rust example verifier
// Version: 0.1.0-candidate
// Description: Checks aliases, paths and compressed payloads; NO GRF or client writes
// Dependency Layer: beam-formats::beam::BeamArchive

use beam_formats::beam::BeamArchive;
use std::error::Error;
use std::path::PathBuf;

const MAX_FILE: u32 = 32 * 1024 * 1024;
const MAX_ENTRY_COUNT: usize = 40;

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
    if path.extension().and_then(|v| v.to_str()) != Some("beam") {
        return Err("Only local .beam candidates can be validated".into());
    }
    let archive = BeamArchive::open(&path)?;
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
        if !target.is_ascii() || !target.starts_with("data\\") ||
            target != logical.replace('/', "\\") {
            return Err("BEAM alias does not match exact Windows GRF destination".into());
        }
        let leaf = logical.rsplit('/').next().unwrap_or("").to_ascii_lowercase();
        if ["data.ini", "clientinfo.xml", "sclientinfo.xml", "sclientinfo2.xml"].contains(&leaf.as_str()) {
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
    println!(
        "BEAM_V1_NATIVE_READBACK_OK entries={count} original_bytes={uncompressed_bytes} write_operations=0"
    );
    Ok(())
}
