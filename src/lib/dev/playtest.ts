// Dev-only playtest hooks for agent-driven testing over the tauri-plugin-mcp
// socket. Loaded exclusively via a dynamic import behind `import.meta.env.DEV`
// (src/main.ts), so Vite statically eliminates this module — and everything it
// pulls in — from production bundles. Verify: `pnpm run build && grep -r
// "__playtest" dist/` must be empty.

import { gameStore } from "../stores/game.svelte";
import { puzzleStore } from "../stores/puzzle.svelte";

type SquareCenter = { x: number; y: number };

type PlaytestApi = {
  getView: () => string;
  getPhase: () => string;
  getFen: () => string | null;
  getCoachingText: () => string | null;
  squareCenter: (square: string) => SquareCenter;
  clickSquare: (square: string) => SquareCenter;
};

declare global {
  interface Window {
    __playtest?: PlaytestApi;
  }
}

// App.svelte registers a closure over its `page` state so drivers can ask
// which screen is showing without reaching into component internals.
let viewGetter: () => string = () => "unknown";

export function registerViewGetter(fn: () => string): void {
  viewGetter = fn;
}

function boardElement(): { board: Element; blackOrientation: boolean } {
  const board = document.querySelector("cg-board");
  if (!board) throw new Error("playtest: no cg-board in the DOM");
  const wrap = board.closest(".cg-wrap");
  return {
    board,
    blackOrientation: wrap?.classList.contains("orientation-black") ?? false,
  };
}

// Mirrors chessground's computeSquareCenter over the live board rect, with
// orientation handling. Returns viewport CSS-pixel coordinates — the same
// space the plugin's simulate_mouse_movement expects (window-logical coords).
function squareCenter(square: string): SquareCenter {
  if (!/^[a-h][1-8]$/.test(square)) {
    throw new Error(`playtest: invalid square "${square}"`);
  }
  const { board, blackOrientation } = boardElement();
  const rect = board.getBoundingClientRect();
  const file = square.charCodeAt(0) - 97; // a → 0
  const rank = square.charCodeAt(1) - 49; // 1 → 0
  const col = blackOrientation ? 7 - file : file;
  const row = blackOrientation ? rank : 7 - rank;
  return {
    x: rect.left + ((col + 0.5) * rect.width) / 8,
    y: rect.top + ((row + 0.5) * rect.height) / 8,
  };
}

// Synthetic click fallback. Chessground binds plain mousedown (no isTrusted
// check), so two clickSquare calls perform a move. Drivers should prefer
// squareCenter + the plugin's native simulate_mouse_movement (trusted input);
// this exists for environments where native injection is unavailable.
function clickSquare(square: string): SquareCenter {
  const center = squareCenter(square);
  const { board } = boardElement();
  const common = {
    bubbles: true,
    cancelable: true,
    clientX: center.x,
    clientY: center.y,
    button: 0,
  };
  board.dispatchEvent(new MouseEvent("mousedown", { ...common, buttons: 1 }));
  board.dispatchEvent(new MouseEvent("mouseup", { ...common, buttons: 0 }));
  board.dispatchEvent(new MouseEvent("click", { ...common, buttons: 0 }));
  return center;
}

function getFen(): string | null {
  // On the problems screen the puzzle board is what's visible; elsewhere the
  // game position (when present) wins.
  if (viewGetter() === "problems") {
    return puzzleStore.currentFen ?? gameStore.position?.fen ?? null;
  }
  return gameStore.position?.fen ?? puzzleStore.currentFen ?? null;
}

function getCoachingText(): string | null {
  return gameStore.latestCoaching?.coachingText ?? null;
}

export async function installPlaytestHooks(): Promise<void> {
  window.__playtest = {
    getView: () => viewGetter(),
    getPhase: () => gameStore.phase,
    getFen,
    getCoachingText,
    squareCenter,
    clickSquare,
  };

  // The plugin's Rust half emits `execute-js` (etc.) events with a
  // correlation ID and waits for the guest to answer. setupPluginListeners
  // registers those guest handlers — without it every execute_js/DOM tool
  // times out. Only meaningful inside a Tauri webview; plain-browser
  // `pnpm dev` has no Tauri IPC, so skip it there.
  if ("__TAURI_INTERNALS__" in window) {
    const { setupPluginListeners } = await import("tauri-plugin-mcp");
    await setupPluginListeners();
    console.info("playtest: MCP guest listeners installed");
  }
  console.info("playtest: window.__playtest ready");
}
