<script lang="ts">
  import SkillRadarChart from "./SkillRadarChart.svelte";
  import RecommendationCard from "./RecommendationCard.svelte";
  import StreakBanner from "./StreakBanner.svelte";
  import AdaptivePromptDialog from "./AdaptivePromptDialog.svelte";
  import ModelDownloadCard from "./ModelDownloadCard.svelte";
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
  let showModelPrompt = $state(false);

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

  async function checkModelStatus() {
    try {
      const status = await api.getLlmStatus();
      // Hide the download card if the model is bundled or already available
      showModelPrompt = !status.available && !status.bundled;
    } catch {
      // Non-blocking
    }
  }

  $effect(() => {
    loadDashboard();
    checkAdaptive();
    checkModelStatus();
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

    <!-- Model Download Prompt -->
    {#if showModelPrompt}
      <ModelDownloadCard {onNavigate} />
    {/if}

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
            <p>Solve puzzles to build your skill profile</p>
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
    {#if showModelPrompt}
      <div class="empty-model-prompt">
        <ModelDownloadCard {onNavigate} />
      </div>
    {/if}
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
    color: var(--cm-text-disabled);
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
    color: var(--cm-text-primary);
    margin: 0;
  }

  .dashboard-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  @media (max-width: 600px) {
    .dashboard-grid {
      grid-template-columns: 1fr;
    }
  }

  .card {
    background: var(--cm-bg-surface);
    border: 1px solid var(--cm-border-light);
    border-radius: 10px;
    padding: 16px;
    transition: border-color var(--cm-transition-normal), box-shadow var(--cm-transition-normal);
  }

  /* Study: parchment texture on cards */
  :global([data-theme="study"]) .card {
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='200' height='200'%3E%3Cfilter id='p'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.8' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23p)' opacity='0.03'/%3E%3C/svg%3E");
    background-repeat: repeat;
    border-color: rgba(139, 109, 71, 0.15);
  }

  /* Grid: glowing card borders on hover */
  :global([data-theme="grid"]) .card:hover {
    border-color: var(--cm-border-glow);
    box-shadow: var(--cm-glow-primary);
  }

  .card-header {
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--cm-text-muted);
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
    color: var(--cm-text-secondary);
  }

  .overall-rating strong {
    font-size: 18px;
    color: var(--cm-accent-secondary-deep);
  }

  .empty-skill {
    text-align: center;
    color: var(--cm-text-muted);
    padding: 24px 16px;
    font-size: 14px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
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

  /* Study: brass bevel on quick-start buttons */
  :global([data-theme="study"]) .qs-btn {
    box-shadow:
      inset 0 1px 0 var(--cm-study-brass-highlight),
      0 1px 2px var(--cm-study-brass-shadow);
  }

  :global([data-theme="study"]) .qs-btn:hover {
    box-shadow:
      inset 0 1px 0 var(--cm-study-brass-highlight),
      0 1px 2px var(--cm-study-brass-shadow),
      var(--cm-study-brass-glow);
  }

  /* Grid: outlined quick-start buttons with glow */
  :global([data-theme="grid"]) .qs-btn {
    background: transparent;
    border: 1px solid currentColor;
  }

  :global([data-theme="grid"]) .qs-btn:hover {
    background: rgba(0, 229, 255, 0.08);
    box-shadow: var(--cm-glow-primary);
  }

  .qs-play {
    background: var(--cm-accent-secondary-deep);
    color: var(--cm-text-inverse);
  }

  .qs-puzzles {
    background: var(--cm-status-success-alt);
    color: var(--cm-text-inverse);
  }

  .qs-openings {
    background: var(--cm-accent-violet);
    color: var(--cm-text-inverse);
  }

  .view-all {
    font-size: 12px;
    color: var(--cm-accent-primary-light);
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
    color: var(--cm-text-primary);
  }

  .stat-label {
    font-size: 11px;
    color: var(--cm-text-disabled);
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
    color: var(--cm-text-muted);
    font-size: 16px;
    margin: 0;
  }

  .empty-model-prompt {
    width: 100%;
    max-width: 480px;
  }
</style>
