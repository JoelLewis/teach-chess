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
    background: white;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    overflow: hidden;
  }

  .stats-bar {
    display: flex;
    justify-content: space-around;
    padding: 12px 16px;
    background: #f8fafc;
    border-bottom: 1px solid #e5e7eb;
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
    color: #1e293b;
  }

  .stat-label {
    font-size: 11px;
    color: #6b7280;
    text-transform: uppercase;
  }

  .filter-toggle {
    padding: 6px 16px;
    background: none;
    border: none;
    border-bottom: 1px solid #e5e7eb;
    color: #6b7280;
    font-size: 12px;
    cursor: pointer;
    text-align: left;
  }

  .filter-toggle:hover {
    color: #1e40af;
  }

  .puzzle-info {
    padding: 12px 16px;
    border-bottom: 1px solid #e5e7eb;
  }

  .info-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }

  .info-label {
    font-size: 12px;
    color: #6b7280;
  }

  .info-value {
    font-size: 14px;
    font-weight: 600;
    color: #1e293b;
  }

  .theme-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-bottom: 8px;
  }

  .theme-tag {
    padding: 2px 8px;
    background: #ede9fe;
    color: #5b21b6;
    border-radius: 4px;
    font-size: 11px;
  }

  .move-progress {
    font-size: 13px;
    color: #6b7280;
  }

  .status-message {
    padding: 24px 16px;
    text-align: center;
    color: #6366f1;
    font-size: 14px;
  }

  .idle-message {
    padding: 24px 16px;
    text-align: center;
  }

  .idle-message p {
    color: #6b7280;
    font-size: 14px;
    margin-bottom: 16px;
  }

  .start-btn {
    padding: 12px 24px;
    background: #1e40af;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .start-btn:hover {
    background: #1e3a8a;
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
    color: #1e293b;
  }

  .feedback-text {
    font-size: 14px;
    font-weight: 500;
    color: #dc2626;
  }

  .correct-text {
    color: #059669;
  }

  .abandon-btn {
    padding: 8px;
    background: #fee2e2;
    color: #dc2626;
    border: 1px solid #fca5a5;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    transition: background 0.15s;
    margin-top: auto;
  }

  .abandon-btn:hover {
    background: #fecaca;
  }
</style>
