// ============================================================
// shortcuts.ts — Keyboard shortcut map
// Actions call store methods directly.
// ============================================================

import { workspace } from './workspace.svelte';
import { commands } from './commands.svelte';
import { reviewStore } from './review.svelte';
import type { WindowType } from '../api/types';

export interface ShortcutDef {
  id: string;
  combo: string; // human-readable, e.g. "Ctrl+1"
  description: string;
  action: () => void;
}

// Map sidebar view index → window type
const SIDEBAR_WINDOWS: WindowType[] = [
  'command',
  'plan',
  'activity',
  'review',
  'monitor',
  'settings',
  'graph',
  // C-7: ExploreWindow Q&A oracle surface (8th sidebar slot, Ctrl+8).
  'explore',
];

/** C-7: returns true when the primary panel's active tab is a ReviewWindow.
 *  Used to guard Ctrl+Shift+A / Ctrl+Shift+R so they don't fire globally. */
function primaryActiveTabIsReview(): boolean {
  const active = workspace.primary.tabs.find(
    (t) => t.id === workspace.primary.activeTabId,
  );
  return active?.type === 'review';
}

// C-7: Ctrl+Shift+W (Tier-0 shell): deferred — 'shell' WindowType not yet
// registered in types.ts WindowType union or workspace VALID_WINDOW_TYPES.
// Wire this entry only after Tier-0 shell ships. See context-package-C7.md
// §5 and plan task #11.

export const SHORTCUTS: ShortcutDef[] = [
  {
    id: 'sidebar-1',
    combo: 'Ctrl+1',
    description: 'Switch to Command window',
    action: () => workspace.setPrimaryWindow('command'),
  },
  {
    id: 'sidebar-2',
    combo: 'Ctrl+2',
    description: 'Switch to Plan window',
    action: () => workspace.setPrimaryWindow('plan'),
  },
  {
    id: 'sidebar-3',
    combo: 'Ctrl+3',
    description: 'Switch to Activity window',
    action: () => workspace.setPrimaryWindow('activity'),
  },
  {
    id: 'sidebar-4',
    combo: 'Ctrl+4',
    description: 'Switch to Review window',
    action: () => workspace.setPrimaryWindow('review'),
  },
  {
    id: 'sidebar-5',
    combo: 'Ctrl+5',
    description: 'Switch to Monitor window',
    action: () => workspace.setPrimaryWindow('monitor'),
  },
  {
    id: 'sidebar-6',
    combo: 'Ctrl+6',
    description: 'Switch to Settings window',
    action: () => workspace.setPrimaryWindow('settings'),
  },
  {
    id: 'sidebar-7',
    combo: 'Ctrl+7',
    description: 'Switch to Graph window',
    action: () => workspace.setPrimaryWindow('graph'),
  },
  {
    // C-7: 8th sidebar slot for the ExploreWindow Q&A oracle surface.
    id: 'sidebar-8',
    combo: 'Ctrl+8',
    description: 'Switch to Explore window',
    action: () => workspace.setPrimaryWindow('explore'),
  },
  {
    id: 'open-graph',
    combo: 'Ctrl+G',
    description: 'Open Graph window in primary panel (B-33)',
    action: () => workspace.openTab('primary', 'graph'),
  },
  {
    id: 'new-tab',
    combo: 'Ctrl+N',
    description: 'New tab in active panel',
    action: () => workspace.openNewTabPicker(),
  },
  {
    id: 'close-tab',
    combo: 'Ctrl+W',
    description: 'Close active tab',
    action: () => {
      const tab = workspace.primary.tabs.find((t) => t.id === workspace.primary.activeTabId);
      if (tab) workspace.closeTab('primary', tab.id);
    },
  },
  {
    id: 'toggle-secondary',
    combo: 'Ctrl+D',
    description: 'Toggle secondary panel',
    action: () => workspace.toggleSecondary(),
  },
  {
    id: 'slash-palette',
    combo: 'Ctrl+.',
    description: 'Open slash command palette',
    action: () => {
      workspace.setPrimaryWindow('command');
      commands.openSlash();
    },
  },
  {
    id: 'settings',
    combo: 'Ctrl+,',
    description: 'Open Settings window',
    action: () => workspace.setPrimaryWindow('settings'),
  },
  {
    // C-7: previously a no-op stub pointing at openCommandPalette.
    // Rewired to open the dedicated RecoveryModal (mounted in
    // Workspace.svelte overlay snippet — see DRIFT-1).
    id: 'recover-closed',
    combo: 'Ctrl+Shift+H',
    description: 'Recover closed window',
    action: () => workspace.openWindowRecovery(),
  },
  {
    // C-7: bulk-accept all hunks in the active ReviewWindow.
    // Guarded — only fires when primary active tab is 'review' to avoid
    // global accept-all from any context.
    id: 'review-accept-all',
    combo: 'Ctrl+Shift+A',
    description: 'Accept all hunks in active Review window',
    action: () => {
      if (primaryActiveTabIsReview()) reviewStore.acceptAllHunks();
    },
  },
  {
    // C-7: bulk-reject all hunks in the active ReviewWindow.
    // Same guard as accept-all.
    id: 'review-reject-all',
    combo: 'Ctrl+Shift+R',
    description: 'Reject all hunks in active Review window',
    action: () => {
      if (primaryActiveTabIsReview()) reviewStore.rejectAllHunks();
    },
  },
  {
    id: 'command-palette',
    combo: 'Ctrl+Shift+P',
    description: 'Open command palette',
    action: () => workspace.openCommandPalette(),
  },
  {
    id: 'dismiss',
    combo: 'Escape',
    description: 'Dismiss overlay or drawer',
    action: () => {
      workspace.closeDrawer();
      workspace.closeCommandPalette();
    },
  },
];

/**
 * Install keyboard listener. Call once on app mount.
 * Returns an uninstall function.
 */
export function installShortcuts(): () => void {
  function handler(e: KeyboardEvent) {
    const ctrl = e.ctrlKey || e.metaKey;
    const shift = e.shiftKey;
    const key = e.key;

    for (const shortcut of SHORTCUTS) {
      if (matchesCombo(shortcut.combo, ctrl, shift, key)) {
        e.preventDefault();
        shortcut.action();
        return;
      }
    }
  }

  window.addEventListener('keydown', handler);
  return () => window.removeEventListener('keydown', handler);
}

function matchesCombo(combo: string, ctrl: boolean, shift: boolean, key: string): boolean {
  const parts = combo.split('+').map((p) => p.trim().toLowerCase());
  const needsCtrl = parts.includes('ctrl') || parts.includes('cmd');
  const needsShift = parts.includes('shift');
  const mainKey = parts[parts.length - 1];

  if (needsCtrl !== ctrl) return false;
  if (needsShift !== shift) return false;
  return key.toLowerCase() === mainKey;
}
