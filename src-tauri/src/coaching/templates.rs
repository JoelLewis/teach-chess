use crate::assessment::rank::RatingBand;
use crate::models::engine::MoveClassification;
use crate::models::heuristics::{GamePhase, PositionalTheme, TacticType};

/// Generic template for a move classification (Tier 1 — always available)
pub fn generic_template(classification: MoveClassification) -> &'static str {
    match classification {
        MoveClassification::Best => "Excellent find! This is the strongest move in the position.",
        MoveClassification::Excellent => "Very good move. You're playing accurately.",
        MoveClassification::Good => "Solid choice. This keeps the position balanced.",
        MoveClassification::Inaccuracy => {
            "This move is okay, but there was a stronger option available."
        }
        MoveClassification::Mistake => {
            "This move gives your opponent an advantage. Look for the stronger alternative."
        }
        MoveClassification::Blunder => {
            "This is a serious error that changes the evaluation significantly."
        }
    }
}

/// Theme-specialized template for error moves: Inaccuracy, Mistake, Blunder (Tier 2)
/// Returns None if no specialized template exists for this theme + classification pair.
pub fn theme_error_template(
    _classification: MoveClassification,
    theme: &PositionalTheme,
) -> Option<&'static str> {
    match theme {
        PositionalTheme::KnightOnRim => Some(
            "Your knight ended up on the rim where it controls fewer squares. \
             Knights are strongest in the center where they can reach up to 8 squares.",
        ),
        PositionalTheme::BishopPairAdvantage => Some(
            "The bishop pair is a valuable asset, especially in open positions. \
             Try to keep the position open to maximize their range.",
        ),
        PositionalTheme::IsolatedQueenPawn => Some(
            "There's an isolated queen pawn. It can be a target since it has no neighboring \
             pawns to defend it, but it also controls central squares.",
        ),
        PositionalTheme::PassedPawn => Some(
            "There's a passed pawn in the position that needs attention. \
             Passed pawns must be either pushed or blockaded.",
        ),
        PositionalTheme::DoubledPawns => Some(
            "There are doubled pawns in the position. They can be weak because \
             they can't protect each other.",
        ),
        PositionalTheme::BackwardPawn => Some(
            "There's a backward pawn that can become a target. It can't advance safely \
             and isn't supported by adjacent pawns.",
        ),
        PositionalTheme::OpenFile => Some(
            "There's an open file available. Getting a rook on an open file gives you \
             control and potential for infiltration.",
        ),
        PositionalTheme::RookOnSeventh => Some(
            "A rook on the seventh rank is very powerful — it attacks pawns \
             and restricts the enemy king.",
        ),
        PositionalTheme::KingSafetyCompromised => Some(
            "Your king's safety is weakened. Consider strengthening your pawn shield \
             or castling if you haven't already.",
        ),
        PositionalTheme::UndevelopedPieces => Some(
            "You still have pieces on the back rank. Prioritize getting all your pieces \
             into the game before launching an attack.",
        ),
        PositionalTheme::CentralControl => Some(
            "Central control is key here. The player who controls the center usually has \
             more piece mobility and attacking chances.",
        ),
        PositionalTheme::PawnChainTension => Some(
            "There's tension in the pawn chain. Consider whether to maintain, release, \
             or increase the tension based on your plan.",
        ),
        PositionalTheme::MaterialImbalance => Some(
            "There's a material imbalance. Consider whether to simplify (if ahead) \
             or complicate the position (if behind).",
        ),
        PositionalTheme::BackRankWeakness => Some(
            "The back rank is vulnerable. Make sure your king has an escape square \
             to avoid a back-rank mate — consider making a 'luft' move.",
        ),
        PositionalTheme::PinnedPiece => Some(
            "There's a pin in the position. Pinned pieces can't move without \
             exposing something more valuable behind them.",
        ),
        PositionalTheme::ForkAvailable => Some(
            "There's a forking opportunity — a single piece attacking two or more \
             targets at once.",
        ),
        PositionalTheme::HangingMaterial => Some(
            "There's undefended material on the board. Always do a 'blunder check' \
             before committing to a move.",
        ),
    }
}

