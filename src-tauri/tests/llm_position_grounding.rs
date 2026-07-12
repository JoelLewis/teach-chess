//! Real-model grounding tests for position-aware LLM coaching.
//!
//! These load the actual Gemma 4 E2B GGUF (~2.9 GiB) from
//! `src-tauri/models/` (run `scripts/fetch-model.sh` first) and are
//! `#[ignore]`d by default. Run in release mode — debug-mode llama.cpp is
//! painfully slow:
//!
//! ```sh
//! cargo test -p chess-mentor --release --test llm_position_grounding -- --ignored --nocapture
//! ```
//!
//! The **mechanical groundedness oracle**: every board square (`[a-h][1-8]`)
//! the model mentions must appear in the prompt text. With a fixed sampling
//! seed, outputs are stable per prompt.

#![cfg(feature = "llm")]

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use chess_mentor_lib::llm::PlayerLevel;
use chess_mentor_lib::llm::channel::FREE_TEXT_GRAMMAR;
use chess_mentor_lib::llm::coach_prompt::{build_user_prompt, system_prompt};
use chess_mentor_lib::llm::llm_support::{CHESS_N_CTX, CHESS_SAMPLER, coaching_generate_options};
use chess_mentor_lib::llm::position_facts::{
    EngineData, MoveInput, build_move_facts, uci_line_to_san,
};
use chess_mentor_lib::models::engine::Score;
use chess_mentor_lib::models::heuristics::CoachingContext;
use sensei_llm::{GenerateOptions, ModelManager, device_name, format_chat};

const MAX_TOKENS: u32 = 128;

/// Serializes generation across tests: they run in parallel threads by
/// default and concurrent contexts on the shared model flake.
static GEN_LOCK: Mutex<()> = Mutex::new(());

/// One case from `tests/fixtures/coaching_eval.json`.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct EvalCase {
    id: String,
    #[allow(dead_code)]
    phase: String,
    fen_before: String,
    player_move_uci: String,
    player_move_san: String,
    best_uci: String,
    pv: Vec<String>,
    refutation_pv: Vec<String>,
    eval_before: Score,
    eval_after: Score,
    classification: String,
    must_mention_any: Vec<String>,
    must_not_mention: Vec<String>,
}

fn fixture_cases() -> Vec<EvalCase> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/coaching_eval.json");
    let json = std::fs::read_to_string(&path).expect("coaching_eval.json readable");
    serde_json::from_str(&json).expect("coaching_eval.json parses")
}

/// Shared model instance: tests run in parallel threads by default, and two
/// concurrent 3-GiB Metal loads flake. `OnceLock` loads once; generation is
/// additionally serialized through a lock in [`generate`].
fn model() -> &'static ModelManager {
    static MODEL: OnceLock<ModelManager> = OnceLock::new();
    MODEL.get_or_init(load_model)
}

fn load_model() -> ModelManager {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("models")
        .join(sensei_llm::GEMMA4_E2B_Q4.filename);
    assert!(
        path.exists(),
        "model file missing at {} — run scripts/fetch-model.sh first",
        path.display()
    );
    let t0 = Instant::now();
    let manager = ModelManager::load(&path).expect("model loads");
    eprintln!(
        "model loaded in {:.1}s on {}",
        t0.elapsed().as_secs_f32(),
        device_name()
    );
    manager
}

fn context_for(fen: &str) -> CoachingContext {
    let pos = chess_mentor_lib::game::parse_fen(fen).expect("fixture FEN parses");
    chess_mentor_lib::heuristics::analyze_position(&pos)
}

/// Build the production user prompt for a fixture case.
fn case_user_prompt(case: &EvalCase) -> String {
    let engine = EngineData {
        eval_before: Some(case.eval_before.clone()),
        eval_after: Some(case.eval_after.clone()),
        best_move_san: uci_line_to_san(&case.fen_before, std::slice::from_ref(&case.best_uci), 1)
            .into_iter()
            .next(),
        pv: case.pv.clone(),
        refutation_pv: case.refutation_pv.clone(),
    };
    let ctx = context_for(&case.fen_before);
    let facts = build_move_facts(
        &MoveInput {
            fen_before: &case.fen_before,
            player_move_san: &case.player_move_san,
            player_move_uci: Some(&case.player_move_uci),
            classification: &case.classification,
        },
        Some(&ctx),
        Some(&engine),
    );
    build_user_prompt(&facts)
}

