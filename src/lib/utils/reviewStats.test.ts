import { describe, expect, it } from "vitest";
import { summarizeEvaluations } from "./reviewStats";
import type { MoveEvaluation } from "../types/engine";

function evaluation(
  isWhite: boolean,
  classification: MoveEvaluation["classification"],
): MoveEvaluation {
  return {
    moveNumber: 1,
    isWhite,
    fenBefore: "",
    playerMoveUci: "e2e4",
    playerMoveSan: "e4",
    engineBestUci: null,
    engineBestSan: null,
    evalBefore: null,
    evalAfter: null,
    classification,
    depth: 10,
    pv: [],
    coachingContext: null,
    coachingText: null,
  };
}

describe("summarizeEvaluations", () => {
  it("counts only the player's moves", () => {
    const evaluations = [
      evaluation(true, "best"),
      evaluation(false, "blunder"),
      evaluation(true, "blunder"),
      evaluation(false, "best"),
    ];

    const white = summarizeEvaluations(evaluations, true);
    expect(white.bestMoves).toBe(1);
    expect(white.blunders).toBe(1);
    expect(white.accuracyPct).toBe(50);

    const black = summarizeEvaluations(evaluations, false);
    expect(black.bestMoves).toBe(1);
    expect(black.blunders).toBe(1);
  });

  it("computes accuracy from best/excellent/good moves", () => {
    const evaluations = [
      evaluation(true, "best"),
      evaluation(true, "excellent"),
      evaluation(true, "good"),
      evaluation(true, "inaccuracy"),
      evaluation(true, "mistake"),
      evaluation(true, "blunder"),
    ];

    const stats = summarizeEvaluations(evaluations, true);
    expect(stats.accuracyPct).toBe(50);
    expect(stats.inaccuracies).toBe(1);
    expect(stats.mistakes).toBe(1);
    expect(stats.blunders).toBe(1);
  });

  it("returns zeros for an empty review", () => {
    const stats = summarizeEvaluations([], true);
    expect(stats.accuracyPct).toBe(0);
    expect(stats.bestMoves).toBe(0);
  });

  it("ignores moves without a classification", () => {
    const stats = summarizeEvaluations([evaluation(true, null)], true);
    expect(stats.accuracyPct).toBe(0);
  });
});
