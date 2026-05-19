<script lang="ts">
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

  interface Props {
    type: WindowType;
  }

  let { type }: Props = $props();
</script>

<div class="window-host-container">
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

<style>
  .window-host-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
  }
</style>
