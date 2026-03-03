<script lang="ts">
  import { puzzleStore } from "../../stores/puzzle.svelte";
  import type { PuzzleCategory } from "../../types/puzzle";

  const categories: { value: PuzzleCategory; label: string }[] = [
    { value: "tactical", label: "Tactical" },
    { value: "positional", label: "Positional" },
    { value: "endgame", label: "Endgame" },
    { value: "opening", label: "Opening" },
    { value: "pattern", label: "Pattern" },
  ];

  function setCategory(cat: PuzzleCategory) {
    puzzleStore.filter = { ...puzzleStore.filter, category: cat };
  }

  function setMinDifficulty(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value);
    puzzleStore.filter = { ...puzzleStore.filter, minDifficulty: val };
  }

  function setMaxDifficulty(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value);
    puzzleStore.filter = { ...puzzleStore.filter, maxDifficulty: val };
  }
</script>

<div class="puzzle-filter">
  <div class="filter-group">
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <label class="filter-label">Category</label>
    <div class="category-buttons">
      {#each categories as cat}
        <button
          class="cat-btn"
          class:active={puzzleStore.filter.category === cat.value}
          onclick={() => setCategory(cat.value)}
        >
          {cat.label}
        </button>
      {/each}
    </div>
  </div>

  <div class="filter-group">
    <!-- svelte-ignore a11y_label_has_associated_control -->
    <label class="filter-label">
      Difficulty: {puzzleStore.filter.minDifficulty ?? 400} – {puzzleStore.filter.maxDifficulty ?? 1600}
    </label>
    <div class="range-inputs">
      <input
        type="range"
        min="400"
        max="2800"
        step="100"
        value={puzzleStore.filter.minDifficulty ?? 400}
        oninput={setMinDifficulty}
        aria-label="Minimum difficulty"
      />
      <input
        type="range"
        min="400"
        max="2800"
        step="100"
        value={puzzleStore.filter.maxDifficulty ?? 1600}
        oninput={setMaxDifficulty}
        aria-label="Maximum difficulty"
      />
    </div>
  </div>
</div>

<style>
  .puzzle-filter {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px 16px;
    border-bottom: 1px solid #e5e7eb;
  }

  .filter-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .filter-label {
    font-size: 12px;
    font-weight: 600;
    color: #6b7280;
    text-transform: uppercase;
  }

  .category-buttons {
    display: flex;
    gap: 4px;
    flex-wrap: wrap;
  }

  .cat-btn {
    padding: 4px 10px;
    font-size: 12px;
    border: 1px solid #d1d5db;
    border-radius: 4px;
    background: white;
    cursor: pointer;
    color: #374151;
    transition: all 0.15s;
  }

  .cat-btn:hover {
    background: #f3f4f6;
  }

  .cat-btn.active {
    background: #1e40af;
    color: white;
    border-color: #1e40af;
  }

  .range-inputs {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .range-inputs input[type="range"] {
    width: 100%;
    accent-color: #1e40af;
  }
</style>
