import { describe, expect, it } from "vitest";
import { gameOverOutcome, gameResult } from "./gameSummary";

describe("gameResult", () => {
  it("maps a checkmate for the player to a win", () => {
    expect(gameResult({ checkmate: { winner: "white" } }, "white")).toEqual({
      card: "win",
      text: "You win by checkmate!",
      detail: "by checkmate",
    });
  });

  it("maps resignation and draw outcomes", () => {
    expect(gameResult({ resignation: { winner: "black" } }, "white").text).toBe("You resigned");
    expect(gameResult("stalemate", "white").text).toBe("Draw by stalemate");
  });

  it("creates a resignation outcome when the board has no outcome", () => {
    expect(gameOverOutcome(null, "0-1", "white")).toEqual({
      resignation: { winner: "black" },
    });
    expect(gameOverOutcome("stalemate", "1/2-1/2", "white")).toBe("stalemate");
  });

  it("shows that a black player resigned from a 1-0 record", () => {
    const outcome = gameOverOutcome(null, "1-0", "black");
    expect(gameResult(outcome, "black").text).toBe("You resigned");
  });

  it("humanizes an opponent personality stored as JSON", () => {
    expect(gameResult("draw", "white", '"solid"').opponent).toBe("Solid");
  });
});
