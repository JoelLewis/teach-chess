<script lang="ts">
  import Sidebar from "./lib/components/layout/Sidebar.svelte";
  import Header from "./lib/components/layout/Header.svelte";
  import GameConfigForm from "./lib/components/game/GameConfig.svelte";
  import PlayScreen from "./lib/components/game/PlayScreen.svelte";
  import GameHistory from "./lib/components/history/GameHistory.svelte";
  import ReviewScreen from "./lib/components/review/ReviewScreen.svelte";
  import { gameStore } from "./lib/stores/game.svelte";
  import { playerStore } from "./lib/stores/player.svelte";
  import * as api from "./lib/api/commands";
  import type { GameConfig } from "./lib/types/game";

  type Page = "home" | "play" | "history" | "review";
  let page = $state<Page>("home");
  let reviewGameId = $state("");

  function navigate(target: Page) {
    page = target;
  }

  async function startGame(config: GameConfig) {
    try {
      await api.startEngine();
      const position = await api.newGame(config);
      gameStore.config = config;
      gameStore.position = position;
      gameStore.phase = "playing";
      page = "play";
    } catch (err) {
      console.error("Failed to start game:", err);
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
</script>

<div class="app-layout">
  <Sidebar currentPage={page} onNavigate={navigate} />

  <div class="main-area">
    <Header playerName={playerStore.displayName} />

    <main class="content">
      {#if page === "home"}
        <div class="home">
          <div class="hero">
            <h1 class="text-3xl font-bold text-gray-800 mb-2">Welcome to ChessMentor</h1>
            <p class="text-gray-600 mb-8">Your AI chess coach. Play, learn, improve.</p>
            <button class="cta-btn" onclick={() => navigate("play")}>
              Start a Game
            </button>
          </div>
        </div>
      {:else if page === "play" && gameStore.phase === "idle"}
        <GameConfigForm onStart={startGame} />
      {:else if page === "play" && (gameStore.phase === "playing" || gameStore.phase === "game-over")}
        <PlayScreen
          onReview={handleReview}
          onNewGame={handleNewGame}
        />
      {:else if page === "history"}
        <GameHistory onReview={handleReview} />
      {:else if page === "review" && reviewGameId}
        <ReviewScreen gameId={reviewGameId} onBack={() => navigate("history")} />
      {:else}
        <GameConfigForm onStart={startGame} />
      {/if}
    </main>
  </div>
</div>

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

  .home {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    min-height: 400px;
  }

  .hero {
    text-align: center;
  }

  .cta-btn {
    padding: 14px 36px;
    background: #1e40af;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 18px;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s;
  }

  .cta-btn:hover {
    background: #1e3a8a;
  }
</style>
