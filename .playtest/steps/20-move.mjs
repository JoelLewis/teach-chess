// Play 1. e4 with native clicks; assert the FEN changes and the engine replies.
import { assert, START_BOARD, boardFen } from "./_helpers.mjs";

export default async (d) => {
  assert.equal(await d.pt("getPhase()"), "playing", "run 10-new-game first");
  const before = await boardFen(d);

  await d.move("e2", "e4");
  await d.waitFor(
    `window.__playtest.getFen()?.split(" ")[0] !== ${JSON.stringify(before)}`,
    { timeoutMs: 10_000 },
  );

  const afterMove = await d.pt("getFen()");
  assert.match(afterMove.split(" ")[0], /4P3/, "no pawn on e4 — click mapping off?");

  // Engine reply: it becomes white's turn again with a changed board.
  await d.waitFor(
    `(() => {
      const f = window.__playtest.getFen();
      return !!f && f.split(" ")[1] === "w" && f.split(" ")[0] !== ${JSON.stringify(START_BOARD)};
    })()`,
    { timeoutMs: 30_000 },
  );
  await d.screenshot("20-after-e4-and-reply");
};
