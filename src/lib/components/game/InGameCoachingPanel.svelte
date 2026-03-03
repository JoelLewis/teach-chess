<script lang="ts">
  import { gameStore } from "../../stores/game.svelte";
  import { CLASSIFICATION_COLORS } from "../../types/engine";

  let visible = $state(true);
  let expanded = $state(false);
  let fadeTimer: ReturnType<typeof setTimeout> | null = null;

  // Auto-fade post-move feedback after 8s (unless expanded)
  $effect(() => {
    const coaching = gameStore.latestCoaching;
    if (coaching && coaching.coachingText) {
      visible = true;
      expanded = false;

      if (fadeTimer) clearTimeout(fadeTimer);
      fadeTimer = setTimeout(() => {
        if (!expanded) visible = false;
      }, 8000);
    }

    return () => {
      if (fadeTimer) clearTimeout(fadeTimer);
    };
  });

  let classColor = $derived(
    gameStore.latestCoaching?.classification
      ? CLASSIFICATION_COLORS[gameStore.latestCoaching.classification]
      : "#888",
  );

  const THEME_LABELS: Record<string, string> = {
    knightOnRim: "Knight on rim",
    bishopPairAdvantage: "Bishop pair",
    isolatedQueenPawn: "Isolated QP",
    passedPawn: "Passed pawn",
    doubledPawns: "Doubled pawns",
    backwardPawn: "Backward pawn",
    openFile: "Open file",
    rookOnSeventh: "Rook on 7th",
    kingSafetyCompromised: "King safety",
    undevelopedPieces: "Development",
    centralControl: "Center control",
    pawnChainTension: "Pawn tension",
    materialImbalance: "Material",
    backRankWeakness: "Back rank",
    pinnedPiece: "Pin",
    forkAvailable: "Fork",
    hangingMaterial: "Hanging piece",
  };
</script>

{#if gameStore.preMoveHint?.hintText && gameStore.isPlayerTurn}
  <div
    class="hint-bar"
    class:hint-tactical={gameStore.preMoveHint.hintType === "tacticalAlert"}
    class:hint-phase={gameStore.preMoveHint.hintType === "phaseTransition"}
    class:hint-strategic={gameStore.preMoveHint.hintType === "strategicReminder"}
  >
    <span class="hint-icon">
      {#if gameStore.preMoveHint.hintType === "tacticalAlert"}
        !
      {:else if gameStore.preMoveHint.hintType === "phaseTransition"}
        ~
      {:else}
        i
      {/if}
    </span>
    <span class="hint-text">{gameStore.preMoveHint.hintText}</span>
  </div>
{/if}

{#if gameStore.latestCoaching?.coachingText && visible}
  <div class="coaching-feedback" class:expanded>
    <div class="feedback-header">
      <span class="classification-badge" style="background: {classColor}">
        {gameStore.latestCoaching.classification}
      </span>
      <span class="move-number">Move {gameStore.latestCoaching.moveNumber}</span>
      <button
        class="expand-btn"
        onclick={() => {
          expanded = !expanded;
          if (expanded && fadeTimer) {
            clearTimeout(fadeTimer);
            fadeTimer = null;
          }
        }}
      >
        {expanded ? "−" : "+"}
      </button>
    </div>
    <p class="feedback-text">{gameStore.latestCoaching.coachingText}</p>
    {#if expanded && gameStore.latestCoaching.coachingContext?.themes.length}
      <div class="theme-tags">
        {#each gameStore.latestCoaching.coachingContext.themes as theme}
          <span class="theme-tag">{THEME_LABELS[theme] ?? theme}</span>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .hint-bar {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 12px;
    line-height: 1.4;
    margin: 4px 0;
  }

  .hint-tactical {
    background: #fffbeb;
    border: 1px solid #f59e0b;
    color: #92400e;
  }

  .hint-phase {
    background: #eff6ff;
    border: 1px solid #60a5fa;
    color: #1e40af;
  }

  .hint-strategic {
    background: #f0fdf4;
    border: 1px solid #86efac;
    color: #166534;
  }

  .hint-icon {
    font-weight: 700;
    font-size: 13px;
    flex-shrink: 0;
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.08);
  }

  .hint-text {
    flex: 1;
  }

  .coaching-feedback {
    padding: 8px 12px;
    margin: 4px 0;
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    transition: all 0.2s;
  }

  .feedback-header {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .classification-badge {
    display: inline-block;
    font-size: 10px;
    color: white;
    padding: 1px 8px;
    border-radius: 3px;
    text-transform: capitalize;
  }

  .move-number {
    font-size: 11px;
    color: #94a3b8;
    flex: 1;
  }

  .expand-btn {
    width: 20px;
    height: 20px;
    border: 1px solid #d1d5db;
    border-radius: 3px;
    background: white;
    cursor: pointer;
    font-size: 14px;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #6b7280;
  }

  .expand-btn:hover {
    background: #f3f4f6;
  }

  .feedback-text {
    font-size: 12px;
    line-height: 1.5;
    color: #334155;
    margin: 4px 0 0;
  }

  .coaching-feedback:not(.expanded) .feedback-text {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .theme-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 3px;
    margin-top: 6px;
  }

  .theme-tag {
    font-size: 9px;
    color: #64748b;
    background: #e2e8f0;
    padding: 1px 5px;
    border-radius: 3px;
  }
</style>
