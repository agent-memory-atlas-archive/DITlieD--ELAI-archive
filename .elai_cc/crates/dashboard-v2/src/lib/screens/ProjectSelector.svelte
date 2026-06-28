<script lang="ts">
  // ============================================================
  // ProjectSelector.svelte — the project-selection screen, LEDGER
  // style (DESIGN.md §13). A desktop software-window on a SOLID --bg
  // (user decision: NO starfield here): a 48px topbar (brand wordmark
  // + center-top LOCAL/CLOUD/HYBRID modes + sun/moon theme toggle), a
  // centered <=840px content column holding the ledger panelbox
  // (titlebar + name-first ruled rows + tabular cols), and a thin
  // ~22px bottom statusbar with REAL info (session id + mode + status
  // note — NOT the dev STYLE/STATE switcher from the mockup).
  //
  // REALLY WIRED: list_projects() on mount drives the full state set
  // (loading -> data | one | empty | offline | error). No faked rows.
  // Columns are real fields only: PROJECT (name), PATH, UPDATED
  // (relative time from last_opened). Row click + "+ new project" run
  // the proven race-safe open sequence into the workspace.
  // ============================================================
  import { onMount } from 'svelte';
  import { session } from '../state/session.svelte';
  import {
    list_projects,
    open_project_from_folder,
    reindex_now,
  } from '../api/stub';
  import { reindex } from '../state/reindex.svelte';
  import { formatRelative } from '../utils/time';
  import type { ProjectEntry, ExecutionMode } from '../api/types';
  import BrandCluster from '../layout/shell/BrandCluster.svelte';
  import TopRightControls from '../chrome/TopRightControls.svelte';

  const MODES: ExecutionMode[] = ['LOCAL', 'CLOUD', 'HYBRID'];

  // ── view state ──────────────────────────────────────────────
  // The state set is driven by the REAL list_projects() outcome:
  //   loading  — call in flight
  //   data     — >1 project
  //   one      — exactly 1 project
  //   empty    — call succeeded but no projects (known store, zero rows)
  //   offline  — Tauri/backend unreachable (the call threw) but we have
  //              localStorage recents to fall back to
  //   error    — call threw AND we have no fallback recents
  type ViewState = 'loading' | 'data' | 'one' | 'empty' | 'offline' | 'error';

  let viewState = $state<ViewState>('loading');
  let rows = $state<ProjectEntry[]>([]);
  let activeMode = $state<ExecutionMode>(session.mode);
  let focusedIndex = $state<number>(-1);
  let stampedPath = $state<string | null>(null); // selected-row underline feedback
  let opening = $state<boolean>(false);
  let openingPath = $state<string | null>(null);

  function entriesFromRecents(): ProjectEntry[] {
    // session.recentProjects are ProjectSummary (last_opened_iso);
    // project them onto the ProjectEntry shape the ledger renders.
    return session.recentProjects.map((p) => ({
      name: p.name,
      path: p.path,
      last_opened_ms: new Date(p.last_opened_iso).getTime(),
    }));
  }

  async function loadProjects() {
    viewState = 'loading';
    try {
      const res = await list_projects();
      if (res.known && res.projects.length > 0) {
        // newest first
        const sorted = [...res.projects].sort(
          (a, b) => b.last_opened_ms - a.last_opened_ms,
        );
        rows = sorted;
        viewState = sorted.length === 1 ? 'one' : 'data';
      } else {
        // known store with zero rows -> try localStorage recents before
        // declaring empty (the user may have history the backend store
        // does not know about, e.g. first run of a new build).
        const fallback = entriesFromRecents();
        if (fallback.length > 0) {
          rows = fallback.sort((a, b) => b.last_opened_ms - a.last_opened_ms);
          viewState = rows.length === 1 ? 'one' : 'data';
        } else {
          rows = [];
          viewState = 'empty';
        }
      }
    } catch (err) {
      // The call threw — backend/Tauri unreachable. Fall back to
      // localStorage recents if we have any (offline), else error.
      console.warn('[ProjectSelector] list_projects failed', err);
      const fallback = entriesFromRecents();
      if (fallback.length > 0) {
        rows = fallback.sort((a, b) => b.last_opened_ms - a.last_opened_ms);
        viewState = 'offline';
      } else {
        rows = [];
        viewState = 'error';
      }
    }
  }

  onMount(() => {
    void loadProjects();

    function handleKey(e: KeyboardEvent) {
      if (e.key === 'n' && !e.ctrlKey && !e.metaKey && !e.altKey) {
        const active = document.activeElement;
        if (active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement) return;
        void openNewProject();
        return;
      }
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        focusedIndex = Math.min(focusedIndex + 1, rows.length - 1);
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        focusedIndex = Math.max(focusedIndex - 1, 0);
      }
      if (e.key === 'Enter' && focusedIndex >= 0) {
        e.preventDefault();
        const entry = rows[focusedIndex];
        if (entry) void openEntry(entry);
      }
      if (e.key === 'Escape') {
        focusedIndex = -1;
      }
    }

    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  });

  // The proven race-safe open sequence (preserved from the prior
  // implementation): open_project_from_folder -> await reindex.ready()
  // -> reindex_now() -> session.openWorkspace.
  async function openByPath(path: string) {
    if (opening) return;
    opening = true;
    openingPath = path;
    stampedPath = path; // mark the chosen row selected (teal underline)
    try {
      const resolved = await open_project_from_folder(path);
      await reindex.ready();
      void reindex_now().catch((e) => console.warn('[ProjectSelector] reindex_now failed', e));
      session.openWorkspace(resolved);
    } catch (err) {
      console.warn('[ProjectSelector] open_project_from_folder failed', err);
      stampedPath = null;
    } finally {
      opening = false;
      openingPath = null;
    }
  }

  function openEntry(entry: ProjectEntry) {
    void openByPath(entry.path);
  }

  async function pickFolder(): Promise<string | null> {
    const isTauri =
      typeof window !== 'undefined' &&
      (Boolean((window as any).__TAURI_INTERNALS__) || Boolean((window as any).__TAURI__));
    if (isTauri) {
      try {
        const dialog = await import('@tauri-apps/plugin-dialog');
        const result = await dialog.open({
          directory: true,
          multiple: false,
          title: 'Choose project folder',
        });
        return typeof result === 'string' ? result : null;
      } catch (err) {
        console.warn('[ProjectSelector] tauri dialog unavailable', err);
        return null;
      }
    }
    // Browser-dev fallback (vite dev only): prompt for a path so the
    // picker is testable without Tauri.
    const v = window.prompt('Project folder path (Tauri dialog unavailable in browser-dev mode)');
    return v?.trim() ? v.trim() : null;
  }

  async function openNewProject() {
    if (opening) return;
    const selected = await pickFolder();
    if (!selected) return;
    await openByPath(selected);
  }

  function selectMode(m: ExecutionMode) {
    activeMode = m;
    session.setMode(m);
  }

  function retry() {
    void loadProjects();
  }

  function updatedLabel(ms: number): string {
    // formatRelative defaults `now` to a fixed mock NOW (2026-04-25);
    // pass the real wall clock so live last_opened_ms renders correctly.
    return formatRelative(ms, Date.now());
  }

  const showList = $derived(viewState === 'data' || viewState === 'one');
