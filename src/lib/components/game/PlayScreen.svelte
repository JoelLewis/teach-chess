<script lang="ts">
  import Chessboard from "../board/Chessboard.svelte";
  import EvalBar from "../board/EvalBar.svelte";
  import MoveList from "../board/MoveList.svelte";
  import LoadingSpinner from "../ui/LoadingSpinner.svelte";
  import InGameCoachingPanel from "./InGameCoachingPanel.svelte";
  import GameOverDialog from "./GameOverDialog.svelte";
  import { gameStore } from "../../stores/game.svelte";
  import { errorStore } from "../../stores/error.svelte";
  import * as api from "../../api/commands";
  import { onEngineInfo } from "../../api/events";
  import type { Position } from "../../types/chess";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  type Props = {
    onReview?: (gameId: string) => void;
    onNewGame?: () => void;
  };

  let { onReview, onNewGame }: Props = $props();

  let position = $derived(gameStore.position);
  let config = $derived(gameStore.config);

  // Personality feedback badge (for CoachPicks/Surprise modes)
  let personalityLabel = $derived.by(() => {
    if (!config || config.opponentMode === "choose" || !gameStore.resolvedPersonality) return null;
    const name = gameStore.resolvedPersonality.charAt(0).toUpperCase() + gameStore.resolvedPersonality.slice(1);
    return `Playing against: ${name}`;
  });

  let showPersonalityBadge = $state(false);

  $effect(() => {
    if (personalityLabel) {
      showPersonalityBadge = true;
      const timer = setTimeout(() => {
        showPersonalityBadge = false;
      }, 10_000);
      return () => clearTimeout(timer);
    }
  });

  async function handlePlayerMove(from: string, to: string) {
    if (!config || !position) return;
    if (gameStore.engineThinking) return;

    const fenBefore = position.fen;
    const uci = `${from}${to}`;

    try {
      // Make the player's move
      const newPosition = await api.makeMove(uci);
      gameStore.position = newPosition;

      // Check if game is over after player move
      if (newPosition.isGameOver) {
        await saveGame();
        return;
      }

      // Evaluate the player's move (coaching — never blocks the game)
      try {
        if (config.coachingLevel !== "silent") {
          const isPlayerWhite = config.playerColor === "white";
          const moveNumber = Math.ceil(newPosition.sanHistory.length / 2);

          const feedback = await api.evaluatePlayerMove(
            fenBefore,
            newPosition.fen,
            isPlayerWhite,
            moveNumber,
            config.coachingLevel,
          );

          gameStore.latestCoaching = feedback;
          // Keep only the last 20 coaching entries to avoid unbounded memory growth
          const history = [...gameStore.coachingHistory, feedback];
          gameStore.coachingHistory = history.length > 20 ? history.slice(-20) : history;

          // Track game phase from coaching context
          if (feedback.coachingContext?.phase) {
            gameStore.currentChessPhase = feedback.coachingContext.phase;
          }
        }
      } catch (err) {
        console.error("Coaching evaluation failed (non-blocking):", err);
      }

      // Request engine move
      await requestEngineMove(newPosition);
    } catch (err) {
      console.error("Move failed:", err);
      errorStore.show(`Move failed: ${err}`);
      const currentPos = await api.getPosition();
      gameStore.position = currentPos;
    }
  }

  async function requestEngineMove(currentPosition: Position) {
    if (!config) return;

    gameStore.engineThinking = true;
    try {
      let moveUci: string;

      // Branch: use personality-based move selection if a personality is active
      if (gameStore.resolvedPersonality) {
        const selected = await api.getOpponentMove(
          currentPosition.fen,
          gameStore.resolvedPersonality,
          config.teachingMode,
          gameStore.weakCategories.length > 0 ? gameStore.weakCategories : undefined,
        );
        moveUci = selected.uci;
      } else {
        const engineMove = await api.getEngineMove(
          currentPosition.fen,
          undefined,
          config.engineStrength.elo,
          config.engineStrength.skillLevel,
        );
        moveUci = engineMove.uci;
      }

      // Apply engine's move
      const newPosition = await api.makeMove(moveUci);
      gameStore.position = newPosition;

      if (newPosition.isGameOver) {
        await saveGame();
        return;
      }

      // Analyze pre-move hints for the player's next turn
      try {
        if (config.coachingLevel === "fullCoach") {
          const isPlayerWhite = config.playerColor === "white";
          const hint = await api.analyzePreMoveHints(
            newPosition.fen,
            gameStore.currentChessPhase,
            config.coachingLevel,
            isPlayerWhite,
            gameStore.resolvedPersonality,
          );
          gameStore.preMoveHint = hint;
        } else {
          gameStore.preMoveHint = null;
        }
      } catch (err) {
        console.error("Pre-move hint failed (non-blocking):", err);
      }
    } catch (err) {
      console.error("Engine move failed:", err);
      errorStore.show(`Engine error: ${err}`);
    } finally {
      gameStore.engineThinking = false;
    }
  }

  async function saveGame() {
    try {
      const record = await api.saveCompletedGame();
      gameStore.lastGameRecord = record;
    } catch (err) {
      console.error("Failed to save game:", err);
    }
    gameStore.phase = "game-over";
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

  // Listen for engine evaluation updates to drive the eval bar
  $effect(() => {
    let unlisten: UnlistenFn | undefined;
    onEngineInfo((info) => {
      gameStore.currentScore = info.score;
    }).then((fn) => (unlisten = fn));

    return () => unlisten?.();
  });

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
          <span class="thinking"><LoadingSpinner size="sm" message="Engine thinking..." /></span>
        {:else if position?.isGameOver}
          <span class="game-over-text">Game Over</span>
        {:else}
          <span>{position?.turn === config?.playerColor ? "Your turn" : "Opponent's turn"}</span>
        {/if}
      </div>
      {#if showPersonalityBadge && personalityLabel}
        <div class="personality-badge">{personalityLabel}</div>
      {/if}
    </div>

    {#if config?.coachingLevel !== "silent"}
      <div class="coaching-section">
        <InGameCoachingPanel />
      </div>
    {/if}

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
    background: var(--cm-bg-surface);
    border-radius: 8px;
    box-shadow: var(--cm-shadow-sm);
    overflow: hidden;
  }

  .panel-header {
    padding: 12px 16px;
    border-bottom: 1px solid var(--cm-border-light);
  }

  .turn-indicator {
    font-size: 14px;
    font-weight: 500;
  }

  .thinking {
    color: var(--cm-accent-primary-light);
  }

  .game-over-text {
    color: var(--cm-status-error);
    font-weight: 600;
  }

  .panel-footer {
    padding: 12px 16px;
    border-top: 1px solid var(--cm-border-light);
    margin-top: auto;
  }

  .coaching-section {
    padding: 4px 8px;
    border-bottom: 1px solid var(--cm-border-light);
    max-height: 180px;
    overflow-y: auto;
  }

  .btn-resign {
    width: 100%;
    padding: 8px;
    background: var(--cm-status-error-bg-alt);
    color: var(--cm-status-error);
    border: 1px solid var(--cm-status-error-lighter);
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    transition: background 0.15s;
  }

  .btn-resign:hover {
    background: var(--cm-status-error-muted);
  }

  .personality-badge {
    margin-top: 4px;
    padding: 3px 8px;
    font-size: 12px;
    color: var(--cm-accent-primary-light);
    background: var(--cm-accent-primary-bg);
    border-radius: 4px;
    animation: fade-out 10s forwards;
  }

  @keyframes fade-out {
    0%, 80% { opacity: 1; }
    100% { opacity: 0; }
  }
</style>
