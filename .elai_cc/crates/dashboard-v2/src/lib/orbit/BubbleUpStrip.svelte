<script lang="ts">
  // BubbleUpStrip — horizontal event chip strip rendered beneath the orbit tree.
  // Subscribes to get_bubble_up_events with 500ms polling.
  // LIVE: empty until B-7 wires bubble_up_event SSB emit

  import { get_bubble_up_events } from '$lib/api/stub';
  import type { BubbleUpEvent, DecompLevel } from '$lib/api/types';

  // Color mapping replicates PlanWindow.svelte:bubbleColor() — do not import
  // the function directly (it lives inside a script block, not exported).
  function bubbleColor(event: BubbleUpEvent): string {
    if (event.escalated) return 'var(--fail)';
    if (event.level === 'baby-step') return 'var(--steel)';
    if (event.level === 'step')      return 'var(--steel)';
    if (event.level === 'phase')     return 'var(--warn)';
    if (event.level === 'concept')   return 'var(--text-ghost)';
    return 'var(--text-ghost)';
  }

  // Chip label from level + escalated flag
  function chipLabel(event: BubbleUpEvent): string {
    if (event.escalated) return 'Escalate';
    const map: Record<DecompLevel, string> = {
      'baby-step': 'BabyStep',
      'step':      'Step',
      'phase':     'Phase',
      'concept':   'Concept',
    };
    return map[event.level] ?? event.level;
  }

  // Internal state — self-polled
  let events = $state<BubbleUpEvent[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);

  async function fetchEvents() {
    try {
      events = await get_bubble_up_events();
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    // Initial fetch
    void fetchEvents();
    // 500ms polling
    const id = setInterval(() => void fetchEvents(), 500);
    return () => clearInterval(id);
  });

  // Sort newest first
  const sorted = $derived([...events].sort((a, b) => b.ts - a.ts));
</script>

<div class="bubble-strip" aria-label="Bubble-up event strip">
  {#if loading && events.length === 0}
    <span class="strip-empty">loading&hellip;</span>
  {:else if error}
    <span class="strip-empty strip-error">{error}</span>
  {:else if sorted.length === 0}
    <!-- LIVE: empty until B-7 wires bubble_up_event SSB emit -->
    <span class="strip-empty">no bubble-up events</span>
  {:else}
    <div class="strip-chips">
      {#each sorted as ev (ev.ts + ev.node_id)}
        <span
          class="chip"
          style:--chip-color={bubbleColor(ev)}
          title={ev.message}
          aria-label="{chipLabel(ev)}: {ev.message}"
        >
          <span class="chip-level">{chipLabel(ev)}</span>
          <span class="chip-id">{ev.node_id}</span>
        </span>
      {/each}
    </div>
  {/if}
</div>

<style>
  .bubble-strip {
    position: absolute;
    bottom: 4px;
    left: 0;
    right: 0;
    padding: var(--s-1) var(--s-4);
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 20px;
    pointer-events: none;
    z-index: 2;
  }

  .strip-empty {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    letter-spacing: 0.06em;
    pointer-events: none;
  }

  .strip-error {
    color: var(--fail);
  }

  .strip-chips {
    display: flex;
    flex-wrap: nowrap;
    gap: var(--s-2);
    overflow: hidden;
    pointer-events: none;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    border-radius: var(--r-pill);
    border: 1px solid var(--chip-color, var(--border-subtle));
    background: transparent;
    font-family: var(--font-mono);
    font-size: 9px;
    white-space: nowrap;
    pointer-events: none;
  }

  .chip-level {
    color: var(--chip-color, var(--text-secondary));
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .chip-id {
    color: var(--text-ghost);
    max-width: 64px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
