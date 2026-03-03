<script lang="ts">
  import { puzzleStore } from "../../stores/puzzle.svelte";
  import { errorStore } from "../../stores/error.svelte";
  import * as api from "../../api/commands";
  import PuzzleHintSystem from "./PuzzleHintSystem.svelte";
  import PuzzleSolution from "./PuzzleSolution.svelte";
  import PuzzleFilter from "./PuzzleFilter.svelte";
  import SkillProfilePanel from "../assessment/SkillProfilePanel.svelte";

  type Props = {
    onLoadNext: () => void;
  };

  let { onLoadNext }: Props = $props();

  let showFilter = $state(false);

  const phase = $derived(puzzleStore.phase);
  const themes = $derived(puzzleStore.themes);
  const difficulty = $derived(puzzleStore.difficulty);
  const stats = $derived(puzzleStore.sessionStats);

  async function handleAbandon() {
    try {
      const result = await api.abandonPuzzle();
      puzzleStore.explanation = result.explanation ?? null;
      puzzleStore.phase = "abandoned";
    } catch (err) {
      errorStore.show(`Failed to abandon puzzle: ${err}`);
    }
  }

  function handleNextPuzzle() {
    puzzleStore.reset();
    onLoadNext();
  }
</script>

<div class="problem-panel">
  <!-- Stats bar -->
  {#if stats}
    <div class="stats-bar">
      <div class="stat">
        <span class="stat-value">{stats.totalSolved}</span>
        <span class="stat-label">Solved</span>
      </div>
      <div class="stat">
        <span class="stat-value">{puzzleStore.solveRate}</span>
        <span class="stat-label">Rate</span>
      </div>
      <div class="stat">
        <span class="stat-value">{stats.currentStreak}</span>
        <span class="stat-label">Streak</span>
      </div>
    </div>
  {/if}

  <!-- Filter toggle -->
  <button class="filter-toggle" onclick={() => (showFilter = !showFilter)}>
    {showFilter ? "Hide Filters" : "Filters"}
  </button>
  {#if showFilter}
    <PuzzleFilter />
  {/if}

  <!-- Puzzle info -->
  {#if puzzleStore.currentPuzzle}
    <div class="puzzle-info">
      <div class="info-row">
        <span class="info-label">Rating</span>
        <span class="info-value">{difficulty}</span>
      </div>
      {#if themes.length > 0}
        <div class="theme-tags">
          {#each themes as theme}
            <span class="theme-tag">{theme}</span>
          {/each}
        </div>
      {/if}
      <div class="move-progress">
        Move {(puzzleStore.lastMoveResult?.currentMoveIndex ?? 0) + 1} of {puzzleStore.currentPuzzle.totalPlayerMoves}
      </div>
    </div>
  {/if}

  <!-- Phase-specific content -->
  {#if phase === "loading"}
    <div class="status-message">Loading puzzle...</div>
  {:else if phase === "idle"}
    <div class="idle-message">
      <p>Solve tactical puzzles to sharpen your chess skills.</p>
      <button class="start-btn" onclick={onLoadNext}>Start Solving</button>
    </div>
  {:else if phase === "solving"}
    <div class="solving-section">
      <div class="turn-indicator">Your turn — find the best move!</div>
      <PuzzleHintSystem />
      <button class="abandon-btn" onclick={handleAbandon}>Give Up</button>
    </div>
  {:else if phase === "incorrect"}
    <div class="incorrect-feedback">
      <div class="feedback-text">That's not right. Try again!</div>
      <PuzzleHintSystem />
      <button class="abandon-btn" onclick={handleAbandon}>Give Up</button>
    </div>
  {:else if phase === "correct"}
    <div class="correct-feedback">
      <div class="feedback-text correct-text">Correct! Keep going...</div>
    </div>
  {:else if phase === "complete" || phase === "abandoned"}
    <PuzzleSolution onNextPuzzle={handleNextPuzzle} />
  {/if}

  <SkillProfilePanel />
</div>

<style>
  .problem-panel {
    display: flex;
    flex-direction: column;
    width: 280px;
    background: var(--cm-bg-surface);
    border-radius: 8px;
    box-shadow: var(--cm-shadow-sm);
    overflow: hidden;
  }

  .stats-bar {
    display: flex;
    justify-content: space-around;
    padding: 12px 16px;
    background: var(--cm-bg-surface-alt);
    border-bottom: 1px solid var(--cm-border-light);
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .stat-value {
    font-size: 18px;
    font-weight: 700;
    color: var(--cm-text-primary);
  }

  .stat-label {
    font-size: 11px;
    color: var(--cm-text-muted);
    text-transform: uppercase;
  }

  .filter-toggle {
    padding: 6px 16px;
    background: none;
    border: none;
    border-bottom: 1px solid var(--cm-border-light);
    color: var(--cm-text-muted);
    font-size: 12px;
    cursor: pointer;
    text-align: left;
  }

  .filter-toggle:hover {
    color: var(--cm-accent-secondary-hover);
  }

  .puzzle-info {
    padding: 12px 16px;
    border-bottom: 1px solid var(--cm-border-light);
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .info-label {
    font-size: 12px;
    color: var(--cm-text-muted);
  }

  .info-value {
    font-size: 14px;
    font-weight: 600;
    color: var(--cm-text-primary);
  }

  .theme-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-bottom: 8px;
  }

  .theme-tag {
    padding: 2px 8px;
    background: var(--cm-accent-violet-bg-alt);
    color: var(--cm-accent-violet-dark);
    border-radius: 4px;
    font-size: 11px;
  }

  .move-progress {
    font-size: 13px;
    color: var(--cm-text-muted);
  }

  .status-message {
    padding: 24px 16px;
    text-align: center;
    color: var(--cm-accent-primary-light);
    font-size: 14px;
  }

  .idle-message {
    padding: 24px 16px;
    text-align: center;
  }

  .idle-message p {
    color: var(--cm-text-muted);
    font-size: 14px;
    margin-bottom: 16px;
  }

  .start-btn {
    padding: 12px 24px;
    background: var(--cm-accent-secondary);
    color: var(--cm-text-inverse);
    border: none;
    border-radius: 8px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .start-btn:hover {
    background: var(--cm-accent-secondary-hover);
  }

  .solving-section,
  .incorrect-feedback,
  .correct-feedback {
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .turn-indicator {
    font-size: 14px;
    font-weight: 500;
    color: var(--cm-text-primary);
  }

  .feedback-text {
    font-size: 14px;
    font-weight: 500;
    color: var(--cm-status-error);
  }

  .correct-text {
    color: var(--cm-status-success-alt);
  }

  .abandon-btn {
    padding: 8px;
    background: var(--cm-status-error-bg-alt);
    color: var(--cm-status-error);
    border: 1px solid var(--cm-status-error-lighter);
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    transition: background 0.15s;
    margin-top: auto;
  }

  .abandon-btn:hover {
    background: var(--cm-status-error-muted);
  }
</style>
