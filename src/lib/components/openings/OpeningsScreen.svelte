<script lang="ts">
  import OpeningLibrary from "./OpeningLibrary.svelte";
  import OpeningDetail from "./OpeningDetail.svelte";
  import RepertoireView from "./RepertoireView.svelte";
  import DrillScreen from "./DrillScreen.svelte";
  import { repertoireStore } from "../../stores/repertoire.svelte";

  const activeTab = $derived(repertoireStore.activeTab);
  const phase = $derived(repertoireStore.phase);

  function setTab(tab: "library" | "repertoire" | "drill") {
    repertoireStore.activeTab = tab;
    if (tab !== "library") {
      repertoireStore.phase = "idle";
    }
  }
</script>

<div class="openings-screen">
  <div class="tabs">
    <button
      class="tab"
      class:active={activeTab === "library"}
      onclick={() => setTab("library")}
    >
      Library
    </button>
    <button
      class="tab"
      class:active={activeTab === "repertoire"}
      onclick={() => setTab("repertoire")}
    >
      My Repertoire
    </button>
    <button
      class="tab"
      class:active={activeTab === "drill"}
      onclick={() => setTab("drill")}
    >
      Drill
    </button>
  </div>

  <div class="tab-content">
    {#if activeTab === "library" && phase === "detail"}
      <OpeningDetail />
    {:else if activeTab === "library"}
      <OpeningLibrary />
    {:else if activeTab === "repertoire"}
      <RepertoireView />
    {:else if activeTab === "drill"}
      <DrillScreen />
    {/if}
  </div>
</div>

<style>
  .openings-screen {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .tabs {
    display: flex;
    background: var(--cm-bg-surface);
    border-bottom: 2px solid var(--cm-border-light);
    padding: 0 24px;
  }

  .tab {
    padding: 12px 20px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    color: var(--cm-text-muted);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .tab:hover {
    color: var(--cm-text-primary);
  }

  .tab.active {
    color: var(--cm-accent-secondary-hover);
    border-bottom-color: var(--cm-accent-secondary-hover);
  }

  .tab-content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }
</style>
