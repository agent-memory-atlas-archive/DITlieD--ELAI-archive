<script lang="ts">
  // ShellStatusBar.svelte — 22px statusbar reskinned to DESIGN.md role
  // tokens (brief §5 F). FSM chip + session id + execution mode + a quiet
  // note. Reads the EXISTING fsm + session stores (no new wiring). State
  // language: silence means safe; the EXECUTING accent line is the only
  // moving thing (DESIGN §6/§7).
  import { fsm } from '../../state/fsm.svelte';
  import { session } from '../../state/session.svelte';

  let fsmState = $derived(fsm.state);
  let mode = $derived(session.mode);
  let gateMode = $derived(session.gateMode);
  let sessionId = $derived(session.sessionId);
  let project = $derived(session.currentProject?.name ?? 'no project');
  let isExecuting = $derived(fsmState === 'EXECUTING');

  function copySessionId() {
    navigator.clipboard?.writeText(sessionId).catch(() => {});
  }
</script>

<footer class="statusbar" class:statusbar--exec={isExecuting}>
  <div class="cluster">
    <span class="fsm">FSM {fsmState}</span>
    <span class="sep" aria-hidden="true">·</span>
    <button class="sid" onclick={copySessionId} title="Click to copy session id">{sessionId}</button>
    <span class="sep" aria-hidden="true">·</span>
    <span class="mode">
      <span class="mode-dot mode-dot--{mode.toLowerCase()}" aria-hidden="true"></span>
      {mode.toLowerCase()}
    </span>
    <span class="sep" aria-hidden="true">·</span>
    <span class="gate" class:gate--auto={gateMode === 'AUTONOMOUS'}>gate {gateMode.toLowerCase()}</span>
    <span class="sep" aria-hidden="true">·</span>
    <span class="proj">{project}</span>
  </div>
  <span class="note">one fix per loop — flat work surface</span>

  {#if isExecuting}
    <span class="exec-line" aria-hidden="true"></span>
  {/if}
</footer>

<style>
  .statusbar {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    height: 22px;
    padding: 0 12px;
    background: var(--bg);
    border-top: 1px solid var(--line);
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.12em;
    color: var(--dim);
    box-sizing: border-box;
    overflow: hidden;
  }

  .cluster {
    display: flex;
    align-items: center;
    gap: 7px;
    min-width: 0;
    overflow: hidden;
  }

  .fsm { color: var(--d-teal-bright); }

  .sep { color: var(--ghost); user-select: none; }

  .sid {
    font-family: var(--font-mono);
    font-size: 9px;
    letter-spacing: 0.12em;
    color: var(--dim);
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease-out);
  }
  .sid:hover { color: var(--read); }
  .sid:focus-visible { box-shadow: inset 0 0 0 1px var(--d-teal); }

  .mode { display: flex; align-items: center; gap: 4px; }
  .mode-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    flex: none;
  }
  .mode-dot--local  { background: var(--d-teal); }
  .mode-dot--cloud  { background: var(--dim); }
  .mode-dot--hybrid { background: conic-gradient(var(--d-teal) 0deg 180deg, var(--dim) 180deg 360deg); }

  .gate--auto { color: var(--d-teal); }

  .proj { color: var(--dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .note { color: var(--ghost); white-space: nowrap; }

  .exec-line {
    position: absolute;
    left: 0;
    bottom: 0;
    width: 100%;
    height: 1px;
    background: var(--d-teal);
    pointer-events: none;
  }
</style>
