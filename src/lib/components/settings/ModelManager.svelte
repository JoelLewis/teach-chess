<script lang="ts">
  import type { ModelStatus, LlmStatus } from "../../types/engine";
  import * as api from "../../api/commands";
  import { onLlmDownloadProgress } from "../../api/events";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  let models = $state<ModelStatus[]>([]);
  let llmStatus = $state<LlmStatus | null>(null);
  let downloading = $state<string | null>(null);
  let downloadProgress = $state({ downloaded: 0, total: 0 });

  async function loadData() {
    try {
      const [m, s] = await Promise.all([
        api.getAvailableModels(),
        api.getLlmStatus(),
      ]);
      models = m;
      llmStatus = s;
      // Extract system memory from first model's metadata if available
      if (m.length > 0 && m[0].systemMemoryMb) {
        systemMemoryMb = m[0].systemMemoryMb;
      }
    } catch (err) {
      console.error("Failed to load model data:", err);
    }
  }

  async function handleDownload(modelId: string) {
    downloading = modelId;
    downloadProgress = { downloaded: 0, total: 0 };
    try {
      await api.downloadModel(modelId);
      await loadData();
    } catch (err) {
      console.error("Download failed:", err);
    } finally {
      downloading = null;
    }
  }

  let systemMemoryMb = $state(0);

  function formatSize(mb: number): string {
    if (mb >= 1000) return `${(mb / 1000).toFixed(1)} GB`;
    return `${mb} MB`;
  }

  function formatBytes(bytes: number): string {
    if (bytes >= 1_000_000_000) return `${(bytes / 1_000_000_000).toFixed(1)} GB`;
    if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(0)} MB`;
    return `${(bytes / 1_000).toFixed(0)} KB`;
  }

  function downloadPercent(): number {
    if (downloadProgress.total <= 0) return 0;
    return Math.round((downloadProgress.downloaded / downloadProgress.total) * 100);
  }

  function modelFitsMemory(ramMb: number): boolean {
    return systemMemoryMb > 0 && ramMb <= systemMemoryMb;
  }

  function isActiveModel(modelId: string): boolean {
    return llmStatus?.modelLoaded === true && llmStatus?.modelId === modelId;
  }

  $effect(() => {
    let unlisten: UnlistenFn | undefined;
    onLlmDownloadProgress((p) => {
      downloadProgress = { downloaded: p.downloadedBytes, total: p.totalBytes };
    }).then((fn) => (unlisten = fn));

    loadData();

    return () => unlisten?.();
  });
</script>

<div class="settings-page">
  <h1 class="title">Settings</h1>

  <section class="section">
    <h2 class="section-title">AI Coaching Model</h2>
    <p class="section-desc">
      Download a local AI model for enhanced coaching feedback.
      Without a model, coaching uses pre-written templates.
    </p>

    {#if llmStatus}
      <div class="status-card">
        <div class="status-row">
          <span class="status-label">Mode</span>
          <span class="status-value" class:active={llmStatus.mode === "llm"}>
            {llmStatus.mode === "llm" ? "AI Coaching" : "Basic Coaching"}
          </span>
        </div>
        {#if llmStatus.modelLoaded}
          <div class="status-row">
            <span class="status-label">Model</span>
            <span class="status-value">{llmStatus.modelId ?? "Unknown"}</span>
          </div>
        {/if}
        {#if systemMemoryMb > 0}
          <div class="status-row">
            <span class="status-label">System RAM</span>
            <span class="status-value">{formatSize(systemMemoryMb)}</span>
          </div>
        {/if}
      </div>
    {/if}

    <div class="model-list">
      {#each models as model}
        <div class="model-card" class:model-active={isActiveModel(model.id)}>
          <div class="model-info">
            <div class="model-name-row">
              <h3 class="model-name">{model.displayName}</h3>
              {#if isActiveModel(model.id)}
                <span class="active-badge">Active</span>
              {/if}
              {#if modelFitsMemory(model.ramRequirementMb)}
                <span class="recommended-badge">Recommended</span>
              {/if}
            </div>
            <div class="model-meta">
              <span>Size: {formatSize(model.fileSizeMb)}</span>
              <span class="ram-indicator" class:ram-ok={modelFitsMemory(model.ramRequirementMb)} class:ram-warn={systemMemoryMb > 0 && !modelFitsMemory(model.ramRequirementMb)}>
                RAM: {formatSize(model.ramRequirementMb)}
              </span>
            </div>
          </div>
          <div class="model-actions">
            {#if model.downloaded}
              <span class="downloaded-badge">Downloaded</span>
            {:else if downloading === model.id}
              <div class="download-progress">
                <div class="progress-bar">
                  <div
                    class="progress-fill"
                    style="width: {downloadPercent()}%"
                  ></div>
                </div>
                <span class="progress-text">
                  {downloadPercent()}%
                  {#if downloadProgress.total > 0}
                    — {formatBytes(downloadProgress.downloaded)} / {formatBytes(downloadProgress.total)}
                  {/if}
                </span>
              </div>
            {:else}
              <button
                class="download-btn"
                onclick={() => handleDownload(model.id)}
              >
                Download
              </button>
            {/if}
          </div>
        </div>
      {/each}

      {#if models.length === 0}
        <p class="empty-text">
          AI coaching requires the app to be built with the LLM feature enabled.
        </p>
      {/if}
    </div>
  </section>
</div>

<style>
  .settings-page {
    padding: 32px;
    max-width: 640px;
  }

  .title {
    font-size: 24px;
    font-weight: 700;
    color: #1e293b;
    margin-bottom: 24px;
  }

  .section {
    margin-bottom: 32px;
  }

  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: #334155;
    margin-bottom: 4px;
  }

  .section-desc {
    font-size: 13px;
    color: #64748b;
    margin-bottom: 16px;
  }

  .status-card {
    background: white;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    padding: 12px 16px;
    margin-bottom: 16px;
  }

  .status-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 0;
  }

  .status-label {
    font-size: 13px;
    color: #64748b;
  }

  .status-value {
    font-size: 13px;
    font-weight: 500;
    color: #334155;
  }

  .status-value.active {
    color: #16a34a;
  }

  .model-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .model-card {
    background: white;
    border: 1px solid #e2e8f0;
    border-radius: 8px;
    padding: 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
  }

  .model-active {
    border-color: #a5b4fc;
    background: #fafbff;
  }

  .model-name-row {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 4px;
  }

  .model-name {
    font-size: 14px;
    font-weight: 600;
    color: #1e293b;
  }

  .active-badge {
    font-size: 10px;
    font-weight: 600;
    color: #16a34a;
    background: #f0fdf4;
    padding: 2px 6px;
    border-radius: 3px;
  }

  .recommended-badge {
    font-size: 10px;
    font-weight: 600;
    color: #1e40af;
    background: #eff6ff;
    padding: 2px 6px;
    border-radius: 3px;
  }

  .ram-indicator.ram-ok {
    color: #16a34a;
  }

  .ram-indicator.ram-warn {
    color: #dc2626;
  }

  .model-meta {
    display: flex;
    gap: 16px;
    font-size: 12px;
    color: #64748b;
  }

  .downloaded-badge {
    font-size: 12px;
    color: #16a34a;
    font-weight: 500;
    padding: 4px 12px;
    background: #f0fdf4;
    border-radius: 4px;
  }

  .download-btn {
    padding: 6px 16px;
    background: #1e40af;
    color: white;
    border: none;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
  }

  .download-btn:hover {
    background: #1e3a8a;
  }

  .download-progress {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 120px;
  }

  .progress-bar {
    height: 4px;
    background: #e5e7eb;
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: #3b82f6;
    transition: width 0.3s;
  }

  .progress-text {
    font-size: 11px;
    color: #64748b;
  }

  .empty-text {
    font-size: 13px;
    color: #94a3b8;
    text-align: center;
    padding: 24px;
  }
</style>
