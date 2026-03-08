import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import GameConfig from "./GameConfig.svelte";

describe("GameConfig", () => {
  it("renders form with start button", () => {
    render(GameConfig, {
      props: {
        onStart: vi.fn(),
      },
    });

    expect(screen.getByText("New Game")).toBeInTheDocument();
    expect(screen.getByText("Start Game")).toBeInTheDocument();
  });

  it("shows loading state when starting", () => {
    render(GameConfig, {
      props: {
        onStart: vi.fn(),
        starting: true,
      },
    });

    expect(screen.getByText("Starting...")).toBeInTheDocument();
    const button = screen.getByRole("button", { name: /starting/i });
    expect(button).toBeDisabled();
  });
});
