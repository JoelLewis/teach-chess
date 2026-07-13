use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Compute a deterministic cache key from the position, player level, and the
/// full built user prompt.
///
/// Format: first 16 hex chars of a hash of `"{fen}|{level}|{user_prompt}"`.
/// The user prompt deterministically contains the classification, played
/// move, best move, engine lines, and all verbalized facts, so everything
/// that shapes the response flows into the key by construction. This is
/// purely a local cache key — cryptographic strength is not needed.
pub fn compute_cache_key(fen: &str, level: &str, user_prompt: &str) -> String {
    let input = format!("{fen}|{level}|{user_prompt}");

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_is_deterministic() {
        let k1 = compute_cache_key("fen1", "beginner", "prompt text");
        let k2 = compute_cache_key("fen1", "beginner", "prompt text");
        assert_eq!(k1, k2);
    }

    #[test]
    fn cache_key_is_16_hex_chars() {
        let key = compute_cache_key("fen", "beginner", "prompt");
        assert_eq!(key.len(), 16);
        assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn different_fen_different_key() {
        let k1 = compute_cache_key("fen1", "beginner", "prompt");
        let k2 = compute_cache_key("fen2", "beginner", "prompt");
        assert_ne!(k1, k2);
    }

    #[test]
    fn different_level_different_key() {
        let k1 = compute_cache_key("fen", "beginner", "prompt");
        let k2 = compute_cache_key("fen", "intermediate", "prompt");
        assert_ne!(k1, k2);
    }

    #[test]
    fn different_prompt_different_key() {
        // Two different mistakes from the same position no longer collide:
        // the played move is part of the prompt.
        let k1 = compute_cache_key("fen", "beginner", "you played Qxb7 - a blunder");
        let k2 = compute_cache_key("fen", "beginner", "you played Rxb7 - a blunder");
        assert_ne!(k1, k2);
    }

    #[test]
    fn identical_position_and_prompt_reuses_cached_response_key() {
        let first = compute_cache_key(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
            "beginner",
            "You played e4 and the move was good.",
        );
        let repeat = compute_cache_key(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
            "beginner",
            "You played e4 and the move was good.",
        );

        assert_eq!(first, repeat);
    }
}
