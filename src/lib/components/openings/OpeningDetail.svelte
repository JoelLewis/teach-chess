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
    color: var(--cm-accent-secondary);
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
    background: var(--cm-accent-secondary-bg);
    color: var(--cm-accent-secondary-text);
    border-radius: 4px;
    font-size: 13px;
    font-weight: 600;
  }

  .opening-title h2 {
    font-size: 22px;
    font-weight: 700;
    color: var(--cm-text-primary);
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
    background: var(--cm-bg-surface);
    border: 1px solid var(--cm-border-medium);
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
    color: var(--cm-text-secondary);
    transition: all 0.15s;
  }

  .board-controls button:hover:not(:disabled) {
    background: var(--cm-bg-hover);
    border-color: var(--cm-text-disabled);
  }

  .board-controls button:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .step-label {
    font-size: 13px;
    color: var(--cm-text-muted);
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
    color: var(--cm-text-tertiary);
    line-height: 1.5;
    margin: 0;
  }

  .move-list h3 {
    font-size: 14px;
    font-weight: 600;
    color: var(--cm-text-primary);
    margin: 0 0 8px;
  }

  .moves {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .move-btn {
    padding: 4px 8px;
    background: var(--cm-bg-surface);
    border: 1px solid var(--cm-border-light);
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
    color: var(--cm-text-secondary);
    transition: all 0.15s;
  }

  .move-btn:hover {
    background: var(--cm-bg-hover);
  }

  .move-btn.active {
    background: var(--cm-accent-secondary-hover);
    color: var(--cm-text-inverse);
    border-color: var(--cm-accent-secondary-hover);
  }

  .move-num {
    color: var(--cm-text-disabled);
    margin-right: 2px;
  }

  .move-btn.active .move-num {
    color: var(--cm-text-inverse-muted);
  }

  .add-repertoire-btn {
    padding: 10px 16px;
    background: var(--cm-status-success-alt);
    color: var(--cm-text-inverse);
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.15s;
  }

  .add-repertoire-btn:hover {
    background: var(--cm-status-success-hover);
  }

  .meta {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 12px;
    border-top: 1px solid var(--cm-border-light);
  }

  .meta-item {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
  }

  .meta-label {
    color: var(--cm-text-muted);
  }

  .meta-value {
    color: var(--cm-text-primary);
    font-weight: 500;
  }
</style>
