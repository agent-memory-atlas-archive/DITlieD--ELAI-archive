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
  import FallbackWindow from './FallbackWindow.svelte';
  // E-33: persistent activity ribbon + per-window mode footer.
  import ActivityRibbon from '../components/ActivityRibbon.svelte';
  import WindowFooterMode from '../components/WindowFooterMode.svelte';
  import { motion } from '../state/motion.svelte';

  interface Props {
    type: WindowType;
  }

  let { type }: Props = $props();

  // Bootstrap the reduced-motion observer once on first WindowHost mount.
  // Idempotent — re-runs are safe because motion.bootstrap() clears the
  // prior listener before re-attaching.
  onMount(() => {
    motion.bootstrap();
    return () => motion.teardown();
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
    {:else}
      <FallbackWindow {type} />
    {/if}
  </div>

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
