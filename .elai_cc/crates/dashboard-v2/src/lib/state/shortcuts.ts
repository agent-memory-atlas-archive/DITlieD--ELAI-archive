// ============================================================
// shortcuts.ts — Keyboard shortcut map
// Actions call store methods directly.
// ============================================================

import { workspace } from './workspace.svelte';
import { commands } from './commands.svelte';
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
];

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
    id: 'recover-closed',
    combo: 'Ctrl+Shift+H',
    description: 'Recover closed window',
    action: () => workspace.openCommandPalette(),
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