/// Positive reinforcement templates for Best/Excellent moves with themes (Tier 3)
/// Returns None if no specialized positive template exists for this theme.
pub fn theme_positive_template(
    classification: MoveClassification,
    theme: &PositionalTheme,
) -> Option<&'static str> {
    match (classification, theme) {
        (MoveClassification::Best, PositionalTheme::CentralControl) => {
            Some("Excellent find! You're maintaining strong control of the center.")
        }
        (MoveClassification::Best, PositionalTheme::PassedPawn) => {
            Some("Great move! Advancing or supporting a passed pawn is often the key to winning.")
        }
        (MoveClassification::Best, PositionalTheme::OpenFile) => {
            Some("Great move! Controlling the open file gives you a lasting advantage.")
        }
        (MoveClassification::Best, PositionalTheme::RookOnSeventh) => {
            Some("Excellent! A rook on the seventh rank dominates the position.")
        }
        (MoveClassification::Best, PositionalTheme::KingSafetyCompromised) => {
            Some("Great find! You're exploiting the weakened king position.")
        }
        (MoveClassification::Best, PositionalTheme::HangingMaterial) => {
            Some("Sharp eye! You spotted the undefended material.")
        }
        (MoveClassification::Best, PositionalTheme::ForkAvailable) => {
            Some("Excellent! You found the fork — attacking multiple targets at once.")
        }
        (MoveClassification::Best, PositionalTheme::BackRankWeakness) => {
            Some("Great find! You're exploiting the back-rank weakness.")
        }
        (MoveClassification::Best, PositionalTheme::PinnedPiece) => {
            Some("Well played! You're taking advantage of the pin.")
        }
        (MoveClassification::Excellent, PositionalTheme::CentralControl) => {
            Some("Good play — maintaining central control keeps your pieces active.")
        }
        (MoveClassification::Excellent, PositionalTheme::OpenFile) => {
            Some("Good eye — controlling the open file gives you a lasting advantage.")
        }
        (MoveClassification::Excellent, PositionalTheme::PassedPawn) => {
            Some("Nice move — supporting or advancing your passed pawn is strong here.")
        }
        (MoveClassification::Excellent, PositionalTheme::BishopPairAdvantage) => {
            Some("Good move — you're making the most of the bishop pair in this position.")
        }
        (MoveClassification::Excellent, PositionalTheme::RookOnSeventh) => {
            Some("Well played — a rook on the seventh rank is a powerful asset.")
        }
        _ => None,
    }
}

/// Rank-calibrated addendum appended to coaching text, keyed on
/// (classification, band). Qualitative only — no fabricated statistics.
///
/// Returns None for classifications where level-relative framing adds
/// nothing (Good/Excellent), so unranked and neutral feedback is unchanged.
pub fn rank_addendum(classification: MoveClassification, band: RatingBand) -> Option<&'static str> {
    match (classification, band) {
        (MoveClassification::Blunder, RatingBand::Novice) => Some(
            "Mistakes like this are the most common at your level - spotting them is the fastest way to improve.",
        ),
        (MoveClassification::Blunder, RatingBand::Developing) => Some(
            "Oversights like this still decide many games at your level - a consistent blunder-check will pay off.",
        ),
        (MoveClassification::Blunder, RatingBand::Advancing) => Some(
            "Players at your level usually catch this - treat it as a reminder to slow down on critical moves.",
        ),
        (MoveClassification::Blunder, RatingBand::Expert) => {
            Some("Even stronger players miss this under pressure.")
        }
        (MoveClassification::Mistake, RatingBand::Novice) => {
            Some("Errors like this are very common at your level - you're in good company.")
        }
        (MoveClassification::Mistake, RatingBand::Developing) => {
            Some("This is one of the most frequent mistakes at your level.")
        }
        (MoveClassification::Mistake, RatingBand::Advancing) => {
            Some("At your level this is usually avoidable with a careful check of candidate moves.")
        }
        (MoveClassification::Mistake, RatingBand::Expert) => {
            Some("Even stronger players slip here occasionally.")
        }
        (MoveClassification::Inaccuracy, RatingBand::Novice) => {
            Some("Don't worry - precision here comes with practice.")
        }
        (MoveClassification::Inaccuracy, RatingBand::Developing) => {
            Some("Small improvements like this add up quickly at your level.")
        }
        (MoveClassification::Inaccuracy, RatingBand::Advancing) => {
            Some("Sharpening these small decisions is what separates your level from the next.")
        }
        (MoveClassification::Inaccuracy, RatingBand::Expert) => {
            Some("At your level, these fine margins are where games are decided.")
        }
        (MoveClassification::Best, RatingBand::Novice) => {
            Some("Finds like this are rare at your level - great progress.")
        }
        (MoveClassification::Best, RatingBand::Developing) => {
            Some("Spotting this consistently will carry you past many players at your level.")
        }
        (MoveClassification::Best, RatingBand::Advancing) => {
            Some("That's exactly the standard expected at your level.")
        }
        (MoveClassification::Best, RatingBand::Expert) => {
            Some("Strong play - even at your level this takes precision.")
        }
        (MoveClassification::Excellent | MoveClassification::Good, _) => None,
    }
}