</script>

<div class="app" aria-label="Project selector">
  <!-- ── topbar (brand + modes + theme toggle + window controls) ──
       The bar IS the custom titlebar (decorations:false): empty areas
       carry data-tauri-drag-region so the window moves; interactive
       children (brand, modes, toggle, controls) sit above it and are
       NOT draggable. Double-click a drag region = toggle-maximize. -->
  <header class="topbar" data-tauri-drag-region>
    <!-- left cluster: shared brand lockup (back chevron + wordmark) +
         the selector-only subtitle -->
    <div class="brand-row">
      <BrandCluster onBack={() => session.openIntro()} backLabel="Back to intro" />
      <span class="brand-sep" aria-hidden="true">·</span>
      <span class="brand-title">session select</span>
    </div>

    <div class="spacer" data-tauri-drag-region></div>

    <!-- center: LOCAL/CLOUD/HYBRID mode selector (DESIGN §13, START-underline) -->
    <div class="modes" role="group" aria-label="Execution mode">
      {#each MODES as m}
        <button
          class="mode"
          class:on={activeMode === m}
          type="button"
          aria-pressed={activeMode === m}
          onclick={() => selectMode(m)}
        >{m}</button>
      {/each}
    </div>

    <!-- right: shared top-right cluster — FIXED at the same viewport
         coordinates as the Intro hero + the workflow shell (top:0,
         right:6px, centered in the 48px zone) so it is pixel-identical
         and never depends on this bar's border/box-model (DESIGN §16). -->
    <div class="tr-cluster">
      <TopRightControls />
    </div>
  </header>

  <!-- ── content shell ── -->
  <div class="content">
    <div class="wrap">
      <div class="panelbox">
        <div class="head">
          <h1>recent sessions</h1>
          <span class="ct">
            {#if showList}{rows.length} {rows.length === 1 ? 'project' : 'projects'}{/if}
          </span>
        </div>

        <div class="states">
          {#if showList}
            <!-- ═══ DATA / ONE ═══ -->
            <div class="cols-head" aria-hidden="true">
              <span class="lh">PROJECT</span>
              <span class="lh">PATH</span><span class="lh upd">UPDATED</span>
            </div>
            <div class="list" aria-label="Recent projects">
              {#each rows as entry, i}
                <button
                  class="row"
                  class:focused={focusedIndex === i}
                  class:sel={stampedPath === entry.path}
                  class:loading={openingPath === entry.path}
                  type="button"
                  aria-label={`Open ${entry.name}`}
                  onmouseenter={() => (focusedIndex = i)}
                  onmouseleave={() => (focusedIndex = -1)}
                  onclick={() => openEntry(entry)}
                >
                  <span class="nm">{entry.name}</span>
                  <span class="col path" data-k="PATH">{entry.path}</span>
                  <span class="col upd" data-k="UPDATED">{updatedLabel(entry.last_opened_ms)}</span>
                </button>
              {/each}
            </div>
            <div class="newrow">
              <button class="newproj" type="button" onclick={openNewProject} disabled={opening}>
                + new project
              </button>
            </div>
          {:else if viewState === 'empty'}
            <!-- ═══ EMPTY ═══ -->
            <div class="empty-box">
              <span class="empty-glyph">○</span>
              <p class="empty-title">No sessions yet</p>
              <p class="empty-sub">Open a folder to start your first ELAI session.</p>
              <button class="newproj base" type="button" onclick={openNewProject} disabled={opening}>
                + new project
              </button>
            </div>
          {:else if viewState === 'loading'}
            <!-- ═══ LOADING ═══ -->
            <div class="skel-wrap" aria-busy="true" aria-label="Loading recent sessions">
              <div class="skel"></div><div class="skel"></div>
              <div class="skel"></div><div class="skel"></div>
            </div>
          {:else if viewState === 'error'}
            <!-- ═══ ERROR (mono inversion) ═══ -->
            <div class="fail-chip"><span class="fg">✕</span> Couldn't read recent sessions</div>
            <div class="state-action">
              <button class="newproj base" type="button" onclick={retry}>retry</button>
            </div>
          {:else if viewState === 'offline'}
            <!-- ═══ OFFLINE ═══ -->
            <div class="warn-box">
              <span class="warn-glyph">▲</span>
              <span>Backend disconnected — showing your local history.</span>
            </div>
            <div class="state-action">
              <button class="newproj base" type="button" onclick={retry}>retry connection</button>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>

  <!-- ── statusbar (REAL info; not the dev STYLE/STATE switcher) ── -->
  <footer class="statusbar">
    <span class="sb-item">{session.sessionId}</span>
    <span class="sb-sep">·</span>
    <span class="sb-item">{activeMode}</span>
    <div class="spacer"></div>
    <span class="sb-note">
      {#if viewState === 'loading'}reading sessions…
      {:else if viewState === 'offline'}backend disconnected
      {:else if viewState === 'error'}session store unavailable
      {:else if viewState === 'empty'}no sessions yet
      {:else}{rows.length} {rows.length === 1 ? 'session' : 'sessions'} ready{/if}
    </span>
  </footer>
</div>

<style>
  .app {
    position: fixed;
    inset: 0;
    display: flex;
    flex-direction: column;
    font-family: var(--font-mono);
    color: var(--text);
    background: var(--bg); /* SOLID software-window — no starfield (§13) */
  }

  /* ── topbar ── */
  .topbar {
    position: relative;
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    gap: 18px;
    /* 67px so the fixed top-right cluster (floating at top:18px) sits
       centered with an 18px inset top + bottom — DESIGN §16 */
    height: 67px;
    padding: 0 20px 0 16px;
    border-bottom: 1px solid var(--line);
    background: var(--bg);
  }
  /* top-right control cluster — fixed at the SAME viewport coordinates as
     the Intro hero + the workflow shell so the 4 buttons are pixel-identical
     across every screen, independent of this bar's border/box-model. §16. */
  .tr-cluster {
    position: fixed;
    top: 18px;
    right: 20px;
    z-index: 10;
    display: flex;
    align-items: center;
  }

  /* brand-row: the shared BrandCluster (back + wordmark) + the
     selector-only subtitle. The cluster itself styles back+wordmark. */
  .brand-row {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: none;
    -webkit-app-region: no-drag;
  }

  .brand-sep {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--ghost);
    line-height: 1;
  }
  .brand-title {
    font-family: var(--font-mono);
    font-size: 12px;
    letter-spacing: 0.1em;
    color: var(--dim);
    font-weight: 400;
  }

  .spacer {
    flex: 1;
  }

  /* modes — center-top of the bar, START-style underline, NO box.
     Absolutely centered against the relative topbar so they sit dead
     center regardless of the brand / toggle widths on either side. */
  .modes {
    position: absolute;
    left: 50%;
    top: 0;
    transform: translateX(-50%);
    height: 100%;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .mode {
    position: relative;
    appearance: none;
    background: none;
    border: 0;
    cursor: pointer;
    font-family: var(--font-mono);
    font-weight: 500;
    font-size: 11px;
    letter-spacing: 0.18em;
    color: var(--dim);
    padding: 16px 16px;
    transition: color var(--dur-fast) var(--ease-out);
  }
  .mode::after {
    content: '';
    position: absolute;
    left: 12px;
    right: 12px;
    bottom: 10px;
    height: 1px;
    background: var(--d-teal);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-slow) var(--ease-out);
  }
  .mode:hover {
    color: var(--read);
  }
  .mode:hover::after {
    transform: scaleX(1);
  }
  .mode.on {
    color: var(--d-teal-bright);
  }
  .mode.on::after {
    transform: scaleX(1);
  }

  /* ── content shell ── */
  .content {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
  }
  .content::-webkit-scrollbar {
    width: 9px;
  }
  .content::-webkit-scrollbar-thumb {
    background: var(--line2);
    border-radius: 5px;
  }
  .content::-webkit-scrollbar-track {
    background: transparent;
  }
  .wrap {
    width: 100%;
    max-width: 840px;
    margin: 0 auto;
    padding: 46px 30px 48px;
  }
  .panelbox {
    position: relative;
    width: 100%;
  }

  .head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 16px;
  }
  .head h1 {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.14em;
    color: var(--read);
  }
  .head h1::before {
    content: '~/ ';
    color: var(--d-teal);
  }
  .head .ct {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--dim);
  }

  /* ── ledger panelbox: the bordered sub-window holding the table ── */
  .states {
    min-height: 392px;
    border: 1px solid var(--line2);
    border-radius: var(--r-lg);
    background: var(--panel);
    overflow: hidden;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.025);
    padding: 18px 20px 22px;
  }

  /* table grid: name-first PROJECT / PATH / UPDATED (no index gutter) */
  .cols-head,
  .row {
    display: grid;
    grid-template-columns: 1.5fr 1.7fr 88px;
    align-items: center;
    column-gap: 16px;
  }
  .cols-head {
    padding: 0 16px 9px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    letter-spacing: 0.2em;
    color: var(--read);
    border-bottom: 1px solid var(--line2);
  }
  .cols-head .upd {
    text-align: right;
  }

  .list {
    display: flex;
    flex-direction: column;
  }
  .row {
    position: relative;
    width: 100%;
    text-align: left;
    appearance: none;
    font: inherit;
    color: inherit;
    padding: 12px 16px;
    cursor: pointer;
    background: transparent;
    border: 0;
    border-bottom: 1px solid var(--line);
    transition: background var(--dur-fast) var(--ease-out);
  }
  .row:last-child {
    border-bottom: 0;
  }
  /* hover/focus/select draws the teal baseline rule center -> out
     (START-style), and a selected row keeps it full. */
  .row::after {
    content: '';
    position: absolute;
    left: 16px;
    right: 16px;
    bottom: -1px;
    height: 1px;
    background: var(--d-teal);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-slow) var(--ease-out);
  }
  .row:hover::after,
  .row.focused::after,
  .row:focus-visible::after,
  .row.sel::after {
    transform: scaleX(1);
  }
  .row:hover .nm,
  .row.focused .nm,
  .row.sel .nm {
    color: var(--d-teal-bright);
  }
  .row.loading {
    opacity: 0.55;
    cursor: wait;
  }

  .nm {
    font-family: var(--font-mono);
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: color var(--dur-fast);
  }
  .col {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--dim);
    white-space: nowrap;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .col.path {
    direction: rtl; /* ellipsise the START of long paths, keep the leaf */
    text-align: left;
  }
  .col.upd {
    text-align: right;
    color: var(--read);
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }

  /* ── new-project control — CENTERED, square hairline + START-style
     center -> out underline-draw ── */
  .newrow {
    display: flex;
    justify-content: center;
    margin-top: 18px;
  }
  .newproj {
    appearance: none;
    cursor: pointer;
    font-family: var(--font-mono);
    background: transparent;
    color: var(--read);
    border: 1px solid var(--line2);
    border-radius: 0;
    padding: 11px 20px;
    font-size: 11px;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    position: relative;
    overflow: hidden;
    transition: background var(--dur-fast), border-color var(--dur-fast), color var(--dur-fast);
  }
  .newproj::after {
    content: '';
    position: absolute;
    left: 0;
    bottom: 0;
    width: 100%;
    height: 1px;
    background: var(--d-teal);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-slow) var(--ease-out);
  }
  .newproj:hover:not(:disabled) {
    color: var(--d-teal-bright);
    border-color: var(--d-teal-soft);
  }
  .newproj:hover:not(:disabled)::after {
    transform: scaleX(1);
  }
  .newproj:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .newproj.base {
    text-transform: none;
    letter-spacing: 0.06em;
    font-size: 12.5px;
    border-radius: var(--r-sm);
    padding: 11px 18px;
  }
  .newproj.base::after {
    display: none;
  }
  .newproj.base:hover:not(:disabled) {
    background: var(--d-teal-ghost);
  }

  /* ── EMPTY ── */
  .empty-box {
    border: 1px dashed var(--line2);
    border-radius: var(--r-md);
    padding: 34px 26px;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
    background: var(--sink);
  }
  .empty-glyph {
    font-family: var(--font-mono);
    font-size: 22px;
    color: var(--ghost);
    line-height: 1;
  }
  .empty-title {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 17px;
    font-weight: 600;
    color: var(--text);
  }
  .empty-sub {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--read);
  }
  .empty-box .newproj {
    margin-top: 6px;
  }

  /* ── LOADING skeleton ── */
  .skel-wrap {
    display: flex;
    flex-direction: column;
    gap: 9px;
  }
  .skel {
    height: 46px;
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    background: var(--sink);
    animation: skelpulse 1.4s ease-in-out infinite;
  }
  .skel:nth-child(2) { animation-delay: 0.18s; }
  .skel:nth-child(3) { animation-delay: 0.36s; }
  .skel:nth-child(4) { animation-delay: 0.54s; }
  @keyframes skelpulse {
    0%, 100% { opacity: 0.4; }
    50% { opacity: 0.8; }
  }

  /* ── ERROR (mono inversion fail-chip) ── */
  .fail-chip {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    background: var(--text);
    color: var(--bg);
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 13px;
    letter-spacing: 0.02em;
    padding: 13px 17px;
    border-radius: var(--r-md);
  }
  .fail-chip .fg {
    font-weight: 700;
  }
  .state-action {
    margin-top: 14px;
  }
  .state-action .newproj {
    margin-top: 0;
  }

  /* ── OFFLINE (warn-triangle box) ── */
  .warn-box {
    display: flex;
    align-items: center;
    gap: 11px;
    border: 1px solid var(--line2);
    border-radius: var(--r-md);
    padding: 15px 17px;
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text);
    background: var(--sink);
  }
  .warn-glyph {
    color: var(--text);
    font-size: 15px;
    line-height: 1;
  }

  /* ── statusbar (~22px target; 26px for legible padding) ── */
  .statusbar {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    gap: 9px;
    height: 26px;
    padding: 0 14px;
    border-top: 1px solid var(--line);
    background: var(--panel);
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.04em;
    color: var(--dim);
  }
  .sb-item {
    color: var(--read);
  }
  .sb-sep {
    color: var(--ghost);
  }
  .sb-note {
    color: var(--dim);
  }
</style>
