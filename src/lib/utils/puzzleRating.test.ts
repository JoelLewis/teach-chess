import { describe, expect, it } from "vitest";
import { puzzleRatingComparison } from "./puzzleRating";
import type { SkillRating } from "../api/bindings";

function skill(rating: number, gamesCount: number): SkillRating {
  return {
    id: "sr1",
    playerId: "p1",
    category: "tactical",
    rating,
    rd: 80,
    volatility: 0.06,
    gamesCount,
    lastUpdated: "",
  };
}

describe("puzzleRatingComparison", () => {
  it("returns null without a rating", () => {
    expect(puzzleRatingComparison(1450, null)).toBeNull();
  });

  it("returns null for a rating with too few games", () => {
    expect(puzzleRatingComparison(1450, skill(1300, 2))).toBeNull();
  });

  it("labels a puzzle well above the player as a stretch", () => {
    expect(puzzleRatingComparison(1450, skill(1300, 10))).toBe(
      "Rated 1450 — above your 1300; a stretch puzzle.",
    );
  });

  it("labels a puzzle well below the player as one to review", () => {
    expect(puzzleRatingComparison(1100, skill(1300, 10))).toBe(
      "Rated 1100 — below your 1300; one to review closely.",
    );
  });

  it("labels a puzzle near the player rating as fair", () => {
    expect(puzzleRatingComparison(1350, skill(1300, 10))).toBe(
      "Rated 1350 — right at your 1300; a fair test.",
    );
  });

  it("rounds fractional Glicko ratings for display", () => {
    expect(puzzleRatingComparison(1450, skill(1287.4, 10))).toBe(
      "Rated 1450 — above your 1287; a stretch puzzle.",
    );
  });

  it("treats the 100-point boundary as above", () => {
    expect(puzzleRatingComparison(1400, skill(1300, 10))).toContain("stretch");
    expect(puzzleRatingComparison(1399, skill(1300, 10))).toContain("fair");
  });
});
