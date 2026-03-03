# Contributing to ChessMentor

Thank you for your interest in contributing! This guide covers the development workflow and conventions.

## Development Environment

### Prerequisites

- Node.js 20+
- Rust stable toolchain with `rustfmt` and `clippy` components
- Platform-specific Tauri dependencies (see README)

### Setup

```bash
git clone <repo-url>
cd chess-mentor
npm install
./scripts/fetch-stockfish.sh
npm run tauri dev
```

## Code Style

### Rust

- `cargo fmt --all` before committing
- `cargo clippy --all-targets -- -D warnings` must pass
- `thiserror` for error types, `tracing` for logging
- All chess logic uses the `shakmaty` crate

### TypeScript / Svelte

- Svelte 5 runes (`$state`, `$derived`, `$effect`) — not Svelte 4 stores
- `npx svelte-check --tsconfig ./tsconfig.json` must pass with 0 errors
- Strict TypeScript — no `any`
- ESM imports only

### General

- No barrel files (index.ts re-exports)
- Direct imports to the source module
- Prefer editing existing files over creating new ones

## Feature Flags

The `llm` feature gate enables local LLM inference via candle:

```bash
cargo check --features llm
cargo test --features llm
cargo clippy --features llm --all-targets -- -D warnings
```

Always verify your changes compile and pass tests with **both** default features and `--features llm`.

## Testing

Before submitting a PR, ensure:

```bash
# Rust
cargo fmt --all -- --check
cargo check && cargo check --features llm
cargo test && cargo test --features llm
cargo clippy --all-targets -- -D warnings

# Frontend
npm run build
npx svelte-check --tsconfig ./tsconfig.json
```

## Pull Request Process

1. Create a feature branch from `main`
2. Make focused, well-scoped changes
3. Ensure all checks pass (see Testing above)
4. Write a clear PR description explaining what and why
5. Reference any related issues

## Content Contributions

### Adding Puzzles

Puzzles are imported from Lichess-format CSV files. The bundled starter set is at `src-tauri/data/puzzles_starter.csv`.

CSV format: `PuzzleId,FEN,Moves,Rating,RatingDeviation,Popularity,NbPlays,Themes,GameUrl,OpeningTags`

### Adding Openings

Openings are imported from JSON files. The bundled starter set is at `src-tauri/data/openings.json`.

Each opening has: `name`, `eco` (code), `color`, `moves` (UCI), and `description`.

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only, consistent with the project license.
