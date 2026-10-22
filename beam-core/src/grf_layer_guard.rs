//! File Path: beam-core/src/grf_layer_guard.rs
//! Purpose: Prevent protected 20220406 custom Beam packages from writing to an unregistered tdata1.grf
//! Module: Rust Tool / client runtime preflight
//! Version: 0.1.0-candidate
//! Description: Read-only DATA.INI/GRF checks before downloader or any game-directory writes
//! Dependency Layer: Config | CompatibilityGuard | GRF
//! This is necessary but NOT proof of client multiple-GRF support or signed publication.

use crate::{Config, Error, Result};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

const OVERLAY: &str = "tdata1.grf";
const GRF_HEADER: &[u8; 15] = b"Master of Magic";

fn deny(reason: &str) -> Error {
    Error::PatchFailed(format!("tdata1 custom GRF preflight: {reason}"))
}

fn basename(name: &str) -> bool {
    let raw = name.as_bytes();
    !raw.is_empty()
        && name.to_ascii_lowercase().ends_with(".grf")
        && raw[0].is_ascii_alphanumeric()
        && raw.iter().all(|b| b.is_ascii_alphanumeric() || matches!(*b, b'.' | b'_' | b'-'))
}

fn parsed_ini_order(content: &[u8]) -> Result<Vec<(usize, String)>> {
    if content.len() > 64 * 1024 || content.contains(&0) {
        return Err(deny("DATA.INI too large or binary"));
    }
    // Latin-1/CP1254 bytes in unrelated sections must remain untouched;
    // read-only parsing considers only ASCII section and priority rows.
    let mut in_data = false;
    let mut found_data = false;
    let mut names = HashSet::new();
    let mut slots = HashSet::new();
    let mut found = Vec::new();
    for raw in content.split(|ch| *ch == b'\n') {
        let text = String::from_utf8_lossy(raw);
        let line = text.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_data = line[1..line.len() - 1].eq_ignore_ascii_case("Data");
            if in_data {
                if found_data {
                    return Err(deny("duplicate [Data] sections"));
                }
                found_data = true;
            }
            continue;
        }
        if !in_data || line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| deny("malformed [Data] priority"))?;
        let idx = key.trim().parse::<usize>().map_err(|_| deny("invalid [Data] slot"))?;
        let name = value.trim();
        if idx >= 10 || !basename(name) || !slots.insert(idx) || !names.insert(name.to_ascii_lowercase()) {
            return Err(deny("duplicate, unsafe or out-of-range GRF entry"));
        }
        found.push((idx, name.to_string()));
    }
    if !found_data || found.is_empty() {
        return Err(deny("missing or empty [Data] section"));
    }
    Ok(found)
}

fn regular_file(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path).map_err(|_| deny("missing required GRF or DATA.INI"))?;
    if meta.file_type().is_symlink() || !meta.file_type().is_file() {
        return Err(deny("refusing linked or non-file GRF/DATA.INI"));
    }
    Ok(())
}

