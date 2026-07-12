<script lang="ts">
  import type { Color, GameOutcome } from "../../api/bindings";
  import GameSummaryCard from "./GameSummaryCard.svelte";
  import { generateGameSummary, getGameReview } from "../../api/commands";
  import { summarizeEvaluations, type GameStats } from "../../utils/reviewStats";

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

  // Unit variants of GameOutcome cross the IPC boundary as plain strings
  // ("stalemate", "draw", …); only checkmate/resignation carry a winner.
  function decisive(
    o: GameOutcome,
  ): { kind: "checkmate" | "resignation"; winner: Color } | null {
    if (typeof o === "string") return null;
    if (o.checkmate) return { kind: "checkmate", winner: o.checkmate.winner };
    if (o.resignation) return { kind: "resignation", winner: o.resignation.winner };
    return null;
  }

  let resultText = $derived.by(() => {
    if (!outcome) return "Game Over";

    const d = decisive(outcome);
    if (d?.kind === "checkmate") {
      return d.winner === playerColor
        ? "You win by checkmate!"
        : "You lost by checkmate";
    }
    if (d?.kind === "resignation") {
      return d.winner === playerColor
        ? "Opponent resigned — you win!"
        : "You resigned";
    }
    switch (outcome) {
      case "stalemate":
        return "Draw by stalemate";
      case "insufficientmaterial":
        return "Draw by insufficient material";
      case "threefoldrepetition":
        return "Draw by threefold repetition";
      case "fiftymoverule":
        return "Draw by fifty-move rule";
      default:
        return "Draw";
    }
  });

  let isWin = $derived.by(() => {
    const d = outcome && decisive(outcome);
    return d ? d.winner === playerColor : false;
  });

  let isLoss = $derived.by(() => {
    const d = outcome && decisive(outcome);
    return d ? d.winner !== playerColor : false;
  });

  let cardResult = $derived<"win" | "loss" | "draw">(
    isWin ? "win" : isLoss ? "loss" : "draw",
  );

  let outcomeDetail = $derived.by(() => {
    if (!outcome) return "";
    const d = decisive(outcome);
    if (d) return `by ${d.kind}`;
    switch (outcome) {
      case "stalemate":
        return "by stalemate";
      case "insufficientmaterial":
        return "by insufficient material";
      case "threefoldrepetition":
        return "by threefold repetition";
      case "fiftymoverule":
        return "by fifty-move rule";
      default:
        return "by agreement";
    }
  });

  // Run a quick review for real stats, then fetch the AI-generated summary.
  // Both steps degrade gracefully: the card renders fine with zero stats
  // and without an AI quote.
  $effect(() => {
    if (!outcome) return;
    let cancelled = false;

    (async () => {
      if (gameId) {
        try {
          const evaluations = await getGameReview(gameId, SUMMARY_REVIEW_DEPTH);
          if (cancelled) return;
          stats = summarizeEvaluations(evaluations, playerColor === "white");
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
    <div id="game-over-title" class="result" class:win={isWin} class:loss={!isWin} aria-live="assertive">
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
