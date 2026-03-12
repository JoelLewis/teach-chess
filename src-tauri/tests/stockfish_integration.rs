//! Integration tests for Stockfish UCI communication.
//!
//! These tests require the real Stockfish binary to be present at
//! `src-tauri/binaries/stockfish-{target}`. Run `./scripts/fetch-stockfish.sh` first.
//!
//! These tests are `#[ignore]`d by default so `cargo test` skips them.
//! Run with: `cargo test -- --ignored`

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::Duration;

fn stockfish_binary_path() -> String {
    let target = if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin"
        } else {
            "x86_64-apple-darwin"
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            "aarch64-unknown-linux-gnu"
        } else {
            "x86_64-unknown-linux-gnu"
        }
    } else if cfg!(target_os = "windows") {
        "x86_64-pc-windows-msvc"
    } else {
        panic!("Unsupported platform for integration tests");
    };

    let ext = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };

    format!("binaries/stockfish-{target}{ext}")
}

struct StockfishProcess {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

impl Drop for StockfishProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn spawn_stockfish() -> StockfishProcess {
    let path = stockfish_binary_path();
    let mut child = Command::new(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| {
            panic!(
                "Failed to spawn Stockfish at {path}: {e}. Run ./scripts/fetch-stockfish.sh first."
            )
        });

    let stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    StockfishProcess {
        child,
        stdin,
        stdout,
    }
}

fn send(stdin: &mut std::process::ChildStdin, cmd: &str) {
    writeln!(stdin, "{cmd}").expect("Failed to write to Stockfish stdin");
    stdin.flush().expect("Failed to flush stdin");
}

fn read_until(
    stdout: &mut BufReader<std::process::ChildStdout>,
    prefix: &str,
    timeout: Duration,
) -> Vec<String> {
    let start = std::time::Instant::now();
    let mut lines = Vec::new();

    loop {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for '{prefix}' from Stockfish");
        }

        let mut line = String::new();
        stdout.read_line(&mut line).expect("Failed to read line");
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let matched = line.starts_with(prefix);
        lines.push(line);
        if matched {
            break;
        }
    }

    lines
}

#[test]
#[ignore]
fn uci_handshake() {
    let mut sf = spawn_stockfish();

    send(&mut sf.stdin, "uci");
    let lines = read_until(&mut sf.stdout, "uciok", Duration::from_secs(10));

    assert!(
        lines.iter().any(|l| l.starts_with("uciok")),
        "Expected uciok in response"
    );
    assert!(
        lines.iter().any(|l| l.starts_with("id name")),
        "Expected 'id name' in UCI response"
    );

    send(&mut sf.stdin, "quit");
}

#[test]
#[ignore]
fn position_eval_returns_bestmove() {
    let mut sf = spawn_stockfish();

    send(&mut sf.stdin, "uci");
    read_until(&mut sf.stdout, "uciok", Duration::from_secs(10));

    send(&mut sf.stdin, "isready");
    read_until(&mut sf.stdout, "readyok", Duration::from_secs(5));

    send(&mut sf.stdin, "position startpos");
    send(&mut sf.stdin, "go depth 10");

    let lines = read_until(&mut sf.stdout, "bestmove", Duration::from_secs(30));

    let bestmove_line = lines.iter().find(|l| l.starts_with("bestmove")).unwrap();
    let parts: Vec<&str> = bestmove_line.split_whitespace().collect();
    assert!(parts.len() >= 2, "bestmove should include a move");

    // Verify the move looks like a valid UCI move (e.g., "e2e4")
    let uci_move = parts[1];
    assert!(
        uci_move.len() >= 4,
        "UCI move should be at least 4 chars: {uci_move}"
    );

    send(&mut sf.stdin, "quit");
}

#[test]
#[ignore]
fn multipv_output() {
    let mut sf = spawn_stockfish();

    send(&mut sf.stdin, "uci");
    read_until(&mut sf.stdout, "uciok", Duration::from_secs(10));

    send(&mut sf.stdin, "setoption name MultiPV value 3");
    send(&mut sf.stdin, "isready");
    read_until(&mut sf.stdout, "readyok", Duration::from_secs(5));

    send(&mut sf.stdin, "position startpos");
    send(&mut sf.stdin, "go depth 10");

    let lines = read_until(&mut sf.stdout, "bestmove", Duration::from_secs(30));

    // Should have info lines with multipv indicators
    let multipv_lines: Vec<&String> = lines.iter().filter(|l| l.contains("multipv")).collect();
    assert!(
        !multipv_lines.is_empty(),
        "Expected multipv info lines in output"
    );

    // Should have at least some lines with multipv 2 or 3
    let has_multipv_2 = multipv_lines.iter().any(|l| l.contains("multipv 2"));
    let has_multipv_3 = multipv_lines.iter().any(|l| l.contains("multipv 3"));
    assert!(has_multipv_2, "Expected multipv 2 in output");
    assert!(has_multipv_3, "Expected multipv 3 in output");

    send(&mut sf.stdin, "quit");
}

#[test]
#[ignore]
fn ucinewgame_resets_state() {
    let mut sf = spawn_stockfish();

    send(&mut sf.stdin, "uci");
    read_until(&mut sf.stdout, "uciok", Duration::from_secs(10));

    // First game
    send(&mut sf.stdin, "isready");
    read_until(&mut sf.stdout, "readyok", Duration::from_secs(5));
    send(&mut sf.stdin, "position startpos");
    send(&mut sf.stdin, "go depth 5");
    read_until(&mut sf.stdout, "bestmove", Duration::from_secs(15));

    // New game
    send(&mut sf.stdin, "ucinewgame");
    send(&mut sf.stdin, "isready");
    read_until(&mut sf.stdout, "readyok", Duration::from_secs(5));

    // Second game from a different position
    send(
        &mut sf.stdin,
        "position fen rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
    );
    send(&mut sf.stdin, "go depth 5");
    let lines = read_until(&mut sf.stdout, "bestmove", Duration::from_secs(15));

    assert!(
        lines.iter().any(|l| l.starts_with("bestmove")),
        "Engine should respond after ucinewgame reset"
    );

    send(&mut sf.stdin, "quit");
}
