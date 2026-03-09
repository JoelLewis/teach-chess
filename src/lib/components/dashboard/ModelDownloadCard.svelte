<script lang="ts">
  import * as api from "../../api/commands";

  type Props = {
    onNavigate: (page: string) => void;
  };

  let { onNavigate }: Props = $props();

  let dismissed = $state(false);
  let downloading = $state(false);
  let downloaded = $state(false);
  let error = $state<string | null>(null);

  async function handleDownload() {
    downloading = true;
    error = null;
    try {
      await api.downloadModel("gemma-2-2b-it-q4");
      downloaded = true;
    } catch (err) {
      error = `Download failed: ${err}`;
      console.error("Model download failed:", err);
    } finally {
      downloading = false;
    }
  }
</script>

{#if !dismissed}
  <div class="model-card" class:model-card-success={downloaded}>
    {#if downloaded}
      <div class="card-body">
        <div class="card-icon">&#10003;</div>
        <div class="card-text">
          <h3 class="card-title">AI Coach Ready</h3>
          <p class="card-desc">Gemma 2B is downloaded. You'll get personalized coaching during games.</p>
        </div>
      </div>
      <div class="card-actions">
        <button class="btn-secondary" onclick={() => { dismissed = true; }}>Got it</button>
      </div>
    {:else if downloading}
      <div class="card-body">
        <div class="card-icon spinner-icon">&#9881;</div>
        <div class="card-text">
          <h3 class="card-title">Downloading AI Coach...</h3>
          <p class="card-desc">Downloading Gemma 2B (~1.5 GB). This may take a few minutes.</p>
        </div>
      </div>
      <div class="progress-bar">
        <div class="progress-indeterminate"></div>
      </div>
    {:else}
      <div class="card-body">
        <div class="card-icon">&#9733;</div>
        <div class="card-text">
          <h3 class="card-title">Enhance Your Coaching</h3>
          <p class="card-desc">
            Download a local AI model for personalized feedback during games. Without it, coaching uses basic templates.
          </p>
          <p class="card-size">~1.5 GB download — runs locally, no internet needed after install</p>
        </div>
      </div>
      {#if error}
        <p class="card-error">{error}</p>
      {/if}
      <div class="card-actions">
        <button class="btn-primary" onclick={handleDownload}>Download Gemma 2B</button>
        <button class="btn-dismiss" onclick={() => { dismissed = true; }}>Not now</button>
        <button class="btn-dismiss" onclick={() => onNavigate("settings")}>Settings</button>
      </div>
    {/if}
  </div>
{/if}

<style>
  .model-card {
    background: var(--cm-bg-surface);
    border: 1px solid var(--cm-accent-secondary);
    border-radius: 10px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .model-card-success {
    border-color: var(--cm-status-success);
  }

  .card-body {
    display: flex;
    gap: 12px;
    align-items: flex-start;
  }

  .card-icon {
    font-size: 24px;
    line-height: 1;
    flex-shrink: 0;
    margin-top: 2px;
  }

  .spinner-icon {
    animation: spin 2s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .card-text {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .card-title {
    font-size: 15px;
    font-weight: 600;
    color: var(--cm-text-primary);
    margin: 0;
  }

  .card-desc {
    font-size: 13px;
    color: var(--cm-text-secondary);
    margin: 0;
  }

  .card-size {
    font-size: 11px;
    color: var(--cm-text-muted);
    margin: 0;
  }

  .card-error {
    font-size: 12px;
    color: var(--cm-status-error);
    margin: 0;
  }

  .card-actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .btn-primary {
    padding: 8px 16px;
    background: var(--cm-accent-secondary-deep);
    color: var(--cm-text-inverse);
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn-primary:hover {
    background: var(--cm-accent-secondary-deeper);
  }

  .btn-secondary {
    padding: 8px 16px;
    background: var(--cm-status-success);
    color: var(--cm-text-inverse);
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }

  .btn-dismiss {
    padding: 8px 12px;
    background: none;
    border: none;
    font-size: 12px;
    color: var(--cm-text-muted);
    cursor: pointer;
  }

  .btn-dismiss:hover {
    color: var(--cm-text-secondary);
  }

  .progress-bar {
    height: 4px;
    background: var(--cm-border-light);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-indeterminate {
    height: 100%;
    width: 30%;
    background: var(--cm-accent-secondary);
    border-radius: 2px;
    animation: slide 1.5s ease-in-out infinite;
  }

  @keyframes slide {
    0% { transform: translateX(-100%); }
    100% { transform: translateX(433%); }
  }
</style>
