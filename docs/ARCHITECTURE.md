# ChessMentor Architecture

## System Overview

ChessMentor is a Tauri 2 desktop application with a Rust backend and Svelte 5 frontend. All chess logic, engine management, database access, and coaching intelligence run in Rust. The frontend is purely a display layer that communicates with the backend via typed Tauri IPC commands and events.

```
┌──────────────────────────────────────────────┐
│                 Svelte 5 Frontend             │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │  Stores   │ │Components│ │  API     │     │
│  │ (runes)  │ │ (views)  │ │ (invoke) │     │
│  └──────────┘ └──────────┘ └────┬─────┘     │
│                                  │ IPC        │
├──────────────────────────────────┼────────────┤
│                 Rust Backend     │            │
│  ┌──────────┐ ┌──────────┐ ┌───┴──────┐     │
│  │  Engine   │ │ Database │ │ Commands │     │
│  │(Stockfish)│ │ (SQLite) │ │ (Tauri)  │     │
│  └──────────┘ └──────────┘ └──────────┘     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │Heuristics│ │ Coaching │ │  LLM     │     │
│  │(analysis)│ │(templates)│ │ (candle) │     │
│  └──────────┘ └──────────┘ └──────────┘     │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐     │
│  │  Puzzle   │ │Repertoire│ │Assessment│     │
│  │  (SRS)   │ │ (drills) │ │(Glicko-2)│     │
│  └──────────┘ └──────────┘ └──────────┘     │
└──────────────────────────────────────────────┘
```

## Rust Module Tree

```
src-tauri/src/
├── main.rs              # Tauri app bootstrap
├── lib.rs               # Module declarations
├── error.rs             # AppError enum (thiserror)
├── config/              # App config (theme, audio) — TOML persistence
├── commands/            # Tauri command handlers (47 total)
│   ├── engine.rs        # start/stop/getMove/analyze
│   ├── game.rs          # newGame/makeMove/resign/save
│   ├── review.rs        # reviewGame/criticalMoments/patterns
│   ├── coaching.rs      # evaluatePlayerMove/preMovehints
│   ├── heuristics.rs    # analyzeHeuristics
│   ├── player.rs        # getOrCreatePlayer
│   ├── puzzle.rs        # loadNextPuzzle/submitMove/hints/stats
│   ├── repertoire.rs    # openings/repertoire/drills
│   ├── assessment.rs    # skillProfile/ratings/difficulty
│   ├── opponent.rs      # getOpponentMove/resolvePersonality
│   ├── dashboard.rs     # getDashboardData/checkAdaptive
│   ├── llm.rs           # modelStatus/download/generateCoaching
│   └── theme.rs         # get/set theme
├── engine/              # Stockfish UCI interface
│   ├── process.rs       # EngineProcess — sidecar lifecycle + UCI I/O
│   ├── uci.rs           # UCI protocol parsing (info, bestmove, options)
│   └── eval.rs          # Move classification + critical moment detection
├── game/                # Chess game state
│   ├── state.rs         # GameState managed state
│   └── moves.rs         # Move application via shakmaty
├── heuristics/          # Position analysis (pure, no async)
│   ├── phase.rs         # Game phase detection (opening/middle/endgame)
│   ├── material.rs      # Material balance + imbalances
│   ├── pawns.rs         # Pawn structure (doubled/isolated/passed/chains)
│   ├── activity.rs      # Piece activity scoring
│   ├── king_safety.rs   # King safety evaluation
│   └── tactics.rs       # Tactical pattern detection (forks/pins/skewers)
├── coaching/            # Coaching text generation
│   ├── mod.rs           # generate_coaching_text + pattern summaries
│   └── templates.rs     # 37 tiered coaching templates
├── llm/                 # Local LLM inference (feature-gated: "llm")
│   ├── candle_backend.rs # GGUF model loading + token generation
│   ├── channel.rs       # Bounded inference channel with deduplication
│   ├── model_manager.rs # Model download + management
│   ├── prompts.rs       # Coaching prompt construction
│   ├── cache.rs         # DB-backed coaching cache
│   └── player_level.rs  # Derive player level from game stats
├── puzzle/              # Tactical puzzle system
│   ├── session.rs       # PuzzleSessionState — solve flow
│   ├── srs.rs           # SM-2 spaced repetition algorithm
│   └── importer.rs      # Lichess CSV import
├── repertoire/          # Opening repertoire
│   ├── session.rs       # RepertoireSessionState — drill flow
│   └── importer.rs      # JSON opening import + position tree builder
├── opponent/            # Opponent personality system
│   ├── personality.rs   # 4 personality profiles + move preference weights
│   ├── selector.rs      # Weighted multi-PV move selection
│   └── teaching.rs      # Weak-category targeting for teaching mode
├── assessment/          # Player assessment
│   ├── glicko2.rs       # Simplified Glicko-2 rating algorithm
│   ├── adaptive.rs      # Adaptive difficulty trigger detection
│   └── mod.rs           # Difficulty targeting
├── models/              # Shared data types (Serialize + Deserialize)
└── db/                  # Database access layer
    ├── connection.rs    # SQLite connection + migration runner
    ├── game.rs          # Game CRUD
    ├── player.rs        # Player CRUD
    ├── puzzle.rs        # Puzzle + attempt queries
    ├── repertoire.rs    # Opening + repertoire + drill queries
    ├── assessment.rs    # Skill rating queries
    └── coaching_cache.rs # LLM coaching cache queries
```