/// Reconstruction of the pre-position-aware prompt (bare enum names in JSON),
/// used to score the old-prompt baseline for comparison.
fn baseline_prompt(case: &EvalCase) -> String {
    let system = "You are a chess coach speaking to a club-level player (~1000-1500). \
                  Use standard chess terminology. Be direct and instructive. \
                  Reference specific squares when relevant. Keep responses to 2-3 sentences. \
                  Generate ONLY the coaching text.";

    let ctx = context_for(&case.fen_before);
    let themes: Vec<String> = ctx
        .themes
        .iter()
        .filter_map(|t| {
            serde_json::to_value(t)
                .ok()?
                .as_str()
                .map(|s| format!("\"{s}\""))
        })
        .collect();
    let tactics: Vec<String> = ctx
        .tactics
        .iter()
        .filter_map(|t| {
            serde_json::to_value(&t.tactic_type)
                .ok()?
                .as_str()
                .map(|s| format!("\"{s}\""))
        })
        .collect();
    let phase = serde_json::to_value(ctx.phase)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "middlegame".to_string());
    let better_move = uci_line_to_san(&case.fen_before, std::slice::from_ref(&case.best_uci), 1)
        .into_iter()
        .next()
        .map(|m| format!("\"{m}\""))
        .unwrap_or_else(|| "null".to_string());

    let context_json = format!(
        r#"{{"classification":"{}","phase":"{}","player_move":"{}","better_move":{},"themes":[{}],"tactics":[{}],"material_balance_cp":{}}}"#,
        case.classification,
        phase,
        case.player_move_san,
        better_move,
        themes.join(","),
        tactics.join(","),
        ctx.material.balance_cp,
    );
    format_chat(system, &context_json)
}

/// Extract every board-square mention (`[a-h][1-8]`) from text, lowercased.
fn extract_squares(text: &str) -> Vec<String> {
    let chars: Vec<char> = text.to_lowercase().chars().collect();
    let mut squares = Vec::new();
    for pair in chars.windows(2) {
        if ('a'..='h').contains(&pair[0]) && ('1'..='8').contains(&pair[1]) {
            squares.push(format!("{}{}", pair[0], pair[1]));
        }
    }
    squares.sort();
    squares.dedup();
    squares
}

/// Squares in `output` that do not appear anywhere in `prompt`.
fn ungrounded_squares(output: &str, prompt: &str) -> Vec<String> {
    let prompt_squares = extract_squares(prompt);
    extract_squares(output)
        .into_iter()
        .filter(|sq| !prompt_squares.contains(sq))
        .collect()
}

fn generate(prompt: &str) -> (String, Duration) {
    let _guard = GEN_LOCK.lock().expect("generation lock");
    let t0 = Instant::now();
    // The exact options the production inference channel uses.
    let opts = coaching_generate_options(MAX_TOKENS, FREE_TEXT_GRAMMAR);
    let text = model()
        .generate_streaming(prompt, &opts, |_| {})
        .expect("generation succeeds");
    (text, t0.elapsed())
}

fn case_by_id<'a>(cases: &'a [EvalCase], id: &str) -> &'a EvalCase {
    cases
        .iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("fixture case {id} missing"))
}

fn assert_no_markers(output: &str) {
    assert!(
        !output.contains("<|turn>") && !output.contains("<turn|>"),
        "output leaked turn markers: {output:?}"
    );
}

/// End-to-end inference smoke test against the real GGUF (ported from the
/// deleted `mentor-llm` crate). Unconstrained generation with ChessMentor's
/// pinned sampler, context size, and channel filtering.
#[test]
#[ignore = "loads the ~2.9 GiB Gemma 4 E2B model"]
fn llm_generates_coherent_text() {
    let _guard = GEN_LOCK.lock().expect("generation lock");

    let prompt = format_chat(
        "You are a concise chess coach.",
        "Reply with one short sentence: why is controlling the center important in chess?",
    );
    let opts = GenerateOptions {
        max_tokens: 128,
        n_ctx: CHESS_N_CTX,
        sampler: CHESS_SAMPLER,
        grammar: None,
        grammar_root: "root".to_string(),
        filter_channels: true,
    };

    let mut streamed = String::new();
    let mut chunks = 0u32;
    let t1 = Instant::now();
    let text = model()
        .generate_streaming(&prompt, &opts, |t| {
            chunks += 1;
            streamed.push_str(t);
        })
        .expect("generation should succeed");
    let gen_secs = t1.elapsed().as_secs_f32();
    eprintln!(
        "generation took {gen_secs:.1}s for {chunks} chunks ({:.2} chunks/s)",
        chunks as f32 / gen_secs
    );
    eprintln!("output: {text}");

    assert!(text.len() > 20, "suspiciously short output: {text:?}");
    assert_no_markers(&text);
    assert!(
        !text.contains("<|channel>") && !text.contains("<channel|>"),
        "output leaked channel markers: {text:?}"
    );
    assert_eq!(
        streamed.trim(),
        text,
        "streamed text should match final text"
    );
}

#[test]
#[ignore = "loads the ~2.9 GiB Gemma 4 E2B model"]
fn hanging_queen_blunder_is_grounded() {
    let cases = fixture_cases();
    let case = case_by_id(&cases, "hanging_queen_opening");
    let user_prompt = case_user_prompt(case);
    let prompt = format_chat(system_prompt(PlayerLevel::Intermediate), &user_prompt);
    let (output, elapsed) = generate(&prompt);

    eprintln!("--- prompt ---\n{user_prompt}\n--- output ({elapsed:?}) ---\n{output}");

    assert!(!output.trim().is_empty(), "empty coaching output");
    assert_no_markers(&output);

    let lower = output.to_lowercase();
    assert!(
        lower.contains("h5") || lower.contains("queen"),
        "output never mentions the hanging queen on h5: {output:?}"
    );

    let ungrounded = ungrounded_squares(&output, &user_prompt);
    assert!(
        ungrounded.is_empty(),
        "output mentions squares not in the prompt: {ungrounded:?}\noutput: {output:?}"
    );
}

