import type { SkillProfile } from "./assessment";
import type { GameRecord } from "./game";
import type { PuzzleSessionStats } from "./puzzle";

export type DashboardData = {
  skillProfile: SkillProfile;
  recentGames: GameRecord[];
  puzzleStats: PuzzleSessionStats;
  dailyRecommendation: DailyRecommendation;
  streak: SessionStreak;
};

export type DailyRecommendation = {
  text: string;
  targetActivity: string;
  targetCategory: string | null;
};

export type SessionStreak = {
  currentDays: number;
  longestDays: number;
  gamesToday: number;
  puzzlesToday: number;
};

export type AdaptivePromptType =
  | "increaseChallenge"
  | "decreaseChallenge"
  | "frustrationDetected"
  | "plateauDetected"
  | "none";

export type AdaptivePrompt = {
  promptType: AdaptivePromptType;
  message: string;
  suggestion: string;
  targetActivity: string;
  targetCategory: string | null;
};
