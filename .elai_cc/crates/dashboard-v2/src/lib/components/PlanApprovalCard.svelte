<!--
  H-37 PlanApprovalCard -- plan-approval gate decision surface.

  cross-lang-pair: dashboard/tauri/api
  Pairs with `get_plan_approval_decisions` in src-tauri/src/commands_h37.rs and
  the `PlanDecisionRow` TypeScript interface in src/lib/api/stub.ts.

  Renders:
   - the recorded plan-approval decision (Approve/Redirect/Reject/AutoAdvance)
     as a coloured badge for the most recent gate invocation
   - the risk_level, target_state, and plan_session_id fields
   - a reason field when the operator provided one (Reject/Redirect paths)
   - graceful "no plan decisions recorded yet" empty state
   - autonomous-mode note when kind == "auto_advance"

  Data path: Tauri `get_plan_approval_decisions` -> SSB plan_decision_event rows.
  Gate is wire-dark unless ELAI_PLAN_APPROVAL_GATE_ENABLE=1. When the gate is
  dark the component renders the empty state (no rows in SSB) rather than a
  special "gate disabled" message, consistent with other wire-dark panels.

  Svelte 5 runes: $state / $derived / $effect.
-->
<script lang="ts">
  import { Badge } from '$lib/ui';
  import { get_plan_approval_decisions, type PlanDecisionRow } from '$lib/api/stub';

  interface Props {
    /** Session ID to filter rows by. Pass "" to show all sessions. */
    sessionId?: string;
    /** Max rows to display (default 5). */
    limit?: number;
  }

  let { sessionId = '', limit = 5 }: Props = $props();

  let decisions = $state<PlanDecisionRow[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function loadDecisions() {
    loading = true;
    error = null;
    try {
      decisions = await get_plan_approval_decisions(sessionId || undefined, limit);
    } catch (e) {
      error = String(e);
      decisions = [];
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void loadDecisions();
  });

  function toneForDecision(kind: string): string {
    switch (kind) {
      case 'approve':
        return 'teal';
      case 'auto_advance':
        return 'muted';
      case 'redirect':
        return 'warn';
      case 'reject':
        return 'fail';
      default:
        return 'muted';
    }
  }

  function labelForDecision(kind: string): string {
    switch (kind) {
      case 'approve':
        return 'APPROVED';
      case 'auto_advance':
        return 'AUTO-ADVANCE';
      case 'redirect':
        return 'REDIRECTED';
      case 'reject':
        return 'REJECTED';
      default:
        return kind.toUpperCase();
    }
  }
</script>

<div class="plan-approval-card">
  <div class="header">
    <span class="title">Plan Approval Gate</span>
    <button class="refresh-btn" onclick={() => void loadDecisions()} aria-label="Refresh">
      {loading ? '…' : '↻'}
    </button>
  </div>

  {#if error}
    <div class="error-msg">{error}</div>
  {:else if decisions.length === 0 && !loading}
    <div class="empty-state">No plan decisions recorded yet.</div>
  {:else}
    <ul class="decision-list">
      {#each decisions as row (row.id)}
        <li class="decision-row">
          <div class="decision-header">
            <Badge tone={toneForDecision(row.decision_kind)}>
              {labelForDecision(row.decision_kind)}
            </Badge>
            <span class="risk-badge risk-{row.risk_level.toLowerCase()}">{row.risk_level}</span>
            <span class="session-id">{row.plan_session_id}</span>
          </div>
          <div class="decision-detail">
            <span class="target">target: <code>{row.target_state}</code></span>
            {#if row.decision_kind === 'auto_advance'}
              <span class="auto-note">autonomous mode -- no human prompt</span>
            {/if}
            {#if row.reason}
              <div class="reason">reason: {row.reason}</div>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .plan-approval-card {
    font-size: 0.82rem;
    padding: 0.5rem 0.75rem;
    background: var(--panel-bg, #1a1a2e);
    border-radius: 4px;
    border: 1px solid var(--panel-border, #333);
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 0.5rem;
  }

  .title {
    font-weight: 600;
    color: var(--text-muted, #aaa);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    font-size: 0.75rem;
  }

  .refresh-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--text-muted, #aaa);
    font-size: 1rem;
    line-height: 1;
    padding: 0 0.25rem;
  }

  .empty-state,
  .error-msg {
    color: var(--text-muted, #888);
    font-style: italic;
    padding: 0.25rem 0;
  }

  .error-msg {
    color: var(--color-fail, #e74c3c);
  }

  .decision-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .decision-row {
    padding: 0.35rem 0.5rem;
    background: var(--row-bg, #111124);
    border-radius: 3px;
  }

  .decision-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .risk-badge {
    font-size: 0.7rem;
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
    font-weight: 600;
  }

  .risk-low {
    background: var(--color-teal-subtle, #0d3d31);
    color: var(--color-teal, #2ecc71);
  }

  .risk-medium {
    background: var(--color-warn-subtle, #3a2e00);
    color: var(--color-warn, #f39c12);
  }

  .risk-high {
    background: var(--color-fail-subtle, #3a0000);
    color: var(--color-fail, #e74c3c);
  }

  .session-id {
    color: var(--text-muted, #666);
    font-size: 0.7rem;
    font-family: var(--font-mono);
  }

  .decision-detail {
    margin-top: 0.2rem;
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    color: var(--text-secondary, #bbb);
  }

  .target code {
    font-size: 0.75rem;
    background: var(--code-bg, #222);
    padding: 0 0.2rem;
    border-radius: 2px;
  }

  .auto-note {
    color: var(--text-muted, #777);
    font-style: italic;
    font-size: 0.72rem;
  }

  .reason {
    color: var(--text-secondary, #bbb);
    font-style: italic;
  }
</style>
