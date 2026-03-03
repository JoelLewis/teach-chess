<script lang="ts">
  import { assessmentStore } from "../../stores/assessment.svelte";
  import { errorStore } from "../../stores/error.svelte";
  import * as api from "../../api/commands";

  const profile = $derived(assessmentStore.profile);
  const loading = $derived(assessmentStore.loading);
  const expanded = $derived(assessmentStore.expanded);

  const CATEGORY_COLORS: Record<string, string> = {
    tactical: "#ef4444",
    positional: "#3b82f6",
    endgame: "#8b5cf6",
    opening: "#f59e0b",
    pattern: "#10b981",
  };

  function categoryLabel(category: string): string {
    return category.charAt(0).toUpperCase() + category.slice(1);
  }

  function ratingColor(rating: number): string {
    if (rating >= 1500) return "#059669";
    if (rating >= 1200) return "#ca8a04";
    return "#dc2626";
  }

  function barWidth(rating: number): number {
    // Scale: 400–2000 maps to 0–100%
    return Math.min(100, Math.max(0, ((rating - 400) / 1600) * 100));
  }

  async function loadProfile() {
    assessmentStore.loading = true;
    try {
      const p = await api.getSkillProfile();
      assessmentStore.profile = p;
    } catch (err) {
      errorStore.show(`Failed to load skill profile: ${err}`);
    } finally {
      assessmentStore.loading = false;
    }
  }

  $effect(() => {
    loadProfile();
  });
</script>

<div class="skill-panel">
  <button
    class="panel-toggle"
    onclick={() => { assessmentStore.expanded = !expanded; }}
  >
    <span class="toggle-label">Skill Ratings</span>
    {#if profile && profile.ratings.length > 0}
      <span class="overall-rating" style:color={ratingColor(profile.overallRating)}>
        {Math.round(profile.overallRating)}
      </span>
    {/if}
    <span class="toggle-arrow">{expanded ? "\u25B2" : "\u25BC"}</span>
  </button>

  {#if expanded}
    <div class="panel-content">
      {#if loading}
        <div class="loading-text">Loading...</div>
      {:else if !profile || profile.ratings.length === 0}
        <div class="empty-text">
          Solve puzzles to build your skill profile.
        </div>
      {:else}
        <div class="ratings-list">
          {#each profile.ratings as rating}
            <div class="rating-row">
              <div class="rating-header">
                <span
                  class="category-dot"
                  style:background={CATEGORY_COLORS[rating.category] ?? "#6b7280"}
                ></span>
                <span class="category-name">{categoryLabel(rating.category)}</span>
                <span class="rating-value" style:color={ratingColor(rating.rating)}>
                  {Math.round(rating.rating)}
                </span>
              </div>
              <div class="rating-bar-bg">
                <div
                  class="rating-bar-fill"
                  style:width="{barWidth(rating.rating)}%"
                  style:background={CATEGORY_COLORS[rating.category] ?? "#6b7280"}
                ></div>
              </div>
              <div class="rating-meta">
                <span>{rating.gamesCount} puzzles</span>
                <span class="rd-text">&plusmn;{Math.round(rating.rd)}</span>
              </div>
            </div>
          {/each}
        </div>

        {#if profile.strongestCategory}
          <div class="insight">
            Strongest: <strong>{categoryLabel(profile.strongestCategory)}</strong>
          </div>
        {/if}
        {#if profile.weakestCategory && profile.weakestCategory !== profile.strongestCategory}
          <div class="insight weak">
            Focus area: <strong>{categoryLabel(profile.weakestCategory)}</strong>
          </div>
        {/if}
      {/if}
    </div>
  {/if}
</div>

<style>
  .skill-panel {
    border-top: 1px solid #e5e7eb;
  }

  .panel-toggle {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 10px 16px;
    background: none;
    border: none;
    cursor: pointer;
    gap: 8px;
    transition: background 0.15s;
  }

  .panel-toggle:hover {
    background: #f8fafc;
  }

  .toggle-label {
    font-size: 13px;
    font-weight: 600;
    color: #374151;
  }

  .overall-rating {
    font-size: 15px;
    font-weight: 700;
    margin-left: auto;
  }

  .toggle-arrow {
    font-size: 10px;
    color: #9ca3af;
  }

  .panel-content {
    padding: 0 16px 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .loading-text,
  .empty-text {
    font-size: 13px;
    color: #9ca3af;
    text-align: center;
    padding: 8px 0;
  }

  .ratings-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .rating-row {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .rating-header {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .category-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .category-name {
    font-size: 12px;
    color: #374151;
    flex: 1;
  }

  .rating-value {
    font-size: 13px;
    font-weight: 700;
  }

  .rating-bar-bg {
    height: 4px;
    background: #f3f4f6;
    border-radius: 2px;
    overflow: hidden;
  }

  .rating-bar-fill {
    height: 100%;
    border-radius: 2px;
    transition: width 0.3s;
  }

  .rating-meta {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: #9ca3af;
  }

  .rd-text {
    opacity: 0.7;
  }

  .insight {
    font-size: 12px;
    color: #059669;
    padding: 6px 10px;
    background: #ecfdf5;
    border-radius: 4px;
  }

  .insight.weak {
    color: #ca8a04;
    background: #fefce8;
  }
</style>