/// Tactic-specific coaching text for error moves (Inaccuracy/Mistake/Blunder)
pub fn tactic_template(tactic: &TacticType) -> &'static str {
    match tactic {
        TacticType::Pin => {
            "There's a pin in the position. A pinned piece can't move without \
             exposing a more valuable piece behind it."
        }
        TacticType::Fork => {
            "Watch out for forks — a single piece attacking two or more targets at once. \
             Always check knight and pawn forks before committing to a move."
        }
        TacticType::Skewer => {
            "There's a skewer in the position — a piece is forced to move, exposing \
             a less valuable piece behind it to capture."
        }
        TacticType::HangingPiece => {
            "There's undefended material on the board. Always do a 'blunder check' — \
             look for any pieces left hanging before you play."
        }
        TacticType::BackRankThreat => {
            "There's a back-rank threat. Make sure your king has an escape square \
             — consider making a 'luft' move to prevent back-rank mates."
        }
        TacticType::DiscoveredAttack => {
            "Watch for discovered attacks — when one piece moves, it can reveal an attack \
             from a piece behind it. These are easy to overlook."
        }
    }
}

/// Phase transition coaching text for when the game shifts between phases
pub fn phase_transition_text(from: &GamePhase, to: &GamePhase) -> &'static str {
    match (from, to) {
        (GamePhase::Opening, GamePhase::Middlegame) => {
            "The opening is over — you're entering the middlegame. Focus on piece activity, \
             creating plans, and looking for tactical opportunities."
        }
        (GamePhase::Middlegame, GamePhase::Endgame) => {
            "The game is transitioning to an endgame. King activity becomes crucial — \
             centralize your king and push passed pawns."
        }
        (GamePhase::Opening, GamePhase::Endgame) => {
            "The game jumped straight to an endgame! Activate your king immediately \
             and focus on pawn structure and piece coordination."
        }
        _ => "",
    }
}

