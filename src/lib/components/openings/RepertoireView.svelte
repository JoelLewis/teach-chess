<script lang="ts">
  import { repertoireStore } from "../../stores/repertoire.svelte";
  import { errorStore } from "../../stores/error.svelte";
  import * as api from "../../api/commands";
  import type { Opening, RepertoireEntry } from "../../types/repertoire";

  let openings = $state<Opening[]>([]);
  let entriesByOpening = $state<Map<string, RepertoireEntry[]>>(new Map());
  let loading = $state(true);

  async function loadRepertoire() {
    loading = true;
    try {
      const allOpenings = await api.getOpenings({});
      openings = allOpenings;

      const map = new Map<string, RepertoireEntry[]>();
      for (const opening of allOpenings) {
        const entries = await api.getRepertoire(opening.id);
        if (entries.length > 0) {
          map.set(opening.id, entries);
        }
      }
      entriesByOpening = map;
    } catch (err) {
      errorStore.show(`Failed to load repertoire: ${err}`);
    } finally {
      loading = false;
    }
  }

  async function removeEntry(entryId: string, openingId: string) {
    try {
      await api.removeFromRepertoire(entryId);
      const entries = await api.getRepertoire(openingId);
      if (entries.length > 0) {
        entriesByOpening.set(openingId, entries);
      } else {
        entriesByOpening.delete(openingId);
      }
      entriesByOpening = new Map(entriesByOpening);
    } catch (err) {
      errorStore.show(`Failed to remove entry: ${err}`);
    }
  }

  function startDrill(openingId: string) {
    repertoireStore.activeTab = "drill";
    // Store which opening to drill — DrillScreen will use it
    const opening = openings.find((o) => o.id === openingId);
    if (opening) {
      repertoireStore.selectedOpening = opening;
    }
  }

  $effect(() => {
    loadRepertoire();
  });

  const openingsWithEntries = $derived(
    openings.filter((o) => entriesByOpening.has(o.id)),
  );
</script>

<div class="repertoire-view">
  <h2>My Repertoire</h2>

  {#if loading}
    <p class="loading-text">Loading repertoire...</p>
  {:else if openingsWithEntries.length === 0}
    <div class="empty-state">
      <p>You haven't added any moves to your repertoire yet.</p>
      <p class="hint-text">
        Browse the Library tab, select an opening, and click "Add this move to repertoire" to build your personal opening book.
      </p>
    </div>
  {:else}
    {#each openingsWithEntries as opening}
      {@const entries = entriesByOpening.get(opening.id) ?? []}
      <div class="opening-group">
        <div class="group-header">
          <div class="group-title">
            <span class="eco-badge">{opening.eco}</span>
            <span class="opening-name">{opening.name}</span>
            <span class="entry-count">{entries.length} moves</span>
          </div>
          <button class="drill-btn" onclick={() => startDrill(opening.id)}>
            Drill
          </button>
        </div>

        <div class="entries-list">
          {#each entries as entry}
            <div class="entry-row">
              <span class="entry-move">{entry.moveSan || entry.moveUci}</span>
              <span class="entry-fen" title={entry.positionFen}>
                at {entry.positionFen.split(" ")[0].slice(0, 20)}...
              </span>
              <button
                class="remove-btn"
                onclick={() => removeEntry(entry.id, opening.id)}
                title="Remove from repertoire"
              >
                &times;
              </button>
            </div>
          {/each}
        </div>
      </div>
    {/each}
  {/if}
</div>

<style>
  .repertoire-view h2 {
    font-size: 20px;
    font-weight: 700;
    color: #1e293b;
    margin: 0 0 20px;
  }

  .loading-text {
    color: #6b7280;
    text-align: center;
    padding: 40px;
  }

  .empty-state {
    text-align: center;
    padding: 40px 20px;
    color: #6b7280;
  }

  .hint-text {
    font-size: 13px;
    color: #9ca3af;
    margin-top: 8px;
  }

  .opening-group {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
    margin-bottom: 12px;
    overflow: hidden;
  }

  .group-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #f8fafc;
    border-bottom: 1px solid #e5e7eb;
  }

  .group-title {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .eco-badge {
    padding: 2px 8px;
    background: #f0f9ff;
    color: #0369a1;
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
  }

  .opening-name {
    font-size: 14px;
    font-weight: 600;
    color: #1e293b;
  }

  .entry-count {
    font-size: 12px;
    color: #9ca3af;
  }

  .drill-btn {
    padding: 6px 16px;
    background: #1e40af;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s;
  }

  .drill-btn:hover {
    background: #1e3a8a;
  }

  .entries-list {
    padding: 8px 16px;
  }

  .entry-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 0;
    border-bottom: 1px solid #f3f4f6;
  }

  .entry-row:last-child {
    border-bottom: none;
  }

  .entry-move {
    font-size: 14px;
    font-weight: 600;
    color: #1e293b;
    min-width: 48px;
  }

  .entry-fen {
    font-size: 12px;
    color: #9ca3af;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .remove-btn {
    background: none;
    border: none;
    color: #d1d5db;
    cursor: pointer;
    font-size: 18px;
    padding: 2px 6px;
    transition: color 0.15s;
  }

  .remove-btn:hover {
    color: #dc2626;
  }
</style>
