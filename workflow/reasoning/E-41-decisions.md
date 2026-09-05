E-41 ci_cd_pr_review — completion-drive decisions

Session: s-25120 (session 2 of 4 in concurrent cluster; sibling sessions
s-31708 was active on parallel E-plans at edit time).
Plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-41_ci_cd_pr_review.md
Edit window: 2026-05-25 (single completion-drive).

Scope shipped (vs. plan)
--------------------------
Plan claimed: 22 missing artifacts across 9 task buckets. After session
brief re-scope (line-by-line PLAN REBASE), the implementation surface
landed as:

  A. github_pr extension modules (added under
     .elai_cc/crates/orchestrator/src/github_pr/):
       - body_composer.rs   — Jinja-style {{key}} substitution, pure stdlib
       - review.rs          — post_review_comment (gh pr comment / gh pr review)
       - comment_watcher.rs — watch_comments (gh pr view --json comments poll)
       - package_verify.rs  — slopsquat scan over gh pr diff
       - git_assist.rs      — suggest_squash_message + detect_force_push
       - semgrep_gate.rs    — semgrep --config <ruleset> --json wrapper
       - auto_merge_guard.rs- classify + apply_branch_protection
       - codeowners_gen.rs  — deterministic CODEOWNERS body generator
       - clock.rs           — shared wall-clock helper (saturating)

  B. SSB variants (5, atomic SCHEMA bump 134 -> 135):
       - PrReviewComposed       { pr_number_hash, comment_kind_hash, ts_ms }
       - PrCommentWatched       { pr_number_hash, new_comments_count, ts_ms }
       - PrPackageVerifyResult  { pr_number_hash, slopsquat_count, ts_ms }
       - PrSemgrepGateResult    { pr_number_hash, findings_count,
                                  severity_max_hash, ts_ms }
       - PrAutoMergeBlocked     { pr_number_hash, block_reason_hash, ts_ms }
     9-tuple cascade x5 single-commit:
       - SsbActionType enum variant
       - as_sql_str arm
       - ssb_action_type_from_str_public arm
       - SQL allowlist CHECK row
       - SsbPayload enum variant
       - action_type() match arm
       - test_payload_for() match arm in test helper
       - all_test_payloads() Vec entry
       - action_type_lock.rs (ALL_VARIANTS + variant_tests + TOTAL_VARIANTS)
     Plus golden file regen: action_type_lock.golden 381 -> 386 entries
     (alphabetical order; 5 pr_* entries inserted between ppm_unavailable
     and preview_available).

  C. CLI surface:
       - crates/cli/src/pr_review_cmd.rs (NEW) — async + sync dispatch
         for 10 subverbs (compose-body / comment / watch / verify-deps /
         semgrep-gate / auto-merge-assess / apply-branch-protection /
         gen-codeowners / suggest-squash / detect-force-push)
       - args.rs CliAction::PrReview + parse_pr_review_subcommand_args
       - main.rs mod declaration + dispatch arm + build_pr_review_action
         helper that maps the flat args layer to the structured
         PrReviewAction enum.

  D. Dashboard:
       - dashboard-v2/src-tauri/src/commands_e41.rs (NEW) — single
         read-only `pr_review_status` Tauri command that aggregates the
         5 new SSB variants into one DTO.
       - lib.rs invoke_handler! append + module registration.
       - api/stub.ts DTOs (5 new) + pr_review_status fn + mock fallback.
       - state/github.svelte.ts state extension (5 new fields +
         refreshPrReviewStatus action).
       - windows/GithubWindow.svelte PR Review section (5 cards: recent
         reviews / watcher tick / slopsquat / semgrep / auto-merge guard)
         + matching CSS, with onMount() invoking refreshPrReviewStatus.

  E. CI yamls (.elai_cc/.github/workflows/):
       - elai-pr-review.yml         — fires on pull_request {opened,
         synchronize, reopened}; spawns 4 ELAI hooks (compose-body /
         verify-deps / semgrep-gate / auto-merge-assess). Soft-fail on
         semgrep + compose-body for now (per Open Question 3 sizing).
       - elai-anti-slopsquat.yml    — scoped to pull_request paths
         Cargo.toml / package.json / pyproject.toml / requirements.txt;
         single verify-deps invocation, BLOCKING on known squat.

  F. Contract test:
       - crates/contracts/tests/e41_ci_cd_pr_review_contract.rs (NEW),
         8 invariants: 5 wire-tag round-trips + SCHEMA floor + 7-function
         shape check + AgentRole cascade-avoidance + payload serde +
         WIRE-DARK CLI subverb anchor grep + anti-deps (no reqwest /
         octocrab / async_trait) + CI yaml on-disk presence.

  G. mistakes.md cluster entry (7th DEFERRAL cluster).

  H. This decisions document.

Cascade avoidance (per session brief)
-------------------------------------

The original E-41 plan §1 §2 added AgentRole::ExternalPrReviewer as the
18th variant. The session-3 brief explicitly forbade this — the cascade
costs ~10 downstream sites (agent_pool x3 + bsp/engine + ipc_gateway +
routing/selective + per_role_runtime + dashboard commands + c45 contract
+ reasoning_surface_contract) and would step on session-3's concurrent
edits.

Resolution: E-41 ships the reviewer surface as ORCHESTRATOR-level
helpers (review::post_review_comment + auto_merge_guard::* + etc) that
already inherit B-37's per-role-runtime routing for any spawned-agent
context. The heterogeneous-reviewer experiment (workflow/audits/E-41-
cross-family-agreement) does not need a separate AgentRole — it dispatches
the existing Reviewer role on a different RuntimeBackend at run-time,
which B-37's PerRoleRuntime::set() already supports.

