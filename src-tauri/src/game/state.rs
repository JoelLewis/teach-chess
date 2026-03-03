use std::collections::HashMap;

use shakmaty::{fen::Fen, san::San, uci::UciMove, CastlingMode, Chess, EnPassantMode, Position as _};

use crate::error::{AppError, GameError};
use crate::models::chess::{Color, GameOutcome, Position};
use crate::models::game::{GameConfig, GameRecord};

pub struct GameState {
    chess: Chess,
    config: Option<GameConfig>,
    san_history: Vec<String>,
    uci_history: Vec<String>,
    fen_history: Vec<String>,
    last_move: Option<[String; 2]>,
    game_id: Option<String>,
    started_at: Option<String>,
    is_resigned: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            chess: Chess::default(),
            config: None,
            san_history: Vec::new(),
            uci_history: Vec::new(),
            fen_history: vec![Fen::from_position(Chess::default(), EnPassantMode::Legal).to_string()],
            last_move: None,
            game_id: None,
            started_at: None,
            is_resigned: false,
        }
    }
}

impl GameState {
    pub fn new_game(&mut self, config: GameConfig) {
        self.chess = Chess::default();
        self.config = Some(config);
        self.san_history.clear();
        self.uci_history.clear();
        self.fen_history = vec![Fen::from_position(Chess::default(), EnPassantMode::Legal).to_string()];
        self.last_move = None;
        self.game_id = Some(uuid::Uuid::new_v4().to_string());
        self.started_at = Some(chrono_now());
        self.is_resigned = false;
    }

    pub fn make_move(&mut self, uci_str: &str) -> Result<(), AppError> {
        if self.config.is_none() {
            return Err(GameError::NoActiveGame.into());
        }
        if self.is_game_over() {
            return Err(GameError::GameOver.into());
        }

        // Try parsing as-is first; if it fails, try adding queen promotion suffix
        let (uci_move, effective_uci) = match uci_str.parse::<UciMove>() {
            Ok(m) => match m.to_move(&self.chess) {
                Ok(_) => (m, uci_str.to_string()),
                Err(_) => {
                    // Might be a promotion without suffix — try with queen
                    let with_promo = format!("{uci_str}q");
                    let promo_uci: UciMove = with_promo
                        .parse()
                        .map_err(|_| GameError::IllegalMove(uci_str.to_string()))?;
                    (promo_uci, with_promo)
                }
            },
            Err(_) => return Err(GameError::IllegalMove(uci_str.to_string()).into()),
        };

        let legal_move = uci_move
            .to_move(&self.chess)
            .map_err(|_| GameError::IllegalMove(effective_uci.clone()))?;

        // Convert to SAN before applying the move
        let san = San::from_move(&self.chess, &legal_move);
        self.san_history.push(san.to_string());
        self.uci_history.push(effective_uci.clone());

        // Extract from/to squares for last_move highlight
        let from = effective_uci[..2].to_string();
        let to = effective_uci[2..4].to_string();
        self.last_move = Some([from, to]);

        // Apply the move
        self.chess.play_unchecked(&legal_move);
        self.fen_history
            .push(Fen::from_position(self.chess.clone(), EnPassantMode::Legal).to_string());

        Ok(())
    }

    pub fn resign(&mut self) -> Result<GameRecord, AppError> {
        let config = self
            .config
            .as_ref()
            .ok_or(GameError::NoActiveGame)?;

        self.is_resigned = true;

        let winner = config.player_color.opposite();
        let result = match winner {
            Color::White => "1-0",
            Color::Black => "0-1",
        };

        Ok(self.build_game_record(result.to_string()))
    }

    pub fn complete_game(&self) -> Result<GameRecord, AppError> {
        self.config
            .as_ref()
            .ok_or(GameError::NoActiveGame)?;

        if !self.is_game_over() {
            return Err(GameError::GameNotOver.into());
        }

        let result = match self.outcome() {
            Some(GameOutcome::Checkmate { winner }) | Some(GameOutcome::Resignation { winner }) => {
                match winner {
                    Color::White => "1-0".to_string(),
                    Color::Black => "0-1".to_string(),
                }
            }
            Some(_) => "1/2-1/2".to_string(),
            None => "1/2-1/2".to_string(),
        };

        Ok(self.build_game_record(result))
    }

    pub fn to_position(&self) -> Position {
        let turn: Color = self.chess.turn().into();
        let dests = self.legal_moves_as_dests();
        let fen = Fen::from_position(self.chess.clone(), EnPassantMode::Legal).to_string();
        let is_check = self.chess.is_check();
        let is_game_over = self.is_game_over();
        let outcome = self.outcome();

        Position {
            fen,
            turn,
            dests,
            last_move: self.last_move.clone(),
            is_check,
            is_game_over,
            outcome,
            move_number: (self.san_history.len() as u32 / 2) + 1,
            san_history: self.san_history.clone(),
        }
    }

    /// Convert legal moves to the format chessground expects:
    /// { "e2": ["e3", "e4"], "g1": ["f3", "h3"] }
    pub fn legal_moves_as_dests(&self) -> HashMap<String, Vec<String>> {
        let mut dests: HashMap<String, Vec<String>> = HashMap::new();

        for m in self.chess.legal_moves() {
            let uci = UciMove::from_move(&m, CastlingMode::Standard);
            let uci_str = uci.to_string();
            let from = uci_str[..2].to_string();
            let to = uci_str[2..4].to_string();

            dests.entry(from).or_default().push(to);
        }

        dests
    }

    pub fn is_game_over(&self) -> bool {
        self.is_resigned || self.chess.is_game_over()
    }

