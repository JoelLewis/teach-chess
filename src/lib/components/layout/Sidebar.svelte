<script lang="ts">
  type Page = "home" | "play" | "problems" | "openings" | "history" | "review" | "settings";

  type Props = {
    currentPage: Page;
    onNavigate: (page: Page) => void;
  };

  let { currentPage, onNavigate }: Props = $props();

  const navItems: { page: Page; label: string; icon: string }[] = [
    { page: "home", label: "Home", icon: "\u2302" },
    { page: "play", label: "Play", icon: "\u265A" },
    { page: "problems", label: "Problems", icon: "\u2617" },
    { page: "openings", label: "Openings", icon: "\u2655" },
    { page: "history", label: "History", icon: "\u2630" },
    { page: "settings", label: "Settings", icon: "\u2699" },
  ];
</script>

<nav class="sidebar" aria-label="Main navigation">
  <div class="logo">
    <span class="logo-icon">\u265E</span>
    <span class="logo-text">ChessMentor</span>
  </div>

  <div class="nav-items">
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={currentPage === item.page}
        aria-current={currentPage === item.page ? "page" : undefined}
        onclick={() => onNavigate(item.page)}
      >
        <span class="nav-icon">{item.icon}</span>
        <span class="nav-label">{item.label}</span>
      </button>
    {/each}
  </div>

  <div class="sidebar-footer">
    <span class="text-xs opacity-50">v0.1.0</span>
  </div>
</nav>

<style>
  .sidebar {
    width: 200px;
    min-height: 100vh;
    background: var(--cm-bg-sidebar);
    color: var(--cm-sidebar-text);
    display: flex;
    flex-direction: column;
    padding: 16px 0;
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 20px 24px;
  }

  .logo-icon {
    font-size: 28px;
  }

  .logo-text {
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .nav-items {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 20px;
    background: none;
    border: none;
    color: var(--cm-text-faint);
    cursor: pointer;
    font-size: 14px;
    width: 100%;
    text-align: left;
    transition: all 0.15s;
  }

  .nav-item:hover {
    color: var(--cm-sidebar-text);
    background: var(--cm-bg-sidebar-hover);
  }

  .nav-item.active {
    color: var(--cm-text-inverse);
    background: var(--cm-bg-sidebar-active);
    border-left: 3px solid var(--cm-accent-secondary);
    padding-left: 17px;
  }

  .nav-icon {
    font-size: 18px;
    width: 24px;
    text-align: center;
  }

  .sidebar-footer {
    margin-top: auto;
    padding: 16px 20px;
  }
</style>
