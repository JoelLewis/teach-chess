import { test, expect, type Page } from "@playwright/test";

// Regression tests for board click handling on the puzzle screen.
//
// The board mounts while the puzzle is still loading (viewOnly: true).
// Chessground binds its pointer listeners and bounds-invalidation listeners
// once at construction and skips them under viewOnly, so the board must be
// remounted (or bounds refreshed) when interactivity or layout changes.
// These tests drive the real frontend with Tauri IPC mocked at the
// __TAURI_INTERNALS__ layer.

type MockConfig = {
  puzzle: unknown;
  onboardingComplete: boolean;
};

const WHITE_PUZZLE = {
  puzzle: {
    id: "test-white",
    fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    solutionMoves: "e2e4",
    themes: "opening",
    category: "tactical",
    difficulty: 1000,
    sourceId: null,
    hintsJson: "[]",
    explanation: "",
  },
  startFen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
  playerColor: "white",
  legalDests: { e2: ["e3", "e4"], d2: ["d3", "d4"] },
  totalPlayerMoves: 1,
  currentMoveIndex: 0,
};

const BLACK_PUZZLE = {
  puzzle: {
    id: "test-black",
    fen: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
    solutionMoves: "e7e5",
    themes: "opening",
    category: "tactical",
    difficulty: 1000,
    sourceId: null,
    hintsJson: "[]",
    explanation: "",
  },
  startFen: "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
  playerColor: "black",
  legalDests: { e7: ["e6", "e5"], d7: ["d6", "d5"] },
  totalPlayerMoves: 1,
  currentMoveIndex: 0,
};

function mockTauri(cfg: MockConfig) {
  if (cfg.onboardingComplete) {
    localStorage.setItem("chessMentor.onboardingComplete", "true");
  }
  let callbackId = 0;
  const internals = {
    metadata: {
      currentWindow: { label: "main" },
      currentWebview: { label: "main" },
    },
    plugins: {},
    transformCallback: () => ++callbackId,
    invoke: async (cmd: string) => {
      switch (cmd) {
        case "get_or_create_player":
          return { id: "p1", displayName: "Player", gamesPlayed: 0 };
        case "get_theme":
          return "study";
        case "load_next_puzzle":
          return cfg.puzzle;
        case "get_puzzle_state":
          return null;
        case "get_puzzle_stats":
          return {
            totalAttempts: 0,
            totalSolved: 0,
            averageTimeMs: 0,
            currentStreak: 0,
            bestStreak: 0,
          };
        case "get_skill_profile":
          return { ratings: [] };
        case "get_dashboard_data":
          return {
            skillProfile: { ratings: [] },
            recentGames: [],
            puzzleStats: {
              totalAttempts: 0,
              totalSolved: 0,
              averageTimeMs: 0,
              currentStreak: 0,
              bestStreak: 0,
            },
            dailyRecommendation: {
              text: "Solve a puzzle",
              targetActivity: "problems",
              targetCategory: null,
            },
            streak: { currentDays: 0, longestDays: 0, gamesToday: 0, puzzlesToday: 0 },
          };
        default:
          return null;
      }
    },
  };
  (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = internals;
}

type Point = { x: number; y: number };
type Box = { x: number; y: number; width: number; height: number };

function squareCenter(box: Box, key: string, orientation: "white" | "black"): Point {
  const file = "abcdefgh".indexOf(key[0]);
  const rank = parseInt(key[1], 10) - 1;
  const col = orientation === "white" ? file : 7 - file;
  const row = orientation === "white" ? 7 - rank : rank;
  const sq = box.width / 8;
  return { x: box.x + (col + 0.5) * sq, y: box.y + (row + 0.5) * sq };
}

async function clickSquare(page: Page, key: string, orientation: "white" | "black"): Promise<Point> {
  const box = await page.locator("cg-board").boundingBox();
  if (!box) throw new Error("board not visible");
  const pt = squareCenter(box, key, orientation);
  await page.mouse.click(pt.x, pt.y);
  return pt;
}

async function expectSelectedAt(page: Page, pt: Point) {
  const selected = page.locator("square.selected");
  await expect(selected).toBeVisible();
  const box = await selected.boundingBox();
  if (!box) throw new Error("selected square not visible");
  expect(Math.abs(box.x + box.width / 2 - pt.x)).toBeLessThan(box.width / 2);
  expect(Math.abs(box.y + box.height / 2 - pt.y)).toBeLessThan(box.height / 2);
}

test("puzzle board accepts clicks when player is white (no orientation flip)", async ({ page }) => {
  await page.addInitScript(mockTauri, { puzzle: WHITE_PUZZLE, onboardingComplete: false });
  await page.goto("/");
  await expect(page.getByText("Your turn — find the best move!")).toBeVisible();

  const pt = await clickSquare(page, "e2", "white");
  await expectSelectedAt(page, pt);
});

test("clicks hit the correct square after the window resizes", async ({ page }) => {
  await page.addInitScript(mockTauri, { puzzle: BLACK_PUZZLE, onboardingComplete: false });
  await page.goto("/");
  await expect(page.getByText("Your turn — find the best move!")).toBeVisible();

  // Shrink the window: the centered board shifts left without resizing.
  const viewport = page.viewportSize();
  if (!viewport) throw new Error("no viewport");
  await page.setViewportSize({ width: viewport.width - 160, height: viewport.height });
  await page.waitForTimeout(250);

  const pt = await clickSquare(page, "e7", "black");
  await expectSelectedAt(page, pt);
});

test("clicks hit the correct square after sidebar collapses on navigation", async ({ page }) => {
  await page.addInitScript(mockTauri, { puzzle: BLACK_PUZZLE, onboardingComplete: true });
  await page.goto("/");

  // Navigating to Problems collapses the sidebar (200px -> 56px), sliding
  // the centered board left while the puzzle screen mounts.
  const nav = page.getByRole("navigation", { name: "Main navigation" });
  await nav.getByRole("button", { name: /problems/i }).click();
  await expect(page.getByText("Your turn — find the best move!")).toBeVisible();

  // Let the sidebar width transition finish.
  await page.waitForTimeout(500);

  const pt = await clickSquare(page, "e7", "black");
  await expectSelectedAt(page, pt);
});
