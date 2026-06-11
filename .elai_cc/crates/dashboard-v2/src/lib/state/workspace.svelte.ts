// ============================================================
// workspace.svelte.ts — Dual-panel layout state
// ============================================================

import type { WindowType, DrawerKey, TabModel, GateMode } from '../api/types';
import { logStub } from '../api/stub';

const STORAGE_KEY = 'elai.dashboard-v2.workspace.v1';
/** H-11: SEPARATE key from STORAGE_KEY so explorer rail width does not
 *  overwrite the left/right panel split ratio (pitfall from context-package:
 *  explorer-rail-width-separate-key). */
const EXPLORER_WIDTH_KEY = 'elai.dashboard-v2.explorer-width.v1';
/** H-12: SEPARATE key for the canvas-as-default flag. MUST NOT reuse STORAGE_KEY
 *  or EXPLORER_WIDTH_KEY — different persistence dimension. Using a separate key
 *  ensures existing persisted workspaces (serialized under STORAGE_KEY) are NOT
 *  invalidated by this preference toggle. Schema-safe by design.
 *  Promotion path (W-2): this flag graduates to default-ON when H-16 retires
 *  the orbit after one release with canvas-as-default shipping without regressions. */
const CANVAS_DEFAULT_KEY = 'elai.dashboard-v2.canvas-default.v1';
/** H-11: default explorer rail width in pixels. */
const EXPLORER_WIDTH_DEFAULT = 250;
/** H-11: minimum explorer rail width in pixels (below this = hidden). */
const EXPLORER_WIDTH_MIN = 120;
/** H-11: maximum explorer rail width in pixels. */
const EXPLORER_WIDTH_MAX = 400;
const VALID_WINDOW_TYPES = new Set<string>(['command', 'plan', 'activity', 'review', 'monitor', 'settings', 'graph', 'idea_framing', 'audit_timeline', 'explore', 'cockpit', 'canvas']);

/** C-7: closed-tab recovery stack max length. FIFO ring-buffer eviction. */
const CLOSED_TABS_MAX = 10;

/** C-7: one entry per closed tab. Identity-by-type semantics — restore re-opens
 *  a fresh tab of the same WindowType (NOT a state-restore of the original tab). */
export interface ClosedTabEntry {
  type: WindowType;
  title: string;
  closedAt: number;
}

