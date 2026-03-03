<script lang="ts">
  import type { GameRecord } from "../../types/game";

  type Props = {
    game: GameRecord;
    onReview: (gameId: string) => void;
  };

  let { game, onReview }: Props = $props();

  let resultLabel = $derived.by(() => {
    switch (game.result) {
      case "1-0":
        return game.playerColor === "white" ? "Win" : "Loss";
      case "0-1":
        return game.playerColor === "black" ? "Win" : "Loss";
      default:
        return "Draw";
    }
  });

  let resultClass = $derived.by(() => {
    if (resultLabel === "Win") return "result-win";
    if (resultLabel === "Loss") return "result-loss";
    return "result-draw";
  });
</script>

<div class="game-card">
  <div class="card-header">
    <div>
      <span class="result-label {resultClass}">{resultLabel}</span>
      <span class="opponent-info">vs Engine ({game.engineElo})</span>
    </div>
    <span class="result-raw">{game.result}</span>
  </div>
  <div class="game-meta">
    {game.moveCount} moves &middot; {game.playerColor === "white" ? "White" : "Black"}
    &middot; {game.timeControl}
  </div>
  <button class="review-btn" onclick={() => onReview(game.id)}>
    Review
  </button>
</div>

<style>
  .game-card {
    padding: 12px 16px;
    border: 1px solid var(--cm-border-light);
    border-radius: 8px;
    background: var(--cm-bg-surface);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }

  .result-label {
    font-weight: 600;
  }

  .result-win {
    color: var(--cm-status-success);
  }

  .result-loss {
    color: var(--cm-status-error);
  }

  .result-draw {
    color: var(--cm-text-secondary);
  }

  .opponent-info {
    font-size: 14px;
    color: var(--cm-text-muted);
    margin-left: 8px;
  }

  .result-raw {
    font-size: 12px;
    color: var(--cm-text-faint);
  }

  .game-meta {
    font-size: 14px;
    color: var(--cm-text-muted);
    margin-top: 4px;
  }

  .review-btn {
    margin-top: 8px;
    padding: 4px 12px;
    font-size: 12px;
    background: var(--cm-bg-hover);
    border: 1px solid var(--cm-border-medium);
    border-radius: 4px;
    cursor: pointer;
  }

  .review-btn:hover {
    background: var(--cm-bg-active);
  }
</style>
