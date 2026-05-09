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
import { logStub, get_gate_status } from '../api/stub';
import { workspace } from './workspace.svelte';

const RECENTS_STORAGE_KEY = 'elai.dashboard-v2.recent_projects';
const RECENTS_MAX = 12;

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
  let screen = $state<'selector' | 'workspace'>('selector');
  let currentProject = $state<ProjectSummary | null>(null);
  let mode = $state<ExecutionMode>('LOCAL');
  let gateMode = $state<GateMode>('INTERACTIVE');
  let model = $state<string>('Qwen2.5-Coder-7B-Instruct-abliterated');
  let sessionId = $state<string>(`s-${Date.now()}`);
  let recentProjects = $state<ProjectSummary[]>(loadRecents());

  return {
    get screen() { return screen; },
    get currentProject() { return currentProject; },
    get mode() { return mode; },
    get gateMode() { return gateMode; },
    get model() { return model; },
    get sessionId() { return sessionId; },
    get recentProjects() { return recentProjects; },

    openSelector() {
      screen = 'selector';
      currentProject = null;
      logStub('session.openSelector');
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

    setMode(m: ExecutionMode) {
      mode = m;
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
  };
}

export const session = createSession();
