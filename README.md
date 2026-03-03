# ChessMentor

A desktop chess coach that combines Stockfish analysis with heuristic coaching, tactical puzzles, opening training, and adaptive difficulty — all running locally with no cloud dependency.

## Features

- **Play vs Stockfish** with configurable strength (1320–3190 ELO) and four opponent personality styles
- **Post-game review** with move classification (best/excellent/good/inaccuracy/mistake/blunder), critical moment detection, and pattern summaries
- **In-game coaching** — real-time feedback at four coaching levels, from full coach to silent
- **Tactical puzzles** with spaced repetition (SM-2) and 114 bundled puzzles
- **Opening repertoire** — study openings, build a repertoire, drill with SRS
- **Adaptive difficulty** — Glicko-2 skill ratings per category, auto-adjusting puzzle difficulty
- **Opponent personalities** — aggressive, positional, trappy, solid — with teaching mode targeting your weak areas
- **Dashboard** — skill radar, streaks, recommendations, adaptive difficulty prompts
- **Two themes** — The Study (warm parchment) and The Grid (cyberpunk neon), plus system auto-detection
- **Optional LLM coaching** — local Gemma 2B inference via candle (feature-gated, no cloud API)

## Screenshot

<!-- TODO: Add screenshot -->

## Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) (stable toolchain)
- System dependencies for Tauri 2:
  - **Linux:** `libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
  - **macOS:** Xcode command line tools

## Quick Start

```bash
# Install frontend dependencies
npm install

# Download Stockfish sidecar binary for your platform
./scripts/fetch-stockfish.sh

# Start the development server
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

Produces platform-specific installers in `src-tauri/target/release/bundle/`.

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for a detailed overview of the system design, module structure, and data flow.

**Key architectural decisions:**
- All chess logic lives in Rust (frontend is display-only)
- IPC via typed Tauri commands and events
- Svelte 5 with runes for reactive UI
- SQLite for game history, puzzle SRS, and skill ratings
- Stockfish runs as a Tauri sidecar process

## LLM Coaching (Optional)

To enable local LLM-powered coaching feedback:

```bash
cargo build --features llm
```

This adds ~200MB to the binary for candle inference. Models are downloaded on first use via the Settings > Model Manager page. Without the `llm` feature, the app uses template-based coaching text (no quality difference for most users).

## License

GPL-3.0-only — see [LICENSE](LICENSE).

This project uses [chessground](https://github.com/lichess-org/chessground) (GPL-3.0-or-later) for the interactive board UI, which requires the entire application to be distributed under GPL-3.0. See [NOTICE](NOTICE) for full third-party attribution.
