use serde::{Deserialize, Serialize};

/// Personality profiles that make the AI opponent feel less engine-like.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PersonalityProfile {
    /// Prefers sharp, tactical play with king attacks
    Aggressive,
    /// Favors clean pawn structure and piece harmony
    Positional,
    /// Sets subtle traps and creates complications
    Trappy,
    /// Prioritizes safety and avoids risk
    Solid,
}

/// How the personality is selected at game start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OpponentMode {
    /// Player picks from the 4 profiles
    Choose,
    /// Randomly assigned at game start
    Surprise,
    /// Picks the profile that best challenges the player's weaknesses
    CoachPicks,
}

impl Default for OpponentMode {
    fn default() -> Self {
        Self::Choose
    }
}

/// Weight table for scoring candidate moves against a personality.
///
/// Each weight is 0.0–1.0 indicating how much the personality values that aspect.
#[derive(Debug, Clone)]
pub struct PersonalityWeights {
    /// Weight for tactical opportunities (forks, pins, discovered attacks)
    pub tactics: f64,
    /// Weight for pawn structure quality
    pub structure: f64,
    /// Weight for attacks near the opponent's king
    pub king_attack: f64,
    /// Weight for piece mobility and activity
    pub activity: f64,
    /// Weight for king safety (own king)
    pub safety: f64,
    /// Weight for positions with hidden traps
    pub trap: f64,
}

impl PersonalityProfile {
    pub fn weights(&self) -> PersonalityWeights {
        match self {
            PersonalityProfile::Aggressive => PersonalityWeights {
                tactics: 0.9,
                structure: 0.2,
                king_attack: 0.8,
                activity: 0.6,
                safety: 0.1,
                trap: 0.3,
            },
            PersonalityProfile::Positional => PersonalityWeights {
                tactics: 0.3,
                structure: 0.9,
                king_attack: 0.2,
                activity: 0.7,
                safety: 0.5,
                trap: 0.1,
            },
            PersonalityProfile::Trappy => PersonalityWeights {
                tactics: 0.7,
                structure: 0.3,
                king_attack: 0.5,
                activity: 0.5,
                safety: 0.2,
                trap: 0.9,
            },
            PersonalityProfile::Solid => PersonalityWeights {
                tactics: 0.2,
                structure: 0.7,
                king_attack: 0.1,
                activity: 0.6,
                safety: 0.9,
                trap: 0.1,
            },
        }
    }

    /// Pick a random personality (for Surprise mode).
    pub fn random() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::SystemTime;

        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
            .hash(&mut hasher);
        let h = hasher.finish();

        match h % 4 {
            0 => PersonalityProfile::Aggressive,
            1 => PersonalityProfile::Positional,
            2 => PersonalityProfile::Trappy,
            _ => PersonalityProfile::Solid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_profiles_produce_weights() {
        let profiles = [
            PersonalityProfile::Aggressive,
            PersonalityProfile::Positional,
            PersonalityProfile::Trappy,
            PersonalityProfile::Solid,
        ];
        for p in &profiles {
            let w = p.weights();
            // All weights should be in [0, 1]
            assert!(w.tactics >= 0.0 && w.tactics <= 1.0);
            assert!(w.structure >= 0.0 && w.structure <= 1.0);
            assert!(w.king_attack >= 0.0 && w.king_attack <= 1.0);
            assert!(w.activity >= 0.0 && w.activity <= 1.0);
            assert!(w.safety >= 0.0 && w.safety <= 1.0);
            assert!(w.trap >= 0.0 && w.trap <= 1.0);
        }
    }

    #[test]
    fn aggressive_prefers_tactics_and_king_attack() {
        let w = PersonalityProfile::Aggressive.weights();
        assert!(w.tactics > w.structure);
        assert!(w.king_attack > w.safety);
    }

    #[test]
    fn positional_prefers_structure() {
        let w = PersonalityProfile::Positional.weights();
        assert!(w.structure > w.tactics);
        assert!(w.structure > w.trap);
    }

    #[test]
    fn trappy_prefers_traps() {
        let w = PersonalityProfile::Trappy.weights();
        assert!(w.trap > w.structure);
        assert!(w.trap > w.safety);
    }

    #[test]
    fn solid_prefers_safety() {
        let w = PersonalityProfile::Solid.weights();
        assert!(w.safety > w.tactics);
        assert!(w.safety > w.king_attack);
    }

    #[test]
    fn random_produces_valid_profile() {
        let p = PersonalityProfile::random();
        // Just check it doesn't panic and returns a valid variant
        let _ = p.weights();
    }

    #[test]
    fn serde_roundtrip() {
        let json = serde_json::to_string(&PersonalityProfile::Trappy).unwrap();
        assert_eq!(json, "\"trappy\"");
        let back: PersonalityProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PersonalityProfile::Trappy);
    }

    #[test]
    fn opponent_mode_default_is_choose() {
        assert_eq!(OpponentMode::default(), OpponentMode::Choose);
    }
}
