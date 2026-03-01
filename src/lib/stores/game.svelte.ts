import type { Position, Color } from "../types/chess";
import type { GameConfig, GameRecord } from "../types/game";
import type { Score } from "../types/engine";

export type GamePhase = "idle" | "configuring" | "playing" | "game-over" | "reviewing";

class GameStore {
  phase = $state<GamePhase>("idle");
  position = $state<Position | null>(null);
  config = $state<GameConfig | null>(null);
  engineThinking = $state(false);
  currentScore = $state<Score | null>(null);
  lastGameRecord = $state<GameRecord | null>(null);

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
  }
}

export const gameStore = new GameStore();
