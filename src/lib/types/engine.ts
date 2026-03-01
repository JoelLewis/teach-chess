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
