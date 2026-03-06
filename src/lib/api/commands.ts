import { invoke } from "@tauri-apps/api/core";
import type { Position } from "../types/chess";
import type {
  GameConfig,
  GameRecord,
  PersonalityProfile,
  OpponentMode,
  SelectedMove,
} from "../types/game";
import type {
  EngineEvaluation,
  EngineMove,
  MoveEvaluation,
  LlmStatus,
  ModelStatus,
  CoachingResponse,
  CoachingContext,
  CoachingLevel,
  InGameCoachingFeedback,
  PreMoveHint,
  CriticalMoment,
  PatternSummary,
  StudySuggestion,
  GamePhase,
} from "../types/engine";

// ─── Game Commands ───────────────────────────────────────────

export function newGame(config: GameConfig): Promise<Position> {
  return invoke<Position>("new_game", { config });
}

export function makeMove(uci: string): Promise<Position> {
  return invoke<Position>("make_move", { uci });
}

export function resign(): Promise<GameRecord> {
  return invoke<GameRecord>("resign");
}

export function saveCompletedGame(): Promise<GameRecord> {
  return invoke<GameRecord>("save_completed_game");
}

export function getPosition(): Promise<Position> {
  return invoke<Position>("get_position");
}

// ─── Engine Commands ─────────────────────────────────────────

export function startEngine(): Promise<boolean> {
  return invoke<boolean>("start_engine");
}

export function stopEngine(): Promise<void> {
  return invoke<void>("stop_engine");
}

export function getEngineMove(
  fen: string,
  depth?: number,
  elo?: number,
  skillLevel?: number,
): Promise<EngineMove> {
  return invoke<EngineMove>("get_engine_move", {
    fen,
    depth: depth ?? null,
    elo: elo ?? null,
    skillLevel: skillLevel ?? null,
  });
}

export function analyzePosition(
  fen: string,
  depth: number,
): Promise<EngineEvaluation> {
  return invoke<EngineEvaluation>("analyze_position", { fen, depth });
}

// ─── Opponent Commands ──────────────────────────────────────

export function getOpponentMove(
  fen: string,
  personality: PersonalityProfile,
  teachingMode: boolean,
  weakCategories?: string[],
  depth?: number,
): Promise<SelectedMove> {
  return invoke<SelectedMove>("get_opponent_move", {
    fen,
    depth: depth ?? null,
    personality,
    teachingMode,
    weakCategories: weakCategories ?? null,
  });
}

export function resolvePersonality(
  mode: OpponentMode,
  explicit?: PersonalityProfile,
): Promise<PersonalityProfile> {
  return invoke<PersonalityProfile>("resolve_personality", {
    mode,
    explicit: explicit ?? null,
  });
}

// ─── Player Commands ─────────────────────────────────────────

type Player = {
  id: string;
  displayName: string;
  createdAt: string;
  gamesPlayed: number;
};

export function getOrCreatePlayer(displayName: string): Promise<Player> {
  return invoke<Player>("get_or_create_player", { displayName });
}

export function updatePlayerSettings(playerId: string, settings: unknown) {
  return invoke("update_player_settings", { playerId, settings });
}

// ─── Review Commands ─────────────────────────────────────────

export function getGameReview(
  gameId: string,
  depth: number,
): Promise<MoveEvaluation[]> {
  return invoke<MoveEvaluation[]>("get_game_review", { gameId, depth });
}

export function getGameHistory(
  limit: number,
  offset: number,
): Promise<GameRecord[]> {
  return invoke<GameRecord[]>("get_game_history", { limit, offset });
}

// ─── In-Game Coaching Commands ──────────────────────────────────

export function evaluatePlayerMove(
  fenBefore: string,
  fenAfter: string,
  isPlayerWhite: boolean,
  moveNumber: number,
  coachingLevel: CoachingLevel,
): Promise<InGameCoachingFeedback> {
  return invoke<InGameCoachingFeedback>("evaluate_player_move", {
    fenBefore,
    fenAfter,
    isPlayerWhite,
    moveNumber,
    coachingLevel,
  });
}

export function analyzePreMoveHints(
  fen: string,
  previousPhase: GamePhase | null,
  coachingLevel: CoachingLevel,
  isPlayerWhite: boolean,
  opponentPersonality?: PersonalityProfile | null,
): Promise<PreMoveHint> {
  return invoke<PreMoveHint>("analyze_pre_move_hints", {
    fen,
    previousPhase,
    coachingLevel,
    isPlayerWhite,
    opponentPersonality: opponentPersonality ?? null,
  });
}

// ─── Review Enhancement Commands ────────────────────────────────

export function getCriticalMoments(
  evaluations: MoveEvaluation[],
  isPlayerWhite: boolean,
): Promise<CriticalMoment[]> {
  return invoke<CriticalMoment[]>("get_critical_moments", {
    evaluations,
    isPlayerWhite,
  });
}

export function getPatternSummary(
  evaluations: MoveEvaluation[],
  isPlayerWhite: boolean,
): Promise<PatternSummary> {
  return invoke<PatternSummary>("get_pattern_summary", {
    evaluations,
    isPlayerWhite,
  });
}

export function getStudySuggestions(
  summary: PatternSummary,
): Promise<StudySuggestion[]> {
  return invoke<StudySuggestion[]>("get_study_suggestions", { summary });
}

// ─── Puzzle Commands ────────────────────────────────────────────

