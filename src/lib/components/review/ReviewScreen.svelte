<script lang="ts">
  import Chessboard from "../board/Chessboard.svelte";
  import EvalBar from "../board/EvalBar.svelte";
  import CoachingPanel from "./CoachingPanel.svelte";
  import PatternSummaryPanel from "./PatternSummaryPanel.svelte";
  import MoveAnnotation from "./MoveAnnotation.svelte";
  import * as api from "../../api/commands";
  import { onReviewProgress } from "../../api/events";
  import { gameStore } from "../../stores/game.svelte";
  import type {
    MoveEvaluation,
    CriticalMoment,
    PatternSummary,
    StudySuggestion,
  } from "../../types/engine";
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
  let criticalMoments = $state<CriticalMoment[]>([]);
  let patternSummary = $state<PatternSummary | null>(null);
  let studySuggestions = $state<StudySuggestion[]>([]);

  let selectedEval = $derived(
    selectedIndex >= 0 ? evaluations[selectedIndex] : null,
  );

  let displayFen = $derived(
    selectedEval?.fenBefore ??
      "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
  );

  let displayScore = $derived(selectedEval?.evalBefore ?? null);

  // Set of critical moment move indices for highlighting in the move list
  let criticalMoveIndices = $derived(
    new Set(criticalMoments.map((m) => m.moveIndex)),
  );

  async function loadReview() {
    loading = true;
    try {
      evaluations = await api.getGameReview(gameId, 18);
      // Load review enhancements after evaluations
      await loadReviewEnhancements();
    } catch (err) {
      console.error("Failed to load review:", err);
    } finally {
      loading = false;
    }
  }

  async function loadReviewEnhancements() {
    if (evaluations.length === 0) return;

    const isPlayerWhite = gameStore.config?.playerColor === "white";

    try {
      criticalMoments = await api.getCriticalMoments(evaluations, isPlayerWhite ?? true);
    } catch (err) {
      console.error("Failed to load critical moments:", err);
    }

    try {
      const summary = await api.getPatternSummary(evaluations, isPlayerWhite ?? true);
      patternSummary = summary;
      studySuggestions = await api.getStudySuggestions(summary);
    } catch (err) {
      console.error("Failed to load pattern summary:", err);
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
  let classificationSummary = $derived.by(() => {
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
        {#each Object.entries(classificationSummary) as [classification, count]}
          {#if count > 0}
            <span class="summary-item">
              {count} {classification}{count !== 1 ? "s" : ""}
            </span>
          {/if}
        {/each}
      </div>

      {#if criticalMoments.length > 0}
        <div class="critical-moments">
          <h4 class="moments-title">Key Moments</h4>
          {#each criticalMoments as moment}
            <button
              class="moment-item"
              class:player-moment={moment.isPlayerMove}
              onclick={() => (selectedIndex = moment.moveIndex)}
            >
              <span class="moment-desc">{moment.description}</span>
            </button>
          {/each}
        </div>
      {/if}

      <PatternSummaryPanel summary={patternSummary} suggestions={studySuggestions} />

      <div class="nav-buttons">
        <button onclick={() => navigateMove(-1)} disabled={selectedIndex <= -1}>
          &#9664;
        </button>
        <button onclick={() => navigateMove(1)} disabled={selectedIndex >= evaluations.length - 1}>
          &#9654;
        </button>
      </div>

      <CoachingPanel evaluation={selectedEval} />

      <div class="move-list">
        {#each evaluations as evaluation, i (i)}
          <div class="move-row" class:critical-move={criticalMoveIndices.has(i)}>
            {#if criticalMoveIndices.has(i)}
              <span class="critical-marker" title="Key moment">!</span>
            {/if}
            <MoveAnnotation
              {evaluation}
              isSelected={selectedIndex === i}
              onClick={() => (selectedIndex = i)}
            />
          </div>
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

  .critical-moments {
    padding: 6px 12px;
    border-bottom: 1px solid #e5e7eb;
  }

  .moments-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #6b7280;
    margin: 0 0 4px;
  }

  .moment-item {
    display: block;
    width: 100%;
    text-align: left;
    padding: 4px 8px;
    margin: 2px 0;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    background: #f9fafb;
    cursor: pointer;
    transition: background 0.15s;
  }

  .moment-item:hover {
    background: #f3f4f6;
  }

  .moment-item.player-moment {
    border-left: 3px solid #ef4444;
  }

  .moment-desc {
    font-size: 11px;
    color: #374151;
    line-height: 1.3;
  }

  .move-list {
    overflow-y: auto;
    flex: 1;
    padding: 4px;
  }

  .move-row {
    display: flex;
    align-items: center;
    position: relative;
  }

  .move-row :global(.move-annotation) {
    flex: 1;
  }

  .critical-move {
    background: #fef9c3;
    border-radius: 3px;
  }

  .critical-marker {
    font-size: 12px;
    font-weight: 700;
    color: #dc2626;
    width: 16px;
    flex-shrink: 0;
    text-align: center;
  }
</style>
