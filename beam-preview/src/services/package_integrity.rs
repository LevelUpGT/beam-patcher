// 📄 Dosya Yolu: /LevelUpGT/beam-patcher/beam-preview/src/services/package_integrity.rs
// 📌 Amac: Yerel paket SHA-256 ve boyutunu profil/commit eslesmeli imzasiz beyana karsilastirmak
// 📌 Modul - Rust Service
// Version: 1.0.0
// Aciklama: Gercek dosyayi hashler; imza veya oyun kurulum yetkisi ASLA uretmez
// Bagimli Oldugu Katman: Config | Tool | View

use crate::config::client_profiles;
use crate::services::manifest_preflight::{self, ManifestPreflightReport};
use crate::tools::package_reader;

pub fn verify_local_unsigned(
    manifest: &[u8],
    package_path: &str,
    profile: &str,
    project: &str,
    branch: &str,
    sha: &str,
) -> Result<ManifestPreflightReport, String> {
    let mut report = manifest_preflight::check(manifest, profile, project, branch, sha)?;
    let claim = manifest_preflight::package_claim(manifest)?;
    let actual = package_reader::sha256_local(package_path, claim.size)?;
    if actual.size != claim.size || !actual.sha256.eq_ignore_ascii_case(&claim.sha256) {
        return Err("Yerel dosya SHA-256 veya boyutu manifest beyanina uymuyor.".into());
    }
    report.status = client_profiles::STATUS_UNSIGNED_PACKAGE_MATCH;
    report.package_bytes_verified = true;
    report.signature_verified = false;
    report.installation_allowed = false;
    report.message = "Dosya SHA-256 IMZASIZ beyana uydu; manifest GUVENILIR DEGIL ve kurulum KAPALI.";
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::io::Write;
    use tempfile::NamedTempFile;

    const COMMIT: &str = "8cfc3211f8ebba2542fe6b6a114a33af0871367e";

    fn manifest(bytes: &[u8]) -> Vec<u8> {
        serde_json::json!({
            "schema": client_profiles::PREVIEW_SCHEMA,
            "profile": "hybrid-2022",
            "project": "translation",
            "repository": "b1glord/ROenglishRE",
            "branch": "translation/tr",
            "commit_sha": COMMIT,
            "engine": "rpatchur",
            "channel": "beta",
            "package_size": bytes.len(),
            "package_sha256": format!("{:x}", Sha256::digest(bytes))
        }).to_string().into_bytes()
    }
    fn verify(bytes: &[u8], path: &str) -> Result<ManifestPreflightReport, String> {
        verify_local_unsigned(bytes, path, "hybrid-2022", "translation", "translation/tr", COMMIT)
    }
    #[test]
    fn actual_bytes_match_but_install_stays_disabled() {
        let contents = b"beam-sandbox-package-only";
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(contents).unwrap();
        let report = verify(&manifest(contents), file.path().to_str().unwrap()).unwrap();
        assert!(report.package_bytes_verified);
        assert!(report.metadata_matches);
        assert!(!report.signature_verified);
        assert!(!report.installation_allowed);
        assert_eq!(report.status, client_profiles::STATUS_UNSIGNED_PACKAGE_MATCH);
    }
    #[test]
    fn same_length_modified_bytes_are_rejected() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"original").unwrap();
        assert!(verify(&manifest(b"tampered"), file.path().to_str().unwrap()).is_err());
    }
    #[test]
    fn mismatched_size_rejected_without_installing() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"original").unwrap();
        assert!(verify(&manifest(b"longer-data"), file.path().to_str().unwrap()).is_err());
    }
    #[test]
    fn wrong_profile_or_declared_digest_rejected() {
        let bytes = b"sample";
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(bytes).unwrap();
        let mut input: serde_json::Value = serde_json::from_slice(&manifest(bytes)).unwrap();
        input["profile"] = serde_json::json!("old-2015");
        assert!(verify(input.to_string().as_bytes(), file.path().to_str().unwrap()).is_err());
        input["profile"] = serde_json::json!("hybrid-2022");
        input["package_sha256"] = serde_json::json!("b".repeat(64));
        assert!(verify(input.to_string().as_bytes(), file.path().to_str().unwrap()).is_err());
    }
    #[test]
    fn missing_and_empty_files_rejected() {
        let empty = NamedTempFile::new().unwrap();
        assert!(verify(&manifest(b"data"), empty.path().to_str().unwrap()).is_err());
        assert!(verify(&manifest(b"data"), "BEAM-no-such-file-test-only.bin").is_err());
    }
    #[cfg(unix)]
    #[test]
    fn symlink_package_rejected() {
        use std::os::unix::fs::symlink;
        let mut real = NamedTempFile::new().unwrap();
        real.write_all(b"local").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let linked = dir.path().join("link.bin");
        symlink(real.path(), &linked).unwrap();
        assert!(verify(&manifest(b"local"), linked.to_str().unwrap()).is_err());
    }
}
