<script lang="ts">
  import type { PatternSummary, StudySuggestion } from "../../types/engine";

  type Props = {
    summary: PatternSummary | null;
    suggestions: StudySuggestion[];
  };

  let { summary, suggestions }: Props = $props();

  const THEME_LABELS: Record<string, string> = {
    knightOnRim: "Knight on rim",
    bishopPairAdvantage: "Bishop pair",
    isolatedQueenPawn: "Isolated QP",
    passedPawn: "Passed pawn",
    doubledPawns: "Doubled pawns",
    backwardPawn: "Backward pawn",
    openFile: "Open file",
    rookOnSeventh: "Rook on 7th",
    kingSafetyCompromised: "King safety",
    undevelopedPieces: "Development",
    centralControl: "Center control",
    pawnChainTension: "Pawn tension",
    materialImbalance: "Material",
    backRankWeakness: "Back rank",
    pinnedPiece: "Pin",
    forkAvailable: "Fork",
    hangingMaterial: "Hanging piece",
  };

  const PHASE_LABELS: Record<string, string> = {
    opening: "Opening",
    middlegame: "Middlegame",
    endgame: "Endgame",
  };
</script>

{#if summary && (summary.totalErrors > 0 || summary.strengths.length > 0)}
  <div class="pattern-summary">
    {#if summary.strengths.length > 0}
      <div class="strengths-section">
        <h4 class="section-title strengths-title">Strengths</h4>
        {#each summary.strengths as strength}
          <p class="strength-item">{strength}</p>
        {/each}
      </div>
    {/if}

    {#if summary.totalErrors > 0}
      <div class="weaknesses-section">
        <h4 class="section-title weaknesses-title">Areas to Improve ({summary.totalErrors} error{summary.totalErrors !== 1 ? "s" : ""})</h4>
        {#if summary.errorThemes.length > 0}
          <div class="theme-list">
            {#each summary.errorThemes as [theme, count]}
              <div class="theme-row">
                <span class="theme-name">{THEME_LABELS[theme] ?? theme}</span>
                <span class="theme-count">{count}</span>
              </div>
            {/each}
          </div>
        {/if}
        {#if Object.keys(summary.errorsByPhase).length > 0}
          <div class="phase-errors">
            {#each Object.entries(summary.errorsByPhase) as [phase, count]}
              <span class="phase-badge">{PHASE_LABELS[phase] ?? phase}: {count}</span>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>
{/if}

{#if suggestions.length > 0}
  <div class="study-suggestions">
    <h4 class="section-title">Study Suggestions</h4>
    {#each suggestions as suggestion}
      <div class="suggestion-card">
        <span class="suggestion-topic">{suggestion.topic}</span>
        <p class="suggestion-desc">{suggestion.description}</p>
      </div>
    {/each}
  </div>
{/if}

<style>
  .pattern-summary {
    padding: 8px 12px;
    border-bottom: 1px solid #e5e7eb;
  }

  .section-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0 0 4px;
  }

  .strengths-title {
    color: #16a34a;
  }

  .weaknesses-title {
    color: #dc2626;
  }

  .strengths-section {
    margin-bottom: 8px;
  }

  .strength-item {
    font-size: 12px;
    color: #166534;
    background: #f0fdf4;
    padding: 3px 8px;
    border-radius: 4px;
    margin: 2px 0;
  }

  .weaknesses-section {
    margin-bottom: 4px;
  }

  .theme-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .theme-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
    padding: 2px 0;
  }

  .theme-name {
    color: #374151;
  }

  .theme-count {
    font-size: 11px;
    font-weight: 600;
    color: #dc2626;
    background: #fee2e2;
    padding: 0 6px;
    border-radius: 8px;
    min-width: 18px;
    text-align: center;
  }

  .phase-errors {
    display: flex;
    gap: 6px;
    margin-top: 4px;
    flex-wrap: wrap;
  }

  .phase-badge {
    font-size: 10px;
    color: #6b7280;
    background: #f3f4f6;
    padding: 1px 6px;
    border-radius: 3px;
  }

  .study-suggestions {
    padding: 8px 12px;
    border-bottom: 1px solid #e5e7eb;
  }

  .suggestion-card {
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
    padding: 6px 10px;
    margin: 4px 0;
  }

  .suggestion-topic {
    font-size: 12px;
    font-weight: 600;
    color: #1e40af;
  }

  .suggestion-desc {
    font-size: 11px;
    line-height: 1.4;
    color: #475569;
    margin: 2px 0 0;
  }
</style>
