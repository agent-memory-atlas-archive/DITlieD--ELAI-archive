// ============================================================
// theme.svelte.ts — dark / light theme store (DESIGN.md §12)
//
// Holds 'dark' | 'light', persists to localStorage, and applies
// data-theme on document.documentElement so the DESIGN role tokens
// (tokens.css :root[data-theme="light"]) flip wholesale. Default
// dark (the brand's resting state). Used by the Intro + ProjectSelector
// brand screens; the rest of the app has no light theme and is
// unaffected (the legacy token family stays dark).
// ============================================================

export type Theme = 'dark' | 'light';

const THEME_STORAGE_KEY = 'elai.dashboard-v2.theme';

function loadTheme(): Theme {
  if (typeof localStorage === 'undefined') return 'dark';
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY);
    if (raw === 'light' || raw === 'dark') return raw;
  } catch {
    // SecurityError under file:// — fall through to default
  }
  return 'dark';
}

function saveTheme(t: Theme) {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(THEME_STORAGE_KEY, t);
  } catch {
    // QuotaExceeded / SecurityError — degrade silently
  }
}

/** Reflect the current theme onto documentElement. Dark is the
 *  default token state so data-theme is only SET for light and
 *  removed for dark — this keeps dark working even if the attribute
 *  was never written. */
function applyToDocument(t: Theme) {
  if (typeof document === 'undefined') return;
  if (t === 'light') {
    document.documentElement.setAttribute('data-theme', 'light');
  } else {
    document.documentElement.setAttribute('data-theme', 'dark');
  }
}

function createTheme() {
  let current = $state<Theme>(loadTheme());
  // Apply the persisted value immediately at module init so the first
  // paint matches the stored choice (no dark->light flash).
  applyToDocument(current);

  return {
    get current() {
      return current;
    },

    set(t: Theme) {
      current = t;
      saveTheme(t);
      applyToDocument(t);
    },

    toggle() {
      this.set(current === 'dark' ? 'light' : 'dark');
    },
  };
}

export const theme = createTheme();
