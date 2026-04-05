# Design Critique Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Address the 5 priority design critique issues: onboarding, sidebar collapse, progressive disclosure, delight moments (pulse animations, milestone toasts, shareable summary card), review reorder, and dashboard hero treatment.

**Architecture:** Frontend-only for Tasks 1–6 and 8. Task 7 (game summary card) touches the Rust backend to add an LLM prompt for game summaries. All Svelte 5 with runes, scoped CSS, and `--cm-*` design tokens.

**Tech Stack:** Svelte 5 (runes), TypeScript, Rust (Tauri commands), CSS custom properties, `candle` LLM inference.

---

## Task 1: Review Panel Reorder (`/polish`)

**Files:**
- Modify: `src/lib/components/review/ReviewScreen.svelte`

The simplest task — just reorder existing markup blocks in the review panel. No new code.

**Current order (lines ~164–215):**
1. Summary badges
2. Key Moments
3. PatternSummaryPanel
4. Nav buttons
5. CoachingPanel
6. Move list

**New order:**
1. Summary badges
2. Key Moments
3. CoachingPanel ← moved up
4. PatternSummaryPanel
5. Nav buttons
6. Move list

- [ ] **Step 1: Reorder markup blocks**

In `ReviewScreen.svelte`, find the `{:else}` branch (after loading/empty states). Move the `<CoachingPanel>` block (currently after nav buttons) to directly after the Key Moments section, before `<PatternSummaryPanel>`.

Specifically, find these two blocks:

```svelte
      <div class="nav-buttons">
        <button onclick={() => navigateMove(-1)} disabled={selectedIndex <= -1}>
          &#9664;
        </button>
        <button onclick={() => navigateMove(1)} disabled={selectedIndex >= evaluations.length - 1}>
          &#9654;
        </button>
      </div>

      <CoachingPanel evaluation={selectedEval} />
```

Cut `<CoachingPanel evaluation={selectedEval} />` from after nav-buttons and paste it directly after the `{/if}` closing tag of the critical-moments section, before `<PatternSummaryPanel>`.

The result should read:

```svelte
      {/if}  <!-- end of critical moments -->

      <CoachingPanel evaluation={selectedEval} />

      <PatternSummaryPanel summary={patternSummary} suggestions={studySuggestions} />

      <div class="nav-buttons">
```

- [ ] **Step 2: Verify**

Run: `npx svelte-check 2>&1 | grep "COMPLETED"`
Expected: Same error/warning count as before.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/review/ReviewScreen.svelte
git commit -m "fix: reorder review panel — coaching above move list for better insight visibility"
```

---

## Task 2: Collapsible Sidebar (`/adapt`)

**Files:**
- Modify: `src/lib/components/layout/Sidebar.svelte`
- Modify: `src/App.svelte`

### Sidebar.svelte Changes

- [ ] **Step 1: Add `collapsed` prop to Sidebar**

In the script block, update the Props type and destructuring:

```ts
// Before
type Props = {
  currentPage: Page;
  onNavigate: (page: Page) => void;
};

// After
type Props = {
  currentPage: Page;
  onNavigate: (page: Page) => void;
  collapsed?: boolean;
};
```

Add to destructuring:

```ts
let { currentPage, onNavigate, collapsed = false }: Props = $props();
```

- [ ] **Step 2: Add collapsed class to sidebar container**

Change the sidebar root div:

```svelte
<!-- Before -->
<nav class="sidebar" aria-label="Main navigation">

<!-- After -->
<nav class="sidebar" class:collapsed aria-label="Main navigation">
```

- [ ] **Step 3: Add title tooltips to nav items when collapsed**

Update each nav button to include a title attribute:

```svelte
<!-- Before -->
<button
  class="nav-item"
  class:active={currentPage === item.page}
  ...
>
  <span class="nav-icon">{item.icon}</span>
  <span class="nav-label">{item.label}</span>
</button>

<!-- After -->
<button
  class="nav-item"
  class:active={currentPage === item.page}
  title={collapsed ? item.label : undefined}
  ...
>
  <span class="nav-icon">{item.icon}</span>
  <span class="nav-label">{item.label}</span>
</button>
```

- [ ] **Step 4: Add collapsed CSS styles**

Append to the `<style>` block:

```css
.sidebar.collapsed {
  width: 56px;
  transition: width var(--cm-transition-normal);
}

.sidebar:not(.collapsed) {
  transition: width var(--cm-transition-normal);
}

