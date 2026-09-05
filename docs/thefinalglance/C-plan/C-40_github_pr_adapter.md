# C-40: GitHub PR Adapter (Devin Parity, Additive)

Objective: ship a minimal GitHub PR adapter so /session merge can complete the
loop from local commit to upstream review surface — open the PR, write a body
templated from the SessionDAG (C-23) branch summary, attach a structured
audit-trail comment, and respond to PR review comments via the same SSB-driven
loop ELAI already uses for interactive sessions. Closes the most visible gap
against Devin's "autonomous PR + review-comment response" demo.

Pre-requisites: C-13 (squash_wip_commits + branch merge primitives) SHIPPED;
C-23 (SessionDAG branch UI + branch_merged SSB row) SHIPPED.
Parallel-safe with: C-35..C-39, C-41..C-53 (orthogonal). Sequence ahead of any
patch that adds external write surfaces (currently none planned in C-N).

Reference grounding (verified via prior oracle 2026-05-17):
- Devin 2.0 autonomous PR + review-comment response loop documented in
  cognition-labs.com and third-party reviews (Idlen 2026). Counter-evidence:
  Devin's original Upwork demo was "borderline deceptive" per Internet of Bugs
  (Wikipedia). Treat Devin's PR-loop as the published feature, not the
  benchmark-grade claim — implementation parity is the goal, not behavioral
  imitation of unverified marketing.
- B-25 (Empire Integration) is DEFERRED and does NOT cover GitHub. Without
  C-40, ELAI has no external code-review surface; sessions terminate at local
  branch merge.

## Context

ELAI's session loop ends at `/session merge` which produces a branch_merged
SSB row and a local fast-forward or squash commit. There is no path to
external review. For solo workflows on local repos, that's fine. For:
1. Hirer/client gigs where the human reviewer is upstream
2. Open-source contributions
3. Multi-developer teams using GitHub as the review surface

ELAI is currently a closed loop. C-40 opens the loop with a minimal,
opt-in, additive adapter that:
- Uses `gh` CLI (already a common Windows + Mac + Linux install) for auth
  and PR mechanics. Falls back to git2 + raw GitHub REST API if `gh` absent
  (gated behind a config flag — `gh` is the recommended path).
- Composes PR body from the SessionDAG branch summary: list of baby steps
  with their acceptance status, audit summaries, stub-tracking-status,
  test-trust metrics (mutation score, MR violations).
- Watches for review comments via `gh pr view --comments` polling (default
  60s interval, configurable) and routes them into the FSM as a new
  Review-Comment-Triggered session.
- Emits SSB rows for every GitHub interaction so the dashboard shows the
  external loop's state without leaving ELAI.

C-40 is ADDITIVE — opt-in per project via elai_cc.toml [github] block.
Sessions that don't configure GitHub continue working identically.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-13_ipc_dispatcher_production_swap.md
- F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-23_sessiondag_branch_ui.md
- .elai_cc/crates/orchestrator/src/session/dag.rs (branch summary source)
- .elai_cc/crates/orchestrator/src/session/merge.rs (where C-13 lands)
- gh CLI documentation: cli.github.com/manual

## Tasks

1. **elai_cc.toml [github] section** — add config block:
   ```
   [github]
   enabled = false  # opt-in per project
   adapter = "gh"  # "gh" or "rest"; gh is default and recommended
   default_base_branch = "main"
   pr_body_template = "default"  # "default" or path to custom template
   review_poll_interval_s = 60
   auto_respond_review_comments = false  # default off; surface to user first
   ```

2. **crates/orchestrator/src/github_pr/ module (NEW, cfg-gated)** —
   feature="github_pr" + cfg-gated module under orchestrator. Exports:
   GithubAdapter trait, GhCliAdapter impl, RestApiAdapter impl,
   PrBodyComposer, ReviewCommentWatcher, ReviewCommentRouter.

3. **GhCliAdapter** — shells out to `gh pr create`, `gh pr view`,
   `gh pr comment`. Parses gh's JSON output (gh has `--json` flag). Surface
   gh-not-installed and gh-not-authenticated as structured errors via SSB.

4. **RestApiAdapter (fallback)** — direct REST calls via reqwest, auth via
   GITHUB_TOKEN env var. Subset of operations: create PR, get PR comments,
   post PR comment. Used only when `adapter = "rest"` OR `gh` is unavailable
   AND user explicitly opted into fallback.

