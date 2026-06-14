// ============================================================
// session.svelte.ts — Project + mode + gate + model session state
//
// Phase A wiring:
//   - Default screen is 'selector' (the project picker shows first).
//     Workspace renders only after a project is opened.
//   - recentProjects is persisted to localStorage so the picker shows
//     the user's actual history rather than mock fixtures.
// ============================================================

import type { ProjectSummary, ExecutionMode, GateMode } from '../api/types';
import { logStub, get_gate_status, set_execution_mode, get_execution_mode } from '../api/stub';
import { workspace } from './workspace.svelte';

const RECENTS_STORAGE_KEY = 'elai.dashboard-v2.recent_projects';
const MODE_STORAGE_KEY = 'elai.dashboard-v2.execution_mode';
const THINKING_STORAGE_KEY = 'elai.dashboard-v2.thinking_enabled';
const EFFORT_STORAGE_KEY = 'elai.dashboard-v2.effort_level';
const RECENTS_MAX = 12;

export type EffortLevel = 'low' | 'medium' | 'high' | 'max';
const EFFORT_LEVELS: readonly EffortLevel[] = ['low', 'medium', 'high', 'max'] as const;

function loadThinking(): boolean {
  if (typeof localStorage === 'undefined') return false;
  try {
    return localStorage.getItem(THINKING_STORAGE_KEY) === '1';
  } catch {
    return false;
  }
}

function saveThinking(v: boolean) {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(THINKING_STORAGE_KEY, v ? '1' : '0');
  } catch {
    // QuotaExceeded / SecurityError — degrade silently
  }
}

function loadEffort(): EffortLevel {
  if (typeof localStorage === 'undefined') return 'medium';
  try {
    const raw = localStorage.getItem(EFFORT_STORAGE_KEY);
    if (raw && (EFFORT_LEVELS as readonly string[]).includes(raw)) return raw as EffortLevel;
  } catch {
    // fall through
  }
  return 'medium';
}

function saveEffort(v: EffortLevel) {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(EFFORT_STORAGE_KEY, v);
  } catch {
    // QuotaExceeded / SecurityError — degrade silently
  }
}

function loadStoredMode(): ExecutionMode | null {
  if (typeof localStorage === 'undefined') return null;
  try {
    const raw = localStorage.getItem(MODE_STORAGE_KEY);
    if (raw === 'LOCAL' || raw === 'CLOUD' || raw === 'HYBRID') return raw;
  } catch {
    // SecurityError under file:// — fall through
  }
  return null;
}

function loadMode(): ExecutionMode {
  // First-paint default. reconcileMode() on boot will replace this with
  // the env-var-resolved backend value if localStorage is empty.
  return loadStoredMode() ?? 'LOCAL';
}

function saveMode(m: ExecutionMode) {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(MODE_STORAGE_KEY, m);
  } catch {
    // QuotaExceeded / SecurityError — degrade silently
  }
}

function loadRecents(): ProjectSummary[] {
  if (typeof localStorage === 'undefined') return [];
  try {
    const raw = localStorage.getItem(RECENTS_STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter(
      (p): p is ProjectSummary =>
        p &&
        typeof p === 'object' &&
        typeof (p as ProjectSummary).name === 'string' &&
        typeof (p as ProjectSummary).path === 'string' &&
        typeof (p as ProjectSummary).last_opened_iso === 'string',
    );
  } catch {
    return [];
  }
}

function saveRecents(list: ProjectSummary[]) {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(RECENTS_STORAGE_KEY, JSON.stringify(list.slice(0, RECENTS_MAX)));
  } catch {
    // QuotaExceeded / SecurityError — degrade silently
  }
}

