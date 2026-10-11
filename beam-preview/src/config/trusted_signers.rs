// 📄 Dosya Yolu: /LevelUpGT/beam-patcher/beam-preview/src/config/trusted_signers.rs
// 📌 Amac: BEAM imzali manifest dogrulamasi icin yerlesik guvenilir acik anahtarlari belirlemek
// 📌 Modul - Config Rust
// Version: 1.0.0
// Aciklama: Gercek surum anahtari kayitli olmadiginda imza kontrolunu kapali tutar
// Bagimli Oldugu Katman: Service

pub const ENVELOPE_SCHEMA: &str = "levelupgt.beam.signed-preview.v1";
pub const SIGNED_DOMAIN: &[u8] = b"LEVELUPGT-BEAM-SIGNED-PREVIEW-V1\0";
pub const MAX_ENVELOPE_BYTES: usize = 160 * 1024;

#[derive(Clone, Copy)]
pub struct TrustedSigner {
    pub key_id: &'static str,
    pub public_key: [u8; 32],
}

// Fail closed: no production signer is provisioned. Test keys exist only in #[cfg(test)].
// Register audited release public keys here only after a separate key-management review.
pub const PRODUCTION_SIGNERS: &[TrustedSigner] = &[];
