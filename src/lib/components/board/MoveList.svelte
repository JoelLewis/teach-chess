<script lang="ts">
  type Props = {
    moves: string[];
    currentMoveIndex?: number;
    onMoveClick?: (index: number) => void;
  };

  let { moves, currentMoveIndex, onMoveClick }: Props = $props();

  // Group moves into pairs (white, black)
  let movePairs = $derived.by(() => {
    const pairs: { number: number; white: string; black?: string }[] = [];
    for (let i = 0; i < moves.length; i += 2) {
      pairs.push({
        number: Math.floor(i / 2) + 1,
        white: moves[i],
        black: moves[i + 1],
      });
    }
    return pairs;
  });

  let listEl: HTMLDivElement;

  // Auto-scroll to bottom when new moves are added
  $effect(() => {
    if (listEl && moves.length > 0) {
      listEl.scrollTop = listEl.scrollHeight;
    }
  });
</script>

<div bind:this={listEl} class="move-list">
  {#each movePairs as pair}
    <div class="move-row">
      <span class="move-number">{pair.number}.</span>
      <button
        class="move"
        class:active={currentMoveIndex === (pair.number - 1) * 2}
        onclick={() => onMoveClick?.((pair.number - 1) * 2)}
      >
        {pair.white}
      </button>
      {#if pair.black}
        <button
          class="move"
          class:active={currentMoveIndex === (pair.number - 1) * 2 + 1}
          onclick={() => onMoveClick?.((pair.number - 1) * 2 + 1)}
        >
          {pair.black}
        </button>
      {/if}
    </div>
  {/each}
</div>

<style>
  .move-list {
    overflow-y: auto;
    max-height: 400px;
    padding: 8px;
    font-family: var(--cm-font-mono);
    font-size: 14px;
  }

  .move-list::-webkit-scrollbar {
    width: 6px;
  }

  .move-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .move-list::-webkit-scrollbar-thumb {
    background: var(--cm-border-default);
    border-radius: 3px;
  }

  .move-row {
    display: flex;
    gap: 4px;
    padding: 2px 0;
  }

  .move-number {
    color: var(--cm-text-muted);
    min-width: 32px;
    text-align: right;
  }

  .move {
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 3px;
    color: inherit;
    font-family: inherit;
    font-size: inherit;
    min-width: 50px;
    text-align: left;
  }

  .move:hover {
    background: var(--cm-border-default);
  }

  .move.active {
    background: var(--cm-accent-secondary-muted);
    font-weight: 600;
  }
</style>
