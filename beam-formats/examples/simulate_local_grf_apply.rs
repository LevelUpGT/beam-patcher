// File Path: beam-formats/examples/simulate_local_grf_apply.rs
// Purpose: apply a local-only BEAM to a NEW sandbox GRF and verify full GRF extract readback
// Module: Rust offline application integration test
// Version: 0.1.0-candidate
// Description: ONLY writes below a fresh patches/output/ directory; NEVER opens installed game GRF for writing
// Dependency Layer: beam-formats::beam::BeamArchive, beam-formats::grf::Grf

use beam_formats::beam::BeamArchive;
use beam_formats::grf::Grf;
use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_FILES: usize = 40;
const MAX_EXPANDED: u64 = 128 * 1024 * 1024;
const MAX_SINGLE: u32 = 32 * 1024 * 1024;
const SANDBOX_NAME: &str = "SANDBOX-tdata1.grf";

fn validate_sandbox_parent(outdir: &Path) -> Result<(), Box<dyn Error>> {
    if outdir.exists() || outdir.is_symlink() {
        return Err("Output directory exists; refusing overwrite".into());
    }
    let parent = outdir.parent().ok_or("Sandbox directory needs a parent")?;
    let grandparent = parent.parent().ok_or("Sandbox parent needs a patches directory")?;
    if parent.file_name() != Some(OsStr::new("output"))
        || grandparent.file_name() != Some(OsStr::new("patches"))
    {
        return Err("Sandbox must be a NEW direct subfolder of patches/output".into());
    }
    if !parent.is_dir() || parent.is_symlink() || grandparent.is_symlink() {
        return Err("Sandbox parent directory must be existing and not linked".into());
    }
    Ok(())
}

