//! Local SQLite history layer — schema and migrations land in M1.

/// Placeholder history store identity.
pub fn store_name() -> &'static str {
    "fileprism-history"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_name_is_stable() {
        assert_eq!(store_name(), "fileprism-history");
    }
}
