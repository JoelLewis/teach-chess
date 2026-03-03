# Changelog

All notable changes to ChessMentor will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

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
