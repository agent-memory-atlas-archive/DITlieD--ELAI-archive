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
          <span class="glyph glyph-vouched">✓</span>{latest.vouched_hunks} vouched
          <span class="glyph glyph-attn">▲</span>{latest.unvouched_hunks} need attention
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
    color: var(--read);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  /* static section label — §4 dot-prefix */
  .panel-header h2 {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    font-size: 13px;
    font-weight: 500;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--text);
  }
  .panel-header h2::before {
    content: '';
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--d-teal);
    flex-shrink: 0;
  }

  .reload-btn {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 8px;
    background: var(--raised);
    border: 1px solid var(--line);
    border-radius: var(--r-xs);
    color: var(--text);
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease-out),
      border-color var(--dur-fast) var(--ease-out);
  }

  .reload-btn:hover {
    background: var(--panel);
    border-color: var(--d-teal-soft);
  }

  /* headline — inset well (§3): a recess, hairline edge */
  .headline {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    background: var(--panel);
  }

  .headline-label {
    color: var(--dim);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .headline-detail {
    color: var(--read);
    font-size: 10px;
  }

  /* DSR chip — §6 state language: shape + teal/dim/inversion, never
     green/amber/red. low = good (teal), mid = warn (▲ dim), high = loud
     (mono-inversion). Glyph injected so it survives a monochrome squint. */
  .dsr {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    font-size: 13px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .dsr::before { font-size: 10px; font-weight: 400; }

  .dsr-low {
    background: var(--d-teal-ghost);
    border: 1px solid var(--d-teal-soft);
    color: var(--d-teal);
  }
  .dsr-low::before { content: '✓'; }

  .dsr-mid {
    background: transparent;
    border: 1px solid var(--line2);
    color: var(--read);
  }
  .dsr-mid::before { content: '▲'; color: var(--text); }

  .dsr-high {
    background: var(--text);
    border: 1px solid var(--text);
    color: var(--void);
  }
  .dsr-high::before { content: '✕'; }

  /* vouched/attention band — teal (vouched/safe) vs ghost neutral
     (needs attention). No red. Hairline divides the two. */
  .band {
    display: flex;
    height: 8px;
    border-radius: var(--r-xs);
    overflow: hidden;
    border: 1px solid var(--line);
  }

  .band-vouched {
    background: var(--d-teal-deep);
  }

  .band-attn {
    background: var(--ghost);
  }

  .band-legend {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 10px;
    color: var(--read);
  }

  .glyph {
    display: inline-block;
    margin-left: 6px;
    font-size: 10px;
    line-height: 1;
  }
  .glyph-vouched { color: var(--d-teal); }
  .glyph-attn { color: var(--text); }

  .summary-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .summary-label {
    color: var(--dim);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .summary-value {
    font-weight: 600;
    font-size: 13px;
    color: var(--text);
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th,
  td {
    text-align: left;
    padding: 4px 6px;
    border-bottom: 1px solid var(--line);
    vertical-align: top;
  }

  tbody tr { background: var(--row); }

  th.num,
  td.num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  th {
    font-weight: 500;
    color: var(--dim);
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
    color: var(--dim);
  }

  .footnote {
    color: var(--dim);
    font-size: 10px;
  }

  /* IPC/load failure — §6 mono-inversion fail card, never red */
  .error-banner {
    background: var(--panel);
    border: 1px solid var(--line2);
    border-left: 3px solid var(--text);
    padding: 8px;
    border-radius: var(--r-sm);
    color: var(--read);
  }

  /* G-5 honesty fix: a git error renders as an explicit fail banner, never a
     clean DSR=0. §6 mono-inversion language (was amber). Distinct from
     .error-banner (an IPC/load failure) only by copy. */
  .diff-unavailable-banner {
    display: flex;
    flex-direction: column;
    gap: 4px;
    background: var(--panel);
    border: 1px solid var(--line2);
    border-left: 3px solid var(--text);
    padding: 8px;
    border-radius: var(--r-sm);
    color: var(--read);
  }

  .banner-detail {
    color: var(--dim);
    font-size: 10px;
  }

  .banner-reason {
    margin: 0;
    font-size: 10px;
    white-space: pre-wrap;
    word-break: break-word;
    color: var(--dim);
  }

  /* n/a chip — §6 mono-inversion (diff couldn't be computed) */
  .dsr-unavailable {
    display: inline-block;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    font-size: 13px;
    font-weight: 600;
    background: var(--text);
    border: 1px solid var(--text);
    color: var(--void);
  }
</style>
