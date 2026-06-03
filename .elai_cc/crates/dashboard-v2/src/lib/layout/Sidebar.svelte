<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Terminal,
    Layers,
    Play,
    GitBranch,
    Activity,
    Settings,
    UserRound,
    Network,
    Compass,
    Menu,
  } from '@lucide/svelte';
  import type { Component } from 'svelte';
  import type { LucideProps } from '@lucide/svelte';
  import type { WindowType, ExecutionMode } from '../api/types';
  import { workspace } from '../state/workspace.svelte';
  import { session } from '../state/session.svelte';

  // @lucide/svelte v1 exports Svelte 5 function components.
  type IconCtor = Component<LucideProps>;

  interface NavItem {
    type: WindowType;
    label: string;
    shortcut: string;
    Icon: IconCtor;
  }

  const navItems: NavItem[] = [
    { type: 'command',  label: 'Command',  shortcut: 'Ctrl+1', Icon: Terminal },
    { type: 'plan',     label: 'Plan',     shortcut: 'Ctrl+2', Icon: Layers },
    { type: 'activity', label: 'Activity', shortcut: 'Ctrl+3', Icon: Play },
    { type: 'review',   label: 'Review',   shortcut: 'Ctrl+4', Icon: GitBranch },
    { type: 'monitor',  label: 'Monitor',  shortcut: 'Ctrl+5', Icon: Activity },
    { type: 'settings', label: 'Settings', shortcut: 'Ctrl+6', Icon: Settings },
    { type: 'graph',    label: 'Graph',    shortcut: 'Ctrl+7', Icon: Network },
    // C-7: Q&A oracle / codebase-explorer surface.
    { type: 'explore',  label: 'Explore',  shortcut: 'Ctrl+8', Icon: Compass },
  ];

  // C-7: hamburger toggle for narrow viewports (< 1000px). Tracks viewport
  // width via a window resize listener so the @media-driven CSS visibility
  // stays in sync with the local sidebarCollapsed state.
  const NARROW_VIEWPORT_PX = 1000;
  let viewportIsNarrow = $state<boolean>(false);
  let sidebarCollapsed = $state<boolean>(false);

  function syncViewport() {
    if (typeof window === 'undefined') return;
    const narrow = window.innerWidth < NARROW_VIEWPORT_PX;
    if (narrow !== viewportIsNarrow) {
      viewportIsNarrow = narrow;
      // Auto-collapse on entering narrow mode; auto-expand on leaving.
      sidebarCollapsed = narrow;
    }
  }

  function toggleSidebar() {
    sidebarCollapsed = !sidebarCollapsed;
  }

  onMount(() => {
    syncViewport();
    window.addEventListener('resize', syncViewport, { passive: true });
    return () => {
      window.removeEventListener('resize', syncViewport);
    };
  });

  // Derive active window type from the workspace primary panel's active tab
  let activeWindow = $derived(
    workspace.primary.tabs.find((t) => t.id === workspace.primary.activeTabId)?.type ?? 'command'
  );

  // Mode cycle order
  const MODE_CYCLE: ExecutionMode[] = ['LOCAL', 'CLOUD', 'HYBRID'];

  function cycleMode() {
    const idx = MODE_CYCLE.indexOf(session.mode);
    const next = MODE_CYCLE[(idx + 1) % MODE_CYCLE.length];
    session.setMode(next);
  }

  function tooltip(node: HTMLElement, opts: { label: string; kbd: string }) {
    let tip: HTMLElement | null = null;
    let showTimer: ReturnType<typeof setTimeout> | null = null;

    function show() {
      if (!tip) {
        tip = document.getElementById('sidebar-tooltip') as HTMLElement | null;
        if (!tip) {
          tip = document.createElement('div');
          tip.id = 'sidebar-tooltip';
          tip.className = 'sidebar-tooltip-global';
          document.body.appendChild(tip);
        }
      }
      const rect = node.getBoundingClientRect();
      tip.innerHTML =
        `<span class="tip-label">${opts.label}</span>` +
        `<kbd class="tip-kbd">${opts.kbd}</kbd>`;
      tip.style.top = `${rect.top + rect.height / 2}px`;
      tip.style.left = `${rect.right + 8}px`;
      tip.style.opacity = '1';
      tip.style.pointerEvents = 'none';
    }

    function hide() {
      if (tip) {
        tip.style.opacity = '0';
      }
    }

    function onMouseEnter() {
      showTimer = setTimeout(show, 400);
    }

    function onMouseLeave() {
      if (showTimer) {
        clearTimeout(showTimer);
        showTimer = null;
      }
      hide();
    }

    node.addEventListener('mouseenter', onMouseEnter);
    node.addEventListener('mouseleave', onMouseLeave);

    return {
      update(newOpts: { label: string; kbd: string }) {
        opts = newOpts;
      },
      destroy() {
        node.removeEventListener('mouseenter', onMouseEnter);
        node.removeEventListener('mouseleave', onMouseLeave);
        if (showTimer) clearTimeout(showTimer);
      },
    };
  }

  function handleNavClick(type: WindowType) {
    workspace.setPrimaryWindow(type);
  }

  // Sidebar-icon drag = OPEN a new tab of this type wherever it's dropped.
  // Distinct from Panel.svelte's tab-pill drag (text/elai-tab) which carries
  // MOVE semantics. Drop handlers in Panel.svelte and AppShell.svelte sniff for
  // this MIME and call workspace.openTab(target, type) on drop.
  const MIME_TAB_TYPE = 'application/x-elai-tab-type';

  function handleNavDragStart(e: DragEvent, type: WindowType) {
    if (!e.dataTransfer) return;
    e.dataTransfer.setData(MIME_TAB_TYPE, type);
    e.dataTransfer.effectAllowed = 'copy';
    // Reuse the existing tabDragging flag so AppShell's collapsed-secondary
    // drop-strip lights up for both kinds of drags. This is a deliberate
    // semantic overload — keeps the visual cue consistent.
    workspace.setTabDragging(true);
  }

  function handleNavDragEnd() {
    workspace.setTabDragging(false);
  }

  function handleWordmarkClick() {
    session.openSelector();
  }

  function handleSessionClick() {
    workspace.openCommandPalette();
  }
