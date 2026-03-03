<script lang="ts">
  import type { Color } from "../../types/chess";
  import type { GameConfig, EngineStrength, PersonalityProfile, OpponentMode } from "../../types/game";
  import type { CoachingLevel } from "../../types/engine";
  import { ENGINE_PRESETS } from "../../types/game";

  type Props = {
    onStart: (config: GameConfig) => void;
  };

  let { onStart }: Props = $props();

  let playerColor = $state<Color>("white");
  let strengthPreset = $state<keyof typeof ENGINE_PRESETS>("beginner");
  let customElo = $state(1350);
  let useCustom = $state(false);
  let coachingLevel = $state<CoachingLevel>("fullCoach");
  let opponentMode = $state<OpponentMode>("choose");
  let personality = $state<PersonalityProfile>("solid");
  let teachingMode = $state(false);

  let engineStrength = $derived<EngineStrength>(
    useCustom
      ? { elo: customElo, skillLevel: Math.round((customElo - 1320) / 94) }
      : ENGINE_PRESETS[strengthPreset],
  );

  const COACHING_OPTIONS: { value: CoachingLevel; label: string; desc: string }[] = [
    { value: "fullCoach", label: "Full Coach", desc: "Pre-move hints + post-move feedback for all moves" },
    { value: "lightTouch", label: "Light Touch", desc: "Post-move feedback for errors and best moves only" },
    { value: "minimal", label: "Minimal", desc: "Only alerts you about blunders" },
    { value: "silent", label: "Silent", desc: "No in-game coaching" },
  ];

  const PERSONALITY_OPTIONS: { value: PersonalityProfile; label: string; desc: string }[] = [
    { value: "aggressive", label: "Aggressive", desc: "Sharp, tactical play with king attacks" },
    { value: "positional", label: "Positional", desc: "Clean pawn structure and piece harmony" },
    { value: "trappy", label: "Trappy", desc: "Sets subtle traps and complications" },
    { value: "solid", label: "Solid", desc: "Prioritizes safety and avoids risk" },
  ];

  const MODE_OPTIONS: { value: OpponentMode; label: string; desc: string }[] = [
    { value: "choose", label: "Choose", desc: "Pick your opponent's style" },
    { value: "surprise", label: "Surprise me", desc: "Random personality each game" },
    { value: "coachPicks", label: "Coach picks", desc: "Targets your weak areas" },
  ];

  function handleStart() {
    onStart({
      playerColor,
      engineStrength,
      timeControl: { initialSecs: 0, incrementSecs: 0 },
      coachingLevel,
      opponentMode,
      personality: opponentMode === "choose" ? personality : null,
      teachingMode,
    });
  }
</script>

<div class="game-config">
  <h2 class="text-xl font-semibold mb-6">New Game</h2>

  <div class="mb-4">
    <span class="block text-sm font-medium mb-2">Play as</span>
    <div class="flex gap-2" role="group" aria-label="Play as">
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
    <span class="block text-sm font-medium mb-2">Engine Strength</span>
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

  <div class="mb-4">
    <span class="block text-sm font-medium mb-2">In-Game Coaching</span>
    <div class="coaching-options">
      {#each COACHING_OPTIONS as option}
        <button
          class="coaching-btn"
          class:active={coachingLevel === option.value}
          onclick={() => (coachingLevel = option.value)}
        >
          <span class="coaching-label">{option.label}</span>
          <span class="coaching-desc">{option.desc}</span>
        </button>
      {/each}
    </div>
  </div>

  <div class="mb-4">
    <span class="block text-sm font-medium mb-2">Opponent Style</span>
    <div class="flex gap-2 mb-2" role="group" aria-label="Opponent mode">
      {#each MODE_OPTIONS as option}
        <button
          class="btn text-xs"
          class:active={opponentMode === option.value}
          onclick={() => (opponentMode = option.value)}
          title={option.desc}
        >
          {option.label}
        </button>
      {/each}
    </div>
    {#if opponentMode === "choose"}
      <div class="coaching-options">
        {#each PERSONALITY_OPTIONS as option}
          <button
            class="coaching-btn"
            class:active={personality === option.value}
            onclick={() => (personality = option.value)}
          >
            <span class="coaching-label">{option.label}</span>
            <span class="coaching-desc">{option.desc}</span>
          </button>
        {/each}
      </div>
    {:else}
      <p class="text-xs text-gray-500 mt-1">
        {opponentMode === "surprise"
          ? "A random personality will be assigned at game start."
          : "The coach will pick a style that challenges your weaknesses."}
      </p>
    {/if}
    <label class="flex items-center gap-2 mt-3 text-sm">
      <input type="checkbox" bind:checked={teachingMode} />
      Teaching mode
      <span class="text-xs text-gray-500">— engine steers into positions that challenge your weak areas</span>
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
    border: 1px solid var(--cm-border-medium);
    border-radius: 6px;
    background: var(--cm-bg-surface);
    cursor: pointer;
    font-size: 14px;
    transition: all 0.15s;
  }

  .btn:hover {
    border-color: var(--cm-text-disabled);
  }

  .btn.active {
    background: var(--cm-accent-secondary-deep);
    color: var(--cm-text-inverse);
    border-color: var(--cm-accent-secondary-deep);
  }

  .btn-primary {
    padding: 10px 24px;
    background: var(--cm-accent-secondary-deep);
    color: var(--cm-text-inverse);
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
    font-weight: 500;
    transition: background 0.15s;
  }

  .btn-primary:hover {
    background: var(--cm-accent-secondary-deeper);
  }

  /* Grid: outlined primary button with glow */
  :global([data-theme="grid"]) .btn-primary {
    background: transparent;
    border: 1px solid var(--cm-accent-primary);
    color: var(--cm-accent-primary);
  }

  :global([data-theme="grid"]) .btn-primary:hover {
    background: rgba(0, 229, 255, 0.1);
    box-shadow: var(--cm-glow-primary);
  }

  .coaching-options {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .coaching-btn {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    padding: 8px 12px;
    border: 1px solid var(--cm-border-medium);
    border-radius: 6px;
    background: var(--cm-bg-surface);
    cursor: pointer;
    text-align: left;
    transition: all 0.15s;
  }

  .coaching-btn:hover {
    border-color: var(--cm-text-disabled);
  }

  .coaching-btn.active {
    background: var(--cm-accent-secondary-bg);
    border-color: var(--cm-accent-secondary-deep);
  }

  .coaching-label {
    font-size: 13px;
    font-weight: 500;
  }

  .coaching-desc {
    font-size: 11px;
    color: var(--cm-text-muted);
    margin-top: 1px;
  }

  .coaching-btn.active .coaching-label {
    color: var(--cm-accent-secondary-deep);
  }
</style>
