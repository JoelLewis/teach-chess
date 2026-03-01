<script lang="ts">
  import Chessboard from "../board/Chessboard.svelte";
  import EvalBar from "../board/EvalBar.svelte";
  import MoveAnnotation from "./MoveAnnotation.svelte";
  import * as api from "../../api/commands";
  import { onReviewProgress } from "../../api/events";
  import type { MoveEvaluation } from "../../types/engine";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  type Props = {
    gameId: string;
    onBack: () => void;
  };

  let { gameId, onBack }: Props = $props();

  let evaluations = $state<MoveEvaluation[]>([]);
  let selectedIndex = $state(-1);
  let loading = $state(true);
  let progress = $state({ current: 0, total: 0 });

  let selectedEval = $derived(
    selectedIndex >= 0 ? evaluations[selectedIndex] : null,
  );

  let displayFen = $derived(
    selectedEval?.fenBefore ??
      "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
  );

  let displayScore = $derived(selectedEval?.evalBefore ?? null);

  async function loadReview() {
    loading = true;
    try {
      evaluations = await api.getGameReview(gameId, 18);
    } catch (err) {
      console.error("Failed to load review:", err);
    } finally {
      loading = false;
    }
  }

  function navigateMove(direction: number) {
    const newIndex = selectedIndex + direction;
    if (newIndex >= -1 && newIndex < evaluations.length) {
      selectedIndex = newIndex;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "ArrowLeft") navigateMove(-1);
    if (event.key === "ArrowRight") navigateMove(1);
  }

  // Summary stats
  let summary = $derived(() => {
    const counts = { best: 0, excellent: 0, good: 0, inaccuracy: 0, mistake: 0, blunder: 0 };
    for (const ev of evaluations) {
      if (ev.classification) {
        counts[ev.classification]++;
      }
    }
    return counts;
  });

  $effect(() => {
    let unlisten: UnlistenFn | undefined;
    onReviewProgress((p) => {
      progress = { current: p.current, total: p.total };
    }).then((fn) => (unlisten = fn));

    loadReview();

    return () => unlisten?.();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="review-screen">
  <div class="board-area">
    <EvalBar score={displayScore} />
    <Chessboard fen={displayFen} viewOnly={true} />
  </div>

  <div class="review-panel">
    <div class="panel-header">
      <button class="back-btn" onclick={onBack}>Back</button>
      <h2 class="text-lg font-semibold">Game Review</h2>
    </div>

    {#if loading}
      <div class="loading">
        <p>Analyzing game...</p>
        {#if progress.total > 0}
          <div class="progress-bar">
            <div
              class="progress-fill"
              style="width: {(progress.current / progress.total) * 100}%"
            ></div>
          </div>
          <p class="text-sm text-gray-500">
            {progress.current} / {progress.total} moves
          </p>
        {/if}
      </div>
    {:else}
      <div class="summary">
        {#each Object.entries(summary()) as [classification, count]}
          {#if count > 0}
            <span class="summary-item">
              {count} {classification}{count !== 1 ? "s" : ""}
            </span>
          {/if}
        {/each}
      </div>

      <div class="nav-buttons">
        <button onclick={() => navigateMove(-1)} disabled={selectedIndex <= -1}>
          &#9664;
        </button>
        <button onclick={() => navigateMove(1)} disabled={selectedIndex >= evaluations.length - 1}>
          &#9654;
        </button>
      </div>

      <div class="move-list">
        {#each evaluations as evaluation, i (i)}
          <MoveAnnotation
            {evaluation}
            isSelected={selectedIndex === i}
            onClick={() => (selectedIndex = i)}
          />
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .review-screen {
    display: flex;
    gap: 24px;
    padding: 24px;
    height: 100%;
    justify-content: center;
  }

  .board-area {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .review-panel {
    width: 300px;
    background: white;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .panel-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid #e5e7eb;
  }

  .back-btn {
    padding: 4px 12px;
    font-size: 13px;
    background: #f3f4f6;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    cursor: pointer;
  }

  .loading {
    padding: 24px;
    text-align: center;
  }

  .progress-bar {
    height: 6px;
    background: #e5e7eb;
    border-radius: 3px;
    overflow: hidden;
    margin: 8px 0;
  }

  .progress-fill {
    height: 100%;
    background: #4f46e5;
    transition: width 0.3s;
  }

  .summary {
    padding: 8px 16px;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    border-bottom: 1px solid #e5e7eb;
    font-size: 12px;
  }

  .summary-item {
    color: #6b7280;
  }

  .nav-buttons {
    display: flex;
    justify-content: center;
    gap: 12px;
    padding: 8px;
  }

  .nav-buttons button {
    padding: 6px 16px;
    background: white;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    cursor: pointer;
    font-size: 16px;
  }

  .nav-buttons button:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .move-list {
    overflow-y: auto;
    flex: 1;
    padding: 4px;
  }
</style>
