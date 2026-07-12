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
- **Local LLM coaching** — Gemma 4 E2B inference via llama.cpp with token streaming (no cloud API)

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
pnpm install

# Download Stockfish sidecar binary for your platform
./scripts/fetch-stockfish.sh    # Linux / macOS / Git Bash on Windows

# Start the development server
pnpm run tauri dev
```

## Build

```bash
# macOS release with the LLM model bundled into the .app (recommended)
pnpm run build:mac

# Bare build (no bundled model — coaching falls back to in-app download)
pnpm run tauri build
```

Produces platform-specific installers in `target/release/bundle/`. `build:mac` runs `scripts/fetch-model.sh` first so the Gemma 4 E2B GGUF (~3.1 GB) lands in `src-tauri/models/`, which `tauri.conf.json` bundles as app resources — the DMG grows by roughly 3 GB.

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

The `llm` feature is on by default. The model is **Gemma 4 E2B (Q4_K_M GGUF)** running locally via [llama.cpp](https://github.com/ggml-org/llama.cpp) (the `mentor-llm` crate in `crates/`) — either bundled with the app (see Build) or downloaded on first use from Settings > Model Manager. To fetch it for development:

```bash
./scripts/fetch-model.sh    # ~3.1 GB GGUF into src-tauri/models/
pnpm run tauri dev
```

### Compute Device

On macOS the default inference device is **Metal** (llama.cpp's Metal backend is always compiled in and all layers are offloaded to the GPU). Elsewhere the default is CPU.

| Configuration | Device | Notes |
|---|---|---|
| default (macOS) | Apple GPU (Metal) | compiled in on macOS; all layers offloaded |
| default (Linux/Windows) | CPU | llama.cpp's optimized CPU backend |
| `llm-cuda` feature | NVIDIA GPU | [CUDA Toolkit](https://developer.nvidia.com/cuda-downloads) with `nvcc` on PATH |
| `CHESS_MENTOR_DEVICE=cpu` | CPU | force CPU everywhere |

The active device is shown in Settings > Model Manager.

### Token Streaming

When an LLM model is loaded, coaching text streams progressively in the review panel as tokens are generated, instead of waiting for the full response. Cache hits and template responses still appear instantly.

### Model Management

If the model isn't bundled, it can be downloaded on first use via Settings > Model Manager (the GGUF comes from `unsloth/gemma-4-E2B-it-GGUF`; the tokenizer is embedded in the GGUF). When no model is available, the app falls back to template-based coaching text.

## License

GPL-3.0-only — see [LICENSE](LICENSE).

This project uses [chessground](https://github.com/lichess-org/chessground) (GPL-3.0-or-later) for the interactive board UI, which requires the entire application to be distributed under GPL-3.0. See [NOTICE](NOTICE) for full third-party attribution.
