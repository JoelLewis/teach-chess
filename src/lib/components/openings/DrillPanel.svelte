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
    background: white;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    overflow: hidden;
  }

  .stats-bar {
    display: flex;
    justify-content: space-around;
    padding: 12px 16px;
    background: #f8fafc;
    border-bottom: 1px solid #e5e7eb;
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
    color: #1e293b;
  }

  .stat-label {
    font-size: 11px;
    color: #6b7280;
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
    color: #6b7280;
  }

  .opening-name {
    font-size: 15px;
    font-weight: 600;
    color: #1e293b;
  }

  .hint-text {
    font-size: 14px;
    color: #6b7280;
    margin: 0;
    text-align: center;
  }

  .start-btn {
    padding: 12px 24px;
    background: #1e40af;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .start-btn:hover {
    background: #1e3a8a;
  }

  .drill-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .progress {
    font-size: 13px;
    color: #6b7280;
  }

  .turn-indicator {
    font-size: 14px;
    font-weight: 500;
    color: #1e293b;
  }

  .feedback {
    font-size: 16px;
    font-weight: 600;
    padding: 12px;
    border-radius: 8px;
    text-align: center;
  }

  .feedback.correct {
    background: #ecfdf5;
    color: #059669;
  }

  .feedback.incorrect {
    background: #fef2f2;
    color: #dc2626;
  }

  .correct-answer {
    font-size: 14px;
    font-weight: 500;
    margin-top: 8px;
  }

  .explanation {
    font-size: 13px;
    color: #4b5563;
    line-height: 1.4;
  }

  .complete-message {
    font-size: 15px;
    font-weight: 600;
    color: #059669;
    text-align: center;
  }

  .next-btn {
    padding: 10px 20px;
    background: #f3f4f6;
    color: #374151;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.15s;
  }

  .next-btn:hover {
    background: #e5e7eb;
  }
</style>
