// Puzzle flow: open Problems, wait for a puzzle board, take a screenshot.
import { assert, clickByText } from "./_helpers.mjs";

export default async (d) => {
  await clickByText(d, "nav.sidebar button", "Problems");
  await d.waitFor('window.__playtest.getView() === "problems"', { timeoutMs: 5_000 });
  await d.waitFor('!!document.querySelector("cg-board")', { timeoutMs: 20_000 });

  // The puzzle FEN surfaces through the shared getFen() fallback.
  const fen = await d.pt("getFen()");
  assert.ok(fen, "no puzzle FEN");
  console.log(`    puzzle fen: ${fen}`);
  await d.screenshot("50-puzzle");
};
