<script lang="ts">
  import { X } from '@lucide/svelte';
  import { workspace } from '../state/workspace.svelte';
  import { session } from '../state/session.svelte';
  import * as stub from '../api/stub';
  import type { DrawerKey } from '../api/types';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();

  interface PaletteAction {
    id: string;
    label: string;
    kbd?: string;
    handler: () => void;
  }

  function toggleDrawer(key: DrawerKey) {
    workspace.toggleDrawer(key);
  }

  const ALL_ACTIONS: PaletteAction[] = [
    { id: 'toggle-thinking', label: 'Toggle Thinking (REFLECT pipeline)', handler: () => session.toggleThinking() },
    { id: 'cycle-effort',    label: 'Cycle Effort Level (low → medium → high → max)', handler: () => session.cycleEffort() },
    { id: 'open-command',   label: 'Open Command',   kbd: 'Ctrl+1', handler: () => workspace.setPrimaryWindow('command') },
    { id: 'open-plan',      label: 'Open Plan',      kbd: 'Ctrl+2', handler: () => workspace.setPrimaryWindow('plan') },
    { id: 'open-activity',  label: 'Open Activity',  kbd: 'Ctrl+3', handler: () => workspace.setPrimaryWindow('activity') },
    { id: 'open-review',    label: 'Open Review',    kbd: 'Ctrl+4', handler: () => workspace.setPrimaryWindow('review') },
    { id: 'open-monitor',   label: 'Open Monitor',   kbd: 'Ctrl+5', handler: () => workspace.setPrimaryWindow('monitor') },
    { id: 'open-settings',  label: 'Open Settings',  kbd: 'Ctrl+6', handler: () => workspace.setPrimaryWindow('settings') },
    { id: 'open-graph',     label: 'Open Graph Window', kbd: 'Ctrl+7', handler: () => workspace.setPrimaryWindow('graph') },
    { id: 'open-audit-timeline', label: 'Open Audit Timeline', handler: () => workspace.setPrimaryWindow('audit_timeline') },
    { id: 'drawer-knowledge',       label: 'Toggle Knowledge Drawer',            handler: () => toggleDrawer('knowledge') },
    { id: 'drawer-reasoning',       label: 'Toggle Reasoning Drawer',            handler: () => toggleDrawer('reasoning') },
    { id: 'drawer-orchestration',   label: 'Toggle Orchestration Drawer',        handler: () => toggleDrawer('orchestration') },
    { id: 'drawer-rules',           label: 'Toggle Rules Drawer',                handler: () => toggleDrawer('rules') },
    { id: 'drawer-research',        label: 'Toggle Research Drawer',             handler: () => toggleDrawer('research') },
    { id: 'drawer-providers',       label: 'Toggle Providers Drawer',            handler: () => toggleDrawer('providers') },
    { id: 'drawer-project-intel',   label: 'Toggle Project Intelligence Drawer', handler: () => toggleDrawer('project-intelligence') },
    { id: 'drawer-anti-staleness',  label: 'Toggle Anti-Staleness Drawer',       handler: () => toggleDrawer('anti-staleness') },
    { id: 'drawer-tools-hooks',     label: 'Toggle Tools & Hooks Drawer',        handler: () => toggleDrawer('tools-hooks') },
    { id: 'drawer-web-quality',     label: 'Toggle Web Quality Drawer',          handler: () => toggleDrawer('web-quality') },
    // B-42 — Cosmos artifact tier drawer.
    { id: 'drawer-cosmos',          label: 'Open Cosmos Drawer',                 handler: () => toggleDrawer('cosmos') },
    {
      id: 'toggle-secondary',
      label: 'Toggle Secondary Panel',
      kbd: 'Ctrl+D',
      handler: () => workspace.toggleSecondaryPanel(),
    },
    {
      id: 'reset-split',
      label: 'Reset Panel Split',
      handler: () => workspace.resetSplit(),
    },
    {
      id: 'mode-local',
      label: 'Switch to LOCAL mode',
      handler: () => session.setMode('LOCAL'),
    },
    {
      id: 'mode-cloud',
      label: 'Switch to CLOUD mode',
      handler: () => session.setMode('CLOUD'),
    },
    {
      id: 'mode-hybrid',
      label: 'Switch to HYBRID mode',
      handler: () => session.setMode('HYBRID'),
    },
    {
      id: 'toggle-gate',
      label: 'Toggle gate: INTERACTIVE / AUTONOMOUS',
      // patch-4G.1: also fire the Tauri set_gate_mode IPC so the backend
      // ModeChanged SSB row + AutonomousGate.set_mode actually flip. Prior
      // to this patch the toggle only updated client-side session state.
      handler: () => {
        const next = session.gateMode === 'INTERACTIVE' ? 'AUTONOMOUS' : 'INTERACTIVE';
        session.setGateMode(next);
        stub.set_gate_mode(next).catch((err) => {
          console.warn('[command-palette] set_gate_mode failed', err);
        });
      },
    },
    {
      id: 'return-selector',
      label: 'Return to project selector',
      handler: () => session.openSelector(),
    },
    // B-9: session branch operations (fork/merge/close are CLI-delegated;
    // commands exist on backend but require SessionDag in AppState — deferred).
    // Opening the command window lets the user type the equivalent CLI command.
    {
      id: 'session-fork',
      label: '/session fork  (opens command window — use CLI)',
      handler: () => workspace.setPrimaryWindow('command'),
    },
    {
      id: 'session-merge',
      label: '/session merge <id>  (opens command window — use CLI)',
      handler: () => workspace.setPrimaryWindow('command'),
    },
    {
      id: 'session-close',
      label: '/session close <id>  (opens command window — use CLI)',
      handler: () => workspace.setPrimaryWindow('command'),
    },
    {
      id: 'session-list',
      label: '/session list  (view session activity)',
      handler: () => workspace.setPrimaryWindow('activity'),
    },
  ];

  let query = $state('');
  let selectedIndex = $state(0);
  let inputEl = $state<HTMLInputElement | undefined>(undefined);

  let filteredActions = $derived(
    query.trim() === ''
      ? ALL_ACTIONS
      : ALL_ACTIONS.filter((a) => {
          const q = query.toLowerCase();
          return a.label.toLowerCase().includes(q) || (a.kbd?.toLowerCase().includes(q) ?? false);
        })
  );

  $effect(() => {
    // Reset selection when filter changes
    selectedIndex = 0;
  });

  $effect(() => {
    if (inputEl) {
      inputEl.focus();
    }
  });

  function executeAction(action: PaletteAction) {
    action.handler();
    workspace.closeCommandPalette();
    onClose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      workspace.closeCommandPalette();
      onClose();
      return;
    }
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, filteredActions.length - 1);
      scrollActiveIntoView();
      return;
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      scrollActiveIntoView();
      return;
    }
    if (e.key === 'Enter') {
      e.preventDefault();
      const action = filteredActions[selectedIndex];
      if (action) executeAction(action);
      return;
    }
  }

  let listEl = $state<HTMLElement | undefined>(undefined);

  function scrollActiveIntoView() {
    if (!listEl) return;
    const activeRow = listEl.querySelector<HTMLElement>('[data-active="true"]');
    if (activeRow) {
      activeRow.scrollIntoView({ block: 'nearest' });
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('palette-backdrop')) {
      workspace.closeCommandPalette();
      onClose();
    }
  }
