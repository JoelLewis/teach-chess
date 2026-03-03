import type { Theme } from "../types/theme";
import * as api from "../api/commands";

function createThemeStore() {
  let current = $state<Theme>("study");

  function apply(theme: Theme) {
    current = theme;
    document.documentElement.dataset.theme = theme;
  }

  return {
    get current() {
      return current;
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
