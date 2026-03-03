export type SkillRating = {
  id: string;
  playerId: string;
  category: string;
  rating: number;
  rd: number;
  volatility: number;
  gamesCount: number;
  lastUpdated: string;
};

export type SkillProfile = {
  ratings: SkillRating[];
  overallRating: number;
  strongestCategory: string | null;
  weakestCategory: string | null;
};

export type DifficultyTarget = {
  targetRating: number;
  minRating: number;
  maxRating: number;
};
