<script lang="ts">
  // ================================================================
  // MediaValidationPanel.svelte — B-24 Phase 4I Group C8.
  //
  // Surfaces the media validator pipeline + RCE feedback loop. Pulls
  // its data from mediaValidation.svelte.ts (reactive store backed
  // by the Tauri media_ssb subscriber).
  //
  // Layout: header + accordion list of per-step entries. Each entry
  // expands to show Tier1 chips, Tier2 advisory + JSON, render frame
  // grid, iteration timeline, and (when verdict === 'HumanReview')
  // Pass / Fail / Revalidate buttons.
  //
  // Anti-stale: Svelte 5 runes ($state / $derived / $effect), Lucide
  // icons via @lucide/svelte (already a dep — package.json L18).
  // ================================================================

  import { onMount, onDestroy } from 'svelte';
  import { mediaValidation } from '$lib/state/media_validation.svelte';
  import type {
    MediaValidationEntry,
    ReviewDecision,
  } from '$lib/api/types';
  import { invoke } from '@tauri-apps/api/core';

  // E-14 — browser e2e status row, fetched once on mount and refreshed
  // on user request. Stays optional so the panel keeps rendering when
  // the backend has no browser_e2e_* SSB rows yet.
  type BrowserE2eStatus = {
    runs_started: number;
    assertions_passed: number;
    assertions_failed: number;
    snapshots_captured: number;
    last_journey_name_hash: string | null;
    last_ts_ms: number | null;
  };
  let browserE2eStatus = $state<BrowserE2eStatus | null>(null);
  let browserE2eError = $state<string>('');

  // Expanded step ids — local UI state; not persisted.
  let expanded = $state<Record<string, boolean>>({});

  // Per-step review submission lock — disables the action buttons while
  // a submit is in flight to prevent double-clicks.
  let reviewBusy = $state<Record<string, boolean>>({});

  // Last review error per step, surfaced inline.
  let reviewError = $state<Record<string, string>>({});

  // Mount: bind the store to live data. unmount: tear down the listener.
  onMount(() => {
    void mediaValidation.mount();
    void refreshBrowserE2eStatus();
  });
  onDestroy(() => {
    mediaValidation.unmount();
  });

  // E-14 — Tauri-side query for the browser-e2e SSB summary. The
  // backend command `browser_e2e_status` aggregates the latest 4
  // browser_e2e_* SSB action types and returns a typed summary.
  async function refreshBrowserE2eStatus(): Promise<void> {
    try {
      const s = await invoke<BrowserE2eStatus>('browser_e2e_status');
      browserE2eStatus = s;
      browserE2eError = '';
    } catch (e) {
      browserE2eStatus = null;
      browserE2eError = String(e);
    }
  }

  // Derived: live entries from the store, already sorted by updated_at desc.
  let entries = $derived(mediaValidation.entries);
  let loading = $derived(mediaValidation.loading);
  let loadError = $derived(mediaValidation.loadError);
  let lastEvent = $derived(mediaValidation.lastEvent);

  function toggle(stepId: string): void {
    expanded = { ...expanded, [stepId]: !expanded[stepId] };
  }

  // B-24 Phase 4I Group C8 audit WARN 3 — type-narrowing guard for the
  // tier2.structured_output field, which is typed as `unknown` in the
  // wire-format types (matches the orchestrator's `serde_json::Value`).
  // The previous `Object.keys(x as object)` cast threw at runtime when
  // structured_output was null/undefined/array/scalar. This narrows
  // safely and yields `false` for all non-record values.
  function isObjectWithKeys(v: unknown): v is Record<string, unknown> {
    return (
      v != null &&
      typeof v === 'object' &&
      !Array.isArray(v) &&
      Object.keys(v as Record<string, unknown>).length > 0
    );
  }

  function basename(path: string): string {
    if (!path) return '';
    const parts = path.split(/[\\/]/);
    return parts[parts.length - 1] || path;
  }

  function verdictTone(verdict: string | null): string {
    if (!verdict) return 'pending';
    switch (verdict) {
      case 'Pass':         return 'pass';
      case 'Fail':         return 'fail';
      case 'HumanReview':  return 'warn';
      case 'Degraded':     return 'degraded';
      default:             return 'pending';
    }
  }

  function mediaIcon(mediaType: string): string {
    switch (mediaType) {
      case 'image':         return 'img';
      case 'model_3d':      return '3d';
      case 'audio':         return 'wav';
      case 'ui_screenshot': return 'ui';
      default:              return 'med';
    }
  }

  function fmtLatency(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  }

  function fmtAge(ms: number): string {
    const diff = Date.now() - ms;
    if (diff < 1000) return 'just now';
    if (diff < 60_000) return `${Math.floor(diff / 1000)}s ago`;
    if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`;
    return `${Math.floor(diff / 3_600_000)}h ago`;
  }

  async function submitReview(
    stepId: string,
    decision: ReviewDecision,
  ): Promise<void> {
    if (reviewBusy[stepId]) return;
    reviewBusy = { ...reviewBusy, [stepId]: true };
    reviewError = { ...reviewError, [stepId]: '' };
    try {
      const result = await mediaValidation.submitReview(stepId, decision);
      if (!result.ok) {
        reviewError = { ...reviewError, [stepId]: result.error ?? 'submit failed' };
      }
    } finally {
      reviewBusy = { ...reviewBusy, [stepId]: false };
    }
  }

  function structuredJson(entry: MediaValidationEntry): string {
    if (!entry.tier2) return '';
    try {
      return JSON.stringify(entry.tier2.structured_output, null, 2);
    } catch {
      return String(entry.tier2.structured_output);
    }
  }

  async function refresh(): Promise<void> {
    await mediaValidation.refresh();
  }
</script>

<div class="media-validation-panel">
  <header class="mvp-header">
    <span class="mvp-title">media validation</span>
    <span class="mvp-count mono">
      {entries.length} {entries.length === 1 ? 'step' : 'steps'}
    </span>
    <button
      class="mvp-refresh"
      onclick={() => void refresh()}
      disabled={loading}
      aria-label="Refresh media validations"
    >{loading ? '…' : 'refresh'}</button>
  </header>

  {#if lastEvent}
    <div class="mvp-event mono" aria-live="polite">
      live: {lastEvent.action} · {lastEvent.step_id}
    </div>
  {/if}

  <!--
    E-14 — Browser E2E summary row. Shows aggregate counts from the
    browser_e2e_run_started / browser_e2e_assertion_passed /
    browser_e2e_assertion_failed / browser_e2e_snapshot_captured SSB
    rows so operators can verify the runner is producing observable
    effect without leaving the media-validation panel.
  -->
  <section class="mvp-browser-e2e" aria-label="browser e2e summary">
    <div class="mvp-be2e-label mono">browser e2e</div>
    {#if browserE2eError}
      <div class="mvp-error mono">browser e2e: {browserE2eError}</div>
    {:else if browserE2eStatus}
      <div class="mvp-be2e-row mono">
        <span class="mvp-be2e-stat">runs: {browserE2eStatus.runs_started}</span>
        <span class="mvp-be2e-stat mvp-be2e-pass">
          asserts pass: {browserE2eStatus.assertions_passed}
        </span>
        <span class="mvp-be2e-stat mvp-be2e-fail">
          asserts fail: {browserE2eStatus.assertions_failed}
        </span>
        <span class="mvp-be2e-stat">
          snapshots: {browserE2eStatus.snapshots_captured}
        </span>
        <button
          class="mvp-refresh"
          onclick={() => void refreshBrowserE2eStatus()}
          aria-label="Refresh browser e2e status"
        >refresh</button>
      </div>
    {:else}
      <div class="mvp-be2e-row mono">no browser-e2e rows yet</div>
    {/if}
  </section>

  {#if loadError}
    <div class="mvp-error mono">load error: {loadError}</div>
  {/if}

  {#if entries.length === 0 && !loadError}
    <div class="mvp-empty">
      <span class="mono">no media validations yet</span>
      <span class="mvp-empty-sub mono">
        validations stream live via SSB when the task runner emits media artifacts
      </span>
    </div>
  {:else}
    <ul class="mvp-list" role="list">
      {#each entries as entry (entry.step_id)}
        {@const isOpen = !!expanded[entry.step_id]}
        {@const tone = verdictTone(entry.final_verdict)}
        <li class="mvp-entry mvp-entry--{tone}" role="listitem">
          <button
            class="mvp-entry-header"
            onclick={() => toggle(entry.step_id)}
            aria-expanded={isOpen}
            aria-controls={`mvp-body-${entry.step_id}`}
          >
            <span class="mvp-icon mono">{mediaIcon(entry.media_type)}</span>
            <span class="mvp-file mono" title={entry.file_path}>
              {basename(entry.file_path) || '(no path)'}
            </span>
            <span class="mvp-step mono">{entry.step_id}</span>
            <span class="mvp-verdict mvp-verdict--{tone} mono">
              {entry.final_verdict ?? 'pending'}
            </span>
            <span class="mvp-age mono">{fmtAge(entry.updated_at)}</span>
          </button>

          {#if isOpen}
            <div class="mvp-body" id={`mvp-body-${entry.step_id}`}>
              <!-- Tier 1 -->
              <section class="mvp-section">
                <div class="mvp-section-label mono">tier 1 (deterministic)</div>
                {#if entry.tier1}
                  <div class="mvp-tier1-summary mono">
                    <span class="mvp-chip mvp-chip--{entry.tier1.pass ? 'pass' : 'fail'}">
                      {entry.tier1.pass ? 'pass' : 'fail'}
                    </span>
                    <span class="mvp-meta">
                      {entry.tier1.check_count} checks · {fmtLatency(entry.tier1.latency_ms)}
                    </span>
                  </div>
                  {#if entry.tier1.checks.length > 0}
                    <div class="mvp-chips">
                      {#each entry.tier1.checks as check (check.name)}
                        <div
                          class="mvp-chip mvp-chip--{check.pass ? 'pass' : 'fail'}"
                          title={`${check.detail} (${fmtLatency(check.latency_ms)})`}
                        >
                          <span class="mvp-chip-name">{check.name}</span>
                          <span class="mvp-chip-detail">{check.detail}</span>
                        </div>
                      {/each}
                    </div>
                  {/if}
                {:else}
                  <div class="mvp-meta mono">no tier 1 result yet</div>
                {/if}
              </section>

              <!-- Tier 2 -->
              <section class="mvp-section">
                <div class="mvp-section-label mono">tier 2 (vlm advisory)</div>
                {#if entry.tier2}
                  <div class="mvp-tier2-summary mono">
                    <span class="mvp-chip mvp-chip--{entry.tier2.advisory_pass ? 'pass' : 'fail'}">
                      {entry.tier2.advisory_pass ? 'advisory pass' : 'advisory fail'}
                    </span>
                    <span class="mvp-meta">
                      {entry.tier2.model_used} · {fmtLatency(entry.tier2.latency_ms)}
                    </span>
                  </div>
                  {#if isObjectWithKeys(entry.tier2.structured_output)}
                    <details class="mvp-details">
                      <summary class="mono">structured output</summary>
                      <pre class="mvp-json mono">{structuredJson(entry)}</pre>
                    </details>
                  {/if}
                {:else}
                  <div class="mvp-meta mono">no tier 2 advisory yet</div>
                {/if}
              </section>

              <!-- Render frames -->
              <section class="mvp-section">
                <div class="mvp-section-label mono">render frames</div>
                {#if entry.render_frames.length === 0}
                  <div class="mvp-meta mono">no frames captured</div>
                {:else}
                  <div class="mvp-frame-grid">
                    {#each entry.render_frames as frame, i (i)}
                      <div class="mvp-frame">
                        {#if frame.image_path}
                          <img
                            class="mvp-frame-img"
                            src={`file://${frame.image_path}`}
                            alt={`frame ${i} from ${frame.adapter_name}`}
                            onerror={(e) => {
                              const t = e.target as HTMLImageElement;
                              t.style.display = 'none';
                              const sib = t.nextElementSibling as HTMLElement | null;
                              if (sib) sib.style.display = 'flex';
                            }}
                          />
                          <div class="mvp-frame-fallback mono" style="display: none">
                            {basename(frame.image_path)}
                          </div>
                        {:else}
                          <div class="mvp-frame-fallback mono">
                            <span>{frame.adapter_name}</span>
                            <span>{frame.frame_count} frame{frame.frame_count === 1 ? '' : 's'}</span>
                          </div>
                        {/if}
                        <div class="mvp-frame-caption mono">
                          {frame.adapter_name} · {fmtLatency(frame.latency_ms)}
                        </div>
                      </div>
                    {/each}
                  </div>
                {/if}
              </section>

              <!-- Feedback iterations -->
              <section class="mvp-section">
                <div class="mvp-section-label mono">
                  feedback iterations
                  <span class="mvp-meta">({entry.feedback_iterations.length})</span>
                </div>
                {#if entry.feedback_iterations.length === 0}
                  <div class="mvp-meta mono">no iterations yet</div>
                {:else}
                  <ol class="mvp-timeline">
                    {#each entry.feedback_iterations as iter (iter.iteration)}
                      <li class="mvp-iter mvp-iter--{iter.converged ? 'converged' : 'open'}">
                        <span class="mvp-iter-n mono">#{iter.iteration}</span>
                        <span class="mvp-iter-summary mono">
                          {iter.issue_count} issue{iter.issue_count === 1 ? '' : 's'}
                          {iter.converged ? '· converged' : ''}
                          {iter.verdict ? `· ${iter.verdict}` : ''}
                        </span>
                        {#if iter.issues.length > 0}
                          <ul class="mvp-iter-issues mono">
                            {#each iter.issues as issue, j (j)}
                              <li>{issue}</li>
                            {/each}
                          </ul>
                        {/if}
                      </li>
                    {/each}
                  </ol>
                {/if}
              </section>

              <!-- HumanReview action row -->
              {#if entry.final_verdict === 'HumanReview'}
                <section class="mvp-section mvp-actions">
                  <div class="mvp-section-label mono">human review required</div>
                  <div class="mvp-action-row">
                    <button
                      class="mvp-btn mvp-btn--pass"
                      disabled={!!reviewBusy[entry.step_id]}
                      onclick={() => void submitReview(entry.step_id, 'Pass')}
                    >Pass</button>
                    <button
                      class="mvp-btn mvp-btn--fail"
                      disabled={!!reviewBusy[entry.step_id]}
                      onclick={() => void submitReview(entry.step_id, 'Fail')}
                    >Fail</button>
                    <button
                      class="mvp-btn mvp-btn--rev"
                      disabled={!!reviewBusy[entry.step_id]}
                      onclick={() => void submitReview(entry.step_id, 'RequestRevalidate')}
                    >Revalidate</button>
                  </div>
                  {#if reviewError[entry.step_id]}
                    <div class="mvp-error mono">
                      submit failed: {reviewError[entry.step_id]}
                    </div>
                  {/if}
                </section>
              {/if}
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .media-validation-panel {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    padding: var(--s-3) var(--s-4);
  }

  .mvp-header {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding-bottom: var(--s-2);
    border-bottom: 1px solid var(--border-subtle);
  }

  .mvp-title {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.10em;
    color: var(--text-ghost);
    flex: 1;
  }

  .mvp-count {
    font-size: 10px;
    color: var(--text-ghost);
  }

  .mvp-refresh {
    background: transparent;
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 8px;
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease-std), color var(--dur-fast) var(--ease-std);
  }
  .mvp-refresh:hover:not(:disabled) {
    border-color: var(--teal-line);
    color: var(--teal);
  }
  .mvp-refresh:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .mvp-event {
    font-size: 10px;
    color: var(--teal);
    opacity: 0.7;
    padding: 2px var(--s-1);
  }

  .mvp-error {
    font-size: 10px;
    color: var(--fail);
    padding: var(--s-1) var(--s-1);
  }

  .mvp-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-1);
    padding: var(--s-6) var(--s-2);
  }
  .mvp-empty-sub {
    font-size: 10px;
    color: var(--text-ghost);
    text-align: center;
  }

  .mvp-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .mvp-entry {
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    overflow: hidden;
    background: rgba(16, 20, 26, 0.25);
  }

  .mvp-entry--pass     { border-left: 2px solid var(--teal); }
  .mvp-entry--fail     { border-left: 2px solid var(--fail); }
  .mvp-entry--warn     { border-left: 2px solid var(--warn); }
  .mvp-entry--degraded { border-left: 2px solid var(--warn); opacity: 0.85; }
  .mvp-entry--pending  { border-left: 2px solid var(--border-default); }

  .mvp-entry-header {
    display: grid;
    grid-template-columns: auto 1fr auto auto auto;
    align-items: center;
    gap: var(--s-3);
    width: 100%;
    padding: var(--s-2) var(--s-3);
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    text-align: left;
    font-family: inherit;
    font-size: inherit;
  }
  .mvp-entry-header:hover {
    background: var(--teal-ghost);
  }

  .mvp-icon {
    font-size: 10px;
    color: var(--text-ghost);
    text-transform: uppercase;
    padding: 1px 4px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
  }
  .mvp-file {
    font-size: 12px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .mvp-step {
    font-size: 10px;
    color: var(--text-ghost);
  }
  .mvp-age {
    font-size: 10px;
    color: var(--text-disabled);
  }

  .mvp-verdict {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    padding: 1px 6px;
    border-radius: var(--r-xs);
    border: 1px solid;
  }
  .mvp-verdict--pass     { color: var(--teal);     border-color: var(--teal-line); }
  .mvp-verdict--fail     { color: var(--fail);     border-color: var(--fail); }
  .mvp-verdict--warn     { color: var(--warn);     border-color: var(--warn); }
  .mvp-verdict--degraded { color: var(--warn);     border-color: var(--warn); opacity: 0.7; }
  .mvp-verdict--pending  { color: var(--text-ghost); border-color: var(--border-default); }

  .mvp-body {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
    padding: var(--s-3);
    border-top: 1px solid var(--border-subtle);
    background: rgba(8, 12, 18, 0.4);
  }

  .mvp-section {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }

  .mvp-section-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.10em;
    color: var(--text-ghost);
  }

  .mvp-meta {
    font-size: 10px;
    color: var(--text-ghost);
  }

  .mvp-chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-1);
  }

  .mvp-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 8px;
    border-radius: var(--r-xs);
    border: 1px solid;
  }
  .mvp-chip--pass { color: var(--teal); border-color: var(--teal-line); }
  .mvp-chip--fail { color: var(--fail); border-color: var(--fail); }

  .mvp-chip-name { font-weight: 500; }
  .mvp-chip-detail { color: var(--text-ghost); }

  .mvp-tier1-summary, .mvp-tier2-summary {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .mvp-details {
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    padding: var(--s-1) var(--s-2);
  }
  .mvp-details summary {
    cursor: pointer;
    color: var(--text-ghost);
    font-size: 10px;
    padding: 2px 0;
  }

  .mvp-json {
    font-size: 10px;
    color: var(--text-secondary);
    white-space: pre-wrap;
    word-break: break-all;
    margin: 0;
    padding: var(--s-1) 0 0;
    max-height: 280px;
    overflow-y: auto;
  }

  .mvp-frame-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
    gap: var(--s-2);
  }
  .mvp-frame {
    display: flex;
    flex-direction: column;
    gap: 2px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    overflow: hidden;
    background: rgba(0, 0, 0, 0.35);
    min-height: 88px;
  }
  .mvp-frame-img {
    width: 100%;
    height: 80px;
    object-fit: cover;
    display: block;
  }
  .mvp-frame-fallback {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 80px;
    font-size: 10px;
    color: var(--text-ghost);
    gap: 2px;
    text-align: center;
    padding: 0 var(--s-1);
  }
  .mvp-frame-caption {
    font-size: 10px;
    color: var(--text-disabled);
    padding: 2px var(--s-1);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .mvp-timeline {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: var(--s-1);
  }
  .mvp-iter {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--s-2);
    padding: 4px var(--s-2);
    border-left: 1px solid var(--border-subtle);
  }
  .mvp-iter--converged { border-left-color: var(--teal); }
  .mvp-iter--open      { border-left-color: var(--warn); }
  .mvp-iter-n {
    font-size: 10px;
    color: var(--text-ghost);
  }
  .mvp-iter-summary {
    font-size: 10px;
  }
  .mvp-iter-issues {
    grid-column: 2;
    list-style: disc;
    padding-left: var(--s-3);
    margin: 2px 0 0;
    color: var(--text-ghost);
    font-size: 10px;
  }

  .mvp-actions { gap: var(--s-2); }
  .mvp-action-row {
    display: flex;
    gap: var(--s-2);
  }

  .mvp-btn {
    background: transparent;
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 12px;
    padding: 4px 12px;
    cursor: pointer;
    transition:
      border-color var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std),
      background var(--dur-fast) var(--ease-std);
  }
  .mvp-btn:hover:not(:disabled) {
    background: var(--teal-ghost);
  }
  .mvp-btn:disabled { opacity: 0.5; cursor: not-allowed; }

  .mvp-btn--pass {
    color: var(--teal);
    border-color: var(--teal-line);
  }
  .mvp-btn--pass:hover:not(:disabled) { color: var(--teal-bright); border-color: var(--teal); }

  .mvp-btn--fail {
    color: var(--fail);
    border-color: var(--fail);
  }

  .mvp-btn--rev {
    color: var(--warn);
    border-color: var(--warn);
  }
</style>
