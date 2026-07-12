<script lang="ts">
  import { Chessground } from "@lichess-org/chessground";
  import { untrack } from "svelte";
  import type { Api } from "@lichess-org/chessground/api";
  import type { Key } from "@lichess-org/chessground/types";
  import type { Color } from "../../api/bindings";

  type Props = {
    fen?: string;
    orientation?: Color;
    turnColor?: Color;
    dests?: Record<string, string[]>;
    viewOnly?: boolean;
    lastMove?: [string, string] | null;
    isCheck?: boolean;
    onMove?: (from: string, to: string) => void;
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
  }: Props = $props();

  let boardEl: HTMLDivElement;
  let containerEl: HTMLDivElement;
  let cg: Api | undefined = $state();

  function destsToMap(d: Record<string, string[]>): Map<Key, Key[]> {
    const map = new Map<Key, Key[]>();
    for (const [from, tos] of Object.entries(d)) {
      map.set(from as Key, tos as Key[]);
    }
    return map;
  }

  // Mount chessground — recreated only when viewOnly changes, since chessground
  // binds pointer and bounds-invalidation listeners at construction and skips
  // them all under viewOnly (set() never rebinds). Other prop changes sync via
  // the effect below without recreating.
  $effect(() => {
    if (!boardEl) return;
    const view = viewOnly;

    const instance = untrack(() =>
      Chessground(boardEl, {
        fen,
        orientation,
        turnColor,
        viewOnly: view,
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
      }),
    );
    cg = instance;

    // Chessground caches the board's screen position and only refreshes it
    // when the board element itself resizes — a board that *moves* (sidebar
    // collapse, window resize, panel changes) hit-tests against stale
    // coordinates. Clear the cache right before chessground handles the
    // event: capture phase on the container runs first.
    const refreshBounds = () => instance.state.dom.bounds.clear();
    containerEl.addEventListener("mousedown", refreshBounds, { capture: true });
    containerEl.addEventListener("touchstart", refreshBounds, { capture: true, passive: true });

    return () => {
      containerEl.removeEventListener("mousedown", refreshBounds, { capture: true });
      containerEl.removeEventListener("touchstart", refreshBounds, { capture: true });
      instance.destroy();
      cg = undefined;
    };
  });

  // Sync props to chessground when they change (without recreating)
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

<div bind:this={containerEl} class="board-container" role="img" aria-label="Chess board — position: {fen ?? 'starting position'}">
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
