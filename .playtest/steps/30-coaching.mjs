// Coaching panel populates after a player move (LLM or template fallback).
import { assert } from "./_helpers.mjs";

export default async (d) => {
  assert.equal(await d.pt("getPhase()"), "playing", "run 10-new-game and 20-move first");

  // Coaching for the last move may still be streaming; allow model warm-up.
  await d.waitFor("!!window.__playtest.getCoachingText()", { timeoutMs: 60_000 });

  const text = await d.pt("getCoachingText()");
  assert.ok(text.length > 0, "coaching text empty");
  console.log(`    coaching: ${text.slice(0, 120)}…`);
  await d.screenshot("30-coaching");
};
