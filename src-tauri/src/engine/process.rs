use std::time::Duration;

use tauri::AppHandle;
use tauri_plugin_shell::process::CommandChild;
use tauri_plugin_shell::ShellExt;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::error::{AppError, EngineError};
use crate::models::engine::{EngineEvaluation, EngineMove, MoveEvaluation};
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

        // Read until bestmove
        self.wait_for_bestmove().await
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

    pub async fn review_game(
        &mut self,
        game: &GameRecord,
        depth: u32,
    ) -> Result<Vec<MoveEvaluation>, AppError> {
        if !self.is_running {
            return Err(EngineError::NotRunning.into());
        }

        // Parse PGN to get positions — for now, return empty
        // Full implementation requires replaying moves and analyzing each position
        let _ = game;
        let _ = depth;

        Ok(Vec::new())
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

    async fn wait_for_bestmove(&mut self) -> Result<EngineMove, AppError> {
        loop {
            let line = self.read_line().await?;
            if let Some(bestmove) = uci::parse_bestmove(&line) {
                return Ok(bestmove);
            }
        }
    }
}
