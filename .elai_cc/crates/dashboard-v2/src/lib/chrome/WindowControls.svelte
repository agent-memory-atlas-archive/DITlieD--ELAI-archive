<script lang="ts">
  // ============================================================
  // WindowControls.svelte — custom titlebar window controls for the
  // frameless (decorations:false) Tauri window. VS Code style:
  // minimize / maximize-restore / close, mounted at the far right of
  // each screen's draggable topbar.
  //
  // DESIGN.md §6 + §10: hairline icon buttons, ONE teal accent, NO
  // red/amber/green. Rest = --dim glyph; hover = --raised wash +
  // --text glyph. The CLOSE hover uses the §6 MONO INVERSION
  // treatment (light --text chip / dark --void glyph) to read as
  // destructive WITHOUT a foreign hue.
  //
  // Browser-dev safe: when not inside the Tauri shell
  // (window.__TAURI_INTERNALS__ absent — the same detection stub.ts
  // uses), the controls render NOTHING (the native browser chrome /
  // dev harness owns the window), so getCurrentWindow() is never
  // called and cannot throw.
  // ============================================================
  import { onMount } from 'svelte';

  // Tauri detection mirrors stub.ts: `__TAURI_INTERNALS__` (v2) or the
  // legacy `__TAURI__`. Either present => we are inside the native shell.
  const IN_TAURI: boolean =
    typeof window !== 'undefined' &&
    (Boolean((window as any).__TAURI_INTERNALS__) ||
      Boolean((window as any).__TAURI__));

  let isMaximized = $state<boolean>(false);

  // Lazily-imported window handle (only inside Tauri). Typed loosely to
  // avoid a hard top-level import of @tauri-apps/api/window in browser-dev.
  type WinHandle = {
    minimize(): Promise<void>;
    toggleMaximize(): Promise<void>;
    close(): Promise<void>;
    isMaximized(): Promise<boolean>;
    onResized(cb: () => void): Promise<() => void>;
  };
  let win: WinHandle | null = null;

  async function ensureWin(): Promise<WinHandle | null> {
    if (!IN_TAURI) return null;
    if (win) return win;
    try {
      const mod = await import('@tauri-apps/api/window');
      win = mod.getCurrentWindow() as unknown as WinHandle;
      return win;
    } catch (e) {
      console.warn('[WindowControls] @tauri-apps/api/window import failed', e);
      return null;
    }
  }

  onMount(() => {
    if (!IN_TAURI) return;
    let unlisten: (() => void) | null = null;
    void (async () => {
      const w = await ensureWin();
      if (!w) return;
      try {
        isMaximized = await w.isMaximized();
        // Keep the maximize<->restore glyph in sync with the real window
        // state (covers OS-side double-click / snap / Win+Up, not just our
        // button). onResized fires on every maximize/unmaximize/snap.
        unlisten = await w.onResized(async () => {
          try {
            isMaximized = await w.isMaximized();
          } catch {
            /* window torn down mid-listen — ignore */
          }
        });
      } catch (e) {
        console.warn('[WindowControls] init failed', e);
      }
    })();
    return () => {
      if (unlisten) unlisten();
    };
  });

  async function onMinimize() {
    const w = await ensureWin();
    if (w) await w.minimize();
  }

  async function onToggleMaximize() {
    const w = await ensureWin();
    if (!w) return;
    await w.toggleMaximize();
    // Optimistic flip; onResized reconciles the authoritative value.
    isMaximized = !isMaximized;
  }

  async function onClose() {
    const w = await ensureWin();
    if (w) await w.close();
  }
</script>

{#if IN_TAURI}
  <!-- Controls sit ABOVE any drag region (no data-tauri-drag-region here)
       so clicks land on the buttons, not the move-window handler. -->
  <div class="winctl" role="group" aria-label="Window controls">
    <button
      class="wc"
      type="button"
      title="Minimize"
      aria-label="Minimize window"
      onclick={onMinimize}
    >
      <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
        <line x1="2" y1="6" x2="10" y2="6" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
      </svg>
    </button>

    <button
      class="wc"
      type="button"
      title={isMaximized ? 'Restore' : 'Maximize'}
      aria-label={isMaximized ? 'Restore window' : 'Maximize window'}
      onclick={onToggleMaximize}
    >
      {#if isMaximized}
        <!-- restore: two offset squares -->
        <svg viewBox="0 0 12 12" width="11" height="11" fill="none" aria-hidden="true">
          <rect x="2.2" y="3.6" width="6.2" height="6.2" rx="0.6" stroke="currentColor" stroke-width="1.1" />
          <path d="M4.2 3.6V2.2h6.2v6.2H8.8" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
        </svg>
      {:else}
        <!-- maximize: single square -->
        <svg viewBox="0 0 12 12" width="11" height="11" fill="none" aria-hidden="true">
          <rect x="2.4" y="2.4" width="7.2" height="7.2" rx="0.6" stroke="currentColor" stroke-width="1.1" />
        </svg>
      {/if}
    </button>

    <button
      class="wc wc-close"
      type="button"
      title="Close"
      aria-label="Close window"
      onclick={onClose}
    >
      <svg viewBox="0 0 12 12" width="11" height="11" aria-hidden="true">
        <line x1="3" y1="3" x2="9" y2="9" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
        <line x1="9" y1="3" x2="3" y2="9" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
      </svg>
    </button>
  </div>
{/if}

<style>
  .winctl {
    display: flex;
    align-items: center;
    gap: 2px;
    /* lift above any sibling drag region so clicks hit the buttons */
    position: relative;
    z-index: 1;
  }
  .wc {
    appearance: none;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 34px;
    height: 30px;
    padding: 0;
    background: transparent;
    border: 0;
    border-radius: var(--r-sm);
    color: var(--dim);
    cursor: pointer;
    transition:
      color var(--dur-fast) var(--ease-out),
      background var(--dur-fast) var(--ease-out);
  }
  .wc:hover {
    color: var(--text);
    background: var(--raised);
  }
  .wc:focus-visible {
    outline: 1px solid var(--d-teal-soft);
    outline-offset: -2px;
  }
  /* CLOSE: §6 MONO INVERSION on hover — light --text chip / dark --void
     glyph. Reads as the loudest/destructive control with ZERO foreign
     hue (no red — §10 ban). */
  .wc-close:hover {
    background: var(--text);
    color: var(--void);
  }
  .wc-close:focus-visible {
    outline-color: var(--text);
  }
  .wc svg {
    display: block;
  }
</style>
