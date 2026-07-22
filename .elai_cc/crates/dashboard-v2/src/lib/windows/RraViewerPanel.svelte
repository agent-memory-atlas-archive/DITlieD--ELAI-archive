<script lang="ts">
  // G-2 / Slice A -- RraViewerPanel.
  //
  // Read-only view of recent run_artifact_emitted SSB rows via the
  // `get_run_artifact` Tauri command
  // (see crates/dashboard-v2/src-tauri/src/commands_g2.rs).
  //
  // Each row records ONE sealed Reproducible-Run Artifact (.rra): a portable
  // bundle of the SSB action stream + determinism pins + an F-gate evidence
  // manifest + a blake3 root, emitted at the `elai run` exit. The panel surfaces
  // the blake3 root, the honest per-lane guarantee level (GD-5: never
  // "reproducible_tokens" for a cloud lane), the F-gate evidence-manifest entry
  // count, the pinned ELAI_SEED, and the session id so an operator can read the
  // artifact's trust posture without opening the file.
  //
  // The producer (run_async exit seal) is DEFAULT-ON; the only opt-out is the
  // kill-switch ELAI_RRA_DISABLE. On a fresh install with no sealed run the feed
  // is empty. Polls on a 5s cadence (mirrors the G-4 / F-13 siblings).
  // cross-lang-pair: dashboard/tauri/api

  import { onMount, onDestroy } from 'svelte';
  import type { RunArtifact } from '../api/types';

  let rows = $state<RunArtifact[] | null>(null);
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
      console.warn(`[rra-viewer] invoke('${cmd}') failed`, e);
      return undefined;
    }
  }

  async function reload() {
    const out = await callInvoke<RunArtifact[]>('get_run_artifact');
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

  // Honest-scope rendering (GD-5): the guarantee level is the load-bearing
  // trust signal. "reproducible_tokens" should NEVER appear (the producer never
  // emits it); if it ever does it is rendered as a loud over-claim warning.
  function guaranteeLabel(g: string): string {
    switch (g) {
      case 'reproducible_action_stream':
        return 'action-stream reproducible';
      case 'verify_divergence':
        return 'verify-divergence (not token-reproducible)';
      case 'reproducible_tokens':
        return 'OVER-CLAIM: tokens-reproducible (unexpected)';
      default:
        return g;
    }
  }

  function guaranteeClass(g: string): string {
    switch (g) {
      case 'reproducible_action_stream':
        return 'guarantee guarantee-stream';
      case 'verify_divergence':
        return 'guarantee guarantee-divergence';
      case 'reproducible_tokens':
        return 'guarantee guarantee-overclaim';
      default:
        return 'guarantee';
    }
  }

  // J-14: tier verdict badge (mirrors CLI render_tier_text at rra_cmd.rs:326).
  // tier_effective is a snake_case TierVerdictLabel joined from the latest
  // TierEvaluated ReproPackEvent row for this session. Honest "—" when absent
  // (LEGACY receipts or verify path SSB emit still DARK).
  function tierLabel(
    t: string | null | undefined,
    downgrades: string[] | null | undefined
  ): string {
    if (t === 'downgraded' && downgrades && downgrades.length > 0) {
      return 'DOWNGRADED';
    }
    switch (t) {
      case 'action_faithful':
        return 'T0 action-faithful';
      case 'token_faithful':
        return 'T1 token-faithful';
      case 'score_faithful':
        return 'T2 score-faithful';
      case 'missing_determinism_basis':
        return 'MISSING-DETERMINISM-BASIS';
      case null:
      case undefined:
        return '—';
      default:
        return t;
    }
  }

  function tierClass(
    t: string | null | undefined,
    downgrades: string[] | null | undefined
  ): string {
    if (t === 'downgraded' && downgrades && downgrades.length > 0) {
      return 'tier tier-downgraded';
    }
    switch (t) {
      case 'action_faithful':
        return 'tier tier-t0';
      case 'token_faithful':
        return 'tier tier-t1';
      case 'score_faithful':
        return 'tier tier-t2';
      case 'missing_determinism_basis':
        return 'tier tier-missing';
      case null:
      case undefined:
        return 'tier tier-none';
      default:
        return 'tier';
    }
  }

  function tierTitle(
    t: string | null | undefined,
    downgrades: string[] | null | undefined
  ): string {
    if (t == null) {
      return 'no tier verdict row (LEGACY or verify not run)';
    }
    const dg = downgrades && downgrades.length > 0 ? downgrades.join('; ') : '';
    if (t === 'downgraded') {
      return dg ? `downgrades: ${dg}` : 'downgraded (no downgrade detail)';
    }
    return dg ? `downgrades: ${dg}` : t;
  }

  // J-14: first-divergence line (mirrors CLI rra_cmd.rs:411-419). Honest "—"
  // when divergence_position is absent (T0 action-faithful or LEGACY).
  function divergenceText(row: RunArtifact): string {
    if (row.divergence_position == null) return '—';
    const exp = row.divergence_expected ?? '';
    const got = row.divergence_actual ?? '';
    return `div @ pos ${row.divergence_position}: expected ${exp}, got ${got}`;
  }

  // H-23: RerunHashEquality badge (mirrors tier badge pattern above).
  // rerun_kind is "equal" / "divergent" joined from the latest
  // RerunHashEquality SSB row for this session_id. Honest "—" when absent.
  function rerunLabel(kind: string | null | undefined): string {
    switch (kind) {
      case 'equal':
        return 'EQUAL';
      case 'divergent':
        return 'DIVERGENT';
      case null:
      case undefined:
        return '—';
      default:
        return kind;
    }
  }

  function rerunClass(kind: string | null | undefined): string {
    switch (kind) {
      case 'equal':
        return 'rerun rerun-equal';
      case 'divergent':
        return 'rerun rerun-divergent';
      case null:
      case undefined:
        return 'rerun rerun-none';
      default:
        return 'rerun';
    }
  }

  function rerunTitle(row: RunArtifact): string {
    if (row.rerun_kind == null) {
      return 'no rerun hash equality row (not yet rerun)';
    }
    const parts: string[] = [row.rerun_kind];
    if (row.rerun_digest_a) parts.push(`a: ${shortHash(row.rerun_digest_a)}`);
    if (row.rerun_digest_b) parts.push(`b: ${shortHash(row.rerun_digest_b)}`);
    if (row.rerun_kind === 'divergent') {
      if (row.rerun_divergent_metric) parts.push(`metric: ${row.rerun_divergent_metric}`);
      if (row.rerun_first_divergence_index != null) parts.push(`idx: ${row.rerun_first_divergence_index}`);
    }
    return parts.join('; ');
  }

  let total = $derived(rows ? rows.length : 0);

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

<section class="rra-panel">
  <header class="panel-header">
    <h2>Reproducible-Run Artifacts (G-2)</h2>
    <button class="reload-btn" onclick={reload}>Reload</button>
  </header>

  {#if !loaded}
    <p class="muted">Loading reproducible-run artifacts…</p>
  {:else if loadError}
    <div class="error-banner">
      <strong>Failed to load run artifacts.</strong>
      <pre>{loadError}</pre>
    </div>
  {:else if rows === null}
    <p class="muted">
      Tauri IPC unavailable (running in browser preview mode). Mount the
      dashboard in the Tauri host to see live run-artifact data.
    </p>
  {:else}
    <div class="summary-row">
      <span class="summary-label">Artifacts sealed:</span>
      <span class="summary-value">{total}</span>
    </div>

    {#if rows.length === 0}
      <p class="muted">
        No <code>run_artifact_emitted</code> rows recorded. A row emits at the
        <code>elai run</code> exit when a Reproducible-Run Artifact
        (<code>.rra</code>) is sealed: the SSB action stream + determinism pins +
        an F-gate evidence manifest + a blake3 root, verifiable offline via
        <code>elai replay --verify</code>. The seal is default-ON (kill-switch
        <code>ELAI_RRA_DISABLE</code> only).
      </p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Session</th>
            <th>Guarantee</th>
            <th>Tier</th>
            <th>Rerun</th>
            <th class="num">Gates</th>
            <th class="num">Seed</th>
            <th>Root (blake3)</th>
            <th>Objective</th>
            <th>Divergence</th>
          </tr>
        </thead>
        <tbody>
          {#each rows as row (row.seq)}
            <tr class="row-rra">
              <td>{fmtTs(row.ts_ms)}</td>
              <td class="session-cell" title={row.session_id}>{row.session_id}</td>
              <td>
                <span
                  class={guaranteeClass(row.guarantee_level)}
                  title={row.guarantee_level}>{guaranteeLabel(row.guarantee_level)}</span
                >
              </td>
              <td class="tier-cell">
                <span
                  class={tierClass(row.tier_effective, row.downgrades)}
                  title={tierTitle(row.tier_effective, row.downgrades)}>{tierLabel(row.tier_effective, row.downgrades)}</span
                >
              </td>
              <td class="rerun-cell">
                <span
                  class={rerunClass(row.rerun_kind)}
                  title={rerunTitle(row)}>{rerunLabel(row.rerun_kind)}</span
                >
              </td>
              <td class="num">{row.gate_count}</td>
              <td class="num">{row.seed}</td>
              <td class="root-cell" title={row.root_blake3}>{shortHash(row.root_blake3)}</td>
              <td class="objective-cell">
                {#if row.objective_hash == null}
                  <span class="badge badge-legacy" title="Pre-I-45 receipt — no objective hash">LEGACY</span>
                {:else}
                  <span class="badge badge-sealed" title={row.objective_hash}>{row.objective_hash.slice(0, 8)}</span>
                {/if}
              </td>
              <td class="divergence-cell" title={divergenceText(row)}>
                {divergenceText(row)}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="footnote">
        The guarantee is action-stream reproducibility + offline F-gate
        verdict-hash re-check, NOT bit-identical token replay (cloud-lane
        <code>system_fingerprint</code> is opaque). <code>elai replay --verify
        &lt;file&gt;</code> recomputes the root (tamper check) and re-derives every
        event hash (divergence check).
      </p>
    {/if}
  {/if}
</section>

<style>
  .rra-panel {
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

  .summary-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .summary-label {
    color: var(--text-secondary);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .summary-value {
    font-weight: 600;
    font-size: 13px;
    color: rgb(110, 180, 140);
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

  .row-rra {
    background: rgba(110, 180, 140, 0.06);
  }

  .session-cell,
  .root-cell {
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .guarantee {
    display: inline-block;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    font-size: 10px;
    letter-spacing: 0.03em;
    background: var(--bg-hover);
    border: 1px solid var(--border-subtle);
  }

  .guarantee-stream {
    background: rgba(110, 180, 140, 0.15);
    border-color: rgba(110, 180, 140, 0.4);
    color: rgb(110, 180, 140);
  }

  .guarantee-divergence {
    background: rgba(200, 170, 90, 0.12);
    border-color: rgba(200, 170, 90, 0.4);
    color: rgb(200, 170, 90);
  }

  .guarantee-overclaim {
    background: rgba(220, 80, 80, 0.15);
    border-color: rgba(220, 80, 80, 0.5);
    color: rgb(220, 80, 80);
    font-weight: 600;
  }

  .muted {
    color: var(--text-secondary);
  }

  .footnote {
    color: var(--text-secondary);
    font-size: 10px;
  }

  /* I-45: objective hash badge */
  .objective-cell {
    white-space: nowrap;
  }

  .badge {
    display: inline-block;
    padding: 1px 5px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .badge-legacy {
    background: var(--border-subtle);
    color: var(--text-secondary);
    border: 1px solid var(--border-default);
  }

  .badge-sealed {
    background: var(--pass-ghost);
    color: var(--pass);
    border: 1px solid var(--pass-soft);
  }

  /* J-14: tier verdict badge + divergence cell */
  .tier-cell,
  .divergence-cell {
    white-space: nowrap;
  }

  .divergence-cell {
    color: var(--text-secondary);
    font-size: 10px;
  }

  .tier {
    display: inline-block;
    padding: 1px 5px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.03em;
    border: 1px solid var(--border-subtle);
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .tier-t0 {
    background: rgba(110, 180, 140, 0.15);
    border-color: rgba(110, 180, 140, 0.4);
    color: rgb(110, 180, 140);
  }

  .tier-t1 {
    background: rgba(110, 160, 220, 0.15);
    border-color: rgba(110, 160, 220, 0.4);
    color: rgb(110, 160, 220);
  }

  .tier-t2 {
    background: rgba(170, 130, 220, 0.15);
    border-color: rgba(170, 130, 220, 0.4);
    color: rgb(170, 130, 220);
  }

  .tier-missing {
    background: rgba(220, 80, 80, 0.15);
    border-color: rgba(220, 80, 80, 0.5);
    color: rgb(220, 80, 80);
  }

  .tier-downgraded {
    background: rgba(220, 160, 80, 0.15);
    border-color: rgba(220, 160, 80, 0.5);
    color: rgb(220, 160, 80);
  }

  .tier-none {
    color: var(--text-secondary);
    font-weight: 400;
    border-color: transparent;
    background: transparent;
  }

  /* H-23: rerun hash equality badge */
  .rerun-cell {
    white-space: nowrap;
  }

  .rerun {
    display: inline-block;
    padding: 1px 5px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.03em;
    border: 1px solid var(--border-subtle);
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .rerun-equal {
    background: rgba(110, 180, 140, 0.15);
    border-color: rgba(110, 180, 140, 0.4);
    color: rgb(110, 180, 140);
  }

  .rerun-divergent {
    background: rgba(220, 160, 80, 0.15);
    border-color: rgba(220, 160, 80, 0.5);
    color: rgb(220, 160, 80);
  }

  .rerun-none {
    color: var(--text-secondary);
    font-weight: 400;
    border-color: transparent;
    background: transparent;
  }

  .error-banner {
    background: var(--fail-ghost);
    border: 1px solid var(--fail-soft);
    padding: 8px;
    border-radius: var(--r-sm);
  }
</style>
