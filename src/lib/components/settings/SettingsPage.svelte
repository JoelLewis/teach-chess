<script lang="ts">
  import ThemeSwitcher from "./ThemeSwitcher.svelte";
  import ModelManager from "./ModelManager.svelte";
</script>

<div class="settings-page">
  <h2 class="page-title">Settings</h2>

  <ThemeSwitcher />

  <section class="section">
    <h3 class="section-title">AI Models</h3>
    <ModelManager />
  </section>
</div>

<style>
  .settings-page {
    padding: 24px;
    max-width: 800px;
  }

  .page-title {
    font-size: 22px;
    font-weight: 700;
    color: var(--cm-text-primary);
    margin-bottom: 24px;
  }

  .section {
    margin-bottom: 24px;
  }

  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--cm-text-primary);
    margin-bottom: 12px;
  }
</style>
