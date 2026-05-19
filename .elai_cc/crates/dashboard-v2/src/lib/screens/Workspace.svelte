<script lang="ts">
  import { onMount } from 'svelte';
  import AppShell from '../layout/AppShell.svelte';
  import Sidebar from '../layout/Sidebar.svelte';
  import Panel from '../layout/Panel.svelte';
  import PanelDivider from '../layout/PanelDivider.svelte';
  import Rail from '../layout/Rail.svelte';
  import StatusBar from '../layout/StatusBar.svelte';
  import Drawer from '../layout/Drawer.svelte';
  import CommandPalette from '../layout/CommandPalette.svelte';
  // C-7: closed-tab recovery surface — DRIFT-1 mount location is the
  // Workspace overlay snippet (not AppShell which only plumbs snippets).
  import RecoveryModal from '../components/RecoveryModal.svelte';
  import RemoteSidecarBootOverlay from './RemoteSidecarBootOverlay.svelte';
  import { workspace } from '../state/workspace.svelte';
  import { session } from '../state/session.svelte';
  import { fsm } from '../state/fsm.svelte';
  import { plan } from '../state/plan.svelte';
  import { activity } from '../state/activity.svelte';
  import type { WindowType } from '../api/types';
  import NotificationManager from '../notifications/NotificationManager.svelte';

  function handleSplitChange(px: number) {
    const container = document.querySelector<HTMLElement>('.panels-area');
    if (container) {
      const total = container.offsetWidth;
      if (total > 0) {
        workspace.setSplit(px / total);
        container.style.setProperty('--panel-split', `${px}px`);
      }
    }
  }

  function handleNewTab(panelId: 'primary' | 'secondary', type: WindowType) {
    workspace.openTab(panelId, type);
  }

  // Show the remote-sidecar boot overlay when launched with ?remote=1 or
  // when ELAI_REMOTE=1 was injected into the window object by elai.exe.
  let showRemoteOverlay = $state(
    typeof window !== 'undefined' &&
      (new URLSearchParams(window.location.search).get('remote') === '1' ||
        Boolean((window as any).__ELAI_REMOTE__))
  );

  // On mount: try to restore persisted layout from localStorage.
  // If absent/corrupt, fall through to gate-mode default.
  // INTERACTIVE -> primary=Activity, secondary=Command.
  // AUTONOMOUS  -> primary=Monitor, secondary=Activity.
  //
  // Phase B wiring: bootstrap the live state stores (FSM/plan/activity)
  // so the workspace renders real backend signals instead of the
  // mock-data seeds the stores used to inherit at import time.
  onMount(() => {
    const restored = workspace.tryLoadFromStorage();
    if (!restored) {
      workspace.applyGateModeLayout(session.gateMode);
    }
    // Enable autosave only AFTER the initial decision so we don't immediately
    // overwrite a stored snapshot with a default-layout snapshot.
    workspace.enablePersistence();

    // patch-4G.1: pull live gate mode from backend so the UI reflects any
    // server-side state (set by another client, prior session, /mode CLI
    // future plumbing, etc.). Fire-and-forget; best-effort.
    session.reconcileGateMode();

    fsm.bootstrap();
    plan.bootstrap();
    activity.bootstrap();

    // C-7: viewport-width auto-collapse for narrow viewports.
    // Below 800px the secondary panel auto-collapses (single-panel view)
    // so the primary panel takes full width. ResizeObserver watches the
    // .panels-area container — driven by viewport changes AND by sidebar-
    // hamburger toggles that change the available area.
    //
    // The observer does NOT fight user intent: once the viewport rises
    // back above the threshold we re-expand. If the user closes a tab
    // in secondary (which already collapses via the existing closeTab
    // logic), this observer doesn't interfere.
    const NARROW_BREAKPOINT_PX = 800;
    let resizeObserver: ResizeObserver | null = null;
    if (typeof ResizeObserver !== 'undefined') {
      const panelsEl = document.querySelector<HTMLElement>('.panels-area');
      if (panelsEl) {
        resizeObserver = new ResizeObserver((entries) => {
          for (const entry of entries) {
            const width = entry.contentRect.width;
            if (width < NARROW_BREAKPOINT_PX && !workspace.secondaryCollapsed) {
              workspace.setSecondaryCollapsed(true);
            } else if (
              width >= NARROW_BREAKPOINT_PX &&
              workspace.secondaryCollapsed &&
              workspace.secondary.tabs.length > 0
            ) {
              workspace.setSecondaryCollapsed(false);
            }
          }
        });
        resizeObserver.observe(panelsEl);
      }
    }

    return () => {
      fsm.teardown();
      plan.teardown();
      activity.teardown();
      if (resizeObserver) resizeObserver.disconnect();
    };
  });
</script>

<AppShell>
  {#snippet sidebar()}
    <Sidebar />
  {/snippet}

  {#snippet primaryPanel()}
    <div
      class="primary-panel-wrap"
      style={workspace.secondaryCollapsed ? 'flex: 1 1 100%' : `flex: 1 1 ${workspace.panelSplit * 100}%`}
    >
      <Panel
        panelId="primary"
        tabs={workspace.primary.tabs.map((t) => ({ id: t.id, type: t.type, title: t.label }))}
        activeTabId={workspace.primary.activeTabId}
        onTabSelect={(id) => workspace.selectTab('primary', id)}
        onTabClose={(id) => workspace.closeTab('primary', id)}
        onNewTab={handleNewTab}
      />
    </div>
  {/snippet}

  {#snippet divider()}
    {#if !workspace.secondaryCollapsed}
      <PanelDivider onSplitChange={handleSplitChange} />
    {/if}
  {/snippet}

  {#snippet secondaryPanel()}
    {#if !workspace.secondaryCollapsed}
      <div class="secondary-panel-wrap">
        <Panel
          panelId="secondary"
          tabs={workspace.secondary.tabs.map((t) => ({ id: t.id, type: t.type, title: t.label }))}
          activeTabId={workspace.secondary.activeTabId}
          onTabSelect={(id) => workspace.selectTab('secondary', id)}
          onTabClose={(id) => workspace.closeTab('secondary', id)}
          onNewTab={handleNewTab}
        />
      </div>
    {/if}
  {/snippet}

  {#snippet rail()}
    <Rail />
  {/snippet}

  {#snippet statusBar()}
    <StatusBar />
  {/snippet}

  {#snippet overlay()}
    {#if showRemoteOverlay}
      <RemoteSidecarBootOverlay
        onDone={() => { showRemoteOverlay = false; }}
        onSkip={() => { showRemoteOverlay = false; }}
      />
    {/if}
    {#if workspace.activeDrawer !== null}
      <Drawer drawerKey={workspace.activeDrawer} onClose={() => workspace.closeDrawer()} />
    {/if}
    {#if workspace.commandPaletteOpen}
      <CommandPalette onClose={() => workspace.closeCommandPalette()} />
    {/if}
    {#if workspace.showRecoveryModal}
      <RecoveryModal />
    {/if}
  {/snippet}
</AppShell>

<!-- Singleton: polls FSM/system status, fires Browser Notifications on key events -->
<NotificationManager />

<style>
  .primary-panel-wrap {
    display: flex;
    flex-direction: column;
    min-width: 320px;
    overflow: hidden;
  }

  .secondary-panel-wrap {
    display: flex;
    flex-direction: column;
    flex: 1 1 auto;
    min-width: 320px;
    overflow: hidden;
  }
</style>
