<script lang="ts">
  import { errorStore } from "../../stores/error.svelte";
</script>

{#if errorStore.message}
  <div class="toast toast-{errorStore.severity}" role="alert">
    <span class="toast-msg">{errorStore.message}</span>
    {#if errorStore.retry}
      <button
        class="toast-retry"
        onclick={() => {
          const fn = errorStore.retry;
          errorStore.dismiss();
          fn?.();
        }}
      >
        Retry
      </button>
    {/if}
    <button class="toast-dismiss" onclick={() => errorStore.dismiss()}>
      &times;
    </button>
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    color: var(--cm-text-inverse);
    padding: 10px 20px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 12px;
    box-shadow: var(--cm-shadow-md);
    z-index: 200;
    font-size: 14px;
    max-width: 500px;
    animation: toast-in 0.2s ease;
  }

  .toast-error {
    background: var(--cm-status-error);
  }

  .toast-warning {
    background: var(--cm-status-warning);
    color: var(--cm-text-primary);
  }

  .toast-info {
    background: var(--cm-accent-secondary);
  }

  @keyframes toast-in {
    from {
      opacity: 0;
      transform: translateX(-50%) translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateX(-50%) translateY(0);
    }
  }

  .toast-msg {
    flex: 1;
  }

  .toast-retry {
    background: rgba(255, 255, 255, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.4);
    color: inherit;
    font-size: 13px;
    padding: 4px 12px;
    border-radius: 4px;
    cursor: pointer;
    white-space: nowrap;
  }

  .toast-retry:hover {
    background: rgba(255, 255, 255, 0.3);
  }

  .toast-dismiss {
    background: none;
    border: none;
    color: inherit;
    font-size: 20px;
    cursor: pointer;
    padding: 0 4px;
    line-height: 1;
    opacity: 0.8;
  }

  .toast-dismiss:hover {
    opacity: 1;
  }
</style>
