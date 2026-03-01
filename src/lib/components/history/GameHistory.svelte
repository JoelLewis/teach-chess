<script lang="ts">
  import GameCard from "./GameCard.svelte";
  import * as api from "../../api/commands";
  import type { GameRecord } from "../../types/game";

  type Props = {
    onReview: (gameId: string) => void;
  };

  let { onReview }: Props = $props();

  let games = $state<GameRecord[]>([]);
  let loading = $state(true);
  let page = $state(0);
  const pageSize = 20;

  async function loadGames() {
    loading = true;
    try {
      games = await api.getGameHistory(pageSize, page * pageSize);
    } catch (err) {
      console.error("Failed to load game history:", err);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    loadGames();
  });
</script>

<div class="game-history">
  <h2 class="text-xl font-semibold mb-4">Game History</h2>

  {#if loading}
    <p class="text-gray-500">Loading...</p>
  {:else if games.length === 0}
    <p class="text-gray-500">No games played yet. Start a new game!</p>
  {:else}
    <div class="games-list">
      {#each games as game (game.id)}
        <GameCard {game} {onReview} />
      {/each}
    </div>

    <div class="pagination">
      {#if page > 0}
        <button onclick={() => { page--; loadGames(); }}>Previous</button>
      {/if}
      {#if games.length === pageSize}
        <button onclick={() => { page++; loadGames(); }}>Next</button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .game-history {
    padding: 24px;
    max-width: 600px;
  }

  .games-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .pagination {
    display: flex;
    gap: 8px;
    justify-content: center;
    margin-top: 16px;
  }

  .pagination button {
    padding: 6px 16px;
    background: white;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }

  .pagination button:hover {
    background: #f3f4f6;
  }
</style>
