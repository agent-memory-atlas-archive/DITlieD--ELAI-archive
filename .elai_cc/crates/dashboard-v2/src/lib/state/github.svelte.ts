// ============================================================
// github.svelte.ts — C-40 GitHub PR adapter store
//
// Live state for the GithubWindow surface. Holds:
//   - cached PR list for the active project's repo
//   - currently focused PR's status snapshot
//   - the last-opened-PR URL surfaced via the active session
//   - status-poll lifecycle (manual refresh; no auto-poll yet to
//     avoid hammering gh's rate limits without explicit user consent)
//
// cross-lang-pair: dashboard/tauri/api
//
// Backed by the Tauri commands declared in
// `dashboard-v2/src-tauri/src/commands_c40.rs`:
//   - github_pr_open / github_pr_list / github_pr_status /
//     github_pr_close / github_pr_get_active_url
// ============================================================

import {
  github_pr_close,
  github_pr_get_active_url,
  github_pr_list,
  github_pr_open,
  github_pr_status,
  logStub,
  pr_review_status,
  type ActivePrUrlDto,
  type PrAutoMergeBlockedDto,
  type PrCommentWatchedDto,
  type PrOpenArgs,
  type PrPackageVerifyResultDto,
  type PrReviewRecentDto,
  type PrSemgrepGateResultDto,
  type PrStatusDto,
  type PrSummaryDto,
} from '../api/stub';

