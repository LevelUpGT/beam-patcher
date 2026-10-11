// 📄 Dosya Yolu: /LevelUpGT/beam-patcher/beam-preview/src/services/experiment_lab.rs
// 📌 Amac: Istemci profili ve sabit commit seciminden salt-okunur deney plani olusturur
// 📌 Modul - Service Rust
// Version: 0.5.0
// Aciklama: Test gerceklestirmeden durum PASS yazmaz; istemci ve oyun dosyalarina dokunmaz
// Bagimli Oldugu Katman: Config | View | Language

use serde::Serialize;
use crate::config::{branch_visibility, client_profiles, experiment_lab as policy};

#[derive(Serialize)]
pub struct ExperimentStep {
    pub id: &'static str,
    pub status: &'static str,
    pub note: &'static str,
}

#[derive(Serialize)]
pub struct ExperimentPlan {
    pub schema: &'static str,
    pub audience: &'static str,
    pub status: &'static str,
    pub profile: String,
    pub engine: &'static str,
    pub project: String,
    pub repository: &'static str,
    pub branch: String,
    pub commit_sha: String,
    pub source_verification: &'static str,
    pub tests: Vec<ExperimentStep>,
    pub installation_allowed: bool,
    pub game_launch_allowed: bool,
    pub player_distribution_allowed: bool,
}

fn valid_branch(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= policy::MAX_BRANCH_LENGTH
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.starts_with('.')
        && !value.ends_with('.')
        && !value.ends_with(".lock")
        && !value.contains("..")
        && !value.contains("//")
        && value.bytes().all(|b| b.is_ascii_alphanumeric()
            || b == b'/' || b == b'-' || b == b'_' || b == b'.')
}

fn valid_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

pub fn build(
    profile: &str,
    project: &str,
    branch: &str,
    sha: &str,
) -> Result<ExperimentPlan, String> {
    let engine = client_profiles::updater_engine(profile)
        .ok_or_else(|| "Desteklenmeyen deneysel istemci profili.".to_string())?;
    let repository = branch_visibility::public_branch_source(project)
        .map_err(str::to_string)?;
    if !valid_branch(branch) || !valid_sha(sha) {
        return Err("Deney icin dal veya tam commit SHA gecersiz.".to_string());
    }
    let steps = [
        (policy::TEST_BRANCH_METADATA, policy::NOTE_BRANCH),
        (policy::TEST_MANIFEST_METADATA, policy::NOTE_MANIFEST),
        (policy::TEST_SIGNATURE, policy::NOTE_SIGNATURE),
        (policy::TEST_PACKAGE_SHA256, policy::NOTE_PACKAGE),
        (policy::TEST_THOR_SANDBOX, policy::NOTE_THOR),
        (policy::TEST_IN_GAME, policy::NOTE_IN_GAME),
        (policy::TEST_ROLLBACK, policy::NOTE_ROLLBACK),
    ]
    .into_iter()
    .map(|(id, note)| ExperimentStep {
        id,
        status: policy::NOT_RUN,
        note,
    })
    .collect();

    Ok(ExperimentPlan {
        schema: policy::PLAN_SCHEMA,
        audience: policy::AUDIENCE,
        status: policy::PLAN_STATUS,
        profile: profile.to_owned(),
        engine,
        project: project.to_owned(),
        repository,
        branch: branch.to_owned(),
        commit_sha: sha.to_ascii_lowercase(),
        source_verification: policy::UNVERIFIED_SOURCE,
        tests: steps,
        installation_allowed: false,
        game_launch_allowed: false,
        player_distribution_allowed: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA: &str = "8cfc3211f8ebba2542fe6b6a114a33af0871367e";

    #[test]
    fn plan_is_developer_only_and_never_claims_tests_ran() {
        let plan = build("hybrid-2022", "translation", "translation/tr", SHA).unwrap();
        assert_eq!(plan.audience, policy::AUDIENCE);
        assert_eq!(plan.status, policy::PLAN_STATUS);
        assert_eq!(plan.source_verification, policy::UNVERIFIED_SOURCE);
        assert_eq!(plan.engine, client_profiles::ENGINE_RPATCHUR);
        assert!(plan.tests.iter().all(|step| step.status == policy::NOT_RUN));
        assert_eq!(plan.tests.len(), 7);
        assert!(!plan.installation_allowed);
        assert!(!plan.game_launch_allowed);
        assert!(!plan.player_distribution_allowed);
    }

    #[test]
    fn each_profile_maps_to_expected_engine_without_side_effects() {
        for (profile, engine) in [
            ("old-2015", client_profiles::ENGINE_THOR),
            ("new-2015", client_profiles::ENGINE_RPATCHUR),
            ("hybrid-2022", client_profiles::ENGINE_RPATCHUR),
        ] {
            let plan = build(profile, "beam", "feat/windows-gui-preview", SHA).unwrap();
            assert_eq!(plan.engine, engine);
            assert_eq!(plan.repository, branch_visibility::REPOSITORY_BEAM);
        }
    }

    #[test]
    fn invalid_repo_profile_branch_or_sha_is_rejected() {
        for (profile, project, branch, sha) in [
            ("nope", "translation", "translation/tr", SHA),
            ("old-2015", "client", "main", SHA),
            ("old-2015", "https://example.org", "main", SHA),
            ("old-2015", "beam", "../unsafe", SHA),
            ("old-2015", "beam", "foo//bar", SHA),
            ("old-2015", "beam", "foo.lock", SHA),
            ("old-2015", "beam", "main", "12345"),
            ("old-2015", "beam", "main", "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"),
        ] {
            assert!(build(profile, project, branch, sha).is_err());
        }
    }

    #[test]
    fn selected_sha_is_normalized_and_data_is_serializable() {
        let plan = build("old-2015", "beam", "main", &SHA.to_ascii_uppercase()).unwrap();
        assert_eq!(plan.commit_sha, SHA);
        let json = serde_json::to_value(&plan).unwrap();
        assert_eq!(json["tests"][0]["status"], policy::NOT_RUN);
        assert_eq!(json["source_verification"], policy::UNVERIFIED_SOURCE);
        assert_eq!(json["installation_allowed"], false);
    }
}