function createSession() {
  // 'intro' is the FIRST screen on launch (DESIGN.md §13 cosmic brand
  // entrance) — START advances to 'selector', a project open advances
  // to 'workspace'.
  let screen = $state<'intro' | 'selector' | 'workspace'>('intro');
  let currentProject = $state<ProjectSummary | null>(null);
  let mode = $state<ExecutionMode>(loadMode());
  let gateMode = $state<GateMode>('INTERACTIVE');
  let model = $state<string>('Qwen2.5-Coder-7B-Instruct-abliterated');
  let sessionId = $state<string>(`s-${Date.now()}`);
  let recentProjects = $state<ProjectSummary[]>(loadRecents());
  // B-39D: REFLECT pipeline toggle + effort level. Persisted in localStorage.
  // When `thinkingEnabled` is true, start_chat_stream routes through the
  // ChatReflectPipeline (TRIAGE→REASONER N→AUDITOR→SYNTHESIZE) instead of
  // forwarding SSE chunks. `effortLevel` maps to REASONER best-of-N width
  // (low=2, medium=4, high=6, max=8).
  let thinkingEnabled = $state<boolean>(loadThinking());
  let effortLevel = $state<EffortLevel>(loadEffort());

  return {
    get screen() { return screen; },
    get currentProject() { return currentProject; },
    get mode() { return mode; },
    get gateMode() { return gateMode; },
    get model() { return model; },
    get sessionId() { return sessionId; },
    get recentProjects() { return recentProjects; },
    get thinkingEnabled() { return thinkingEnabled; },
    get effortLevel() { return effortLevel; },

    /** Return to the brand entrance (intro/boot) screen.
     *  Used by the ProjectSelector back button (DESIGN §13: every
     *  non-workspace screen must be reachable from the others). */
    openIntro() {
      screen = 'intro';
      logStub('session.openIntro');
    },

    openSelector() {
      screen = 'selector';
      currentProject = null;
      logStub('session.openSelector');
    },

    /** §13 brand entrance: START on the Intro screen advances to the
     *  project-selection screen. The runtime-mode + theme choices made
     *  on Intro already persisted via session.setMode / theme.set, so
     *  they carry into the selector with no extra plumbing. */
    enterSelector() {
      screen = 'selector';
      logStub('session.enterSelector');
    },

    openWorkspace(project: ProjectSummary) {
      currentProject = project;
      screen = 'workspace';
      const filtered = recentProjects.filter((p) => p.path !== project.path);
      recentProjects = [project, ...filtered].slice(0, RECENTS_MAX);
      saveRecents(recentProjects);
      logStub('session.openWorkspace', { project: project.name });
    },

    forgetProject(path: string) {
      recentProjects = recentProjects.filter((p) => p.path !== path);
      saveRecents(recentProjects);
    },

    /** Reconcile frontend mode with backend AppState.execution_mode at boot.
     *
     *  Two persistence layers compete:
     *    (a) Backend AppState resets every launch from
     *        `ELAI_DEFAULT_EXECUTION_MODE` env var (set by `elai.bat cloud`)
     *        or "LOCAL" default. In-process only.
     *    (b) Frontend localStorage carries the user's last UI choice across
     *        dashboard restarts.
     *
     *  Resolution rule: an env-var override (backend != "LOCAL" on first
     *  read) wins for that session — `elai.bat cloud` users expect their
     *  one-shot mode flag to take effect even if localStorage still says
     *  LOCAL from a prior launch. Otherwise localStorage is the source of
     *  truth and we push it to the backend. */
    async reconcileMode(): Promise<void> {
      try {
        const stored = loadStoredMode();
        const backend = await get_execution_mode().catch(() => null);
        const backendMode =
          backend && (backend.mode === 'LOCAL' || backend.mode === 'CLOUD' || backend.mode === 'HYBRID')
            ? (backend.mode as ExecutionMode)
            : null;

        if (backendMode && backendMode !== 'LOCAL' && stored !== backendMode) {
          // Env var (ELAI_DEFAULT_EXECUTION_MODE) was set by the launcher —
          // adopt it for this session without overwriting localStorage so the
          // user's persistent choice survives the next non-cloud launch.
          mode = backendMode;
        } else if (stored !== null) {
          // Persistent choice wins — sync it to the backend mirror.
          mode = stored;
          await set_execution_mode(stored);
        } else if (backendMode !== null) {
          // No localStorage yet — seed it from the backend so subsequent
          // launches without an env var still respect this value.
          mode = backendMode;
          saveMode(backendMode);
        }
      } catch (err) {
        console.warn('[session] reconcileMode failed', err);
      }
    },

    setMode(m: ExecutionMode) {
      mode = m;
      saveMode(m);
      // Fire-and-forget backend sync so the AppState mirror, SSB
      // ExecutionModeChanged row, and any future provider-router lookup all
      // see the same value. Errors are swallowed — the localStorage write
      // already happened and the UI reflects the new mode regardless.
      void set_execution_mode(m).catch((err) => {
        console.warn('[session] set_execution_mode failed', err);
      });
      logStub('session.setMode', { mode: m });
    },

    setGateMode(g: GateMode) {
      gateMode = g;
      workspace.applyGateModeLayout(g);
      logStub('session.setGateMode', { gateMode: g });
    },

    /**
     * patch-4G.1: pull the live gate mode from the backend on workspace
     * bootstrap so a server-side toggle (e.g. another client, or the next
     * session inheriting persisted backend state) propagates into the UI
     * without forcing the user to re-toggle Settings.
     *
     * Best-effort: silent no-op when the IPC fails (cold start, no SSB
     * writer, etc.). Backend canonicalises to upper-case "INTERACTIVE" /
     * "AUTONOMOUS"; defensive normalisation here in case a future caller
     * returns lower-case.
     */
    async reconcileGateMode(): Promise<void> {
      try {
        const status = await get_gate_status();
        const raw = String(status?.mode ?? '').toUpperCase();
        if (raw === 'INTERACTIVE' || raw === 'AUTONOMOUS') {
          if (gateMode !== raw) {
            gateMode = raw as GateMode;
            workspace.applyGateModeLayout(gateMode);
            logStub('session.reconcileGateMode', { gateMode });
          }
        }
      } catch {
        // non-fatal — keep current local state
      }
    },

    setModel(m: string) {
      model = m;
      logStub('session.setModel', { model: m });
    },

    /** B-39D toggle. When true, the next chat turn submits with
     *  thinking_enabled=true and the backend invokes ChatReflectPipeline. */
    setThinking(v: boolean) {
      thinkingEnabled = v;
      saveThinking(v);
      logStub('session.setThinking', { thinkingEnabled: v });
    },
    toggleThinking() {
      this.setThinking(!thinkingEnabled);
    },

    /** B-39D effort level. Maps to REASONER best-of-N width. */
    setEffort(level: EffortLevel) {
      effortLevel = level;
      saveEffort(level);
      logStub('session.setEffort', { effortLevel: level });
    },
    cycleEffort() {
      const idx = EFFORT_LEVELS.indexOf(effortLevel);
      const next = EFFORT_LEVELS[(idx + 1) % EFFORT_LEVELS.length];
      this.setEffort(next);
    },
  };
}

export const session = createSession();
