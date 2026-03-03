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
    color: var(--cm-text-primary);
    margin: 0 0 20px;
  }

  .loading-text {
    color: var(--cm-text-muted);
    text-align: center;
    padding: 40px;
  }

  .empty-state {
    text-align: center;
    padding: 40px 20px;
    color: var(--cm-text-muted);
  }

  .hint-text {
    font-size: 13px;
    color: var(--cm-text-disabled);
    margin-top: 8px;
  }

  .opening-group {
    background: var(--cm-bg-surface);
    border: 1px solid var(--cm-border-light);
    border-radius: 8px;
    margin-bottom: 12px;
    overflow: hidden;
  }

  .group-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: var(--cm-bg-surface-alt);
    border-bottom: 1px solid var(--cm-border-light);
  }

  .group-title {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .eco-badge {
    padding: 2px 8px;
    background: var(--cm-accent-secondary-bg);
    color: var(--cm-accent-secondary-text);
    border-radius: 4px;
    font-size: 12px;
    font-weight: 600;
  }

  .opening-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--cm-text-primary);
  }

  .entry-count {
    font-size: 12px;
    color: var(--cm-text-disabled);
  }

  .drill-btn {
    padding: 6px 16px;
    background: var(--cm-accent-secondary-hover);
    color: var(--cm-text-inverse);
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s;
  }

  .drill-btn:hover {
    background: var(--cm-accent-secondary-hover);
  }

  .entries-list {
    padding: 8px 16px;
  }

  .entry-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 0;
    border-bottom: 1px solid var(--cm-bg-hover);
  }

  .entry-row:last-child {
    border-bottom: none;
  }

  .entry-move {
    font-size: 14px;
    font-weight: 600;
    color: var(--cm-text-primary);
    min-width: 48px;
  }

  .entry-fen {
    font-size: 12px;
    color: var(--cm-text-disabled);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .remove-btn {
    background: none;
    border: none;
    color: var(--cm-border-medium);
    cursor: pointer;
    font-size: 18px;
    padding: 2px 6px;
    transition: color 0.15s;
  }

  .remove-btn:hover {
    color: var(--cm-status-error);
  }
</style>
