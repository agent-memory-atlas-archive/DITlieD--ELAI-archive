<script lang="ts">
  import { X } from '@lucide/svelte';
  import DrawerHost from '../drawers/DrawerHost.svelte';
  import type { DrawerKey } from '../api/types';
  import { workspace } from '../state/workspace.svelte';

  interface Props {
    drawerKey: DrawerKey;
    onClose: () => void;
  }

  let { drawerKey, onClose }: Props = $props();

  const DRAWER_LABELS: Record<DrawerKey, string> = {
    'knowledge':            'KNOWLEDGE',
    'reasoning':            'REASONING',
    'orchestration':        'ORCHESTRATION',
    'rules':                'RULES',
    'research':             'RESEARCH',
    'providers':            'PROVIDERS',
    'project-intelligence': 'PROJECT INTELLIGENCE',
    'anti-staleness':       'ANTI-STALENESS',
    'tools-hooks':          'TOOLS & HOOKS',
    'web-quality':          'WEB QUALITY',
  };

  function handleBackdropClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains('drawer-backdrop')) {
      workspace.closeDrawer();
      onClose();
    }
  }

  function closeDrawer() {
    workspace.closeDrawer();
    onClose();
  }
</script>

<div
  class="drawer-backdrop"
  role="presentation"
  onclick={handleBackdropClick}
>
  <div
    class="drawer"
    role="dialog"
    aria-modal="true"
    aria-label="{DRAWER_LABELS[drawerKey]} drawer"
  >
    <div class="drawer-header">
      <span class="drawer-title">{DRAWER_LABELS[drawerKey]}</span>
      <button
        class="drawer-close"
        onclick={closeDrawer}
        aria-label="Close {DRAWER_LABELS[drawerKey]} drawer"
      >
        <X size={14} stroke-width={1.5} />
      </button>
    </div>

    <div class="drawer-body">
      <DrawerHost {drawerKey} />
    </div>
  </div>
</div>

<style>
  .drawer-backdrop {
    position: absolute;
    inset: 0;
    background: rgba(7, 9, 11, 0.10);
    pointer-events: auto;
    display: flex;
    justify-content: flex-end;
  }

  .drawer {
    display: flex;
    flex-direction: column;
    width: 360px;
    height: 100%;
    background: var(--bg-surface);
    border-left: 1px solid var(--border-default);
    pointer-events: auto;
    animation: drawerSlideIn 220ms var(--ease-out) both;
  }

  @keyframes drawerSlideIn {
    from { transform: translateX(100%); }
    to   { transform: translateX(0); }
  }

  .drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 40px;
    padding: 0 16px;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .drawer-title {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .drawer-close {
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
    transition: color 120ms var(--ease-std), background 120ms var(--ease-std);
  }

  .drawer-close:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .drawer-close:focus-visible {
    box-shadow: var(--ring);
  }

  .drawer-body {
    display: flex;
    flex-direction: column;
    flex: 1 1 auto;
    padding: 16px;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--border-default) transparent;
  }

</style>
