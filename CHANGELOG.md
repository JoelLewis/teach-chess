# Changelog

All notable changes to ChessMentor will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-07-12

### Added

- **Position-aware coaching** — LLM coaching prompts are now grounded in verified position facts (tactical motif descriptions, a post-move tactic diff, SAN best line and refutation, bucketed evaluation swing) instead of bare theme labels, with grammar-constrained decoding to prevent format leakage.
- **Rank-calibrated feedback** — coaching and puzzle results adapt to the player's skill level (rating bands from the per-category Glicko-2 ratings), with qualitative phrasing and a puzzle-difficulty comparison. New players see unchanged feedback until they have a rated history.
- **Back/forward view navigation** — Cmd+[ / Cmd+], Alt+Left/Right, and mouse buttons 3/4, with a capped view-history stack.
- **Dev-only MCP playtest capability** — an opt-in socket (`pnpm playtest`) that lets tooling drive the app (screenshots, clicks, `window.__playtest` hooks), plus a checked-in driver script. Excluded from release builds.

### Changed

- **Local LLM coaching now runs on Gemma 4 E2B** (Apache 2.0) via llama.cpp, replacing the candle/Gemma 3 stack. Metal acceleration is enabled by default on macOS.
- **Shared `sensei-llm` crate** — the LLM inference, download, and streaming layer is now consumed from the shared sensei-kit repository rather than vendored.
- **Spaced repetition migrated from SM-2 to FSRS** (`rs-fsrs`), unifying puzzle and opening-drill scheduling; existing review state is preserved.
- Rust edition 2024; dependency alignment; dead-code cleanup.

### Fixed

- Puzzle attempt timestamps recorded the next-review date instead of the attempt time, skewing dashboard counts.
- The coaching response cache could return the wrong explanation when two different mistakes shared a position and classification.
- Draw outcomes threw a runtime error in the game-over dialog due to a wire-format mismatch (surfaced and fixed by generated IPC bindings).
- CI: the release build workflow failed at startup because signing steps referenced the `secrets` context in `if:` conditions.

## [0.1.0] - 2026-03-03

Initial release.

### Added

#### Core Gameplay (Phase 1)
- Play against Stockfish with configurable strength (1320–3190 ELO)
- Post-game review with move classification (best/excellent/good/inaccuracy/mistake/blunder)
- Game history stored in SQLite with PGN recording
- Evaluation bar with real-time engine score updates
- Interactive chessboard via chessground

#### Coaching (Phase 2)
- Heuristic position analysis: phase detection, material balance, pawn structure, piece activity, king safety, tactical patterns
- Template-based coaching feedback with 37 coaching templates across 3 tiers
- Optional local LLM coaching via candle + Gemma 2B (feature-gated with `--features llm`)
- In-game coaching at four levels: Full Coach, Light Touch, Minimal, Silent
- Pre-move strategic hints and phase transition alerts
- Post-game pattern summaries with study suggestions and critical moment detection

#### Training (Phase 3)
- Tactical puzzle mode with 114 bundled puzzles from Lichess
- Spaced repetition (SM-2) for puzzle scheduling
- Progressive 3-tier hint system for puzzles
- Opening repertoire: 40 curated openings with position tree navigation
- Repertoire drill mode with SRS-based scheduling
- Glicko-2 skill ratings per category (tactical/positional/endgame/opening/pattern)
- Adaptive difficulty: auto-targeting puzzles within rating ±100

#### Opponent Personalities (Phase 4)
- Four personality profiles: Aggressive, Positional, Trappy, Solid
- Three opponent modes: Choose, Surprise, Coach Picks
- Teaching mode: engine steers into positions that challenge weak areas
- Multi-PV weighted move selection per personality profile

#### Dashboard & Polish (Phase 4B + 5)
- Dashboard with skill radar chart, recommendation cards, streak tracking
- Adaptive difficulty prompts (frustration/plateau/increase detection)
- Two visual themes: The Study (warm parchment) and The Grid (cyberpunk neon)
- System theme auto-detection (follows OS dark/light preference)
- Theme flash prevention on page load
- Screen transition animations
- Loading spinner for engine initialization
- Parallelized game start sequence

#### Accessibility (Phase 5)
- ARIA landmarks and roles across navigation, dialogs, and interactive components
- Focus trap in game-over dialog with Escape key support
- Skip-to-content link for keyboard navigation
- Focus-visible outlines on all interactive elements
- Color contrast improvements for WCAG AA compliance
- `prefers-reduced-motion` support disabling all animations

#### Infrastructure (Phase 5)
- CI: cargo fmt check, version sync validation, LLM-feature clippy and test runs
- Cancel stale Stockfish evaluations on new requests
- LLM inference deduplication for identical prompts