Anti-deps (per CLAUDE.md HARD RULE)
-----------------------------------

Plan §1 mentioned a RestApiAdapter using reqwest. Session brief
forbade new HTTP deps. Resolution: every gh-side action shells out via
`gh` CLI (already on PATH, already classified RiskLevel::SharedState by
tools::action_classifier). REST-shape calls (apply_branch_protection)
use `gh api --method PUT` passthrough. No reqwest / octocrab / async_trait
landed; contract test (7) locks this.

Cite-discipline (per memory feedback_paper_citation_verify)
-----------------------------------------------------------

Plan cites arXiv:2507.02778 (Self-Correction Bench) + arXiv:2603.26130
(SWE-PRBench). Per cite-discipline both IDs need a WebSearch pass before
being quoted with effect-size + page anchor. 2603.26130 is structurally
suspect (arXiv 2026 IDs use YYMM.NNNNN format; 2603.* is unusual). Both
cites are DEFERRED for this session — implementation surface is grounded
against the codebase (sibling C-40 + per_role_runtime + action_classifier)
not against the papers. See mistakes.md 7th cluster entry for the full
DEFERRAL rationale + mechanical follow-on.

The cross-family heterogeneous reviewer EXPERIMENT (the audit artifact
workflow/audits/E-41-cross-family-agreement-2026-05-19.md, N>=20 PR
comparisons against Greptile/CodeRabbit free-tier) is RUNPOD-PENDING.
That artifact is the natural home for the quoted effect-size + cite
anchor once the experiment runs on a real corpus.

WIRE-DARK avoidance
-------------------

E37_WIRE_DARK_GAP_PATTERN documented in s-25120 reasoning: shipping
library types + functions without production call sites is an
anti-pattern. E-41 anchors every new module via the
`crates/cli/src/pr_review_cmd.rs` dispatcher (single CLI subverb per
module function). The contract test (6) greps the pr_review_cmd.rs
source for each module function name; absence fails the build.

Concurrent SCHEMA race resolution
---------------------------------

Session-3 active at edit time. Read ssb.rs:1611 immediately before
write: SCHEMA_VERSION=134; bumped to 135 atomic with the 5 variant
landings. TOTAL_VARIANTS 381 -> 386. action_type_lock.golden regen done
in-line (5 new alphabetically-sorted entries inserted between
ppm_unavailable and preview_available; ordinals 0250-0254; all
subsequent ordinals shifted +5).

CI gate posture
---------------

Plan §3 / §8 / §risks: server-side enforcement is GitHub branch-protection,
not just a YAML workflow. E-41 ships both:
  - the 2 yamls (elai-pr-review.yml + elai-anti-slopsquat.yml) so the
    PR-open path runs the hooks
  - the one-shot `elai pr-review apply-branch-protection --repo
    owner/name --branch main` CLI subverb that calls `gh api PUT
    /repos/.../branches/.../protection` with the ELAI-recommended payload
    (required_pull_request_reviews + dismiss_stale_reviews + required_
    linear_history + allow_force_pushes=false).

Semgrep soft-fail
-----------------

Plan §3 says "soft-mode for the first 10 PRs" to size the FP rate.
Implementation: elai-pr-review.yml's semgrep-gate step ends in `|| true`,
so the step does not fail the workflow regardless of finding count. The
auto-merge-assess step is BLOCKING (its exit code drives the workflow's
final status), and the auto_merge_guard classifier blocks any PR that
fails the size / multi-file / owned-paths checks. After 10 real PRs land,
the semgrep step graduates to BLOCKING by dropping the `|| true` suffix.

What did NOT ship (deferred)
----------------------------

  - The 18th AgentRole variant (cascade avoidance per brief).
  - The cross-family-agreement audit artifact (workflow/audits/E-41-...md
    with N>=20 PR comparisons) — needs real PR corpus + Greptile /
    CodeRabbit free-tier comparisons + sidecar GPU time. RUNPOD-PENDING.
  - The paper cite verification (arXiv:2507.02778 + arXiv:2603.26130) —
    DEFERRED per 7th cluster entry in mistakes.md.
  - PrStatusPanel.svelte as a standalone window (kept the PR Review
    section inside GithubWindow per the plan's "OPTIONAL CREATE" note
    + the session brief's avoid-VALID_WINDOW_TYPES-cascade guidance).
  - The flaky-test quarantine extension to .claude/tools/failure_taxonomy.py
    (plan §9) — out of scope for this completion-drive; can land in a
    follow-up patch-N when the auditor pass actually surfaces a flaky
    test on the E-41 lane.
  - Git trailer for AI attribution (plan §6) — every Generated-By /
    Reviewed-By-Bot trailer remains out-of-band; not landed in
    git_assist.rs because the user-facing surface does not need it for
    the wire-shape contract (commit-cadence.md HARD RULE: no Co-Authored-By
    in subject/body; trailers are an OPTIONAL nice-to-have, not a
    blocker).

Build expectation
-----------------

`cargo check -p elai-orchestrator --workspace` should pass after these
edits. `cargo test -p elai-orchestrator --test action_type_lock` should
pass with TOTAL_VARIANTS=386. `cargo test -p elai_cc_contracts --test
e41_ci_cd_pr_review_contract` should pass on all 8 invariants. Live
`elai pr-review` calls require gh on PATH + authenticated; no live
exercise in this completion-drive (RUNPOD-PENDING).
