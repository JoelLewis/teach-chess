import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Score, InGameCoachingFeedback } from "../types/engine";

export type EngineInfoPayload = {
  depth: number;
  score: Score;
  nodes: number;
  pv: string[];
};

export type ReviewProgressPayload = {
  current: number;
  total: number;
};

export function onEngineThinking(
  callback: (thinking: boolean) => void,
): Promise<UnlistenFn> {
  return listen<boolean>("engine-thinking", (event) => {
    callback(event.payload);
  });
}

export function onEngineInfo(
  callback: (info: EngineInfoPayload) => void,
): Promise<UnlistenFn> {
  return listen<EngineInfoPayload>("engine-info", (event) => {
    callback(event.payload);
  });
}

export function onEngineReady(
  callback: () => void,
): Promise<UnlistenFn> {
  return listen("engine-ready", () => {
    callback();
  });
}

export function onReviewProgress(
  callback: (progress: ReviewProgressPayload) => void,
): Promise<UnlistenFn> {
  return listen<ReviewProgressPayload>("review-progress", (event) => {
    callback(event.payload);
  });
}

// ─── In-Game Coaching Events ─────────────────────────────────────

export function onInGameCoaching(
  callback: (feedback: InGameCoachingFeedback) => void,
): Promise<UnlistenFn> {
  return listen<InGameCoachingFeedback>("in-game-coaching", (event) => {
    callback(event.payload);
  });
}

// ─── LLM Events ─────────────────────────────────────────────────

export type LlmDownloadProgressPayload = {
  downloadedBytes: number;
  totalBytes: number;
};

export function onLlmDownloadProgress(
  callback: (progress: LlmDownloadProgressPayload) => void,
): Promise<UnlistenFn> {
  return listen<LlmDownloadProgressPayload>(
    "llm-download-progress",
    (event) => {
      callback(event.payload);
    },
  );
}
