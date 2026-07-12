// Shared helpers for playtest step files. Not a step itself — the driver's
// `run` verb only executes files you name explicitly.
import assert from "node:assert/strict";

export { assert };

export const START_BOARD = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

/** Click the first element matching `selector` whose text includes `text`. */
export async function clickByText(d, selector, text) {
  const clicked = await d.js(
    `(() => {
      const els = [...document.querySelectorAll(${JSON.stringify(selector)})];
      const el = els.find((e) => e.textContent.includes(${JSON.stringify(text)}));
      if (!el) return false;
      el.click();
      return true;
    })()`,
  );
  assert.equal(clicked, true, `no ${selector} containing "${text}"`);
}

/** Assert the app shows no error toast. */
export async function assertNoErrorToast(d) {
  const toast = await d.js(`document.querySelector('.toast[role="alert"]')?.textContent ?? null`);
  assert.equal(toast, null, `error toast visible: ${toast}`);
}

/** Board half of the current FEN. */
export async function boardFen(d) {
  const fen = await d.pt("getFen()");
  return fen ? fen.split(" ")[0] : null;
}