.sidebar.collapsed .logo-text {
  opacity: 0;
  overflow: hidden;
  width: 0;
}

.sidebar.collapsed .nav-label {
  opacity: 0;
  overflow: hidden;
  width: 0;
}

.sidebar.collapsed .nav-item {
  padding: 10px 0;
  justify-content: center;
}

.sidebar.collapsed .nav-item.active {
  padding-left: 0;
}

.sidebar.collapsed .logo {
  padding: 8px 0 24px;
  justify-content: center;
}

.sidebar.collapsed .sidebar-footer {
  display: none;
}
```

- [ ] **Step 5: Update App.svelte to derive collapsed state**

In `src/App.svelte`, in the script block, after the `page` state declaration, add:

```ts
let sidebarCollapsed = $derived(["play", "problems", "review"].includes(page));
```

Update the Sidebar rendering in the markup:

```svelte
<!-- Before -->
<Sidebar currentPage={page} onNavigate={navigate} />

<!-- After -->
<Sidebar currentPage={page} onNavigate={navigate} collapsed={sidebarCollapsed} />
```

- [ ] **Step 6: Verify**

Run: `npx svelte-check 2>&1 | grep "COMPLETED"`
Then `npm run tauri dev` — navigate between dashboard (full sidebar) and play (collapsed sidebar). Verify smooth transition, icons stay visible, labels hide.

- [ ] **Step 7: Commit**

```bash
git add src/lib/components/layout/Sidebar.svelte src/App.svelte
git commit -m "feat: collapse sidebar to icons during play, puzzles, and review"
```

---

## Task 3: GameConfig Progressive Disclosure (`/distill`)

**Files:**
- Modify: `src/lib/components/game/GameConfig.svelte`

- [ ] **Step 1: Add disclosure state and preference persistence**

In the script block, add:

```ts
let advancedOpen = $state(false);

// Load saved preferences from localStorage
function loadPreferences() {
  try {
    const saved = localStorage.getItem("chessMentor.gamePrefs");
    if (saved) {
      const prefs = JSON.parse(saved);
      if (prefs.playerColor) playerColor = prefs.playerColor;
      if (prefs.strengthPreset) strengthPreset = prefs.strengthPreset;
      if (prefs.coachingLevel) coachingLevel = prefs.coachingLevel;
      if (prefs.opponentMode) opponentMode = prefs.opponentMode;
      if (prefs.personality) personality = prefs.personality;
      if (prefs.teachingMode !== undefined) teachingMode = prefs.teachingMode;
    }
  } catch { /* ignore corrupt localStorage */ }
}

function savePreferences() {
  localStorage.setItem("chessMentor.gamePrefs", JSON.stringify({
    playerColor, strengthPreset, coachingLevel, opponentMode, personality, teachingMode,
  }));
}

