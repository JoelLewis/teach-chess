import { describe, it, expect, vi, beforeEach } from "vitest";
import { render } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { puzzleStore } from "../../stores/puzzle.svelte";

// Mock the Chessboard component (uses chessground which needs real DOM)
// Uses __mocks__/Chessboard.svelte in the board directory
vi.mock("../board/Chessboard.svelte");

// Mock ProblemPanel too since it has its own dependencies
// Uses __mocks__/ProblemPanel.svelte in the problems directory
vi.mock("./ProblemPanel.svelte");

describe("ProblemScreen", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    puzzleStore.reset();
  });

  it("loads first puzzle on mount", async () => {
    const mockPuzzleState = {
      puzzle: {
        id: "puzzle-1",
        fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        moves: "e2e4",
        themes: "tactical",
        difficulty: 1200,
      },
      startFen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
      playerColor: "white" as const,
      legalDests: {},
    };

    const mockStats = {
      totalAttempts: 0,
      totalSolved: 0,
      currentStreak: 0,
      bestStreak: 0,
    };

    const mockedInvoke = vi.mocked(invoke);
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "load_next_puzzle") return mockPuzzleState;
      if (cmd === "get_puzzle_stats") return mockStats;
      return null;
    });

    // Verify the puzzle store starts in idle phase
    expect(puzzleStore.phase).toBe("idle");

    // Import and render the component
    const { default: ProblemScreen } = await import("./ProblemScreen.svelte");
    render(ProblemScreen);

    // Wait for the effect to fire and the async loadNextPuzzle to complete
    await vi.waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("load_next_puzzle", expect.any(Object));
    });
  });
});
