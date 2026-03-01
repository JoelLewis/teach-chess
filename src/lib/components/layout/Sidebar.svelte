<script lang="ts">
  type Page = "home" | "play" | "history" | "review";

  type Props = {
    currentPage: Page;
    onNavigate: (page: Page) => void;
  };

  let { currentPage, onNavigate }: Props = $props();

  const navItems: { page: Page; label: string; icon: string }[] = [
    { page: "home", label: "Home", icon: "\u2302" },
    { page: "play", label: "Play", icon: "\u265A" },
    { page: "history", label: "History", icon: "\u2630" },
  ];
</script>

<nav class="sidebar">
  <div class="logo">
    <span class="logo-icon">\u265E</span>
    <span class="logo-text">ChessMentor</span>
  </div>

  <div class="nav-items">
    {#each navItems as item}
      <button
        class="nav-item"
        class:active={currentPage === item.page}
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
    background: #1e293b;
    color: #e2e8f0;
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
    color: #94a3b8;
    cursor: pointer;
    font-size: 14px;
    width: 100%;
    text-align: left;
    transition: all 0.15s;
  }

  .nav-item:hover {
    color: #e2e8f0;
    background: rgba(255, 255, 255, 0.05);
  }

  .nav-item.active {
    color: white;
    background: rgba(255, 255, 255, 0.1);
    border-left: 3px solid #3b82f6;
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
