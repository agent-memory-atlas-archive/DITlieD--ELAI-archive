<script lang="ts">
  // E-4 Task 13: InterventionControlBar.
  //
  // Always-visible control bar surfacing the 3 intervention verbs
  // (/pause /redirect /cancel) wired to the FSM + SSB. Visible only
  // during EXECUTING / AUDITING / RESEARCHING states (the FSM states
  // where intervention is meaningful — IDLE / COMPLETE don't need
  // it, MEMORIZING is short-lived).
  //
  // Wire shape mirrors crates::orchestrator::chat_ux::InterventionVerb
  // (Pause | Redirect | Cancel). The actual FSM mutation happens via
  // existing Tauri commands (the slash command handler in
  // crates/cli/src/live_cli.rs is the producer side; this bar is the
  // dashboard surface).
  //
  // Grounding:
  //   * developers.googleblog.com (Google ADK pause/resume).
  //   * docs.ag-ui.com (intervention verbs spec).
  //   * allenpike.com/2025/post-chat-llm-ui (control bar argument).
  //
  // Svelte 5 runes only. No createEventDispatcher; callback props
  // for click handlers.

  import { fsm } from '$lib/state/fsm.svelte';
  import type { FsmState } from '$lib/api/types';

  type InterventionVerb = 'pause' | 'redirect' | 'cancel';

  interface Props {
    /** Optional override; if null the bar reads `fsm.state` directly. */
    state?: FsmState | null;
    onverb?: (verb: InterventionVerb) => void;
  }

  const { state = null, onverb }: Props = $props();

  // INTERVENABLE_STATES: per CLAUDE.md FSM section, intervention is
  // meaningful while the model is actively producing or auditing.
  // IDLE / COMPLETE are no-ops; MEMORIZING is sub-second so user-side
  // intervention is impractical (and would corrupt RAG indexing per
  // the MEMORIZING delegation rule).
  const INTERVENABLE: ReadonlySet<FsmState> = new Set<FsmState>([
    'EXECUTING',
    'AUDITING',
    'RESEARCHING',
  ]);

  let effectiveState = $derived<FsmState>(state ?? fsm.state);
  let visible = $derived(INTERVENABLE.has(effectiveState));

  async function callInvoke<T>(
    cmd: string,
    args?: Record<string, unknown>,
  ): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail =
      !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[intervention-bar] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function fire(verb: InterventionVerb) {
    // commands_e4::commit_intervention_verb — wire-dark until the
    // command lands; the producer-side path also goes through the
    // existing INTERVENE card surface as a fallback.
    await callInvoke('commit_intervention_verb', { verb });
    if (typeof onverb === 'function') onverb(verb);
  }
</script>

{#if visible}
  <nav
    class="icb"
    aria-label="intervention control bar"
    data-fsm-state={effectiveState}
  >
    <span class="icb-label mono">intervene</span>
    <button
      type="button"
      class="icb-btn icb-btn--pause"
      onclick={() => void fire('pause')}
      title="/pause — halt current step"
    >pause</button>
    <button
      type="button"
      class="icb-btn icb-btn--redirect"
      onclick={() => void fire('redirect')}
      title="/redirect — replan from here"
    >redirect</button>
    <button
      type="button"
      class="icb-btn icb-btn--cancel"
      onclick={() => void fire('cancel')}
      title="/cancel — abort task"
    >cancel</button>
  </nav>
{/if}

<style>
  .icb {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px var(--s-3, 8px);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-pill, 999px);
    background: var(--bg-surface);
    font-family: var(--font-mono);
    font-size: 10px;
    pointer-events: auto;
  }
  .icb-label {
    color: var(--text-ghost);
    text-transform: uppercase;
    letter-spacing: 0.08em;
    margin-right: 2px;
  }
  .icb-btn {
    appearance: none;
    border: 1px solid currentColor;
    background: transparent;
    padding: 2px 8px;
    border-radius: var(--r-pill, 999px);
    font-family: inherit;
    font-size: inherit;
    color: var(--text-secondary);
    cursor: pointer;
    text-transform: lowercase;
    line-height: 1.2;
    transition: color 120ms ease, background 120ms ease;
  }
  .icb-btn:hover {
    background: var(--bg-surface-hover, rgba(255, 255, 255, 0.04));
    color: var(--text-primary);
  }
  .icb-btn--pause {
    color: var(--yellow, #facc15);
  }
  .icb-btn--redirect {
    color: var(--teal, #2dd4bf);
  }
  .icb-btn--cancel {
    color: var(--red, #f87171);
  }
  .mono {
    font-family: var(--font-mono);
  }
</style>
