<script lang="ts">
  // C-40 — GitHub PR adapter surface (gh CLI shell-out).
  //
  // Renders three sub-panels stacked vertically:
  //   1. Open-PR drawer (title / body / head / base / draft)
  //   2. PR list table (filterable by state + limit)
  //   3. Status snapshot for the focused PR (CI rollup + review)
  //
  // cross-lang-pair: dashboard/tauri/api
  //
  // Backed by `lib/state/github.svelte.ts` which wraps the four Tauri
  // commands declared in `dashboard-v2/src-tauri/src/commands_c40.rs`.
  // No auto-poll — refresh is user-driven so gh's rate limit is not
  // hammered without explicit consent. Mock-mode fallback degrades
  // gracefully: an empty list renders with a "no PRs yet" empty state.

  import { onMount } from 'svelte';
  import { githubStore } from '../state/github.svelte';

  // -------- bootstrap --------
  onMount(() => {
    // Load the active-PR badge once; safe even when the SSB writer is
    // missing (returns null fields).
    void githubStore.refreshActivePr();
    // E-41: load the PR Review section snapshot (read-only).
    void githubStore.refreshPrReviewStatus();
  });

  // -------- handlers --------
  async function onRefreshList(): Promise<void> {
    await githubStore.refreshList();
  }

  async function onSubmitOpen(): Promise<void> {
    await githubStore.submitOpen();
    // Bubble the active-PR badge update when the open succeeded.
    void githubStore.refreshActivePr();
  }

  function onFocus(prNumber: number): void {
    githubStore.setFocusedPrNumber(prNumber);
    void githubStore.refreshStatus();
  }

  async function onCloseFocused(): Promise<void> {
    await githubStore.closeFocused();
  }
</script>

