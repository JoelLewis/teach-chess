<script lang="ts">
  import { repertoireStore } from "../../stores/repertoire.svelte";
  import { errorStore } from "../../stores/error.svelte";
  import * as api from "../../api/commands";

  let colorFilter = $state<string | undefined>(undefined);
  const openings = $derived(repertoireStore.openings);

  async function loadOpenings() {
    try {
      const result = await api.getOpenings({
        color: colorFilter,
      });
      repertoireStore.openings = result;
    } catch (err) {
      errorStore.show(`Failed to load openings: ${err}`);
    }
  }

  async function selectOpening(openingId: string) {
    try {
      const [opening, positions] = await api.getOpeningDetail(openingId);
      repertoireStore.selectedOpening = opening;
      repertoireStore.openingPositions = positions;
      repertoireStore.phase = "detail";
    } catch (err) {
      errorStore.show(`Failed to load opening: ${err}`);
    }
  }

  $effect(() => {
    loadOpenings();
  });
</script>

<div class="library">
  <div class="library-header">
    <h2>Opening Library</h2>
    <div class="color-filter">
      <button
        class="filter-btn"
        class:active={colorFilter === undefined}
        onclick={() => { colorFilter = undefined; loadOpenings(); }}
      >All</button>
      <button
        class="filter-btn"
        class:active={colorFilter === "white"}
        onclick={() => { colorFilter = "white"; loadOpenings(); }}
      >White</button>
      <button
        class="filter-btn"
        class:active={colorFilter === "black"}
        onclick={() => { colorFilter = "black"; loadOpenings(); }}
      >Black</button>
    </div>
  </div>

  <div class="openings-grid">
    {#each openings as opening}
      <button class="opening-card" onclick={() => selectOpening(opening.id)}>
        <div class="card-header">
          <span class="eco-badge">{opening.eco}</span>
          <span class="color-dot" class:white={opening.color === "white"} class:black={opening.color === "black"}></span>
        </div>
        <div class="card-name">{opening.name}</div>
        <div class="card-desc">{opening.description}</div>
        <div class="card-footer">
          <span class="difficulty">Rating ~{opening.difficulty}</span>
          {#if opening.themes}
            <span class="themes">{opening.themes.split(",").slice(0, 2).join(", ")}</span>
          {/if}
        </div>
      </button>
    {/each}

    {#if openings.length === 0}
      <p class="empty-text">No openings found. Try changing the filter.</p>
    {/if}
  </div>
</div>

<style>
  .library-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }

  .library-header h2 {
    font-size: 20px;
    font-weight: 700;
    color: #1e293b;
    margin: 0;
  }

  .color-filter {
    display: flex;
    gap: 4px;
  }

  .filter-btn {
    padding: 6px 14px;
    border: 1px solid #e5e7eb;
    border-radius: 6px;
    background: white;
    color: #6b7280;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .filter-btn:hover {
    border-color: #3b82f6;
    color: #1e40af;
  }

  .filter-btn.active {
    background: #1e40af;
    color: white;
    border-color: #1e40af;
  }

  .openings-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 16px;
  }

  .opening-card {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    padding: 16px;
    cursor: pointer;
    text-align: left;
    transition: all 0.15s;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .opening-card:hover {
    border-color: #3b82f6;
    box-shadow: 0 2px 8px rgba(59, 130, 246, 0.1);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .eco-badge {
    padding: 2px 8px;
    background: #f0f9ff;
    color: #0369a1;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
  }

  .color-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 1px solid #d1d5db;
  }

  .color-dot.white {
    background: white;
  }

  .color-dot.black {
    background: #1e293b;
  }

  .card-name {
    font-size: 15px;
    font-weight: 600;
    color: #1e293b;
  }

  .card-desc {
    font-size: 13px;
    color: #6b7280;
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: auto;
    padding-top: 8px;
    border-top: 1px solid #f3f4f6;
  }

  .difficulty {
    font-size: 12px;
    color: #6b7280;
  }

  .themes {
    font-size: 11px;
    color: #8b5cf6;
  }

  .empty-text {
    color: #9ca3af;
    text-align: center;
    grid-column: 1 / -1;
    padding: 40px;
  }
</style>
