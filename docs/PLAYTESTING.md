# ChessMentor Playtest Checklist

> Distributing a build to testers? See [RELEASE.md](RELEASE.md) first — local
> DMGs are ad-hoc signed and need a workaround (or notarization) on other Macs.

A structured pass over every user-facing flow. Run either in dev (`npm run tauri dev`)
or against the installed app from `npm run build:mac`. Items marked **[installed]**
only apply to the built .app/DMG.

Expected timings on an 8 GB Apple Silicon machine: model warm-up ~15s after launch
(background — UI is usable immediately), coaching responses ~3–6s once warm.

## 1. First launch & onboarding
- [ ] Fresh start (optional): quit the app and delete the app-data folder
      (`~/Library/Application Support/com.chessmentor.app`) to simulate a new user.
- [ ] App launches to the dashboard; no error toasts.
- [ ] First-run onboarding offers a starter puzzle; completing it celebrates and lands you back cleanly.
- [ ] Repeat fresh start and use **Skip** instead — onboarding never reappears.

## 2. Dashboard
- [ ] Skill radar, streak banner, and recommendation cards render (placeholders OK for a new profile).
- [ ] AI Coach card reflects model state: "ready" when bundled/downloaded, download prompt otherwise.
- [ ] Stat values pulse when they change (e.g., after a puzzle).

## 3. Puzzles
- [ ] Load a puzzle; hint system reveals progressively; wrong moves give feedback without breaking state.
- [ ] Solve several in a row — streak counter increments; milestone toast at 5.
- [ ] Filter by theme/difficulty and confirm the next puzzle respects it.
- [ ] Abandon a puzzle mid-way; session stats stay consistent.

## 4. New game & opponents
- [ ] Game config: progressive disclosure works; all three opponent modes selectable
      (choose / surprise / coach picks); teaching mode toggle; coaching level selector.
- [ ] Start a game as Black — engine moves first within a few seconds.
- [ ] Personality badge appears for surprise/coach-picks modes and fades after ~10s.

## 5. In-game coaching (the centerpiece)
- [ ] Make a few reasonable moves: coaching panel comments within ~3–6s, text **streams in**
      token by token (LLM mode) rather than appearing all at once.
- [ ] Hang a piece deliberately: feedback identifies the mistake; tone matches your level.
- [ ] Repeat the identical position/mistake in a new game: response is instant (cache).
- [ ] Pre-move hints appear at higher coaching levels; "silent" level shows nothing.
- [ ] Eval bar tracks the engine's view of the position.
- [ ] Sidebar collapses to icons during play.

## 6. Game over & summary card
- [ ] Resign: first click asks for confirmation; confirming ends the game.
- [ ] Game-over dialog: result text correct; stats (accuracy, best moves, blunders)
      fill in after a short background review — allow ~10–30s for longer games.
- [ ] Opponent line shows the real personality and ELO (e.g., "vs Aggressive (1500 ELO)"),
      not a generic placeholder.
- [ ] AI quote appears below the stats — specific to the game, under ~30 words,
      doesn't start with "Great"/"Good". If the model is missing it falls back to a
      sensible template sentence.

## 7. Review screen
- [ ] Open a finished game from history: move list with classification badges
      (best → blunder), arrow-key navigation, eval bar updates per move.
- [ ] Coaching panel explains mistakes; critical moments and pattern summary populate.
- [ ] Study suggestions reflect the game's actual weaknesses.

## 8. Openings
- [ ] Browse the library; open a detail view; add a line to the repertoire.
- [ ] Drill the repertoire: correct moves advance, wrong moves correct you; drill stats update.

## 9. Settings & model manager
- [ ] Theme switch (The Study / The Grid) applies instantly and persists across restart.
- [ ] Model Manager shows: model available, device **cpu**, size ~770 MB,
      **bundled: yes** on the installed app **[installed]**.
- [ ] Keyboard shortcuts dialog opens with `?`.

## 10. Model download flow (dev only)
Only meaningful when the model is NOT bundled (dev with `src-tauri/models/` emptied):
- [ ] Dashboard card offers download; progress bar shows speed/ETA; completes ~800 MB.
- [ ] Coaching works right after download without restarting the app.

## 11. Offline & resource checks [installed]
- [ ] Turn Wi-Fi off, launch the installed app: dashboard loads, engine plays,
      coaching streams — no network needed.
- [ ] Activity Monitor: app memory roughly 2–2.5 GB during a game with coaching
      (model resident); CPU settles when idle.

## Known behaviors (not bugs)
- First coaching response can take a few extra seconds if it beats the startup warm-up.
- Game-over stats wait on a quick engine review; the dialog is interactive meanwhile.
- Identical positions return cached coaching instantly with no streaming animation.
