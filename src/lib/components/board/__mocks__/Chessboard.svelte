<script lang="ts">
  // Mock Chessboard for testing — renders a simple div instead of chessground
  let { fen, orientation, turnColor, dests, viewOnly, lastMove, isCheck, onMove }: {
    fen?: string;
    orientation?: string;
    turnColor?: string;
    dests?: Record<string, string[]>;
    viewOnly?: boolean;
    lastMove?: [string, string] | null;
    isCheck?: boolean;
    onMove?: (from: string, to: string) => void;
  } = $props();
</script>

<div data-testid="chessboard" data-fen={fen} data-orientation={orientation}></div>
