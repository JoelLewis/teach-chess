// Resign → confirm → game-over dialog → review screen with a view-only board.
import { assert, clickByText } from "./_helpers.mjs";

export default async (d) => {
  assert.equal(await d.pt("getPhase()"), "playing", "run 10-new-game first");

  await clickByText(d, "button.btn-resign", "Resign");
  await d.waitFor('!!document.querySelector("button.btn-resign-yes")', { timeoutMs: 5_000 });
  await clickByText(d, "button.btn-resign-yes", "Yes");

  await d.waitFor('window.__playtest.getPhase() === "game-over"', { timeoutMs: 10_000 });
  await d.waitFor('!!document.querySelector("button.btn-review")', { timeoutMs: 10_000 });
  await d.screenshot("40-game-over-dialog");

  await clickByText(d, "button.btn-review", "Review Game");
  await d.waitFor('window.__playtest.getView() === "review"', { timeoutMs: 10_000 });
  await d.waitFor('!!document.querySelector("cg-board")', { timeoutMs: 15_000 });
  await d.sleep(500); // let the screen-fade transition finish before capturing

  // chessground marks interactive boards `manipulable`; viewOnly boards lack it.
  const viewOnly = await d.js(
    '(() => { const w = document.querySelector(".cg-wrap"); return !!w && !w.classList.contains("manipulable"); })()',
  );
  assert.ok(viewOnly, "review board should be view-only");
  await d.screenshot("40-review-screen");
};
