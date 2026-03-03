use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Compute a deterministic cache key from position and coaching parameters.
///
/// Format: first 16 hex chars of a hash of `"{fen}|{level}|{classification}|{sorted_themes}"`.
/// This is purely a local cache key — cryptographic strength is not needed.
pub fn compute_cache_key(
    fen: &str,
    level: &str,
    classification: &str,
    themes: &[String],
) -> String {
    let mut sorted_themes = themes.to_vec();
    sorted_themes.sort();
    let input = format!(
        "{}|{}|{}|{}",
        fen,
        level,
        classification,
        sorted_themes.join(",")
    );

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_is_deterministic() {
        let k1 = compute_cache_key("fen1", "beginner", "blunder", &["a".into()]);
        let k2 = compute_cache_key("fen1", "beginner", "blunder", &["a".into()]);
        assert_eq!(k1, k2);
    }

    #[test]
    fn cache_key_is_16_hex_chars() {
        let key = compute_cache_key("fen", "beginner", "best", &[]);
        assert_eq!(key.len(), 16);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn different_fen_different_key() {
        let k1 = compute_cache_key("fen1", "beginner", "blunder", &[]);
        let k2 = compute_cache_key("fen2", "beginner", "blunder", &[]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn different_level_different_key() {
        let k1 = compute_cache_key("fen", "beginner", "blunder", &[]);
        let k2 = compute_cache_key("fen", "intermediate", "blunder", &[]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn different_classification_different_key() {
        let k1 = compute_cache_key("fen", "beginner", "blunder", &[]);
        let k2 = compute_cache_key("fen", "beginner", "mistake", &[]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn different_themes_different_key() {
        let k1 = compute_cache_key("fen", "beginner", "blunder", &["a".into()]);
        let k2 = compute_cache_key("fen", "beginner", "blunder", &["b".into()]);
        assert_ne!(k1, k2);
    }

    #[test]
    fn theme_order_does_not_matter() {
        let k1 = compute_cache_key("fen", "beginner", "blunder", &["a".into(), "b".into()]);
        let k2 = compute_cache_key("fen", "beginner", "blunder", &["b".into(), "a".into()]);
        assert_eq!(k1, k2);
    }
}
