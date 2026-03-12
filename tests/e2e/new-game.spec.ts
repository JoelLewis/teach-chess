import { test, expect } from "@playwright/test";

// Smoke test: verify the app loads and renders the main UI.
// Full Tauri IPC is not available in browser-only Playwright tests,
// so this validates the frontend shell loads correctly.

test("app loads and shows the main layout", async ({ page }) => {
  await page.goto("/");
  // Wait for the Svelte app to mount
  await expect(page.locator("#app")).toBeVisible();
});

test("new game button is visible", async ({ page }) => {
  await page.goto("/");
  // Look for a new game action in the UI
  const newGameButton = page.getByRole("button", { name: /new game/i });
  await expect(newGameButton).toBeVisible();
});

test("chessboard renders", async ({ page }) => {
  await page.goto("/");
  // chessground renders a cg-board element
  const board = page.locator("cg-board");
  await expect(board).toBeVisible();
});
