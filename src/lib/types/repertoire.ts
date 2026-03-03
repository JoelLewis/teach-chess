export type Opening = {
  id: string;
  name: string;
  eco: string;
  color: "white" | "black";
  description: string;
  moves: string;
  themes: string;
  difficulty: number;
};

export type OpeningPosition = {
  id: string;
  openingId: string;
  fen: string;
  moveIndex: number;
  parentFen: string | null;
  moveUci: string;
  moveSan: string;
};

export type RepertoireEntry = {
  id: string;
  playerId: string;
  openingId: string;
  positionFen: string;
  moveUci: string;
  moveSan: string;
  notes: string;
};

export type DrillState = {
  opening: Opening;
  currentEntry: RepertoireEntry;
  fen: string;
  opponentMove: string | null;
  playerColor: "white" | "black";
  legalDests: Record<string, string[]>;
  entriesTotal: number;
  entriesRemaining: number;
};

export type DrillMoveResult = {
  correct: boolean;
  correctMove: string | null;
  isComplete: boolean;
  entriesRemaining: number;
  explanation: string | null;
};

export type RepertoireFilter = {
  color?: string;
  ecoPrefix?: string;
  minDifficulty?: number;
  maxDifficulty?: number;
};

export type DrillStats = {
  totalDrills: number;
  totalCorrect: number;
  currentStreak: number;
  openingsDrilled: number;
};