    pub fn outcome(&self) -> Option<GameOutcome> {
        if self.is_resigned {
            let config = self.config.as_ref()?;
            let winner = config.player_color.opposite();
            return Some(GameOutcome::Resignation { winner });
        }

        if !self.chess.is_game_over() {
            return None;
        }

        let outcome = self.chess.outcome()?;
        match outcome {
            shakmaty::Outcome::Decisive { winner } => {
                Some(GameOutcome::Checkmate {
                    winner: winner.into(),
                })
            }
            shakmaty::Outcome::Draw => {
                if self.chess.is_stalemate() {
                    Some(GameOutcome::Stalemate)
                } else if self.chess.is_insufficient_material() {
                    Some(GameOutcome::InsufficientMaterial)
                } else {
                    Some(GameOutcome::Draw)
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn fen(&self) -> String {
        Fen::from_position(self.chess.clone(), EnPassantMode::Legal).to_string()
    }

    pub fn pgn(&self) -> String {
        let mut pgn = String::new();
        for (i, san) in self.san_history.iter().enumerate() {
            if i % 2 == 0 {
                pgn.push_str(&format!("{}. ", i / 2 + 1));
            }
            pgn.push_str(san);
            pgn.push(' ');
        }

        if let Some(outcome) = self.outcome() {
            match outcome {
                GameOutcome::Checkmate { winner } | GameOutcome::Resignation { winner } => {
                    match winner {
                        Color::White => pgn.push_str("1-0"),
                        Color::Black => pgn.push_str("0-1"),
                    }
                }
                _ => pgn.push_str("1/2-1/2"),
            }
        }

        pgn.trim().to_string()
    }

    #[allow(dead_code)]
    pub fn config(&self) -> Option<&GameConfig> {
        self.config.as_ref()
    }

    #[allow(dead_code)]
    pub fn fen_history(&self) -> &[String] {
        &self.fen_history
    }

    #[allow(dead_code)]
    pub fn san_history(&self) -> &[String] {
        &self.san_history
    }

    fn build_game_record(&self, result: String) -> GameRecord {
        let config = self.config.as_ref().unwrap();
        GameRecord {
            id: self.game_id.clone().unwrap_or_default(),
            player_id: String::new(), // set by caller
            pgn: self.pgn(),
            result,
            player_color: config.player_color,
            engine_elo: config.engine_strength.elo,
            move_count: self.san_history.len() as u32,
            started_at: self.started_at.clone().unwrap_or_default(),
            ended_at: Some(chrono_now()),
            time_control: format!(
                "{}+{}",
                config.time_control.initial_secs, config.time_control.increment_secs
            ),
            opponent_personality: config
                .personality
                .as_ref()
                .map(|p| serde_json::to_string(p).unwrap_or_default()),
            teaching_mode: config.teaching_mode,
        }
    }
}

fn chrono_now() -> String {
    // Simple ISO 8601 timestamp without chrono dependency
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    // Format as basic timestamp (good enough for MVP, can use chrono later)
    format!("{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::game::{EngineStrength, TimeControl};

    fn test_config() -> GameConfig {
        GameConfig {
            player_color: Color::White,
            engine_strength: EngineStrength::beginner(),
            time_control: TimeControl::default(),
            coaching_level: Default::default(),
            opponent_mode: Default::default(),
            personality: None,
            teaching_mode: false,
        }
    }

    #[test]
    fn new_game_starts_at_initial_position() {
        let mut state = GameState::default();
        state.new_game(test_config());
        let pos = state.to_position();

        assert_eq!(pos.turn, Color::White);
        assert!(!pos.is_game_over);
        assert!(!pos.is_check);
        assert!(pos.san_history.is_empty());
        assert!(!pos.dests.is_empty());
    }

    #[test]
    fn make_legal_move() {
        let mut state = GameState::default();
        state.new_game(test_config());

        state.make_move("e2e4").unwrap();
        let pos = state.to_position();

        assert_eq!(pos.turn, Color::Black);
        assert_eq!(pos.san_history, vec!["e4"]);
        assert_eq!(pos.last_move, Some(["e2".to_string(), "e4".to_string()]));
    }

    #[test]
    fn reject_illegal_move() {
        let mut state = GameState::default();
        state.new_game(test_config());

        let result = state.make_move("e2e5");
        assert!(result.is_err());
    }

    #[test]
    fn detect_scholars_mate() {
        let mut state = GameState::default();
        state.new_game(test_config());

        state.make_move("e2e4").unwrap();
        state.make_move("e7e5").unwrap();
        state.make_move("d1h5").unwrap();
        state.make_move("b8c6").unwrap();
        state.make_move("f1c4").unwrap();
        state.make_move("g8f6").unwrap();
        state.make_move("h5f7").unwrap();

        let pos = state.to_position();
        assert!(pos.is_game_over);
        assert!(matches!(
            pos.outcome,
            Some(GameOutcome::Checkmate {
                winner: Color::White
            })
        ));
    }

    #[test]
    fn legal_moves_as_dests_format() {
        let mut state = GameState::default();
        state.new_game(test_config());

        let dests = state.legal_moves_as_dests();
        // e2 pawn should be able to go to e3 and e4
        let e2_dests = dests.get("e2").unwrap();
        assert!(e2_dests.contains(&"e3".to_string()));
        assert!(e2_dests.contains(&"e4".to_string()));
    }

    #[test]
    fn resign_ends_game() {
        let mut state = GameState::default();
        state.new_game(test_config());

        let record = state.resign().unwrap();
        assert_eq!(record.result, "0-1"); // White resigns, Black wins
        assert!(state.is_game_over());
    }
}
