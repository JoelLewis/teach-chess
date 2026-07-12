// Thin wrappers around the generated tauri-specta bindings (./bindings.ts).
// Unwraps the Result envelope so callers keep plain promise semantics:
// resolved with data, rejected with the serialized backend error.
import { commands } from "./bindings";
import type {
  AdaptivePrompt,
  CoachingContext,
  CoachingEngineData,
  CoachingLevel,
  CoachingResponse,
  CriticalMoment,
  DashboardData,
  DifficultyTarget,
  DrillMoveResult,
  DrillState,
  DrillStats,
  EngineEvaluation,
  EngineMove,
  GameConfig,
  GamePhase,
  GameRecord,
  InGameCoachingFeedback,
  LlmStatus,
  ModelStatus,
  MoveEvaluation,
  Opening,
  OpeningPosition,
  OpponentMode,
  PatternSummary,
  PersonalityProfile,
  Player,
  PlayerSettings,
  Position,
  PreMoveHint,
  PuzzleFilter,
  PuzzleMoveResult,
  PuzzleSessionStats,
  PuzzleState,
  RepertoireEntry,
  RepertoireFilter,
  SelectedMove,
  SkillProfile,
  SkillRating,
  StudySuggestion,
} from "./bindings";
import type { Theme } from "../types/theme";

type CommandResult<T> = { status: "ok"; data: T } | { status: "error"; error: string };

async function unwrap<T>(result: Promise<CommandResult<T>>): Promise<T> {
  const r = await result;
  if (r.status === "error") throw r.error;
  return r.data;
}

// ─── Game Commands ───────────────────────────────────────────

export function newGame(config: GameConfig): Promise<Position> {
  return unwrap(commands.newGame(config));
}

export function makeMove(uci: string): Promise<Position> {
  return unwrap(commands.makeMove(uci));
}

export function resign(): Promise<GameRecord> {
  return unwrap(commands.resign());
}

export function saveCompletedGame(): Promise<GameRecord> {
  return unwrap(commands.saveCompletedGame());
}

export function getPosition(): Promise<Position> {
  return unwrap(commands.getPosition());
}

// ─── Engine Commands ─────────────────────────────────────────

export function startEngine(): Promise<boolean> {
  return unwrap(commands.startEngine());
}

export async function stopEngine(): Promise<void> {
  await unwrap(commands.stopEngine());
}

export function getEngineMove(
  fen: string,
  depth?: number,
  elo?: number,
  skillLevel?: number,
): Promise<EngineMove> {
  return unwrap(
    commands.getEngineMove(fen, depth ?? null, elo ?? null, skillLevel ?? null),
  );
}

export function analyzePosition(
  fen: string,
  depth: number,
): Promise<EngineEvaluation> {
  return unwrap(commands.analyzePosition(fen, depth));
}

// ─── Opponent Commands ──────────────────────────────────────

export function getOpponentMove(
  fen: string,
  personality: PersonalityProfile,
  teachingMode: boolean,
  weakCategories?: string[],
  depth?: number,
): Promise<SelectedMove> {
  return unwrap(
    commands.getOpponentMove(
      fen,
      depth ?? null,
      personality,
      teachingMode,
      weakCategories ?? null,
    ),
  );
}

export function resolvePersonality(
  mode: OpponentMode,
  explicit?: PersonalityProfile,
): Promise<PersonalityProfile> {
  return unwrap(commands.resolvePersonality(mode, explicit ?? null));
}

// ─── Player Commands ─────────────────────────────────────────

export function getOrCreatePlayer(displayName: string): Promise<Player> {
  return unwrap(commands.getOrCreatePlayer(displayName));
}

export function updatePlayerSettings(
  playerId: string,
  settings: PlayerSettings,
): Promise<Player> {
  return unwrap(commands.updatePlayerSettings(playerId, settings));
}

// ─── Review Commands ─────────────────────────────────────────

export function getGameReview(
  gameId: string,
  depth: number,
): Promise<MoveEvaluation[]> {
  return unwrap(commands.getGameReview(gameId, depth));
}

export function getGameHistory(
  limit: number,
  offset: number,
): Promise<GameRecord[]> {
  return unwrap(commands.getGameHistory(limit, offset));
}

// ─── In-Game Coaching Commands ──────────────────────────────────

export function evaluatePlayerMove(
  fenBefore: string,
  fenAfter: string,
  isPlayerWhite: boolean,
  moveNumber: number,
  coachingLevel: CoachingLevel,
): Promise<InGameCoachingFeedback> {
  return unwrap(
    commands.evaluatePlayerMove(
      fenBefore,
      fenAfter,
      isPlayerWhite,
      moveNumber,
      coachingLevel,
    ),
  );
}

export function analyzePreMoveHints(
  fen: string,
  previousPhase: GamePhase | null,
  coachingLevel: CoachingLevel,
  isPlayerWhite: boolean,
  opponentPersonality?: PersonalityProfile | null,
): Promise<PreMoveHint> {
  return unwrap(
    commands.analyzePreMoveHints(
      fen,
      previousPhase,
      coachingLevel,
      isPlayerWhite,
      opponentPersonality ?? null,
    ),
  );
}

// ─── Review Enhancement Commands ────────────────────────────────

export function getCriticalMoments(
  evaluations: MoveEvaluation[],
  isPlayerWhite: boolean,
): Promise<CriticalMoment[]> {
  return commands.getCriticalMoments(evaluations, isPlayerWhite);
}

export function getPatternSummary(
  evaluations: MoveEvaluation[],
  isPlayerWhite: boolean,
): Promise<PatternSummary> {
  return commands.getPatternSummary(evaluations, isPlayerWhite);
}

