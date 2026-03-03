<script lang="ts">
  import type { SkillRating } from "../../types/assessment";

  type Props = {
    ratings: SkillRating[];
    size?: number;
  };

  let { ratings, size = 200 }: Props = $props();

  const CATEGORY_COLORS: Record<string, string> = {
    tactical: "var(--cm-skill-tactical)",
    positional: "var(--cm-skill-positional)",
    endgame: "var(--cm-skill-endgame)",
    opening: "var(--cm-skill-opening)",
    pattern: "var(--cm-skill-pattern)",
  };

  const CATEGORIES = ["tactical", "positional", "endgame", "opening", "pattern"];
  const MAX_RATING = 2000;
  const MIN_RATING = 400;

  let cx = $derived(size / 2);
  let cy = $derived(size / 2);
  let radius = $derived(size / 2 - 30);

  function angleFor(i: number, total: number): number {
    return (Math.PI * 2 * i) / total - Math.PI / 2;
  }

  function pointAt(i: number, total: number, r: number): { x: number; y: number } {
    const angle = angleFor(i, total);
    return {
      x: cx + r * Math.cos(angle),
      y: cy + r * Math.sin(angle),
    };
  }

  let ratingMap = $derived.by(() => {
    const map = new Map<string, number>();
    for (const r of ratings) {
      map.set(r.category, r.rating);
    }
    return map;
  });

  let polygonPoints = $derived.by(() => {
    return CATEGORIES.map((cat, i) => {
      const rating = ratingMap.get(cat) ?? 1200;
      const normalized = Math.max(0, Math.min(1, (rating - MIN_RATING) / (MAX_RATING - MIN_RATING)));
      const r = normalized * radius;
      const p = pointAt(i, CATEGORIES.length, r);
      return `${p.x},${p.y}`;
    }).join(" ");
  });

  let gridLevels = [0.25, 0.5, 0.75, 1.0];
</script>

<svg width={size} height={size} viewBox="0 0 {size} {size}" class="radar-chart">
  <!-- Grid rings -->
  {#each gridLevels as level}
    <polygon
      points={CATEGORIES.map((_, i) => {
        const p = pointAt(i, CATEGORIES.length, radius * level);
        return `${p.x},${p.y}`;
      }).join(" ")}
      fill="none"
      stroke="var(--cm-border-light)"
      stroke-width="1"
    />
  {/each}

  <!-- Axis lines -->
  {#each CATEGORIES as _, i}
    {@const p = pointAt(i, CATEGORIES.length, radius)}
    <line x1={cx} y1={cy} x2={p.x} y2={p.y} stroke="var(--cm-border-light)" stroke-width="1" />
  {/each}

  <!-- Data polygon -->
  <polygon
    points={polygonPoints}
    fill="var(--cm-skill-radar-fill)"
    stroke="var(--cm-skill-radar-stroke)"
    stroke-width="2"
  />

  <!-- Data points and labels -->
  {#each CATEGORIES as cat, i}
    {@const rating = ratingMap.get(cat) ?? 1200}
    {@const normalized = Math.max(0, Math.min(1, (rating - MIN_RATING) / (MAX_RATING - MIN_RATING)))}
    {@const dp = pointAt(i, CATEGORIES.length, normalized * radius)}
    {@const lp = pointAt(i, CATEGORIES.length, radius + 18)}
    <circle
      cx={dp.x}
      cy={dp.y}
      r="4"
      fill={CATEGORY_COLORS[cat] ?? "var(--cm-text-muted)"}
    />
    <text
      x={lp.x}
      y={lp.y}
      text-anchor="middle"
      dominant-baseline="central"
      class="label"
      fill={CATEGORY_COLORS[cat] ?? "var(--cm-text-muted)"}
    >
      {cat.charAt(0).toUpperCase() + cat.slice(1)}
    </text>
  {/each}
</svg>

<style>
  .radar-chart {
    display: block;
    margin: 0 auto;
  }

  .label {
    font-size: 10px;
    font-weight: 600;
  }
</style>
