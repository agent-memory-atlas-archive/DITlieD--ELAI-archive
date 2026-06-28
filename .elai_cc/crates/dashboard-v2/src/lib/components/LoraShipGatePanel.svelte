<!--
  E-47 LoraShipGatePanel -- read-only gate status surface.

  cross-lang-pair: dashboard/tauri/api
  Pairs with `get_lora_ship_gate_status` in src-tauri/src/commands.rs.

  Renders:
   - current [finetune.lora] config block (enabled / ship_gate_token /
     confirm_user_understanding) from the Tauri command response.
   - gate_status badge: refused / calibration_missing / calibration_passed /
     wire_dark (when no SSB rows exist and gate is off).
   - recent lora_ship_gate_refused SSB rows table (newest first).
   - calibration probe results if present in the most recent row's
     details_json (mmlu_delta_pct + humaneval_delta_pct vs the -2.0pp floor).
   - intentionally NO Enable button -- flip requires editing
     .elai_cc/elai_cc.toml by hand (per workflow/decisions/E-47-flip-runbook.md).
   - graceful "no gate events recorded" empty state.

  Wire-dark by default: the Tauri command returns an empty envelope when
  ELAI_LORA_SHIP_GATE_ENABLE is unset. The panel renders the config block
  regardless so the operator can see the current token at a glance.

  Svelte 5 runes: $state / $derived / $effect.
