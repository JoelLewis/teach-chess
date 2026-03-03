<script lang="ts">
  import { themeStore } from "../../stores/theme.svelte";
  import type { Theme } from "../../types/theme";

  const themes: { id: Theme; name: string; description: string }[] = [
    {
      id: "study",
      name: "The Study",
      description: "Warm wood and parchment. A cozy reading room for quiet concentration.",
    },
    {
      id: "grid",
      name: "The Grid",
      description: "Dark neon. A cyberpunk interface for late-night sessions.",
    },
  ];
</script>

<section class="theme-section">
  <h3 class="section-title">Theme</h3>
  <div class="theme-cards">
    {#each themes as theme}
      <button
        class="theme-card"
        class:active={themeStore.current === theme.id}
        onclick={() => themeStore.set(theme.id)}
      >
        <div class="theme-preview" data-preview={theme.id}>
          <div class="preview-sidebar"></div>
          <div class="preview-content">
            <div class="preview-header"></div>
            <div class="preview-board"></div>
          </div>
        </div>
        <div class="theme-info">
          <span class="theme-name">{theme.name}</span>
          <span class="theme-desc">{theme.description}</span>
        </div>
      </button>
    {/each}
  </div>
</section>

<style>
  .theme-section {
    margin-bottom: 24px;
  }

  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--cm-text-primary);
    margin-bottom: 12px;
  }

  .theme-cards {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .theme-card {
    display: flex;
    flex-direction: column;
    border: 2px solid var(--cm-border-light);
    border-radius: 10px;
    overflow: hidden;
    cursor: pointer;
    background: var(--cm-bg-surface);
    transition: border-color 0.15s, box-shadow 0.15s;
    text-align: left;
    padding: 0;
  }

  .theme-card:hover {
    border-color: var(--cm-text-faint);
  }

  .theme-card.active {
    border-color: var(--cm-accent-primary);
    box-shadow: 0 0 0 1px var(--cm-accent-primary);
  }

  .theme-preview {
    height: 80px;
    display: flex;
    overflow: hidden;
  }

  /* Study preview */
  .theme-preview[data-preview="study"] {
    background: #f1f5f9;
  }

  .theme-preview[data-preview="study"] .preview-sidebar {
    width: 30px;
    background: #1e293b;
  }

  .theme-preview[data-preview="study"] .preview-header {
    height: 8px;
    background: #ffffff;
    border-bottom: 1px solid #e2e8f0;
  }

  .theme-preview[data-preview="study"] .preview-board {
    width: 48px;
    height: 48px;
    margin: 8px auto;
    background: #f0d9b5;
    border-radius: 2px;
    background-image:
      linear-gradient(45deg, rgba(0,0,0,0.15) 25%, transparent 25%),
      linear-gradient(-45deg, rgba(0,0,0,0.15) 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, rgba(0,0,0,0.15) 75%),
      linear-gradient(-45deg, transparent 75%, rgba(0,0,0,0.15) 75%);
    background-size: 12px 12px;
    background-position: 0 0, 0 6px, 6px -6px, -6px 0;
  }

  /* Grid preview */
  .theme-preview[data-preview="grid"] {
    background: #0a0a0f;
  }

  .theme-preview[data-preview="grid"] .preview-sidebar {
    width: 30px;
    background: #070b14;
    border-right: 1px solid rgba(0, 229, 255, 0.1);
  }

  .theme-preview[data-preview="grid"] .preview-header {
    height: 8px;
    background: #12121a;
    border-bottom: 1px solid rgba(0, 229, 255, 0.08);
  }

  .theme-preview[data-preview="grid"] .preview-board {
    width: 48px;
    height: 48px;
    margin: 8px auto;
    background: #0e0e1a;
    border-radius: 2px;
    border: 1px solid rgba(0, 229, 255, 0.15);
    background-image:
      linear-gradient(45deg, rgba(0,229,255,0.04) 25%, transparent 25%),
      linear-gradient(-45deg, rgba(0,229,255,0.04) 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, rgba(0,229,255,0.04) 75%),
      linear-gradient(-45deg, transparent 75%, rgba(0,229,255,0.04) 75%);
    background-size: 12px 12px;
    background-position: 0 0, 0 6px, 6px -6px, -6px 0;
  }

  .preview-content {
    flex: 1;
  }

  .theme-info {
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .theme-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--cm-text-primary);
  }

  .theme-desc {
    font-size: 11px;
    color: var(--cm-text-muted);
    line-height: 1.4;
  }
</style>
