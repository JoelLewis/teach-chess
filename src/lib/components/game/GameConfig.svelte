<script lang="ts">
  import type { Color } from "../../api/bindings";
  import type { GameConfig, EngineStrength, PersonalityProfile, OpponentMode } from "../../api/bindings";
  import type { CoachingLevel } from "../../api/bindings";
  import { ENGINE_PRESETS } from "../../types/game";
  import LoadingSpinner from "../ui/LoadingSpinner.svelte";
  import Button from "../ui/Button.svelte";

  type Props = {
    onStart: (config: GameConfig) => void;
    starting?: boolean;
  };

  let { onStart, starting = false }: Props = $props();

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

  let advancedOpen = $state(false);

  function loadPreferences() {
    try {
      const saved = localStorage.getItem("chessMentor.gamePrefs");
      if (saved) {
        const prefs = JSON.parse(saved);
        if (prefs.playerColor) playerColor = prefs.playerColor;
        if (prefs.strengthPreset) strengthPreset = prefs.strengthPreset;
        if (prefs.coachingLevel) coachingLevel = prefs.coachingLevel;
        if (prefs.opponentMode) opponentMode = prefs.opponentMode;
        if (prefs.personality) personality = prefs.personality;
        if (prefs.teachingMode !== undefined) teachingMode = prefs.teachingMode;
      }
    } catch { /* ignore corrupt localStorage */ }
  }

  function savePreferences() {
    localStorage.setItem("chessMentor.gamePrefs", JSON.stringify({
      playerColor, strengthPreset, coachingLevel, opponentMode, personality, teachingMode,
    }));
  }

  loadPreferences();

  function handleStart() {
    savePreferences();
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
  <h2 class="config-heading">New Game</h2>

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

  <Button variant="primary" size="lg" onclick={handleStart} disabled={starting}>
    {#if starting}
      <LoadingSpinner size="sm" />
      Starting...
    {:else}
      Start Game
    {/if}
  </Button>

  <div class="advanced-section">
    <button class="disclosure-toggle" onclick={() => (advancedOpen = !advancedOpen)}>
      <span class="disclosure-arrow">{advancedOpen ? "▾" : "▸"}</span>
      Customize coaching, opponent style, and more...
    </button>

    <div class="disclosure-body" class:open={advancedOpen}>
      <div class="disclosure-inner">
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
            <p class="mode-desc">
              {opponentMode === "surprise"
                ? "A random personality will be assigned at game start."
                : "The coach will pick a style that challenges your weaknesses."}
            </p>
          {/if}
        </div>

        <div class="mb-4">
          <label class="flex items-center gap-2 text-sm">
            <input type="checkbox" bind:checked={teachingMode} />
            Teaching mode
            <span class="teaching-desc">— engine steers into positions that challenge your weak areas</span>
          </label>
        </div>
      </div>
    </div>

    {#if !advancedOpen}
      <p class="defaults-note">
        Defaults: {COACHING_OPTIONS.find(o => o.value === coachingLevel)?.label ?? "Full Coach"} coaching, {PERSONALITY_OPTIONS.find(o => o.value === personality)?.label ?? "Solid"} opponent
      </p>
    {/if}
  </div>
</div>

<style>
  .game-config {
    max-width: 400px;
    margin: 0 auto;
    padding: 24px;
    width: 100%;
    box-sizing: border-box;
  }

  .config-heading {
    font-size: 20px;
    font-weight: 600;
    margin-bottom: 24px;
    color: var(--cm-text-primary);
  }

  .mode-desc {
    font-size: 12px;
    color: var(--cm-text-muted);
    margin-top: 4px;
  }

  .teaching-desc {
    font-size: 12px;
    color: var(--cm-text-muted);
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

  .advanced-section {
    margin-top: 16px;
    border-top: 1px solid var(--cm-border-light);
    padding-top: 12px;
  }

  .disclosure-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: none;
    color: var(--cm-text-muted);
    font-size: 13px;
    cursor: pointer;
    padding: 4px 0;
  }

  .disclosure-toggle:hover {
    color: var(--cm-text-secondary);
  }

  .disclosure-arrow {
    font-size: 12px;
    width: 12px;
  }

  .disclosure-body {
    display: grid;
    grid-template-rows: 0fr;
    transition: grid-template-rows var(--cm-transition-normal);
  }

  .disclosure-body.open {
    grid-template-rows: 1fr;
  }

  .disclosure-inner {
    overflow: hidden;
    padding-top: 12px;
  }

  .defaults-note {
    font-size: 12px;
    color: var(--cm-text-muted);
    margin-top: 8px;
    font-style: italic;
  }
</style>
