# Plan: Dev-only MCP Playtest Capability

Status: planned, not yet implemented. Estimate ~1.5–2 days across 7 independently verifiable steps.
Mirrors GoSensei's `tauri-plugin-mcp` socket, plus fixes the half of it GoSensei never wired up.

## Root cause: why execute_js / DOM tools time out in GoSensei

The plugin's JS tooling has two halves and GoSensei installed only one:

1. Rust side (`tools/execute_js.rs` + `emit_and_wait`): `execute_js` does not `webview.eval()`;
   it emits a Tauri event `execute-js` with a `_correlationId` and waits on
   `execute-js-response-{id}`. No listener in the webview → "Timeout waiting for JS execution".
2. The injected `listener_patch.js` is only an addEventListener monkey-patch feeding
   `get_page_map` interactivity detection — it answers nothing.
3. The real handlers live in the guest npm package **`tauri-plugin-mcp`** (v0.1.0):
   `setupPluginListeners()` registers listeners for `execute-js`, `get-page-map`,
   `get-element-position`, etc. The app frontend must import and call it — GoSensei never did
   (it only has `tauri-plugin-mcp-server`, the unrelated stdio↔socket bridge CLI).

Fix per app: `pnpm add -D tauri-plugin-mcp` + call `setupPluginListeners()` in a dev-only path.
Risk check at implementation: confirm npm 0.1.0 carries the correlation-ID protocol
(`npm pack tauri-plugin-mcp && tar xOf … | grep _correlationId`); if stale, vendor a ~60-line
minimal `execute-js` guest handler instead. Capabilities: guest responds via `event.emit`,
covered by `core:default` — no capability changes.

## Security finding (already fixed in teach-go PR #12)

teach-go had `default = ["mcp", "llm"]` and `release.yml` (tauri-action) builds default features —
published releases shipped the unauthenticated socket server. Chess must keep `mcp` OUT of
default features so every existing build path (App Store workflow included) excludes it by
default.

## Step 1 — Cargo wiring (~0.5h)

`src-tauri/Cargo.toml`:

```toml
[dependencies]
# dev-only agent-driving socket; never in default features (App Store!)
tauri-plugin-mcp = { git = "https://github.com/P3GLEG/tauri-plugin-mcp", rev = "49fbecb2dcfac7f5739a79ae22bcf9452dd8d228", optional = true }

[features]
default = ["llm"]          # unchanged — mcp is opt-in
mcp = ["dep:tauri-plugin-mcp"]
```

`src-tauri/src/lib.rs` (mirror teach-go's builder block):

```rust
#[allow(unused_mut)]
let mut builder = tauri::Builder::default().plugin(tauri_plugin_shell::init());
#[cfg(feature = "mcp")]
{
    builder = builder.plugin(tauri_plugin_mcp::init_with_config(
        tauri_plugin_mcp::PluginConfig::new("ChessMentor".to_string())
            .start_socket_server(true)
            .socket_path("/tmp/chessmentor-mcp.sock".into()),
    ));
}
```

`package.json`: `"playtest": "tauri dev --features mcp"`.

Release-exclusion verification: `cargo tree -i tauri-plugin-mcp` (absent by default, present with
`--features mcp`); after `tauri build`, `strings <binary> | grep -iE 'tauri-mcp|chessmentor-mcp|TAURI_MCP'`
→ empty (positive control: the mcp build hits). Optional CI guard in appstore.yml.

## Step 2 — Guest bindings (execute_js fix) (~1h, +1h if npm stale)

`pnpm add -D tauri-plugin-mcp`; verify tarball protocol; dev-only dynamic import (below) calls
`setupPluginListeners()`. Verify `execute_js "1+1"` round-trips over the socket.

## Step 3 — `window.__playtest` hook (~1.5h)

New `src/lib/dev/playtest.ts`, loaded ONLY via:

```ts
// src/main.ts
if (import.meta.env.DEV) {
  import("./lib/dev/playtest").then((p) => p.installPlaytestHooks());
}
```

Exposes `{ getView, getPhase, getFen, getCoachingText, squareCenter("e4"), clickSquare("e4") }`.
`squareCenter` mirrors chessground's computeSquareCenter over `cg-board.getBoundingClientRect()`
with orientation handling; `clickSquare` dispatches synthetic mousedown/mouseup/click (chessground
binds plain mousedown, no isTrusted check) — but drivers should prefer `squareCenter` + the
plugin's native `simulate_mouse_movement` (trusted input). App.svelte registers a view getter via
a small dev-gated `$effect`. JS-exclusion proof: `pnpm run build && grep -r "__playtest" dist/` → empty
(Vite statically eliminates the DEV branch).

## Step 4 — Checked-in driver (~2–3h)

`scripts/playtest-driver.mjs` (zero-dep node ≥18; per-repo copy, NOT in sensei-kit — it's a pure
Rust workspace; revisit if a third app appears). Transport: newline-JSON over
`PLAYTEST_SOCKET ?? /tmp/chessmentor-mcp.sock`. API: `cmd`, `js`, `pt(expr)`,
`screenshot(name)` (saves data-URL JPEG to `.playtest/shots/`), `click(x,y)`,
`clickSquare("e4")` (squareCenter → native click), `move("e2","e4")`,
`waitFor(jsExpr, {timeoutMs})`. CLI mode + step files under `.playtest/steps/`
(gitignored except steps/). Document in docs/PLAYTESTING.md.

## Step 5 — Smoke-test step files (~2h)

Onboarding reset+skip → home; new game vs engine → phase/FEN asserts; `move("e2","e4")` → FEN flip
+ engine reply; coaching panel populates (waitFor ≤20s) incl. deliberate hung piece →
classification badge; puzzle flow (wrong then right moves, streak counter); review screen
(resign → dialog → move nav, viewOnly board); theme persistence; no error toast at end.

## Step 6 — Exclusion verification + optional CI guard (~0.5h)

## Step 7 — Back-port to teach-go (~1h)

- Same guest-bindings fix (`setupPluginListeners()` dev-gated) → repairs execute_js et al.
- `default` features already fixed (PR #12); pin plugin rev (done in PR #12); add `playtest` script (done).
- Copy driver with `PLAYTEST_SOCKET=/tmp/gosensei-mcp.sock`; go-specific `__playtest`
  (`clickPoint("q16")` over goban geometry).

## Session-proven protocol notes (for the driver)

- Envelope: `{"command","payload","id"}\n` → `{"success","data","error","id"}\n` (NOT MCP JSON-RPC)
- Working today: `ping`, `list_windows`, `take_screenshot` (data:image/jpeg;base64),
  `simulate_mouse_movement {window_label,x,y,click}` in window-logical coords
  (screenshot px ≈ logical × 0.8525 at scaleFactor 2 — calibrate once per run)
