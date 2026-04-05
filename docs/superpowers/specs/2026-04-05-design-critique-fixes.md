# Design Critique Fixes — Spec

## Overview

Address the 5 priority issues and 4 provocative questions from the design critique. The goal is to give ChessMentor more emotional impact, better onboarding, and reduced cognitive load — without changing the core architecture or losing the app's thoughtful teaching personality.

## Decisions

All decisions were made in brainstorming with visual mockups:

- **First-run:** Skippable guided first puzzle → then dashboard
- **Sidebar:** Collapses to 56px icons-only during play/puzzles/review
- **GameConfig:** Progressive disclosure — color + difficulty + Start visible; coaching/opponent/teaching behind toggle
- **Delight:** Stat pulse animations, milestone toasts, shareable post-game narrative card
- **Review panel:** Reorder sections — coaching/key moments/patterns above move list

---

## 1. Skippable First-Puzzle Onboarding (`/onboard`)

**What:** On first launch (player has 0 games AND 0 puzzles solved), bypass the dashboard and drop the user into a single easy puzzle with full coaching. A "Skip" button in the corner returns to the dashboard.

**Flow:**
1. App detects first-run: `localStorage.getItem("chessMentor.onboardingComplete")` is null
2. Instead of rendering Dashboard, render ProblemScreen with a special `firstRun: true` prop
3. The puzzle screen shows a banner: "Welcome! Try solving this puzzle to see how coaching works."
4. A "Skip to dashboard →" link sits below the banner (subtle, not competing with the puzzle)
5. After solving (or skipping), navigate to Dashboard. The skill radar now has one data point if they solved it.
6. First-run state is stored: `localStorage.setItem("chessMentor.onboardingComplete", "true")`

**Files affected:**
- `src/App.svelte` — first-run detection and routing
- `src/lib/components/problems/ProblemScreen.svelte` — accept `firstRun` prop, show banner + skip link
- New CSS for the onboarding banner (scoped in ProblemScreen)

**What NOT to build:**
- No name entry dialog (keep "Player" default — users can change in settings later)
- No tutorial overlay or tooltip tour
- No multi-step assessment flow

---

## 2. Collapsible Sidebar During Focused Modes (`/adapt`)

**What:** When the active page is `play`, `problems`, or `review`, the sidebar shrinks from 200px to 56px (icon column only, labels hidden). On `home`, `history`, `openings`, `settings`, it stays full 200px.

**Behavior:**
- Transition: `width` animates with `var(--cm-transition-normal)` (200ms ease)
- Icons remain centered in the 56px column
- Nav item labels get `opacity: 0` and `overflow: hidden` when collapsed
- Active indicator (left border) still visible
- Logo text hides, knight icon stays
- Version text in footer hides
- Tooltip on hover shows the label text when collapsed (using `title` attribute)

**Files affected:**
- `src/lib/components/layout/Sidebar.svelte` — accept `collapsed` prop, conditional CSS
- `src/App.svelte` — derive `sidebarCollapsed` from current page: `$derived(["play", "problems", "review"].includes(page))`
- CSS changes: `.sidebar.collapsed` variant with 56px width, hidden labels

**Board sizing benefit:** The board area gains 144px of horizontal space. The `min(80vh, 560px)` board size could be increased or the side panel gets more breathing room.

---

## 3. GameConfig Progressive Disclosure (`/distill`)

**What:** The game setup form shows only two choices upfront: color and difficulty. The "Start Game" button sits immediately below. Advanced options (coaching level, opponent style, personality, teaching mode) are behind a disclosure toggle: "▸ Customize coaching, opponent style, and more..."

**Behavior:**
- Default state: color buttons + difficulty presets + Start Game + collapsed advanced section
- Clicking the disclosure toggle expands to show coaching, opponent, and teaching mode sections (with smooth height animation using `grid-template-rows: 0fr` → `1fr`)
- A muted note below the toggle: "Defaults: Full coaching, Solid opponent, Teaching mode off"
- User preferences persist to localStorage. On return visits, the form remembers their last settings but still starts collapsed (showing the saved defaults in the note text)
- The disclosure state itself does NOT persist — always starts collapsed

**Files affected:**
- `src/lib/components/game/GameConfig.svelte` — restructure markup, add disclosure state, persist preferences
- CSS: disclosure animation, section grouping

**Defaults (unchanged):**
- Color: White
- Difficulty: Beginner (1350 ELO)
- Coaching: Full Coach
- Opponent mode: Choose → Solid
- Teaching mode: Off

---

## 4. Delight Moments (`/delight`)

### 4a. Stat Pulse Animations

