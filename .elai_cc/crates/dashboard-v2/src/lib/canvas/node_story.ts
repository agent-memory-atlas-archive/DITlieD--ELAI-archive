// node_story.ts — the "NEW DIFF" node-story model + the EXPLAIN caption layer.
//
// The user's centerpiece (canvas-design-rethink.md): opening a node tells its
// whole STORY — HISTORY / EXPLAIN / WIRING / RECEIPTS / COST / ACTIVITY — and
// the raw diff is COLLAPSED, one click deep, never shoved first. Diff fatigue
// is real (SO2025 66% almost-right); the story is what the user actually wants.
//
// EXPLAIN layer (B-13 captions.ts pattern): the plain-language captions are
// CACHED deterministic strings, not a live LLM call from the UI. A toggle swaps
// the cached string in/out; there is no network call on the render path. When a
// backend lands, these are read from the DKG concept field; the shape is stable.

export type StorySection =
  | 'history'
  | 'explain'
  | 'wiring'
  | 'receipts'
  | 'cost'
  | 'activity';

export const STORY_SECTIONS: readonly StorySection[] = Object.freeze([
  'history',
  'explain',
  'wiring',
  'receipts',
  'cost',
  'activity',
]);

export const STORY_SECTION_LABEL: Record<StorySection, string> = {
  history: 'HISTORY',
  explain: 'EXPLAIN',
  wiring: 'WIRING',
  receipts: 'RECEIPTS',
  cost: 'COST',
  activity: 'ACTIVITY',
};

/** One change in the node's HISTORY timeline (every ELAI edit). */
export interface HistoryEntry {
  ts: string;            // relative, e.g. "12m ago"
  summary: string;       // what changed (captured AT change time — near-free)
  why: string;           // why it changed
  scope: string;         // files / range
  confidence: 'high' | 'medium' | 'low';
  /** Gate receipt chips (G-5). */
  receipts: string[];
  /** The collapsed raw diff body (revealed one click deep). */
  diff: string;
}

export interface WiringFact {
  label: string;
  value: string;
  verdict: 'wired' | 'declared' | 'broken' | 'verified';
}

export interface ReceiptChip {
  gate: string;
  verdict: 'pass' | 'fail' | 'warn';
  detail: string;
}

export interface NodeStory {
  nodeId: string;
  title: string;
  /** The plain-language EXPLAIN caption (cached; toggled, no LLM call). */
  explainCached: string;
  /** A one-line safety verdict (most-requested absent vibecoder feature). */
  safetyVerdict: string;
  history: HistoryEntry[];
  wiring: WiringFact[];
  receipts: ReceiptChip[];
  cost: { tokensIn: number; tokensOut: number; usd: string; model: string };
  activity: { ts: string; line: string }[];
}

// ---------------------------------------------------------------------------
// EXPLAIN caption table — deterministic, no LLM (B-13 pattern).
//
// Until the DKG concept field is wired, an EXPLAIN string falls back to this
// frozen table keyed by a coarse node-kind. The toggle in the node-story panel
// swaps between the technical title and one of these cached plain captions.
// ---------------------------------------------------------------------------
const EXPLAIN_TABLE: Readonly<Record<string, string>> = Object.freeze({
  store: 'This holds the data the rest of the screen reads from. When it changes, the views update.',
  command: 'This is a button the user presses; it asks the engine to do one thing and waits for the answer.',
  gate: 'This is a check that has to pass before the work is allowed to continue.',
  renderer: 'This draws something on screen from data it is given; it does not decide what the data is.',
  default: 'This is one piece of the system. Open its sections to see what it does, how it is wired, and what it cost.',
});

/** Deterministic EXPLAIN lookup. Pure; no network. */
export function explainFor(nodeKind: string | null | undefined): string {
  if (!nodeKind) return EXPLAIN_TABLE.default;
  return EXPLAIN_TABLE[nodeKind] ?? EXPLAIN_TABLE.default;
}

// ---------------------------------------------------------------------------
// MOCK node-story (backend pending). The real story reads from the SSB journal
// + G-5 receipts + the DKG; this stand-in proves the surface's full shape.
// ---------------------------------------------------------------------------
export const NODE_STORY_FIXTURE: NodeStory = {
  nodeId: 'src/lib/state/shell.svelte.ts',
  title: 'shell.svelte.ts — workflow-shell IA state',
  explainCached:
    'This remembers which of the four tabs you have open and whether the side menu is showing. It does not draw anything itself.',
  safetyVerdict: 'Safe change: touches UI state only, no file writes, no network, no irreversible action.',
  history: [
    {
      ts: '12m ago',
      summary: 'Added the four-question-tab IA state wrapping the existing window machinery',
      why: 'The redesign maps each tab to an existing WindowType so no store or Tauri command is removed.',
      scope: 'shell.svelte.ts (+149 lines, new file)',
      confidence: 'high',
      receipts: ['svelte-check ✓', 'no new dep ✓'],
      diff: '+ export type ShellTab = \'progress\' | \'codebase\' | \'frontend\' | \'knowledge\';\n+ export const TAB_WINDOW: Record<ShellTab, WindowType> = { ... };',
    },
    {
      ts: '4m ago',
      summary: 'Persisted the active tab to localStorage so it survives a reload',
      why: 'Users expect the tab they left on to still be selected next session.',
      scope: 'shell.svelte.ts L72-106',
      confidence: 'high',
      receipts: ['svelte-check ✓'],
      diff: '+ function persistActiveTab() { localStorage.setItem(ACTIVE_TAB_KEY, activeTab); }',
    },
  ],
  wiring: [
    { label: 'consumed by', value: 'WorkflowShell.svelte, TabBar.svelte', verdict: 'wired' },
    { label: 'renders through', value: 'WindowHost (existing)', verdict: 'wired' },
    { label: 'settings modal', value: 'SettingsModal.svelte', verdict: 'verified' },
  ],
  receipts: [
    { gate: 'svelte-check', verdict: 'pass', detail: '0 new errors' },
    { gate: 'no-new-dep', verdict: 'pass', detail: 'no package.json change' },
    { gate: 'token-discipline', verdict: 'warn', detail: 'one legacy token still referenced (deferred)' },
  ],
  cost: { tokensIn: 18420, tokensOut: 2110, usd: '$0.07', model: 'sonnet-4.6' },
  activity: [
    { ts: '12m ago', line: 'created shell.svelte.ts' },
    { ts: '8m ago', line: 'wired TAB_WINDOW into WorkflowShell' },
    { ts: '4m ago', line: 'added localStorage persistence' },
    { ts: '1m ago', line: 'svelte-check passed' },
  ],
};
