<script lang="ts">
  type Props = {
    result: "win" | "loss" | "draw";
    outcomeDetail: string;
    opponentInfo: string;
    moveCount: number;
    accuracy: number;
    bestMoves: number;
    inaccuracies: number;
    blunders: number;
    aiQuote: string | null;
  };

  let {
    result,
    outcomeDetail,
    opponentInfo,
    moveCount,
    accuracy,
    bestMoves,
    inaccuracies,
    blunders,
    aiQuote,
  }: Props = $props();

  let resultLabel = $derived(
    result === "win" ? "Victory" : result === "loss" ? "Defeat" : "Draw",
  );

  let today = new Date().toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });

  let copied = $state(false);

  function copyToClipboard() {
    const lines = [
      `\u265E ChessMentor \u2014 ${resultLabel} ${outcomeDetail}`,
      `${moveCount} moves \u00B7 ${opponentInfo}`,
      `Accuracy: ${accuracy}% | Best: ${bestMoves} | Blunders: ${blunders}`,
    ];
    if (aiQuote) {
      lines.push(`"${aiQuote}"`);
    }
    navigator.clipboard.writeText(lines.join("\n")).then(() => {
      copied = true;
      setTimeout(() => (copied = false), 2000);
    });
  }
</script>

<div class="summary-card">
  <div class="card-header">
    <span class="brand">{"\u265E"} ChessMentor</span>
    <span class="date">{today}</span>
  </div>

  <div class="card-body">
    <h2
      class="result-label"
      class:win={result === "win"}
      class:loss={result === "loss"}
      class:draw={result === "draw"}
    >
      {resultLabel}
    </h2>
    <p class="outcome-detail">{outcomeDetail} &middot; {moveCount} moves</p>
    <p class="opponent-info">{opponentInfo}</p>

    <div class="stat-row">
      <div class="stat">
        <span class="stat-value">{accuracy}%</span>
        <span class="stat-label">Accuracy</span>
      </div>
      <div class="stat">
        <span class="stat-value best">{bestMoves}</span>
        <span class="stat-label">Best</span>
      </div>
      <div class="stat">
        <span class="stat-value inaccuracy">{inaccuracies}</span>
        <span class="stat-label">Inaccuracies</span>
      </div>
      <div class="stat">
        <span class="stat-value blunder">{blunders}</span>
        <span class="stat-label">Blunders</span>
      </div>
    </div>

    {#if aiQuote}
      <blockquote class="ai-quote">&ldquo;{aiQuote}&rdquo;</blockquote>
    {/if}
  </div>

  <div class="card-footer">
    <span class="footer-brand">{"\u265E"} chessmentor.app</span>
    <button class="copy-btn" onclick={copyToClipboard}>
      {copied ? "Copied!" : "Copy summary"}
    </button>
  </div>
</div>

<style>
  .summary-card {
    background: var(--cm-bg-surface);
    border: 1px solid var(--cm-border-medium);
    border-radius: 12px;
    overflow: hidden;
    max-width: 380px;
    width: 100%;
    margin-top: 16px;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 16px;
    border-bottom: 1px solid var(--cm-border-light);
    font-size: 12px;
  }

  .brand {
    font-weight: 600;
    color: var(--cm-text-primary);
  }

  .date {
    color: var(--cm-text-muted);
  }

  .card-body {
    padding: 20px 16px 16px;
    text-align: center;
  }

  .result-label {
    font-size: 28px;
    font-weight: 800;
    margin: 0 0 4px;
    letter-spacing: -0.5px;
  }

  .result-label.win {
    color: var(--cm-status-success);
  }

  .result-label.loss {
    color: var(--cm-status-error);
  }

  .result-label.draw {
    color: var(--cm-accent-primary);
  }

  .outcome-detail {
    color: var(--cm-text-secondary);
    font-size: 14px;
    margin: 0 0 2px;
  }

  .opponent-info {
    color: var(--cm-text-muted);
    font-size: 13px;
    margin: 0 0 16px;
  }

  .stat-row {
    display: flex;
    justify-content: center;
    gap: 20px;
    margin-bottom: 16px;
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

  .stat-value.best {
    color: var(--cm-status-success);
  }

  .stat-value.inaccuracy {
    color: var(--cm-status-warning, #e6a700);
  }

  .stat-value.blunder {
    color: var(--cm-status-error);
  }

  .stat-label {
    font-size: 11px;
    color: var(--cm-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .ai-quote {
    font-style: italic;
    color: var(--cm-text-secondary);
    font-size: 13px;
    line-height: 1.5;
    margin: 0;
    padding: 12px 16px;
    border-left: 3px solid var(--cm-accent-primary);
    background: var(--cm-bg-hover);
    border-radius: 0 6px 6px 0;
    text-align: left;
  }

  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 16px;
    border-top: 1px solid var(--cm-border-light);
    font-size: 12px;
  }

  .footer-brand {
    color: var(--cm-text-muted);
  }

  .copy-btn {
    background: none;
    border: 1px solid var(--cm-border-medium);
    border-radius: 6px;
    padding: 4px 12px;
    font-size: 12px;
    color: var(--cm-text-secondary);
    cursor: pointer;
    transition: background 0.15s;
  }

  .copy-btn:hover {
    background: var(--cm-bg-hover);
  }
</style>
