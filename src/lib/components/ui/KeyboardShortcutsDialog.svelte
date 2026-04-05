<script lang="ts">
  type Props = {
    open: boolean;
    onClose: () => void;
    shortcuts: Array<{ key: string; description: string }>;
  };

  let { open, onClose, shortcuts }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div class="overlay" role="dialog" aria-modal="true" aria-label="Keyboard shortcuts" onkeydown={handleKeydown}>
    <div class="dialog">
      <div class="dialog-header">
        <h3 class="dialog-title">Keyboard Shortcuts</h3>
        <button class="close-btn" onclick={onClose}>&times;</button>
      </div>
      <div class="shortcut-list">
        {#each shortcuts as { key, description }}
          <div class="shortcut-row">
            <kbd class="shortcut-key">{key}</kbd>
            <span class="shortcut-desc">{description}</span>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: var(--cm-bg-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 150;
  }

  .dialog {
    background: var(--cm-bg-surface);
    border-radius: 10px;
    box-shadow: var(--cm-shadow-lg);
    min-width: 280px;
    max-width: 400px;
    overflow: hidden;
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--cm-border-light);
  }

  .dialog-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--cm-text-primary);
    margin: 0;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 20px;
    cursor: pointer;
    color: var(--cm-text-muted);
    padding: 0 4px;
  }

  .shortcut-list {
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .shortcut-row {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .shortcut-key {
    min-width: 32px;
    padding: 3px 8px;
    background: var(--cm-bg-surface-alt);
    border: 1px solid var(--cm-border-medium);
    border-radius: 4px;
    font-size: 12px;
    font-family: var(--cm-font-mono);
    text-align: center;
  }

  .shortcut-desc {
    font-size: 13px;
    color: var(--cm-text-secondary);
  }
</style>
