<script lang="ts">
  // D-27 GAP-2: CheckerStatusPanel.
  //
  // Compact panel showing the most recent CheckerResultEmitted activity
  // (D-11 substrate) grouped by CheckerKind. Polls
  // `get_checker_results_recent` every `pollMs` ms (default 15s; matches
  // the D-26 inline-widget cadence target).
  //
  // Surface per CheckerKind:
  //   * pass_count
  //   * fail_count
  //   * inapplicable_count
  //   * error_count
  //
  // Read-only — uses the D-11 commands_d11::get_checker_results_recent
  // command which already aggregates `counts_by_kind`. The panel
  // re-buckets the flat row list per CheckerKind so the operator sees
  // an at-a-glance health matrix.
  //
  // Used inline (NOT a drawer). Composed into ModeRoutingDashboardView
  // alongside SycophancyFlagStrip.
  //
  // Grounding: D-11 plan + commands_d11.rs KIND_TABLE (8 closed-enum
  // CheckerKind variants).

  import { onMount, onDestroy } from 'svelte';
  import type { RecentCheckerResults, CheckerResultRow } from '../api/types';

  interface Props {
    limit?: number;
    pollMs?: number;
  }
  const { limit = 20, pollMs = 15_000 }: Props = $props();

  // Closed-enum CheckerKind list (mirror of commands_d11.rs::KIND_TABLE).
  // Locks the panel column set so empty kinds still render a row.
  const CHECKER_KINDS: ReadonlyArray<string> = [
    'sympy',
    'z3',
    'compile_rust',
    'regex_match',
    'json_schema_validate',
    'numeric_range',
    'compile_py',
    'registry_check',
  ];

  type KindHistogram = {
    pass: number;
    fail: number;
    inapplicable: number;
    error: number;
  };

  let recent = $state<RecentCheckerResults | null>(null);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);
  let pollHandle: ReturnType<typeof setInterval> | null = null;

  async function callInvoke<T>(
    cmd: string,
    args?: Record<string, unknown>,
  ): Promise<T | undefined> {
    if (typeof window === 'undefined') return undefined;
    const tauriAvail = !!(window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
    if (!tauriAvail) return undefined;
    try {
      const mod = await import('@tauri-apps/api/core');
      return (await mod.invoke<T>(cmd, args)) as T;
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.warn(`[checker-status] invoke('${cmd}') failed`, e);
      loadError = msg;
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<RecentCheckerResults>('get_checker_results_recent', { limit });
    if (out !== undefined) {
      recent = out;
      loadError = null;
    }
    loaded = true;
  }

  function emptyHistogram(): KindHistogram {
    return { pass: 0, fail: 0, inapplicable: 0, error: 0 };
  }

  // Re-bucket the flat row list into per-CheckerKind histograms. Seeded
  // with the closed-enum list so kinds with zero rows still render.
  function rebucket(rows: ReadonlyArray<CheckerResultRow>): Record<string, KindHistogram> {
    const out: Record<string, KindHistogram> = {};
    for (const k of CHECKER_KINDS) {
      out[k] = emptyHistogram();
    }
    for (const r of rows) {
      const slot = out[r.checker_kind] ?? emptyHistogram();
      switch (r.result_kind) {
        case 'pass':         slot.pass         += 1; break;
        case 'fail':         slot.fail         += 1; break;
        case 'inapplicable': slot.inapplicable += 1; break;
        case 'error':        slot.error        += 1; break;
        default: /* unknown result_kind — silently ignored */ break;
      }
      out[r.checker_kind] = slot;
    }
    return out;
  }

  const histograms = $derived.by(() => {
    if (!recent) return rebucket([]);
    return rebucket(recent.rows);
  });

  const totalRows = $derived(recent?.total ?? 0);

  function healthClass(h: KindHistogram): string {
    if (h.error > 0) return 'csp-row-error';
    if (h.fail > 0)  return 'csp-row-fail';
    if (h.pass > 0)  return 'csp-row-pass';
    return 'csp-row-idle';
  }

  onMount(() => {
    reload();
    pollHandle = setInterval(reload, pollMs);
  });

  onDestroy(() => {
    if (pollHandle !== null) {
      clearInterval(pollHandle);
      pollHandle = null;
    }
  });
</script>

<div class="csp-root" aria-label="Constructive checker status">
  <div class="csp-header">
    <span class="csp-title">CHECKER STATUS</span>
    <span class="csp-count">{totalRows} rows in window</span>
  </div>

  {#if !loaded}
    <p class="csp-empty">loading checker status…</p>
  {:else if loadError !== null}
    <p class="csp-load-error" role="status">backend error: {loadError}</p>
  {:else}
    <table class="csp-table">
      <thead>
        <tr>
          <th class="csp-th-kind">checker_kind</th>
          <th class="csp-num">pass</th>
          <th class="csp-num">fail</th>
          <th class="csp-num">inapplicable</th>
          <th class="csp-num">error</th>
        </tr>
      </thead>
      <tbody>
        {#each CHECKER_KINDS as kind (kind)}
          {@const h = histograms[kind] ?? { pass: 0, fail: 0, inapplicable: 0, error: 0 }}
          <tr class={healthClass(h)}>
            <td class="csp-id">{kind}</td>
            <td class="csp-num">{h.pass}</td>
            <td class="csp-num csp-num-fail">{h.fail}</td>
            <td class="csp-num">{h.inapplicable}</td>
            <td class="csp-num csp-num-error">{h.error}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .csp-root {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px 10px;
    border: 1px solid var(--border-subtle, #2a2a2a);
    border-radius: var(--r-sm, 4px);
    background: var(--bg-input, #181818);
  }

  .csp-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .csp-title {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    color: var(--text-ghost, #888);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .csp-count {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    color: var(--text-ghost, #888);
  }

  .csp-empty {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    color: var(--text-ghost, #888);
    margin: 0;
  }

  .csp-load-error {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    color: var(--warn, #f9c44d);
    margin: 0;
  }

  .csp-table {
    width: 100%;
    border-collapse: collapse;
    font-family: var(--font-mono, monospace);
    font-size: 11px;
  }

  .csp-table th,
  .csp-table td {
    border-bottom: 1px solid var(--border-subtle, #232323);
    padding: 3px 6px;
    text-align: left;
  }

  .csp-th-kind {
    color: var(--text-muted, #888);
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.05em;
    font-weight: 500;
  }

  .csp-table th.csp-num {
    color: var(--text-muted, #888);
    font-size: 10px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    font-weight: 500;
    text-align: right;
  }

  .csp-id {
    font-family: var(--font-mono, monospace);
    font-size: 10px;
    color: var(--text-secondary, #c4c4c4);
  }

  .csp-num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .csp-num-fail {
    color: var(--fail, #dc3c3c);
  }

  .csp-num-error {
    color: var(--warn, #f9c44d);
  }

  .csp-row-error {
    background: rgba(249, 196, 77, 0.06);
  }
  .csp-row-fail {
    background: rgba(220, 60, 60, 0.06);
  }
  .csp-row-pass {
    background: rgba(74, 200, 136, 0.04);
  }
  /* idle kinds (no rows in window) stay neutral — no extra rule. */
</style>
