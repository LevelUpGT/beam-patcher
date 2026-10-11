// 📄 Dosya Yolu: /LevelUpGT/beam-patcher/beam-preview/src/config/client_profiles.rs
// 📌 Amac: BEAM istemci profilini, paket motorunu ve manifest onizleme sinirlarini tanimlamak
// 📌 Modul - Config Rust
// Version: 1.0.0
// Aciklama: Eski 2015, yeni 2015, 2022 Hybrid icin salt okunur eslesme kurallari
// Bagimli Oldugu Katman: Service

pub const PROFILE_OLD_2015: &str = "old-2015";
pub const PROFILE_NEW_2015: &str = "new-2015";
pub const PROFILE_HYBRID_2022: &str = "hybrid-2022";

pub const ENGINE_THOR: &str = "legacy-thor";
pub const ENGINE_RPATCHUR: &str = "rpatchur";
pub const PREVIEW_SCHEMA: &str = "levelupgt.beam.manifest-preview.v1";
pub const STATUS_UNVERIFIED: &str = "PREVIEW_ONLY_UNSIGNED";
pub const CHANNEL_STABLE: &str = "stable";
pub const CHANNEL_BETA: &str = "beta";

pub const MAX_MANIFEST_BYTES: usize = 64 * 1024;
pub const MAX_ARCHIVE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

pub fn updater_engine(profile: &str) -> Option<&'static str> {
    match profile {
        PROFILE_OLD_2015 => Some(ENGINE_THOR),
        PROFILE_NEW_2015 | PROFILE_HYBRID_2022 => Some(ENGINE_RPATCHUR),
        _ => None,
    }
}
