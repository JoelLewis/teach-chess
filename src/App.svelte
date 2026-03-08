<script lang="ts">
  import Sidebar from "./lib/components/layout/Sidebar.svelte";
  import Header from "./lib/components/layout/Header.svelte";
  import ErrorToast from "./lib/components/layout/ErrorToast.svelte";
  import Dashboard from "./lib/components/dashboard/Dashboard.svelte";
  import GameConfigForm from "./lib/components/game/GameConfig.svelte";
  import PlayScreen from "./lib/components/game/PlayScreen.svelte";
  import GameHistory from "./lib/components/history/GameHistory.svelte";
  import ReviewScreen from "./lib/components/review/ReviewScreen.svelte";
  import SettingsPage from "./lib/components/settings/SettingsPage.svelte";
  import ProblemScreen from "./lib/components/problems/ProblemScreen.svelte";
  import OpeningsScreen from "./lib/components/openings/OpeningsScreen.svelte";
  import { gameStore } from "./lib/stores/game.svelte";
  import { playerStore } from "./lib/stores/player.svelte";
  import { errorStore } from "./lib/stores/error.svelte";
  import { themeStore } from "./lib/stores/theme.svelte";
  import * as api from "./lib/api/commands";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import type { GameConfig } from "./lib/types/game";

  type Page = "home" | "play" | "problems" | "openings" | "history" | "review" | "settings";
  let page = $state<Page>("home");
  let reviewGameId = $state("");
  let gameStarting = $state(false);

  // Unique key for screen transitions — changes when the displayed screen changes
  let screenKey = $derived(page === "play" ? `play-${gameStore.phase}` : page);

  function navigate(target: Page) {
    page = target;
  }

  async function startGame(config: GameConfig) {
    gameStarting = true;
    try {
      // Run engine start in parallel with personality/skill profile resolution
      const personalityPromise = (async () => {
        try {
          const resolved = await api.resolvePersonality(
            config.opponentMode,
            config.personality ?? undefined,
          );
          gameStore.resolvedPersonality = resolved;
        } catch (err) {
          console.error("Personality resolution failed (non-blocking):", err);
          gameStore.resolvedPersonality = null;
        }

        // Load weak categories for teaching mode (depends on nothing)
        if (config.teachingMode) {
          try {
            const profile = await api.getSkillProfile();
            const sorted = [...profile.ratings].sort((a, b) => a.rating - b.rating);
            gameStore.weakCategories = sorted.slice(0, 2).map((r) => r.category);
          } catch {
            // Non-blocking — teaching mode still works without weak categories
          }
        }
      })();

      await Promise.all([api.startEngine(), personalityPromise]);

      const position = await api.newGame(config);
      gameStore.config = config;
      gameStore.position = position;
      gameStore.phase = "playing";
      page = "play";
    } catch (err) {
      console.error("Failed to start game:", err);
      errorStore.show(`Failed to start game: ${err}`);
    } finally {
      gameStarting = false;
    }
  }

  function handleReview(gameId: string) {
    reviewGameId = gameId;
    page = "review";
  }

  function handleNewGame() {
    gameStore.reset();
    page = "home";
  }

  // Initialize theme on startup
  $effect(() => {
    themeStore.load();
  });

  // Initialize player on startup
  $effect(() => {
    api.getOrCreatePlayer("Player").then((player) => {
      playerStore.id = player.id;
      playerStore.displayName = player.displayName;
      playerStore.gamesPlayed = player.gamesPlayed;
    }).catch((err) => {
      console.error("Failed to initialize player:", err);
      errorStore.show("Failed to initialize player profile");
    });
  });

  // Clean up engine when window closes
  $effect(() => {
    let closing = false;
    const appWindow = getCurrentWindow();
    const unlistenPromise = appWindow.onCloseRequested(async (event) => {
      if (closing) return;
      event.preventDefault();
      closing = true;
      try {
        await api.stopEngine();
      } catch {
        // Ignore cleanup errors
      }
      await appWindow.destroy();
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  });
</script>

<a href="#main-content" class="skip-link">Skip to content</a>
<div class="app-layout">
  <Sidebar currentPage={page} onNavigate={navigate} />

  <div class="main-area">
    <Header playerName={playerStore.displayName} />

    <main class="content" id="main-content">
      {#key screenKey}
        <div class="screen-transition">
          {#if page === "home"}
            <Dashboard onNavigate={(p) => navigate(p as Page)} onReview={handleReview} />
          {:else if page === "play" && gameStore.phase === "idle"}
            <GameConfigForm onStart={startGame} starting={gameStarting} />
          {:else if page === "play" && (gameStore.phase === "playing" || gameStore.phase === "game-over")}
            <PlayScreen
              onReview={handleReview}
              onNewGame={handleNewGame}
            />
          {:else if page === "problems"}
            <ProblemScreen />
          {:else if page === "openings"}
            <OpeningsScreen />
          {:else if page === "history"}
            <GameHistory onReview={handleReview} />
          {:else if page === "review" && reviewGameId}
            <ReviewScreen gameId={reviewGameId} onBack={() => navigate("history")} />
          {:else if page === "settings"}
            <SettingsPage />
          {:else}
            <GameConfigForm onStart={startGame} starting={gameStarting} />
          {/if}
        </div>
      {/key}
    </main>
  </div>
</div>

<ErrorToast />

<style>
  .app-layout {
    display: flex;
    min-height: 100vh;
  }

  .main-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: var(--cm-bg-app);
  }

  /* Study: linen grain via procedural SVG noise */
  :global([data-theme="study"]) .main-area {
    background-color: var(--cm-bg-app);
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='300' height='300'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.65' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.04'/%3E%3C/svg%3E");
    background-repeat: repeat;
  }

  /* Grid: vignette gradient */
  :global([data-theme="grid"]) .main-area {
    background: radial-gradient(
      ellipse at 60% 45%,
      #12121a 0%,
      #0a0a0f 70%
    );
  }

  .content {
    flex: 1;
    overflow-y: auto;
  }

  :global(.skip-link) {
    position: absolute;
    top: -100%;
    left: 16px;
    z-index: 1000;
    padding: 8px 16px;
    background: var(--cm-accent-primary);
    color: var(--cm-text-inverse);
    border-radius: 0 0 6px 6px;
    text-decoration: none;
    font-size: 14px;
  }

  :global(.skip-link:focus) {
    top: 0;
  }

  .screen-transition {
    animation: fade-in 0.15s ease;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

</style>
