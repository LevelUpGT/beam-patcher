// 📄 Dosya Yolu: /LevelUpGT/beam-patcher/beam-preview/src/services/manifest_preflight.rs
// 📌 Amac: Yerel manifest taslagini secilen istemci ve sabit commit ile karsilastirmak
// 📌 Modul - Service Rust
// Version: 1.0.0
// Aciklama: Yalnizca metadata dogrular; imza, paket icerigi ve kurulum onayi vermez
// Bagimli Oldugu Katman: Config | Tool | View | Language

use crate::config::{branch_visibility, client_profiles};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestPreview {
    schema: String,
    profile: String,
    project: String,
    repository: String,
    branch: String,
    commit_sha: String,
    engine: String,
    channel: String,
    package_sha256: String,
    package_size: u64,
}

#[derive(Serialize)]
pub struct ManifestPreflightReport {
    pub status: &'static str,
    pub profile: String,
    pub engine: String,
    pub project: String,
    pub branch: String,
    pub commit_sha: String,
    pub channel: String,
    pub metadata_matches: bool,
    pub package_bytes_verified: bool,
    pub signature_verified: bool,
    pub installation_allowed: bool,
    pub message: &'static str,
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        && value.bytes().any(|byte| byte != b'0')
}

fn is_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn valid_label(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= 255
        && !value.chars().any(char::is_control)
}

