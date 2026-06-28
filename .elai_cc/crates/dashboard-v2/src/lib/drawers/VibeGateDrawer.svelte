<script lang="ts">
  // F-1 / A5 -- VibeGateDrawer.
  //
  // Read-only view of recent VibeGateBlocked SSB rows via the
  // `list_vibegate_blocks` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_vibegate.rs).
  //
  // Renders:
  //   * last 50 VibeGateBlocked rows across CodeVuln / Secret /
  //     DepHallucinated / Skill arms, newest first
  //   * per-row kind badge with severity color
  //   * rule_id + cwe + file_path + source_blake3 (truncated) +
  //     citation + fsm_state
  //   * "env gate inactive" hint when ELAI_VIBE_GATE_ENABLE is unset
  //
  // Wire-dark: when the env gate is off, no rows are written and this
  // drawer renders the inactive-gate hint. The Tauri command still
  // returns cleanly so the panel mounts without errors.
  //
  // Polls on a 5s cadence (mirrors IncidentsDrawer's sibling pattern).
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { VibeGateRow } from '../api/types';

  interface VibeGateBlocksResponse {
    rows: VibeGateRow[];
    env_active: boolean;
  }

  let rows = $state<VibeGateRow[]>([]);
  let envActive = $state(false);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(
    cmd: string,
    args?: Record<string, unknown>
  ): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown })
      .__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      console.warn(`[vibegate] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<VibeGateBlocksResponse>('list_vibegate_blocks', {
      limit: 50,
    });
    if (out === undefined) {
      rows = [];
      envActive = false;
      loadError = null;
    } else {
      rows = out.rows;
      envActive = out.env_active;
      loadError = null;
    }
    loaded = true;
  }

  function shortHash(s: string | null | undefined): string {
    if (!s) return '—';
    if (s.length <= 16) return s;
    return `${s.slice(0, 8)}…${s.slice(-4)}`;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function kindLabel(kind: string): string {
    switch (kind) {
      case 'CodeVuln':
        return 'CODE VULN';
      case 'Secret':
        return 'SECRET';
      case 'DepHallucinated':
        return 'DEP HALLUC';
      case 'Skill':
        return 'SKILL';
      default:
        return kind;
    }
  }

  function kindSeverity(kind: string): 'block' | 'warn' | 'info' {
    switch (kind) {
      case 'CodeVuln':
      case 'Secret':
        return 'block';
      case 'DepHallucinated':
        return 'warn';
      case 'Skill':
        return 'info';
      default:
        return 'info';
    }
  }

  onMount(() => {
    reload();
    pollHandle = setInterval(reload, 5_000);
  });

  onDestroy(() => {
    if (pollHandle) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<section class="vibegate-panel">
  <header class="panel-header">
    <h2>VibeGate (F-1)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading VibeGate rows…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load VibeGate rows.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if !envActive}
    <p class="muted">
      VibeGate env gate inactive. Set <code>ELAI_VIBE_GATE_ENABLE=1</code> to
      enable code-vuln + secret scanning. No rows are written until the gate
      is active.
    </p>
  {:else if rows.length === 0}
    <p class="muted">
      No VibeGate blocks recorded yet. Blocks are emitted by the orchestrator
      <code>InProcessHookDispatcher</code> when a code-vuln or secret pattern
      is detected during pre-tool-use gating.
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Time</th>
          <th>Kind</th>
          <th>File</th>
          <th>Rule / CWE</th>
          <th>Hash / Citation</th>
          <th>State</th>
        </tr>
      </thead>
      <tbody>
        {#each rows as row (row.seq)}
          {@const sev = kindSeverity(row.kind)}
          <tr class="row-{sev}">
            <td>{fmtTs(row.ts)}</td>
            <td><span class="badge badge-{sev}">{kindLabel(row.kind)}</span></td>
            <td class="path" title={row.file_path}>{row.file_path}</td>
            <td>
              {#if row.rule_id || row.cwe}
                <span class="rule-chip">
                  {row.rule_id ?? ''}{row.rule_id && row.cwe ? ' / ' : ''}{row.cwe ?? ''}
                </span>
              {:else}
                —
              {/if}
            </td>
            <td>
              {#if row.citation}
                <span class="citation">{row.citation}</span>
              {:else}
                <code title={row.source_blake3}>{shortHash(row.source_blake3)}</code>
              {/if}
            </td>
            <td>{row.fsm_state ?? '—'}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</section>

<style>
  .vibegate-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .panel-header h2 {
    margin: 0;
    font-size: 13px;
    font-weight: 500;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .reload-btn {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 8px;
    background: var(--bg-hover);
    border: 1px solid var(--border-default);
    border-radius: var(--r-xs);
    color: var(--text-primary);
    cursor: pointer;
  }

  .reload-btn:hover {
    background: var(--bg-surface);
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th,
  td {
    text-align: left;
    padding: 4px 6px;
    border-bottom: 1px solid var(--border-subtle);
  }

  th {
    font-weight: 500;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 10px;
  }

  .row-block {
    background: rgba(220, 80, 80, 0.07);
  }

  .row-warn {
    background: rgba(220, 180, 80, 0.05);
  }

  .badge {
    display: inline-block;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    font-size: 10px;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    background: var(--bg-hover);
    border: 1px solid var(--border-subtle);
  }

  .badge-block {
    background: rgba(220, 80, 80, 0.15);
    border-color: rgba(220, 80, 80, 0.4);
  }

  .badge-warn {
    background: rgba(220, 180, 80, 0.15);
    border-color: rgba(220, 180, 80, 0.4);
  }

  .badge-info {
    background: rgba(80, 150, 220, 0.15);
    border-color: rgba(80, 150, 220, 0.4);
  }

  .path {
    max-width: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-secondary);
  }

  .rule-chip {
    font-size: 10px;
    color: var(--text-secondary);
  }

  .citation {
    color: var(--text-secondary);
    font-size: 10px;
  }

  .muted {
    color: var(--text-secondary);
  }

  .error-banner {
    background: rgba(220, 80, 80, 0.1);
    border: 1px solid rgba(220, 80, 80, 0.4);
    padding: 8px;
    border-radius: var(--r-sm);
  }
</style>
