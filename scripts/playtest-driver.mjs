#!/usr/bin/env node
// Playtest driver for the dev-only MCP socket (tauri-plugin-mcp).
// Zero dependencies; node >= 18. Works as a CLI and as an importable module.
//
// Start the app first: `pnpm playtest` (tauri dev --features mcp), which
// serves the socket at /tmp/chessmentor-mcp.sock.
//
// CLI:
//   node scripts/playtest-driver.mjs ping
//   node scripts/playtest-driver.mjs cmd take_screenshot '{"quality":80}'
//   node scripts/playtest-driver.mjs js '1 + 1'
//   node scripts/playtest-driver.mjs pt 'getFen()'
//   node scripts/playtest-driver.mjs screenshot after-move
//   node scripts/playtest-driver.mjs click 480 360
//   node scripts/playtest-driver.mjs click-square e4
//   node scripts/playtest-driver.mjs move e2 e4
//   node scripts/playtest-driver.mjs wait-for '__playtest.getPhase() === "playing"' 15000
//   node scripts/playtest-driver.mjs run .playtest/steps/new-game.mjs
//
// Module:
//   import { createDriver } from "./scripts/playtest-driver.mjs";
//   const d = await createDriver();
//   await d.move("e2", "e4");
//   await d.close();
//
// Step files (.playtest/steps/*.mjs) export a default async function that
// receives the driver:   export default async (d) => { await d.ping(); };
//
// Env:
//   PLAYTEST_SOCKET   socket path   (default /tmp/chessmentor-mcp.sock)
//   PLAYTEST_WINDOW   window label  (default main)
//   PLAYTEST_OFFSET_X / PLAYTEST_OFFSET_Y
//       calibration offset (logical px) added to native click coordinates,
//       for setups where the webview origin != window origin.

import net from "node:net";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const SOCKET_PATH = process.env.PLAYTEST_SOCKET ?? "/tmp/chessmentor-mcp.sock";
const WINDOW_LABEL = process.env.PLAYTEST_WINDOW ?? "main";
const SHOTS_DIR = ".playtest/shots";
const OFFSET_X = Number(process.env.PLAYTEST_OFFSET_X ?? 0);
const OFFSET_Y = Number(process.env.PLAYTEST_OFFSET_Y ?? 0);

export async function createDriver({ socketPath = SOCKET_PATH, windowLabel = WINDOW_LABEL } = {}) {
  const socket = await connect(socketPath);
  let buffer = "";
  let nextId = 1;
  const pending = new Map(); // id → {resolve, reject}

  socket.on("data", (chunk) => {
    buffer += chunk.toString("utf8");
    let nl;
    while ((nl = buffer.indexOf("\n")) !== -1) {
      const line = buffer.slice(0, nl);
      buffer = buffer.slice(nl + 1);
      if (!line.trim()) continue;
      let msg;
      try {
        msg = JSON.parse(line);
      } catch (err) {
        rejectAll(new Error(`unparseable response: ${line.slice(0, 200)} (${err.message})`));
        continue;
      }
      // Match by echoed id; fall back to FIFO (requests are serialized).
      const key = msg.id ?? pending.keys().next().value;
      const waiter = pending.get(key);
      if (!waiter) continue;
      pending.delete(key);
      waiter.resolve(msg);
    }
  });
  socket.on("error", (err) => rejectAll(err));
  socket.on("close", () => rejectAll(new Error("socket closed")));

  // Dev-launched Tauri windows may start hidden and unfocused. These calls are
  // idempotent and make native clicks and screenshots reliable for every driver.
  for (const operation of ["show", "focus"]) {
    try {
      await cmd("manage_window", { window_label: windowLabel, operation });
    } catch {
      // Window management is optional; native driving can still work without it.
    }
  }

  function rejectAll(err) {
    for (const { reject } of pending.values()) reject(err);
    pending.clear();
  }

  /** Send a raw command; resolves with the full {success,data,error,id} envelope. */
  function cmdRaw(command, payload = {}) {
    const id = String(nextId++);
    const line = JSON.stringify({ command, payload, id }) + "\n";
    return new Promise((resolve, reject) => {
      pending.set(id, { resolve, reject });
      socket.write(line, (err) => {
        if (err) {
          pending.delete(id);
          reject(err);
        }
      });
    });
  }

  /** Send a command; throws on failure, returns response.data. */
  async function cmd(command, payload = {}) {
    const res = await cmdRaw(command, payload);
    if (!res.success) throw new Error(`${command} failed: ${res.error ?? "unknown error"}`);
    return res.data;
  }

  /** Evaluate a JS expression in the webview; returns the decoded value. */
  async function js(code, { timeoutMs = 5000 } = {}) {
    const data = await cmd("execute_js", {
      window_label: windowLabel,
      code,
      timeout_ms: timeoutMs,
    });
    return decodeJsResult(data);
  }

  /** Evaluate an expression against window.__playtest, e.g. pt('getFen()'). */
  function pt(expr) {
    return js(`window.__playtest.${expr}`);
  }

  /** Take a screenshot and save it under .playtest/shots/<name>.jpg. */
  async function screenshot(name) {
    const data = await cmd("take_screenshot", { window_label: windowLabel, quality: 80 });
    const dataUrl = data?.data;
    if (typeof dataUrl !== "string" || !dataUrl.startsWith("data:image/")) {
      throw new Error(`take_screenshot returned no image data: ${JSON.stringify(data).slice(0, 200)}`);
    }
    const base64 = dataUrl.slice(dataUrl.indexOf(",") + 1);
    fs.mkdirSync(SHOTS_DIR, { recursive: true });
    const file = path.join(SHOTS_DIR, `${name}.jpg`);
    fs.writeFileSync(file, Buffer.from(base64, "base64"));
    return file;
  }

  /** Native (trusted) click at window-logical coordinates. */
  async function click(x, y) {
    return cmd("simulate_mouse_movement", {
      window_label: windowLabel,
      x: Math.round(x + OFFSET_X),
      y: Math.round(y + OFFSET_Y),
      click: true,
      button: "left",
    });
  }

  /** Ask the app where a square is, then natively click it. */
  async function clickSquare(square) {
    const center = await pt(`squareCenter(${JSON.stringify(square)})`);
    await click(center.x, center.y);
    return center;
  }

  /** Play a move by clicking the origin then the destination square. */
  async function move(from, to) {
    await clickSquare(from);
    await sleep(150); // let chessground register the selection
    await clickSquare(to);
  }

  /** Poll a JS expression until it is truthy; returns its final value. */
  async function waitFor(jsExpr, { timeoutMs = 10_000, intervalMs = 250 } = {}) {
    const deadline = Date.now() + timeoutMs;
    let last;
    while (Date.now() < deadline) {
      try {
        last = await js(jsExpr);
        if (last) return last;
      } catch (err) {
        // Transient evaluation failures (e.g. mid-reload) — keep polling.
        last = `<error: ${err.message}>`;
      }
      await sleep(intervalMs);
    }
    throw new Error(`waitFor timed out after ${timeoutMs}ms: ${jsExpr} (last value: ${JSON.stringify(last)})`);
  }

  function ping() {
    return cmd("ping", { value: "playtest" });
  }

  function close() {
    socket.end();
    socket.destroy();
  }

  return { cmd, cmdRaw, js, pt, screenshot, click, clickSquare, move, waitFor, ping, sleep, close };
}

