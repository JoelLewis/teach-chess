<script lang="ts">
  import type { DailyRecommendation } from "../../types/dashboard";

  type Props = {
    recommendation: DailyRecommendation;
    onAction: (activity: string, category: string | null) => void;
  };

  let { recommendation, onAction }: Props = $props();

  const activityLabels: Record<string, string> = {
    problems: "Start Puzzles",
    play: "Play a Game",
    openings: "Practice Openings",
  };
</script>

<div class="recommendation-card">
  <div class="rec-header">Today's Suggestion</div>
  <p class="rec-text">{recommendation.text}</p>
  <button
    class="rec-action"
    onclick={() => onAction(recommendation.targetActivity, recommendation.targetCategory)}
  >
    {activityLabels[recommendation.targetActivity] ?? "Go"}
  </button>
</div>

<style>
  .recommendation-card {
    background: linear-gradient(135deg, #eef2ff 0%, #faf5ff 100%);
    border: 1px solid #c7d2fe;
    border-radius: 10px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .rec-header {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #6366f1;
  }

  .rec-text {
    font-size: 14px;
    color: #374151;
    line-height: 1.5;
    margin: 0;
  }

  .rec-action {
    align-self: flex-start;
    padding: 8px 20px;
    background: #6366f1;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .rec-action:hover {
    background: #4f46e5;
  }
</style>
