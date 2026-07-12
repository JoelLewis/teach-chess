<script lang="ts">
  import { onMount } from "svelte";
  import { puzzleStore } from "../../stores/puzzle.svelte";
  import * as api from "../../api/commands";
  import { puzzleRatingComparison } from "../../utils/puzzleRating";

  type Props = {
    onNextPuzzle: () => void;
  };

  let { onNextPuzzle }: Props = $props();

  const solved = $derived(puzzleStore.phase === "complete");
  const explanation = $derived(puzzleStore.explanation ?? "");

  // On a failed puzzle, compare its rating against the player's Glicko-2
  // rating for the puzzle's category. Unrated players see nothing.
  let ratingNote = $state<string | null>(null);

  onMount(async () => {
    if (puzzleStore.phase === "complete") return;
    const puzzle = puzzleStore.currentPuzzle?.puzzle;
    if (!puzzle) return;
    try {
      const skill = await api.getSkillRating(puzzle.category);
      ratingNote = puzzleRatingComparison(puzzle.difficulty, skill);
    } catch {
      // Rating unavailable is non-fatal — just omit the comparison.
    }
  });
</script>

<div class="solution-panel" class:solved class:failed={!solved}>
  <div class="result-header">
    {#if solved}
      <span class="result-icon correct-icon">&#10003;</span>
      <span class="result-text">Correct!</span>
    {:else}
      <span class="result-icon failed-icon">&#10007;</span>
      <span class="result-text">Not quite</span>
    {/if}
  </div>

  {#if !solved && ratingNote}
    <p class="rating-note">{ratingNote}</p>
  {/if}

  {#if explanation}
    <div class="explanation">
      {#each explanation.split("\n") as line}
        {#if line.trim()}
          <p>{line}</p>
        {/if}
      {/each}
    </div>
  {/if}

  <button class="next-btn" onclick={onNextPuzzle}>Next Puzzle</button>
</div>

<style>
  .solution-panel {
    padding: 16px;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .solution-panel.solved {
    background: var(--cm-status-success-bg-alt);
    border: 1px solid var(--cm-status-success-lighter);
  }

  .solution-panel.failed {
    background: var(--cm-status-error-bg);
    border: 1px solid var(--cm-status-error-lighter);
  }

  .result-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .result-icon {
    font-size: 20px;
    font-weight: 700;
  }

  .correct-icon {
    color: var(--cm-status-success-alt);
  }

  .failed-icon {
    color: var(--cm-status-error);
  }

  .result-text {
    font-size: 16px;
    font-weight: 600;
  }

  .solved .result-text {
    color: var(--cm-status-success-alt);
  }

  .failed .result-text {
    color: var(--cm-status-error);
  }

  .rating-note {
    margin: 0;
    font-size: 13px;
    line-height: 1.5;
    color: var(--cm-text-secondary);
    font-style: italic;
  }

  .explanation {
    font-size: 13px;
    line-height: 1.5;
    color: var(--cm-text-secondary);
  }

  .explanation p {
    margin: 0 0 4px;
  }

  .next-btn {
    padding: 10px;
    background: var(--cm-accent-secondary-hover);
    color: var(--cm-text-inverse);
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 500;
    transition: background 0.15s;
  }

  .next-btn:hover {
    background: var(--cm-accent-secondary-hover);
  }
</style>
