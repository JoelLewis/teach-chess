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
    background: white;
    border-bottom: 2px solid #e5e7eb;
    padding: 0 24px;
  }

  .tab {
    padding: 12px 20px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
    color: #6b7280;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
  }

  .tab:hover {
    color: #1e293b;
  }

  .tab.active {
    color: #1e40af;
    border-bottom-color: #1e40af;
  }

  .tab-content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
  }
</style>