fn simulate(beam_path: &Path, outdir: &Path) -> Result<(usize, u64), Box<dyn Error>> {
    validate_sandbox_parent(outdir)?;
    if !beam_path.is_file() || beam_path.is_symlink()
        || beam_path.extension() != Some(OsStr::new("beam"))
    {
        return Err("Input must be an existing regular local .beam file".into());
    }

    let archive = BeamArchive::open(beam_path)?;
    if archive.version != 1 {
        return Err("Only BEAM v1 is supported in sandbox validation".into());
    }
    let names = archive.list_files();
    if names.is_empty() || names.len() > MAX_FILES {
        return Err("Unexpected sandbox BEAM entry count".into());
    }

    // Validate every path, size, alias and BEAM MD5 BEFORE the first sandbox write.
    let mut records: Vec<(String, Vec<u8>)> = Vec::with_capacity(names.len());
    let mut targets = HashSet::new();
    let mut total_bytes = 0u64;
    for name in names {
        let entry = archive.get_entry(name).ok_or("Missing BEAM table entry")?;
        if !name.is_ascii() || !name.starts_with("data/")
            || name.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        {
            return Err("Unsupported, unsafe or non-ASCII BEAM logical path".into());
        }
        let alias = entry.grf_path.as_deref().ok_or("Missing explicit GRF alias")?;
        if !alias.is_ascii() || alias != name.replace('/', "\\")
            || !alias.starts_with("data\\")
        {
            return Err("GRF destination alias is not the exact Windows data path".into());
        }
        let leaf = name.rsplit('/').next().unwrap_or("").to_ascii_lowercase();
        if ["data.ini", "clientinfo.xml", "sclientinfo.xml", "sclientinfo2.xml"]
            .contains(&leaf.as_str())
        {
            return Err("Protected client config cannot be applied to sandbox GRF".into());
        }
        if !targets.insert(alias.to_ascii_lowercase()) {
            return Err("Case-insensitive GRF target collision".into());
        }
        if entry.uncompressed_size > MAX_SINGLE {
            return Err("BEAM entry exceeds per-file sandbox bound".into());
        }
        total_bytes += u64::from(entry.uncompressed_size);
        if total_bytes > MAX_EXPANDED {
            return Err("BEAM exceeds total sandbox content bound".into());
        }
        if !archive.verify_file(name)? {
            return Err("Source BEAM MD5 mismatch".into());
        }
        let data = archive.extract_file(name)?;
        if data.len() as u64 != u64::from(entry.uncompressed_size) {
            return Err("Extracted BEAM source size mismatch".into());
        }
        records.push((alias.to_string(), data));
    }

    fs::create_dir(outdir)?; // Atomic NEW output folder; no overwrite or reuse.
    let grf_path = outdir.join(SANDBOX_NAME);
    if grf_path.exists() || grf_path.is_symlink() {
        return Err("Sandbox GRF path already exists".into());
    }
    let mut grf = Grf::create_new(&grf_path)?;
    if grf.version != 0x200 {
        return Err("Unexpected new GRF version".into());
    }
    for (alias, data) in &records {
        grf.patch_file(alias, data)?;
    }
    if !grf.has_pending_changes() {
        return Err("No pending GRF sandbox changes".into());
    }
    grf.save()?; // Only a freshly created SANDBOX-tdata1.grf in NEW patches/output child.
    drop(grf);

    let opened = Grf::open(&grf_path)?;
    if opened.version != 0x200 || opened.file_count() != records.len() {
        return Err("Reopened sandbox GRF version or entry count mismatch".into());
    }
    for (alias, expected) in &records {
        let entry = opened.get_entry(alias).ok_or("GRF table destination was not saved")?;
        if entry.raw_filename.as_slice() != alias.as_bytes() {
            return Err("Raw GRF path bytes changed while saving".into());
        }
        let actual = opened.extract_file(alias)?;
        if actual != *expected {
            return Err("GRF extracted payload differs from original BEAM file".into());
        }
    }
    Ok((records.len(), total_bytes))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os();
    let _program = args.next();
    let beam: PathBuf = args
        .next()
        .ok_or("Usage: simulate_local_grf_apply <existing-local.beam> <NEW-patches/output/sandbox>")?
        .into();
    let output: PathBuf = args
        .next()
        .ok_or("Missing NEW sandbox output directory argument")?
        .into();
    if args.next().is_some() {
        return Err("Extra argument not allowed".into());
    }
    let (count, bytes) = simulate(&beam, &output)?;
    println!(
        "GRF_SANDBOX_ROUNDTRIP_OK entries={count} original_bytes={bytes} installed_client_writes=0"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn fixture() -> (PathBuf, PathBuf, PathBuf) {
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "beam_grf_sandbox_{}_{}", std::process::id(), id
        ));
        let parent = root.join("patches").join("output");
        fs::create_dir_all(&parent).unwrap();
        let beam = root.join("source.beam");
        let sandbox = parent.join("local-only");
        (root, beam, sandbox)
    }

    fn write_beam(path: &Path, name: &str, grf_alias: &str) {
        let mut b = BeamArchive::new();
        b.add_file_with_grf_path(name, grf_alias, b"synthetic replacement of a renewal skill")
            .unwrap();
        b.save(path).unwrap();
    }

    #[test]
    fn apply_to_new_sandbox_and_reopen_original_bytes() {
        let (root, beam, output) = fixture();
        write_beam(&beam, "data/luafiles514/skillinfolist.lub",
                   "data\\luafiles514\\skillinfolist.lub");
        let before = fs::read(&beam).unwrap();
        let (count, size) = simulate(&beam, &output).unwrap();
        assert_eq!((count, size), (1, 40));
        assert_eq!(fs::read(&beam).unwrap(), before);
        let grf = Grf::open(output.join(SANDBOX_NAME)).unwrap();
        assert_eq!(grf.file_count(), 1);
        assert_eq!(
            grf.extract_file("data\\luafiles514\\skillinfolist.lub").unwrap(),
            b"synthetic replacement of a renewal skill"
        );
        assert!(!output.join("DATA.INI").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn refuses_preexisting_output_folder_even_if_empty() {
        let (root, beam, output) = fixture();
        write_beam(&beam, "data/test.lub", "data\\test.lub");
        fs::create_dir(&output).unwrap();
        assert!(simulate(&beam, &output).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn refuses_protected_client_settings_without_creating_sandbox() {
        let (root, beam, output) = fixture();
        write_beam(&beam, "data/clientinfo.xml", "data\\clientinfo.xml");
        assert!(simulate(&beam, &output).is_err());
        assert!(!output.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn refuses_unknown_grf_alias_before_creating_sandbox() {
        let (root, beam, output) = fixture();
        write_beam(&beam, "data/skillinfo.lub", "data\\wrong.lub");
        assert!(simulate(&beam, &output).is_err());
        assert!(!output.exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn refuses_output_path_outside_patches_output() {
        let (root, beam, _output) = fixture();
        write_beam(&beam, "data/skillinfo.lub", "data\\skillinfo.lub");
        assert!(simulate(&beam, &root.join("game").join("tdata1")).is_err());
        let _ = fs::remove_dir_all(root);
    }
}
