<script lang="ts">
  import type { MoveEvaluation } from "../../types/engine";
  import { formatScore, CLASSIFICATION_COLORS } from "../../types/engine";

  type Props = {
    evaluation: MoveEvaluation;
    isSelected: boolean;
    onClick: () => void;
  };

  let { evaluation, isSelected, onClick }: Props = $props();

  let classColor = $derived(
    evaluation.classification
      ? CLASSIFICATION_COLORS[evaluation.classification]
      : "#888",
  );

  let evalText = $derived(
    evaluation.evalAfter ? formatScore(evaluation.evalAfter) : "",
  );
</script>

<button
  class="annotation"
  class:selected={isSelected}
  onclick={onClick}
>
  <span class="move-num">
    {#if evaluation.isWhite}
      {evaluation.moveNumber}.
    {:else}
      {evaluation.moveNumber}...
    {/if}
  </span>
  <span class="san">{evaluation.playerMoveSan}</span>
  {#if evaluation.classification}
    <span class="badge" style="background: {classColor}">
      {evaluation.classification}
    </span>
  {/if}
  <span class="eval-text">{evalText}</span>
</button>

<style>
  .annotation {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border: none;
    background: none;
    cursor: pointer;
    border-radius: 4px;
    width: 100%;
    text-align: left;
    font-size: 13px;
  }

  .annotation:hover {
    background: #f3f4f6;
  }

  .annotation.selected {
    background: #dbeafe;
  }

  .move-num {
    color: #9ca3af;
    min-width: 30px;
    font-family: monospace;
  }

  .san {
    font-weight: 500;
    min-width: 48px;
    font-family: monospace;
  }

  .badge {
    font-size: 10px;
    color: white;
    padding: 1px 6px;
    border-radius: 3px;
    text-transform: capitalize;
  }

  .eval-text {
    color: #6b7280;
    font-family: monospace;
    font-size: 12px;
    margin-left: auto;
  }
</style>
