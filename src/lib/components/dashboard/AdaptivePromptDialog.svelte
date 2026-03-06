<script lang="ts">
  import type { AdaptivePrompt } from "../../types/dashboard";

  type Props = {
    prompt: AdaptivePrompt;
    onAccept: (activity: string, category: string | null) => void;
    onDismiss: () => void;
  };

  let { prompt, onAccept, onDismiss }: Props = $props();

  const typeColors: Record<string, string> = {
    increaseChallenge: "var(--cm-status-success-alt)",
    decreaseChallenge: "var(--cm-status-warning-dark)",
    frustrationDetected: "var(--cm-status-error)",
    plateauDetected: "var(--cm-accent-primary-light)",
  };

  const typeLabels: Record<string, string> = {
    increaseChallenge: "Level Up",
    decreaseChallenge: "Adjust Difficulty",
    frustrationDetected: "Take a Breather",
    plateauDetected: "Break Through",
  };

  let accentColor = $derived(typeColors[prompt.promptType] ?? "var(--cm-accent-primary-light)");
  let label = $derived(typeLabels[prompt.promptType] ?? "Suggestion");

  let dialogEl: HTMLDivElement | undefined = $state();
  let acceptBtnEl: HTMLButtonElement | undefined = $state();

  $effect(() => {
    acceptBtnEl?.focus();
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onDismiss();
      return;
    }
    if (e.key === "Tab" && dialogEl) {
      const focusable = dialogEl.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
      );
      if (focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault();
          last.focus();
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div class="overlay" role="dialog" aria-modal="true" onkeydown={handleKeydown}>
  <div class="dialog" style:border-top-color={accentColor} bind:this={dialogEl}>
    <div class="badge" style:color={accentColor}>{label}</div>
    <p class="message">{prompt.message}</p>
    <p class="suggestion">{prompt.suggestion}</p>
    <div class="actions">
      <button
        class="btn-accept"
        style:background={accentColor}
        onclick={() => onAccept(prompt.targetActivity, prompt.targetCategory)}
        bind:this={acceptBtnEl}
      >
        Sounds good
      </button>
      <button class="btn-dismiss" onclick={onDismiss}>
        Not now
      </button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: var(--cm-bg-overlay-light);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .dialog {
    background: var(--cm-bg-surface);
    border-radius: 12px;
    padding: 24px;
    max-width: 420px;
    width: 90%;
    border-top: 4px solid;
    box-shadow: var(--cm-shadow-xl);
  }

  .badge {
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 12px;
  }

  .message {
    font-size: 15px;
    color: var(--cm-text-secondary);
    line-height: 1.6;
    margin: 0 0 8px;
  }

  .suggestion {
    font-size: 14px;
    color: var(--cm-text-muted);
    line-height: 1.5;
    margin: 0 0 20px;
  }

  .actions {
    display: flex;
    gap: 10px;
  }

  .btn-accept {
    flex: 1;
    padding: 10px 16px;
    color: var(--cm-text-inverse);
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  .btn-accept:hover {
    opacity: 0.9;
  }

  .btn-dismiss {
    padding: 10px 16px;
    background: var(--cm-bg-hover);
    color: var(--cm-text-muted);
    border: 1px solid var(--cm-border-medium);
    border-radius: 6px;
    font-size: 14px;
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn-dismiss:hover {
    background: var(--cm-bg-active);
  }
</style>
