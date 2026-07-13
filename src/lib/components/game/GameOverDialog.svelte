<script lang="ts">
  import type { Color, GameOutcome } from "../../api/bindings";
  import GameSummaryCard from "./GameSummaryCard.svelte";
  import { generateGameSummary, getGameReview } from "../../api/commands";
  import { summarizeEvaluations, type GameStats } from "../../utils/reviewStats";
  import { gameResult } from "../../utils/gameSummary";

  type Props = {
    outcome: GameOutcome | null;
    playerColor: Color;
    moveCount: number;
    gameId: string | null;
    opponentInfo?: string;
    onReview: () => void;
    onNewGame: () => void;
  };

  let {
    outcome,
    playerColor,
    moveCount,
    gameId,
    opponentInfo = "vs Opponent",
    onReview,
    onNewGame,
  }: Props = $props();

  /** Depth for the quick post-game review that feeds the summary stats.
   *  Kept shallow so stats arrive within a few seconds; the Review screen
   *  re-analyzes at full depth. */
  const SUMMARY_REVIEW_DEPTH = 10;

  let dialogEl: HTMLDivElement;
  let primaryBtnEl: HTMLButtonElement;
  let aiQuote = $state<string | null>(null);
  let stats = $state<GameStats>({
    accuracyPct: 0,
    bestMoves: 0,
    inaccuracies: 0,
    mistakes: 0,
    blunders: 0,
  });
  let statsReady = $state(false);

  let result = $derived(gameResult(outcome, playerColor));
  let resultText = $derived(result.text);
  let isWin = $derived(result.card === "win");
  let isLoss = $derived(result.card === "loss");
  let cardResult = $derived(result.card);
  let outcomeDetail = $derived(result.detail);

  // Run a quick review for real stats, then fetch the AI-generated summary.
  // Both steps degrade gracefully: the card renders fine with zero stats
  // and without an AI quote.
  $effect(() => {
    if (!outcome) return;
    let cancelled = false;
    statsReady = false;

    (async () => {
      if (gameId) {
        try {
          const evaluations = await getGameReview(gameId, SUMMARY_REVIEW_DEPTH);
          if (cancelled) return;
          stats = summarizeEvaluations(evaluations, playerColor === "white");
          statsReady = true;
        } catch (err) {
          console.error("Post-game review failed (non-blocking):", err);
        }
      }

      try {
        const text = await generateGameSummary({
          result: cardResult,
          outcomeType: outcomeDetail,
          moveCount,
          accuracyPct: stats.accuracyPct,
          bestMoves: stats.bestMoves,
          blunders: stats.blunders,
          mistakes: stats.mistakes,
          inaccuracies: stats.inaccuracies,
        });
        if (!cancelled) aiQuote = text;
      } catch {
        // Silently fail — the card renders fine without an AI quote
      }
    })();

    return () => {
      cancelled = true;
    };
  });

  // Auto-focus primary button and set up focus trap + Escape key
  $effect(() => {
    primaryBtnEl?.focus();

    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        onNewGame();
        return;
      }

      if (e.key === "Tab" && dialogEl) {
        const focusable = dialogEl.querySelectorAll<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
        );
        if (focusable.length === 0) return;

        const first = focusable[0];
        const last = focusable[focusable.length - 1];

        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  });
</script>

<div class="overlay" role="dialog" aria-modal="true" aria-labelledby="game-over-title">
  <div class="dialog" bind:this={dialogEl}>
    <div id="game-over-title" class="result" class:win={isWin} class:loss={isLoss} aria-live="assertive">
      {resultText}
    </div>
    <p class="move-count">Game lasted {moveCount} moves</p>
    <div class="actions">
      <button class="btn-review" bind:this={primaryBtnEl} onclick={onReview}>Review Game</button>
      <button class="btn-new" onclick={onNewGame}>New Game</button>
    </div>
    {#if outcome}
      <GameSummaryCard
        result={cardResult}
        {outcomeDetail}
        {opponentInfo}
        {moveCount}
        statsReady={statsReady}
        accuracy={stats.accuracyPct}
        bestMoves={stats.bestMoves}
        inaccuracies={stats.inaccuracies}
        blunders={stats.blunders}
        {aiQuote}
      />
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: var(--cm-bg-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .dialog {
    background: var(--cm-bg-surface);
    border-radius: 12px;
    padding: 32px;
    max-width: 400px;
    text-align: center;
    box-shadow: var(--cm-shadow-lg);
  }

  .result {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: 8px;
  }

  .result.win {
    color: var(--cm-status-success);
  }

  .result.loss {
    color: var(--cm-status-error);
  }

  .move-count {
    color: var(--cm-text-muted);
    margin-bottom: 24px;
  }

  .actions {
    display: flex;
    gap: 12px;
    justify-content: center;
  }

  .btn-review {
    padding: 10px 20px;
    background: var(--cm-accent-primary);
    color: var(--cm-text-inverse);
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }

  .btn-review:hover {
    background: var(--cm-accent-primary-hover);
  }

  .btn-new {
    padding: 10px 20px;
    background: var(--cm-bg-surface);
    color: var(--cm-text-secondary);
    border: 1px solid var(--cm-border-medium);
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }

  .btn-new:hover {
    background: var(--cm-bg-hover);
  }
</style>
