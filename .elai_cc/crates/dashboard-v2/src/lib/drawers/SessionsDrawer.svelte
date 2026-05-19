<script lang="ts">
  // C-23 — Sessions drawer hosting the BranchTree + per-branch SessionDag
  // actions. Mounts BranchTree.svelte and wires onfork / onmerge / onclose
  // callbacks to the real fork_session_branch / merge_session_branch /
  // close_session_branch Tauri commands. Reads `sessionBranches` from the
  // orchestrationData store (the existing get_session_branches feed).
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import BranchTree from '$lib/ui/BranchTree.svelte';
  import { orchestrationData } from '$lib/state/drawers.svelte';
  import Divider from '$lib/ui/Divider.svelte';

  onMount(() => {
    orchestrationData.bootstrap();
    return orchestrationData.teardown;
  });

  const branches = $derived(orchestrationData.sessionBranches);

  // C-23: per-branch action status (last command outcome). Keyed by
  // branch id so the drawer can render a per-row "merge requested"
  // banner. Cleared on successful refresh.
  let lastStatus = $state<string | null>(null);
  let lastError = $state<string | null>(null);

  async function refresh() {
    try {
      await orchestrationData.bootstrap();
    } catch (err) {
      console.debug('[SessionsDrawer] refresh failed', err);
    }
  }

  async function handleFork() {
    lastError = null;
    lastStatus = null;
    // Per Q2: the Tauri command param is `name: String`; the dashboard
    // generates a short timestamp-based default when the user has not
    // entered a name in BranchTree. BranchTree's onfork prop carries no
    // arg, so we synthesise the name here.
    const ts = new Date().toISOString().replace(/[-:T.]/g, '').slice(0, 14);
    const name = `branch-${ts}`;
    try {
      const result = await invoke('fork_session_branch', { name });
      lastStatus = `Forked ${name}: ${JSON.stringify(result)}`;
      await refresh();
    } catch (err) {
      lastError = `Fork failed: ${err}`;
    }
  }

  async function handleMerge(id: string) {
    lastError = null;
    lastStatus = null;
    // BranchTree passes the branch row id, which is the SSB-derived
    // identifier. For the get_session_branches feed the row id has the
    // shape "handoff-{seq}-winner" / "handoff-{seq}-abandoned-{name}".
    // For active branches (no SSB row yet) the label IS the branch name.
    // Pass the label so the orchestrator's merge_branch lookup matches.
    const branch = branches.find((b) => b.id === id);
    const name = branch?.label ?? id;
    try {
      const result = await invoke('merge_session_branch', { name });
      lastStatus = `Merged ${name}: ${JSON.stringify(result)}`;
      await refresh();
    } catch (err) {
      lastError = `Merge failed: ${err}`;
    }
  }

  async function handleClose(id: string) {
    lastError = null;
    lastStatus = null;
    const branch = branches.find((b) => b.id === id);
    const name = branch?.label ?? id;
    try {
      const result = await invoke('close_session_branch', { name });
      lastStatus = `Closed ${name}: ${JSON.stringify(result)}`;
      await refresh();
    } catch (err) {
      lastError = `Close failed: ${err}`;
    }
  }
</script>

<div class="drawer">
  <section class="section">
    <h3 class="section-header">SESSION BRANCHES</h3>
    <p class="section-subtitle">
      Active fork / merge / close operations against the SessionDag for the
      currently-resolved session (latest under
      <code>.elai_cc/sessions/&lt;session-id&gt;/dag.db</code>).
    </p>
    <BranchTree
      {branches}
      onfork={handleFork}
      onmerge={handleMerge}
      onclose={handleClose}
    />
    {#if lastStatus}
      <div class="status-line status-ok">{lastStatus}</div>
    {/if}
    {#if lastError}
      <div class="status-line status-err">{lastError}</div>
    {/if}
  </section>

  <Divider />

  <section class="section">
    <h3 class="section-header">HOW IT WORKS</h3>
    <ul class="note">
      <li>
        Each branch op emits a typed SSB row
        (<code>branch_forked</code> / <code>branch_merged</code> /
        <code>branch_closed</code>) — visible in the ActivityWindow feed.
      </li>
      <li>
        Per-call DAG open: the dashboard does not hold a long-lived
        SessionDag handle. Each command opens
        <code>dag.db</code> for the active session, performs the op, and
        closes the handle.
      </li>
      <li>
        <code>/continue</code> in CommandWindow now calls
        <code>load_handoff</code> (Tauri) — the textarea-prepopulate
        fallback only fires if the command fails.
      </li>
    </ul>
  </section>
</div>

<style>
  .drawer {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 12px 14px;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-header {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin: 0;
  }

  .section-subtitle {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    line-height: 1.5;
    margin: 0;
  }

  .status-line {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 4px 8px;
    border-radius: var(--r-xs);
    border: 1px solid var(--border-subtle);
  }

  .status-ok {
    color: var(--teal);
    background: var(--bg-hover);
  }

  .status-err {
    color: var(--fail);
    background: var(--bg-hover);
  }

  .note {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    margin: 0;
    padding-left: 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  code {
    color: var(--text-secondary);
  }
</style>
