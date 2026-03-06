<script lang="ts">
  import type { MoveEvaluation, CoachingSource } from "../../types/engine";
  import { CLASSIFICATION_COLORS } from "../../types/engine";
  import { generateCoaching } from "../../api/commands";
  import { onLlmToken } from "../../api/events";

  type Props = {
    evaluation: MoveEvaluation | null;
  };

  let { evaluation }: Props = $props();

  let llmText = $state<string | null>(null);
  let llmSource = $state<CoachingSource | null>(null);
  let llmLoading = $state(false);
  let streamedText = $state("");
  let isStreaming = $state(false);

  let displayText = $derived(
    isStreaming ? streamedText : (llmText ?? evaluation?.coachingText ?? null),
  );

  let classColor = $derived(
    evaluation?.classification
      ? CLASSIFICATION_COLORS[evaluation.classification]
      : "var(--cm-text-muted)",
  );

  let themes = $derived(evaluation?.coachingContext?.themes ?? []);

  // When the evaluation changes, try to get LLM-enhanced coaching with streaming
  $effect(() => {
    const eval_ = evaluation;
    llmText = null;
    llmSource = null;
    llmLoading = false;
    streamedText = "";
    isStreaming = false;

    if (!eval_?.classification || !eval_?.fenBefore) return;

    let cancelled = false;
    llmLoading = true;

    const requestId = crypto.randomUUID();
    let unlisten: (() => void) | undefined;

    // Subscribe to token stream before calling generate
    onLlmToken((event) => {
      if (cancelled || event.requestId !== requestId) return;

      if (event.type === "token") {
        streamedText += event.text;
        isStreaming = true;
      } else if (event.type === "done") {
        streamedText = event.fullText;
        isStreaming = false;
      }
    }).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    generateCoaching(
      eval_.fenBefore,
      eval_.classification,
      eval_.coachingContext,
      eval_.playerMoveSan,
      eval_.engineBestSan,
      requestId,
    )
      .then((response) => {
        if (cancelled) return;
        isStreaming = false;
        if (response.source !== "template") {
          llmText = response.text;
        }
        llmSource = response.source;
      })
      .catch(() => {
        // Silently fall back to template text
        isStreaming = false;
      })
      .finally(() => {
        if (!cancelled) llmLoading = false;
      });

    return () => {
      cancelled = true;
      isStreaming = false;
      unlisten?.();
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
    background: var(--cm-bg-surface-alt);
    border: 1px solid var(--cm-border-default);
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
    color: var(--cm-text-inverse);
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
    background: var(--cm-accent-secondary-bg-alt);
    color: var(--cm-accent-secondary-hover);
  }

  .source-cached {
    background: var(--cm-border-default);
    color: var(--cm-text-muted);
  }

  .loading-dots {
    display: inline-flex;
    gap: 1px;
    font-size: 14px;
    color: var(--cm-text-faint);
  }

  .dot {
    animation: blink 1.4s infinite both;
  }
  .dot:nth-child(2) { animation-delay: 0.2s; }
  .dot:nth-child(3) { animation-delay: 0.4s; }

  @keyframes blink {
    0%, 80%, 100% { opacity: 0.2; }
    40% { opacity: 1; }
  }

  .coaching-text {
    font-size: 13px;
    line-height: 1.5;
    color: var(--cm-text-primary);
    margin: 0 0 6px;
  }

  .theme-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .theme-tag {
    font-size: 10px;
    color: var(--cm-text-muted);
    background: var(--cm-border-default);
    padding: 1px 6px;
    border-radius: 3px;
  }
</style>
