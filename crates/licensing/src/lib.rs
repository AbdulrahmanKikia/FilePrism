//! Offline Ed25519 license validation and trial entitlement (M7).
//!
//! License keys are `FP1.<base64url(payload_json)>.<base64url(64-byte-sig)>`.
//! The signature covers the raw UTF-8 payload bytes (before base64).
//!
//! Trial: full-feature for [`TRIAL_DAYS`] from first launch (persisted by the host).

mod key;
mod trial;

#[cfg(feature = "mint")]
pub mod mint;

pub use key::{
    embedded_verifying_key, encode_license_key, parse_license_key, verifying_key_from_hex,
    LicenseClaims, LicensePayload, LICENSE_PREFIX, PAYLOAD_VERSION,
};
pub use trial::{
    resolve_entitlement, Entitlement, EntitlementKind, TrialConfig, TRIAL_DAYS,
};

use thiserror::Error;

/// Errors from license parsing / verification.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LicenseError {
    #[error("empty license key")]
    Empty,
    #[error("malformed license key")]
    Malformed,
    #[error("unsupported license version")]
    UnsupportedVersion,
    #[error("invalid license payload: {0}")]
    Payload(String),
    #[error("invalid signature")]
    BadSignature,
    #[error("invalid verifying key")]
    BadVerifyingKey,
}

/// Returns whether a string looks like a FilePrism license key (prefix + shape).
pub fn looks_like_key(key: &str) -> bool {
    let key = key.trim();
    if key.is_empty() {
        return false;
    }
    let mut parts = key.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(LICENSE_PREFIX), Some(p), Some(s), None) if !p.is_empty() && !s.is_empty()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    fn sample_payload() -> LicensePayload {
        LicensePayload {
            v: PAYLOAD_VERSION,
            lid: "lic_test_1".into(),
            email: Some("dev@example.com".into()),
            issued: 1_700_000_000,
            updates_until: 1_731_568_000,
        }
    }

    fn sign_key(sk: &SigningKey, payload: &LicensePayload) -> String {
        let body = serde_json::to_vec(payload).unwrap();
        let sig = sk.sign(&body);
        encode_license_key(&body, sig.to_bytes().as_ref())
    }

    #[test]
    fn empty_key_rejected() {
        assert!(!looks_like_key(""));
        assert!(!looks_like_key("   "));
        assert!(!looks_like_key("not-a-key"));
    }

    #[test]
    fn round_trip_verify() {
        let sk = SigningKey::generate(&mut OsRng);
        let key = sign_key(&sk, &sample_payload());
        assert!(looks_like_key(&key));
        let claims = parse_license_key(&key, &sk.verifying_key()).unwrap();
        assert_eq!(claims.lid, "lic_test_1");
        assert_eq!(claims.email.as_deref(), Some("dev@example.com"));
        assert!(claims.updates_active_at(1_710_000_000));
        assert!(!claims.updates_active_at(1_740_000_000));
    }

    #[test]
    fn bad_signature_rejected() {
        let sk = SigningKey::generate(&mut OsRng);
        let other = SigningKey::generate(&mut OsRng);
        let key = sign_key(&sk, &sample_payload());
        let err = parse_license_key(&key, &other.verifying_key()).unwrap_err();
        assert_eq!(err, LicenseError::BadSignature);
    }

    #[test]
    fn trial_then_licensed() {
        let cfg = TrialConfig::default();
        let started = 1_700_000_000_i64;
        let day = 86_400_i64;

        let trial = resolve_entitlement(None, Some(started), started + day, &cfg, None);
        assert!(matches!(
            trial.kind,
            EntitlementKind::Trial {
                days_remaining: 13,
                ..
            }
        ));
        assert!(trial.full_feature);

        let expired = resolve_entitlement(
            None,
            Some(started),
            started + (cfg.days as i64) * day + 1,
            &cfg,
            None,
        );
        assert_eq!(expired.kind, EntitlementKind::TrialExpired);
        assert!(!expired.full_feature);

        let sk = SigningKey::generate(&mut OsRng);
        let key = sign_key(&sk, &sample_payload());
        let claims = parse_license_key(&key, &sk.verifying_key()).unwrap();
        let licensed = resolve_entitlement(
            Some(&claims),
            Some(started),
            1_710_000_000,
            &cfg,
            None,
        );
        assert!(matches!(licensed.kind, EntitlementKind::Licensed { .. }));
        assert!(licensed.full_feature);
    }

    #[test]
    fn embedded_key_is_32_bytes() {
        let vk = embedded_verifying_key();
        assert_eq!(vk.as_bytes().len(), 32);
    }
}
