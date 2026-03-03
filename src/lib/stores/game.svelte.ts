import type { Position, Color } from "../types/chess";
import type { GameConfig, GameRecord, PersonalityProfile } from "../types/game";
import type {
  Score,
  InGameCoachingFeedback,
  PreMoveHint,
  GamePhase as ChessPhase,
} from "../types/engine";

export type GamePhase = "idle" | "configuring" | "playing" | "game-over" | "reviewing";

class GameStore {
  phase = $state<GamePhase>("idle");
  position = $state<Position | null>(null);
  config = $state<GameConfig | null>(null);
  engineThinking = $state(false);
  currentScore = $state<Score | null>(null);
  lastGameRecord = $state<GameRecord | null>(null);

  // In-game coaching state
  latestCoaching = $state<InGameCoachingFeedback | null>(null);
  preMoveHint = $state<PreMoveHint | null>(null);
  currentChessPhase = $state<ChessPhase | null>(null);
  coachingHistory = $state<InGameCoachingFeedback[]>([]);

  // Opponent personality state (resolved once at game start)
  resolvedPersonality = $state<PersonalityProfile | null>(null);

  // Weak categories for teaching mode (loaded from skill profile at game start)
  weakCategories = $state<string[]>([]);

  get playerColor(): Color | null {
    return this.config?.playerColor ?? null;
  }

  get isPlayerTurn(): boolean {
    if (!this.position || !this.config) return false;
    return this.position.turn === this.config.playerColor;
  }

  reset() {
    this.phase = "idle";
    this.position = null;
    this.config = null;
    this.engineThinking = false;
    this.currentScore = null;
    this.lastGameRecord = null;
    this.latestCoaching = null;
    this.preMoveHint = null;
    this.currentChessPhase = null;
    this.coachingHistory = [];
    this.resolvedPersonality = null;
    this.weakCategories = [];
  }
}

export const gameStore = new GameStore();
