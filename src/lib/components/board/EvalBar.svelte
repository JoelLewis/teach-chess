<script lang="ts">
  import type { Score } from "../../types/engine";
  import { formatScore, scoreToBarValue } from "../../types/engine";

  type Props = {
    score: Score | null;
    orientation?: "white" | "black";
  };

  let { score, orientation = "white" }: Props = $props();

  let barValue = $derived(score ? scoreToBarValue(score) : 0.5);
  let displayValue = $derived(score ? formatScore(score) : "0.0");

  // If viewing from black's perspective, flip the bar
  let whiteHeight = $derived(
    orientation === "white" ? barValue * 100 : (1 - barValue) * 100,
  );

  // Convert bar value (0-1) to centipawn-like range for aria
  let ariaValue = $derived(Math.round((barValue - 0.5) * 20 * 100) / 100);
</script>

<div
  class="eval-bar"
  title={displayValue}
  role="meter"
  aria-valuenow={ariaValue}
  aria-valuemin={-10}
  aria-valuemax={10}
  aria-label="Position evaluation: {displayValue}"
>
  <div class="eval-black" style="height: {100 - whiteHeight}%">
    {#if whiteHeight < 50}
      <span class="eval-label">{displayValue}</span>
    {/if}
  </div>
  <div class="eval-white" style="height: {whiteHeight}%">
    {#if whiteHeight >= 50}
      <span class="eval-label">{displayValue}</span>
    {/if}
  </div>
</div>

<style>
  .eval-bar {
    width: 28px;
    height: min(80vh, 560px);
    display: flex;
    flex-direction: column;
    border-radius: 4px;
    overflow: hidden;
    border: 1px solid var(--cm-border-medium);
    font-size: 11px;
    font-weight: 600;
    user-select: none;
  }

  .eval-black {
    background: var(--cm-eval-bar-black);
    color: var(--cm-text-inverse);
    display: flex;
    align-items: flex-end;
    justify-content: center;
    transition: height 0.3s ease;
  }

  .eval-white {
    background: var(--cm-board-light);
    color: var(--cm-text-primary);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    transition: height 0.3s ease;
  }

  .eval-label {
    padding: 2px 0;
    writing-mode: vertical-lr;
    text-orientation: mixed;
    font-family: var(--cm-font-mono);
  }
</style>
