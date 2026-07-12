// Start a new game vs the engine as white; assert phase + start FEN.
import { assert, clickByText, START_BOARD, boardFen } from "./_helpers.mjs";

export default async (d) => {
  // Deterministic config: play as white so the start position is on the board.
  await d.js(
    `(() => {
      localStorage.setItem("chessMentor.gamePrefs", JSON.stringify({ playerColor: "white", strengthPreset: "beginner", coachingLevel: "fullCoach", opponentMode: "choose", personality: "solid", teachingMode: false }));
      return true;
    })()`,
  );

  await clickByText(d, "nav.sidebar button", "Play");
  await d.waitFor('window.__playtest.getView() === "play"', { timeoutMs: 5_000 });
  await d.screenshot("10-game-config");

  await clickByText(d, "button", "Start Game");
  // Engine start can take a few seconds on first launch.
  await d.waitFor('window.__playtest.getPhase() === "playing"', { timeoutMs: 30_000 });

  assert.equal(await boardFen(d), START_BOARD, "expected the start position");
  await d.screenshot("10-game-started");
};
