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
