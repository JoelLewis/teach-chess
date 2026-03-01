class PlayerStore {
  id = $state<string | null>(null);
  displayName = $state("Player");
  gamesPlayed = $state(0);
}

export const playerStore = new PlayerStore();