/// Personality-aware pre-move hint text.
///
/// Returns a context-sensitive hint based on the opponent's personality,
/// or None if no personality-specific hint applies.
pub fn personality_hint(
    personality: &crate::opponent::personality::PersonalityProfile,
) -> &'static str {
    use crate::opponent::personality::PersonalityProfile;
    match personality {
        PersonalityProfile::Aggressive => {
            "Your opponent plays aggressively — make sure your king is safe before attacking."
        }
        PersonalityProfile::Positional => {
            "Your opponent favors solid positions — look for pawn breaks and active piece play."
        }
        PersonalityProfile::Trappy => {
            "Be careful — your opponent may be setting up a trap. Double-check every capture."
        }
        PersonalityProfile::Solid => {
            "Your opponent plays safely — look for ways to create imbalances and seize the initiative."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_personalities_have_hints() {
        use crate::opponent::personality::PersonalityProfile;
        let profiles = [
            PersonalityProfile::Aggressive,
            PersonalityProfile::Positional,
            PersonalityProfile::Trappy,
            PersonalityProfile::Solid,
        ];
        for p in &profiles {
            let hint = personality_hint(p);
            assert!(!hint.is_empty(), "Empty personality hint for {p:?}");
        }
    }

    #[test]
    fn all_classifications_have_generic_template() {
        let classifications = [
            MoveClassification::Best,
            MoveClassification::Excellent,
            MoveClassification::Good,
            MoveClassification::Inaccuracy,
            MoveClassification::Mistake,
            MoveClassification::Blunder,
        ];
        for c in &classifications {
            let text = generic_template(*c);
            assert!(!text.is_empty(), "Empty generic template for {c:?}");
        }
    }

    #[test]
    fn all_themes_have_error_template() {
        let themes = [
            PositionalTheme::KnightOnRim,
            PositionalTheme::BishopPairAdvantage,
            PositionalTheme::IsolatedQueenPawn,
            PositionalTheme::PassedPawn,
            PositionalTheme::DoubledPawns,
            PositionalTheme::BackwardPawn,
            PositionalTheme::OpenFile,
            PositionalTheme::RookOnSeventh,
            PositionalTheme::KingSafetyCompromised,
            PositionalTheme::UndevelopedPieces,
            PositionalTheme::CentralControl,
            PositionalTheme::PawnChainTension,
            PositionalTheme::MaterialImbalance,
            PositionalTheme::BackRankWeakness,
            PositionalTheme::PinnedPiece,
            PositionalTheme::ForkAvailable,
            PositionalTheme::HangingMaterial,
        ];
        for theme in &themes {
            let text = theme_error_template(MoveClassification::Mistake, theme);
            assert!(text.is_some(), "No error template for theme {theme:?}");
            assert!(
                !text.unwrap().is_empty(),
                "Empty error template for theme {theme:?}"
            );
        }
    }

    #[test]
    fn all_tactics_have_template() {
        let tactics = [
            TacticType::Pin,
            TacticType::Fork,
            TacticType::Skewer,
            TacticType::HangingPiece,
            TacticType::BackRankThreat,
            TacticType::DiscoveredAttack,
        ];
        for tactic in &tactics {
            let text = tactic_template(tactic);
            assert!(!text.is_empty(), "Empty tactic template for {tactic:?}");
        }
    }

    #[test]
    fn rank_addendum_covers_all_bands_for_errors_and_best() {
        let bands = [
            RatingBand::Novice,
            RatingBand::Developing,
            RatingBand::Advancing,
            RatingBand::Expert,
        ];
        let ranked = [
            MoveClassification::Best,
            MoveClassification::Inaccuracy,
            MoveClassification::Mistake,
            MoveClassification::Blunder,
        ];
        for c in &ranked {
            let mut texts = Vec::new();
            for b in &bands {
                let text = rank_addendum(*c, *b);
                assert!(text.is_some(), "No rank addendum for {c:?} at {b:?}");
                let text = text.unwrap();
                assert!(!text.contains('%'), "No fabricated statistics: {text}");
                texts.push(text);
            }
            // Each band gets distinct phrasing.
            for pair in texts.windows(2) {
                assert_ne!(pair[0], pair[1], "Duplicate addendum for {c:?}");
            }
        }
    }

    #[test]
    fn rank_addendum_absent_for_neutral_moves() {
        for b in [
            RatingBand::Novice,
            RatingBand::Developing,
            RatingBand::Advancing,
            RatingBand::Expert,
        ] {
            assert!(rank_addendum(MoveClassification::Good, b).is_none());
            assert!(rank_addendum(MoveClassification::Excellent, b).is_none());
        }
    }

    #[test]
    fn phase_transitions_have_text() {
        let transitions = [
            (GamePhase::Opening, GamePhase::Middlegame),
            (GamePhase::Middlegame, GamePhase::Endgame),
            (GamePhase::Opening, GamePhase::Endgame),
        ];
        for (from, to) in &transitions {
            let text = phase_transition_text(from, to);
            assert!(
                !text.is_empty(),
                "Empty phase transition text for {from:?} -> {to:?}"
            );
        }
    }
}
