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

  let resultColor = $derived.by(() => {
    if (resultLabel === "Win") return "text-green-600";
    if (resultLabel === "Loss") return "text-red-600";
    return "text-gray-600";
  });
</script>

<div class="game-card">
  <div class="flex justify-between items-start">
    <div>
      <span class="font-semibold {resultColor}">{resultLabel}</span>
      <span class="text-sm text-gray-500 ml-2">vs Engine ({game.engineElo})</span>
    </div>
    <span class="text-xs text-gray-400">{game.result}</span>
  </div>
  <div class="text-sm text-gray-500 mt-1">
    {game.moveCount} moves &middot; {game.playerColor === "white" ? "White" : "Black"}
    &middot; {game.timeControl}
  </div>
  <button class="review-btn mt-2" onclick={() => onReview(game.id)}>
    Review
  </button>
</div>

<style>
  .game-card {
    padding: 12px 16px;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    background: white;
  }

  .review-btn {
    padding: 4px 12px;
    font-size: 12px;
    background: #f3f4f6;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    cursor: pointer;
  }

  .review-btn:hover {
    background: #e5e7eb;
  }
</style>
