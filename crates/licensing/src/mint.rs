//! Optional license minting for CI / operator tooling (`mint` feature).
//!
//! Requires `FILEPRISM_LICENSE_SIGNING_SEED` = 64 hex chars (32-byte Ed25519 seed).
//! Never commit the seed.

use ed25519_dalek::{Signer, SigningKey};

use crate::key::{encode_license_key, LicensePayload, PAYLOAD_VERSION};
use crate::LicenseError;

fn seed_from_hex(hex: &str) -> Result<[u8; 32], LicenseError> {
    let hex = hex.trim();
    if hex.len() != 64 {
        return Err(LicenseError::BadVerifyingKey);
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|_| LicenseError::BadVerifyingKey)?;
    }
    Ok(out)
}

/// Load signing key from `FILEPRISM_LICENSE_SIGNING_SEED`.
pub fn signing_key_from_env() -> Result<SigningKey, LicenseError> {
    let hex = std::env::var("FILEPRISM_LICENSE_SIGNING_SEED")
        .map_err(|_| LicenseError::BadVerifyingKey)?;
    let seed = seed_from_hex(&hex)?;
    Ok(SigningKey::from_bytes(&seed))
}

/// Mint a `FP1.…` key for the given claims.
pub fn mint_license(
    signing_key: &SigningKey,
    lid: impl Into<String>,
    email: Option<String>,
    issued: i64,
    updates_until: i64,
) -> Result<String, LicenseError> {
    let payload = LicensePayload {
        v: PAYLOAD_VERSION,
        lid: lid.into(),
        email,
        issued,
        updates_until,
    };
    let body = serde_json::to_vec(&payload).map_err(|e| LicenseError::Payload(e.to_string()))?;
    let sig = signing_key.sign(&body);
    Ok(encode_license_key(&body, sig.to_bytes().as_ref()))
}
