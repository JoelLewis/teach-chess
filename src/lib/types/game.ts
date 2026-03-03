import type { Color } from "./chess";
import type { CoachingLevel } from "./engine";

export type EngineStrength = {
  elo: number;
  skillLevel: number;
};

export type TimeControl = {
  initialSecs: number;
  incrementSecs: number;
};

// ─── Opponent Personality Types ───────────────────────────────

export type PersonalityProfile = "aggressive" | "positional" | "trappy" | "solid";

export type OpponentMode = "choose" | "surprise" | "coachPicks";

export type SelectedMove = {
  uci: string;
  personality: PersonalityProfile;
  personalityScore: number;
  teachingScore: number;
};

// ─── Game Configuration ───────────────────────────────────────

export type GameConfig = {
  playerColor: Color;
  engineStrength: EngineStrength;
  timeControl: TimeControl;
  coachingLevel: CoachingLevel;
  opponentMode: OpponentMode;
  personality: PersonalityProfile | null;
  teachingMode: boolean;
};

export type GameRecord = {
  id: string;
  playerId: string;
  pgn: string;
  result: string;
  playerColor: Color;
  engineElo: number;
  moveCount: number;
  startedAt: string;
  endedAt: string | null;
  timeControl: string;
  opponentPersonality: string | null;
  teachingMode: boolean;
};

export const ENGINE_PRESETS = {
  beginner: { elo: 1350, skillLevel: 1 } satisfies EngineStrength,
  intermediate: { elo: 1800, skillLevel: 8 } satisfies EngineStrength,
  advanced: { elo: 2200, skillLevel: 14 } satisfies EngineStrength,
  maximum: { elo: 3190, skillLevel: 20 } satisfies EngineStrength,
} as const;
