<script lang="ts">
  // G-5 / Slice A -- CockpitWindow (reviewer-cockpit Diff-Surface-to-Review).
  //
  // Read-only view of recent review_surface_scored SSB rows via the
  // `get_review_surface` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_g5.rs).
  //
  // Each row records ONE reviewer-cockpit DSR score emitted by the
  // `elai review --cockpit` producer: the DSR ratio (un-vouched changed lines /
  // total changed lines, recomputed byte-identically from the pinned git-diff +
  // the seq-ASC SSB stream, GD-2), the vouched / un-vouched / total changed-line
  // counts, and the pinned diff-base commit. The window renders the headline DSR
  // + a vouched/un-vouched band so a reviewer skims the VOUCHED surface and
  // focuses attention on the un-vouched fraction.
  //
  // SILENT-BY-DEFAULT (DD7): the DSR + counts are the deterministic surface; no
  // LLM-opinion text is rendered. NO measured review-time number appears
  // anywhere (GD-3/GD-5: the time-to-first-approve study is a deferred research
  // protocol, NOT a claimed result).
  //
  // The producer is DEFAULT-ON; the only opt-out is the kill-switch
  // ELAI_REVIEW_COCKPIT_DISABLE. On a fresh install with no scored review the
  // feed is empty. Polls on a 5s cadence (mirrors the G-2 RraViewerPanel).
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { ReviewSurface } from '../api/types';

  let rows = $state<ReviewSurface[] | null>(null);
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
      console.warn(`[cockpit] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<ReviewSurface[]>('get_review_surface');
    if (out === undefined) {
      rows = null;
      loadError = null;
    } else {
      rows = out;
      loadError = null;
    }
    loaded = true;
  }

  function fmtTs(ms: number | null | undefined): string {
    if (!ms) return '—';
    try {
      return new Date(ms).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return '—';
    }
  }

  function shortHash(h: string | null | undefined): string {
    if (!h) return '—';
    return h.length > 12 ? `${h.slice(0, 12)}…` : h;
  }

  function dsrPct(dsr: number): string {
    const clamped = Math.max(0, Math.min(1, dsr));
    return `${(clamped * 100).toFixed(1)}%`;
  }

  // DSR severity band: lower is better (less of the diff needs full attention).
  // These are RENDER thresholds only — NOT a measured guarantee.
  function dsrClass(dsr: number): string {
    if (dsr <= 0.25) return 'dsr dsr-low';
    if (dsr <= 0.6) return 'dsr dsr-mid';
    return 'dsr dsr-high';
  }

  // Vouched-fraction width for the band (the inverse of the DSR over hunks).
  function vouchedPct(row: ReviewSurface): number {
    const total = row.vouched_hunks + row.unvouched_hunks;
    if (total <= 0) return 0;
    return Math.round((row.vouched_hunks / total) * 100);
  }

  // G-5 honesty fix: a git error must NEVER render as a clean DSR=0. A row whose
  // diff was unavailable shows an explicit banner / "n/a" instead of a green 0.0.
  // `diff_available` is serde(default = true), so pre-fix rows (which only ever
  // existed on a successful diff path) keep rendering normally.
  function diffUnavailable(row: ReviewSurface | null): boolean {
    return row != null && row.diff_available === false;
  }

  let total = $derived(rows ? rows.length : 0);
  let latest = $derived(rows && rows.length > 0 ? rows[0] : null);

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

<section class="cockpit-panel">
  <header class="panel-header">
    <h2>Reviewer Cockpit — Diff-Surface-to-Review (G-5)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading review surfaces…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load review surfaces.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows === null}
    <p class="muted">
      Tauri IPC unavailable (running in browser preview mode). Mount the
      dashboard in the Tauri host to see live review-surface data.
    </p>
  {:else}
    {#if latest && diffUnavailable(latest)}
      <div class="diff-unavailable-banner" role="alert">
        <strong>Diff unavailable — DSR not computed.</strong>
        <span class="banner-detail"
          >The working-tree diff could not be obtained (git error). This is NOT a
          clean review.</span
        >
        {#if latest.diff_unavailable_reason}
          <pre class="banner-reason">{latest.diff_unavailable_reason}</pre>
        {/if}
      </div>
    {:else if latest}
      <div class="headline">
        <span class="headline-label">Latest DSR</span>
        <span class={dsrClass(latest.dsr)} title="Diff-Surface-to-Review ratio"
          >{dsrPct(latest.dsr)}</span
        >
        <span class="headline-detail">
          {latest.attention_lines} of {latest.total_changed_lines} changed lines need full attention
        </span>
        <div class="band" aria-label="vouched vs un-vouched hunks">
          <div class="band-vouched" style={`width:${vouchedPct(latest)}%`}></div>
          <div class="band-attn" style={`width:${100 - vouchedPct(latest)}%`}></div>
        </div>
        <span class="band-legend">
          <span class="dot dot-vouched"></span>{latest.vouched_hunks} vouched
          <span class="dot dot-attn"></span>{latest.unvouched_hunks} need attention
        </span>
      </div>
    {/if}

    <div class="summary-row">
      <span class="summary-label">Reviews scored:</span>
      <span class="summary-value">{total}</span>
    </div>

    {#if rows.length === 0}
      <p class="muted">
        No <code>review_surface_scored</code> rows recorded. A row emits when
        <code>elai review --cockpit</code> scores the working-tree diff: the DSR
        (un-vouched changed lines / total changed lines) recomputed
        byte-identically from the pinned git-diff + the seq-ASC SSB vouch stream.
        A hunk is vouched by a passing F-4 test (stdout-hash bound) or a clean
        write-gate state; a fired gate / failing test surfaces a finding. The
        producer is default-ON (kill-switch <code>ELAI_REVIEW_COCKPIT_DISABLE</code>
        only).
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Session</th>
            <th class="num">DSR</th>
            <th class="num">Attn / Total</th>
            <th class="num">Vouched / Un-vouched</th>
            <th>Diff base</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.seq)}
            <tr>
              <td>{fmtTs(row.ts_ms)}</td>
              <td class="session-cell" title={row.session_id}>{row.session_id}</td>
              {#if diffUnavailable(row)}
                <td class="num"
                  ><span class="dsr-unavailable" title={row.diff_unavailable_reason}
                    >n/a</span
                  ></td
                >
                <td class="num" colspan="2" title={row.diff_unavailable_reason}
                  >diff unavailable (git error)</td
                >
              {:else}
                <td class="num"
                  ><span class={dsrClass(row.dsr)}>{dsrPct(row.dsr)}</span></td
                >
                <td class="num">{row.attention_lines} / {row.total_changed_lines}</td>
                <td class="num">{row.vouched_hunks} / {row.unvouched_hunks}</td>
              {/if}
              <td class="base-cell" title={row.base_commit}>{shortHash(row.base_commit)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="footnote">
        DSR is the fraction of changed lines NOT covered by a deterministic vouch
        (a passing test or clean write-gate). Vouching is hunk/file-level, not
        per-line. The DSR recomputes byte-identically from the pinned git-diff +
        the SSB stream. The time-to-first-approve study (cockpit vs plain diff) is
        a deferred research protocol — no measured review-time number is claimed.
      </p>
    {/if}
  {/if}
</section>

<style>
  .cockpit-panel {
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
    font-size: 11px;
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

  .headline {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    background: var(--bg-hover);
  }

  .headline-label {
    color: var(--text-secondary);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .headline-detail {
    color: var(--text-secondary);
    font-size: 11px;
  }

  .dsr {
    display: inline-block;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    font-size: 13px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .dsr-low {
    background: rgba(110, 180, 140, 0.15);
    border: 1px solid rgba(110, 180, 140, 0.4);
    color: rgb(110, 180, 140);
  }

  .dsr-mid {
    background: rgba(200, 170, 90, 0.12);
    border: 1px solid rgba(200, 170, 90, 0.4);
    color: rgb(200, 170, 90);
  }

  .dsr-high {
    background: rgba(220, 80, 80, 0.15);
    border: 1px solid rgba(220, 80, 80, 0.5);
    color: rgb(220, 80, 80);
  }

  .band {
    display: flex;
    height: 8px;
    border-radius: var(--r-xs);
    overflow: hidden;
    border: 1px solid var(--border-subtle);
  }

  .band-vouched {
    background: rgba(110, 180, 140, 0.55);
  }

  .band-attn {
    background: rgba(220, 80, 80, 0.45);
  }

  .band-legend {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 10px;
    color: var(--text-secondary);
  }

  .dot {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    margin-left: 6px;
  }

  .dot-vouched {
    background: rgb(110, 180, 140);
  }

  .dot-attn {
    background: rgb(220, 80, 80);
  }

  .summary-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .summary-label {
    color: var(--text-secondary);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .summary-value {
    font-weight: 600;
    font-size: 13px;
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
    vertical-align: top;
  }

  th.num,
  td.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  th {
    font-weight: 500;
    color: var(--text-secondary);
    letter-spacing: 0.04em;
    text-transform: uppercase;
    font-size: 10px;
  }

  .session-cell,
  .base-cell {
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .muted {
    color: var(--text-secondary);
  }

  .footnote {
    color: var(--text-secondary);
    font-size: 11px;
  }

  .error-banner {
    background: rgba(220, 80, 80, 0.1);
    border: 1px solid rgba(220, 80, 80, 0.4);
    padding: 8px;
    border-radius: var(--r-sm);
  }

  /* G-5 honesty fix: a git error renders as an explicit amber banner, never a
     green DSR=0. Distinct from .error-banner (an IPC/load failure). */
  .diff-unavailable-banner {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: rgba(200, 170, 90, 0.12);
    border: 1px solid rgba(200, 170, 90, 0.5);
    padding: 8px;
    border-radius: var(--r-sm);
    color: rgb(200, 170, 90);
  }

  .banner-detail {
    color: var(--text-secondary);
    font-size: 11px;
  }

  .banner-reason {
    margin: 0;
    font-size: 11px;
    white-space: pre-wrap;
    word-break: break-word;
    color: var(--text-secondary);
  }

  .dsr-unavailable {
    display: inline-block;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    font-size: 13px;
    font-weight: 600;
    background: rgba(200, 170, 90, 0.12);
    border: 1px solid rgba(200, 170, 90, 0.5);
    color: rgb(200, 170, 90);
  }
</style>
