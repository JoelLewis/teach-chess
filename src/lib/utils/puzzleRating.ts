import type { SkillRating } from "../api/bindings";

/**
 * Minimum rated games before a category rating is trusted for comparisons.
 * Mirrors `MIN_RATED_GAMES` in `src-tauri/src/assessment/rank.rs`.
 */
export const MIN_RATED_GAMES = 3;

/** Rating gap treated as "at your level" rather than above/below. */
const COMFORT_MARGIN = 100;

/**
 * One-line comparison of a puzzle's rating against the player's Glicko-2
 * rating for the puzzle's category, e.g.
 * "Rated 1450 — above your 1300; a stretch puzzle."
 *
 * Returns null for players without a trusted rating (graceful degradation).
 */
export function puzzleRatingComparison(
  puzzleRating: number,
  skill: SkillRating | null,
): string | null {
  if (!skill || skill.gamesCount < MIN_RATED_GAMES) return null;

  const player = Math.round(skill.rating);
  const diff = puzzleRating - player;
  if (diff >= COMFORT_MARGIN) {
    return `Rated ${puzzleRating} — above your ${player}; a stretch puzzle.`;
  }
  if (diff <= -COMFORT_MARGIN) {
    return `Rated ${puzzleRating} — below your ${player}; one to review closely.`;
  }
  return `Rated ${puzzleRating} — right at your ${player}; a fair test.`;
}
