export type PuzzleCategory =
  | "tactical"
  | "positional"
  | "endgame"
  | "opening"
  | "pattern";

export type Puzzle = {
  id: string;
  fen: string;
  solutionMoves: string;
  themes: string;
  category: PuzzleCategory;
  difficulty: number;
  sourceId: string | null;
  hintsJson: string;
  explanation: string;
};

export type PuzzleState = {
  puzzle: Puzzle;
  startFen: string;
  playerColor: "white" | "black";
  legalDests: Record<string, string[]>;
  totalPlayerMoves: number;
  currentMoveIndex: number;
};

export type PuzzleMoveResult = {
  correct: boolean;
  isComplete: boolean;
  fen: string | null;
  legalDests: Record<string, string[]> | null;
  lastMove: [string, string] | null;
  currentMoveIndex: number;
  explanation: string | null;
};

export type PuzzleFilter = {
  category?: PuzzleCategory;
  minDifficulty?: number;
  maxDifficulty?: number;
  themes?: string[];
};

export type PuzzleSessionStats = {
  totalAttempts: number;
  totalSolved: number;
  averageTimeMs: number;
  currentStreak: number;
  bestStreak: number;
};
