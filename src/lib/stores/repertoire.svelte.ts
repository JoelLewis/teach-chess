import type {
  Opening,
  OpeningPosition,
  RepertoireEntry,
  DrillState,
  DrillMoveResult,
  DrillStats,
  RepertoireFilter,
} from "../types/repertoire";

export type RepertoirePhase =
  | "idle"
  | "browsing"
  | "detail"
  | "editing"
  | "drilling"
  | "drill-result";

class RepertoireStore {
  phase = $state<RepertoirePhase>("idle");
  openings = $state<Opening[]>([]);
  selectedOpening = $state<Opening | null>(null);
  openingPositions = $state<OpeningPosition[]>([]);
  repertoireEntries = $state<RepertoireEntry[]>([]);
  drillState = $state<DrillState | null>(null);
  lastDrillResult = $state<DrillMoveResult | null>(null);
  drillStats = $state<DrillStats | null>(null);
  filter = $state<RepertoireFilter>({});
  activeTab = $state<"library" | "repertoire" | "drill">("library");

  get drillFen(): string | null {
    return this.drillState?.fen ?? null;
  }

  get drillDests(): Record<string, string[]> {
    return this.drillState?.legalDests ?? {};
  }

  get drillColor(): "white" | "black" {
    return this.drillState?.playerColor ?? "white";
  }

  get drillAccuracy(): string {
    if (!this.drillStats || this.drillStats.totalDrills === 0) return "—";
    const pct = Math.round(
      (this.drillStats.totalCorrect / this.drillStats.totalDrills) * 100,
    );
    return `${pct}%`;
  }

  reset() {
    this.phase = "idle";
    this.selectedOpening = null;
    this.openingPositions = [];
    this.repertoireEntries = [];
    this.drillState = null;
    this.lastDrillResult = null;
  }
}

export const repertoireStore = new RepertoireStore();
