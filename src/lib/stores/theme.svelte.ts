import type { Theme } from "../types/theme";
import * as api from "../api/commands";

type ResolvedTheme = "study" | "grid";

function resolveSystemTheme(): ResolvedTheme {
  if (typeof window === "undefined") return "study";
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "grid" : "study";
}

function createThemeStore() {
  let current = $state<Theme>("study");
  let resolved = $state<ResolvedTheme>("study");
  let mediaQuery: MediaQueryList | null = null;
  let mediaHandler: ((e: MediaQueryListEvent) => void) | null = null;

  function applyResolved(theme: ResolvedTheme) {
    resolved = theme;
    document.documentElement.dataset.theme = theme;
  }

  function apply(theme: Theme) {
    current = theme;

    // Sync to localStorage for flash-prevention script in index.html
    try { localStorage.setItem("cm-theme", theme); } catch {}

    // Clean up previous listener
    if (mediaQuery && mediaHandler) {
      mediaQuery.removeEventListener("change", mediaHandler);
      mediaHandler = null;
    }

    if (theme === "system") {
      applyResolved(resolveSystemTheme());
      // Listen for OS theme changes
      mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      mediaHandler = (e: MediaQueryListEvent) => {
        applyResolved(e.matches ? "grid" : "study");
      };
      mediaQuery.addEventListener("change", mediaHandler);
    } else {
      applyResolved(theme);
    }
  }

  return {
    get current() {
      return current;
    },
    get resolved() {
      return resolved;
    },

    async load() {
      try {
        const theme = await api.getTheme();
        apply(theme);
      } catch {
        apply("study");
      }
    },

    async set(theme: Theme) {
      apply(theme);
      try {
        await api.setTheme(theme);
      } catch (err) {
        console.error("Failed to persist theme:", err);
      }
    },
  };
}

export const themeStore = createThemeStore();
