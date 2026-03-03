<script lang="ts">
  import type { MoveEvaluation, CoachingSource } from "../../types/engine";
  import { CLASSIFICATION_COLORS } from "../../types/engine";
  import { generateCoaching } from "../../api/commands";

  type Props = {
    evaluation: MoveEvaluation | null;
  };

  let { evaluation }: Props = $props();

  let llmText = $state<string | null>(null);
  let llmSource = $state<CoachingSource | null>(null);
  let llmLoading = $state(false);

  let displayText = $derived(llmText ?? evaluation?.coachingText ?? null);

  let classColor = $derived(
    evaluation?.classification
      ? CLASSIFICATION_COLORS[evaluation.classification]
      : "#888",
  );

  let themes = $derived(evaluation?.coachingContext?.themes ?? []);

  // When the evaluation changes, try to get LLM-enhanced coaching
  $effect(() => {
    const eval_ = evaluation;
    llmText = null;
    llmSource = null;
    llmLoading = false;

    if (!eval_?.classification || !eval_?.fenBefore) return;

    let cancelled = false;
    llmLoading = true;

    generateCoaching(
      eval_.fenBefore,
      eval_.classification,
      eval_.coachingContext,
      eval_.playerMoveSan,
      eval_.engineBestSan,
    )
      .then((response) => {
        if (cancelled) return;
        if (response.source !== "template") {
          llmText = response.text;
        }
        llmSource = response.source;
      })
      .catch(() => {
        // Silently fall back to template text
      })
      .finally(() => {
        if (!cancelled) llmLoading = false;
      });

    return () => {
      cancelled = true;
    };
  });

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
</script>

{#if displayText}
  <div class="coaching-panel">
    <div class="badge-row">
      {#if evaluation?.classification}
        <span class="classification-badge" style="background: {classColor}">
          {evaluation.classification}
        </span>
      {/if}
      {#if llmSource === "llm"}
        <span class="source-badge source-ai">AI</span>
      {:else if llmSource === "cache"}
        <span class="source-badge source-cached">cached</span>
      {/if}
      {#if llmLoading}
        <span class="loading-dots">
          <span class="dot">.</span><span class="dot">.</span><span class="dot">.</span>
        </span>
      {/if}
    </div>
    <p class="coaching-text">{displayText}</p>
    {#if themes.length > 0}
      <div class="theme-tags">
        {#each themes as theme}
          <span class="theme-tag">{THEME_LABELS[theme] ?? theme}</span>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .coaching-panel {
    padding: 10px 16px;
    margin: 0 4px;
    background: #f8fafc;
    border: 1px solid #e2e8f0;
    border-radius: 6px;
  }

  .badge-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 6px;
  }

  .classification-badge {
    display: inline-block;
    font-size: 10px;
    color: white;
    padding: 1px 8px;
    border-radius: 3px;
    text-transform: capitalize;
  }

  .source-badge {
    display: inline-block;
    font-size: 9px;
    padding: 1px 6px;
    border-radius: 3px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .source-ai {
    background: #dbeafe;
    color: #1d4ed8;
  }

  .source-cached {
    background: #e2e8f0;
    color: #64748b;
  }

  .loading-dots {
    display: inline-flex;
    gap: 1px;
    font-size: 14px;
    color: #94a3b8;
  }

  .dot {
    animation: blink 1.4s infinite both;
  }
  .dot:nth-child(2) { animation-delay: 0.2s; }
  .dot:nth-child(3) { animation-delay: 0.4s; }

  @keyframes blink {
    0%, 80%, 100% { opacity: 0; }
    40% { opacity: 1; }
  }

  .coaching-text {
    font-size: 13px;
    line-height: 1.5;
    color: #334155;
    margin: 0 0 6px;
  }

  .theme-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .theme-tag {
    font-size: 10px;
    color: #64748b;
    background: #e2e8f0;
    padding: 1px 6px;
    border-radius: 3px;
  }
</style>
