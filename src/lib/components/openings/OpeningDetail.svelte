<script lang="ts">
  import Chessboard from "../board/Chessboard.svelte";
  import { repertoireStore } from "../../stores/repertoire.svelte";
  import { errorStore } from "../../stores/error.svelte";
  import * as api from "../../api/commands";

  const opening = $derived(repertoireStore.selectedOpening);
  const positions = $derived(repertoireStore.openingPositions);

  let currentStep = $state(-1);

  const currentFen = $derived(
    currentStep >= 0 && currentStep < positions.length
      ? positions[currentStep].fen
      : "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
  );

  const lastMove = $derived<[string, string] | null>(
    currentStep >= 0 && currentStep < positions.length
      ? [
          positions[currentStep].moveUci.slice(0, 2),
          positions[currentStep].moveUci.slice(2, 4),
        ]
      : null,
  );

  const moveSans = $derived(positions.map((p) => p.moveSan));

  function goBack() {
    repertoireStore.phase = "browsing";
    repertoireStore.selectedOpening = null;
    repertoireStore.openingPositions = [];
  }

  function stepForward() {
    if (currentStep < positions.length - 1) {
      currentStep += 1;
    }
  }

  function stepBack() {
    if (currentStep >= 0) {
      currentStep -= 1;
    }
  }

  function goToStep(idx: number) {
    currentStep = idx;
  }

  async function addToRepertoire(posIdx: number) {
    if (!opening) return;
    const pos = positions[posIdx];
    const parentFen =
      posIdx > 0
        ? positions[posIdx - 1].fen
        : "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    try {
      await api.addToRepertoire(
        opening.id,
        parentFen,
        pos.moveUci,
        pos.moveSan,
      );
      // Refresh entries
      repertoireStore.repertoireEntries = await api.getRepertoire(opening.id);
    } catch (err) {
      errorStore.show(`Failed to add to repertoire: ${err}`);
    }
  }
</script>

{#if opening}
  <div class="detail">
    <div class="detail-header">
      <button class="back-btn" onclick={goBack}>
        &larr; Back to Library
      </button>
      <div class="opening-title">
        <span class="eco">{opening.eco}</span>
        <h2>{opening.name}</h2>
      </div>
    </div>

    <div class="detail-layout">
      <div class="board-area">
        <Chessboard
          fen={currentFen}
          orientation={opening.color}
          viewOnly={true}
          {lastMove}
        />
        <div class="board-controls">
          <button onclick={() => goToStep(-1)} disabled={currentStep < 0}>
            &laquo;
          </button>
          <button onclick={stepBack} disabled={currentStep < 0}>
            &lsaquo;
          </button>
          <span class="step-label">
            {currentStep + 1} / {positions.length}
          </span>
          <button onclick={stepForward} disabled={currentStep >= positions.length - 1}>
            &rsaquo;
          </button>
          <button onclick={() => goToStep(positions.length - 1)} disabled={currentStep >= positions.length - 1}>
            &raquo;
          </button>
        </div>
      </div>

      <div class="info-panel">
        <p class="description">{opening.description}</p>

        <div class="move-list">
          <h3>Moves</h3>
          <div class="moves">
            {#each moveSans as san, i}
              <button
                class="move-btn"
                class:active={i === currentStep}
                onclick={() => goToStep(i)}
              >
                {#if i % 2 === 0}
                  <span class="move-num">{Math.floor(i / 2) + 1}.</span>
                {/if}
                {san}
              </button>
            {/each}
          </div>
        </div>

        {#if currentStep >= 0}
          <button
            class="add-repertoire-btn"
            onclick={() => addToRepertoire(currentStep)}
          >
            Add this move to repertoire
          </button>
        {/if}

        <div class="meta">
          <div class="meta-item">
            <span class="meta-label">Difficulty</span>
            <span class="meta-value">{opening.difficulty}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">Color</span>
            <span class="meta-value">{opening.color}</span>
          </div>
          {#if opening.themes}
            <div class="meta-item">
              <span class="meta-label">Themes</span>
              <span class="meta-value">{opening.themes}</span>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .detail-header {
    margin-bottom: 20px;
  }

  .back-btn {
    background: none;
    border: none;
    color: #3b82f6;
    cursor: pointer;
    font-size: 14px;
    padding: 0;
    margin-bottom: 8px;
  }

  .back-btn:hover {
    text-decoration: underline;
  }

  .opening-title {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .eco {
    padding: 4px 10px;
    background: #f0f9ff;
    color: #0369a1;
    border-radius: 4px;
    font-size: 13px;
    font-weight: 600;
  }

  .opening-title h2 {
    font-size: 22px;
    font-weight: 700;
    color: #1e293b;
    margin: 0;
  }

  .detail-layout {
    display: flex;
    gap: 24px;
    align-items: flex-start;
  }

  .board-area {
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: center;
  }

  .board-controls {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .board-controls button {
    padding: 6px 12px;
    background: white;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
    color: #374151;
    transition: all 0.15s;
  }

  .board-controls button:hover:not(:disabled) {
    background: #f3f4f6;
    border-color: #9ca3af;
  }

  .board-controls button:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .step-label {
    font-size: 13px;
    color: #6b7280;
    min-width: 60px;
    text-align: center;
  }

  .info-panel {
    flex: 1;
    max-width: 320px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .description {
    font-size: 14px;
    color: #4b5563;
    line-height: 1.5;
    margin: 0;
  }

  .move-list h3 {
    font-size: 14px;
    font-weight: 600;
    color: #1e293b;
    margin: 0 0 8px;
  }

  .moves {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .move-btn {
    padding: 4px 8px;
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    color: #374151;
    transition: all 0.15s;
  }

  .move-btn:hover {
    background: #f3f4f6;
  }

  .move-btn.active {
    background: #1e40af;
    color: white;
    border-color: #1e40af;
  }

  .move-num {
    color: #9ca3af;
    margin-right: 2px;
  }

  .move-btn.active .move-num {
    color: rgba(255, 255, 255, 0.7);
  }

  .add-repertoire-btn {
    padding: 10px 16px;
    background: #059669;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s;
  }

  .add-repertoire-btn:hover {
    background: #047857;
  }

  .meta {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 12px;
    border-top: 1px solid #e5e7eb;
  }

  .meta-item {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
  }

  .meta-label {
    color: #6b7280;
  }

  .meta-value {
    color: #1e293b;
    font-weight: 500;
  }
</style>