</script>

<div
  class="palette-backdrop"
  role="presentation"
  onclick={handleBackdropClick}
  onkeydown={handleKeydown}
>
  <div
    class="palette"
    role="dialog"
    aria-modal="true"
    aria-label="Command palette"
  >
    <div class="palette-input-row">
      <input
        class="palette-input"
        type="text"
        placeholder="Search actions..."
        bind:value={query}
        bind:this={inputEl}
        onkeydown={handleKeydown}
        autocomplete="off"
        spellcheck={false}
        aria-label="Search actions"
        aria-controls="palette-list"
        aria-activedescendant={filteredActions[selectedIndex]
          ? `palette-item-${filteredActions[selectedIndex].id}`
          : undefined}
      />
      <button
        class="palette-close"
        onclick={() => { workspace.closeCommandPalette(); onClose(); }}
        aria-label="Close command palette"
      >
        <X size={14} stroke-width={1.5} />
      </button>
    </div>

    <div
      class="palette-list"
      id="palette-list"
      role="listbox"
      bind:this={listEl}
    >
      {#each filteredActions as action, i (action.id)}
        <button
          class="palette-item"
          class:palette-item--active={i === selectedIndex}
          id="palette-item-{action.id}"
          role="option"
          aria-selected={i === selectedIndex}
          data-active={i === selectedIndex ? 'true' : 'false'}
          onclick={() => executeAction(action)}
          onmouseenter={() => { selectedIndex = i; }}
        >
          {#if i === selectedIndex}
            <span class="item-active-bar" aria-hidden="true"></span>
          {/if}
          <span class="item-label">{action.label}</span>
          {#if action.kbd}
            <kbd class="item-kbd">{action.kbd}</kbd>
          {/if}
        </button>
      {/each}

      {#if filteredActions.length === 0}
        <div class="palette-empty">
          <span class="empty-label">No matching actions</span>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .palette-backdrop {
    position: absolute;
    inset: 0;
    background: var(--bg-overlay);
    backdrop-filter: blur(4px);
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 12px;
    pointer-events: auto;
    animation: enterFade 100ms var(--ease-out) both;
  }

  @keyframes enterFade {
    from { opacity: 0; }
    to   { opacity: 1; }
  }

  .palette {
    width: 560px;
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--r-md);
    overflow: hidden;
    animation: paletteEnter 220ms var(--ease-out) both;
    max-height: calc(100vh - 24px);
    display: flex;
    flex-direction: column;
  }

  @keyframes paletteEnter {
    from { opacity: 0; transform: translateY(4px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .palette-input-row {
    display: flex;
    align-items: center;
    border-bottom: 1px solid var(--border-subtle);
    padding: 0 12px;
    flex-shrink: 0;
  }

  .palette-input {
    flex: 1 1 auto;
    height: 44px;
    background: none;
    border: none;
    outline: none;
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text-primary);
    caret-color: var(--teal);
  }

  .palette-input::placeholder {
    color: var(--text-ghost);
  }

  .palette-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-secondary);
    border-radius: var(--r-xs);
    outline: none;
    flex-shrink: 0;
    transition: color 120ms var(--ease-std);
  }

  .palette-close:hover {
    color: var(--text-primary);
  }

  .palette-close:focus-visible {
    box-shadow: var(--ring);
  }

  .palette-list {
    overflow-y: auto;
    max-height: calc(8 * 36px);
    scrollbar-width: thin;
    scrollbar-color: var(--border-default) transparent;
  }

  .palette-item {
    position: relative;
    display: flex;
    align-items: center;
    width: 100%;
    height: 36px;
    padding: 0 16px;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    outline: none;
    gap: 8px;
    transition: background 120ms var(--ease-std);
  }

  .palette-item:hover {
    background: var(--bg-hover);
  }

  .palette-item--active {
    background: var(--teal-ghost);
  }

  .item-active-bar {
    position: absolute;
    left: 0;
    top: 8px;
    bottom: 8px;
    width: 2px;
    background: var(--teal);
    border-radius: 0 var(--r-xs) var(--r-xs) 0;
  }

  .item-label {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
    flex: 1 1 auto;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-kbd {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--steel);
    background: var(--steel-ghost);
    border-radius: var(--r-xs);
    padding: 2px 5px;
    flex-shrink: 0;
    border: none;
  }

  .palette-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 60px;
  }

  .empty-label {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
  }
</style>
