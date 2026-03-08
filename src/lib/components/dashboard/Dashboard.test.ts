import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";

describe("Dashboard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows loading state then loads data", async () => {
    const mockDashboardData = {
      skillProfile: {
        ratings: [],
        overallRating: 1200,
        strongestCategory: null,
        weakestCategory: null,
      },
      recentGames: [],
      puzzleStats: {
        totalAttempts: 0,
        totalSolved: 0,
        currentStreak: 0,
        bestStreak: 0,
      },
      dailyRecommendation: {
        text: "Start with some tactical puzzles",
        targetActivity: "problems",
        targetCategory: "tactical",
      },
      streak: {
        currentDays: 0,
        longestDays: 0,
        gamesToday: 0,
        puzzlesToday: 0,
      },
    };

    const mockAdaptivePrompt = {
      promptType: "none",
    };

    const mockedInvoke = vi.mocked(invoke);
    mockedInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "get_dashboard_data") return mockDashboardData;
      if (cmd === "check_adaptive_difficulty") return mockAdaptivePrompt;
      return null;
    });

    const { default: Dashboard } = await import("./Dashboard.svelte");
    render(Dashboard, {
      props: {
        onNavigate: vi.fn(),
        onReview: vi.fn(),
      },
    });

    // Should show loading initially
    expect(screen.getByText("Loading dashboard...")).toBeInTheDocument();

    // Wait for data to load
    await vi.waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("get_dashboard_data");
    });
  });
});