function connect(socketPath) {
  return new Promise((resolve, reject) => {
    const socket = net.createConnection(socketPath);
    socket.once("connect", () => resolve(socket));
    socket.once("error", (err) =>
      reject(new Error(`cannot connect to ${socketPath} — is the app running via \`pnpm playtest\`? (${err.message})`)),
    );
  });
}

// execute_js returns {result: string, type: typeof}. Objects arrive
// JSON-stringified; primitives arrive String()-ed.
function decodeJsResult(data) {
  const { result, type } = data ?? {};
  switch (type) {
    case "object":
      try {
        return JSON.parse(result);
      } catch {
        return result; // e.g. "null" edge cases or unstringifiable objects
      }
    case "number":
      return Number(result);
    case "boolean":
      return result === "true";
    case "undefined":
      return undefined;
    default:
      return result;
  }
}

export function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

// ---------------------------------------------------------------- CLI mode
const isMain = process.argv[1] && path.resolve(process.argv[1]) === new URL(import.meta.url).pathname;

if (isMain) {
  const [verb, ...args] = process.argv.slice(2);
  if (!verb || verb === "--help" || verb === "-h") {
    printUsage();
    process.exit(verb ? 0 : 1);
  }
  let driver;
  try {
    driver = await createDriver();
  } catch (err) {
    console.error(`error: ${err.message}`);
    process.exit(1);
  }
  try {
    const out = await runCli(driver, verb, args);
    if (out !== undefined) console.log(typeof out === "string" ? out : JSON.stringify(out, null, 2));
  } catch (err) {
    console.error(`error: ${err.message}`);
    process.exitCode = 1;
  } finally {
    driver.close();
  }
}

async function runCli(d, verb, args) {
  switch (verb) {
    case "ping":
      return d.ping();
    case "cmd":
      return d.cmd(args[0], args[1] ? JSON.parse(args[1]) : {});
    case "js":
      return d.js(args.join(" "));
    case "pt":
      return d.pt(args.join(" "));
    case "screenshot":
      return d.screenshot(args[0] ?? `shot-${Date.now()}`);
    case "click":
      return d.click(Number(args[0]), Number(args[1]));
    case "click-square":
      return d.clickSquare(args[0]);
    case "move":
      return d.move(args[0], args[1]);
    case "wait-for":
      return d.waitFor(args[0], { timeoutMs: args[1] ? Number(args[1]) : undefined });
    case "run": {
      for (const file of args) {
        const mod = await import(path.resolve(file));
        if (typeof mod.default !== "function") {
          throw new Error(`${file} does not export a default async function`);
        }
        console.log(`--- running ${file}`);
        await mod.default(d);
        console.log(`--- ok ${file}`);
      }
      return "all steps passed";
    }
    default:
      printUsage();
      throw new Error(`unknown verb: ${verb}`);
  }
}

function printUsage() {
  console.error(`usage: playtest-driver.mjs <verb> [...args]
  ping
  cmd <command> [payload-json]
  js <expression>
  pt <expression>            (evaluated against window.__playtest)
  screenshot [name]          (saved to ${SHOTS_DIR}/<name>.jpg)
  click <x> <y>              (window-logical coords, native input)
  click-square <square>      (e.g. e4)
  move <from> <to>           (e.g. e2 e4)
  wait-for <expr> [timeoutMs]
  run <step-file.mjs> [...]  (step files under .playtest/steps/)`);
}