When a numeric stat updates (puzzle streak, rating, accuracy%), the number does a brief scale pulse: `transform: scale(1.15)` for 200ms, then settles back to `scale(1)`. CSS-only using a class toggle.

**Implementation:** A reusable `AnimatedNumber` pattern — whenever the displayed value changes, add a `.pulse` class that triggers the animation, remove it on `animationend`.

**Files affected:**
- `src/lib/components/dashboard/Dashboard.svelte` — stat values
- `src/lib/components/problems/ProblemPanel.svelte` — streak, solved count
- CSS: `.stat-pulse` keyframe animation

### 4b. Milestone Toasts

Use the existing `errorStore.show()` with `severity: "info"` to announce milestones:
- "New puzzle streak: 5 in a row!" (on streak milestones: 5, 10, 25, 50)
- "Personal best accuracy: 94%!" (when accuracy exceeds previous best)
- "Rating milestone: 1500!" (on round-number crossings)

**Implementation:** Milestone detection logic in the Rust backend. A new Tauri command `check_milestones()` called after each game/puzzle completion returns an array of milestone messages (if any). The frontend shows them as info toasts.

**Files affected:**
- `src-tauri/src/commands/` — new `check_milestones` command (or add to existing game/puzzle completion flow)
- `src/lib/components/game/PlayScreen.svelte` — call after game over
- `src/lib/components/problems/ProblemScreen.svelte` — call after puzzle completion

### 4c. Shareable Post-Game Narrative Summary Card

A styled summary card shown in the GameOverDialog (or as a new section below the outcome). Contains:
- ChessMentor branding + date
- Result (Victory/Defeat/Draw) in large colored text
- Outcome detail: "by checkmate · 34 moves · White"
- Opponent info: "vs Beginner (1350)"
- Stat row: Accuracy%, Best moves, Inaccuracies, Blunders (color-coded)
- AI coaching quote in an italicized block: "Strong tactical play in the middlegame. Your knight maneuver on move 22 was particularly well-timed."
- Footer: ♞ chessmenter.app

**Sharing mechanism:** A "Copy summary" button that copies a text version to clipboard (since Tauri doesn't easily export HTML as image). Format:

```
♞ ChessMentor — Victory by checkmate
34 moves · White vs Beginner (1350)
Accuracy: 92% | Best: 5 | Blunders: 0
"Strong tactical play in the middlegame."
```

**AI quote generation:** Reuse the existing `generate_coaching()` LLM pipeline with a new prompt template for game summaries. If the LLM is unavailable, show a template-based summary (e.g., "Solid game with no blunders.").

**Files affected:**
- New: `src/lib/components/game/GameSummaryCard.svelte` — the visual card component
- Modify: `src/lib/components/game/GameOverDialog.svelte` — embed the summary card
- `src-tauri/src/llm/prompts.rs` — new summary prompt template
- `src-tauri/src/commands/llm.rs` — new `generate_game_summary` command (or extend coaching)
- CSS: card styling respects current theme (Study warm / Grid neon)

---

## 5. Review Panel Reorder (`/polish`)

**What:** In ReviewScreen, reorder the side panel sections so coaching and insights appear above the move list (currently they're below).

**New order (top to bottom):**
1. Panel header (back + "Game Review")
2. Summary badges (blunders, mistakes, etc.)
3. Key Moments section
4. Coaching Panel (for selected move)
5. Pattern Summary + Study Suggestions
6. Navigation arrows (prev/next)
7. Move list (scrollable, takes remaining space)

**Rationale:** The most valuable insights (coaching, key moments, patterns) are now visible without scrolling. The move list — which is a navigation tool, not an insight — sinks to the bottom where it can scroll independently.

**Files affected:**
- `src/lib/components/review/ReviewScreen.svelte` — reorder markup blocks

---

## 6. Dashboard Hero Treatment (`/bolder`)

**What:** Make the dashboard's first-visit empty state and the recommendation card more visually prominent.

**Empty state (first visits with data):**
- The recommendation card gets a larger treatment: full-width instead of half-grid, with accent-colored left border and slightly larger text
- Quick Start buttons get more vertical space and visual weight

**Returning user dashboard:**
- The recommendation card spans full width above the 2-column grid (not inside it)
- This makes the daily coaching suggestion the first thing a returning player sees

**Files affected:**
- `src/lib/components/dashboard/Dashboard.svelte` — move recommendation card above grid, full-width treatment
- CSS: recommendation card hero styling

---

## Out of Scope

- Sidebar responsive collapse for mobile/tablet (separate concern)
- Name entry onboarding dialog
- Skill radar growing animation (requires skill categories to exist)
- Empty state radar evolution (same dependency)
- Header changes (keep as-is)
- Typography token adoption across all components (already addressed in audit round)
