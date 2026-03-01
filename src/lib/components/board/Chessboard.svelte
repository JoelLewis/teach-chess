<script lang="ts">
  import { Chessground } from "@lichess-org/chessground";
  import type { Api } from "@lichess-org/chessground/api";
  import type { Key } from "@lichess-org/chessground/types";
  import type { Color } from "../../types/chess";

  type Props = {
    fen?: string;
    orientation?: Color;
    turnColor?: Color;
    dests?: Record<string, string[]>;
    viewOnly?: boolean;
    lastMove?: [string, string] | null;
    isCheck?: boolean;
    onMove?: (from: string, to: string) => void;
    onPromotion?: (from: string, to: string) => Promise<string>;
  };

  let {
    fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    orientation = "white",
    turnColor = "white",
    dests = {},
    viewOnly = false,
    lastMove = null,
    isCheck = false,
    onMove,
    onPromotion,
  }: Props = $props();

  let boardEl: HTMLDivElement;
  let cg: Api | undefined = $state();

  // Convert Record<string, string[]> to Map<Key, Key[]> for chessground
  function destsToMap(d: Record<string, string[]>): Map<Key, Key[]> {
    const map = new Map<Key, Key[]>();
    for (const [from, tos] of Object.entries(d)) {
      map.set(from as Key, tos as Key[]);
    }
    return map;
  }

  // Mount chessground when element is ready
  $effect(() => {
    if (!boardEl) return;

    cg = Chessground(boardEl, {
      fen,
      orientation,
      turnColor,
      viewOnly,
      movable: {
        free: false,
        color: turnColor,
        dests: destsToMap(dests),
        showDests: true,
      },
      highlight: {
        lastMove: true,
        check: true,
      },
      animation: {
        enabled: true,
        duration: 200,
      },
      draggable: {
        enabled: true,
        showGhost: true,
      },
      events: {
        move: (orig: Key, dest: Key) => {
          onMove?.(orig, dest);
        },
      },
    });

    return () => {
      cg?.destroy();
    };
  });

  // Sync props to chessground when they change
  $effect(() => {
    if (!cg) return;

    cg.set({
      fen,
      orientation,
      turnColor,
      viewOnly,
      lastMove: lastMove ? (lastMove as [Key, Key]) : undefined,
      check: isCheck,
      movable: {
        color: viewOnly ? undefined : turnColor,
        dests: destsToMap(dests),
      },
    });
  });
</script>

<div class="board-container">
  <div bind:this={boardEl} class="board"></div>
</div>

<style>
  .board-container {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
  }

  .board {
    width: min(80vh, 560px);
    height: min(80vh, 560px);
  }
</style>
