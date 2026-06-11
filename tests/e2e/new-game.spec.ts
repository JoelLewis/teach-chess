import { test, expect } from "@playwright/test";

// Smoke test: verify the app loads and renders the main UI.
// Full Tauri IPC is not available in browser-only Playwright tests,
// so this validates the frontend shell loads correctly.

test("app loads and shows the main layout", async ({ page }) => {
  await page.goto("/");
  // Wait for the Svelte app to mount
  await expect(page.locator("#app")).toBeVisible();
});

test("play navigation is visible", async ({ page }) => {
  await page.goto("/");
  // The sidebar offers the Play entry point for starting a game
  const nav = page.getByRole("navigation", { name: "Main navigation" });
  await expect(nav.getByRole("button", { name: /play/i })).toBeVisible();
});

test("chessboard renders", async ({ page }) => {
  await page.goto("/");
  // chessground renders a cg-board element
  const board = page.locator("cg-board");
  await expect(board).toBeVisible();
});