</script>

<nav
  class="sidebar"
  class:sidebar--narrow={viewportIsNarrow}
  class:sidebar--collapsed={viewportIsNarrow && sidebarCollapsed}
  aria-label="Primary navigation"
>
  <div class="wordmark-wrap">
    <button
      class="wordmark"
      onclick={handleWordmarkClick}
      title="Back to project selector"
      aria-label="ELAI — open project selector"
    >
      ELAI
    </button>
  </div>

  {#if viewportIsNarrow}
    <button
      class="hamburger-btn"
      onclick={toggleSidebar}
      aria-label={sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
      aria-expanded={!sidebarCollapsed}
      data-testid="sidebar-hamburger"
    >
      <Menu size={18} stroke-width={1.5} />
    </button>
  {/if}

  <div class="divider" role="separator" aria-hidden="true"></div>

  <ul class="nav-list" role="list">
    {#each navItems as item (item.type)}
      {@const isActive = activeWindow === item.type}
      <li>
        <button
          class="nav-btn"
          class:nav-btn--active={isActive}
          draggable={true}
          onclick={() => handleNavClick(item.type)}
          ondragstart={(e) => handleNavDragStart(e, item.type)}
          ondragend={handleNavDragEnd}
          aria-label={item.label}
          aria-current={isActive ? 'page' : undefined}
          use:tooltip={{ label: item.label, kbd: item.shortcut }}
        >
          {#if isActive}
            <span class="active-bar" aria-hidden="true"></span>
          {/if}
          <span class="icon-wrap" aria-hidden="true">
            <item.Icon size={20} stroke-width={1.5} />
          </span>
        </button>
      </li>
    {/each}
  </ul>

  <div class="sidebar-bottom">
    <button
      class="mode-dot-btn"
      onclick={cycleMode}
      title="Mode: {session.mode} — click to cycle"
      aria-label="Mode: {session.mode} — click to cycle"
    >
      <span
        class="mode-dot"
        class:mode-dot--local={session.mode === 'LOCAL'}
        class:mode-dot--cloud={session.mode === 'CLOUD'}
        class:mode-dot--hybrid={session.mode === 'HYBRID'}
        aria-hidden="true"
      ></span>
    </button>
    <button
      class="session-btn"
      onclick={handleSessionClick}
      aria-label="Open command palette"
      use:tooltip={{ label: 'Command Palette', kbd: 'Ctrl+Shift+P' }}
    >
      <UserRound size={20} stroke-width={1.5} />
    </button>
  </div>
</nav>

<style>
  /* C-6: width 64px -> 48px to match spec dashboard-design.txt:182.
     Coupled change in AppShell.svelte grid-template-columns. */
  .sidebar {
    display: flex;
    flex-direction: column;
    width: 48px;
    height: 100%;
    background: var(--bg-void);
  }

  .wordmark-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 64px;
    flex-shrink: 0;
  }

  .wordmark {
    font-family: var(--font-display);
    font-weight: 200;
    font-size: 17px;
    letter-spacing: 0.22em;
    color: var(--teal);
    text-transform: uppercase;
    background: none;
    border: none;
    cursor: pointer;
    padding: 6px 4px 6px 6px; /* nudge to optically center after letter-spacing */
    border-radius: var(--r-xs);
    outline: none;
    transition: box-shadow 220ms var(--ease-std), color 220ms var(--ease-std), text-shadow 220ms var(--ease-std);
    line-height: 1;
    text-shadow: 0 0 12px rgba(30, 138, 122, 0.18);
  }

  .wordmark:hover {
    color: var(--teal-bright);
    text-shadow: 0 0 18px rgba(43, 174, 152, 0.45);
  }

  .wordmark:focus-visible {
    box-shadow: var(--ring);
  }

  .divider {
    width: 40px;
    height: 1px;
    background: var(--border-subtle);
    margin: 0 auto;
    flex-shrink: 0;
  }

  .nav-list {
    display: flex;
    flex-direction: column;
    list-style: none;
    margin: 0;
    padding: 8px 0;
    flex: 1 1 auto;
  }

  .nav-btn {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 44px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-secondary);
    outline: none;
    transition: color 120ms var(--ease-std), background 120ms var(--ease-std);
  }

  .nav-btn:hover {
    color: var(--text-primary);
  }

  .nav-btn:focus-visible {
    box-shadow: inset 0 0 0 1px var(--teal);
  }

  .nav-btn--active {
    color: var(--teal);
    background: var(--teal-ghost);
  }

  .active-bar {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%) scaleY(1);
    transform-origin: top;
    width: 2px;
    height: 24px;
    background: var(--teal);
    border-radius: 0 var(--r-xs) var(--r-xs) 0;
    animation: activeBarEnter 140ms var(--ease-out) both;
  }

  @keyframes activeBarEnter {
    from { transform: translateY(-50%) scaleY(0); }
    to   { transform: translateY(-50%) scaleY(1); }
  }

  .icon-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .sidebar-bottom {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding-bottom: 12px;
    flex-shrink: 0;
  }

  .mode-dot-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    background: none;
    border: none;
    cursor: pointer;
    border-radius: 50%;
    outline: none;
    transition: box-shadow 120ms var(--ease-std);
  }

  .mode-dot-btn:hover {
    box-shadow: 0 0 0 2px var(--border-default);
  }

  .mode-dot-btn:focus-visible {
    box-shadow: var(--ring);
  }

  .mode-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
    transition: background 220ms var(--ease-std);
  }

  .mode-dot--local {
    background: var(--teal);
  }

  .mode-dot--cloud {
    background: var(--steel);
  }

  .mode-dot--hybrid {
    background: conic-gradient(var(--teal) 0deg 180deg, var(--steel) 180deg 360deg);
  }

  .session-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-secondary);
    border-radius: var(--r-sm);
    outline: none;
    transition: color 120ms var(--ease-std);
  }

  .session-btn:hover {
    color: var(--text-primary);
  }

  .session-btn:focus-visible {
    box-shadow: var(--ring);
  }

  /* C-7: hamburger button — visible only when viewport-narrow class is
     applied by the parent (mirrors @media in base.css). The button itself
     toggles sidebar-collapsed which hides the nav-list. */
  .hamburger-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 48px;
    height: 36px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-secondary);
    outline: none;
    transition: color var(--dur-fast) var(--ease-std);
  }

  .hamburger-btn:hover {
    color: var(--text-primary);
  }

  .hamburger-btn:focus-visible {
    box-shadow: inset 0 0 0 1px var(--teal);
  }

  /* When narrow + collapsed: hide nav-list to leave only the wordmark + hamburger.
     The sidebar itself still occupies its 48px column so the panels-area
     reflow stays predictable. */
  .sidebar--narrow.sidebar--collapsed .nav-list {
    display: none;
  }

  /* When narrow + collapsed: drop the bottom controls too so the surface
     reads as a pure icon strip. The user can re-expand via the hamburger. */
  .sidebar--narrow.sidebar--collapsed .sidebar-bottom {
    display: none;
  }
</style>

<!-- Global tooltip style — injected once into document -->
<svelte:head>
  <style>
    .sidebar-tooltip-global {
      position: fixed;
      transform: translateY(-50%);
      display: flex;
      align-items: center;
      gap: 8px;
      background: var(--bg-base);
      border: 1px solid var(--border-default);
      border-radius: 4px;
      padding: 4px 8px;
      font-family: var(--font-mono);
      font-size: 10px;
      color: var(--text-primary);
      white-space: nowrap;
      pointer-events: none;
      z-index: 9999;
      opacity: 0;
      transition: opacity 100ms ease;
    }
    .sidebar-tooltip-global .tip-kbd {
      font-size: 10px;
      color: var(--steel);
      background: var(--steel-ghost);
      border-radius: 2px;
      padding: 1px 4px;
    }
  </style>
</svelte:head>
