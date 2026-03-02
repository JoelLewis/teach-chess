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
</script>

<div class="overlay">
  <div class="dialog">
    <div class="result" class:win={isWin} class:loss={!isWin}>
      {resultText}
    </div>
    <p class="move-count">{moveCount} moves played</p>
    <div class="actions">
      <button class="btn-review" onclick={onReview}>Review Game</button>
      <button class="btn-new" onclick={onNewGame}>New Game</button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .dialog {
    background: white;
    border-radius: 12px;
    padding: 32px;
    max-width: 400px;
    text-align: center;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
  }

  .result {
    font-size: 24px;
    font-weight: 700;
    margin-bottom: 8px;
  }

  .result.win {
    color: #16a34a;
  }

  .result.loss {
    color: #dc2626;
  }

  .move-count {
    color: #6b7280;
    margin-bottom: 24px;
  }

  .actions {
    display: flex;
    gap: 12px;
    justify-content: center;
  }

  .btn-review {
    padding: 10px 20px;
    background: #4f46e5;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }

  .btn-review:hover {
    background: #4338ca;
  }

  .btn-new {
    padding: 10px 20px;
    background: white;
    color: #374151;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }

  .btn-new:hover {
    background: #f3f4f6;
  }
</style>