-->
<script lang="ts">
  interface LoraShipGateRow {
    seq: number;
    ts: string;
    phase: string;
    agent_id: string | null;
    session_id: string | null;
    details_json: Record<string, unknown>;
  }

  interface LoraShipGateStatus {
    gate_status: string;
    gate_enabled: boolean;
    ship_gate_token: string;
    confirm_user_understanding: boolean;
    rows: LoraShipGateRow[];
  }

  let status = $state<LoraShipGateStatus | null>(null);
  let error = $state<string | null>(null);
  let tauriInvoke: (<T>(cmd: string, args?: Record<string, unknown>) => Promise<T>) | null = null;

  async function ensureInvoke(): Promise<typeof tauriInvoke> {
    if (tauriInvoke !== null) return tauriInvoke;
    const w = typeof window !== 'undefined' ? (window as unknown as Record<string, unknown>) : {};
    if (!w.__TAURI_INTERNALS__ && !w.__TAURI__) {
      return null;
    }
    try {
      const mod = await import('@tauri-apps/api/core');
      tauriInvoke = mod.invoke;
    } catch (err) {
      console.warn('[LoraShipGatePanel] @tauri-apps/api/core import failed', err);
      tauriInvoke = null;
    }
    return tauriInvoke;
  }

  async function refresh() {
    try {
      const inv = await ensureInvoke();
      if (!inv) {
        status = null;
        return;
      }
      status = await inv<LoraShipGateStatus>('get_lora_ship_gate_status');
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  $effect(() => {
    refresh();
    const id = setInterval(refresh, 15_000);
    return () => clearInterval(id);
  });

  function statusColor(s: string): string {
    if (s === 'calibration_passed') return '#1a7';
    if (s === 'calibration_missing') return '#d80';
    if (s === 'refused') return '#d33';
    return '#888';
  }

  function phaseLabel(phase: string): string {
    if (phase === 'calibration_passed') return 'PASSED';
    if (phase === 'calibration_missing') return 'MISSING';
    if (phase === 'refused') return 'REFUSED';
    return phase.toUpperCase();
  }

  function fmtTs(ts: string): string {
    try {
      return new Date(ts).toISOString().slice(0, 19).replace('T', ' ');
    } catch {
      return ts;
    }
  }

  function calibrationDetails(row: LoraShipGateRow): { mmlu: number | null; humaneval: number | null } {
    const d = row.details_json;
    const mmlu =
      typeof d['mmlu_delta_pct'] === 'number' ? d['mmlu_delta_pct'] as number : null;
    const humaneval =
      typeof d['humaneval_delta_pct'] === 'number' ? d['humaneval_delta_pct'] as number : null;
    return { mmlu, humaneval };
  }

  const FLOOR = -2.0;

  function deltaColor(v: number | null): string {
    if (v === null) return '#888';
    return v >= FLOOR ? '#1a7' : '#d33';
  }
</script>

<section class="lora-ship-gate-panel" aria-label="LoRA ship gate">
  <h3 class="panel-heading">LoRA ship gate (E-47)</h3>

  {#if error}
    <p class="panel-error">Error: {error}</p>
  {:else if !status}
    <p class="panel-empty">Loading…</p>
  {:else}
    <!-- Config block -->
    <div class="config-block">
      <div class="config-row">
        <span class="config-label">gate_enabled</span>
        <span class="config-value" style="color: {status.gate_enabled ? '#1a7' : '#888'}">
          {status.gate_enabled ? 'true' : 'false'}
        </span>
      </div>
      <div class="config-row">
        <span class="config-label">ship_gate_token</span>
        <code class="config-code">{status.ship_gate_token}</code>
      </div>
      <div class="config-row">
        <span class="config-label">confirm_user_understanding</span>
        <span class="config-value" style="color: {status.confirm_user_understanding ? '#1a7' : '#d80'}">
          {status.confirm_user_understanding ? 'true' : 'false'}
        </span>
      </div>
      <div class="config-row">
        <span class="config-label">gate_status</span>
        <span class="gate-badge" style="color: {statusColor(status.gate_status)}">
          {phaseLabel(status.gate_status)}
        </span>
      </div>
    </div>

    <p class="no-enable-note">
      No Enable button -- flip requires editing .elai_cc/elai_cc.toml by hand.
      See workflow/decisions/E-47-flip-runbook.md.
    </p>

    <!-- Recent SSB rows -->
    {#if status.rows.length === 0}
      <p class="panel-empty">
        No gate events recorded.
        {#if !status.gate_enabled}
          Set ELAI_LORA_SHIP_GATE_ENABLE=1 to emit SSB rows.
        {/if}
      </p>
    {:else}
      <table class="gate-rows-table" aria-label="Recent gate events">
        <thead>
          <tr>
            <th>seq</th>
            <th>ts</th>
            <th>phase</th>
            <th>session</th>
            <th>mmlu Δ</th>
            <th>humaneval Δ</th>
          </tr>
        </thead>
        <tbody>
          {#each status.rows as row (row.seq)}
            {@const cal = calibrationDetails(row)}
            <tr>
              <td>{row.seq}</td>
              <td class="ts-cell">{fmtTs(row.ts)}</td>
              <td>
                <span class="phase-badge" style="color: {statusColor(row.phase)}">
                  {phaseLabel(row.phase)}
                </span>
              </td>
              <td class="session-cell">{row.session_id ?? '—'}</td>
              <td style="color: {deltaColor(cal.mmlu)}">
                {cal.mmlu !== null ? `${cal.mmlu.toFixed(1)}pp` : '—'}
              </td>
              <td style="color: {deltaColor(cal.humaneval)}">
                {cal.humaneval !== null ? `${cal.humaneval.toFixed(1)}pp` : '—'}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      <p class="floor-note">Floor: {FLOOR}pp on both MMLU and HumanEval (arXiv:2405.09673).</p>
    {/if}
  {/if}
</section>

<style>
  .lora-ship-gate-panel {
    padding: 0.75rem 1rem;
    font-size: 0.85rem;
  }

  .panel-heading {
    font-size: 0.9rem;
    font-weight: 600;
    margin: 0 0 0.6rem;
  }

  .config-block {
    display: grid;
    gap: 0.25rem 0;
    margin-bottom: 0.5rem;
  }

  .config-row {
    display: flex;
    gap: 0.5rem;
    align-items: baseline;
  }

  .config-label {
    color: #aaa;
    min-width: 11rem;
    font-family: var(--font-mono);
    font-size: 0.8rem;
  }

  .config-value {
    font-family: var(--font-mono);
  }

  .config-code {
    font-family: var(--font-mono);
    background: #1e1e1e;
    padding: 0 0.25rem;
    border-radius: 2px;
    font-size: 0.78rem;
    word-break: break-all;
  }

  .gate-badge,
  .phase-badge {
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 0.8rem;
  }

  .no-enable-note {
    color: #777;
    font-size: 0.75rem;
    margin: 0.25rem 0 0.75rem;
  }

  .panel-empty {
    color: #777;
    font-style: italic;
    margin: 0.5rem 0;
  }

  .panel-error {
    color: #d33;
    margin: 0.5rem 0;
  }

  .gate-rows-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.78rem;
    margin-top: 0.5rem;
  }

  .gate-rows-table th {
    text-align: left;
    color: #aaa;
    border-bottom: 1px solid #333;
    padding: 0.2rem 0.4rem;
  }

  .gate-rows-table td {
    padding: 0.2rem 0.4rem;
    border-bottom: 1px solid #222;
    vertical-align: middle;
  }

  .ts-cell {
    font-family: var(--font-mono);
    white-space: nowrap;
    color: #888;
  }

  .session-cell {
    font-family: var(--font-mono);
    max-width: 8rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .floor-note {
    color: #666;
    font-size: 0.72rem;
    margin: 0.4rem 0 0;
  }
</style>
