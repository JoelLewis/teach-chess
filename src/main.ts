import App from "./App.svelte";
import { mount } from "svelte";
import "./app.css";

const app = mount(App, {
  target: document.getElementById("app")!,
});

// Dev-only playtest hooks (agent driving over the MCP socket). The DEV guard
// is statically eliminated by Vite, so nothing below ships in production.
if (import.meta.env.DEV) {
  import("./lib/dev/playtest")
    .then((p) => p.installPlaytestHooks())
    .catch((err) => console.error("playtest hooks failed to install:", err));
}

export default app;
