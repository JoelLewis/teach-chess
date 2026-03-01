<script lang="ts">
  import type { Color } from "../../types/chess";
  import type { GameConfig, EngineStrength } from "../../types/game";
  import { ENGINE_PRESETS } from "../../types/game";

  type Props = {
    onStart: (config: GameConfig) => void;
  };

  let { onStart }: Props = $props();

  let playerColor = $state<Color>("white");
  let strengthPreset = $state<keyof typeof ENGINE_PRESETS>("beginner");
  let customElo = $state(1350);
  let useCustom = $state(false);

  let engineStrength = $derived<EngineStrength>(
    useCustom
      ? { elo: customElo, skillLevel: Math.round((customElo - 1320) / 94) }
      : ENGINE_PRESETS[strengthPreset],
  );

  function handleStart() {
    onStart({
      playerColor,
      engineStrength,
      timeControl: { initialSecs: 0, incrementSecs: 0 },
    });
  }
</script>

<div class="game-config">
  <h2 class="text-xl font-semibold mb-6">New Game</h2>

  <div class="mb-4">
    <label class="block text-sm font-medium mb-2">Play as</label>
    <div class="flex gap-2">
      <button
        class="btn"
        class:active={playerColor === "white"}
        onclick={() => (playerColor = "white")}
      >
        White
      </button>
      <button
        class="btn"
        class:active={playerColor === "black"}
        onclick={() => (playerColor = "black")}
      >
        Black
      </button>
    </div>
  </div>

  <div class="mb-4">
    <label class="block text-sm font-medium mb-2">Engine Strength</label>
    {#if !useCustom}
      <div class="flex flex-wrap gap-2">
        {#each Object.keys(ENGINE_PRESETS) as preset}
          <button
            class="btn"
            class:active={strengthPreset === preset}
            onclick={() => (strengthPreset = preset as keyof typeof ENGINE_PRESETS)}
          >
            {preset.charAt(0).toUpperCase() + preset.slice(1)}
            <span class="text-xs opacity-70 ml-1">
              ({ENGINE_PRESETS[preset as keyof typeof ENGINE_PRESETS].elo})
            </span>
          </button>
        {/each}
      </div>
    {:else}
      <div class="flex items-center gap-3">
        <input
          type="range"
          min="1320"
          max="3190"
          bind:value={customElo}
          class="flex-1"
        />
        <span class="font-mono text-sm w-12">{customElo}</span>
      </div>
    {/if}
    <label class="flex items-center gap-2 mt-2 text-sm">
      <input type="checkbox" bind:checked={useCustom} />
      Custom ELO
    </label>
  </div>

  <button class="btn-primary mt-4" onclick={handleStart}>
    Start Game
  </button>
</div>

<style>
  .game-config {
    max-width: 400px;
    padding: 24px;
  }

  .btn {
    padding: 8px 16px;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: white;
    cursor: pointer;
    font-size: 14px;
    transition: all 0.15s;
  }

  .btn:hover {
    border-color: #9ca3af;
  }

  .btn.active {
    background: #1e40af;
    color: white;
    border-color: #1e40af;
  }

  .btn-primary {
    padding: 10px 24px;
    background: #1e40af;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
    font-weight: 500;
    transition: background 0.15s;
  }

  .btn-primary:hover {
    background: #1e3a8a;
  }
</style>
