<script lang="ts">
  import GameCard from "./GameCard.svelte";
  import * as api from "../../api/commands";
  import { errorStore } from "../../stores/error.svelte";
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
      errorStore.show("Failed to load game history");
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    // Reactively re-runs when `page` changes (read inline to track dependency)
    void page;
    loadGames();
  });
</script>

<div class="game-history">
  <h2 class="section-heading">Game History</h2>

  {#if loading}
    <p class="state-message">Loading...</p>
  {:else if games.length === 0}
    <p class="state-message">No games yet — start playing to see your history here.</p>
  {:else}
    <div class="games-list">
      {#each games as game (game.id)}
        <GameCard {game} {onReview} />
      {/each}
    </div>

    <div class="pagination">
      {#if page > 0}
        <button onclick={() => page--}>Previous</button>
      {/if}
      {#if games.length === pageSize}
        <button onclick={() => page++}>Next</button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .game-history {
    padding: 24px;
    max-width: 600px;
  }

  .section-heading {
    font-size: 20px;
    font-weight: 600;
    margin-bottom: 16px;
    color: var(--cm-text-primary);
  }

  .state-message {
    color: var(--cm-text-muted);
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
    background: var(--cm-bg-surface);
    border: 1px solid var(--cm-border-medium);
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }

  .pagination button:hover {
    background: var(--cm-bg-hover);
  }
</style>
