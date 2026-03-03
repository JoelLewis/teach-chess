<script lang="ts">
  import Sidebar from "./lib/components/layout/Sidebar.svelte";
  import Header from "./lib/components/layout/Header.svelte";
  import ErrorToast from "./lib/components/layout/ErrorToast.svelte";
  import Dashboard from "./lib/components/dashboard/Dashboard.svelte";
  import GameConfigForm from "./lib/components/game/GameConfig.svelte";
  import PlayScreen from "./lib/components/game/PlayScreen.svelte";
  import GameHistory from "./lib/components/history/GameHistory.svelte";
  import ReviewScreen from "./lib/components/review/ReviewScreen.svelte";
  import ModelManager from "./lib/components/settings/ModelManager.svelte";
  import ProblemScreen from "./lib/components/problems/ProblemScreen.svelte";
  import OpeningsScreen from "./lib/components/openings/OpeningsScreen.svelte";
  import { gameStore } from "./lib/stores/game.svelte";
  import { playerStore } from "./lib/stores/player.svelte";
  import { errorStore } from "./lib/stores/error.svelte";
  import * as api from "./lib/api/commands";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import type { GameConfig } from "./lib/types/game";

  type Page = "home" | "play" | "problems" | "openings" | "history" | "review" | "settings";
  let page = $state<Page>("home");
  let reviewGameId = $state("");

  function navigate(target: Page) {
    page = target;
  }

  async function startGame(config: GameConfig) {
    try {
      await api.startEngine();

      // Resolve opponent personality before the first move
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

      // Load weak categories for teaching mode
      if (config.teachingMode) {
        try {
          const profile = await api.getSkillProfile();
          const sorted = [...profile.ratings].sort((a, b) => a.rating - b.rating);
          gameStore.weakCategories = sorted.slice(0, 2).map((r) => r.category);
        } catch {
          // Non-blocking — teaching mode still works without weak categories
        }
      }

      const position = await api.newGame(config);
      gameStore.config = config;
      gameStore.position = position;
      gameStore.phase = "playing";
      page = "play";
    } catch (err) {
      console.error("Failed to start game:", err);
      errorStore.show(`Failed to start game: ${err}`);
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
      await appWindow.close();
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  });
</script>

<div class="app-layout">
  <Sidebar currentPage={page} onNavigate={navigate} />

  <div class="main-area">
    <Header playerName={playerStore.displayName} />

    <main class="content">
      {#if page === "home"}
        <Dashboard onNavigate={(p) => navigate(p as Page)} onReview={handleReview} />
      {:else if page === "play" && gameStore.phase === "idle"}
        <GameConfigForm onStart={startGame} />
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
        <ModelManager />
      {:else}
        <GameConfigForm onStart={startGame} />
      {/if}
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
    background: #f1f5f9;
  }

  .content {
    flex: 1;
    overflow-y: auto;
  }

</style>
