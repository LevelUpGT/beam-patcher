// 📄 Dosya Yolu: /LevelUpGT/beam-patcher/beam-preview/src/services/signed_manifest.rs
// 📌 Amac: Ed25519 imzali yerel manifest taslagini guvenilir anahtarlarla dogrulamak
// 📌 Modul - Service Rust
// Version: 1.0.0
// Aciklama: Imzayi payload'in tam byte dizisi ve baglamsal prefix uzerinden kontrol eder
// Bagimli Oldugu Katman: Config | Tool | View | Language

use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::Deserialize;

use crate::config::{client_profiles, trusted_signers};
use crate::services::manifest_preflight::{self, ManifestPreflightReport};
use trusted_signers::TrustedSigner;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: String,
    key_id: String,
    payload_b64: String,
    signature_b64: String,
}

pub fn check(
    envelope_bytes: &[u8],
    profile: &str,
    project: &str,
    branch: &str,
    sha: &str,
    trusted: &[TrustedSigner],
) -> Result<ManifestPreflightReport, String> {
    if envelope_bytes.is_empty()
        || envelope_bytes.len() > trusted_signers::MAX_ENVELOPE_BYTES
    {
        return Err("Imzali manifest boyutu gecersiz.".to_string());
    }
    let envelope: Envelope = serde_json::from_slice(envelope_bytes)
        .map_err(|_| "Imzali manifest zarfi gecersiz.".to_string())?;
    if envelope.schema != trusted_signers::ENVELOPE_SCHEMA {
        return Err("Imzali manifest semasi desteklenmiyor.".to_string());
    }
    if envelope.key_id.is_empty()
        || envelope.key_id.len() > 64
        || !envelope.key_id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("Imza anahtari kimligi gecersiz.".to_string());
    }
    let mut candidates = trusted.iter().filter(|entry| entry.key_id == envelope.key_id);
    let signer = candidates.next()
        .ok_or("BEAM icin guvenilir imza anahtari kayitli degil.")?;
    if candidates.next().is_some() {
        return Err("Imza anahtari kimligi cakismasi.".to_string());
    }
    let payload = STANDARD.decode(&envelope.payload_b64)
        .map_err(|_| "Imzali manifest payload kodlamasi gecersiz.".to_string())?;
    if payload.is_empty() || payload.len() > client_profiles::MAX_MANIFEST_BYTES {
        return Err("Imzali manifest payload boyutu gecersiz.".to_string());
    }
    let signature_bytes = STANDARD.decode(&envelope.signature_b64)
        .map_err(|_| "Imza kodlamasi gecersiz.".to_string())?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| "Ed25519 imzasi gecersiz.".to_string())?;
    let verifying_key = VerifyingKey::from_bytes(&signer.public_key)
        .map_err(|_| "Guvenilir acik anahtar gecersiz.".to_string())?;
    let mut message = Vec::with_capacity(trusted_signers::SIGNED_DOMAIN.len() + payload.len());
    message.extend_from_slice(trusted_signers::SIGNED_DOMAIN);
    message.extend_from_slice(&payload);
    verifying_key.verify_strict(&message, &signature)
        .map_err(|_| "Manifest dijital imzasi dogrulanamadi.".to_string())?;

    let mut report = manifest_preflight::check(&payload, profile, project, branch, sha)?;
    report.status = "PREVIEW_SIGNATURE_VALID_NO_PACKAGE";
    report.signature_verified = true;
    report.package_bytes_verified = false;
    report.installation_allowed = false;
    report.message = "Dijital imza metadata icin gecerli. Paket baytlari dogrulanmadi; kurulum KAPALI.";
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    const COMMIT: &str = "8cfc3211f8ebba2542fe6b6a114a33af0871367e";
    const HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const KEY_ID: &str = "rfc8032-fixture";
    const TEST_SECRET: [u8; 32] = [42; 32];

    fn payload() -> Vec<u8> {
        format!(r#"{{"schema":"{}","profile":"hybrid-2022","project":"translation","repository":"b1glord/ROenglishRE","branch":"translation/tr","commit_sha":"{}","engine":"rpatchur","channel":"beta","package_sha256":"{}","package_size":12500}}"#,
            client_profiles::PREVIEW_SCHEMA, COMMIT, HASH).into_bytes()
    }
    fn trusted() -> Vec<TrustedSigner> {
        let key = SigningKey::from_bytes(&TEST_SECRET);
        vec![TrustedSigner {
            key_id: KEY_ID,
            public_key: key.verifying_key().to_bytes(),
        }]
    }
    fn envelope(input: &[u8]) -> Vec<u8> {
        let signer = SigningKey::from_bytes(&TEST_SECRET);
        let mut msg = trusted_signers::SIGNED_DOMAIN.to_vec();
        msg.extend_from_slice(input);
        let signature = signer.sign(&msg);
        serde_json::json!({
            "schema": trusted_signers::ENVELOPE_SCHEMA,
            "key_id": KEY_ID,
            "payload_b64": STANDARD.encode(input),
            "signature_b64": STANDARD.encode(signature.to_bytes()),
        }).to_string().into_bytes()
    }
    fn verify(blob: &[u8], keys: &[TrustedSigner]) -> Result<ManifestPreflightReport, String> {
        check(blob, "hybrid-2022", "translation", "translation/tr", COMMIT, keys)
    }

    #[test]
    fn signed_metadata_is_verified_but_still_not_installable() {
        let report = verify(&envelope(&payload()), &trusted()).unwrap();
        assert!(report.metadata_matches);
        assert!(report.signature_verified);
        assert!(!report.package_bytes_verified);
        assert!(!report.installation_allowed);
    }
    #[test]
    fn no_release_key_means_no_signature_accepted() {
        assert!(verify(&envelope(&payload()), trusted_signers::PRODUCTION_SIGNERS).is_err());
        assert!(trusted_signers::PRODUCTION_SIGNERS.is_empty());
    }
    #[test]
    fn tampering_and_unknown_signer_are_rejected() {
        let valid_payload = payload();
        let valid = envelope(&valid_payload);
        let altered = String::from_utf8(valid.clone()).unwrap()
            .replace(KEY_ID, "not-a-trusted-key");
        assert!(verify(altered.as_bytes(), &trusted()).is_err());

        let mut envelope_json: serde_json::Value = serde_json::from_slice(&valid).unwrap();
        envelope_json["payload_b64"] = serde_json::json!(STANDARD.encode(
            String::from_utf8(valid_payload).unwrap().replace("hybrid-2022", "old-2015")));
        assert!(verify(envelope_json.to_string().as_bytes(), &trusted()).is_err());

        let mut another: serde_json::Value = serde_json::from_slice(&valid).unwrap();
        another["signature_b64"] = serde_json::json!(STANDARD.encode([0_u8; 64]));
        assert!(verify(another.to_string().as_bytes(), &trusted()).is_err());
    }
    #[test]
    fn correctly_signed_but_wrong_profile_is_rejected() {
        let body = String::from_utf8(payload()).unwrap().replace("hybrid-2022", "old-2015");
        assert!(verify(&envelope(body.as_bytes()), &trusted()).is_err());
    }
    #[test]
    fn unknown_fields_bad_base64_duplicate_keys_and_size_are_rejected() {
        let valid = String::from_utf8(envelope(&payload())).unwrap();
        let extra = valid.replacen("{", "{\"install\":true,", 1);
        assert!(verify(extra.as_bytes(), &trusted()).is_err());
        let duplicate = valid.replacen("{", "{\"key_id\":\"rfc8032-fixture\",", 1);
        assert!(verify(duplicate.as_bytes(), &trusted()).is_err());
        let broken: serde_json::Value = serde_json::from_str(&valid).unwrap();
        let mut broken = broken;
        broken["payload_b64"] = serde_json::json!("$$NOT-BASE64");
        assert!(verify(broken.to_string().as_bytes(), &trusted()).is_err());
        assert!(verify(&vec![b'a'; trusted_signers::MAX_ENVELOPE_BYTES + 1], &trusted()).is_err());
    }
    #[test]
    fn different_domain_signature_is_rejected() {
        let sk = SigningKey::from_bytes(&TEST_SECRET);
        let mut env: serde_json::Value = serde_json::from_slice(&envelope(&payload())).unwrap();
        env["signature_b64"] = serde_json::json!(
            STANDARD.encode(sk.sign(&payload()).to_bytes())
        );
        assert!(verify(env.to_string().as_bytes(), &trusted()).is_err());
    }
}
