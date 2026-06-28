<script lang="ts">
  // BubbleUpStrip — horizontal event chip strip rendered beneath the orbit tree.
  // Subscribes to get_bubble_up_events with 500ms polling.
  // B-7 STEP 11: now consumes the typed BubbleUpTypedPayload surface
  // (from_level / to_level / action_kind / reason / failure_count / threshold).

  import { get_bubble_up_events } from '$lib/api/stub';
  import type { BubbleUpEvent } from '$lib/api/types';

  // Color by action_kind severity — escalate is fail; replan_phase /
  // replan_concept are warn; lower-level actions stay quiet (steel).
  function bubbleColor(event: BubbleUpEvent): string {
    if (event.action_kind === 'escalate') return 'var(--fail)';
    if (event.action_kind === 'replan_phase' || event.action_kind === 'replan_concept') return 'var(--warn)';
    if (event.from_level === 'BabyStep') return 'var(--steel)';
    if (event.from_level === 'Step') return 'var(--steel)';
    if (event.from_level === 'Phase') return 'var(--warn)';
    return 'var(--text-ghost)';
  }

  // Chip label: show the cascade direction (from → to) or action for same-level.
  function chipLabel(event: BubbleUpEvent): string {
    const actionMap: Record<string, string> = {
      'research_alternative': 'RESEARCH',
      'redecompose_step': 'REDECOMP',
      'replan_phase': 'REPLAN-P',
      'replan_concept': 'REPLAN-C',
      'diagnose_and_retry': 'RETRY',
      'escalate': 'ESCALATE',
    };
    return actionMap[event.action_kind] ?? event.action_kind.toUpperCase();
  }

  // Short level label for the cascade direction.
  function shortLevel(level: string): string {
    const map: Record<string, string> = {
      'BabyStep': 'BS',
      'Step': 'S',
      'Phase': 'P',
      'Concept': 'C',
    };
    return map[level] ?? level.slice(0, 2).toUpperCase();
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

  // Stable index-based ordering (payload no longer carries a timestamp).
  const sorted = $derived([...events]);
</script>

<div class="bubble-strip" aria-label="Bubble-up event strip">
  {#if loading && events.length === 0}
    <span class="strip-empty">loading&hellip;</span>
  {:else if error}
    <span class="strip-empty strip-error">{error}</span>
  {:else if sorted.length === 0}
    <span class="strip-empty">no bubble-up events</span>
  {:else}
    <div class="strip-chips">
      {#each sorted as ev, i (i)}
        <span
          class="chip"
          style:--chip-color={bubbleColor(ev)}
          title="{chipLabel(ev)}: {ev.reason} ({ev.failure_count}/{ev.threshold})"
          aria-label="{chipLabel(ev)} {shortLevel(ev.from_level)}→{shortLevel(ev.to_level)}: {ev.reason}"
        >
          <span class="chip-level">{chipLabel(ev)}</span>
          <span class="chip-cascade">{shortLevel(ev.from_level)}→{shortLevel(ev.to_level)}</span>
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
    font-size: 10px;
    white-space: nowrap;
    pointer-events: none;
  }

  .chip-level {
    color: var(--chip-color, var(--text-secondary));
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .chip-cascade {
    color: var(--text-ghost);
  }
</style>
