import type { MoveEvaluation } from "../api/bindings";

export type GameStats = {
  accuracyPct: number;
  bestMoves: number;
  inaccuracies: number;
  mistakes: number;
  blunders: number;
};

/** Summarize a game review into player-move stats for the summary card and LLM prompt. */
export function summarizeEvaluations(
  evaluations: MoveEvaluation[],
  isPlayerWhite: boolean,
): GameStats {
  const playerMoves = evaluations.filter((ev) => ev.isWhite === isPlayerWhite);
  const counts = { best: 0, excellent: 0, good: 0, inaccuracy: 0, mistake: 0, blunder: 0 };
  for (const ev of playerMoves) {
    if (ev.classification) {
      counts[ev.classification]++;
    }
  }
  const total = playerMoves.length;
  const accurate = counts.best + counts.excellent + counts.good;
  return {
    accuracyPct: total === 0 ? 0 : Math.round((accurate / total) * 100),
    bestMoves: counts.best,
    inaccuracies: counts.inaccuracy,
    mistakes: counts.mistake,
    blunders: counts.blunder,
  };
}
