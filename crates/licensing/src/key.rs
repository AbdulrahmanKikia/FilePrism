//! License key encode / decode / verify.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::LicenseError;

/// Wire prefix for FilePrism license keys.
pub const LICENSE_PREFIX: &str = "FP1";

/// Current payload schema version.
pub const PAYLOAD_VERSION: u32 = 1;

/// Development / placeholder verifying key (32-byte hex).
///
/// Replace with the production public key before release. The matching signing
/// seed must live only in secrets (`FILEPRISM_LICENSE_SIGNING_SEED`), never in git.
/// Override at runtime with `FILEPRISM_LICENSE_PUBKEY` (64 hex chars).
pub const EMBEDDED_PUBLIC_KEY_HEX: &str =
    "fc072821d80311333e81ddddd14d4c009cfbd54cbc4c95e92bfed53a21a8e9c4";

/// Signed license payload (JSON inside the key).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicensePayload {
    pub v: u32,
    pub lid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Unix seconds (UTC) when the license was issued.
    pub issued: i64,
    /// Unix seconds (UTC) through which paid updates are entitled (typically issue + 1 year).
    pub updates_until: i64,
}

/// Verified claims extracted from a license key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseClaims {
    pub lid: String,
    pub email: Option<String>,
    pub issued: i64,
    pub updates_until: i64,
}

impl LicenseClaims {
    pub fn from_payload(p: LicensePayload) -> Self {
        Self {
            lid: p.lid,
            email: p.email,
            issued: p.issued,
            updates_until: p.updates_until,
        }
    }

    /// Whether the one-year update window is still active at `now` (unix secs).
    pub fn updates_active_at(&self, now: i64) -> bool {
        now <= self.updates_until
    }
}

/// Build `FP1.<payload_b64>.<sig_b64>` from raw payload bytes and a 64-byte signature.
pub fn encode_license_key(payload_bytes: &[u8], signature: &[u8]) -> String {
    format!(
        "{LICENSE_PREFIX}.{}.{}",
        URL_SAFE_NO_PAD.encode(payload_bytes),
        URL_SAFE_NO_PAD.encode(signature)
    )
}

/// Parse and verify a license key against `verifying_key`.
pub fn parse_license_key(
    key: &str,
    verifying_key: &VerifyingKey,
) -> Result<LicenseClaims, LicenseError> {
    let key = key.trim();
    if key.is_empty() {
        return Err(LicenseError::Empty);
    }

    let mut parts = key.split('.');
    let (prefix, payload_b64, sig_b64) = match (parts.next(), parts.next(), parts.next(), parts.next())
    {
        (Some(p), Some(a), Some(b), None) => (p, a, b),
        _ => return Err(LicenseError::Malformed),
    };

    if prefix != LICENSE_PREFIX {
        return Err(LicenseError::UnsupportedVersion);
    }

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64.as_bytes())
        .map_err(|_| LicenseError::Malformed)?;
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(sig_b64.as_bytes())
        .map_err(|_| LicenseError::Malformed)?;

    if sig_bytes.len() != 64 {
        return Err(LicenseError::Malformed);
    }

    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&sig_bytes);
    let signature = Signature::from_bytes(&sig_arr);

    verifying_key
        .verify(&payload_bytes, &signature)
        .map_err(|_| LicenseError::BadSignature)?;

    let payload: LicensePayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| LicenseError::Payload(e.to_string()))?;

    if payload.v != PAYLOAD_VERSION {
        return Err(LicenseError::UnsupportedVersion);
    }
    if payload.lid.trim().is_empty() {
        return Err(LicenseError::Payload("missing lid".into()));
    }
    if payload.updates_until < payload.issued {
        return Err(LicenseError::Payload(
            "updates_until before issued".into(),
        ));
    }

    Ok(LicenseClaims::from_payload(payload))
}

/// Parse a 32-byte verifying key from lowercase/uppercase hex.
pub fn verifying_key_from_hex(hex: &str) -> Result<VerifyingKey, LicenseError> {
    let bytes = decode_hex32(hex.trim()).ok_or(LicenseError::BadVerifyingKey)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| LicenseError::BadVerifyingKey)
}

/// Verifying key used by the desktop app: `FILEPRISM_LICENSE_PUBKEY` env, else embedded.
pub fn embedded_verifying_key() -> VerifyingKey {
    if let Ok(hex) = std::env::var("FILEPRISM_LICENSE_PUBKEY") {
        if let Ok(vk) = verifying_key_from_hex(&hex) {
            return vk;
        }
    }
    verifying_key_from_hex(EMBEDDED_PUBLIC_KEY_HEX).expect("embedded public key")
}

fn decode_hex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        let byte = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok()?;
        out[i] = byte;
    }
    Some(out)
}
