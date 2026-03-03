use crate::engine::uci::MultiPvLine;
use crate::models::engine::Score;
use crate::models::heuristics::{CoachingContext, TacticType};

use super::personality::PersonalityWeights;

/// A scored candidate move with personality and teaching bonuses.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScoredCandidate {
    pub uci_move: String,
    pub score: Score,
    pub personality_score: f64,
    pub teaching_score: f64,
    pub combined_score: f64,
}

/// Score a position's features against personality weights.
///
/// Returns a 0.0–1.0 score indicating how well the resulting position
/// matches the personality's preferences.
pub fn compute_personality_score(weights: &PersonalityWeights, context: &CoachingContext) -> f64 {
    let mut scores: Vec<f64> = Vec::new();
    let mut weight_sum = 0.0;

    // Tactics: how many tactical motifs exist in the position
    let tactic_count = context.tactics.len() as f64;
    let tactics_score = (tactic_count / 3.0).min(1.0); // normalize: 3+ tactics = 1.0
    scores.push(tactics_score * weights.tactics);
    weight_sum += weights.tactics;

    // Structure: inverse of pawn weakness count
    let pawn_weaknesses = context.pawns.white.isolated.len()
        + context.pawns.white.doubled.len()
        + context.pawns.white.backward.len()
        + context.pawns.black.isolated.len()
        + context.pawns.black.doubled.len()
        + context.pawns.black.backward.len();
    let structure_score = 1.0 - (pawn_weaknesses as f64 / 8.0).min(1.0);
    scores.push(structure_score * weights.structure);
    weight_sum += weights.structure;

    // King attack: opponent's king safety weakness
    let opp_king = &context.king_safety.black; // simplified: use black as "opponent"
    let shield_ratio = if opp_king.pawn_shield_max > 0 {
        1.0 - (opp_king.pawn_shield_count as f64 / opp_king.pawn_shield_max as f64)
    } else {
        0.5
    };
    let king_attack_score = (shield_ratio + opp_king.open_files_near_king as f64 * 0.2).min(1.0);
    scores.push(king_attack_score * weights.king_attack);
    weight_sum += weights.king_attack;

    // Activity: piece mobility
    let total_mobility =
        (context.activity.white.total_mobility + context.activity.black.total_mobility) as f64;
    let activity_score = (total_mobility / 60.0).min(1.0); // 60+ moves combined = highly active
    scores.push(activity_score * weights.activity);
    weight_sum += weights.activity;

    // Safety: own king's pawn shield strength
    let own_king = &context.king_safety.white;
    let own_shield = if own_king.pawn_shield_max > 0 {
        own_king.pawn_shield_count as f64 / own_king.pawn_shield_max as f64
    } else {
        0.5
    };
    let safety_score = own_shield;
    scores.push(safety_score * weights.safety);
    weight_sum += weights.safety;

    // Trap: positions with subtle tactical themes (hanging pieces, pins, forks from opponent side)
    let trap_tactics = context.tactics.iter().filter(|t| {
        matches!(
            t.tactic_type,
            TacticType::Pin | TacticType::Fork | TacticType::HangingPiece
        )
    });
    let trap_score = (trap_tactics.count() as f64 / 2.0).min(1.0);
    scores.push(trap_score * weights.trap);
    weight_sum += weights.trap;

    if weight_sum > 0.0 {
        scores.iter().sum::<f64>() / weight_sum
    } else {
        0.5
    }
}

/// How close a candidate's evaluation is to the best move.
/// Returns 1.0 for the best move, decreasing toward 0.0 for worse moves.
fn eval_closeness(candidate_score: &Score, best_score: &Score) -> f64 {
    let candidate_cp = score_to_cp(candidate_score);
    let best_cp = score_to_cp(best_score);
    let diff = (best_cp - candidate_cp).abs() as f64;

    // 0cp diff = 1.0, 200cp diff = ~0.0
    (1.0 - diff / 200.0).max(0.0)
}

/// Convert a Score to centipawns for comparison.
fn score_to_cp(score: &Score) -> i32 {
    match score {
        Score::Cp { value } => *value,
        Score::Mate { moves } => {
            if *moves > 0 {
                10000 - *moves * 100
            } else {
                -10000 - *moves * 100
            }
        }
    }
}

