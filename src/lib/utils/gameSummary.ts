import type { Color, GameOutcome } from "../api/bindings";
import { humanizeLabel } from "./format";

export type GameResult = {
  card: "win" | "loss" | "draw";
  text: string;
  detail: string;
  opponent?: string;
};

function decisive(outcome: GameOutcome): { kind: "checkmate" | "resignation"; winner: Color } | null {
  if (typeof outcome === "string") return null;
  if (outcome.checkmate) return { kind: "checkmate", winner: outcome.checkmate.winner };
  if (outcome.resignation) return { kind: "resignation", winner: outcome.resignation.winner };
  return null;
}

export function opponentName(stored: string | null | undefined): string | null {
  if (!stored) return null;
  try {
    const parsed: unknown = JSON.parse(stored);
    return typeof parsed === "string" ? humanizeLabel(parsed) : null;
  } catch {
    return humanizeLabel(stored);
  }
}

export function gameOverOutcome(
  boardOutcome: GameOutcome | null,
  recordResult: string | null | undefined,
  playerColor: Color,
): GameOutcome | null {
  if (boardOutcome) return boardOutcome;
  if (recordResult == null) return null;
  const winner = recordResult === "1-0"
    ? "white"
    : recordResult === "0-1"
      ? "black"
      : playerColor === "white" ? "black" : "white";
  return { resignation: { winner } };
}

export function gameResult(
  outcome: GameOutcome | null,
  playerColor: Color,
  storedOpponent?: string | null,
): GameResult {
  const opponent = opponentName(storedOpponent) ?? undefined;
  if (!outcome) return { card: "draw", text: "Game Over", detail: "" };
  const d = decisive(outcome);
  if (d) {
    const won = d.winner === playerColor;
    const reason = `by ${d.kind}`;
    return {
      card: won ? "win" : "loss",
      text: d.kind === "checkmate"
        ? won ? "You win by checkmate!" : "You lost by checkmate"
        : won ? "Opponent resigned — you win!" : "You resigned",
      detail: reason,
      opponent,
    };
  }
  const detail = outcome === "stalemate" ? "by stalemate"
    : outcome === "insufficientmaterial" ? "by insufficient material"
    : outcome === "threefoldrepetition" ? "by threefold repetition"
    : outcome === "fiftymoverule" ? "by fifty-move rule" : "by agreement";
  return {
    card: "draw",
    text: detail === "by agreement" ? "Draw" : `Draw ${detail}`,
    detail,
    opponent,
  };
}
