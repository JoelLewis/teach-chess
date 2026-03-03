<script lang="ts">
  type Props = {
    playerName?: string;
  };

  let { playerName = "Player" }: Props = $props();
</script>

<header class="app-header">
  <div class="header-content">
    <span class="player-name">{playerName}</span>
  </div>
</header>

<style>
  .app-header {
    height: 48px;
    background: var(--cm-bg-surface);
    border-bottom: 1px solid var(--cm-border-light);
    display: flex;
    align-items: center;
    padding: 0 24px;
  }

  .header-content {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    width: 100%;
  }

  .player-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--cm-text-secondary);
  }
</style>
