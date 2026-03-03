<script lang="ts">
  import { errorStore } from "../../stores/error.svelte";
</script>

{#if errorStore.message}
  <div class="toast" role="alert">
    <span class="toast-msg">{errorStore.message}</span>
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
    background: var(--cm-status-error);
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

  .toast-dismiss {
    background: none;
    border: none;
    color: var(--cm-text-inverse);
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
