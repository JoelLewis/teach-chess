# Plan: Position-Aware LLM Coaching

Status: planned, not yet implemented. Estimate ~13–18 hours across 4 independently-shippable steps.

## Problem

The coaching prompt (`src-tauri/src/llm/` + `crates/mentor-llm/src/prompts.rs`) passes only
classification/phase/themes/tactics enum names — no actual position — so the Gemma 4 E2B model's
board-specific phrasing is plausible-sounding theme inference, not analysis. A real-model playtest
measured ~21 tok/s generation and 0.6–2s per coaching response against a 30s budget, leaving ~15×
latency headroom to spend on richer context.

## Current-state findings

- `heuristics::analyze_position` already produces fully verbalized, square-grounded facts
  (e.g. `TacticalMotif.description` = "Knight on d4 forks queen on b7 and rook on f1") — the prompt
  builder reduces these to bare enum names. The grounding facts exist and are dropped on the floor.
- Context is pre-move only: `CoachingContext` is computed on `fen_before`; the consequence of a
  blunder (the newly hanging piece, the new fork) lives in `fen_after` and is never analyzed.
  This is the single biggest grounding gap.
- Engine data is partially plumbed: `MoveEvaluation.engine_best_san` is `None` during review
  (needs shakmaty conversion), and the PV of `eval_after` (the refutation line) is computed then
  discarded. `generate_coaching` receives none of the eval/PV data.
- Cache bug: the response cache key (`fen|level|classification|themes`) omits the played move — two
  different mistakes from the same position with the same classification already collide, and this
  becomes a visible correctness bug once responses name the played move.
- teach-go's LLM prompt is natural-language prose sections with a GBNF grammar; teach-chess's
  mentor-llm has no grammar path yet. Both stacks are being unified into sensei-kit; prompt content
  is app-side by design.

## Design decisions

### 1. Position representation: verbalized facts, not FEN

Feed the model: (a) `TacticalMotif.description` strings verbatim, (b) verbalized pawn-structure /
king-safety / activity facts filtered for relevance, (c) a **post-move fact diff** (new tactics that
appeared after the played move), and (d) a compact piece list per side
(`White: Kg1, Qd1, Ra1, ..., pawns a2 b2 ...`) as an existence reference, placed last and
subordinated to the facts.

Do **not** include raw FEN: 2B-class models reliably fail at FEN decoding (run-length digits, rank
order, case), so a FEN would cause hallucination rather than prevent it. What a 2B model can do is
restate and lightly connect facts stated in plain English. "Local region around the mistake" was
rejected: the post-move tactic diff captures the local consequence with less code and zero ambiguity.

### 2. Engine data: all three, verbalized

- Played vs best move in SAN (fix `engine_best_san: None` via shakmaty conversion).
- Eval delta in plain language, bucketed from the student's perspective ("You went from slightly
  better (+0.8) to losing (−2.4) — about a 3-pawn swing."). Buckets: equal / slightly (±0.3–1.0) /
  clearly (1.0–3.0) / winning-losing (>3.0) / mate-in-N.
- Two lines, ≤3 plies each, in SAN: the best line (PV of `eval_before`) and the refutation
  (PV of `eval_after` — requires persisting a new `refutation_pv: Vec<String>` on `MoveEvaluation`).

### 3. Token budget: ~450–650 prompt tokens is comfortably safe

System ~120 + user ~300–450 (facts ~150–250, engine ~60, piece list ~70) + turn markers ~15
≈ 450–650 tokens vs ~150 today. Metal prefill for a 2B model is 10–30× generation speed; even a
conservative 100 tok/s floor gives ~6.5s worst-case prefill (realistic ~2s). Generation unchanged
(worst 128 tokens / 21 tok/s ≈ 6.1s). Totals: typical ~3–4s, worst ~13s vs the 30s budget.
Bump `N_CTX` 1024 → 2048 for margin. Enforce a 2,600-char cap on the built user prompt, truncating
facts in priority order: post-move tactics → pre-move tactics → eval/lines → king safety → pawns →
activity → piece list.

### 4. Anti-hallucination

- System prompt rewrite: base every claim ONLY on the listed facts; never mention a piece, square,
  threat, or plan not in the facts; 2–3 sentences; output only coaching text.
- `MAX_TOKENS` stays 128 (typical outputs are 15–40 tokens; more invites rambling).
- GBNF: port teach-go's grammar-capable generation into mentor-llm and use a minimal free-text
  grammar `root ::= [^<>]{1,600}` — hard-blocks turn/channel-marker leakage and runaway length
  without forcing a tagged shape (chess has no field to extract). Pre-aligns the two model.rs files
  for sensei-kit. Temperature 0.3 and fixed seed stay (test determinism).

### 5. Cache key v2 (correctness fix)

Replace `compute_cache_key(fen, level, classification, themes)` with
`compute_cache_key(fen, level, user_prompt)` — hash `"{fen}|{level}|{user_prompt}"`. The prompt
deterministically contains classification, played move, best move, lines, and all facts, so
everything flows into the key by construction. Old rows age out via the 30-day TTL. Requires
building the prompt before the cache check in `commands/llm.rs::generate_coaching` (pure + cheap).

### 6. sensei-kit split placement

