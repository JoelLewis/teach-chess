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
- **Local LLM coaching** — Gemma 3 1B inference via candle with token streaming (no cloud API)

## Screenshot

<!-- TODO: Add screenshot -->

## Prerequisites

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://rustup.rs/) (stable toolchain)
- System dependencies for Tauri 2:
  - **Linux:** `libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`
  - **macOS:** Xcode command line tools
  - **Windows:** [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with "Desktop development with C++" workload, plus [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 11)

## Quick Start

```bash
# Install frontend dependencies
npm install

# Download Stockfish sidecar binary for your platform
./scripts/fetch-stockfish.sh    # Linux / macOS / Git Bash on Windows

# Start the development server
npm run tauri dev
```

## Build

```bash
# macOS release with the LLM model bundled into the .app (recommended)
npm run build:mac

# Bare build (no bundled model — coaching falls back to in-app download)
npm run tauri build
```

Produces platform-specific installers in `src-tauri/target/release/bundle/`. `build:mac` runs `scripts/fetch-model.sh` first so the Gemma 3 1B GGUF (~770 MB) and tokenizer land in `src-tauri/models/`, which `tauri.conf.json` bundles as app resources — the DMG grows by roughly 0.8 GB.

> **Note:** bundling the model means redistributing Gemma weights, which are subject to [Google's Gemma Terms of Use](https://ai.google.dev/gemma/terms). Include the Gemma notice in release distributions.

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for a detailed overview of the system design, module structure, and data flow.

**Key architectural decisions:**
- All chess logic lives in Rust (frontend is display-only)
- IPC via typed Tauri commands and events
- Svelte 5 with runes for reactive UI
- SQLite for game history, puzzle SRS, and skill ratings
- Stockfish runs as a Tauri sidecar process

## LLM Coaching

The `llm` feature is on by default. The model is **Gemma 3 1B (Q4_K_M GGUF)** running locally via candle — either bundled with the app (see Build) or downloaded on first use from Settings > Model Manager. To fetch it for development:

```bash
./scripts/fetch-model.sh    # ~770 MB GGUF + ~33 MB tokenizer into src-tauri/models/
npm run tauri dev
```

### Compute Device

The default inference device is **CPU**, which comfortably meets the coaching latency targets for Gemma 3 1B (a few seconds per response on Apple Silicon).

| Configuration | Device | Notes |
|---|---|---|
| default | CPU | recommended on macOS |
| `CHESS_MENTOR_DEVICE=metal` | Apple GPU (Metal) | compiled in on macOS, but **opt-in**: candle 0.9's quantized-gemma3 Metal path measured ~250x slower than CPU on an 8 GB M3 (the 262k-vocab output projection thrashes); may be worth trying on 16 GB+ machines |
| `llm-cuda` feature | NVIDIA GPU → CPU fallback | [CUDA Toolkit](https://developer.nvidia.com/cuda-downloads) with `nvcc` on PATH |
| `CHESS_MENTOR_DEVICE=cpu` | CPU | force CPU everywhere |

The active device is shown in Settings > Model Manager.

### Token Streaming

When an LLM model is loaded, coaching text streams progressively in the review panel as tokens are generated, instead of waiting for the full response. Cache hits and template responses still appear instantly.

### Model Management

If the model isn't bundled, it can be downloaded on first use via Settings > Model Manager (the GGUF comes from `unsloth/gemma-3-1b-it-GGUF`, the tokenizer from `unsloth/gemma-3-1b-it` — GGUF-only repos don't ship `tokenizer.json`). When no model is available, the app falls back to template-based coaching text.

## License

GPL-3.0-only — see [LICENSE](LICENSE).

This project uses [chessground](https://github.com/lichess-org/chessground) (GPL-3.0-or-later) for the interactive board UI, which requires the entire application to be distributed under GPL-3.0. See [NOTICE](NOTICE) for full third-party attribution.
