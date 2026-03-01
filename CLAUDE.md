# ChessMentor — Project Conventions

## Stack
- **Framework:** Tauri 2.x (Rust backend + web frontend)
- **Frontend:** Svelte 5 with runes (`$state`, `$derived`, `$effect`)
- **Styling:** Tailwind CSS 4
- **Backend:** Rust (all chess logic, engine management, database)
- **Database:** SQLite via rusqlite (bundled), migrations in `src-tauri/migrations/`
- **Chess engine:** Stockfish as Tauri sidecar binary
- **Chess logic:** shakmaty crate (legal moves, FEN, SAN, PGN)
- **License:** GPL-3.0-only (required by chessground dependency)

## Architecture Rules
- **All chess logic lives in Rust.** The frontend is display-only — no chess validation, move generation, or game state management in TypeScript.
- **IPC boundary:** All frontend-backend communication uses typed Tauri commands (`invoke()`) and events (`listen()`).
- **Typed API layer:** `src/lib/api/commands.ts` wraps all `invoke()` calls with proper TypeScript types. Never call `invoke()` directly from components.

## Svelte Patterns
- Use Svelte 5 runes: `$state`, `$derived`, `$effect` — NOT Svelte 4 stores (`writable`, `readable`).
- Reactive state files use `.svelte.ts` extension.
- No barrel files — use direct imports.

## Rust Patterns
- `thiserror` for all error types, with `Serialize` derive for IPC.
- `shakmaty` for all chess operations — do not implement custom move validation.
- Async via tokio for engine I/O.
- `tracing` for logging (not `println!` or `eprintln!`).

## Database
- SQLite with WAL mode, opened in Tauri app data directory.
- Migrations are plain SQL files in `src-tauri/migrations/`, applied on startup.
- Use parameterized queries — never string interpolation for SQL.

## Testing
- `cargo test` for Rust unit tests
- `svelte-check` for TypeScript type checking
- `npm run tauri dev` for integration testing

## Build & Run
- Dev: `npm run tauri dev`
- Build: `npm run tauri build`
- Stockfish: `./scripts/fetch-stockfish.sh` (downloads platform binary to `src-tauri/binaries/`)

## Sidecar Convention
Stockfish binaries are NOT committed to git. The sidecar naming convention is:
`stockfish-{target-triple}[.exe]` (e.g., `stockfish-x86_64-unknown-linux-gnu`)

## Code Style
- TypeScript: strict mode, ESM only, `type` over `interface`, no `any`
- Rust: follow clippy, `Result<T, E>` for errors, derive Debug/Clone/Serialize on types
- No barrel files in either language
