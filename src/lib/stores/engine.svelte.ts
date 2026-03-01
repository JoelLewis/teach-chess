class EngineStore {
  isRunning = $state(false);
  isThinking = $state(false);
  currentDepth = $state(0);
  currentNodes = $state(0);
}

export const engineStore = new EngineStore();
