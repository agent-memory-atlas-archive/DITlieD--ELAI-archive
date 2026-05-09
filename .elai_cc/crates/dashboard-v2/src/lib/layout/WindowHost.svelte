<script lang="ts">
  import type { WindowType } from '../api/types';

  import CommandWindow from '../windows/CommandWindow.svelte';
  import ExploreWindow from '../windows/ExploreWindow.svelte';
  import ActivityWindow from '../windows/ActivityWindow.svelte';
  import PlanWindow from '../windows/PlanWindow.svelte';
  import ReviewWindow from '../windows/ReviewWindow.svelte';
  import MonitorWindow from '../windows/MonitorWindow.svelte';
  import SettingsWindow from '../windows/SettingsWindow.svelte';
  import FallbackWindow from './FallbackWindow.svelte';

  interface Props {
    type: WindowType;
  }

  let { type }: Props = $props();
</script>

<div class="window-host-container">
  {#if type === 'command'}
    <CommandWindow />
  {:else if type === 'explore'}
    <ExploreWindow />
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
