<script lang="ts">
  import { onMount } from 'svelte';
  // Redesign 2026-06-14: the workflow shell is now the 4-question-tab IA
  // (WorkflowShell). It re-homes the existing windows through WindowHost and
  // keeps every store + Tauri command working. The legacy AppShell + its
  // Sidebar/Panel/PanelDivider/Rail/StatusBar chrome are no longer mounted
  // here (they remain in-tree for the panel internals + later token pass).
  import WorkflowShell from '../layout/shell/WorkflowShell.svelte';
  import Drawer from '../layout/Drawer.svelte';
  import CommandPalette from '../layout/CommandPalette.svelte';
  // C-7: closed-tab recovery surface — rendered via the WorkflowShell overlay
  // snippet (the new shell plumbs the same overlays the legacy AppShell did).
  import RecoveryModal from '../components/RecoveryModal.svelte';
  import RemoteSidecarBootOverlay from './RemoteSidecarBootOverlay.svelte';
  import { workspace } from '../state/workspace.svelte';
  import { session } from '../state/session.svelte';
  import { fsm } from '../state/fsm.svelte';
  import { plan } from '../state/plan.svelte';
  import { activity } from '../state/activity.svelte';
  import NotificationManager from '../notifications/NotificationManager.svelte';

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
    // H-12: restore the canvas-as-default flag from localStorage. Must run AFTER
    // tryLoadFromStorage so the workspace layout is already set before we
    // conditionally override the primary panel. If canvasAsDefault is true and
    // the user has not customized the layout (userCustomized=false after a fresh
    // boot), setCanvasAsDefault(true) will apply canvas as the primary panel —
    // matching the W-2 acceptance criterion "canvas-as-default active at boot".
    workspace.loadCanvasDefault();
    // Guard: only apply the canvas-as-default override if no canvas tab already
    // exists in the primary panel. Without this guard, every boot where
    // canvasAsDefault && !userCustomized would call setCanvasAsDefault(true) and
    // replace primary with a NEW makePanel('canvas'), discarding the stored tab id.
    // Idempotent across restarts: if the tab is already there (from a prior stored
    // layout), skip the override entirely.
    const primaryAlreadyHasCanvas = workspace.primary.tabs.some(
      (t) => t.type === 'canvas',
    );
    if (workspace.canvasAsDefault && !workspace.userCustomized && !primaryAlreadyHasCanvas) {
      workspace.setCanvasAsDefault(true);
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

    // B-26 T6: auto-collapse the secondary panel below 800px viewport.
    // workspace.setSecondaryCollapsed is the canonical API (workspace.svelte.ts:622);
    // base.css:245 @media (max-width:799px) targets .secondary-collapsed.
    // Mirror Sidebar.svelte:72 resize-listener shape with passive + cleanup.
    const SECONDARY_COLLAPSE_PX = 800;
    function syncSecondaryCollapse() {
      if (typeof window === 'undefined') return;
      workspace.setSecondaryCollapsed(window.innerWidth < SECONDARY_COLLAPSE_PX);
    }
    syncSecondaryCollapse();
    window.addEventListener('resize', syncSecondaryCollapse, { passive: true });

    return () => {
      window.removeEventListener('resize', syncSecondaryCollapse);
      fsm.teardown();
      plan.teardown();
      activity.teardown();
    };
  });
</script>

<WorkflowShell>
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
</WorkflowShell>

<!-- Singleton: polls FSM/system status, fires Browser Notifications on key events -->
<NotificationManager />