5. **PrBodyComposer** — takes a SessionDAG branch summary (existing struct
   from C-23) and renders a PR body template. Default template includes:
   - Title: first non-empty baby-step description
   - Summary: list of baby steps with acceptance status badges
   - Audit trail: links to SSB log artifact (committed under
     .elai/audit/{session_id}.md)
   - Test trust: mutation score, MR coverage, IntentPair coverage from
     C-29 + C-28 metrics
   - Stub status: open vs closed stubs from C-1..C-8 SSB
   - **Honesty section**: "This PR was authored by ELAI via session {id}.
     Mutation score: X%. Wiring contract: pass/fail. Reviewer should verify
     ACCEPTANCE checklist below." (no marketing puffery; just facts)

6. **ReviewCommentWatcher** — async task spawned when --watch-reviews flag
   passed to `/session merge --to-pr`. Polls gh pr view every
   review_poll_interval_s. New comments emit SSB review_comment_received
   row.

7. **ReviewCommentRouter** — when auto_respond_review_comments = true AND
   a review_comment_received SSB row appears, opens a new ELAI session
   (Review-Comment-Triggered) scoped to the file mentioned in the comment.
   Default off — most workflows want human-in-the-loop here.

8. **SSB action types** — add 6 new strings: github_pr_created,
   github_pr_body_posted, github_comment_received, github_comment_routed_to_session,
   github_adapter_error, github_review_poll_tick. SCHEMA_VERSION bump.

9. **CLI subcommands** — add to `elai session merge`:
   - `--to-pr` (creates PR after local merge)
   - `--base <branch>` (override default base)
   - `--watch-reviews` (launch ReviewCommentWatcher)
   - `--draft` (open as draft PR)

10. **Tests first (TDD)**:
    - 4A.X.3: PrBodyComposer renders deterministic body from a synthetic
      SessionDAG fixture
    - 4A.X.4: GhCliAdapter create_pr is feature-tested against a mocked
      `gh` CLI (PATH-injected wrapper script) — verifies JSON-output parsing
    - 4A.X.5: ReviewCommentWatcher emits SSB row on first poll detection
      of a new comment (mocked gh output)
    - 4A.X.6: ReviewCommentRouter does NOT auto-trigger when
      auto_respond_review_comments = false
    - 4A.X.7: Adapter falls back to RestApi when gh absent AND
      adapter = "rest" set explicitly; refuses fallback when not opted in

11. **Production wiring** — wire CLI flags through to session merge path.
    GithubAdapter lifecycle bound to session lifetime; watcher torn down
    on session end.

12. **Dashboard panel (dashboard-v2/GithubPanel.svelte, PLANNED)** —
    shows current PR URL, comment count, last-poll timestamp, watcher
    status. Optional but high-value.

13. **Contract tests** — tests/contracts/github_adapter_contract.rs verifying
    both adapter impls satisfy the same trait shape.

## WIRING_CONTRACT

produces_symbols:
  - GithubAdapter trait, GhCliAdapter, RestApiAdapter (crates/orchestrator/src/github_pr/)
  - PrBodyComposer, ReviewCommentWatcher, ReviewCommentRouter
  - SSB action_types: github_pr_created, github_pr_body_posted,
    github_comment_received, github_comment_routed_to_session,
    github_adapter_error, github_review_poll_tick
  - CLI flags: --to-pr, --base, --watch-reviews, --draft
  - GithubPanel.svelte (dashboard-v2, PLANNED)
  - elai_cc.toml [github] config section
consumes_symbols:
  - SessionDAG branch summary (C-23)
  - session merge primitive (C-13)
  - SSB event bus
  - gh CLI (external) OR reqwest (REST fallback) + GITHUB_TOKEN env var
reachable_from:
  - elai session merge --to-pr
  - elai_cc.toml [github.enabled]
stub_ban:
  - GhCliAdapter MUST detect missing gh binary and emit structured SSB row,
    NOT silently fall through to RestApi (fallback is opt-in)
  - RestApiAdapter MUST refuse to operate without GITHUB_TOKEN; never embed
    a placeholder or hardcoded token
  - ReviewCommentRouter default = false; flipping to true requires explicit
    config change AND a confirmation prompt on first session that hits it
  - PR body MUST include the honesty section verbatim — no marketing text
frontend_pair: GithubPanel.svelte (PLANNED — implement after Rust types)
observable_effect:
  trigger: elai session merge --to-pr --watch-reviews --base main
  expect_ssb: github_pr_created + github_pr_body_posted on PR open;
              github_review_poll_tick every review_poll_interval_s while
              session is active; github_comment_received when reviewer
              comments
  expect_artifact: PR URL printed to stdout + dashboard GithubPanel shows
                   live PR state

## DETERMINISTIC_CHECK

