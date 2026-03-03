<script lang="ts">
  import Chessboard from "../board/Chessboard.svelte";
  import DrillPanel from "./DrillPanel.svelte";
  import { repertoireStore } from "../../stores/repertoire.svelte";
  import { errorStore } from "../../stores/error.svelte";
  import * as api from "../../api/commands";

  const drillState = $derived(repertoireStore.drillState);
  const phase = $derived(repertoireStore.phase);
  const fen = $derived(repertoireStore.drillFen);
  const orientation = $derived(repertoireStore.drillColor);
  const canMove = $derived(phase === "drilling");
  const dests = $derived(canMove ? repertoireStore.drillDests : {});
  const turnColor = $derived(canMove ? orientation : undefined);

  async function startDrill() {
    const opening = repertoireStore.selectedOpening;
    if (!opening) {
      errorStore.show("Select an opening to drill from My Repertoire");
      return;
    }

    try {
      const state = await api.startRepertoireDrill(opening.id);
      repertoireStore.drillState = state;
      repertoireStore.lastDrillResult = null;
      repertoireStore.phase = "drilling";

      const stats = await api.getDrillStats();
      repertoireStore.drillStats = stats;
    } catch (err) {
      errorStore.show(`Failed to start drill: ${err}`);
    }
  }

  async function handleMove(from: string, to: string) {
    if (!canMove) return;

    const uci = `${from}${to}`;
    try {
      const result = await api.submitDrillMove(uci);
      repertoireStore.lastDrillResult = result;

      if (result.correct) {
        if (result.isComplete) {
          repertoireStore.phase = "drill-result";
          const stats = await api.getDrillStats();
          repertoireStore.drillStats = stats;
        } else {
          // Brief correct feedback, then update state
          repertoireStore.phase = "drill-result";
          setTimeout(async () => {
            // The backend already advanced to the next entry,
            // so we need to refresh the drill state
            try {
              const state = await api.startRepertoireDrill(
                drillState?.opening.id ?? "",
              );
              repertoireStore.drillState = state;
              repertoireStore.lastDrillResult = null;
              repertoireStore.phase = "drilling";
            } catch {
              // Drill might be complete
              repertoireStore.phase = "idle";
            }
          }, 1000);
        }
      } else {
        // Show incorrect feedback with correct answer
        repertoireStore.phase = "drill-result";
      }
    } catch (err) {
      errorStore.show(`Move error: ${err}`);
    }
  }

  function handleNextDrill() {
    repertoireStore.reset();
    startDrill();
  }

  $effect(() => {
    api.getDrillStats().then((stats) => {
      repertoireStore.drillStats = stats;
    }).catch(() => {});
  });
</script>

<div class="drill-screen">
  {#if phase === "drilling" || phase === "drill-result"}
    <div class="board-area">
      <Chessboard
        fen={fen ?? undefined}
        {orientation}
        {turnColor}
        {dests}
        viewOnly={!canMove}
        onMove={handleMove}
      />
    </div>
  {:else}
    <div class="board-area">
      <Chessboard viewOnly={true} />
    </div>
  {/if}

  <DrillPanel onStartDrill={startDrill} onNextDrill={handleNextDrill} />
</div>

<style>
  .drill-screen {
    display: flex;
    gap: 24px;
    align-items: flex-start;
    justify-content: center;
  }

  .board-area {
    display: flex;
    gap: 8px;
    align-items: center;
  }
</style>
