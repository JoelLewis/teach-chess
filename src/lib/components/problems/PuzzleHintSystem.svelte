<script lang="ts">
  import { puzzleStore } from "../../stores/puzzle.svelte";
  import { errorStore } from "../../stores/error.svelte";
  import * as api from "../../api/commands";

  let loading = $state(false);

  const maxHints = 3;
  const hintsUsed = $derived(puzzleStore.hintsRevealed.length);
  const canReveal = $derived(
    hintsUsed < maxHints && puzzleStore.phase === "solving",
  );

  async function revealHint() {
    if (!canReveal || loading) return;
    loading = true;
    try {
      const hint = await api.requestPuzzleHint();
      if (hint) {
        puzzleStore.hintsRevealed = [...puzzleStore.hintsRevealed, hint];
      }
    } catch (err) {
      errorStore.show(`Failed to get hint: ${err}`);
    } finally {
      loading = false;
    }
  }
</script>

<div class="hint-system">
  {#if puzzleStore.hintsRevealed.length > 0}
    <div class="hints-list">
      {#each puzzleStore.hintsRevealed as hint, i}
        <div class="hint-card tier-{i + 1}">
          <span class="hint-label">Hint {i + 1}</span>
          <p class="hint-text">{hint}</p>
        </div>
      {/each}
    </div>
  {/if}

  {#if canReveal}
    <button class="hint-btn" onclick={revealHint} disabled={loading}>
      {#if loading}
        Getting hint...
      {:else}
        Hint ({hintsUsed}/{maxHints})
      {/if}
    </button>
  {:else if hintsUsed >= maxHints}
    <span class="hints-exhausted">All hints used</span>
  {/if}
</div>

<style>
  .hint-system {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .hints-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .hint-card {
    padding: 8px 12px;
    border-radius: 6px;
    font-size: 13px;
  }

  .hint-card.tier-1 {
    background: #fef3c7;
    border-left: 3px solid #f59e0b;
  }

  .hint-card.tier-2 {
    background: #fed7aa;
    border-left: 3px solid #f97316;
  }

  .hint-card.tier-3 {
    background: #fecaca;
    border-left: 3px solid #ef4444;
  }

  .hint-label {
    font-weight: 600;
    font-size: 11px;
    text-transform: uppercase;
    opacity: 0.7;
  }

  .hint-text {
    margin: 4px 0 0;
  }

  .hint-btn {
    padding: 8px 12px;
    background: #fef3c7;
    color: #92400e;
    border: 1px solid #fcd34d;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    transition: background 0.15s;
  }

  .hint-btn:hover:not(:disabled) {
    background: #fde68a;
  }

  .hint-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .hints-exhausted {
    font-size: 12px;
    color: #9ca3af;
    text-align: center;
  }
</style>
