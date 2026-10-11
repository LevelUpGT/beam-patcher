// 📄 Dosya Yolu: /LevelUpGT/beam-patcher/beam-preview/src/config/experiment_lab.rs
// 📌 Amac: Deneysel BEAM laboratuvari icin sabit rapor kimliklerini ve guvenlik politikasini tanimlar
// 📌 Modul - Config Rust
// Version: 0.5.0
// Aciklama: Oyuncu dagitimi olmayan deney plani, yalnizca NOT_RUN durumlariyla olusturulur
// Bagimli Oldugu Katman: Service

pub const PLAN_SCHEMA: &str = "levelupgt.beam.experiment-plan.v1";
pub const AUDIENCE: &str = "DEVELOPER_ONLY";
pub const PLAN_STATUS: &str = "PLAN_ONLY";
pub const NOT_RUN: &str = "NOT_RUN";
pub const UNVERIFIED_SOURCE: &str = "USER_SELECTION_UNVERIFIED";
pub const TEST_BRANCH_METADATA: &str = "branch_metadata";
pub const TEST_MANIFEST_METADATA: &str = "manifest_metadata";
pub const TEST_SIGNATURE: &str = "ed25519_signature";
pub const TEST_PACKAGE_SHA256: &str = "package_sha256";
pub const TEST_THOR_SANDBOX: &str = "thor_sandbox";
pub const TEST_IN_GAME: &str = "in_game_client";
pub const TEST_ROLLBACK: &str = "rollback_recovery";
pub const MAX_BRANCH_LENGTH: usize = 255;

pub const NOTE_BRANCH: &str = "Selected name and SHA are not independently verified by this plan.";
pub const NOTE_MANIFEST: &str = "Requires a separately reviewed local manifest.";
pub const NOTE_SIGNATURE: &str = "Requires trusted signing keys; none are configured for release.";
pub const NOTE_PACKAGE: &str = "Requires real package bytes and verified SHA-256.";
pub const NOTE_THOR: &str = "Only isolated loose-client THOR smoke tests are supported.";
pub const NOTE_IN_GAME: &str = "Real Ragnarok client, login and visual checks not performed.";
pub const NOTE_ROLLBACK: &str = "Real installation rollback and recovery not performed.";