export function getStudySuggestions(
  summary: PatternSummary,
): Promise<StudySuggestion[]> {
  return commands.getStudySuggestions(summary);
}

// ─── Puzzle Commands ────────────────────────────────────────────

export function loadNextPuzzle(
  filter: Partial<PuzzleFilter>,
): Promise<PuzzleState> {
  return unwrap(
    commands.loadNextPuzzle({
      category: filter.category ?? null,
      minDifficulty: filter.minDifficulty ?? null,
      maxDifficulty: filter.maxDifficulty ?? null,
      themes: filter.themes ?? null,
    }),
  );
}

export function getPuzzleState(): Promise<PuzzleState | null> {
  return unwrap(commands.getPuzzleState());
}

export function submitPuzzleMove(uci: string): Promise<PuzzleMoveResult> {
  return unwrap(commands.submitPuzzleMove(uci));
}

export function requestPuzzleHint(): Promise<string | null> {
  return unwrap(commands.requestPuzzleHint());
}

export function abandonPuzzle(): Promise<PuzzleMoveResult> {
  return unwrap(commands.abandonPuzzle());
}

export async function savePuzzleResult(solved: boolean): Promise<void> {
  await unwrap(commands.savePuzzleResult(solved));
}

export function getPuzzleStats(): Promise<PuzzleSessionStats> {
  return unwrap(commands.getPuzzleStats());
}

export function importPuzzlesFromCsv(
  path: string,
  minRating: number,
  maxRating: number,
): Promise<number> {
  return unwrap(commands.importPuzzlesFromCsv(path, minRating, maxRating));
}

export function getPuzzleThemes(): Promise<string[]> {
  return unwrap(commands.getPuzzleThemes());
}

// ─── Repertoire Commands ──────────────────────────────────────────

export function getOpenings(
  filter: Partial<RepertoireFilter>,
): Promise<Opening[]> {
  return unwrap(
    commands.getOpenings({
      color: filter.color ?? null,
      ecoPrefix: filter.ecoPrefix ?? null,
      minDifficulty: filter.minDifficulty ?? null,
      maxDifficulty: filter.maxDifficulty ?? null,
    }),
  );
}

export function getOpeningDetail(
  openingId: string,
): Promise<[Opening, OpeningPosition[]]> {
  return unwrap(commands.getOpeningDetail(openingId));
}

export function getRepertoire(openingId: string): Promise<RepertoireEntry[]> {
  return unwrap(commands.getRepertoire(openingId));
}

export async function addToRepertoire(
  openingId: string,
  positionFen: string,
  moveUci: string,
  moveSan: string,
): Promise<void> {
  await unwrap(
    commands.addToRepertoire(openingId, positionFen, moveUci, moveSan),
  );
}

export async function removeFromRepertoire(entryId: string): Promise<void> {
  await unwrap(commands.removeFromRepertoire(entryId));
}

export function startRepertoireDrill(openingId: string): Promise<DrillState> {
  return unwrap(commands.startRepertoireDrill(openingId));
}

export function submitDrillMove(uci: string): Promise<DrillMoveResult> {
  return unwrap(commands.submitDrillMove(uci));
}

export function getDrillStats(): Promise<DrillStats> {
  return unwrap(commands.getDrillStats());
}

export function importOpeningsFromJson(path: string): Promise<number> {
  return unwrap(commands.importOpeningsFromJson(path));
}

// ─── Assessment Commands ────────────────────────────────────────

export function getSkillProfile(): Promise<SkillProfile> {
  return unwrap(commands.getSkillProfile());
}

export function getSkillRating(category: string): Promise<SkillRating | null> {
  return unwrap(commands.getSkillRating(category));
}

export function getDifficultyTarget(
  category: string,
): Promise<DifficultyTarget> {
  return unwrap(commands.getDifficultyTarget(category));
}

// ─── Dashboard Commands ──────────────────────────────────────────

export function getDashboardData(): Promise<DashboardData> {
  return unwrap(commands.getDashboardData());
}

export function checkAdaptiveDifficulty(): Promise<AdaptivePrompt> {
  return unwrap(commands.checkAdaptiveDifficulty());
}

// ─── LLM Commands ───────────────────────────────────────────────

export function getLlmStatus(): Promise<LlmStatus> {
  return unwrap(commands.getLlmStatus());
}

export async function downloadModel(modelId: string): Promise<void> {
  await unwrap(commands.downloadModel(modelId));
}

export function getAvailableModels(): Promise<ModelStatus[]> {
  return unwrap(commands.getAvailableModels());
}

// ─── Theme Commands ─────────────────────────────────────────────

export async function getTheme(): Promise<Theme> {
  return (await unwrap(commands.getTheme())) as Theme;
}

export async function setTheme(theme: Theme): Promise<void> {
  await unwrap(commands.setTheme(theme));
}

export function generateGameSummary(params: {
  result: string;
  outcomeType: string;
  moveCount: number;
  accuracyPct: number;
  bestMoves: number;
  blunders: number;
  mistakes: number;
  inaccuracies: number;
}): Promise<string> {
  return unwrap(
    commands.generateGameSummary(
      params.result,
      params.outcomeType,
      params.moveCount,
      params.accuracyPct,
      params.bestMoves,
      params.blunders,
      params.mistakes,
      params.inaccuracies,
    ),
  );
}

export function generateCoaching(
  fen: string,
  classification: string,
  coachingContext: CoachingContext | null,
  playerMoveSan: string,
  engineBestSan: string | null,
  engineData: CoachingEngineData | null,
  requestId?: string,
): Promise<CoachingResponse> {
  return unwrap(
    commands.generateCoaching(
      fen,
      classification,
      coachingContext,
      playerMoveSan,
      engineBestSan,
      engineData,
      requestId ?? null,
    ),
  );
}
