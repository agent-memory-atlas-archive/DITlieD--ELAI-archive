// ============================================================
// shell.svelte.ts — Workflow-shell IA state (redesign 2026-06-14)
//
// Holds the NEW 4-question-tab IA chrome state that wraps the
// existing window/panel machinery:
//   - activeTab: which of the 4 question-tabs is showing
//   - flyoutOpen: the left->right menu/file-explorer flyout
//   - settingsOpen: the settings POPUP MODAL (not a window/tab)
//   - chatWidth: px width of the first-class chat pane beside the canvas
//
// This is a CHROME store; it does NOT replace workspace.svelte.ts
// (the window/tab/drag machinery + Tauri wiring stay intact). Each
// question-tab maps to an EXISTING WindowType rendered through the
// existing WindowHost. The chat is the existing CommandWindow.
//
// DESIGN.md §8 IA: primary nav = 4 question-tabs; chat alongside
// canvas; menu+files = zero-width flyout; settings = modal.
// ============================================================

import type { WindowType } from '../api/types';

/** The four question-tabs (brief §4). KNOWLEDGE name is provisional. */
export type ShellTab = 'progress' | 'codebase' | 'frontend' | 'knowledge';

export const SHELL_TABS: ShellTab[] = ['progress', 'codebase', 'frontend', 'knowledge'];

/** ALL-CAPS display labels for the tab bar. */
export const TAB_LABEL: Record<ShellTab, string> = {
  progress: 'PROGRESS',
  codebase: 'CODEBASE',
  frontend: 'FRONTEND',
  knowledge: 'KNOWLEDGE',
};

/**
 * Tab -> existing WindowType mapping. Each tab renders an EXISTING window
 * through WindowHost so every store + Tauri command stays wired:
 *   PROGRESS  -> plan      (plan tree + run state — the default home)
 *   CODEBASE  -> graph     (code graph + lenses; was the old PROJECT view)
 *   FRONTEND  -> canvas    (the visual/frontend canvas; was the old APP view)
 *   KNOWLEDGE -> explore   (memory / research / retrieval brain)
 * Refinement of the per-tab surfaces (lenses, notes-brain) is the
 * canvas-phase job; this mapping re-homes the existing surfaces cleanly.
 */
export const TAB_WINDOW: Record<ShellTab, WindowType> = {
  progress: 'plan',
  codebase: 'graph',
  frontend: 'canvas',
  knowledge: 'explore',
};

const CHAT_WIDTH_KEY = 'elai.dashboard-v2.shell.chat-width.v1';
const ACTIVE_TAB_KEY = 'elai.dashboard-v2.shell.active-tab.v1';
const CHAT_WIDTH_DEFAULT = 320;
const CHAT_WIDTH_MIN = 240;
const CHAT_WIDTH_MAX = 560;

function loadChatWidth(): number {
  if (typeof localStorage === 'undefined') return CHAT_WIDTH_DEFAULT;
  try {
    const raw = localStorage.getItem(CHAT_WIDTH_KEY);
    if (raw !== null) {
      const n = parseInt(raw, 10);
      if (!isNaN(n)) return Math.max(CHAT_WIDTH_MIN, Math.min(CHAT_WIDTH_MAX, n));
    }
  } catch {
    /* localStorage unavailable */
  }
  return CHAT_WIDTH_DEFAULT;
}

function loadActiveTab(): ShellTab {
  if (typeof localStorage === 'undefined') return 'progress';
  try {
    const raw = localStorage.getItem(ACTIVE_TAB_KEY);
    if (raw && SHELL_TABS.includes(raw as ShellTab)) return raw as ShellTab;
  } catch {
    /* localStorage unavailable */
  }
  return 'progress';
}

function createShell() {
  // PROGRESS is the default home (brief §4).
  let activeTab = $state<ShellTab>(loadActiveTab());
  let flyoutOpen = $state<boolean>(false);
  let settingsOpen = $state<boolean>(false);
  let chatWidth = $state<number>(loadChatWidth());

  function persistChatWidth() {
    if (typeof localStorage === 'undefined') return;
    try {
      localStorage.setItem(CHAT_WIDTH_KEY, String(chatWidth));
    } catch {
      /* degrade silently */
    }
  }

  function persistActiveTab() {
    if (typeof localStorage === 'undefined') return;
    try {
      localStorage.setItem(ACTIVE_TAB_KEY, activeTab);
    } catch {
      /* degrade silently */
    }
  }

  return {
    get activeTab() { return activeTab; },
    get flyoutOpen() { return flyoutOpen; },
    get settingsOpen() { return settingsOpen; },
    get chatWidth() { return chatWidth; },

    /** The WindowType the active tab renders through WindowHost. */
    get activeWindow(): WindowType { return TAB_WINDOW[activeTab]; },

    setTab(tab: ShellTab) {
      if (!SHELL_TABS.includes(tab)) return;
      activeTab = tab;
      persistActiveTab();
    },

    openFlyout() { flyoutOpen = true; },
    closeFlyout() { flyoutOpen = false; },
    toggleFlyout() { flyoutOpen = !flyoutOpen; },

    openSettings() {
      // Settings opens from the flyout; close the flyout as it opens.
      flyoutOpen = false;
      settingsOpen = true;
    },
    closeSettings() { settingsOpen = false; },

    setChatWidth(px: number) {
      chatWidth = Math.max(CHAT_WIDTH_MIN, Math.min(CHAT_WIDTH_MAX, px));
      persistChatWidth();
    },

    /** Esc handler order: modal first, then flyout. Returns true if it
     *  consumed the key (so the shell can stop propagation). */
    handleEscape(): boolean {
      if (settingsOpen) { settingsOpen = false; return true; }
      if (flyoutOpen) { flyoutOpen = false; return true; }
      return false;
    },
  };
}

export const shell = createShell();
