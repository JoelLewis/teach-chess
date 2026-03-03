<script lang="ts">
  import Chessboard from "../board/Chessboard.svelte";
  import ProblemPanel from "./ProblemPanel.svelte";
  import { puzzleStore } from "../../stores/puzzle.svelte";
  import { errorStore } from "../../stores/error.svelte";
  import * as api from "../../api/commands";

  const fen = $derived(puzzleStore.currentFen);
  const orientation = $derived(puzzleStore.playerColor);
  const phase = $derived(puzzleStore.phase);

  // Only allow moves when solving or retrying after incorrect
  const canMove = $derived(phase === "solving" || phase === "incorrect");

  // Turn color matches player color (it's always the player's turn in puzzles)
  const turnColor = $derived(canMove ? orientation : undefined);
  const dests = $derived(canMove ? puzzleStore.legalDests : {});

  const lastMove = $derived(puzzleStore.lastMoveResult?.lastMove ?? null);

  async function loadNextPuzzle() {
    puzzleStore.phase = "loading";
    try {
      const state = await api.loadNextPuzzle(puzzleStore.filter);
      puzzleStore.currentPuzzle = state;
      puzzleStore.lastMoveResult = null;
      puzzleStore.hintsRevealed = [];
      puzzleStore.explanation = null;
      puzzleStore.phase = "solving";

      // Refresh stats
      const stats = await api.getPuzzleStats();
      puzzleStore.sessionStats = stats;
    } catch (err) {
      errorStore.show(`No puzzles available: ${err}`);
      puzzleStore.phase = "idle";
    }
  }

  async function handleMove(from: string, to: string) {
    if (!canMove) return;

    const uci = `${from}${to}`;

    try {
      const result = await api.submitPuzzleMove(uci);
      puzzleStore.lastMoveResult = result;

      if (result.correct) {
        if (result.isComplete) {
          // Puzzle solved! Save result and show explanation
          puzzleStore.explanation = result.explanation ?? null;
          try {
            await api.savePuzzleResult(true);
          } catch (err) {
            console.error("Failed to save puzzle result:", err);
          }
          puzzleStore.phase = "complete";

          // Refresh stats
          const stats = await api.getPuzzleStats();
          puzzleStore.sessionStats = stats;
        } else {
          // Show brief correct feedback, then auto-continue
          puzzleStore.phase = "correct";
          setTimeout(() => {
            if (puzzleStore.phase === "correct") {
              puzzleStore.phase = "solving";
            }
          }, 600);
        }
      } else {
        puzzleStore.phase = "incorrect";
      }
    } catch (err) {
      errorStore.show(`Move error: ${err}`);
    }
  }

  // Load stats on mount
  $effect(() => {
    api.getPuzzleStats().then((stats) => {
      puzzleStore.sessionStats = stats;
    }).catch(() => {
      // Stats unavailable is non-fatal
    });
  });
</script>

<div class="problem-screen">
  <div class="board-area">
    <Chessboard
      fen={fen ?? undefined}
      {orientation}
      turnColor={turnColor}
      dests={dests}
      viewOnly={!canMove}
      {lastMove}
      onMove={handleMove}
    />
  </div>

  <ProblemPanel onLoadNext={loadNextPuzzle} />
</div>

<style>
  .problem-screen {
    display: flex;
    gap: 24px;
    padding: 24px;
    height: 100%;
    align-items: flex-start;
    justify-content: center;
  }

  .board-area {
    display: flex;
    gap: 8px;
    align-items: center;
  }
</style>
