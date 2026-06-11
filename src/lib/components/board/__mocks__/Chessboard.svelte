<script lang="ts">
  // Mock Chessboard for testing — renders a simple div instead of chessground.
  // Real-component props not used by the mock are simply ignored by Svelte.
  let { fen, orientation }: { fen?: string; orientation?: string } = $props();
</script>

<div data-testid="chessboard" data-fen={fen} data-orientation={orientation}></div>