function createGithubStore() {
  // PR list (current `--state` + `--limit` snapshot).
  let prs = $state<PrSummaryDto[]>([]);
  // Last fetch's status / loading flags so the panel can render skeletons.
  let listLoading = $state<boolean>(false);
  let listError = $state<string | null>(null);

  // Focused PR id + its status snapshot.
  let focusedPrNumber = $state<number | null>(null);
  let focusedStatus = $state<PrStatusDto | null>(null);
  let statusLoading = $state<boolean>(false);
  let statusError = $state<string | null>(null);

  // Active URL surfaced via the last `github_pr_open` response. The
  // SSB row only carries a hashed URL prefix, so this store is the
  // source of truth for the human-readable URL within the dashboard
  // session.
  let activePr = $state<ActivePrUrlDto>({ url: null, pr_number: null, ts_ms: null });

  // Filter state for `github_pr_list`. Kept here so multiple sub-views
  // share one filter without prop-drilling.
  let listRepo = $state<string | null>(null);
  let listStateFilter = $state<string>('open');
  let listLimit = $state<number>(20);

  // Open-PR form state. Bound by the GithubWindow's create-PR drawer.
  let formTitle = $state<string>('');
  let formBody = $state<string>('');
  let formHead = $state<string>('');
  let formBase = $state<string>('main');
  let formDraft = $state<boolean>(true);
  let openSubmitting = $state<boolean>(false);
  let openSubmitError = $state<string | null>(null);
  let lastOpenedPr = $state<PrSummaryDto | null>(null);

  // E-41 PR review surface — latest snapshot of the 5 PR review SSB
  // variants (review_composed, comment_watched, package_verify_result,
  // semgrep_gate_result, auto_merge_blocked). Defaults degrade
  // gracefully in mock mode (empty arrays / null entries).
  let recentReviews = $state<PrReviewRecentDto[]>([]);
  let lastWatcherTick = $state<PrCommentWatchedDto | null>(null);
  let lastSlopsquat = $state<PrPackageVerifyResultDto | null>(null);
  let lastSemgrep = $state<PrSemgrepGateResultDto | null>(null);
  let lastAutoMergeBlock = $state<PrAutoMergeBlockedDto | null>(null);

  return {
    // --- read-side accessors (Svelte 5 runes get-only) ---
    get prs() { return prs; },
    get listLoading() { return listLoading; },
    get listError() { return listError; },
    get focusedPrNumber() { return focusedPrNumber; },
    get focusedStatus() { return focusedStatus; },
    get statusLoading() { return statusLoading; },
    get statusError() { return statusError; },
    get activePr() { return activePr; },
    get listRepo() { return listRepo; },
    get listStateFilter() { return listStateFilter; },
    get listLimit() { return listLimit; },
    get formTitle() { return formTitle; },
    get formBody() { return formBody; },
    get formHead() { return formHead; },
    get formBase() { return formBase; },
    get formDraft() { return formDraft; },
    get openSubmitting() { return openSubmitting; },
    get openSubmitError() { return openSubmitError; },
    get lastOpenedPr() { return lastOpenedPr; },
    // E-41 PR review section
    get recentReviews() { return recentReviews; },
    get lastWatcherTick() { return lastWatcherTick; },
    get lastSlopsquat() { return lastSlopsquat; },
    get lastSemgrep() { return lastSemgrep; },
    get lastAutoMergeBlock() { return lastAutoMergeBlock; },

    // --- mutators for form fields ---
    setListRepo(v: string | null) { listRepo = v; },
    setListStateFilter(v: string) { listStateFilter = v; },
    setListLimit(v: number) { listLimit = v; },
    setFormTitle(v: string) { formTitle = v; },
    setFormBody(v: string) { formBody = v; },
    setFormHead(v: string) { formHead = v; },
    setFormBase(v: string) { formBase = v; },
    setFormDraft(v: boolean) { formDraft = v; },
    setFocusedPrNumber(v: number | null) {
      focusedPrNumber = v;
      // Drop the stale status snapshot so the UI shows a skeleton
      // until the new pr's status arrives.
      focusedStatus = null;
      statusError = null;
    },

    // --- side-effecting actions ---

    /**
     * Refresh the PR list from the active repo. Driven by the user
     * (button click); no auto-poll to respect gh rate limits.
     */
    async refreshList(): Promise<void> {
      logStub('github_store.refreshList');
      listLoading = true;
      listError = null;
      try {
        prs = await github_pr_list(
          listRepo ?? undefined,
          listStateFilter,
          listLimit,
        );
      } catch (e) {
        listError = String(e);
      } finally {
        listLoading = false;
      }
    },

    /**
     * Fetch the status snapshot for [focusedPrNumber]. Emits one
     * `github_pr_status_fetched` SSB row per call (server-side).
     */
    async refreshStatus(): Promise<void> {
      const n = focusedPrNumber;
      if (n === null) {
        statusError = 'no PR focused';
        return;
      }
      logStub('github_store.refreshStatus', { prNumber: n });
      statusLoading = true;
      statusError = null;
      try {
        focusedStatus = await github_pr_status(n, listRepo ?? undefined);
      } catch (e) {
        statusError = String(e);
      } finally {
        statusLoading = false;
      }
    },

    /**
     * Open a new PR from the current form fields. On success, refreshes
     * the list, sets `lastOpenedPr`, and clears the form.
     */
    async submitOpen(): Promise<PrSummaryDto | null> {
      if (formTitle.trim() === '' || formHead.trim() === '') {
        openSubmitError = 'title and head branch are required';
        return null;
      }
      openSubmitting = true;
      openSubmitError = null;
      const args: PrOpenArgs = {
        repo: listRepo ?? undefined,
        title: formTitle,
        body: formBody,
        head: formHead,
        base: formBase,
        draft: formDraft,
      };
      try {
        const summary = await github_pr_open(args);
        lastOpenedPr = summary;
        // Refresh in the background; we deliberately don't await so the
        // user sees the success state immediately.
        void this.refreshList();
        // Clear the form on success so accidental double-submits don't
        // re-open the same PR.
        formTitle = '';
        formBody = '';
        return summary;
      } catch (e) {
        openSubmitError = String(e);
        return null;
      } finally {
        openSubmitting = false;
      }
    },

    /**
     * Close (without merging) the currently focused PR.
     */
    async closeFocused(): Promise<boolean> {
      const n = focusedPrNumber;
      if (n === null) {
        statusError = 'no PR focused';
        return false;
      }
      try {
        await github_pr_close(n, listRepo ?? undefined);
        // Refresh both list + status so the panel reflects the close.
        await Promise.all([this.refreshList(), this.refreshStatus()]);
        return true;
      } catch (e) {
        statusError = String(e);
        return false;
      }
    },

    /**
     * Reload the "active PR" badge from the latest SSB
     * `github_pr_opened` row. Read-only; safe to call on mount.
     */
    async refreshActivePr(): Promise<void> {
      try {
        activePr = await github_pr_get_active_url();
      } catch (e) {
        console.debug('[github_store] refreshActivePr failed', e);
      }
    },

    /**
     * E-41: reload the PR Review section snapshot from the latest SSB
     * rows. Read-only; safe to call on mount alongside refreshActivePr.
     */
    async refreshPrReviewStatus(): Promise<void> {
      try {
        logStub('github_store.refreshPrReviewStatus');
        const s = await pr_review_status();
        recentReviews = s.recent_reviews;
        lastWatcherTick = s.last_watcher_tick;
        lastSlopsquat = s.last_slopsquat;
        lastSemgrep = s.last_semgrep;
        lastAutoMergeBlock = s.last_auto_merge_block;
      } catch (e) {
        console.debug('[github_store] refreshPrReviewStatus failed', e);
      }
    },
  };
}

export const githubStore = createGithubStore();