| Stays reusable (mentor-llm → sensei-kit) | App-side (src-tauri) |
|---|---|
| `format_chat`, `ChannelFilter`, `ModelManager` (+ grammar-capable generate), `download.rs`, `LlmError` | All prompt content: system prompts, fact verbalization (`llm/position_facts.rs`), prompt builders (`llm/coach_prompt.rs`), `PlayerLevel`, game-summary prompt |

## Proposed prompt (intermediate level)

System:

```
You are a chess coach speaking to a club-level player (~1000-1500) who just
played a move. You will be given verified facts about the position computed
by a chess engine and analysis heuristics.

Rules you must follow:
- Base every claim ONLY on the facts listed. Never mention a piece, square,
  threat, or plan that is not in the facts.
- If a better move is given, name it and explain it using the given lines.
- Refer to squares only if they appear in the facts.
- 2-3 sentences. Output only the coaching text - no headers, no lists,
  no notation dumps.
```

User (sections omitted when empty; positive moves replace "punishment" with "why it works"):

```
Move 18 as White: you played Qxb7 - a blunder. You went from slightly
better (+0.8) to losing (-2.4), about a 3-pawn swing.

Better was Rad1. Best line: Rad1 Nc6 d5.
After your move the opponent's punishment is: Nc5 Qa6 Rb8.

What your move changed (new problems after Qxb7):
- Queen on b7 is undefended and attacked
- Knight on c5 forks queen on b7 and rook on f1

Position facts (before your move):
- Open file: c
- Your king: castled kingside, pawn shield intact (3/3)
- Opponent king: castled kingside, 1 open file nearby
- Material: equal

Pieces - White: Kg1 Qb7 Ra1 Rf1 Nf3 Bd3, pawns a2 b2 f2 g2 h2.
Black: Kg8 Qd8 Rb8 Rf8 Nc5 Bd6, pawns a7 e5 f7 g7 h7.
```

## Implementation steps

1. **Persist missing engine data** (~1.5h) — `engine/process.rs` + `models/engine.rs`: add
   `refutation_pv` from `eval_after.pv`; fill `engine_best_san` via shakmaty; regenerate bindings.
2. **New pure module `src-tauri/src/llm/position_facts.rs`** (~3–4h, TDD) — `EngineData`,
   `MoveFacts`, `build_move_facts(...)`, `verbalize_eval_swing`, `uci_line_to_san`,
   `post_move_tactic_diff` (applies the move, re-runs `analyze_position` on `fen_after`, reports
   new motif descriptions for/against the mover), `piece_list`. Pure shakmaty, unit-testable
   without the model.
3. **Move prompt content app-side: `src-tauri/src/llm/coach_prompt.rs`** (~2–3h) — anti-hallucination
   system prompts (3 levels), `build_coaching_prompt` with the char cap, game-summary prompt moves
   here; `mentor-llm/prompts.rs` shrinks to `format_chat`; `PlayerLevel` moves app-side.
4. **Command + frontend plumbing** (~1.5–2h) — `generate_coaching` gains optional
   `engine_data: Option<CoachingEngineData>` (specta type); prompt built before cache check;
   CoachingPanel passes eval/PV fields; regenerate bindings.
5. **Cache key v2** (~0.5–1h) — as above.
6. **Grammar port + N_CTX 2048** (~1–2h, separable) — port teach-go's
   `generate_streaming_with_grammar`; free-text grammar constant passed from `channel.rs`.
7. **Verification** (~3–4h):
   - Unit tests: eval-swing bucket table, `uci_line_to_san`, tactic diff on a constructed
     hanging-queen blunder, prompt-cap truncation, facts-appear-in-prompt assertions.
   - Ignored real-model tests (`src-tauri/tests/llm_position_grounding.rs`): hanging-queen and
     missed-fork cases with content assertions, plus the **mechanical groundedness oracle** —
     regex all `[a-h][1-8]` squares out of the model output and assert every one appears in the
     prompt text (fixed seed makes outputs stable per prompt).
   - Eval set `src-tauri/tests/fixtures/coaching_eval.json`: 10–12 cases
     `{fen_before, player_move_uci, best_uci, pv, refutation_pv, classification, must_mention_any,
     must_not_mention}` across phases; runner asserts ≥80% and prints per-case report; record the
     old-prompt baseline first to quantify the improvement.
   - Manual playtest: real-game review, latency under ~10s worst case, streamed text references
     real squares.

### Rollout order

Steps 1→2→3 land together (no behavior change until wired), then 4+5 atomically (behavior change +
cache fix), then 6, then 7 continuously. Each step compiles and passes tests independently.

## Critical files

- `src-tauri/src/commands/llm.rs` — command rewiring, cache-check reorder, `CoachingEngineData`
- `crates/mentor-llm/src/prompts.rs` — shrink to `format_chat` (sensei-kit prep)
- `src-tauri/src/llm/mod.rs` — new `position_facts.rs` / `coach_prompt.rs` modules
- `src-tauri/src/engine/process.rs` — `engine_best_san`, persist `refutation_pv`
- `src-tauri/src/llm/cache.rs` — cache key v2
- `crates/mentor-llm/src/model.rs` — grammar-capable generation, `N_CTX` bump
