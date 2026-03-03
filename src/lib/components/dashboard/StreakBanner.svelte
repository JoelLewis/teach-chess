<script lang="ts">
  import type { SessionStreak } from "../../types/dashboard";

  type Props = {
    streak: SessionStreak;
  };

  let { streak }: Props = $props();
</script>

<div class="streak-banner">
  <div class="streak-item">
    <span class="streak-value">{streak.currentDays}</span>
    <span class="streak-label">day streak</span>
  </div>
  <div class="streak-divider"></div>
  <div class="streak-item">
    <span class="streak-value">{streak.gamesToday}</span>
    <span class="streak-label">{streak.gamesToday === 1 ? "game" : "games"} today</span>
  </div>
  <div class="streak-divider"></div>
  <div class="streak-item">
    <span class="streak-value">{streak.puzzlesToday}</span>
    <span class="streak-label">{streak.puzzlesToday === 1 ? "puzzle" : "puzzles"} today</span>
  </div>
  {#if streak.longestDays > streak.currentDays}
    <div class="streak-divider"></div>
    <div class="streak-item muted">
      <span class="streak-value">{streak.longestDays}</span>
      <span class="streak-label">best streak</span>
    </div>
  {/if}
</div>

<style>
  .streak-banner {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 10px 16px;
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 8px;
  }

  .streak-item {
    display: flex;
    align-items: baseline;
    gap: 4px;
  }

  .streak-value {
    font-size: 20px;
    font-weight: 700;
    color: #1e40af;
  }

  .streak-label {
    font-size: 12px;
    color: #6b7280;
  }

  .streak-divider {
    width: 1px;
    height: 24px;
    background: #e5e7eb;
  }

  .muted .streak-value {
    color: #9ca3af;
  }
</style>
