<script lang="ts">
  import SkillRadarChart from "./SkillRadarChart.svelte";
  import RecommendationCard from "./RecommendationCard.svelte";
  import StreakBanner from "./StreakBanner.svelte";
  import AdaptivePromptDialog from "./AdaptivePromptDialog.svelte";
  import GameCard from "../history/GameCard.svelte";
  import * as api from "../../api/commands";
  import { errorStore } from "../../stores/error.svelte";
  import { playerStore } from "../../stores/player.svelte";
  import type { DashboardData, AdaptivePrompt } from "../../types/dashboard";

  type Props = {
    onNavigate: (page: string) => void;
    onReview: (gameId: string) => void;
  };

  let { onNavigate, onReview }: Props = $props();

  let data = $state<DashboardData | null>(null);
  let loading = $state(true);
  let adaptivePrompt = $state<AdaptivePrompt | null>(null);

  async function loadDashboard() {
    loading = true;
    try {
      data = await api.getDashboardData();
    } catch (err) {
      console.error("Failed to load dashboard:", err);
      errorStore.show(`Failed to load dashboard: ${err}`);
    } finally {
      loading = false;
    }
  }

  async function checkAdaptive() {
    try {
      const prompt = await api.checkAdaptiveDifficulty();
      if (prompt.promptType !== "none") {
        // Delay showing the prompt so the dashboard renders first
        setTimeout(() => {
          adaptivePrompt = prompt;
        }, 1000);
      }
    } catch {
      // Non-blocking
    }
  }

  function handleRecommendationAction(activity: string, _category: string | null) {
    onNavigate(activity === "play" ? "play" : activity === "openings" ? "openings" : "problems");
  }

  function handleAdaptiveAccept(activity: string, _category: string | null) {
    adaptivePrompt = null;
    onNavigate(activity === "play" ? "play" : activity === "openings" ? "openings" : "problems");
  }

  $effect(() => {
    loadDashboard();
    checkAdaptive();
  });
</script>

{#if loading}
  <div class="dashboard-loading">
    <p class="loading-text">Loading dashboard...</p>
  </div>
{:else if data}
  <div class="dashboard">
    <!-- Welcome & Streak -->
    <div class="dashboard-header">
      <h1 class="welcome">Welcome back, {playerStore.displayName}</h1>
      <StreakBanner streak={data.streak} />
    </div>

    <!-- Skill Overview + Recommendation -->
    <div class="dashboard-grid">
      <div class="card skill-card">
        <div class="card-header">Skill Overview</div>
        {#if data.skillProfile.ratings.length > 0}
          <SkillRadarChart ratings={data.skillProfile.ratings} />
          <div class="overall-rating">
            Overall: <strong>{Math.round(data.skillProfile.overallRating)}</strong>
          </div>
        {:else}
          <div class="empty-skill">
            <p>No skill data yet.</p>
            <p class="empty-hint">Solve puzzles to build your profile.</p>
          </div>
        {/if}
      </div>

      <RecommendationCard
        recommendation={data.dailyRecommendation}
        onAction={handleRecommendationAction}
      />
    </div>

    <!-- Quick Start -->
    <div class="card">
      <div class="card-header">Quick Start</div>
      <div class="quick-start">
        <button class="qs-btn qs-play" onclick={() => onNavigate("play")}>
          Play a Game
        </button>
        <button class="qs-btn qs-puzzles" onclick={() => onNavigate("problems")}>
          Puzzles
        </button>
        <button class="qs-btn qs-openings" onclick={() => onNavigate("openings")}>
          Openings
        </button>
      </div>
    </div>

    <!-- Recent Games -->
    {#if data.recentGames.length > 0}
      <div class="card">
        <div class="card-header">
          Recent Games
          <button class="view-all" onclick={() => onNavigate("history")}>View All</button>
        </div>
        <div class="recent-games">
          {#each data.recentGames as game (game.id)}
            <GameCard {game} {onReview} />
          {/each}
        </div>
      </div>
    {/if}

    <!-- Puzzle Stats -->
    {#if data.puzzleStats.totalAttempts > 0}
      <div class="card">
        <div class="card-header">Puzzle Progress</div>
        <div class="puzzle-stats">
          <div class="stat">
            <span class="stat-value">{data.puzzleStats.totalSolved}</span>
            <span class="stat-label">solved</span>
          </div>
          <div class="stat">
            <span class="stat-value">
              {data.puzzleStats.totalAttempts > 0
                ? Math.round((data.puzzleStats.totalSolved / data.puzzleStats.totalAttempts) * 100)
                : 0}%
            </span>
            <span class="stat-label">solve rate</span>
          </div>
          <div class="stat">
            <span class="stat-value">{data.puzzleStats.bestStreak}</span>
            <span class="stat-label">best streak</span>
          </div>
        </div>
      </div>
    {/if}
  </div>
{:else}
  <div class="dashboard-empty">
    <h1 class="welcome">Welcome to ChessMentor</h1>
    <p class="empty-tagline">Your AI chess coach. Play, learn, improve.</p>
    <div class="quick-start">
      <button class="qs-btn qs-play" onclick={() => onNavigate("play")}>
        Start a Game
      </button>
      <button class="qs-btn qs-puzzles" onclick={() => onNavigate("problems")}>
        Try Puzzles
      </button>
    </div>
  </div>
{/if}

{#if adaptivePrompt}
  <AdaptivePromptDialog
    prompt={adaptivePrompt}
    onAccept={handleAdaptiveAccept}
    onDismiss={() => { adaptivePrompt = null; }}
  />
{/if}

<style>
  .dashboard {
    padding: 24px;
    max-width: 720px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .dashboard-loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 300px;
  }

  .loading-text {
    color: #9ca3af;
    font-size: 14px;
  }

  .dashboard-header {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .welcome {
    font-size: 24px;
    font-weight: 700;
    color: #1e293b;
    margin: 0;
  }

  .dashboard-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .card {
    background: white;
    border: 1px solid #e5e7eb;
    border-radius: 10px;
    padding: 16px;
  }

  .card-header {
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #6b7280;
    margin-bottom: 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .skill-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .overall-rating {
    font-size: 14px;
    color: #374151;
  }

  .overall-rating strong {
    font-size: 18px;
    color: #1e40af;
  }

  .empty-skill {
    text-align: center;
    color: #9ca3af;
    padding: 24px 0;
    font-size: 14px;
  }

  .empty-hint {
    font-size: 12px;
    margin-top: 4px;
  }

  .quick-start {
    display: flex;
    gap: 10px;
  }

  .qs-btn {
    flex: 1;
    padding: 12px 16px;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .qs-btn:hover {
    opacity: 0.9;
  }

  .qs-play {
    background: #1e40af;
    color: white;
  }

  .qs-puzzles {
    background: #059669;
    color: white;
  }

  .qs-openings {
    background: #7c3aed;
    color: white;
  }

  .view-all {
    font-size: 12px;
    color: #6366f1;
    background: none;
    border: none;
    cursor: pointer;
    font-weight: 500;
  }

  .view-all:hover {
    text-decoration: underline;
  }

  .recent-games {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .puzzle-stats {
    display: flex;
    gap: 24px;
  }

  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .stat-value {
    font-size: 22px;
    font-weight: 700;
    color: #1e293b;
  }

  .stat-label {
    font-size: 11px;
    color: #9ca3af;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .dashboard-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 400px;
    gap: 16px;
    text-align: center;
  }

  .empty-tagline {
    color: #6b7280;
    font-size: 16px;
    margin: 0;
  }
</style>