/// No behavioral change for other GRF names.
/// A protected 20220406 tdata1 deploy must have an explicitly registered and
/// pre-created overlay; we never let BEAM silently make an ignored GRF here.
pub(crate) fn require_tdata1_layer_ready(config: &Config) -> Result<()> {
    if !config.patcher.target_grf.eq_ignore_ascii_case(OVERLAY) {
        return Ok(());
    }
    let Some(ref compat) = config.patcher.compatibility_guard else {
        return Err(deny("tdata1 needs profile-bound patch compatibility guard"));
    };
    if compat.region != "levelupgt" || compat.channel != "custom" {
        return Err(deny("wrong region or channel for tdata1 overlay"));
    }
    let dir = match &config.app.game_directory {
        Some(dir) => PathBuf::from(dir),
        None => crate::get_executable_dir()?,
    };
    let ini_path = dir.join("DATA.INI");
    regular_file(&ini_path)?;
    let content = fs::read(&ini_path)?;
    let entries = parsed_ini_order(&content)?;
    if entries.iter().find(|(i, _)| *i == 0).map(|(_, n)| n.as_str()) != Some(OVERLAY) {
        return Err(deny("DATA.INI must register 0=tdata1.grf as top priority"));
    }
    for (_idx, name) in &entries {
        regular_file(&dir.join(name))?;
    }
    let overlay_path = dir.join(OVERLAY);
    let mut f = File::open(&overlay_path)?;
    let mut header = [0u8; 15];
    f.read_exact(&mut header).map_err(|_| deny("overlay GRF is truncated"))?;
    if &header != GRF_HEADER {
        return Err(deny("overlay GRF has invalid header"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::CompatibilityGuardConfig;

    fn cfg(target: &str, dir: &Path) -> Config {
        let mut config = Config::default();
        config.app.game_directory = Some(dir.to_string_lossy().into_owned());
        config.patcher.target_grf = target.to_string();
        config.patcher.compatibility_guard = Some(CompatibilityGuardConfig {
            profile_id: "levelupgt-custom-20220406".into(),
            region: "levelupgt".into(),
            channel: "custom".into(),
            client_exe_sha256: "a".repeat(64),
            manifest_file: "regional-patches.json".into(),
        });
        config
    }

    fn temp_root() -> PathBuf {
        let millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("beam-tdata1-test-{}-{millis}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn preserves_legacy_unrelated_grf_behavior() {
        let c = cfg("ragnardata.grf", Path::new("/does-not-exist"));
        assert!(require_tdata1_layer_ready(&c).is_ok());
    }

    #[test]
    fn parses_legacy_ini_and_opaque_other_sections_without_rewriting() {
        let data = b"[Data]\r\n0=tdata1.grf\r\n1=redata.grf\r\n9=rdata.grf\r\n[RagnaGuard]\r\nOpaque=UNKNOWN\r\n";
        let entries = parsed_ini_order(data).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].1, "tdata1.grf");
    }

    #[test]
    fn refuses_wrong_priority_zero_or_duplicate_slots() {
        assert!(parsed_ini_order(b"[Data]\n0=rdata.grf\n0=tdata1.grf\n").is_err());
        assert!(parsed_ini_order(b"[Data]\n0=tdata1.grf\n10=data.grf\n").is_err());
        assert!(parsed_ini_order(b"[Data]\n0=tdata1.grf\n1=../data.grf\n").is_err());
    }

    #[test]
    fn rejects_unregistered_overlay_before_io() {
        let dir = temp_root();
        let ini = dir.join("DATA.INI");
        fs::write(&ini, b"[Data]\r\n1=data.grf\r\n").unwrap();
        fs::write(dir.join("data.grf"), b"stub").unwrap();
        let r = require_tdata1_layer_ready(&cfg(OVERLAY, &dir));
        let _ = fs::remove_dir_all(dir);
        assert!(r.is_err());
    }

    #[test]
    fn rejects_missing_or_invalid_overlay_grf() {
        let dir = temp_root();
        fs::write(dir.join("DATA.INI"), b"[Data]\r\n0=tdata1.grf\r\n1=data.grf\r\n").unwrap();
        fs::write(dir.join("data.grf"), b"stub").unwrap();
        assert!(require_tdata1_layer_ready(&cfg(OVERLAY, &dir)).is_err());
        fs::write(dir.join(OVERLAY), b"not a grf!").unwrap();
        assert!(require_tdata1_layer_ready(&cfg(OVERLAY, &dir)).is_err());
        fs::write(dir.join(OVERLAY), GRF_HEADER).unwrap();
        assert!(require_tdata1_layer_ready(&cfg(OVERLAY, &dir)).is_ok());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_unprotected_custom_overlay() {
        let dir = temp_root();
        let mut c = cfg(OVERLAY, &dir);
        c.patcher.compatibility_guard = None;
        assert!(require_tdata1_layer_ready(&c).is_err());
        let _ = fs::remove_dir_all(dir);
    }
}
