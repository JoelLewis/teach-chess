use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use tauri::Emitter;

use crate::engine::eval;
use crate::error::{AppError, EngineError};
use crate::models::engine::{EngineEvaluation, EngineMove, MoveEvaluation, Score};
use crate::models::game::GameRecord;

use super::uci;

pub struct EngineProcess {
    child: Option<CommandChild>,
    stdout_rx: Option<mpsc::Receiver<String>>,
    is_running: bool,
}

impl Default for EngineProcess {
    fn default() -> Self {
        Self {
            child: None,
            stdout_rx: None,
            is_running: false,
        }
    }
}

impl EngineProcess {
    pub async fn start(&mut self, app: &AppHandle) -> Result<(), AppError> {
        if self.is_running {
            return Err(EngineError::AlreadyRunning.into());
        }

        let sidecar = app
            .shell()
            .sidecar("stockfish")
            .map_err(|e| EngineError::ProcessError(e.to_string()))?;

        let (mut rx, child) = sidecar
            .spawn()
            .map_err(|e| EngineError::ProcessError(e.to_string()))?;

        // Channel for forwarding stdout lines from the event receiver
        let (stdout_tx, stdout_rx) = mpsc::channel::<String>(256);

        // Spawn task to read engine stdout events
        tokio::spawn(async move {
            use tauri_plugin_shell::process::CommandEvent;
            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stdout(line) => {
                        let line = String::from_utf8_lossy(&line).trim().to_string();
                        if !line.is_empty() {
                            debug!("Engine: {line}");
                            if stdout_tx.send(line).await.is_err() {
                                break;
                            }
                        }
                    }
                    CommandEvent::Stderr(line) => {
                        let line = String::from_utf8_lossy(&line).trim().to_string();
                        if !line.is_empty() {
                            warn!("Engine stderr: {line}");
                        }
                    }
                    CommandEvent::Terminated(_) => {
                        info!("Engine process terminated");
                        break;
                    }
                    _ => {}
                }
            }
        });

        self.child = Some(child);
        self.stdout_rx = Some(stdout_rx);
        self.is_running = true;

        // Initialize UCI protocol
        self.send("uci").await?;
        self.wait_for("uciok").await?;
        self.send("isready").await?;
        self.wait_for("readyok").await?;

        info!("Stockfish engine started and ready");
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), AppError> {
        if !self.is_running {
            return Ok(());
        }

        let _ = self.send("quit").await;

        if let Some(child) = self.child.take() {
            let _ = child.kill();
        }

        self.stdout_rx = None;
        self.is_running = false;

        info!("Engine stopped");
        Ok(())
    }

    pub async fn get_move(
        &mut self,
        fen: &str,
        depth: Option<u32>,
        elo: Option<u32>,
        skill_level: Option<u8>,
        app: Option<&AppHandle>,
    ) -> Result<EngineMove, AppError> {
        if !self.is_running {
            return Err(EngineError::NotRunning.into());
        }

        // Set strength options
        for cmd in uci::strength_commands(elo, skill_level) {
            self.send(&cmd).await?;
        }

        // Set position and search
        self.send(&uci::position_command(fen)).await?;

        let go_cmd = match depth {
            Some(d) => uci::go_depth_command(d),
            None => "go depth 12".to_string(),
        };
        self.send(&go_cmd).await?;

        // Read until bestmove, emitting info events along the way
        self.wait_for_bestmove_with_info(app).await
    }

    pub async fn analyze(
        &mut self,
        fen: &str,
        depth: u32,
    ) -> Result<EngineEvaluation, AppError> {
        if !self.is_running {
            return Err(EngineError::NotRunning.into());
        }

        // Disable strength limits for analysis
        self.send("setoption name UCI_LimitStrength value false")
            .await?;

        self.send(&uci::position_command(fen)).await?;
        self.send(&uci::go_depth_command(depth)).await?;

        // Read info lines and bestmove
        let mut last_info = uci::InfoLine::default();
        loop {
            let line = self.read_line().await?;

            if let Some(info) = uci::parse_info(&line) {
                // Only track multipv 1 (or no multipv)
                if info.multipv.is_none() || info.multipv == Some(1) {
                    last_info = info;
                }
            }

            if let Some(bestmove) = uci::parse_bestmove(&line) {
                return Ok(uci::info_to_evaluation(&last_info, &bestmove.uci));
            }
        }
    }

    /// Run multi-PV analysis at full strength to get the top N candidate moves.
    ///
    /// Returns up to `num_pvs` candidate lines sorted by PV index (best first).
    /// Strength limits are disabled so evaluations are accurate for candidate scoring.
    pub async fn get_multi_pv(
        &mut self,
        fen: &str,
        depth: u32,
        num_pvs: u32,
    ) -> Result<Vec<uci::MultiPvLine>, AppError> {
        if !self.is_running {
            return Err(EngineError::NotRunning.into());
        }

        // Disable strength limits — we need accurate evals for candidate filtering
        self.send("setoption name UCI_LimitStrength value false")
            .await?;
        self.send(&format!("setoption name MultiPV value {num_pvs}"))
            .await?;

        self.send(&uci::position_command(fen)).await?;
        self.send(&uci::go_depth_command(depth)).await?;

        // Collect the highest-depth info line for each PV index
        let mut best_lines: std::collections::HashMap<u32, uci::InfoLine> =
            std::collections::HashMap::new();

        loop {
            let line = self.read_line().await?;

            if let Some(info) = uci::parse_info(&line) {
                if let Some(idx) = info.multipv {
                    let existing_depth = best_lines
                        .get(&idx)
                        .and_then(|i| i.depth)
                        .unwrap_or(0);
                    if info.depth.unwrap_or(0) >= existing_depth {
                        best_lines.insert(idx, info);
                    }
                }
            }

            if uci::parse_bestmove(&line).is_some() {
                break;
            }
        }

        // Reset MultiPV to default
        self.send("setoption name MultiPV value 1").await?;

        // Convert to MultiPvLine, sorted by PV index
        let mut result: Vec<uci::MultiPvLine> = best_lines
            .into_iter()
            .filter_map(|(idx, info)| {
                let uci_move = info.pv.first()?.clone();
                let score = info.score?;
                Some(uci::MultiPvLine {
                    pv_index: idx,
                    uci_move,
                    score,
                    depth: info.depth.unwrap_or(0),
                })
            })
            .collect();

        result.sort_by_key(|l| l.pv_index);
        Ok(result)
    }

    pub async fn review_game(
        &mut self,
        game: &GameRecord,
        depth: u32,
        app: Option<&AppHandle>,
    ) -> Result<Vec<MoveEvaluation>, AppError> {
        if !self.is_running {
            return Err(EngineError::NotRunning.into());
        }

        // Replay the PGN to extract positions and moves
        let replay = replay_pgn(&game.pgn)?;
        let total = replay.len();

        if total == 0 {
            return Ok(Vec::new());
        }

        // Disable strength limits for analysis
        self.send("setoption name UCI_LimitStrength value false")
            .await?;

        let mut evaluations = Vec::new();

        // Cache: eval_after from the previous move is eval_before for the next move,
        // since the position after move N is the position before move N+1.
        let mut cached_eval: Option<EngineEvaluation> = None;

        // Analyze position before each move
        for (i, step) in replay.iter().enumerate() {
            // Emit progress
            if let Some(app) = app {
                let _ = app.emit("review-progress", ReviewProgressPayload {
                    current: (i + 1) as u32,
                    total: total as u32,
                });
            }

            // Reuse cached eval_after from previous move as eval_before, or analyze fresh
            let eval_before = match cached_eval.take() {
                Some(cached) => cached,
                None => self.analyze(&step.fen_before, depth).await?,
            };

            // Analyze position after the move
            let eval_after = self.analyze(&step.fen_after, depth).await?;
            cached_eval = Some(eval_after.clone());

            let is_white = step.is_white;
            let classification = eval::classify_move(
                &eval_before.score,
                &eval_after.score,
                is_white,
            );

            // Compute coaching context from the position before the move
            let coaching_context = crate::game::parse_fen(&step.fen_before)
                .ok()
                .map(|pos| crate::heuristics::analyze_position(&pos));

            let coaching_text = coaching_context.as_ref().map(|ctx| {
                crate::coaching::generate_coaching_text(&classification, ctx)
            });

            evaluations.push(MoveEvaluation {
                move_number: step.move_number,
                is_white,
                fen_before: step.fen_before.clone(),
                player_move_uci: step.uci.clone(),
                player_move_san: step.san.clone(),
                engine_best_uci: Some(eval_before.best_move.clone()),
                engine_best_san: None, // Would need shakmaty to convert
                eval_before: Some(eval_before.score),
                eval_after: Some(eval_after.score),
                classification: Some(classification),
                depth,
                pv: eval_before.pv,
                coaching_context,
                coaching_text,
            });
        }

        Ok(evaluations)
    }

    async fn send(&mut self, cmd: &str) -> Result<(), AppError> {
        let child = self
            .child
            .as_mut()
            .ok_or(EngineError::NotRunning)?;

        debug!("-> Engine: {cmd}");
        let cmd_with_newline = format!("{cmd}\n");
        child
            .write(cmd_with_newline.as_bytes())
            .map_err(|e| EngineError::ProcessError(e.to_string()))?;

        Ok(())
    }

    async fn read_line(&mut self) -> Result<String, AppError> {
        let rx = self
            .stdout_rx
            .as_mut()
            .ok_or(EngineError::NotRunning)?;

        match tokio::time::timeout(Duration::from_secs(30), rx.recv()).await {
            Ok(Some(line)) => Ok(line),
            Ok(None) => Err(EngineError::ProcessError("Engine stdout closed".to_string()).into()),
            Err(_) => Err(EngineError::Timeout.into()),
        }
    }

    async fn wait_for(&mut self, expected: &str) -> Result<(), AppError> {
        loop {
            let line = self.read_line().await?;
            if line.starts_with(expected) {
                return Ok(());
            }
        }
    }

    #[allow(dead_code)]
    async fn wait_for_bestmove(&mut self) -> Result<EngineMove, AppError> {
        loop {
            let line = self.read_line().await?;
            if let Some(bestmove) = uci::parse_bestmove(&line) {
                return Ok(bestmove);
            }
        }
    }

    async fn wait_for_bestmove_with_info(
        &mut self,
        app: Option<&AppHandle>,
    ) -> Result<EngineMove, AppError> {
        loop {
            let line = self.read_line().await?;

            if let Some(info) = uci::parse_info(&line) {
                if (info.multipv.is_none() || info.multipv == Some(1)) && info.score.is_some() {
                    if let Some(app) = app {
                        let _ = app.emit("engine-info", EngineInfoPayload {
                            depth: info.depth.unwrap_or(0),
                            score: info.score.clone().unwrap(),
                            nodes: info.nodes.unwrap_or(0),
                            pv: info.pv.clone(),
                        });
                    }
                }
            }

            if let Some(bestmove) = uci::parse_bestmove(&line) {
                return Ok(bestmove);
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct EngineInfoPayload {
    depth: u32,
    score: Score,
    nodes: u64,
    pv: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ReviewProgressPayload {
    current: u32,
    total: u32,
}

struct ReplayStep {
    move_number: u32,
    is_white: bool,
    fen_before: String,
    fen_after: String,
    uci: String,
    san: String,
}

/// Replay a PGN string to extract positions and moves
fn replay_pgn(pgn: &str) -> Result<Vec<ReplayStep>, AppError> {
    use shakmaty::{fen::Fen, san::San, uci::UciMove, CastlingMode, Chess, EnPassantMode, Position as _};

    let mut chess = Chess::default();
    let mut steps = Vec::new();

    // Strip result from end of PGN
    let pgn_clean = pgn
        .replace("1-0", "")
        .replace("0-1", "")
        .replace("1/2-1/2", "")
        .replace('*', "");

    // Parse SAN tokens from the PGN
    for token in pgn_clean.split_whitespace() {
        // Skip move numbers like "1." or "2."
        if token.ends_with('.') || token.is_empty() {
            continue;
        }

        let fen_before = Fen::from_position(chess.clone(), EnPassantMode::Legal).to_string();
        let is_white = chess.turn() == shakmaty::Color::White;
        let move_number = chess.fullmoves().get();

        // Parse SAN
        let san: San = match token.parse() {
            Ok(s) => s,
            Err(_) => continue, // Skip unparseable tokens
        };

        let legal_move = match san.to_move(&chess) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let san_str = token.to_string();
        let uci = UciMove::from_move(&legal_move, CastlingMode::Standard).to_string();

        chess.play_unchecked(&legal_move);

        let fen_after = Fen::from_position(chess.clone(), EnPassantMode::Legal).to_string();

        steps.push(ReplayStep {
            move_number,
            is_white,
            fen_before,
            fen_after,
            uci,
            san: san_str,
        });
    }

    Ok(steps)
}
