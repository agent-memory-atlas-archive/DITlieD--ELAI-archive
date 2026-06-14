<script lang="ts">
  // E-33 WindowFooterMode — per-window footer strip showing FSM state +
  // substate + the "mode I'm in" hint. Claude Code CLI / Cline / Aider all
  // add a footer mode indicator because the #1 user-error class is "what
  // mode am I in" (anyonebuilds.com plan-mode + wil.dev TUI guide). Distinct
  // from ActivityRibbon: the ribbon is the LIVE OPERATION ticker; this
  // footer is the STATIC MODE / GATE indicator.

  import { fsm } from '$lib/state/fsm.svelte';
  import { session } from '$lib/state/session.svelte';

  interface Props {
    /** Optional caller-supplied scope label, e.g. "ExploreWindow". */
    scope?: string;
  }
  let { scope = '' }: Props = $props();

  let fsmState = $derived(fsm.state);
  let agentSubstate = $derived(fsm.agentSubstate);
  let mode = $derived(session.mode);
  let gateMode = $derived(session.gateMode);
</script>

<footer class="window-footer-mode" aria-label="Window mode indicator">
  {#if scope}
    <span class="ftr-scope">{scope}</span>
    <span class="ftr-sep">·</span>
  {/if}
  <span class="ftr-cell" title="dispatch mode">
    <span class="ftr-key">mode</span>
    <span class="ftr-val">{mode.toLowerCase()}</span>
  </span>
  <span class="ftr-sep">·</span>
  <span class="ftr-cell" title="autonomous-gate state">
    <span class="ftr-key">gate</span>
    <span class="ftr-val">{gateMode.toLowerCase()}</span>
  </span>
  <span class="ftr-sep">·</span>
  <span class="ftr-cell ftr-fsm" title="FSM state / substate">
    <span class="ftr-key">fsm</span>
    <span class="ftr-val">{fsmState.toLowerCase()}{agentSubstate ? `:${agentSubstate}` : ''}</span>
  </span>
</footer>

<style>
  .window-footer-mode {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 18px;
    min-height: 18px;
    padding: 0 var(--s-3);
    border-top: 1px solid var(--line);
    background: var(--bg);
    font-family: var(--font-mono);
    font-size: 9px;
    color: var(--dim);
    user-select: none;
    flex: 0 0 auto;
  }
  .ftr-cell {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  .ftr-key {
    color: var(--dim);
  }
  .ftr-val {
    color: var(--read);
  }
  .ftr-scope {
    color: var(--read);
    font-weight: 500;
  }
  .ftr-sep {
    color: var(--ghost);
  }
</style>
