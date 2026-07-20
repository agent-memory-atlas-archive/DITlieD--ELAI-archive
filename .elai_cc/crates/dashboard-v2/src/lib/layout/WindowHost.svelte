<script lang="ts">
  import { onMount } from 'svelte';
  import type { WindowType } from '../api/types';

  import CommandWindow from '../windows/CommandWindow.svelte';
  import ActivityWindow from '../windows/ActivityWindow.svelte';
  import PlanWindow from '../windows/PlanWindow.svelte';
  import ReviewWindow from '../windows/ReviewWindow.svelte';
  import MonitorWindow from '../windows/MonitorWindow.svelte';
  import SettingsWindow from '../windows/SettingsWindow.svelte';
  import GraphWindow from '../windows/GraphWindow.svelte';
  // B-40: Idea-Framer pre-decomposition surface (FRAMING substate).
  import IdeaFramingWindow from '../windows/IdeaFramingWindow.svelte';
  // B-41: per-level auditor timeline window.
  import AuditTimelineWindow from '../windows/AuditTimelineWindow.svelte';
  // C-7: Q&A oracle / codebase-explorer surface.
  import ExploreWindow from '../windows/ExploreWindow.svelte';
  // C-40: GitHub PR adapter window (gh CLI shell-out).
  import GithubWindow from '../windows/GithubWindow.svelte';
  // E-40: orchestrator-rooted symbol-graph operator window.
  import SymbolGraphWindow from '../windows/SymbolGraphWindow.svelte';
  // G-5: reviewer-cockpit Diff-Surface-to-Review (DSR) window.
  import CockpitWindow from '../windows/CockpitWindow.svelte';
  // H-10: sigma/WebGL codebase canvas — code graph + dead-code islands +
  // PageRank hub overlay + semantic zoom + LOD.
  import CanvasWindow from '../windows/CanvasWindow.svelte';
  // J-29 SLICE-1: usage rollup window (J-28 usage_ledger consumer).
  import UsageWindow from '../windows/UsageWindow.svelte';
  import FallbackWindow from './FallbackWindow.svelte';
  // E-33: persistent activity ribbon + per-window mode footer.
  import ActivityRibbon from '../components/ActivityRibbon.svelte';
  import WindowFooterMode from '../components/WindowFooterMode.svelte';
  import SnackBarPermission from '../components/SnackBarPermission.svelte';
  import { motion } from '../state/motion.svelte';
  import { grc } from '../state/grc.svelte';
  import { permissionPrompts } from '../state/permission_prompts.svelte';
  import { uxPrefs } from '../state/ux_prefs.svelte';

  interface Props {
    type: WindowType;
  }

  let { type }: Props = $props();

  // Bootstrap the reduced-motion observer + GRC confidence store +
  // permission-prompt store once on first WindowHost mount. Each
  // bootstrap is idempotent so re-mounts (window switches) are safe.
  onMount(() => {
    motion.bootstrap();
    // UX prefs must bootstrap AFTER motion so the persisted motion override
    // applies through motion.setUserOverride before the first render tick.
    uxPrefs.bootstrap();
    grc.bootstrap();
    permissionPrompts.bootstrap();
    return () => {
      motion.teardown();
      grc.teardown();
      permissionPrompts.teardown();
    };
  });

  // Human-readable scope label for the per-window footer.
  const SCOPE_LABEL: Record<string, string> = {
    command: 'command',
    activity: 'activity',
    plan: 'plan',
    review: 'review',
    monitor: 'monitor',
    settings: 'settings',
    graph: 'graph',
    idea_framing: 'idea-framing',
    audit_timeline: 'audit-timeline',
    explore: 'explore',
    github: 'github',
    symbol_graph: 'symbol-graph',
    cockpit: 'cockpit',
    canvas: 'canvas',
    usage: 'usage',
  };
  let scope = $derived(SCOPE_LABEL[type] ?? String(type));
</script>

<div class="window-host-container">
  <!-- E-33: persistent ribbon — survives all window switches. -->
  <ActivityRibbon />

  <div class="window-host-body">
    {#if type === 'command'}
      <CommandWindow />
    {:else if type === 'activity'}
      <ActivityWindow />
    {:else if type === 'plan'}
      <PlanWindow />
    {:else if type === 'review'}
      <ReviewWindow />
    {:else if type === 'monitor'}
      <MonitorWindow />
    {:else if type === 'settings'}
      <SettingsWindow />
    {:else if type === 'graph'}
      <GraphWindow />
    {:else if type === 'idea_framing'}
      <IdeaFramingWindow />
    {:else if type === 'audit_timeline'}
      <AuditTimelineWindow />
    {:else if type === 'explore'}
      <ExploreWindow />
    {:else if type === 'github'}
      <GithubWindow />
    {:else if type === 'symbol_graph'}
      <SymbolGraphWindow />
    {:else if type === 'cockpit'}
      <CockpitWindow />
    {:else if type === 'canvas'}
      <CanvasWindow />
    {:else if type === 'usage'}
      <UsageWindow />
    {:else}
      <FallbackWindow {type} />
    {/if}
  </div>

  <!-- E-33: snack-bar permission prompts. Hidden when prompts list is
       empty (the component itself renders nothing in that case), so the
       mount is cheap. Buttons route through permissionPrompts.* helpers
       which deduplicate per id. -->
  <SnackBarPermission
    prompts={permissionPrompts.prompts}
    onApprove={(id) => permissionPrompts.approve(id)}
    onReject={(id) => permissionPrompts.reject(id)}
    onAlwaysAllow={(id) => permissionPrompts.alwaysAllow(id)}
  />

  <!-- E-33: per-window mode footer — distinct from app-level StatusBar. -->
  <WindowFooterMode {scope} />
</div>

<style>
  .window-host-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
  }
  .window-host-body {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>
