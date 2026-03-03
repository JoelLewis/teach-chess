<script lang="ts">
  import { repertoireStore } from "../../stores/repertoire.svelte";

  type Props = {
    onStartDrill: () => void;
    onNextDrill: () => void;
  };

  let { onStartDrill, onNextDrill }: Props = $props();

  const phase = $derived(repertoireStore.phase);
  const drillState = $derived(repertoireStore.drillState);
  const lastResult = $derived(repertoireStore.lastDrillResult);
  const stats = $derived(repertoireStore.drillStats);
  const opening = $derived(repertoireStore.selectedOpening);
</script>

<div class="drill-panel">
  <!-- Stats bar -->
  {#if stats}
    <div class="stats-bar">
      <div class="stat">
        <span class="stat-value">{stats.totalCorrect}</span>
        <span class="stat-label">Correct</span>
      </div>
      <div class="stat">
        <span class="stat-value">{repertoireStore.drillAccuracy}</span>
        <span class="stat-label">Accuracy</span>
      </div>
      <div class="stat">
        <span class="stat-value">{stats.currentStreak}</span>
        <span class="stat-label">Streak</span>
      </div>
    </div>
  {/if}

  {#if phase === "idle" || phase === "browsing"}
    <div class="idle-section">
      {#if opening}
        <div class="selected-opening">
          <span class="opening-label">Selected:</span>
          <span class="opening-name">{opening.name}</span>
        </div>
        <button class="start-btn" onclick={onStartDrill}>
          Start Drill
        </button>
      {:else}
        <p class="hint-text">
          Select an opening from My Repertoire to start drilling.
        </p>
      {/if}
    </div>
  {:else if phase === "drilling"}
    <div class="drilling-section">
      {#if drillState}
        <div class="drill-info">
          <div class="opening-name">{drillState.opening.name}</div>
          <div class="progress">
            Move {drillState.entriesTotal - drillState.entriesRemaining + 1} of {drillState.entriesTotal}
          </div>
        </div>
        <div class="turn-indicator">
          Your turn — play the repertoire move!
        </div>
      {/if}
    </div>
  {:else if phase === "drill-result"}
    <div class="result-section">
      {#if lastResult}
        {#if lastResult.correct}
          <div class="feedback correct">Correct!</div>
        {:else}
          <div class="feedback incorrect">
            Not quite.
            {#if lastResult.correctMove}
              <div class="correct-answer">
                Correct: {lastResult.correctMove}
              </div>
            {/if}
          </div>
          {#if lastResult.explanation}
            <div class="explanation">{lastResult.explanation}</div>
          {/if}
        {/if}

        {#if lastResult.isComplete}
          <div class="complete-message">Drill complete!</div>
          <button class="start-btn" onclick={onNextDrill}>
            Drill Again
          </button>
        {:else if !lastResult.correct}
          <button class="next-btn" onclick={onNextDrill}>
            Continue
          </button>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .drill-panel {
    display: flex;
    flex-direction: column;
    width: 280px;
    background: var(--cm-bg-surface);
    border-radius: 8px;
    box-shadow: var(--cm-shadow-sm);
    overflow: hidden;
  }

  .stats-bar {
    display: flex;
    justify-content: space-around;
    padding: 12px 16px;
    background: var(--cm-bg-surface-alt);
    border-bottom: 1px solid var(--cm-border-light);
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .stat-value {
    font-size: 18px;
    font-weight: 700;
    color: var(--cm-text-primary);
  }

  .stat-label {
    font-size: 11px;
    color: var(--cm-text-muted);
    text-transform: uppercase;
  }

  .idle-section,
  .drilling-section,
  .result-section {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .selected-opening {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .opening-label {
    font-size: 12px;
    color: var(--cm-text-muted);
  }

  .opening-name {
    font-size: 15px;
    font-weight: 600;
    color: var(--cm-text-primary);
  }

  .hint-text {
    font-size: 14px;
    color: var(--cm-text-muted);
    margin: 0;
    text-align: center;
  }

  .start-btn {
    padding: 12px 24px;
    background: var(--cm-accent-secondary-hover);
    color: var(--cm-text-inverse);
    border: none;
    border-radius: 8px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .start-btn:hover {
    background: var(--cm-accent-secondary-hover);
  }

  .drill-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .progress {
    font-size: 13px;
    color: var(--cm-text-muted);
  }

  .turn-indicator {
    font-size: 14px;
    font-weight: 500;
    color: var(--cm-text-primary);
  }

  .feedback {
    font-size: 16px;
    font-weight: 600;
    padding: 12px;
    border-radius: 8px;
    text-align: center;
  }

  .feedback.correct {
    background: var(--cm-status-success-bg-alt);
    color: var(--cm-status-success-alt);
  }

  .feedback.incorrect {
    background: var(--cm-status-error-bg);
    color: var(--cm-status-error);
  }

  .correct-answer {
    font-size: 14px;
    font-weight: 500;
    margin-top: 8px;
  }

  .explanation {
    font-size: 13px;
    color: var(--cm-text-tertiary);
    line-height: 1.4;
  }

  .complete-message {
    font-size: 15px;
    font-weight: 600;
    color: var(--cm-status-success-alt);
    text-align: center;
  }

  .next-btn {
    padding: 10px 20px;
    background: var(--cm-bg-hover);
    color: var(--cm-text-secondary);
    border: 1px solid var(--cm-border-medium);
    border-radius: 6px;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .next-btn:hover {
    background: var(--cm-bg-active);
  }
</style>