## Svelte Component Tree

```
src/
├── App.svelte                    # App shell — routing, game lifecycle
├── lib/
│   ├── api/commands.ts           # Typed wrappers for all 47 Tauri commands
│   ├── api/events.ts             # Tauri event listeners
│   ├── stores/                   # Svelte 5 rune-based state (.svelte.ts)
│   │   ├── game.svelte.ts        # Game state (position, config, phase)
│   │   ├── player.svelte.ts      # Player identity
│   │   ├── theme.svelte.ts       # Theme state + system detection
│   │   ├── puzzle.svelte.ts      # Puzzle session state
│   │   ├── repertoire.svelte.ts  # Repertoire/drill state
│   │   ├── assessment.svelte.ts  # Skill profile cache
│   │   └── error.svelte.ts       # Error toast state
│   ├── types/                    # TypeScript type definitions
│   └── components/
│       ├── layout/               # Sidebar, Header, ErrorToast
│       ├── dashboard/            # Dashboard, SkillRadar, Recommendations
│       ├── board/                # Chessboard, EvalBar, MoveList
│       ├── game/                 # GameConfig, PlayScreen, GameOverDialog
│       ├── review/               # ReviewScreen, CoachingPanel, PatternSummary
│       ├── problems/             # ProblemScreen, PuzzleHints, PuzzleFilter
│       ├── openings/             # OpeningsScreen, Library, Drill
│       ├── assessment/           # SkillProfilePanel
│       ├── settings/             # SettingsPage, ThemeSwitcher, ModelManager
│       └── ui/                   # LoadingSpinner (reusable primitives)
```

## Data Flow: Game Loop

```
User clicks "Start Game"
  → App.svelte: startGame(config)
    → Promise.all([startEngine(), resolvePersonality() + getSkillProfile()])
    → newGame(config) → Rust creates GameState, returns Position
    → gameStore.phase = "playing", navigate to PlayScreen

User makes a move (drag/click on chessground)
  → PlayScreen: handlePlayerMove(from, to)
    → makeMove(uci) → Rust validates with shakmaty, updates GameState
    → evaluatePlayerMove() → depth-10 engine eval + heuristic analysis
    → requestEngineMove() → getOpponentMove() or getEngineMove()
    → makeMove(engineUci) → opponent's move applied
    → analyzePreMoveHints() → strategic hints for next turn

Game ends (checkmate/resignation/draw)
  → saveCompletedGame() → Rust persists to SQLite with PGN
  → GameOverDialog shown (focus-trapped)
  → User chooses "Review" or "New Game"
```

## Data Flow: Coaching Pipeline

```
Position + Move
  → Heuristic Analysis (pure Rust, <1ms)
    → GamePhase, MaterialBalance, PawnStructure, PieceActivity,
      KingSafety, TacticalPatterns → CoachingContext

  → Engine Evaluation (Stockfish, ~200ms at depth 10)
    → score_before, score_after → MoveClassification

  → Template Selection (deterministic)
    → match (classification, context.themes) → coaching_text

  → LLM Enhancement (optional, ~2-5s)
    → cache check → prompt construction → candle inference → cache store
    → Replaces template text with LLM-generated coaching
```

## Database Schema

Seven migrations in `src-tauri/migrations/`:

| Table | Purpose |
|-------|---------|
| `player` | Player profiles (id, display_name, created_at) |
| `game` | Game records (PGN, result, config, timestamps) |
| `move_annotation` | Per-move analysis (classification, eval, coaching_text) |
| `coaching_cache` | LLM response cache (keyed by FEN + context hash) |
| `puzzle` | Tactical puzzles (FEN, moves, rating, themes, hints) |
| `puzzle_attempt` | Puzzle solve history + SRS fields (ease, interval, next_review) |
| `opening` | Opening definitions (name, ECO, moves, description) |
| `opening_position` | Position tree nodes (FEN, SAN, parent links) |
| `repertoire_entry` | User's repertoire selections (opening + position refs) |
| `repertoire_drill_attempt` | Drill history + SRS fields |
| `skill_rating` | Glicko-2 ratings per category per player |

SQLite with WAL mode. All queries use parameterized statements. Migrations run automatically on startup.
