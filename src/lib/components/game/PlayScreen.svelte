<script lang="ts">
  import Chessboard from "../board/Chessboard.svelte";
  import EvalBar from "../board/EvalBar.svelte";
  import MoveList from "../board/MoveList.svelte";
  import GameOverDialog from "./GameOverDialog.svelte";
  import { gameStore } from "../../stores/game.svelte";
  import * as api from "../../api/commands";
  import type { Position } from "../../types/chess";

  type Props = {
    onReview?: (gameId: string) => void;
    onNewGame?: () => void;
  };

  let { onReview, onNewGame }: Props = $props();

  let position = $derived(gameStore.position);
  let config = $derived(gameStore.config);

  async function handlePlayerMove(from: string, to: string) {
    if (!config || !position) return;
    if (gameStore.engineThinking) return;

    // Check if this is a promotion (pawn reaching last rank)
    const uci = `${from}${to}`;

    try {
      // Make the player's move
      const newPosition = await api.makeMove(uci);
      gameStore.position = newPosition;

      // Check if game is over after player move
      if (newPosition.isGameOver) {
        gameStore.phase = "game-over";
        return;
      }

      // Request engine move
      await requestEngineMove(newPosition);
    } catch (err) {
      console.error("Move failed:", err);
      // Refresh position to reset the board
      const currentPos = await api.getPosition();
      gameStore.position = currentPos;
    }
  }

  async function requestEngineMove(currentPosition: Position) {
    if (!config) return;

    gameStore.engineThinking = true;
    try {
      const engineMove = await api.getEngineMove(
        currentPosition.fen,
        undefined,
        config.engineStrength.elo,
        config.engineStrength.skillLevel,
      );

      // Apply engine's move
      const newPosition = await api.makeMove(engineMove.uci);
      gameStore.position = newPosition;

      if (newPosition.isGameOver) {
        gameStore.phase = "game-over";
      }
    } catch (err) {
      console.error("Engine move failed:", err);
    } finally {
      gameStore.engineThinking = false;
    }
  }

  async function handleResign() {
    try {
      const record = await api.resign();
      gameStore.lastGameRecord = record;
      gameStore.phase = "game-over";
    } catch (err) {
      console.error("Resign failed:", err);
    }
  }

  // If player is black, request engine's first move
  $effect(() => {
    if (
      config &&
      position &&
      config.playerColor === "black" &&
      position.sanHistory.length === 0
    ) {
      requestEngineMove(position);
    }
  });
</script>

<div class="play-screen">
  <div class="board-area">
    <EvalBar score={gameStore.currentScore} orientation={config?.playerColor} />
    <Chessboard
      fen={position?.fen}
      orientation={config?.playerColor}
      turnColor={position?.turn}
      dests={gameStore.isPlayerTurn && !gameStore.engineThinking ? position?.dests ?? {} : {}}
      lastMove={position?.lastMove}
      isCheck={position?.isCheck}
      onMove={handlePlayerMove}
    />
  </div>

  <div class="side-panel">
    <div class="panel-header">
      <div class="turn-indicator">
        {#if gameStore.engineThinking}
          <span class="thinking">Engine thinking...</span>
        {:else if position?.isGameOver}
          <span class="game-over-text">Game Over</span>
        {:else}
          <span>{position?.turn === config?.playerColor ? "Your turn" : "Opponent's turn"}</span>
        {/if}
      </div>
    </div>

    <MoveList moves={position?.sanHistory ?? []} />

    <div class="panel-footer">
      {#if !position?.isGameOver}
        <button class="btn-resign" onclick={handleResign}>Resign</button>
      {/if}
    </div>
  </div>

  {#if gameStore.phase === "game-over"}
    <GameOverDialog
      outcome={position?.outcome ?? null}
      playerColor={config?.playerColor ?? "white"}
      moveCount={position?.sanHistory.length ?? 0}
      onReview={() => onReview?.(gameStore.lastGameRecord?.id ?? "")}
      onNewGame={() => onNewGame?.()}
    />
  {/if}
</div>

<style>
  .play-screen {
    display: flex;
    gap: 24px;
    padding: 24px;
    height: 100%;
    align-items: flex-start;
    justify-content: center;
  }

  .board-area {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .side-panel {
    display: flex;
    flex-direction: column;
    width: 280px;
    background: white;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    overflow: hidden;
  }

  .panel-header {
    padding: 12px 16px;
    border-bottom: 1px solid #e5e7eb;
  }

  .turn-indicator {
    font-size: 14px;
    font-weight: 500;
  }

  .thinking {
    color: #6366f1;
  }

  .game-over-text {
    color: #dc2626;
    font-weight: 600;
  }

  .panel-footer {
    padding: 12px 16px;
    border-top: 1px solid #e5e7eb;
    margin-top: auto;
  }

  .btn-resign {
    width: 100%;
    padding: 8px;
    background: #fee2e2;
    color: #dc2626;
    border: 1px solid #fca5a5;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    transition: background 0.15s;
  }

  .btn-resign:hover {
    background: #fecaca;
  }
</style>
