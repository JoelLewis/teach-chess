export type Score =
  | { type: "cp"; value: number }
  | { type: "mate"; moves: number };

export type EngineEvaluation = {
  score: Score;
  depth: number;
  pv: string[];
  nodes: number;
  bestMove: string;
};

export type EngineMove = {
  uci: string;
  ponder: string | null;
};

export type MoveClassification =
  | "best"
  | "excellent"
  | "good"
  | "inaccuracy"
  | "mistake"
  | "blunder";

export type Side = "white" | "black";

export type GamePhase = "opening" | "middlegame" | "endgame";

export type PieceCounts = {
  pawns: number;
  knights: number;
  bishops: number;
  rooks: number;
  queens: number;
};

export type MaterialImbalance =
  | { type: "bishopPair"; side: Side }
  | { type: "exchangeUp"; side: Side }
  | { type: "exchangeDown"; side: Side }
  | { type: "queenVsPieces"; side: Side };

export type MaterialBalance = {
  white: PieceCounts;
  black: PieceCounts;
  balanceCp: number;
  imbalances: MaterialImbalance[];
};

export type SidePawnStructure = {
  isolated: string[];
  doubled: string[];
  passed: string[];
  backward: string[];
};

export type PawnChain = {
  side: Side;
  squares: string[];
};

export type PawnStructure = {
  white: SidePawnStructure;
  black: SidePawnStructure;
  chains: PawnChain[];
  openFiles: string[];
  halfOpenFilesWhite: string[];
  halfOpenFilesBlack: string[];
};

export type PieceActivityDetail = {
  square: string;
  piece: string;
  mobility: number;
  isCentralized: boolean;
  isOnRim: boolean;
};

export type SideActivity = {
  totalMobility: number;
  developedMinors: number;
  totalMinors: number;
  rookOnOpenFile: boolean;
  rookOnSeventh: boolean;
  pieces: PieceActivityDetail[];
};

export type PieceActivity = {
  white: SideActivity;
  black: SideActivity;
};

export type SideKingSafety = {
  kingSquare: string;
  pawnShieldCount: number;
  pawnShieldMax: number;
  hasCastled: boolean;
  canCastle: boolean;
  openFilesNearKing: number;
  kingZoneAttacks: number;
};

export type KingSafety = {
  white: SideKingSafety;
  black: SideKingSafety;
};

export type TacticType =
  | "pin"
  | "fork"
  | "skewer"
  | "hangingPiece"
  | "backRankThreat"
  | "discoveredAttack";

export type TacticalMotif = {
  tacticType: TacticType;
  side: Side;
  square: string;
  description: string;
};

export type PositionalTheme =
  | "knightOnRim"
  | "bishopPairAdvantage"
  | "isolatedQueenPawn"
  | "passedPawn"
  | "doubledPawns"
  | "backwardPawn"
  | "openFile"
  | "rookOnSeventh"
  | "kingSafetyCompromised"
  | "undevelopedPieces"
  | "centralControl"
  | "pawnChainTension"
  | "materialImbalance"
  | "backRankWeakness"
  | "pinnedPiece"
  | "forkAvailable"
  | "hangingMaterial";

export type CoachingContext = {
  fen: string;
  phase: GamePhase;
  material: MaterialBalance;
  pawns: PawnStructure;
  activity: PieceActivity;
  kingSafety: KingSafety;
  tactics: TacticalMotif[];
  themes: PositionalTheme[];
};

export type MoveEvaluation = {
  moveNumber: number;
  isWhite: boolean;
  fenBefore: string;
  playerMoveUci: string;
  playerMoveSan: string;
  engineBestUci: string | null;
  engineBestSan: string | null;
  evalBefore: Score | null;
  evalAfter: Score | null;
  classification: MoveClassification | null;
  depth: number;
  pv: string[];
  coachingContext: CoachingContext | null;
  coachingText: string | null;
};

/** Convert a Score to a display string */
export function formatScore(score: Score): string {
  if (score.type === "mate") {
    return score.moves > 0 ? `M${score.moves}` : `-M${Math.abs(score.moves)}`;
  }
  const value = score.value / 100;
  return value >= 0 ? `+${value.toFixed(1)}` : value.toFixed(1);
}

/** Convert a Score to a normalized 0-1 value for the eval bar (0.5 = equal) */
export function scoreToBarValue(score: Score): number {
  if (score.type === "mate") {
    return score.moves > 0 ? 1 : 0;
  }
  // Sigmoid-like scaling: ±500cp maps to roughly 0.1-0.9
  const cp = score.value;
  return 1 / (1 + Math.pow(10, -cp / 400));
}

/** Classification color mapping for UI */
export const CLASSIFICATION_COLORS: Record<MoveClassification, string> = {
  best: "#26a641",
  excellent: "#57ab5a",
  good: "#8ddb8c",
  inaccuracy: "#d29922",
  mistake: "#e5534b",
  blunder: "#da3633",
} as const;

// ─── In-Game Coaching Types ──────────────────────────────────────

export type CoachingLevel = "fullCoach" | "lightTouch" | "minimal" | "silent";

export type InGameCoachingFeedback = {
  classification: MoveClassification;
  coachingText: string;
  evalBefore: Score;
  evalAfter: Score;
  engineBestUci: string;
  coachingContext: CoachingContext | null;
  moveNumber: number;
};

export type PreMoveHintType =
  | "tacticalAlert"
  | "phaseTransition"
  | "strategicReminder"
  | "none";

export type PreMoveHint = {
  hintText: string | null;
  hintType: PreMoveHintType;
  themes: PositionalTheme[];
};

// ─── Post-Game Review Enhancement Types ──────────────────────────

export type CriticalMoment = {
  moveIndex: number;
  evalSwingCp: number;
  description: string;
  isPlayerMove: boolean;
};

export type PatternSummary = {
  totalErrors: number;
  errorThemes: [PositionalTheme, number][];
  missedTactics: [TacticType, number][];
  errorsByPhase: Record<string, number>;
  strengths: string[];
};

export type StudySuggestion = {
  topic: string;
  description: string;
  priority: number;
};

// ─── LLM Coaching Types ────────────────────────────────────────

export type PlayerLevel = "beginner" | "intermediate" | "upperIntermediate";

export type CoachingSource = "cache" | "llm" | "template";

export type CoachingResponse = {
  text: string;
  source: CoachingSource;
};

export type LlmStatus = {
  available: boolean;
  modelLoaded: boolean;
  modelId: string | null;
  mode: "llm" | "template";
};

export type ModelStatus = {
  id: string;
  displayName: string;
  downloaded: boolean;
  fileSizeMb: number;
  ramRequirementMb: number;
  systemMemoryMb: number;
};