<div class="github-window">
  <header class="github-header">
    <h2>GitHub Pull Requests</h2>
    <div class="active-pr-badge">
      {#if githubStore.activePr.pr_number}
        <span class="badge">
          active PR #{githubStore.activePr.pr_number}
        </span>
      {:else}
        <span class="badge badge-muted">no active PR</span>
      {/if}
    </div>
  </header>

  <!-- ============ Open-PR drawer ============ -->
  <section class="open-drawer">
    <h3>Open a pull request</h3>
    <div class="form-row">
      <label>
        repo (owner/name, blank = current remote)
        <input
          type="text"
          value={githubStore.listRepo ?? ''}
          oninput={(e) => githubStore.setListRepo(
            (e.currentTarget as HTMLInputElement).value || null,
          )}
          placeholder="owner/name"
        />
      </label>
    </div>
    <div class="form-row">
      <label>
        title (required)
        <input
          type="text"
          value={githubStore.formTitle}
          oninput={(e) => githubStore.setFormTitle(
            (e.currentTarget as HTMLInputElement).value,
          )}
          placeholder="add C-40 GitHub PR adapter"
        />
      </label>
    </div>
    <div class="form-row">
      <label>
        body
        <textarea
          rows="4"
          value={githubStore.formBody}
          oninput={(e) => githubStore.setFormBody(
            (e.currentTarget as HTMLTextAreaElement).value,
          )}
          placeholder="see plan; mutation score X%, wiring contract pass/fail..."
        ></textarea>
      </label>
    </div>
    <div class="form-row form-row-inline">
      <label>
        head branch (required)
        <input
          type="text"
          value={githubStore.formHead}
          oninput={(e) => githubStore.setFormHead(
            (e.currentTarget as HTMLInputElement).value,
          )}
          placeholder="feature/c-40"
        />
      </label>
      <label>
        base branch
        <input
          type="text"
          value={githubStore.formBase}
          oninput={(e) => githubStore.setFormBase(
            (e.currentTarget as HTMLInputElement).value,
          )}
          placeholder="main"
        />
      </label>
      <label class="checkbox">
        <input
          type="checkbox"
          checked={githubStore.formDraft}
          onchange={(e) => githubStore.setFormDraft(
            (e.currentTarget as HTMLInputElement).checked,
          )}
        />
        draft
      </label>
    </div>
    <div class="form-row">
      <button
        type="button"
        class="primary"
        onclick={onSubmitOpen}
        disabled={githubStore.openSubmitting
          || githubStore.formTitle.trim() === ''
          || githubStore.formHead.trim() === ''}
      >
        {githubStore.openSubmitting ? 'opening...' : 'open PR'}
      </button>
      {#if githubStore.openSubmitError}
        <span class="error">{githubStore.openSubmitError}</span>
      {/if}
      {#if githubStore.lastOpenedPr && githubStore.lastOpenedPr.url}
        <span class="success">
          opened: <a href={githubStore.lastOpenedPr.url}
            >#{githubStore.lastOpenedPr.number}</a>
        </span>
      {/if}
    </div>
  </section>

  <!-- ============ PR list ============ -->
  <section class="list-panel">
    <header class="list-header">
      <h3>PR list</h3>
      <div class="filter-row">
        <label>
          state
          <select
            value={githubStore.listStateFilter}
            onchange={(e) => githubStore.setListStateFilter(
              (e.currentTarget as HTMLSelectElement).value,
            )}
          >
            <option value="open">open</option>
            <option value="closed">closed</option>
            <option value="merged">merged</option>
            <option value="all">all</option>
          </select>
        </label>
        <label>
          limit
          <input
            type="number"
            min="1"
            max="100"
            value={githubStore.listLimit}
            oninput={(e) => githubStore.setListLimit(
              Number.parseInt((e.currentTarget as HTMLInputElement).value, 10) || 20,
            )}
          />
        </label>
        <button type="button" onclick={onRefreshList}>
          {githubStore.listLoading ? 'refreshing...' : 'refresh'}
        </button>
      </div>
    </header>

    {#if githubStore.listError}
      <div class="error">{githubStore.listError}</div>
    {/if}

    {#if githubStore.prs.length === 0 && !githubStore.listLoading}
      <p class="empty-state">No PRs in this filter. Click refresh to fetch live data.</p>
    {:else}
      <table class="pr-table">
        <thead>
          <tr>
            <th>#</th>
            <th>state</th>
            <th>head</th>
            <th>title</th>
            <th>draft</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each githubStore.prs as pr (pr.number)}
            <tr
              class:focused={githubStore.focusedPrNumber === pr.number}
            >
              <td>#{pr.number}</td>
              <td>{pr.state}</td>
              <td><code>{pr.head_ref_name}</code></td>
              <td>
                <a href={pr.url}>{pr.title}</a>
              </td>
              <td>{pr.is_draft ? 'yes' : ''}</td>
              <td>
                <button
                  type="button"
                  onclick={() => onFocus(pr.number)}
                >
                  focus
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </section>

  <!-- ============ Focused PR status ============ -->
  <section class="status-panel">
    <h3>Focused PR status</h3>
    {#if githubStore.focusedPrNumber === null}
      <p class="empty-state">Click a "focus" button in the list to see CI + review state.</p>
    {:else}
      <div class="status-actions">
        <span>PR #{githubStore.focusedPrNumber}</span>
        <button type="button" onclick={() => githubStore.refreshStatus()}>
          {githubStore.statusLoading ? 'fetching...' : 'refresh status'}
        </button>
        <button
          type="button"
          class="danger"
          onclick={onCloseFocused}
        >
          close PR
        </button>
      </div>
      {#if githubStore.statusError}
        <div class="error">{githubStore.statusError}</div>
      {/if}
      {#if githubStore.focusedStatus}
        <dl class="status-fields">
          <dt>state</dt>
          <dd>{githubStore.focusedStatus.state}</dd>
          <dt>review</dt>
          <dd>
            {githubStore.focusedStatus.review_state_label}
          </dd>
          <dt>CI rollup</dt>
          <dd
            class:ci-success={githubStore.focusedStatus.ci_rollup_label === 'success'}
            class:ci-pending={githubStore.focusedStatus.ci_rollup_label === 'pending'}
            class:ci-failing={githubStore.focusedStatus.ci_rollup_label === 'failing'}
          >
            {githubStore.focusedStatus.ci_rollup_label}
          </dd>
        </dl>
        {#if githubStore.focusedStatus.check_runs.length > 0}
          <table class="checks-table">
            <thead>
              <tr>
                <th>check</th>
                <th>status</th>
                <th>conclusion</th>
              </tr>
            </thead>
            <tbody>
              {#each githubStore.focusedStatus.check_runs as c (c.name)}
                <tr>
                  <td>{c.name}</td>
                  <td>{c.status}</td>
                  <td>{c.conclusion || '(pending)'}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      {/if}
    {/if}
  </section>

  <!-- ============ E-41: PR Review section ============ -->
  <!--
    Surfaces ELAI's PR review extensions: recent reviews, comment-watch
    polling tick, slopsquat dep scan results, semgrep gate findings, and
    auto-merge block reasons. Read-only here — write actions live behind
    the `elai pr-review` CLI subverbs. The store fields default to empty
    arrays so the section renders cleanly when the SSB writer is missing
    (mock-mode parity with the C-40 panels above).
  -->
  <section class="pr-review-panel">
    <h3>PR Review (E-41)</h3>
    <div class="pr-review-grid">
      <div class="pr-review-card">
        <h4>Recent reviews</h4>
        {#if githubStore.recentReviews.length === 0}
          <p class="empty-state">No reviews recorded yet.</p>
        {:else}
          <ul>
            {#each githubStore.recentReviews as r (r.ts_ms + '-' + r.pr_number_hash)}
              <li>
                <code>kind={r.comment_kind_label}</code>
                <span class="muted">@ {r.ts_ms}</span>
              </li>
            {/each}
          </ul>
        {/if}
      </div>

      <div class="pr-review-card">
        <h4>Watcher tick</h4>
        {#if githubStore.lastWatcherTick === null}
          <p class="empty-state">No watcher run recorded.</p>
        {:else}
          <p>
            <strong>{githubStore.lastWatcherTick.new_comments_count}</strong>
            new comment(s) on last poll
          </p>
          <p class="muted">ts {githubStore.lastWatcherTick.ts_ms}</p>
        {/if}
      </div>

      <div class="pr-review-card">
        <h4>Slopsquat scan</h4>
        {#if githubStore.lastSlopsquat === null}
          <p class="empty-state">No verify-deps run recorded.</p>
        {:else}
          <p>
            <strong>{githubStore.lastSlopsquat.slopsquat_count}</strong>
            suspect dep(s) on last scan
          </p>
        {/if}
      </div>

      <div class="pr-review-card">
        <h4>Semgrep gate</h4>
        {#if githubStore.lastSemgrep === null}
          <p class="empty-state">No semgrep run recorded.</p>
        {:else}
          <p>
            findings: <strong>{githubStore.lastSemgrep.findings_count}</strong>
          </p>
          <p class="muted">severity_max hash {githubStore.lastSemgrep.severity_max_hash}</p>
        {/if}
      </div>

      <div class="pr-review-card">
        <h4>Auto-merge guard</h4>
        {#if githubStore.lastAutoMergeBlock === null}
          <p class="empty-state">No block events recorded.</p>
        {:else}
          <p>
            last block: <code>reason_hash={githubStore.lastAutoMergeBlock.block_reason_hash}</code>
          </p>
        {/if}
      </div>
    </div>
  </section>
</div>

<style>
  .github-window {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    padding: 1rem;
    overflow-y: auto;
    height: 100%;
  }

  .github-header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    border-bottom: 1px solid var(--surface-border, #444);
    padding-bottom: 0.5rem;
  }

  .github-header h2 {
    margin: 0;
    font-size: 1.25rem;
  }

  .badge {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    background: var(--bg-surface);
    font-size: 0.85rem;
  }

  .badge-muted {
    color: var(--text-secondary);
  }

  section h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1rem;
    color: var(--text-secondary);
  }

  .form-row {
    margin-bottom: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .form-row-inline {
    flex-direction: row;
    gap: 0.75rem;
    align-items: flex-end;
  }

  .form-row label {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .form-row.form-row-inline label {
    flex: 1;
  }

  .form-row input[type='text'],
  .form-row input[type='number'],
  .form-row textarea,
  .filter-row input[type='number'],
  .filter-row select {
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    color: var(--text-primary);
    padding: 0.25rem 0.5rem;
    font-family: inherit;
    font-size: 0.9rem;
  }

  .form-row.form-row-inline .checkbox {
    flex: 0 0 auto;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
  }

  button {
    background: var(--bg-surface);
    color: var(--text-primary);
    border: 1px solid var(--border-default);
    padding: 0.25rem 0.75rem;
    font-family: inherit;
    cursor: pointer;
  }

  button:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button.primary {
    background: var(--purple);
    color: var(--text-primary);
    border-color: var(--purple);
  }

  button.danger {
    background: var(--fail-ghost);
    border-color: var(--fail-soft);
  }

  .error {
    color: var(--fail);
    font-size: 0.85rem;
  }

  .success {
    color: var(--pass);
    font-size: 0.85rem;
  }

  .list-header,
  .filter-row {
    display: flex;
    align-items: flex-end;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .filter-row label {
    display: flex;
    flex-direction: column;
    gap: 0.125rem;
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .empty-state {
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-style: italic;
  }

  .pr-table,
  .checks-table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 0.5rem;
  }

  .pr-table th,
  .pr-table td,
  .checks-table th,
  .checks-table td {
    text-align: left;
    padding: 0.25rem 0.5rem;
    border-bottom: 1px solid var(--border-default);
    font-size: 0.85rem;
    overflow-wrap: anywhere;
  }

  .pr-table th,
  .checks-table th {
    color: var(--text-secondary);
    font-weight: 600;
  }

  tr.focused {
    background: var(--bg-surface);
  }

  .status-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }

  .status-fields {
    display: grid;
    grid-template-columns: 8rem 1fr;
    gap: 0.25rem 1rem;
    margin: 0;
    font-size: 0.9rem;
  }

  .status-fields dt {
    color: var(--text-secondary);
  }

  .status-fields dd {
    margin: 0;
  }

  .ci-success {
    color: var(--pass);
  }
  .ci-pending {
    color: var(--warn);
  }
  .ci-failing {
    color: var(--fail);
  }

  a {
    color: var(--teal);
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }

  /* E-41 PR Review section */
  .pr-review-panel {
    border-top: 1px solid var(--border-default);
    padding-top: 0.75rem;
  }

  .pr-review-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 0.75rem;
  }

  .pr-review-card {
    background: var(--bg-surface);
    border-radius: 0.25rem;
    padding: 0.5rem 0.75rem;
  }

  .pr-review-card h4 {
    margin: 0 0 0.25rem 0;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .pr-review-card ul {
    margin: 0;
    padding-left: 1rem;
    font-size: 0.85rem;
  }

  .muted {
    color: var(--text-secondary);
    font-size: 0.8rem;
  }
</style>
