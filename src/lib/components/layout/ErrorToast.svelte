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
    background: #dc2626;
    color: white;
    padding: 10px 20px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 200;
    font-size: 14px;
    max-width: 500px;
  }

  .toast-msg {
    flex: 1;
  }

  .toast-dismiss {
    background: none;
    border: none;
    color: white;
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
