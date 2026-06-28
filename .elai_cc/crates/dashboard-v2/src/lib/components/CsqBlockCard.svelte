<script lang="ts">
  // B-5: CSQ BLOCK intervention card.
  //
  // Surfaces a RED intervention card whenever a `csq_evaluated` SSB row
  // arrives with severity == Critical AND guard_present == false — i.e. the
  // GRC engine flagged a counterfactual-safety-question (CSQ) at Critical
  // severity on a baby step that has NO "STOP-Verified:" guard token. These
  // are exactly the verdicts that gate (block) a step in the runtime, so the
  // dashboard mirrors that block visually and offers a Decline action.
  //
  // Data source: `get_csq_log` (the full verdict log, every csq_evaluated
  // row). We filter client-side rather than reusing `get_csq_pending`
  // (which already filters blocked=true) because the plan's trigger
  // condition is explicitly "severity=Critical AND guard_present=false",
  // which is the producer-side definition of blocked — filtering on the raw
  // fields keeps the card's contract self-evident.
  //
  // Decline action: there is no dashboard-originated typed `ReplanRequest`
  // Tauri command (the runtime's ReplanRequest::Major emission is deferred
  // per task_runner.rs:8735, and no FSM-mutation command is exposed to the
  // dashboard). The honest in-scope wiring is `submit_task` with a replan
  // request task string — `submit_task` writes a TaskSubmitted SSB row the
  // agent runtime consumes, which is the only FSM-affecting affordance the
  // dashboard has. See workflow/reasoning/B-5-decisions.md.

  import { onMount } from 'svelte';
  import { Card, Badge, Button } from '$lib/ui';
  import { get_csq_log, submit_task, logStub } from '../api/stub';
  import type { CsqLogRow } from '../api/types';

  // Poll cadence for the CSQ verdict log. Matches the reasoningData drawer
  // cadence (10s) so the card and the CSQ-pending list stay roughly in sync.
  const POLL_MS = 10_000;

  let rows = $state<CsqLogRow[]>([]);
  // Track which CSQ rows the user has dismissed/actioned this session so the
  // card does not re-surface the same blocked verdict after a Decline.
  let actioned = $state<Set<number>>(new Set());
  let declining = $state<number | null>(null);

  // A row is a "block" iff its conclusion is Critical-severity AND no guard
  // token was present. The SSB payload stores `conclusion` as the raw
  // severity label string (e.g. "Critical"); match case-insensitively so a
  // "CRITICAL" / "critical" producer-side label is still caught.
  function isCriticalNoGuard(r: CsqLogRow): boolean {
    const sev = (r.conclusion ?? '').toString().toLowerCase();
    return sev.includes('crit') && r.guard_present === false;
  }

  const blocking = $derived(
    rows.filter((r) => isCriticalNoGuard(r) && !actioned.has(r.seq)),
  );
  const hasBlocking = $derived(blocking.length > 0);

  async function refresh() {
    try {
      rows = await get_csq_log();
    } catch (err) {
      console.debug('[CsqBlockCard] refresh failed', err);
    }
  }

  async function declineRow(r: CsqLogRow) {
    declining = r.seq;
    try {
      // Submit a replan-request task. The agent runtime interprets this as
      // a request to re-plan the current baby step rather than proceed past
      // the unguarded Critical CSQ. Carries the CSQ seq for traceability.
      await submit_task(
        `replan requested: critical CSQ #${r.csq_id} blocked a baby step ` +
          `without a STOP-Verified guard (check_count=${r.check_count}). ` +
          `Re-plan to add the missing guard or revise the approach.`,
      );
      logStub('csq-block.decline', { seq: r.seq, csq_id: r.csq_id });
      // Mark actioned so the card collapses for this verdict (reassign for
      // Svelte 5 reactivity — mutating a Set in place does not re-trigger).
      actioned = new Set(actioned).add(r.seq);
    } catch (err) {
      console.warn('[CsqBlockCard] decline submit_task failed', err);
    } finally {
      declining = null;
    }
  }

  function dismissRow(r: CsqLogRow) {
    logStub('csq-block.dismiss', { seq: r.seq, csq_id: r.csq_id });
    actioned = new Set(actioned).add(r.seq);
  }

  function fmtTs(ts: number): string {
    if (!ts) return '--:--:--';
    return new Date(ts).toISOString().slice(11, 19);
  }

  onMount(() => {
    void refresh();
    const interval = setInterval(() => void refresh(), POLL_MS);
    return () => clearInterval(interval);
  });
</script>

{#if hasBlocking}
  <Card variant="error" padding={12}>
    <div class="csq-block-head">
      <Badge tone="fail" size="xs">CSQ BLOCK</Badge>
      <span class="csq-block-title">
        {blocking.length} critical CSQ{blocking.length === 1 ? '' : 's'} without guard
      </span>
    </div>
    <div class="csq-block-list">
      {#each blocking as r (r.seq)}
        <div class="csq-block-row">
          <div class="csq-block-meta">
            <Badge tone="fail" size="xs">CRITICAL</Badge>
            <span class="csq-block-id mono">CSQ #{r.csq_id}</span>
            <span class="csq-block-ts">{fmtTs(r.ts_ms)}</span>
          </div>
          <div class="csq-block-detail">
            Unguarded Critical counterfactual-safety verdict ({r.check_count} check{r.check_count === 1 ? '' : 's'}).
            A "STOP-Verified:" guard is required before this baby step may proceed.
          </div>
          <div class="csq-block-actions">
            <Button
              variant="ghost"
              size="sm"
              onclick={() => void declineRow(r)}
            >
              {declining === r.seq ? 'Requesting replan…' : 'Decline (replan)'}
            </Button>
            <Button variant="text" size="sm" onclick={() => dismissRow(r)}>
              Dismiss
            </Button>
          </div>
        </div>
      {/each}
    </div>
  </Card>
{/if}

<style>
  .csq-block-head {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
  }
  .csq-block-title {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fail);
    font-weight: 600;
  }
  .csq-block-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .csq-block-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding-bottom: 8px;
    border-bottom: 1px solid color-mix(in srgb, var(--fail) 25%, transparent);
  }
  .csq-block-row:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }
  .csq-block-meta {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .csq-block-id {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
  }
  .csq-block-ts {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    margin-left: auto;
  }
  .csq-block-detail {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    line-height: var(--leading-code);
  }
  .csq-block-actions {
    display: flex;
    gap: 4px;
    margin-top: 2px;
  }
  .mono {
    font-family: var(--font-mono);
  }
</style>
