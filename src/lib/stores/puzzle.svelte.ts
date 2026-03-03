import type {
  PuzzleState,
  PuzzleMoveResult,
  PuzzleFilter,
  PuzzleSessionStats,
} from "../types/puzzle";

export type PuzzlePhase =
  | "idle"
  | "loading"
  | "solving"
  | "correct"
  | "incorrect"
  | "complete"
  | "abandoned";

class PuzzleStore {
  phase = $state<PuzzlePhase>("idle");
  currentPuzzle = $state<PuzzleState | null>(null);
  hintsRevealed = $state<string[]>([]);
  sessionStats = $state<PuzzleSessionStats | null>(null);
  lastMoveResult = $state<PuzzleMoveResult | null>(null);
  filter = $state<PuzzleFilter>({
    category: "tactical",
    minDifficulty: 400,
    maxDifficulty: 1600,
  });
  explanation = $state<string | null>(null);

  get currentFen(): string | null {
    if (this.lastMoveResult?.fen) return this.lastMoveResult.fen;
    return this.currentPuzzle?.startFen ?? null;
  }

  get legalDests(): Record<string, string[]> {
    if (this.lastMoveResult?.legalDests) return this.lastMoveResult.legalDests;
    return this.currentPuzzle?.legalDests ?? {};
  }

  get playerColor(): "white" | "black" {
    return this.currentPuzzle?.playerColor ?? "white";
  }

  get themes(): string[] {
    const raw = this.currentPuzzle?.puzzle.themes ?? "";
    return raw
      .split(",")
      .map((t) => t.trim())
      .filter((t) => t.length > 0);
  }

  get difficulty(): number {
    return this.currentPuzzle?.puzzle.difficulty ?? 0;
  }

  get solveRate(): string {
    if (!this.sessionStats || this.sessionStats.totalAttempts === 0) return "—";
    const pct = Math.round(
      (this.sessionStats.totalSolved / this.sessionStats.totalAttempts) * 100,
    );
    return `${pct}%`;
  }

  reset() {
    this.phase = "idle";
    this.currentPuzzle = null;
    this.hintsRevealed = [];
    this.lastMoveResult = null;
    this.explanation = null;
  }
}

export const puzzleStore = new PuzzleStore();
