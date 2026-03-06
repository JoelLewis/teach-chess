<script lang="ts">
  import type { Color, GameOutcome } from "../../types/chess";

  type Props = {
    outcome: GameOutcome | null;
    playerColor: Color;
    moveCount: number;
    onReview: () => void;
    onNewGame: () => void;
  };

  let { outcome, playerColor, moveCount, onReview, onNewGame }: Props = $props();

  let dialogEl: HTMLDivElement;
  let primaryBtnEl: HTMLButtonElement;

  let resultText = $derived.by(() => {
    if (!outcome) return "Game Over";

    if ("checkmate" in outcome) {
      return outcome.checkmate.winner === playerColor
        ? "You win by checkmate!"
        : "You lost by checkmate";
    }
    if ("resignation" in outcome) {
      return outcome.resignation.winner === playerColor
        ? "Opponent resigned — you win!"
        : "You resigned";
    }
    if ("stalemate" in outcome) return "Draw by stalemate";
    if ("insufficientMaterial" in outcome) return "Draw by insufficient material";
    if ("threefoldRepetition" in outcome) return "Draw by threefold repetition";
    if ("fiftyMoveRule" in outcome) return "Draw by fifty-move rule";
    return "Draw";
  });

  let isWin = $derived.by(() => {
    if (!outcome) return false;
    if ("checkmate" in outcome) return outcome.checkmate.winner === playerColor;
    if ("resignation" in outcome) return outcome.resignation.winner === playerColor;
    return false;
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
    <div id="game-over-title" class="result" class:win={isWin} class:loss={!isWin}>
      {resultText}
    </div>
    <p class="move-count">Game lasted {moveCount} moves</p>
    <div class="actions">
      <button class="btn-review" bind:this={primaryBtnEl} onclick={onReview}>Review Game</button>
      <button class="btn-new" onclick={onNewGame}>New Game</button>
    </div>
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
