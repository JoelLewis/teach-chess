<script lang="ts">
  import type { AdaptivePrompt } from "../../types/dashboard";

  type Props = {
    prompt: AdaptivePrompt;
    onAccept: (activity: string, category: string | null) => void;
    onDismiss: () => void;
  };

  let { prompt, onAccept, onDismiss }: Props = $props();

  const typeColors: Record<string, string> = {
    increaseChallenge: "#059669",
    decreaseChallenge: "#ca8a04",
    frustrationDetected: "#dc2626",
    plateauDetected: "#6366f1",
  };

  const typeLabels: Record<string, string> = {
    increaseChallenge: "Level Up",
    decreaseChallenge: "Adjust Difficulty",
    frustrationDetected: "Take a Breather",
    plateauDetected: "Break Through",
  };

  let accentColor = $derived(typeColors[prompt.promptType] ?? "#6366f1");
  let label = $derived(typeLabels[prompt.promptType] ?? "Suggestion");
</script>

<div class="overlay" role="dialog" aria-modal="true">
  <div class="dialog" style:border-top-color={accentColor}>
    <div class="badge" style:color={accentColor}>{label}</div>
    <p class="message">{prompt.message}</p>
    <p class="suggestion">{prompt.suggestion}</p>
    <div class="actions">
      <button
        class="btn-accept"
        style:background={accentColor}
        onclick={() => onAccept(prompt.targetActivity, prompt.targetCategory)}
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
    background: rgba(0, 0, 0, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .dialog {
    background: white;
    border-radius: 12px;
    padding: 24px;
    max-width: 420px;
    width: 90%;
    border-top: 4px solid;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.15);
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
    color: #374151;
    line-height: 1.6;
    margin: 0 0 8px;
  }

  .suggestion {
    font-size: 14px;
    color: #6b7280;
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
    color: white;
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
    background: #f3f4f6;
    color: #6b7280;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    font-size: 14px;
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn-dismiss:hover {
    background: #e5e7eb;
  }
</style>