import type {
  PuzzleState,
  PuzzleMoveResult,
  PuzzleFilter,
  PuzzleSessionStats,
} from "../types/puzzle";

export function loadNextPuzzle(filter: PuzzleFilter): Promise<PuzzleState> {
  return invoke<PuzzleState>("load_next_puzzle", { filter });
}

export function getPuzzleState(): Promise<PuzzleState | null> {
  return invoke<PuzzleState | null>("get_puzzle_state");
}

export function submitPuzzleMove(uci: string): Promise<PuzzleMoveResult> {
  return invoke<PuzzleMoveResult>("submit_puzzle_move", { uci });
}

export function requestPuzzleHint(): Promise<string | null> {
  return invoke<string | null>("request_puzzle_hint");
}

export function abandonPuzzle(): Promise<PuzzleMoveResult> {
  return invoke<PuzzleMoveResult>("abandon_puzzle");
}

export function savePuzzleResult(solved: boolean): Promise<void> {
  return invoke<void>("save_puzzle_result", { solved });
}

export function getPuzzleStats(): Promise<PuzzleSessionStats> {
  return invoke<PuzzleSessionStats>("get_puzzle_stats");
}

export function importPuzzlesFromCsv(
  path: string,
  minRating: number,
  maxRating: number,
): Promise<number> {
  return invoke<number>("import_puzzles_from_csv", {
    path,
    minRating,
    maxRating,
  });
}

export function getPuzzleThemes(): Promise<string[]> {
  return invoke<string[]>("get_puzzle_themes");
}

// ─── Repertoire Commands ──────────────────────────────────────────

import type {
  Opening,
  OpeningPosition,
  RepertoireEntry,
  RepertoireFilter,
  DrillState,
  DrillMoveResult,
  DrillStats,
} from "../types/repertoire";

export function getOpenings(filter: RepertoireFilter): Promise<Opening[]> {
  return invoke<Opening[]>("get_openings", { filter });
}

export function getOpeningDetail(
  openingId: string,
): Promise<[Opening, OpeningPosition[]]> {
  return invoke<[Opening, OpeningPosition[]]>("get_opening_detail", {
    openingId,
  });
}

export function getRepertoire(openingId: string): Promise<RepertoireEntry[]> {
  return invoke<RepertoireEntry[]>("get_repertoire", { openingId });
}

export function addToRepertoire(
  openingId: string,
  positionFen: string,
  moveUci: string,
  moveSan: string,
): Promise<void> {
  return invoke<void>("add_to_repertoire", {
    openingId,
    positionFen,
    moveUci,
    moveSan,
  });
}

export function removeFromRepertoire(entryId: string): Promise<void> {
  return invoke<void>("remove_from_repertoire", { entryId });
}

export function startRepertoireDrill(openingId: string): Promise<DrillState> {
  return invoke<DrillState>("start_repertoire_drill", { openingId });
}

export function submitDrillMove(uci: string): Promise<DrillMoveResult> {
  return invoke<DrillMoveResult>("submit_drill_move", { uci });
}

export function getDrillStats(): Promise<DrillStats> {
  return invoke<DrillStats>("get_drill_stats");
}

export function importOpeningsFromJson(path: string): Promise<number> {
  return invoke<number>("import_openings_from_json", { path });
}

// ─── Assessment Commands ────────────────────────────────────────

import type {
  SkillProfile,
  SkillRating,
  DifficultyTarget,
} from "../types/assessment";

export function getSkillProfile(): Promise<SkillProfile> {
  return invoke<SkillProfile>("get_skill_profile");
}

export function getSkillRating(
  category: string,
): Promise<SkillRating | null> {
  return invoke<SkillRating | null>("get_skill_rating", { category });
}

export function getDifficultyTarget(
  category: string,
): Promise<DifficultyTarget> {
  return invoke<DifficultyTarget>("get_difficulty_target", { category });
}

// ─── Dashboard Commands ──────────────────────────────────────────

import type { DashboardData, AdaptivePrompt } from "../types/dashboard";
import type { Theme } from "../types/theme";

export function getDashboardData(): Promise<DashboardData> {
  return invoke<DashboardData>("get_dashboard_data");
}

export function checkAdaptiveDifficulty(): Promise<AdaptivePrompt> {
  return invoke<AdaptivePrompt>("check_adaptive_difficulty");
}

// ─── LLM Commands ───────────────────────────────────────────────

export function getLlmStatus(): Promise<LlmStatus> {
  return invoke<LlmStatus>("get_llm_status");
}

export function downloadModel(modelId: string): Promise<void> {
  return invoke<void>("download_model", { modelId });
}

export function getAvailableModels(): Promise<ModelStatus[]> {
  return invoke<ModelStatus[]>("get_available_models");
}

// ─── Theme Commands ─────────────────────────────────────────────

export function getTheme(): Promise<Theme> {
  return invoke<Theme>("get_theme");
}

export function setTheme(theme: Theme): Promise<void> {
  return invoke<void>("set_theme", { theme });
}

export function generateCoaching(
  fen: string,
  classification: string,
  coachingContext: CoachingContext | null,
  playerMoveSan: string,
  engineBestSan: string | null,
  requestId?: string,
): Promise<CoachingResponse> {
  return invoke<CoachingResponse>("generate_coaching", {
    fen,
    classification,
    coachingContext,
    playerMoveSan,
    engineBestSan,
    requestId: requestId ?? null,
  });
}