/// Score and select a move from multi-PV candidates using personality weights.
///
/// Algorithm:
/// 1. Filter out candidates that are >100cp worse than the best
/// 2. Score each candidate: 0.70 * personality + 0.20 * teaching + 0.10 * eval_closeness
/// 3. Softmax-weighted random selection (temperature=0.3)
pub fn select_move(
    candidates: &[MultiPvLine],
    weights: &PersonalityWeights,
    contexts: &[(String, CoachingContext)], // (uci_move, context_after)
    teaching_scores: &[(String, f64)],      // (uci_move, teaching_score)
) -> Option<ScoredCandidate> {
    if candidates.is_empty() {
        return None;
    }

    let best_cp = score_to_cp(&candidates[0].score);

    // Filter: reject moves >100cp worse than best
    let viable: Vec<&MultiPvLine> = candidates
        .iter()
        .filter(|c| {
            let cp = score_to_cp(&c.score);
            (best_cp - cp).abs() <= 100
        })
        .collect();

    if viable.is_empty() {
        // Shouldn't happen, but fall back to best move
        let c = &candidates[0];
        return Some(ScoredCandidate {
            uci_move: c.uci_move.clone(),
            score: c.score.clone(),
            personality_score: 0.5,
            teaching_score: 0.0,
            combined_score: 1.0,
        });
    }

    // Score each viable candidate
    let mut scored: Vec<ScoredCandidate> = viable
        .iter()
        .map(|c| {
            let personality_score = contexts
                .iter()
                .find(|(m, _)| *m == c.uci_move)
                .map(|(_, ctx)| compute_personality_score(weights, ctx))
                .unwrap_or(0.5);

            let teaching_score = teaching_scores
                .iter()
                .find(|(m, _)| *m == c.uci_move)
                .map(|(_, s)| *s)
                .unwrap_or(0.0);

            let closeness = eval_closeness(&c.score, &candidates[0].score);

            let combined = 0.70 * personality_score + 0.20 * teaching_score + 0.10 * closeness;

            ScoredCandidate {
                uci_move: c.uci_move.clone(),
                score: c.score.clone(),
                personality_score,
                teaching_score,
                combined_score: combined,
            }
        })
        .collect();

    // Softmax selection with temperature=0.3
    softmax_select(&mut scored)
}