loadPreferences();
```

Update `handleStart` to save preferences before calling onStart:

```ts
function handleStart() {
  savePreferences();
  // ... existing config creation and onStart call
}
```

- [ ] **Step 2: Restructure markup with disclosure**

Reorganize the form sections. Keep Play As and Engine Strength visible. Move coaching, opponent, and teaching mode into a disclosure section.

After the Engine Strength section and before the submit button, replace the coaching/opponent/teaching sections with:

```svelte
<Button variant="primary" size="lg" onclick={handleStart} disabled={!canStart || starting}>
  {#if starting}
    <LoadingSpinner size="sm" /> Starting...
  {:else}
    Start Game
  {/if}
</Button>

<div class="advanced-section">
  <button class="disclosure-toggle" onclick={() => (advancedOpen = !advancedOpen)}>
    <span class="disclosure-arrow">{advancedOpen ? "▾" : "▸"}</span>
    Customize coaching, opponent style, and more...
  </button>

  <div class="disclosure-body" class:open={advancedOpen}>
    <div class="disclosure-inner">
      <!-- Coaching section (existing markup, moved here) -->
      <div class="mb-4">
        <label class="section-label">In-Game Coaching</label>
        <!-- ... existing coaching buttons ... -->
      </div>

      <!-- Opponent Style section (existing markup, moved here) -->
      <div class="mb-4">
        <label class="section-label">Opponent Style</label>
        <!-- ... existing mode + personality + teaching mode ... -->
      </div>
    </div>
  </div>

  {#if !advancedOpen}
    <p class="defaults-note">
      Defaults: {COACHING_OPTIONS.find(c => c.value === coachingLevel)?.label ?? "Full Coach"} coaching, {PERSONALITY_OPTIONS.find(p => p.value === personality)?.label ?? "Solid"} opponent
    </p>
  {/if}
</div>
```

Note: The submit Button moves ABOVE the advanced section (this is the key UX change — Start is immediately accessible).

- [ ] **Step 3: Add disclosure CSS**

```css
.advanced-section {
  margin-top: 16px;
  border-top: 1px solid var(--cm-border-light);
  padding-top: 12px;
}

.disclosure-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: none;
  color: var(--cm-text-muted);
  font-size: 13px;
  cursor: pointer;
  padding: 4px 0;
}

.disclosure-toggle:hover {
  color: var(--cm-text-secondary);
}

.disclosure-arrow {
  font-size: 12px;
  width: 12px;
}

.disclosure-body {
  display: grid;
  grid-template-rows: 0fr;
  transition: grid-template-rows var(--cm-transition-normal);
}

.disclosure-body.open {
  grid-template-rows: 1fr;
}

.disclosure-inner {
  overflow: hidden;
  padding-top: 12px;
}

.defaults-note {
  font-size: 12px;
  color: var(--cm-text-muted);
  margin-top: 8px;
  font-style: italic;
}
```

- [ ] **Step 4: Verify**

Run: `npx svelte-check 2>&1 | grep "COMPLETED"`
Then `npm run tauri dev` — navigate to New Game. Verify:
- Only color + difficulty + Start visible initially
- Disclosure toggle opens advanced options with smooth animation
- Start Game works from collapsed state (uses defaults)
- Preferences persist across page navigations

- [ ] **Step 5: Commit**

```bash
git add src/lib/components/game/GameConfig.svelte
git commit -m "feat: progressive disclosure in GameConfig — essential options first, advanced behind toggle"
```

---

## Task 4: Dashboard Hero Treatment (`/bolder`)

**Files:**
- Modify: `src/lib/components/dashboard/Dashboard.svelte`

- [ ] **Step 1: Move RecommendationCard above the grid**

In Dashboard.svelte, find the `<div class="dashboard-grid">` section. The RecommendationCard is currently the second child inside the grid. Move it BEFORE the grid:

```svelte
<!-- Before: RecommendationCard inside grid -->
<div class="dashboard-grid">
  <div class="card skill-card">...</div>
  <RecommendationCard ... />
</div>

<!-- After: RecommendationCard above grid -->
{#if data.dailyRecommendation}
  <div class="hero-recommendation">
    <RecommendationCard
      recommendation={data.dailyRecommendation}
      onAction={handleRecommendationAction}
    />
  </div>
{/if}

<div class="dashboard-grid">
  <div class="card skill-card">...</div>
  <!-- grid is now single-column for skill card, or add other content -->
</div>
```

- [ ] **Step 2: Add hero styling**

```css
.hero-recommendation {
  margin-bottom: 20px;
}

.hero-recommendation :global(.recommendation-card) {
  border-left: 4px solid var(--cm-accent-primary);
}
```

If the RecommendationCard doesn't have a `.recommendation-card` class on its root element, target it via the wrapper instead:

```css
.hero-recommendation {
  margin-bottom: 20px;
  border-left: 4px solid var(--cm-accent-primary);
  border-radius: var(--cm-radius-md);
  overflow: hidden;
}
```

- [ ] **Step 3: Verify**

Run: `npm run tauri dev` — Dashboard should show the recommendation card as a full-width banner above the grid.

- [ ] **Step 4: Commit**

```bash
git add src/lib/components/dashboard/Dashboard.svelte
git commit -m "feat: promote recommendation card to hero position above dashboard grid"
```

---

## Task 5: Stat Pulse Animation (`/delight`)

**Files:**
- Modify: `src/styles/tokens.css` (add keyframe)
- Modify: `src/lib/components/dashboard/Dashboard.svelte` (stat values)
- Modify: `src/lib/components/problems/ProblemPanel.svelte` (streak/solved count)

- [ ] **Step 1: Add pulse keyframe to tokens.css**

Before the `/* ─── Global Focus Styles ───` section, add:

```css
/* ─── Stat Pulse Animation ─────────────────────────────────── */

@keyframes stat-pulse {
  0% { transform: scale(1); }
  50% { transform: scale(1.15); }
  100% { transform: scale(1); }
}

.stat-pulse {
  animation: stat-pulse 200ms ease;
  display: inline-block;
}
```

- [ ] **Step 2: Apply pulse to Dashboard stat values**

In Dashboard.svelte, find the stat value elements (the `.stat-value` spans showing puzzle count, accuracy, streak). The exact implementation: each stat value needs a reactive class toggle.

Add a helper in the script block:

```ts
let prevStats = $state({ solved: 0, accuracy: 0, streak: 0 });
let pulsing = $state<Set<string>>(new Set());

function checkPulse(key: string, current: number, prev: number) {
  if (current !== prev && prev !== 0) {
    pulsing.add(key);
    setTimeout(() => { pulsing.delete(key); pulsing = new Set(pulsing); }, 250);
  }
}
```

Then on each stat element:

```svelte
<span class="stat-value" class:stat-pulse={pulsing.has("solved")}>
  {data.puzzleStats.solved}
</span>
```

Use `$effect` to detect changes and trigger pulses:

```ts
$effect(() => {
  if (data?.puzzleStats) {
    checkPulse("solved", data.puzzleStats.solved, prevStats.solved);
    checkPulse("accuracy", data.puzzleStats.accuracy, prevStats.accuracy);
    checkPulse("streak", data.puzzleStats.currentStreak, prevStats.streak);
    prevStats = {
      solved: data.puzzleStats.solved,
      accuracy: data.puzzleStats.accuracy,
      streak: data.puzzleStats.currentStreak,
    };
  }
});
```

- [ ] **Step 3: Commit**

```bash
git add src/styles/tokens.css src/lib/components/dashboard/Dashboard.svelte
git commit -m "feat: add stat pulse animation on value changes"
```

---

## Task 6: Milestone Toasts (`/delight`)

**Files:**
- Modify: `src/lib/components/problems/ProblemScreen.svelte` (or ProblemPanel — wherever puzzle completion is handled)

Frontend-only milestone detection — check streak/accuracy after each puzzle and show info toasts.

- [ ] **Step 1: Add milestone detection**

In the component that handles puzzle completion, after a successful solve, add:

```ts
import { errorStore } from "../../stores/error.svelte";

function checkMilestones(streak: number, accuracy: number) {
  const streakMilestones = [5, 10, 25, 50, 100];
  if (streakMilestones.includes(streak)) {
    errorStore.show(`Puzzle streak: ${streak} in a row!`, { severity: "info", duration: 4000 });
  }
}
```

Call `checkMilestones()` after each puzzle completion event, passing the current streak from the puzzle store.

Read the file first to find exactly where puzzle completion is handled and what variables are available.

- [ ] **Step 2: Commit**

```bash
git add src/lib/components/problems/ProblemScreen.svelte
git commit -m "feat: show milestone toasts on puzzle streak achievements"
```

---

## Task 7: Shareable Game Summary Card (`/delight`)

This is the most complex task — it has a Rust backend component (LLM summary prompt) and a new Svelte component.

**Files:**
- Create: `src/lib/components/game/GameSummaryCard.svelte`
- Modify: `src/lib/components/game/GameOverDialog.svelte`
- Modify: `src-tauri/src/llm/prompts.rs`
- Modify: `src-tauri/src/commands/llm.rs`
- Modify: `src/lib/api/commands.ts`

### 7a: Backend — Game Summary Prompt

- [ ] **Step 1: Add summary prompt template in prompts.rs**

In `src-tauri/src/llm/prompts.rs`, add a new public function:

```rust
pub fn build_game_summary_prompt(
    result: &str,           // "win", "loss", "draw"
    outcome_type: &str,     // "checkmate", "resignation", "stalemate", etc.
    move_count: usize,
    accuracy_pct: f64,
    best_moves: usize,
    blunders: usize,
    mistakes: usize,
    inaccuracies: usize,
) -> String {
    let context = format!(
        r#"{{"result":"{}","outcome":"{}","moves":{},"accuracy":{:.0},"bestMoves":{},"blunders":{},"mistakes":{},"inaccuracies":{}}}"#,
        result, outcome_type, move_count, accuracy_pct, best_moves, blunders, mistakes, inaccuracies
    );

    format!(
        "<start_of_turn>user\n\
         You are a chess coach writing a brief, encouraging one-sentence summary of a student's game. \
         Be specific about what went well or what to improve. \
         Reference concrete aspects like tactical play, endgame technique, or opening preparation. \
         Keep it under 30 words. Do not start with \"Great\" or \"Good\".\n\n\
         {}<end_of_turn>\n\
         <start_of_turn>model\n",
        context
    )
}
```

- [ ] **Step 2: Add generate_game_summary command in llm.rs**

In `src-tauri/src/commands/llm.rs`, add a new Tauri command. Follow the same pattern as `generate_coaching` but simpler (no caching needed, no streaming — just return the full text):

```rust
#[tauri::command]
pub async fn generate_game_summary(
    result: String,
    outcome_type: String,
    move_count: usize,
    accuracy_pct: f64,
    best_moves: usize,
    blunders: usize,
    mistakes: usize,
    inaccuracies: usize,
    app: tauri::AppHandle,
) -> Result<String, crate::error::AppError> {
    let prompt = crate::llm::prompts::build_game_summary_prompt(
        &result, &outcome_type, move_count, accuracy_pct,
        best_moves, blunders, mistakes, inaccuracies,
    );

    // Try LLM generation with timeout
    match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        crate::llm::channel::generate_text(&app, &prompt),
    ).await {
        Ok(Ok(text)) => Ok(text),
        _ => {
            // Fallback template
            let template = if blunders == 0 && mistakes == 0 {
                "Solid game with no major errors.".to_string()
            } else if blunders > 2 {
                "A challenging game — focus on avoiding blunders in critical moments.".to_string()
            } else {
                format!("A {}-move game with room to improve accuracy.", move_count)
            };
            Ok(template)
        }
    }
}
```

Register this command in the Tauri builder (find where `generate_coaching` is registered and add `generate_game_summary` next to it).

- [ ] **Step 3: Add TypeScript wrapper in commands.ts**

In `src/lib/api/commands.ts`, add:

```ts
export function generateGameSummary(params: {
  result: string;
  outcomeType: string;
  moveCount: number;
  accuracyPct: number;
  bestMoves: number;
  blunders: number;
  mistakes: number;
  inaccuracies: number;
}): Promise<string> {
  return invoke<string>("generate_game_summary", {
    result: params.result,
    outcomeType: params.outcomeType,
    moveCount: params.moveCount,
    accuracyPct: params.accuracyPct,
    bestMoves: params.bestMoves,
    blunders: params.blunders,
    mistakes: params.mistakes,
    inaccuracies: params.inaccuracies,
  });
}
```

- [ ] **Step 4: Commit backend**

```bash
git add src-tauri/src/llm/prompts.rs src-tauri/src/commands/llm.rs src/lib/api/commands.ts
git commit -m "feat: add game summary LLM prompt and Tauri command"
```

### 7b: Frontend — GameSummaryCard Component

- [ ] **Step 5: Create GameSummaryCard.svelte**

Create `src/lib/components/game/GameSummaryCard.svelte`:

```svelte
<script lang="ts">
  type Props = {
    result: "win" | "loss" | "draw";
    outcomeDetail: string;
    opponentInfo: string;
    moveCount: number;
    accuracy: number;
    bestMoves: number;
    inaccuracies: number;
    blunders: number;
    aiQuote: string | null;
  };

  let {
    result,
    outcomeDetail,
    opponentInfo,
    moveCount,
    accuracy,
    bestMoves,
    inaccuracies,
    blunders,
    aiQuote,
  }: Props = $props();

  let resultLabel = $derived(
    result === "win" ? "Victory" : result === "loss" ? "Defeat" : "Draw"
  );

  let copied = $state(false);

  function copyToClipboard() {
    const quote = aiQuote ? `\n"${aiQuote}"` : "";
    const text = `♞ ChessMentor — ${resultLabel} ${outcomeDetail}\n${moveCount} moves · ${opponentInfo}\nAccuracy: ${accuracy}% | Best: ${bestMoves} | Blunders: ${blunders}${quote}`;
    navigator.clipboard.writeText(text).then(() => {
      copied = true;
      setTimeout(() => (copied = false), 2000);
    });
  }

  let today = new Date().toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
</script>

<div class="summary-card">
  <div class="card-top">
    <span class="branding">ChessMentor</span>
    <span class="date">{today}</span>
  </div>

  <div class="result-block">
    <h3 class="result-text" class:win={result === "win"} class:loss={result === "loss"}>
      {resultLabel}
    </h3>
    <p class="outcome-detail">{outcomeDetail}</p>
    <p class="opponent-info">{opponentInfo}</p>
  </div>

  <div class="stat-row">
    <div class="stat">
      <span class="stat-value">{accuracy}%</span>
      <span class="stat-label">Accuracy</span>
    </div>
    <div class="stat">
      <span class="stat-value best">{bestMoves}</span>
      <span class="stat-label">Best</span>
    </div>
    <div class="stat">
      <span class="stat-value inaccuracy">{inaccuracies}</span>
      <span class="stat-label">Inaccuracy</span>
    </div>
    <div class="stat">
      <span class="stat-value blunder">{blunders}</span>
      <span class="stat-label">Blunders</span>
    </div>
  </div>

  {#if aiQuote}
    <blockquote class="ai-quote">"{aiQuote}"</blockquote>
  {/if}

  <div class="card-footer">
    <span class="footer-brand">♞ chessmenter.app</span>
    <button class="copy-btn" onclick={copyToClipboard}>
      {copied ? "Copied!" : "Copy summary"}
    </button>
  </div>
</div>

<style>
  .summary-card {
    background: var(--cm-bg-surface-alt);
    border: 1px solid var(--cm-border-light);
    border-radius: var(--cm-radius-lg);
    padding: 20px;
    margin-top: 20px;
  }

  .card-top {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--cm-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 12px;
  }

  .result-block {
    text-align: center;
    margin-bottom: 16px;
  }

  .result-text {
    font-size: 28px;
    font-weight: 700;
    margin: 0 0 4px;
    color: var(--cm-text-primary);
  }

  .result-text.win { color: var(--cm-status-success); }
  .result-text.loss { color: var(--cm-status-error); }

  .outcome-detail {
    font-size: 13px;
    color: var(--cm-text-secondary);
    margin: 0;
  }

  .opponent-info {
    font-size: 12px;
    color: var(--cm-text-muted);
    margin: 2px 0 0;
  }

  .stat-row {
    display: flex;
    justify-content: space-around;
    padding: 12px 0;
    border-top: 1px solid var(--cm-border-light);
    border-bottom: 1px solid var(--cm-border-light);
    margin-bottom: 12px;
  }

  .stat {
    text-align: center;
  }

  .stat-value {
    font-size: 18px;
    font-weight: 700;
    display: block;
    color: var(--cm-text-primary);
  }

  .stat-value.best { color: var(--cm-status-success); }
  .stat-value.inaccuracy { color: var(--cm-status-warning); }
  .stat-value.blunder { color: var(--cm-status-error); }

  .stat-label {
    font-size: 10px;
    color: var(--cm-text-muted);
    text-transform: uppercase;
  }

  .ai-quote {
    font-size: 13px;
    font-style: italic;
    color: var(--cm-text-secondary);
    background: var(--cm-bg-hover);
    padding: 10px 14px;
    border-radius: var(--cm-radius-md);
    margin: 0 0 12px;
    line-height: 1.5;
    border: none;
  }

  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .footer-brand {
    font-size: 11px;
    color: var(--cm-text-disabled);
  }

  .copy-btn {
    padding: 6px 14px;
    font-size: 12px;
    background: var(--cm-bg-surface);
    border: 1px solid var(--cm-border-medium);
    border-radius: var(--cm-radius-sm);
    cursor: pointer;
    color: var(--cm-text-secondary);
    transition: background var(--cm-transition-fast);
  }

  .copy-btn:hover {
    background: var(--cm-bg-hover);
  }
</style>
```

- [ ] **Step 6: Embed in GameOverDialog**

In `GameOverDialog.svelte`:

1. Add imports:
```ts
import GameSummaryCard from "./GameSummaryCard.svelte";
import * as api from "../../api/commands";
```

2. Add state for the AI quote:
```ts
let aiQuote = $state<string | null>(null);
```

3. Add an effect to generate the summary when the dialog appears:
```ts
$effect(() => {
  if (outcome) {
    api.generateGameSummary({
      result: isWin ? "win" : outcome.type === "draw" ? "draw" : "loss",
      outcomeType: outcome.type,
      moveCount: moveCount,
      accuracyPct: 0, // Will need to be passed as a prop or computed
      bestMoves: 0,
      blunders: 0,
      mistakes: 0,
      inaccuracies: 0,
    }).then((quote) => { aiQuote = quote; })
      .catch(() => { aiQuote = null; });
  }
});
```

Note: The exact stats (accuracy, bestMoves, etc.) will need to come from the game evaluation data. Check what's available in the `outcome` prop or add additional props to GameOverDialog. Read the file to determine what data is available.

4. Add the card after the actions div:
```svelte
<GameSummaryCard
  result={isWin ? "win" : outcome?.type === "draw" ? "draw" : "loss"}
  outcomeDetail="by {outcome?.type ?? 'unknown'}"
  opponentInfo="vs {gameStore.config?.engineStrength?.elo ?? 'Engine'}"
  moveCount={moveCount}
  accuracy={0}
  bestMoves={0}
  inaccuracies={0}
  blunders={0}
  aiQuote={aiQuote}
/>
```

The implementer should read the actual props/stores available to fill in real stat values.

- [ ] **Step 7: Commit frontend**

```bash
git add src/lib/components/game/GameSummaryCard.svelte src/lib/components/game/GameOverDialog.svelte
git commit -m "feat: add shareable post-game narrative summary card with AI quote"
```

---

## Task 8: First-Puzzle Onboarding (`/onboard`)

**Files:**
- Modify: `src/App.svelte`
- Modify: `src/lib/components/problems/ProblemScreen.svelte`

- [ ] **Step 1: Add first-run detection to App.svelte**

In the script block, add state:

```ts
let onboardingComplete = $state(
  typeof localStorage !== "undefined" &&
  localStorage.getItem("chessMentor.onboardingComplete") === "true"
);

function completeOnboarding() {
  localStorage.setItem("chessMentor.onboardingComplete", "true");
  onboardingComplete = true;
  navigate("home");
}
```

- [ ] **Step 2: Route to puzzle on first run**

In the markup, update the content section. When `page === "home"` and onboarding is not complete, show ProblemScreen instead of Dashboard:

```svelte
{#if page === "home"}
  {#if !onboardingComplete}
    <ProblemScreen
      firstRun={true}
      onSkip={completeOnboarding}
      onComplete={completeOnboarding}
    />
  {:else}
    <Dashboard ... />
  {/if}
```

Add import for ProblemScreen if not already imported.

- [ ] **Step 3: Update ProblemScreen to accept firstRun props**

In `ProblemScreen.svelte` (or the appropriate problem component), update the Props type:

```ts
type Props = {
  // ... existing props
  firstRun?: boolean;
  onSkip?: () => void;
  onComplete?: () => void;
};
```

Add a banner at the top of the markup when `firstRun` is true:

```svelte
{#if firstRun}
  <div class="onboarding-banner">
    <p class="onboarding-text">Welcome! Try solving this puzzle to see how coaching works.</p>
    <button class="skip-link" onclick={onSkip}>Skip to dashboard →</button>
  </div>
{/if}
```

When a puzzle is solved in firstRun mode, call `onComplete?.()` instead of loading the next puzzle.

Add CSS:

```css
.onboarding-banner {
  padding: 12px 16px;
  background: var(--cm-accent-primary-bg);
  border-bottom: 1px solid var(--cm-accent-primary-lighter);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.onboarding-text {
  font-size: 14px;
  color: var(--cm-accent-primary-text);
  margin: 0;
}

.skip-link {
  background: none;
  border: none;
  color: var(--cm-text-muted);
  font-size: 13px;
  cursor: pointer;
  text-decoration: underline;
}
```

- [ ] **Step 4: Verify**

Clear localStorage (`localStorage.removeItem("chessMentor.onboardingComplete")`) and reload. Should see puzzle screen with banner. Solve it or click skip — should navigate to dashboard and not show again.

- [ ] **Step 5: Commit**

```bash
git add src/App.svelte src/lib/components/problems/ProblemScreen.svelte
git commit -m "feat: add skippable first-puzzle onboarding for new users"
```

---

## Verification

After all tasks are complete:

1. **Type check:** `npx svelte-check` — no new errors
2. **Build check:** `cargo build --manifest-path src-tauri/Cargo.toml` — Rust compiles
3. **Visual test:** `npm run tauri dev` — test:
   - First run: puzzle onboarding → skip or solve → dashboard
   - Dashboard: recommendation card at top (hero), stat pulses on update
   - New Game: progressive disclosure form, preferences persist
   - Play: sidebar collapses to icons, smooth transition
   - Game Over: summary card with AI quote and copy button
   - Review: coaching and key moments visible above move list
   - Puzzle streak: milestone toast at 5
4. **Theme test:** Both Study and Grid themes render correctly on all new elements
