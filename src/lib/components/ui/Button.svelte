<script lang="ts">
  import type { Snippet } from "svelte";

  type Props = {
    variant?: "primary" | "secondary" | "ghost" | "danger";
    size?: "sm" | "md" | "lg";
    disabled?: boolean;
    onclick?: () => void;
    children: Snippet;
  };

  let { variant = "secondary", size = "md", disabled = false, onclick, children }: Props = $props();
</script>

<button
  class="btn btn-{variant} btn-{size}"
  {disabled}
  {onclick}
>
  {@render children()}
</button>

<style>
  .btn {
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 500;
    transition: all var(--cm-transition-fast);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  /* Sizes */
  .btn-sm { padding: 6px 12px; font-size: 13px; }
  .btn-md { padding: 8px 16px; font-size: 14px; }
  .btn-lg { padding: 10px 24px; font-size: 16px; }

  /* Variants */
  .btn-primary {
    background: var(--cm-accent-secondary-deep);
    color: var(--cm-text-inverse);
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--cm-accent-secondary-deeper);
  }

  .btn-secondary {
    background: var(--cm-bg-surface);
    color: var(--cm-text-primary);
    border: 1px solid var(--cm-border-medium);
  }

  .btn-secondary:hover:not(:disabled) {
    background: var(--cm-bg-hover);
  }

  .btn-ghost {
    background: transparent;
    color: var(--cm-text-secondary);
  }

  .btn-ghost:hover:not(:disabled) {
    background: var(--cm-bg-hover);
  }

  .btn-danger {
    background: var(--cm-status-error-bg-alt);
    color: var(--cm-status-error);
    border: 1px solid var(--cm-status-error-lighter);
  }

  .btn-danger:hover:not(:disabled) {
    background: var(--cm-status-error-muted);
  }

  /* Grid theme overrides */
  :global([data-theme="grid"]) .btn-primary {
    background: transparent;
    border: 1px solid var(--cm-accent-primary);
    color: var(--cm-accent-primary);
  }

  :global([data-theme="grid"]) .btn-primary:hover:not(:disabled) {
    background: rgba(0, 229, 255, 0.1);
    box-shadow: var(--cm-glow-primary);
  }
</style>
