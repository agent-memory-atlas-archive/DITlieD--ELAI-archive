<script lang="ts">
  import { onMount } from 'svelte';
  import { session } from '../state/session.svelte';
  import { open_project_from_folder, reindex_now } from '../api/stub';
  import { reindex } from '../state/reindex.svelte';
  import { formatRelative } from '../utils/time';
  import type { ProjectSummary, ExecutionMode } from '../api/types';

  // TODO replace with ../ui/SegmentedControl.svelte after Wave 2
  // Inline fallback implementation until PRIMITIVES agent delivers it

  let mounted = $state(false);
  let activeMode = $state<ExecutionMode>(session.mode);
  let focusedIndex = $state<number>(-1);
  let opening = $state<boolean>(false);
  let openingPath = $state<string | null>(null);

  onMount(() => {
    // Small tick so CSS transitions play after first paint
    requestAnimationFrame(() => { mounted = true; });

    function handleKey(e: KeyboardEvent) {
      if (e.key === 'n' && !e.ctrlKey && !e.metaKey && !e.altKey) {
        const active = document.activeElement;
        if (active instanceof HTMLInputElement || active instanceof HTMLTextAreaElement) return;
        openNewProject();
        return;
      }
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        focusedIndex = Math.min(focusedIndex + 1, session.recentProjects.length - 1);
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        focusedIndex = Math.max(focusedIndex - 1, 0);
      }
      if (e.key === 'Enter' && focusedIndex >= 0) {
        e.preventDefault();
        const project = session.recentProjects[focusedIndex];
        if (project) openProject(project);
      }
      if (e.key === 'Escape') {
        focusedIndex = -1;
      }
    }

    window.addEventListener('keydown', handleKey);
    return () => window.removeEventListener('keydown', handleKey);
  });

  async function openProject(project: ProjectSummary) {
    if (opening) return;
    opening = true;
    openingPath = project.path;
    try {
      const resolved = await open_project_from_folder(project.path);
      // Wait for the reindex-progress listener before firing reindex_now() —
      // otherwise the synchronous 'started' event from the backend may arrive
      // before the listener is registered (W-followup-2 race).
      await reindex.ready();
      void reindex_now().catch((err) => console.warn('[ProjectSelector] reindex_now failed', err));
      session.openWorkspace(resolved);
    } finally {
      opening = false;
      openingPath = null;
    }
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
    // Browser-dev fallback (vite dev only): prompt for a path so the picker is testable without Tauri.
    const v = window.prompt('Project folder path (Tauri dialog unavailable in browser-dev mode)');
    return v?.trim() ? v.trim() : null;
  }

  async function openNewProject() {
    if (opening) return;
    opening = true;
    try {
      const selected = await pickFolder();
      if (!selected) return;
      openingPath = selected;
      const project = await open_project_from_folder(selected);
      await reindex.ready();
      void reindex_now().catch((err) => console.warn('[ProjectSelector] reindex_now failed', err));
      session.openWorkspace(project);
    } catch (err) {
      console.warn('[ProjectSelector] open_project_from_folder failed', err);
    } finally {
      opening = false;
      openingPath = null;
    }
  }

  function selectMode(m: ExecutionMode) {
    activeMode = m;
    session.setMode(m);
  }

  const MODES: ExecutionMode[] = ['LOCAL', 'CLOUD', 'HYBRID'];
</script>

