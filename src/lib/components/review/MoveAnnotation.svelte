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
      : "var(--cm-text-muted)",
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
    background: var(--cm-bg-hover);
  }

  .annotation.selected {
    background: var(--cm-accent-secondary-bg-alt);
  }

  .move-num {
    color: var(--cm-text-disabled);
    min-width: 30px;
    font-family: var(--cm-font-mono);
  }

  .san {
    font-weight: 500;
    min-width: 48px;
    font-family: var(--cm-font-mono);
  }

  .badge {
    font-size: 10px;
    color: var(--cm-text-inverse);
    padding: 1px 6px;
    border-radius: 3px;
    text-transform: capitalize;
  }

  .eval-text {
    color: var(--cm-text-muted);
    font-family: var(--cm-font-mono);
    font-size: 12px;
    margin-left: auto;
  }
</style>