function makeTabId(): string {
  // Stable UUID across hydrate cycles. crypto.randomUUID is universally available
  // in modern browsers + Tauri webview; fallback retained for hardened envs.
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return `tab-${crypto.randomUUID()}`;
  }
  return `tab-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
}

export interface PanelState {
  tabs: TabModel[];
  activeTabId: string;
}

export interface WorkspaceSerialized {
  schema: 'v1';
  primary: PanelState;
  secondary: PanelState;
  panelSplit: number;
  secondaryCollapsed: boolean;
  userCustomized: boolean;
}

function makePanel(type: WindowType): PanelState {
  const id = makeTabId();
  const label = type.charAt(0).toUpperCase() + type.slice(1);
  return { tabs: [{ id, type, label }], activeTabId: id };
}

function isValidPanelState(p: unknown): p is PanelState {
  if (!p || typeof p !== 'object') return false;
  const o = p as Record<string, unknown>;
  if (!Array.isArray(o.tabs)) return false;
  if (typeof o.activeTabId !== 'string') return false;
  for (const t of o.tabs) {
    if (!t || typeof t !== 'object') return false;
    const tt = t as Record<string, unknown>;
    if (typeof tt.id !== 'string') return false;
    if (typeof tt.label !== 'string') return false;
    if (typeof tt.type !== 'string' || !VALID_WINDOW_TYPES.has(tt.type)) return false;
  }
  return true;
}

function isValidSerialized(d: unknown): d is WorkspaceSerialized {
  if (!d || typeof d !== 'object') return false;
  const o = d as Record<string, unknown>;
  if (o.schema !== 'v1') return false;
  if (!isValidPanelState(o.primary)) return false;
  if (!isValidPanelState(o.secondary)) return false;
  if (typeof o.panelSplit !== 'number' || !isFinite(o.panelSplit)) return false;
  if (typeof o.secondaryCollapsed !== 'boolean') return false;
  if (typeof o.userCustomized !== 'boolean') return false;
  // Primary must have at least one tab (invariant).
  if ((o.primary as PanelState).tabs.length === 0) return false;
  return true;
}

function createWorkspace() {
  // Default: PRIMARY = ACTIVITY, SECONDARY = COMMAND (INTERACTIVE gate layout)
  let primary = $state<PanelState>(makePanel('activity'));
  let secondary = $state<PanelState>(makePanel('command'));
  let panelSplit = $state<number>(0.5);
  let secondaryCollapsed = $state<boolean>(false);
  let activeDrawer = $state<DrawerKey | null>(null);
  let commandPaletteOpen = $state<boolean>(false);
  /** Set to true once the user manually opens/closes/moves a tab. Blocks gate-layout resets. */
  let userCustomized = $state<boolean>(false);
  /** When true, the new-tab picker overlay is shown. Target panel in newTabPickerTarget. */
  let newTabPickerOpen = $state<boolean>(false);
  /** Which panel the new-tab picker should open the chosen tab in. */
  let newTabPickerTarget = $state<'primary' | 'secondary'>('primary');
  /**
   * Tracks an in-progress drag that may create / move a tab.
   * Set by tab-pill drag (move semantics) AND by sidebar-icon drag (open semantics).
   * Used to expose drop zone for collapsed secondary.
   */
  let tabDragging = $state<boolean>(false);
  /**
   * H-11: explorer rail width in pixels. Persisted under EXPLORER_WIDTH_KEY
   * (NOT STORAGE_KEY -- orthogonal dimension, must not overwrite panelSplit).
   * 0 = collapsed/hidden. Default: EXPLORER_WIDTH_DEFAULT (250px).
   */
  let explorerWidth = $state<number>(EXPLORER_WIDTH_DEFAULT);
  /**
   * H-12: canvas-as-default flag. When true, new sessions open with the Canvas
   * window as the primary panel (instead of Activity). Persisted under
   * CANVAS_DEFAULT_KEY (NOT STORAGE_KEY — separate dimension). Default: false.
   * The orbit remains accessible as a tab until H-16 retires it.
   * Promotion path (W-2): graduates to default-ON at H-16 after one release.
   */
  let canvasAsDefault = $state<boolean>(false);
  /**
   * C-7: ring buffer of recently-closed tabs (newest at index 0).
   * Bounded at CLOSED_TABS_MAX; oldest entry evicted on overflow.
   * Populated by closeTab(); read by RecoveryModal via the exported getter.
   */
  let closedTabs = $state<ClosedTabEntry[]>([]);
  /** C-7: RecoveryModal visibility flag. Toggled by openWindowRecovery() /
   *  closeWindowRecovery(). Mounted inside Workspace.svelte overlay snippet. */
  let showRecoveryModal = $state<boolean>(false);

  // Debounced persist machinery.
  let persistTimer: ReturnType<typeof setTimeout> | null = null;
  let persistEnabled = false; // Suppressed during initial hydrate.

  function serialize(): WorkspaceSerialized {
    return {
      schema: 'v1',
      primary: { tabs: primary.tabs.map((t) => ({ ...t })), activeTabId: primary.activeTabId },
      secondary: { tabs: secondary.tabs.map((t) => ({ ...t })), activeTabId: secondary.activeTabId },
      panelSplit,
      secondaryCollapsed,
      userCustomized,
    };
  }

  function persistNow() {
    if (typeof localStorage === 'undefined') return;
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(serialize()));
    } catch (err) {
      // QuotaExceeded, SecurityError (private mode), or storage disabled — degrade silently.
      console.debug('[workspace] persist failed', err);
    }
  }

  function schedulePersist() {
    if (!persistEnabled) return;
    if (persistTimer !== null) clearTimeout(persistTimer);
    persistTimer = setTimeout(() => {
      persistTimer = null;
      persistNow();
    }, 250);
  }

  // H-11: explorer rail width persistence (separate key, separate debounce timer).
  let explorerWidthTimer: ReturnType<typeof setTimeout> | null = null;

  function persistExplorerWidthNow() {
    if (typeof localStorage === 'undefined') return;
    try {
      localStorage.setItem(EXPLORER_WIDTH_KEY, String(explorerWidth));
    } catch (err) {
      console.debug('[workspace] persist explorerWidth failed', err);
    }
  }

  function scheduleExplorerWidthPersist() {
    if (explorerWidthTimer !== null) clearTimeout(explorerWidthTimer);
    explorerWidthTimer = setTimeout(() => {
      explorerWidthTimer = null;
      persistExplorerWidthNow();
    }, 250);
  }

  /** H-11: set explorer rail width (clamped to EXPLORER_WIDTH_MIN..MAX, or 0 for hidden). */
  function setExplorerWidth(px: number) {
    const clamped =
      px <= 0 ? 0 : Math.max(EXPLORER_WIDTH_MIN, Math.min(EXPLORER_WIDTH_MAX, px));
    explorerWidth = clamped;
    scheduleExplorerWidthPersist();
  }

  /** H-11: load persisted explorer width from localStorage. */
  function loadExplorerWidth() {
    if (typeof localStorage === 'undefined') return;
    try {
      const raw = localStorage.getItem(EXPLORER_WIDTH_KEY);
      if (raw !== null) {
        const n = parseInt(raw, 10);
        if (!isNaN(n)) {
          explorerWidth = n <= 0 ? 0 : Math.max(EXPLORER_WIDTH_MIN, Math.min(EXPLORER_WIDTH_MAX, n));
        }
      }
    } catch {
      // localStorage unavailable — use default
    }
  }

  // H-12: canvas-as-default flag persistence (separate key from STORAGE_KEY).
  function persistCanvasDefaultNow() {
    if (typeof localStorage === 'undefined') return;
    try {
      localStorage.setItem(CANVAS_DEFAULT_KEY, canvasAsDefault ? '1' : '0');
    } catch (err) {
      console.debug('[workspace] persist canvasAsDefault failed', err);
    }
  }

  /** H-12: load persisted canvas-as-default flag from localStorage. */
  function loadCanvasDefault() {
    if (typeof localStorage === 'undefined') return;
    try {
      const raw = localStorage.getItem(CANVAS_DEFAULT_KEY);
      if (raw !== null) {
        canvasAsDefault = raw === '1';
      }
    } catch {
      // localStorage unavailable — use default (false)
    }
  }

  /** H-12: set canvas-as-default flag. When true, applies canvas as primary
   *  panel unless the user has already customized the workspace layout.
   *  The single-consumer SSB handover (canvas.svelte.ts activateCanvas /
   *  deactivateCanvas) handles the subscription transfer when the canvas
   *  window mounts. */
  function setCanvasAsDefault(on: boolean) {
    canvasAsDefault = on;
    persistCanvasDefaultNow();
    // If enabling and the user has not customized the layout, apply it now.
    if (on && !userCustomized) {
      primary = makePanel('canvas');
    }
  }

  function hydrate(data: WorkspaceSerialized) {
    if (!isValidSerialized(data)) {
      console.debug('[workspace] hydrate rejected: invalid shape');
      return false;
    }
    // Suppress persistence while applying snapshot.
    const wasEnabled = persistEnabled;
    persistEnabled = false;
    primary = { tabs: data.primary.tabs.map((t) => ({ ...t })), activeTabId: data.primary.activeTabId };
    secondary = { tabs: data.secondary.tabs.map((t) => ({ ...t })), activeTabId: data.secondary.activeTabId };
    panelSplit = Math.max(0.2, Math.min(0.8, data.panelSplit));
    secondaryCollapsed = data.secondaryCollapsed;
    userCustomized = data.userCustomized;
    persistEnabled = wasEnabled;
    console.debug('[workspace] hydrate ok', { primaryTabs: primary.tabs.length, secondaryTabs: secondary.tabs.length, secondaryCollapsed });
    return true;
  }

  /** Try to load serialized state from localStorage. Returns true on success. */
  function tryLoadFromStorage(): boolean {
    if (typeof localStorage === 'undefined') return false;
    let raw: string | null;
    try {
      raw = localStorage.getItem(STORAGE_KEY);
    } catch {
      return false;
    }
    if (!raw) return false;
    let parsed: unknown;
    try {
      parsed = JSON.parse(raw);
    } catch {
      try { localStorage.removeItem(STORAGE_KEY); } catch { /* ignore */ }
      return false;
    }
    if (!isValidSerialized(parsed)) {
      try { localStorage.removeItem(STORAGE_KEY); } catch { /* ignore */ }
      return false;
    }
    return hydrate(parsed);
  }

  /** Enable autosave. Call AFTER the first hydrate-or-default decision is made. */
  function enablePersistence() {
    persistEnabled = true;
  }

  function findTab(panelId: 'primary' | 'secondary', tabId: string): TabModel | undefined {
    const panel = panelId === 'primary' ? primary : secondary;
    return panel.tabs.find((t) => t.id === tabId);
  }

  function openTab(panelId: 'primary' | 'secondary', type: WindowType) {
    const label = type.charAt(0).toUpperCase() + type.slice(1);
    const id = makeTabId();
    const tab: TabModel = { id, type, label };
    if (panelId === 'primary') {
      primary = { ...primary, tabs: [...primary.tabs, tab], activeTabId: id };
    } else {
      secondary = { ...secondary, tabs: [...secondary.tabs, tab], activeTabId: id };
      // Auto-expand secondary if it was collapsed — opening a tab implies user wants it visible.
      if (secondaryCollapsed) secondaryCollapsed = false;
    }
    userCustomized = true;
    newTabPickerOpen = false;
    logStub('workspace.openTab', { panelId, type, id });
    schedulePersist();
  }

  function closeTab(panelId: 'primary' | 'secondary', tabId: string) {
    // C-7: capture the closing tab BEFORE mutation so RecoveryModal can re-open it.
    // Identity-by-type semantics (DRIFT-5): only WindowType + title are preserved;
    // window-internal state is NOT restored.
    const sourcePanel = panelId === 'primary' ? primary : secondary;
    const closingTab = sourcePanel.tabs.find((t) => t.id === tabId);

    if (panelId === 'primary') {
      const remaining = primary.tabs.filter((t) => t.id !== tabId);
      // Primary panel must always have at least one tab (no empty primary).
      if (remaining.length === 0) return;
      const activeTabId = primary.activeTabId === tabId
        ? (remaining[remaining.length - 1]?.id ?? remaining[0]?.id ?? '')
        : primary.activeTabId;
      primary = { ...primary, tabs: remaining, activeTabId };
    } else {
      const remaining = secondary.tabs.filter((t) => t.id !== tabId);
      if (remaining.length === 0) {
        // Closing the last tab in secondary collapses the secondary panel
        // (keeps state intact for re-expansion via Ctrl+D).
        secondary = { ...secondary, tabs: [], activeTabId: '' };
        secondaryCollapsed = true;
      } else {
        const activeTabId = secondary.activeTabId === tabId
          ? (remaining[remaining.length - 1]?.id ?? remaining[0]?.id ?? '')
          : secondary.activeTabId;
        secondary = { ...secondary, tabs: remaining, activeTabId };
      }
    }

    // C-7: push to closedTabs ring buffer (FIFO eviction at CLOSED_TABS_MAX).
    // Only records tabs that were actually removed (closing-the-only-primary-tab
    // returns early above, so closingTab here always refers to a removed tab).
    if (closingTab) {
      const entry: ClosedTabEntry = {
        type: closingTab.type,
        title: closingTab.label,
        closedAt: Date.now(),
      };
      const next = [entry, ...closedTabs];
      closedTabs = next.length > CLOSED_TABS_MAX ? next.slice(0, CLOSED_TABS_MAX) : next;
    }

    userCustomized = true;
    logStub('workspace.closeTab', { panelId, tabId });
    schedulePersist();
  }

  // C-7: RecoveryModal visibility actions. Mounted inside Workspace.svelte
  // overlay snippet (DRIFT-1). Triggered by the Ctrl+Shift+H shortcut.
  function openWindowRecovery() {
    showRecoveryModal = true;
    logStub('workspace.openWindowRecovery', { count: closedTabs.length });
  }

  function closeWindowRecovery() {
    showRecoveryModal = false;
    logStub('workspace.closeWindowRecovery');
  }

  function selectTab(panelId: 'primary' | 'secondary', tabId: string) {
    if (panelId === 'primary') {
      primary = { ...primary, activeTabId: tabId };
    } else {
      secondary = { ...secondary, activeTabId: tabId };
    }
    logStub('workspace.selectTab', { panelId, tabId });
    schedulePersist();
  }

  function moveTab(tabId: string, toPanel: 'primary' | 'secondary') {
    // Find which panel currently owns the tab
    const fromPrimary = primary.tabs.some((t) => t.id === tabId);
    const fromPanel = fromPrimary ? 'primary' : 'secondary';
    if (fromPanel === toPanel) return;

    const tab = fromPrimary
      ? primary.tabs.find((t) => t.id === tabId)
      : secondary.tabs.find((t) => t.id === tabId);
    if (!tab) return;

    // primary must always have >=1 tab — refusing the move avoids closeTab no-op + duplicate-append.
    if (fromPanel === 'primary' && primary.tabs.length === 1) {
      console.warn('[workspace.moveTab] refused: primary has only 1 tab, cannot move it to secondary');
      return;
    }

    // Reveal collapsed secondary BEFORE adding the tab so the panel is visible when the tab appears.
    if (toPanel === 'secondary' && secondaryCollapsed) {
      secondaryCollapsed = false;
    }

    // Remove from source
    closeTab(fromPanel, tabId);

    // Add to dest
    if (toPanel === 'primary') {
      primary = { ...primary, tabs: [...primary.tabs, tab], activeTabId: tab.id };
    } else {
      secondary = { ...secondary, tabs: [...secondary.tabs, tab], activeTabId: tab.id };
    }
    userCustomized = true;
    console.debug('[workspace] moveTab', { tabId, fromPanel, toPanel });
    logStub('workspace.moveTab', { tabId, fromPanel, toPanel });
    schedulePersist();
  }

  function setPrimaryWindow(type: WindowType | string) {
    const resolved = VALID_WINDOW_TYPES.has(type) ? (type as WindowType) : 'command';

    // 1. If a tab of this type already exists in the primary panel, focus it.
    const existingPrimary = primary.tabs.find((t) => t.type === resolved);
    if (existingPrimary) {
      primary = { ...primary, activeTabId: existingPrimary.id };
      logStub('workspace.setPrimaryWindow.focusPrimary', { type: resolved, id: existingPrimary.id });
      schedulePersist();
      return;
    }

    // 2. If a tab of this type exists in the secondary panel, focus it there
    //    (also expand secondary if it was collapsed) — avoids creating duplicate.
    const existingSecondary = secondary.tabs.find((t) => t.type === resolved);
    if (existingSecondary) {
      secondary = { ...secondary, activeTabId: existingSecondary.id };
      if (secondaryCollapsed) secondaryCollapsed = false;
      logStub('workspace.setPrimaryWindow.focusSecondary', { type: resolved, id: existingSecondary.id });
      schedulePersist();
      return;
    }

    // 3. Fallback: replace primary with a fresh single-tab of that type.
    const id = makeTabId();
    const label = resolved.charAt(0).toUpperCase() + resolved.slice(1);
    primary = { tabs: [{ id, type: resolved, label }], activeTabId: id };
    logStub('workspace.setPrimaryWindow', { type: resolved });
    schedulePersist();
  }

  function toggleSecondary() {
    secondaryCollapsed = !secondaryCollapsed;
    // Note: deliberately NOT re-seeding when expanding into an empty secondary.
    // Reason: user explicitly closed the last tab (that's what collapsed it). Re-seeding
    // 'command' is opinionated and surprising. Instead, the empty Panel renders a
    // prominent "+" affordance that opens the window-type picker scoped to secondary
    // (see Panel.svelte). Sidebar-icon drag onto the right-edge drop strip is the
    // other way to populate it.
    logStub('workspace.toggleSecondary', { collapsed: secondaryCollapsed });
    schedulePersist();
  }

  function resetSplit() {
    panelSplit = 0.5;
    logStub('workspace.resetSplit');
    schedulePersist();
  }

  function setSplit(pct: number) {
    panelSplit = Math.max(0.2, Math.min(0.8, pct));
    schedulePersist();
  }

  function toggleDrawer(key: DrawerKey) {
    activeDrawer = activeDrawer === key ? null : key;
    logStub('workspace.toggleDrawer', { key, open: activeDrawer !== null });
  }

  function closeDrawer() {
    activeDrawer = null;
    logStub('workspace.closeDrawer');
  }

  function openCommandPalette() {
    commandPaletteOpen = true;
    logStub('workspace.openCommandPalette');
  }

  function closeCommandPalette() {
    commandPaletteOpen = false;
    logStub('workspace.closeCommandPalette');
  }

  /**
   * Apply gate-mode-aware default layout.
   * INTERACTIVE → primary=ACTIVITY, secondary=COMMAND
   * AUTONOMOUS  → primary=MONITOR,   secondary=ACTIVITY
   * Skipped if user has manually customized tabs.
   */
  function applyGateModeLayout(mode: GateMode) {
    if (userCustomized) return;
    if (mode === 'INTERACTIVE') {
      primary = makePanel('activity');
      secondary = makePanel('command');
    } else {
      primary = makePanel('monitor');
      secondary = makePanel('activity');
    }
    logStub('workspace.applyGateModeLayout', { mode });
    schedulePersist();
  }

  /**
   * Explicit user-driven layout override from Settings.
   * Bypasses the userCustomized gate (this IS the user choosing).
   * 'activity-left'  → primary=ACTIVITY, secondary=COMMAND  (INTERACTIVE default)
   * 'activity-right' → primary=COMMAND,  secondary=ACTIVITY (mirror)
   * 'auto'           → re-apply current session gate-mode default
   */
  function applyPanelLayoutOverride(layout: 'activity-left' | 'activity-right' | 'auto', gateMode: GateMode) {
    if (layout === 'activity-left') {
      primary = makePanel('activity');
      secondary = makePanel('command');
      secondaryCollapsed = false;
    } else if (layout === 'activity-right') {
      primary = makePanel('command');
      secondary = makePanel('activity');
      secondaryCollapsed = false;
    } else {
      // 'auto' — match gate-mode default regardless of userCustomized flag
      if (gateMode === 'INTERACTIVE') {
        primary = makePanel('activity');
        secondary = makePanel('command');
      } else {
        primary = makePanel('monitor');
        secondary = makePanel('activity');
      }
      secondaryCollapsed = false;
    }
    logStub('workspace.applyPanelLayoutOverride', { layout, gateMode });
    schedulePersist();
  }

  function openNewTabPicker(target: 'primary' | 'secondary' = 'primary') {
    newTabPickerTarget = target;
    newTabPickerOpen = true;
    // If aiming at the secondary panel while it's collapsed, expand it now so the
    // newly-created tab is immediately visible after the user picks a type.
    if (target === 'secondary' && secondaryCollapsed) {
      secondaryCollapsed = false;
      schedulePersist();
    }
    logStub('workspace.openNewTabPicker', { target });
  }

  function closeNewTabPicker() {
    newTabPickerOpen = false;
  }

  function setTabDragging(v: boolean) {
    tabDragging = v;
  }

  /** Force-expand the secondary panel without re-seeding tabs. Used by sidebar-icon drop logic. */
  function expandSecondary() {
    if (secondaryCollapsed) {
      secondaryCollapsed = false;
      schedulePersist();
    }
  }

  return {
    get primary() { return primary; },
    get secondary() { return secondary; },
    get panelSplit() { return panelSplit; },
    get secondaryCollapsed() { return secondaryCollapsed; },
    get activeDrawer() { return activeDrawer; },
    get commandPaletteOpen() { return commandPaletteOpen; },
    get userCustomized() { return userCustomized; },
    get newTabPickerOpen() { return newTabPickerOpen; },
    get newTabPickerTarget() { return newTabPickerTarget; },
    get tabDragging() { return tabDragging; },
    // C-7: closed-tab recovery surface.
    get closedTabs() { return closedTabs; },
    get showRecoveryModal() { return showRecoveryModal; },

    openTab,
    closeTab,
    selectTab,
    moveTab,
    setPrimaryWindow,
    toggleSecondary,
    resetSplit,
    setSplit,
    toggleDrawer,
    closeDrawer,
    openCommandPalette,
    closeCommandPalette,
    applyGateModeLayout,
    applyPanelLayoutOverride,
    openNewTabPicker,
    closeNewTabPicker,
    setTabDragging,
    expandSecondary,
    // C-7: window-recovery actions.
    openWindowRecovery,
    closeWindowRecovery,
    /** C-7: set secondaryCollapsed externally (used by ResizeObserver in
     *  Workspace.svelte to auto-collapse below 800px viewport). */
    setSecondaryCollapsed(v: boolean) {
      if (secondaryCollapsed === v) return;
      secondaryCollapsed = v;
      schedulePersist();
    },

    // Persistence API
    serialize,
    hydrate,
    tryLoadFromStorage,
    enablePersistence,
    persistNow,

    // Aliases used by SHELL-wave AppShell.svelte / Rail.svelte
    toggleSecondaryPanel() { toggleSecondary(); },
    closeOverlay() {
      activeDrawer = null;
      commandPaletteOpen = false;
      newTabPickerOpen = false;
      showRecoveryModal = false;
      logStub('workspace.closeOverlay');
    },
    /** Alias for CommandPalette.svelte compatibility. */
    resetPanelSplit() { resetSplit(); },

    /** Accepts lowercase or uppercase drawer key. */
    setActiveDrawer(key: string | null) {
      if (key === null) {
        activeDrawer = null;
      } else {
        activeDrawer = key.toLowerCase() as DrawerKey;
      }
      logStub('workspace.setActiveDrawer', { key: activeDrawer });
    },

    // H-11: explorer rail width (separate localStorage key from panelSplit).
    get explorerWidth() { return explorerWidth; },
    setExplorerWidth,
    loadExplorerWidth,

    // H-12: canvas-as-default flag (separate localStorage key CANVAS_DEFAULT_KEY).
    get canvasAsDefault() { return canvasAsDefault; },
    setCanvasAsDefault,
    loadCanvasDefault,
  };
}

export const workspace = createWorkspace();