/// Softmax-weighted random selection from scored candidates.
fn softmax_select(candidates: &mut [ScoredCandidate]) -> Option<ScoredCandidate> {
    if candidates.is_empty() {
        return None;
    }
    if candidates.len() == 1 {
        return Some(candidates[0].clone());
    }

    let temperature = 0.3;

    // Compute softmax weights
    let max_score = candidates
        .iter()
        .map(|c| c.combined_score)
        .fold(f64::NEG_INFINITY, f64::max);

    let exp_scores: Vec<f64> = candidates
        .iter()
        .map(|c| ((c.combined_score - max_score) / temperature).exp())
        .collect();

    let sum: f64 = exp_scores.iter().sum();

    // Simple deterministic-ish selection using hash of scores
    // (real random would require the `rand` crate, which we avoid)
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for c in candidates.iter() {
        c.uci_move.hash(&mut hasher);
        c.combined_score.to_bits().hash(&mut hasher);
    }
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .hash(&mut hasher);
    let hash = hasher.finish();

    // Use hash to pick a probability bucket
    let random_val = (hash % 10000) as f64 / 10000.0;
    let mut cumulative = 0.0;

    for (i, exp_s) in exp_scores.iter().enumerate() {
        cumulative += exp_s / sum;
        if random_val < cumulative {
            return Some(candidates[i].clone());
        }
    }

    // Fallback: return the highest scored
    candidates.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());
    Some(candidates[0].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::heuristics::*;

    fn default_context() -> CoachingContext {
        CoachingContext {
            fen: "startpos".to_string(),
            phase: GamePhase::Opening,
            material: MaterialBalance {
                white: PieceCounts {
                    pawns: 8,
                    knights: 2,
                    bishops: 2,
                    rooks: 2,
                    queens: 1,
                },
                black: PieceCounts {
                    pawns: 8,
                    knights: 2,
                    bishops: 2,
                    rooks: 2,
                    queens: 1,
                },
                balance_cp: 0,
                imbalances: vec![],
            },
            pawns: PawnStructure {
                white: SidePawnStructure {
                    isolated: vec![],
                    doubled: vec![],
                    passed: vec![],
                    backward: vec![],
                },
                black: SidePawnStructure {
                    isolated: vec![],
                    doubled: vec![],
                    passed: vec![],
                    backward: vec![],
                },
                chains: vec![],
                open_files: vec![],
                half_open_files_white: vec![],
                half_open_files_black: vec![],
            },
            activity: PieceActivity {
                white: SideActivity {
                    total_mobility: 20,
                    developed_minors: 2,
                    total_minors: 4,
                    rook_on_open_file: false,
                    rook_on_seventh: false,
                    pieces: vec![],
                },
                black: SideActivity {
                    total_mobility: 20,
                    developed_minors: 2,
                    total_minors: 4,
                    rook_on_open_file: false,
                    rook_on_seventh: false,
                    pieces: vec![],
                },
            },
            king_safety: KingSafety {
                white: SideKingSafety {
                    king_square: "g1".to_string(),
                    pawn_shield_count: 3,
                    pawn_shield_max: 3,
                    has_castled: true,
                    can_castle: false,
                    open_files_near_king: 0,
                    king_zone_attacks: 0,
                },
                black: SideKingSafety {
                    king_square: "g8".to_string(),
                    pawn_shield_count: 3,
                    pawn_shield_max: 3,
                    has_castled: true,
                    can_castle: false,
                    open_files_near_king: 0,
                    king_zone_attacks: 0,
                },
            },
            tactics: vec![],
            themes: vec![],
        }
    }

    fn tactical_context() -> CoachingContext {
        let mut ctx = default_context();
        ctx.tactics.push(TacticalMotif {
            tactic_type: TacticType::Fork,
            side: Side::White,
            square: "e4".to_string(),
            description: "Knight fork".to_string(),
        });
        ctx.tactics.push(TacticalMotif {
            tactic_type: TacticType::Pin,
            side: Side::White,
            square: "c4".to_string(),
            description: "Bishop pin".to_string(),
        });
        ctx.king_safety.black.pawn_shield_count = 1;
        ctx
    }

    #[test]
    fn personality_score_in_range() {
        let ctx = default_context();
        let profiles = [
            super::super::personality::PersonalityProfile::Aggressive,
            super::super::personality::PersonalityProfile::Positional,
            super::super::personality::PersonalityProfile::Trappy,
            super::super::personality::PersonalityProfile::Solid,
        ];
        for p in &profiles {
            let s = compute_personality_score(&p.weights(), &ctx);
            assert!((0.0..=1.0).contains(&s), "Score {s} out of range for {p:?}");
        }
    }

    #[test]
    fn aggressive_prefers_tactical_positions() {
        use super::super::personality::PersonalityProfile;

        let calm = default_context();
        let sharp = tactical_context();

        let agg_w = PersonalityProfile::Aggressive.weights();
        let calm_score = compute_personality_score(&agg_w, &calm);
        let sharp_score = compute_personality_score(&agg_w, &sharp);

        assert!(
            sharp_score > calm_score,
            "Aggressive should prefer tactical ({sharp_score}) over calm ({calm_score})"
        );
    }

    #[test]
    fn select_move_returns_viable_candidate() {
        let candidates = vec![
            MultiPvLine {
                pv_index: 1,
                uci_move: "e2e4".to_string(),
                score: Score::cp(30),
                depth: 14,
            },
            MultiPvLine {
                pv_index: 2,
                uci_move: "d2d4".to_string(),
                score: Score::cp(25),
                depth: 14,
            },
            MultiPvLine {
                pv_index: 3,
                uci_move: "c2c4".to_string(),
                score: Score::cp(15),
                depth: 14,
            },
        ];

        let ctx1 = default_context();
        let ctx2 = tactical_context();
        let ctx3 = default_context();

        let contexts = vec![
            ("e2e4".to_string(), ctx1),
            ("d2d4".to_string(), ctx2),
            ("c2c4".to_string(), ctx3),
        ];

        let weights = super::super::personality::PersonalityProfile::Aggressive.weights();
        let result = select_move(&candidates, &weights, &contexts, &[]);

        assert!(result.is_some());
        let selected = result.unwrap();
        // Should select one of the viable candidates
        assert!(["e2e4", "d2d4", "c2c4"].contains(&selected.uci_move.as_str()));
    }

    #[test]
    fn select_move_filters_bad_candidates() {
        let candidates = vec![
            MultiPvLine {
                pv_index: 1,
                uci_move: "e2e4".to_string(),
                score: Score::cp(100),
                depth: 14,
            },
            MultiPvLine {
                pv_index: 2,
                uci_move: "a2a3".to_string(),
                score: Score::cp(-150),
                depth: 14,
            },
        ];

        let ctx = default_context();
        let contexts = vec![("e2e4".to_string(), ctx.clone()), ("a2a3".to_string(), ctx)];

        let weights = super::super::personality::PersonalityProfile::Solid.weights();
        let result = select_move(&candidates, &weights, &contexts, &[]);

        assert!(result.is_some());
        // a2a3 is 250cp worse, should be filtered out
        assert_eq!(result.unwrap().uci_move, "e2e4");
    }

    #[test]
    fn eval_closeness_best_move_is_one() {
        let score = Score::cp(50);
        assert!((eval_closeness(&score, &score) - 1.0).abs() < 0.001);
    }

    #[test]
    fn eval_closeness_decreases_with_distance() {
        let best = Score::cp(50);
        let close = Score::cp(30);
        let far = Score::cp(-100);

        assert!(eval_closeness(&close, &best) > eval_closeness(&far, &best));
    }

    #[test]
    fn empty_candidates_returns_none() {
        let weights = super::super::personality::PersonalityProfile::Solid.weights();
        assert!(select_move(&[], &weights, &[], &[]).is_none());
    }
}
