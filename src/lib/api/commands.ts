import { invoke } from "@tauri-apps/api/core";
import type { Position } from "../types/chess";
import type { GameConfig, GameRecord } from "../types/game";
import type { EngineEvaluation, EngineMove, MoveEvaluation } from "../types/engine";

// ─── Game Commands ───────────────────────────────────────────

export function newGame(config: GameConfig): Promise<Position> {
  return invoke<Position>("new_game", { config });
}

export function makeMove(uci: string): Promise<Position> {
  return invoke<Position>("make_move", { uci });
}

export function resign(): Promise<GameRecord> {
  return invoke<GameRecord>("resign");
}

export function getPosition(): Promise<Position> {
  return invoke<Position>("get_position");
}

// ─── Engine Commands ─────────────────────────────────────────

export function startEngine(): Promise<boolean> {
  return invoke<boolean>("start_engine");
}

export function stopEngine(): Promise<void> {
  return invoke<void>("stop_engine");
}

export function getEngineMove(
  fen: string,
  depth?: number,
  elo?: number,
  skillLevel?: number,
): Promise<EngineMove> {
  return invoke<EngineMove>("get_engine_move", {
    fen,
    depth: depth ?? null,
    elo: elo ?? null,
    skillLevel: skillLevel ?? null,
  });
}

export function analyzePosition(
  fen: string,
  depth: number,
): Promise<EngineEvaluation> {
  return invoke<EngineEvaluation>("analyze_position", { fen, depth });
}

// ─── Player Commands ─────────────────────────────────────────

export function getOrCreatePlayer(displayName: string) {
  return invoke("get_or_create_player", { displayName });
}

export function updatePlayerSettings(playerId: string, settings: unknown) {
  return invoke("update_player_settings", { playerId, settings });
}

// ─── Review Commands ─────────────────────────────────────────

export function getGameReview(
  gameId: string,
  depth: number,
): Promise<MoveEvaluation[]> {
  return invoke<MoveEvaluation[]>("get_game_review", { gameId, depth });
}

export function getGameHistory(
  limit: number,
  offset: number,
): Promise<GameRecord[]> {
  return invoke<GameRecord[]>("get_game_history", { limit, offset });
}
