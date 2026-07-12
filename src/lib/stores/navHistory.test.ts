import { describe, it, expect } from "vitest";
import { createNavHistory, type NavEntry, type Page } from "./navHistory.svelte";

function entry(page: Page, reviewGameId = ""): NavEntry {
  return { page, reviewGameId };
}

describe("createNavHistory", () => {
  it("starts with empty back and forward stacks", () => {
    const nav = createNavHistory();
    expect(nav.canGoBack).toBe(false);
    expect(nav.canGoForward).toBe(false);
    expect(nav.goBack(entry("home"))).toBeNull();
    expect(nav.goForward(entry("home"))).toBeNull();
  });

  it("push records the page being left", () => {
    const nav = createNavHistory();
    nav.push(entry("home"));
    expect(nav.canGoBack).toBe(true);
    expect(nav.canGoForward).toBe(false);
  });

  it("goBack returns the last pushed entry and moves current to forward", () => {
    const nav = createNavHistory();
    nav.push(entry("home"));
    const back = nav.goBack(entry("history"));
    expect(back).toEqual(entry("home"));
    expect(nav.canGoBack).toBe(false);
    expect(nav.canGoForward).toBe(true);
  });

  it("goForward reverses goBack", () => {
    const nav = createNavHistory();
    nav.push(entry("home"));
    nav.goBack(entry("review", "game-7"));
    const fwd = nav.goForward(entry("home"));
    expect(fwd).toEqual(entry("review", "game-7"));
    expect(nav.canGoBack).toBe(true);
    expect(nav.canGoForward).toBe(false);
  });

  it("walks back and forward through multiple entries in order", () => {
    const nav = createNavHistory();
    nav.push(entry("home"));
    nav.push(entry("history"));
    nav.push(entry("review", "game-3"));
    // current page is "settings"
    expect(nav.goBack(entry("settings"))).toEqual(entry("review", "game-3"));
    expect(nav.goBack(entry("review", "game-3"))).toEqual(entry("history"));
    expect(nav.goForward(entry("history"))).toEqual(entry("review", "game-3"));
    expect(nav.goBack(entry("review", "game-3"))).toEqual(entry("history"));
    expect(nav.goBack(entry("history"))).toEqual(entry("home"));
    expect(nav.goBack(entry("home"))).toBeNull();
  });

  it("push clears the forward stack", () => {
    const nav = createNavHistory();
    nav.push(entry("home"));
    nav.goBack(entry("openings"));
    expect(nav.canGoForward).toBe(true);
    nav.push(entry("home"));
    expect(nav.canGoForward).toBe(false);
  });

  it("caps the back stack, dropping the oldest entries", () => {
    const nav = createNavHistory(3);
    nav.push(entry("home"));
    nav.push(entry("history"));
    nav.push(entry("problems"));
    nav.push(entry("review", "game-1"));
    expect(nav.goBack(entry("play"))).toEqual(entry("review", "game-1"));
    expect(nav.goBack(entry("review", "game-1"))).toEqual(entry("problems"));
    expect(nav.goBack(entry("problems"))).toEqual(entry("history"));
    // "home" was dropped by the cap
    expect(nav.goBack(entry("history"))).toBeNull();
  });

  it("caps at 50 by default", () => {
    const nav = createNavHistory();
    for (let i = 0; i < 60; i++) {
      nav.push(entry("review", `game-${i}`));
    }
    let steps = 0;
    while (nav.goBack(entry("home")) !== null) {
      steps++;
    }
    expect(steps).toBe(50);
  });

  it("clear empties both stacks", () => {
    const nav = createNavHistory();
    nav.push(entry("home"));
    nav.push(entry("history"));
    nav.goBack(entry("settings"));
    nav.clear();
    expect(nav.canGoBack).toBe(false);
    expect(nav.canGoForward).toBe(false);
  });
});
