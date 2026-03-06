<script lang="ts">
  type Props = {
    message?: string;
    size?: "sm" | "md" | "lg";
  };

  let { message, size = "md" }: Props = $props();
</script>

<div class="spinner-container" class:sm={size === "sm"} class:lg={size === "lg"}>
  <div class="spinner" role="status" aria-label={message ?? "Loading"}>
    <div class="spinner-ring"></div>
  </div>
  {#if message}
    <span class="spinner-message">{message}</span>
  {/if}
</div>

<style>
  .spinner-container {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .spinner-ring {
    width: 24px;
    height: 24px;
    border: 3px solid var(--cm-border-medium);
    border-top-color: var(--cm-accent-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .sm .spinner-ring {
    width: 16px;
    height: 16px;
    border-width: 2px;
  }

  .lg .spinner-ring {
    width: 32px;
    height: 32px;
    border-width: 4px;
  }

  .spinner-message {
    font-size: 13px;
    color: var(--cm-text-secondary);
  }

  .sm .spinner-message {
    font-size: 12px;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
