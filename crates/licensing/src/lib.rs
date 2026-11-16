//! Offline Ed25519 license validation — stub until M7.

/// Returns whether a license key string looks non-empty (placeholder).
pub fn looks_like_key(key: &str) -> bool {
    !key.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_key_rejected() {
        assert!(!looks_like_key(""));
        assert!(!looks_like_key("   "));
    }
}