<div class="selector-root" aria-label="Project selector">
  <!-- Wordmark block -->
  <div class="center-block">
    <span
      class="wordmark"
      class:mounted
      aria-label="ELAI"
      role="img"
    >ELAI</span>

    <p class="tagline" class:mounted>a deterministic development engine</p>

    <!-- Mode selector (above the divider so it sits with the wordmark) -->
    <div class="mode-selector" class:mounted aria-label="Execution mode">
      {#each MODES as m}
        <button
          class="mode-btn"
          class:active={activeMode === m}
          type="button"
          onclick={() => selectMode(m)}
          aria-pressed={activeMode === m}
        >{m}</button>
      {/each}
    </div>

    <!-- Divider -->
    <div class="divider" class:mounted aria-hidden="true"></div>

    <!-- Recent projects list -->
    <ul
      class="project-list"
      class:mounted
      role="listbox"
      aria-label="Recent projects"
    >
      {#each session.recentProjects as project, i}
        <li
          class="project-row"
          class:focused={focusedIndex === i}
          class:loading={openingPath === project.path}
          role="option"
          aria-selected={focusedIndex === i}
          tabindex="0"
          onmouseenter={() => { focusedIndex = i; }}
          onmouseleave={() => { focusedIndex = -1; }}
          onclick={() => openProject(project)}
          onkeydown={(e) => { if (e.key === 'Enter') openProject(project); }}
          style:animation-delay="{220 + i * 40}ms"
        >
          <span class="row-inset" aria-hidden="true"></span>
          <span class="project-name">{project.name}</span>
          <span class="project-path">{project.path}</span>
          <span class="project-time">{formatRelative(new Date(project.last_opened_iso).getTime())}</span>
        </li>
      {/each}
    </ul>

    <!-- New project button (outlined, no plus prefix) -->
    <button
      class="new-project-btn"
      class:mounted
      onclick={openNewProject}
      disabled={opening}
      type="button"
    >New Project</button>
  </div>
</div>

<style>
  .selector-root {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background:
      radial-gradient(
        ellipse 60% 45% at 50% 38%,
        rgba(30, 138, 122, 0.08) 0%,
        rgba(30, 138, 122, 0.03) 35%,
        transparent 70%
      ),
      radial-gradient(
        ellipse 90% 60% at 50% 100%,
        rgba(11, 14, 18, 0.0) 0%,
        rgba(7, 9, 11, 0.6) 60%,
        rgba(7, 9, 11, 0.95) 100%
      ),
      var(--bg-void);
    overflow: hidden;
  }

  /* Subtle starfield: tiny dim teal points scattered behind content */
  .selector-root::before {
    content: '';
    position: absolute;
    inset: 0;
    pointer-events: none;
    background-image:
      radial-gradient(1px 1px at 14% 22%, rgba(30,138,122,0.18), transparent 80%),
      radial-gradient(1px 1px at 78% 18%, rgba(30,138,122,0.10), transparent 80%),
      radial-gradient(1px 1px at 28% 74%, rgba(90,122,158,0.10), transparent 80%),
      radial-gradient(1px 1px at 86% 62%, rgba(30,138,122,0.10), transparent 80%),
      radial-gradient(1px 1px at 52% 88%, rgba(90,122,158,0.08), transparent 80%);
    opacity: 0.7;
  }

  /* ── Center block ── */
  .center-block {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-3);
    width: 100%;
    max-width: 480px;
    padding: 0 var(--s-4);
  }

  /* ── Wordmark ── */
  .wordmark {
    font-family: var(--font-display);
    font-weight: 200;
    letter-spacing: 0.15em;
    font-size: 56px;
    color: var(--teal);
    line-height: 1.2;
    user-select: none;
    opacity: 0;
    transform: translateY(-4px);
    transition:
      opacity 400ms var(--ease-out),
      transform 400ms var(--ease-out),
      color var(--dur-fast) var(--ease-std);
  }

  .wordmark.mounted {
    opacity: 1;
    transform: translateY(0);
  }

  /* ── Tagline ── */
  .tagline {
    margin: 0;
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 400;
    color: var(--text-secondary);
    letter-spacing: 0.02em;
    opacity: 0;
    transition: opacity 300ms var(--ease-out) 600ms;
  }

  .tagline.mounted {
    opacity: 1;
  }

  /* ── New project button — matches the .btn-outlined style used in
         the CommandWindow card buttons (Reject / Modify / Take over). ── */
  .new-project-btn {
    background: transparent;
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    padding: 4px 14px;
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
    letter-spacing: 0.01em;
    opacity: 0;
    transition:
      opacity 200ms var(--ease-out) 1400ms,
      background var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std);
    margin-top: var(--s-3);
  }

  .new-project-btn.mounted {
    opacity: 1;
  }

  .new-project-btn:hover:not(:disabled) {
    background: var(--teal-ghost);
    color: var(--text-primary);
    border-color: var(--teal-line);
  }

  .new-project-btn:focus-visible {
    outline: none;
    box-shadow: var(--ring);
  }

  .new-project-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  /* ── Divider ── */
  .divider {
    width: 100%;
    height: 1px;
    background: var(--border-subtle);
    opacity: 0;
    transition: opacity 200ms var(--ease-out) 1100ms;
  }

  .divider.mounted {
    opacity: 1;
  }

  /* ── Project list ── */
  .project-list {
    list-style: none;
    margin: 0;
    padding: 0;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .project-row {
    position: relative;
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: var(--s-3);
    height: 36px;
    padding: 0 var(--s-3);
    cursor: pointer;
    border-radius: var(--r-sm);
    outline: none;
    opacity: 0;
    animation: enterFade 200ms var(--ease-out) forwards;
    transition: background var(--dur-fast) var(--ease-std);
    overflow: hidden;
  }

  .project-row:hover,
  .project-row.focused {
    background: var(--teal-ghost);
  }

  .project-row.loading {
    opacity: 0.5;
    cursor: wait;
  }

  .project-row:focus-visible {
    box-shadow: inset 0 0 0 1px var(--teal-line);
  }

  /* Left inset bar */
  .row-inset {
    position: absolute;
    left: 0;
    top: 0;
    width: 2px;
    height: 100%;
    background: var(--teal);
    transform: scaleY(0);
    transform-origin: top;
    transition: transform 140ms var(--ease-out);
  }

  .project-row:hover .row-inset,
  .project-row.focused .row-inset {
    transform: scaleY(1);
  }

  .project-name {
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 400;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .project-path {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    justify-self: start;
  }

  .project-time {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
    white-space: nowrap;
    justify-self: end;
    flex-shrink: 0;
  }

  /* ── Mode selector ── */
  .mode-selector {
    display: flex;
    gap: var(--s-4);
    margin-top: var(--s-2);
    opacity: 0;
    transition: opacity 200ms var(--ease-out) 1020ms;
  }

  .mode-selector.mounted {
    opacity: 1;
  }

  .mode-btn {
    background: none;
    border: none;
    padding: var(--s-1) var(--s-1);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 400;
    color: var(--text-ghost);
    letter-spacing: 0.08em;
    cursor: pointer;
    position: relative;
    transition:
      color var(--dur-fast) var(--ease-std);
    outline: none;
  }

  .mode-btn::after {
    content: '';
    position: absolute;
    bottom: -2px;
    left: 0;
    right: 0;
    height: 1px;
    background: var(--teal);
    transform: scaleX(0);
    transition: transform var(--dur-base) var(--ease-out);
  }

  .mode-btn.active {
    color: var(--teal);
  }

  .mode-btn.active::after {
    transform: scaleX(1);
  }

  .mode-btn:hover:not(.active) {
    color: var(--text-secondary);
  }

  .mode-btn:focus-visible {
    box-shadow: var(--ring);
    border-radius: var(--r-xs);
  }

  /* ── Animations ── */
  @keyframes enterFade {
    from { opacity: 0; transform: translateY(2px); }
    to   { opacity: 1; transform: translateY(0); }
  }
</style>
