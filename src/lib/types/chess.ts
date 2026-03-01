export type Color = "white" | "black";

export type PieceRole = "pawn" | "knight" | "bishop" | "rook" | "queen" | "king";

export type GameOutcome =
  | { checkmate: { winner: Color } }
  | { stalemate: Record<string, never> }
  | { insufficientMaterial: Record<string, never> }
  | { threefoldRepetition: Record<string, never> }
  | { fiftyMoveRule: Record<string, never> }
  | { resignation: { winner: Color } }
  | { draw: Record<string, never> };

export type Position = {
  fen: string;
  turn: Color;
  /** Legal destinations: maps "e2" → ["e3", "e4"] for chessground */
  dests: Record<string, string[]>;
  lastMove: [string, string] | null;
  isCheck: boolean;
  isGameOver: boolean;
  outcome: GameOutcome | null;
  moveNumber: number;
  sanHistory: string[];
};

export type LegalMove = {
  uci: string;
  san: string;
  from: string;
  to: string;
  promotion: PieceRole | null;
};