pub fn check(
    bytes: &[u8],
    selected_profile: &str,
    selected_project: &str,
    selected_branch: &str,
    selected_sha: &str,
) -> Result<ManifestPreflightReport, String> {
    if bytes.is_empty() || bytes.len() > client_profiles::MAX_MANIFEST_BYTES {
        return Err("Manifest boyutu gecersiz.".to_string());
    }
    let engine = client_profiles::updater_engine(selected_profile)
        .ok_or("Desteklenmeyen istemci profili.")?;
    let repository = branch_visibility::public_branch_source(selected_project)
        .map_err(str::to_string)?;
    if !valid_label(selected_branch) || !is_commit(selected_sha) {
        return Err("Secili branch veya commit kimligi gecersiz.".to_string());
    }

    let manifest: ManifestPreview = serde_json::from_slice(bytes)
        .map_err(|_| "Manifest JSON semasi gecersiz.".to_string())?;
    if manifest.schema != client_profiles::PREVIEW_SCHEMA {
        return Err("Manifest semasi desteklenmiyor.".to_string());
    }
    if manifest.profile != selected_profile
        || manifest.project != selected_project
        || manifest.repository != repository
        || manifest.branch != selected_branch
        || !manifest.commit_sha.eq_ignore_ascii_case(selected_sha)
        || manifest.engine != engine
    {
        return Err("Manifest secili profil, depo, dal veya commit ile eslesmiyor.".to_string());
    }
    if !valid_label(&manifest.branch) || !is_commit(&manifest.commit_sha) {
        return Err("Manifest branch veya commit kimligi gecersiz.".to_string());
    }
    if manifest.channel != client_profiles::CHANNEL_STABLE
        && manifest.channel != client_profiles::CHANNEL_BETA
    {
        return Err("Manifest dagitim kanali desteklenmiyor.".to_string());
    }
    if !is_sha256(&manifest.package_sha256)
        || manifest.package_size == 0
        || manifest.package_size > client_profiles::MAX_ARCHIVE_BYTES
    {
        return Err("Paket ozeti veya boyutu gecersiz.".to_string());
    }

    // This preview validates declared metadata only. It NEVER authorizes update.
    Ok(ManifestPreflightReport {
        status: client_profiles::STATUS_UNVERIFIED,
        profile: manifest.profile,
        engine: manifest.engine,
        project: manifest.project,
        branch: manifest.branch,
        commit_sha: manifest.commit_sha,
        channel: manifest.channel,
        metadata_matches: true,
        package_bytes_verified: false,
        signature_verified: false,
        installation_allowed: false,
        message: "Metadata eslesti. Imza ve paket icerigi DOGRULANMADI; kurulum kapali.",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA: &str = "8cfc3211f8ebba2542fe6b6a114a33af0871367e";
    const PACKAGE_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    fn fixture() -> Vec<u8> {
        format!(r#"{{"schema":"{}","profile":"hybrid-2022","project":"translation","repository":"b1glord/ROenglishRE","branch":"translation/tr","commit_sha":"{}","engine":"rpatchur","channel":"beta","package_sha256":"{}","package_size":12500}}"#,
          client_profiles::PREVIEW_SCHEMA, SHA, PACKAGE_SHA).into_bytes()
    }
    fn valid(bytes: &[u8]) -> Result<ManifestPreflightReport, String> {
        check(bytes, client_profiles::PROFILE_HYBRID_2022,
              branch_visibility::PROJECT_TRANSLATION, "translation/tr", SHA)
    }

    #[test]
    fn matching_metadata_is_never_a_trusted_install_manifest() {
        let report = valid(&fixture()).unwrap();
        assert!(report.metadata_matches);
        assert!(!report.package_bytes_verified);
        assert!(!report.signature_verified);
        assert!(!report.installation_allowed);
        assert_eq!(report.status, client_profiles::STATUS_UNVERIFIED);
    }

    #[test]
    fn all_three_profiles_have_distinct_declared_engine_policy() {
        assert_eq!(client_profiles::updater_engine(client_profiles::PROFILE_OLD_2015),
                   Some(client_profiles::ENGINE_THOR));
        assert_eq!(client_profiles::updater_engine(client_profiles::PROFILE_NEW_2015),
                   Some(client_profiles::ENGINE_RPATCHUR));
        assert_eq!(client_profiles::updater_engine(client_profiles::PROFILE_HYBRID_2022),
                   Some(client_profiles::ENGINE_RPATCHUR));
        assert!(client_profiles::updater_engine("unknown").is_none());
    }

    #[test]
    fn rejects_wrong_profile_repository_branch_and_sha() {
        let source = String::from_utf8(fixture()).unwrap();
        for changed in [
            source.replace("hybrid-2022", "old-2015"),
            source.replace("b1glord/ROenglishRE", "evil/repo"),
            source.replace("translation/tr", "master"),
            source.replace(SHA, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        ] {
            assert!(valid(changed.as_bytes()).is_err());
        }
    }

    #[test]
    fn rejects_invalid_hash_channel_schema_size_and_unknown_fields() {
        let source = String::from_utf8(fixture()).unwrap();
        for changed in [
            source.replace(PACKAGE_SHA, &"0".repeat(64)),
            source.replace(PACKAGE_SHA, "not-a-sha"),
            source.replace("\"beta\"", "\"nightly\""),
            source.replace(client_profiles::PREVIEW_SCHEMA, "unknown-schema"),
            source.replace("\"package_size\":12500", "\"package_size\":0"),
            source.replace("\"package_size\":12500", "\"package_size\":99999999999"),
            source.replace("\"package_size\":12500", "\"package_size\":12500,\"install\":true"),
        ] {
            assert!(valid(changed.as_bytes()).is_err());
        }
    }

    #[test]
    fn rejects_duplicate_json_fields_private_repo_and_bad_selection() {
        let source = String::from_utf8(fixture()).unwrap();
        let duplicate = source.replace("\"channel\":\"beta\"",
          "\"channel\":\"beta\",\"channel\":\"stable\"");
        assert!(valid(duplicate.as_bytes()).is_err());
        assert!(check(&fixture(), "hybrid-2022", "client", "translation/tr", SHA).is_err());
        assert!(check(&fixture(), "hybrid-2022", "translation", "translation/tr", "bad").is_err());
        assert!(valid(&vec![b' '; client_profiles::MAX_MANIFEST_BYTES + 1]).is_err());
    }
}