```bash
# 1. crates/orchestrator/src/github_pr/ exists and is feature-gated
grep 'features.*github_pr' .elai_cc/Cargo.toml

# 2. All 6 adapter types present
rg "GithubAdapter|GhCliAdapter|RestApiAdapter|PrBodyComposer|ReviewCommentWatcher|ReviewCommentRouter" .elai_cc/crates/orchestrator/src/github_pr

# 3. 6 SSB action types
rg "github_pr_created|github_pr_body_posted|github_comment_received|github_comment_routed_to_session|github_adapter_error|github_review_poll_tick" .elai_cc/crates/orchestrator/src

# 4. CLI flags
rg "to-pr|watch-reviews" .elai_cc/crates/cli/src/main.rs

# 5. TDD tests pass
cargo test --features github_pr -p elai-orchestrator -- 2>&1 | grep "test result.*ok"

# 6. Contract test passes
cargo test --features github_pr -- github_pr_contract 2>&1 | grep "test result.*ok"

# 7. gh-mock fixture present
test -f .elai_cc/crates/orchestrator/src/github_pr/tests/fixtures/gh_mock.sh

# 8. SCHEMA_VERSION bumped
grep "SCHEMA_VERSION" .elai_cc/crates/orchestrator/src/ssb/schema.rs | head -1
```

## ACCEPTANCE

- [ ] elai_cc.toml [github] block validates at startup
- [ ] crates/orchestrator/src/github_pr/ compiles only with feature="github_pr";
      default build clean
- [ ] All 6 adapter types implemented with no placeholder bodies
- [ ] GhCliAdapter parses gh --json output correctly (TDD 4A.X.4)
- [ ] RestApiAdapter requires GITHUB_TOKEN; never hardcoded
- [ ] PrBodyComposer renders deterministic body from fixture
- [ ] PR body includes honesty section verbatim (mutation score, wiring,
      stub status, reviewer-instruction line)
- [ ] ReviewCommentWatcher emits SSB on detection
- [ ] ReviewCommentRouter default off; flip requires explicit config
- [ ] CLI flags --to-pr, --base, --watch-reviews, --draft wired
- [ ] All 6 SSB action types emitted with non-null fields
- [ ] SCHEMA_VERSION bumped; migration tested
- [ ] TDD tests 4A.X.3..4A.X.7 written and passing BEFORE wiring
- [ ] Contract tests pass
- [ ] End-to-end smoke against a real GitHub repo (separate workflow test
      gated behind GITHUB_TOKEN secret) successfully opens, comments on,
      and closes a draft PR
- [ ] GithubPanel.svelte renders live state (PLANNED — separate follow-up
      OK if Rust side complete)

## Files

- ADD: .elai_cc/crates/orchestrator/src/github_pr/mod.rs (feature-gated)
- ADD: .elai_cc/crates/orchestrator/src/github_pr/adapter_trait.rs (GithubAdapter)
- ADD: .elai_cc/crates/orchestrator/src/github_pr/gh_cli.rs
- ADD: .elai_cc/crates/orchestrator/src/github_pr/rest_api.rs
- ADD: .elai_cc/crates/orchestrator/src/github_pr/body_composer.rs (PrBodyComposer)
- ADD: .elai_cc/crates/orchestrator/src/github_pr/comment_watcher.rs
- ADD: .elai_cc/crates/orchestrator/src/github_pr/router.rs
- ADD: .elai_cc/crates/cli/src/github_pr_cmd.rs (CLI dispatcher entry)
- ADD: .elai_cc/crates/contracts/tests/github_pr_contract.rs
- ADD: .elai_cc/crates/orchestrator/src/github_pr/tests/fixtures/gh_mock.sh
- ADD: .elai_cc/templates/pr_body_default.md
- ADD: .elai_cc/crates/dashboard-v2/src/lib/components/GithubPanel.svelte (PLANNED)
- MODIFY: .elai_cc/crates/orchestrator/Cargo.toml (feature = "github_pr")
- MODIFY: .elai_cc/crates/cli/src/main.rs (--to-pr, --base, --watch-reviews, --draft flags)
- MODIFY: .elai_cc/crates/orchestrator/src/session/merge.rs (PR-trigger path)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb/schema.rs (6 new action types + SCHEMA_VERSION bump)
- MODIFY: .elai_cc/elai_cc.toml ([github] section)
- MODIFY: .elai_cc/docs/cli.md (--to-pr usage)

## Affected Verification Reports

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\C-plan\C-13_ipc_dispatcher_production_swap.report.md
  — annotate the upstream PR surface arrival.
- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\C-plan\C-23_sessiondag_branch_ui.report.md
  — annotate the consumer arrival (PrBodyComposer reads branch summary).

## Next

C-40 closure lets ELAI complete its loop on hirer-side workflows where the
human reviewer is upstream. Especially valuable for: Emil-style gig where
the hirer's GitHub repo is the review surface; OSS contribution flow; team
review workflows. C-40 does NOT enable automatic merges — review-comment
auto-response defaults off, and the PR remains draft unless the user marks
ready-for-review manually. ELAI is the author, not the maintainer.