#[test]
#[ignore = "loads the ~2.9 GiB Gemma 4 E2B model"]
fn missed_fork_is_grounded() {
    let cases = fixture_cases();
    let case = case_by_id(&cases, "missed_knight_fork");

    let user_prompt = case_user_prompt(case);
    assert!(
        user_prompt.contains("forks"),
        "pre-move facts should include the fork motif:\n{user_prompt}"
    );

    let prompt = format_chat(system_prompt(PlayerLevel::Intermediate), &user_prompt);
    let (output, elapsed) = generate(&prompt);

    eprintln!("--- prompt ---\n{user_prompt}\n--- output ({elapsed:?}) ---\n{output}");

    assert!(!output.trim().is_empty(), "empty coaching output");
    assert_no_markers(&output);

    let lower = output.to_lowercase();
    assert!(
        lower.contains("d6") || lower.contains("fork") || lower.contains("queen"),
        "output never mentions the missed fork on d6: {output:?}"
    );

    let ungrounded = ungrounded_squares(&output, &user_prompt);
    assert!(
        ungrounded.is_empty(),
        "output mentions squares not in the prompt: {ungrounded:?}\noutput: {output:?}"
    );
}

/// Score one generated response against a case's criteria.
/// Returns (passed, failure reasons).
fn score_output(output: &str, user_prompt: &str, case: &EvalCase) -> (bool, Vec<String>) {
    let mut reasons = Vec::new();
    let lower = output.to_lowercase();

    if output.trim().is_empty() {
        reasons.push("empty output".to_string());
    }
    if !case.must_mention_any.is_empty()
        && !case
            .must_mention_any
            .iter()
            .any(|m| lower.contains(&m.to_lowercase()))
    {
        reasons.push(format!("mentions none of {:?}", case.must_mention_any));
    }
    for banned in &case.must_not_mention {
        if lower.contains(&banned.to_lowercase()) {
            reasons.push(format!("mentions banned term {banned:?}"));
        }
    }
    let ungrounded = ungrounded_squares(output, user_prompt);
    if !ungrounded.is_empty() {
        reasons.push(format!("ungrounded squares {ungrounded:?}"));
    }

    (reasons.is_empty(), reasons)
}

#[test]
#[ignore = "loads the ~2.9 GiB Gemma 4 E2B model and runs the full eval set"]
fn coaching_eval_set_scores_at_least_80_percent() {
    let cases = fixture_cases();
    assert!(
        (10..=12).contains(&cases.len()),
        "expected 10-12 cases, got {}",
        cases.len()
    );
    let mut passed = 0usize;
    let mut baseline_passed = 0usize;
    let mut latencies = Vec::new();

    eprintln!("\n=== coaching eval set ({} cases) ===", cases.len());
    for case in &cases {
        // New position-aware prompt.
        let user_prompt = case_user_prompt(case);
        let prompt = format_chat(system_prompt(PlayerLevel::Intermediate), &user_prompt);
        let (output, elapsed) = generate(&prompt);
        latencies.push(elapsed);
        let (ok, reasons) = score_output(&output, &user_prompt, case);
        passed += usize::from(ok);

        // Old-prompt baseline (report only, no assertion).
        let old_prompt = baseline_prompt(case);
        let (old_output, _) = generate(&old_prompt);
        // The baseline is scored against its own prompt text so square
        // mentions are judged by the same rule it was generated under.
        let (old_ok, old_reasons) = score_output(&old_output, &old_prompt, case);
        baseline_passed += usize::from(old_ok);

        eprintln!(
            "[{}] {} ({elapsed:?})\n  new: {}\n  old: {} {}",
            if ok { "PASS" } else { "FAIL" },
            case.id,
            if ok {
                output.clone()
            } else {
                format!("{reasons:?} — {output}")
            },
            if old_ok { "pass" } else { "fail" },
            if old_ok {
                String::new()
            } else {
                format!("{old_reasons:?}")
            },
        );
    }

    let total = cases.len();
    let avg = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let max = latencies.iter().max().copied().unwrap_or_default();
    eprintln!(
        "\nscore: {passed}/{total} ({:.0}%) | old-prompt baseline: {baseline_passed}/{total} ({:.0}%)",
        100.0 * passed as f64 / total as f64,
        100.0 * baseline_passed as f64 / total as f64,
    );
    eprintln!("latency: avg {avg:?}, max {max:?} (generation only, prefill included)");

    assert!(
        passed as f64 / total as f64 >= 0.8,
        "eval set below 80%: {passed}/{total}"
    );
}
