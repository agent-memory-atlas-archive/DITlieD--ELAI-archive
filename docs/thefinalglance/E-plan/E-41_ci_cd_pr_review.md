# E-41: 4-leg CI/CD defense + heterogeneous PR reviewer + GitHub Adapter wiring
Objective: stand up the four-leg defense pattern for AI-built code (in-flow auditor / PR open / deploy preview / post-merge observability), wire a heterogeneous external reviewer call (different model family than the author) into the PR-open gate, and ship the GitHub Adapter surface so ELAI can open/comment/respond to PR reviews end-to-end. The win is bounded false-positive review noise + measurable change-failure-rate guard against AI-co-authored PR risk.
Estimated effort: 3 sessions, ~10-12 h
Pre-requisites: C-40 PLANNED (GitHub PR Adapter — Devin parity), C-23 SHIPPED (per C-SUMMARY_INDEX line 635 pre-req chain), B-37 SHIPPED (PerRoleRuntime for cross-family routing), B-9 SHIPPED (SessionDag for branch-per-plan)
Parallel-safe with: E-42 (grammars — orthogonal), E-44 (durability — different surface), E-45 (drift gate — feeds the PR CI but separate hook), C-40 (this E-N supersedes/grounds C-40's gh adapter scope), E-22 (skill export — different lane)
Source report: workflow/improvement-sweep-2026-05-19/reports/41-ci-cd-pr-review.md

## Context

The May 2026 CI/CD report documents a sharp risk delta for AI-co-authored PRs: 1.7x more issues per PR (CodeRabbit state of AI vs human), incident-to-PR ratio up 242.7% at high AI adoption, change failure rate +30%, incidents/PR +23.5% YoY (Faros AI Engineering Report 2026). Self-Correction Bench (arXiv:2507.02778) measures a 64.5% blind-spot rate when an LLM reviews its own family's output; SWE-PRBench (arXiv:2603.26130) shows 8 frontier models detect only 15-31% of human-flagged issues diff-only. ELAI's current loop is single-family: the auditor agent reviews the executor's work using the same model spine, so an in-flow PASS does not constitute meaningful external review.

Industry has converged on a 4-leg defense (cited: github/awesome-copilot CI best practices, semgrep.dev 2026, augmentcode AI verification, testdino flaky 2026, incident.io AI SRE 2026): (1) pre-commit lint/type/forbidden-pattern; (2) PR open SAST + slopsquat/SCA scan + heterogeneous AI reviewer; (3) deploy preview with VLM judge for UI-touching changes (Cosmos worker territory — already shipped per B-42); (4) post-merge canary + AI-SRE watch with auto-rollback. Greptile's Mar 2026 independent eval (dev.to/_vjk) hit 0 FP across 120 findings on 146 PRs / 679 findings; CodeRabbit at ~15% FP; Qodo Merge multi-agent (Feb 2026) at F1 60.1%. Self-hosted PR-Agent (Qodo OSS) avoids the SaaS bill for an org-of-one.

The cheapest external reviewer for ELAI is its own remote sidecar (Qwen2.5-Coder-3B-Instruct-abliterated) reviewing PRs authored by cloud Sonnet/Opus, and vice versa — cross-family without paid SaaS. This is the report's Open Question 4 ("can the local sidecar serve as the heterogeneous reviewer") and the answer is "measure it." E-41 ships the harness that lets the user run the experiment.

The auto-merge anti-pattern is explicit: AI-co-authored PRs auto-merging on green CI is the documented failure mode at scale (kenimo49 dev.to 2026 on squash, Univ. Waterloo early 2026 on 1-in-4 flawed output, Augment Code on 4.6x review pickup and 32.7% AI-PR acceptance). ELAI must require at least one human approval on multi-file PRs and an external-reviewer PASS, with stale-review dismissal on new pushes and linear history.

## Existing state (B/C/D coverage)

- C-40 PLANNED (GitHub PR Adapter — Devin Parity), per C-SUMMARY_INDEX lines 633-638: adds `crates/github_adapter/` (feature-gated), `GhCliAdapter` + `RestApiAdapter`, `PrBodyComposer` (with honesty section), `ReviewCommentWatcher` + `Router` (default OFF auto-respond), `--to-pr/--base/--watch-reviews/--draft` flags, 6 `github_*` SSB action types, end-to-end draft PR smoke. E-41 EXTENDS C-40: same adapter surface, but adds the heterogeneous-reviewer hook + Semgrep/SCA gating + per-baby-step commit policy + git-trailer attribution; C-40 remains the pre-req for the adapter primitive.
- C-39 SHIPPED-or-IN-FLIGHT (MCP Bridge CLI Surface) per C-SUMMARY_INDEX line 626-631: the `--via-mcp` flag and MCPClient/Dispatch surface are the right substrate to call out to external reviewers if Greptile/CodeRabbit/Qodo expose MCP endpoints. EXTENSION: E-41 reuses this transport rather than building a second HTTP client.
- B-37 SHIPPED (Multi-Runtime Backend Abstraction — Phase 4M, commit fd5c2e5 per memory project_b37_shipped.md): 11 backends + RuntimeAdapter + PerRoleRuntime. EXTENSION: the heterogeneous-reviewer hook spawns a Reviewer role on a different backend than the Author role; B-37's routing layer already supports per-role backend selection, so E-41 only adds a new `AgentRole::ExternalPrReviewer` and routes it.
- B-9 SHIPPED (SessionDag Activation, s-59964): branch-per-plan and 85% handoff exist. EXTENSION: E-41 adds the per-baby-step commit policy (`git merge --no-ff` at plan close preserving bisect) on top of the existing branch infrastructure; no SessionDag schema change.
- B-26 PARTIAL→DONE (dashboard completion): the PR-status panel would land here; E-41 lists it as an OPTIONAL `frontend_pair` not a blocker.
- The CLAUDE.md commit cadence rule (`.claude/rules/commit-cadence.md`) already prohibits per-phase commits and AI attribution trailers in subjects/bodies. E-41 honors both: git trailers stay machine-readable (`Generated-By:`, `Reviewed-By-Bot:`) and out of the subject line per maintainer voice.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\41-ci-cd-pr-review.md` (full)
- `F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-SUMMARY_INDEX.md` lines 620-660 (C-39 + C-40 + C-42)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 215-228 (B-9 SessionDag), lines 780-790 (B-37)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs` (existing SSB action types — confirm naming convention for new `github_*` rows from C-40)
- `F:\.ELAI_workflow\.claude\rules\commit-cadence.md` (maintainer voice + one-changeset-per-task rule)
- WebFetch: docs.github.com branch-protection (CODEOWNERS, stale-review dismissal, linear history)
- WebFetch: github.com/Codium-ai/pr-agent (Qodo PR-Agent OSS self-host)
- WebFetch: semgrep.dev/blog/2026/operationalizing-ai-powered-detection (AI ruleset)
- WebFetch: arXiv:2507.02778 (Self-Correction Bench — verify ID before citing per memory feedback_paper_citation_verify)
- WebFetch: arXiv:2603.26130 (SWE-PRBench — verify ID)

## Tasks

1. GitHub Adapter primitive (consumes C-40 — coordinate with that phase). If C-40 is not yet in flight, E-41 lands the adapter surface itself: new crate `crates/github_adapter/` (feature-gated) with `GhCliAdapter` (shells out to the `gh` CLI; verify version + auth at startup with `gh auth status` and emit `github_adapter_auth_ok` SSB row or fail with actionable error) and `RestApiAdapter` (reqwest + token from `GH_TOKEN` env). Surface: `open_pr(base, head, title, body, draft) -> PrNumber`, `post_review_comment(pr, path, line, body)`, `list_review_comments(pr) -> Vec<ReviewComment>`, `set_pr_status(pr, state)`. Anti-slopsquatting: confirm `gh` CLI is the official GitHub CLI from cli.github.com before any shell-out path.

2. Heterogeneous external reviewer hook. Add `AgentRole::ExternalPrReviewer` and route it through B-37's PerRoleRuntime to a DIFFERENT backend than `AgentRole::Executor` was running on (e.g., Executor=cloud Sonnet, Reviewer=local Qwen2.5-Coder-3B; or Executor=local, Reviewer=cloud). Reviewer gets the diff + the brief + the auditor's verdict; emits `pr_external_review_started` and `pr_external_review_verdict` SSB rows with score ∈ {PASS, BLOCK, REQUEST_CHANGES} and a structured comment list. The report's Open Question 4 is the validation target — output `workflow/audits/E-41-cross-family-agreement-2026-05-19.md` with N≥20 PR comparisons against Greptile/CodeRabbit free tier on a public corpus to size baseline κ. Cited grounding: arXiv:2507.02778 (Self-Correction Bench 64.5% same-family blind-spot) and arXiv:2603.26130 (SWE-PRBench 15-31% diff-only detection).

3. Semgrep AI-ruleset PR CI stage. Add a Semgrep GH Action invocation (or local equivalent via `semgrep --config p/ai-anti-patterns`) that runs after the auditor's in-flow PASS, before the external reviewer. Fails the PR on banned tokens already in CLAUDE.md (`.unwrap()` in non-test, force-unwrap, hardcoded secrets, missing safety settings on LLM calls). Reference: semgrep.dev/blog/2026/operationalizing-ai-powered-detection (8x TP, 50% less noise, 100+ AI anti-pattern rules). Emit `semgrep_ai_findings` SSB row with finding count by severity. Soft-mode for the first 10 PRs to size the FP rate per report Open Question 3 (retroactive dry-run on last 100 commits).

4. Dependency-add as privileged tool call. Hook into the executor's edit-path: when a Write/Edit adds an import line matching `^(use |import |from |require\()` AND the package isn't in the existing dependency manifest (Cargo.toml / pyproject.toml / package.json), call `verify_package(name, ecosystem)` before the write commits. `verify_package` queries the registry (`cargo search` for crates.io, `pypi.org/pypi/{pkg}/json` for pypi, `npm view {pkg} version dist-tags time` for npm) and rejects on (a) name not found, (b) no release in 18 months, (c) author/owner mismatch heuristics. Already required by CLAUDE.md Anti-Slopsquatting HARD RULE — E-41 makes it an enforced hook, not a policy line. Cited: snyk.io/articles/slopsquatting-mitigation-strategies (~20% AI-suggested pkg hallucinated). SSB row: `dependency_verification_result`.

5. Per-baby-step commit + plan-close merge policy. The branch-per-plan exists already via B-9 SessionDag; add a commit-cadence assistant in `crates/orchestrator/src/git_assist.rs` (NEW) that (a) at WIP cleanup squashes intra-baby-step `fix`/`WIP` commits, (b) preserves baby-step boundaries — one commit per baby step labeled `<plan-id>/<phase>/<step>`, (c) at plan-close uses `git merge --no-ff` into main so `git bisect` can isolate failing steps. Cited: kenimo49 dev.to 2026 ("AI PRs 3-10x bigger destroys bisect under squash"). Respect `.claude/rules/commit-cadence.md` HARD RULE: ONE commit-equivalent unit per plan at the merge boundary; the per-baby-step commits feed bisect, the merge commit is the user-visible task close.

6. Git trailer for AI attribution (machine-readable only, NEVER in subject/body). Append `Generated-By: elai/<author-role>/<model-id>` and `Reviewed-By-Bot: <reviewer-role>/<model-id>` as git trailers on the merge commit. CLAUDE.md HARD RULE bans Co-Authored-By and AI attribution in subject/body — trailers are out-of-band, machine-readable, and explicitly cited as the right path in the report (deployhq.com/git/committing-ai-generated-code). Subjects stay maintainer voice per `.claude/rules/commit-cadence.md`.

7. Branch protection + CODEOWNERS auto-generate. At plan-bootstrap, scan the planned `files touched` and generate a `.github/CODEOWNERS` (or update the existing one) mapping touched paths to the human owner per `projectBrief.md`. Generated CODEOWNERS is gitignored by default; the user opts in by `elai pr config --enforce-codeowners`. Emit `codeowners_generated` SSB row. Cited grounding: docs.github.com branch protection + Augment Code "contract violations sail through diff review at agent velocity."

8. Auto-merge guard. Disable auto-merge entirely for multi-file or >100-LOC PRs. The exception ladder: auto-merge OK only when (a) machine-generated and reversible (Dependabot patch + green CI), (b) <100 LOC delta, (c) all touched paths are unowned per CODEOWNERS. Otherwise emit `auto_merge_blocked_reason` SSB and require human approval. Cited grounding: Faros 2026 + Univ. Waterloo 2026 + Self-Correction Bench.

9. Flaky-test quarantine policy. The auditor already classifies test failures; extend its taxonomy to {bug, flaky, env} per `.claude/tools/failure_taxonomy.py`. Flaky tests move to a separate `tests/quarantine/` suite with a `will_resolve_in` baby-step reference (mirrors stub registry); 2-sprint TTL after which the flake re-blocks merge. Cited: testdino.com/blog/flaky-test-detection-tools (named owner + removal date) and functionize.com (>70% flakes are timing/data not selector).

## WIRING_CONTRACT

produces_symbols:
  - github_adapter::GhCliAdapter::open_pr(base: &str, head: &str, title: &str, body: &str, draft: bool) -> Result<PrNumber>
  - github_adapter::GhCliAdapter::post_review_comment(pr: PrNumber, path: &str, line: u32, body: &str) -> Result<()>
  - github_adapter::RestApiAdapter::set_pr_status(pr: PrNumber, state: PrStatus) -> Result<()>
  - orchestrator::AgentRole::ExternalPrReviewer
  - orchestrator::ssb::SsbActionType::PrExternalReviewStarted
  - orchestrator::ssb::SsbActionType::PrExternalReviewVerdict
  - orchestrator::ssb::SsbActionType::SemgrepAiFindings
  - orchestrator::ssb::SsbActionType::DependencyVerificationResult
  - orchestrator::ssb::SsbActionType::CodeownersGenerated
  - orchestrator::ssb::SsbActionType::AutoMergeBlockedReason
  - orchestrator::ssb::SsbActionType::GithubAdapterAuthOk
  - orchestrator::git_assist::compose_plan_merge(plan_id: &str) -> Result<MergeCommit>
  - cli::pr::open|close|review-comment|set-status (new subcommands)
consumes_symbols:
  - B-37 PerRoleRuntime::route_role(role: AgentRole) -> BackendHandle (for ExternalPrReviewer)
  - C-39 MCPClient (optional transport for Greptile/Qodo if they ship MCP)
  - .claude/tools/failure_taxonomy.py (flaky classifier)
  - .claude/rules/commit-cadence.md (subject/body/trailer constraints)
reachable_from:
  - crates/cli/src/main.rs (new `pr` subcommand tree)
  - crates/orchestrator/src/task_runner.rs (at plan-close, invokes git_assist::compose_plan_merge then github_adapter::open_pr)
  - crates/orchestrator/src/auditor.rs (after PASS, dispatches AgentRole::ExternalPrReviewer)
stub_ban:
  - no Co-Authored-By or AI attribution in commit subject/body (HARD RULE, enforced in git_assist tests)
  - no .unwrap() in github_adapter (production code; Result/Option propagation)
  - no silent slopsquat: every new dep import path goes through verify_package
  - no auto-merge in default config for multi-file PRs (test that auto_merge_blocked_reason fires)
  - no same-family ExternalPrReviewer (test asserts backend != Executor's backend)
frontend_pair: crates/dashboard-v2/src/lib/windows/PrStatusPanel.svelte (NEW, optional — show PR number, external reviewer verdict, dependency check, Semgrep findings; gated behind B-26 dashboard completion)
observable_effect:
  trigger: "elai pr open --base main --head plan/E-41 --draft" on a plan that has passed in-flow audit
  expect_ssb: "github_adapter_auth_ok" then "pr_external_review_started" then "pr_external_review_verdict" (PASS|BLOCK) — verifies the full chain
  expect_artifact: A draft PR exists on the configured GitHub repo, body composed per PrBodyComposer with mutation score + wiring contract + stub status + reviewer-instruction line, AND a "Generated-By:" git trailer on the merge commit (NOT in subject)

## DETERMINISTIC_CHECK

```bash
# 1. GitHub adapter crate exists and exports the 4 primitives
rg "pub fn open_pr|pub fn post_review_comment|pub fn list_review_comments|pub fn set_pr_status" .elai_cc/crates/github_adapter --type rust
# Expected: ≥4 hits

# 2. ExternalPrReviewer role exists and is routed
rg "ExternalPrReviewer" .elai_cc/crates/orchestrator --type rust
# Expected: ≥2 hits (enum variant + routing match arm)

# 3. New SSB action types are registered
rg "PrExternalReviewStarted|PrExternalReviewVerdict|SemgrepAiFindings|DependencyVerificationResult|CodeownersGenerated|AutoMergeBlockedReason|GithubAdapterAuthOk" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: ≥7 hits (one per variant)

# 4. Verify-package hook present on edit path
rg "verify_package" .elai_cc/crates/orchestrator/src --type rust
# Expected: ≥1 hit (callsite) + ≥1 hit (impl)

# 5. git_assist compose_plan_merge present, --no-ff enforced
rg "compose_plan_merge|--no-ff|no_ff" .elai_cc/crates/orchestrator/src/git_assist.rs
# Expected: function def + literal "--no-ff" arg

# 6. Co-Authored-By / AI attribution NOT in commit composers
! rg "Co-Authored-By|Generated-By:" .elai_cc/crates/orchestrator/src/git_assist.rs --type rust | rg -v 'trailer|test'
# Expected: only trailer-related references in tests; never in subject/body composers

# 7. Cross-family reviewer test asserts backend mismatch
rg "assert.*backend.*!=|assert_ne!.*backend" .elai_cc/crates/orchestrator/tests/cross_family_reviewer.rs
# Expected: ≥1 hit

# 8. Audit artifact present
test -f workflow/audits/E-41-cross-family-agreement-2026-05-19.md
# Expected: file exists with ≥20 rows
```

## ACCEPTANCE

- [ ] `crates/github_adapter/` compiles + tests green; feature-gated so default builds don't require `gh` CLI
- [ ] `elai pr open` round-trips against a sandbox GitHub repo (use a fixture / mock for CI; live test gated on `GH_TOKEN`)
- [ ] `AgentRole::ExternalPrReviewer` routes to a backend that is provably different from the Executor's backend (assert in test)
- [ ] Semgrep AI ruleset runs on PR open; emits `semgrep_ai_findings`; soft-fail for first 10 PRs (count, don't block)
- [ ] `verify_package` blocks an obviously-hallucinated package name in a fixture test (e.g., `npm install reqquests`)
- [ ] Per-baby-step commits preserved at plan-close merge; `git bisect run cargo test -p contracts` over the merge commit can isolate a failing step
- [ ] Commit subject + body contain NO AI attribution (regex check in test); `Generated-By:` and `Reviewed-By-Bot:` trailers present on merge commit only
- [ ] CODEOWNERS auto-generated to `.github/CODEOWNERS` (gitignored by default; opt-in flag emits the real file)
- [ ] Auto-merge disabled for any PR >100 LOC OR touching ≥2 files OR with owned paths; SSB row records the block reason
- [ ] Flaky-test quarantine path: a deterministic flaky-injection test moves to `tests/quarantine/` with a `will_resolve_in` marker

## Tests to add

- `crates/github_adapter/tests/open_pr_dry_run.rs`: `--dry-run` constructs the right `gh pr create` shell-out
- `crates/github_adapter/tests/post_comment_payload.rs`: REST adapter assembles the right JSON body for `/repos/{owner}/{repo}/pulls/{n}/comments`
- `crates/orchestrator/tests/cross_family_reviewer.rs`: ExternalPrReviewer routes to non-Executor backend
- `crates/orchestrator/tests/verify_package_slopsquat.rs`: hallucinated package name rejected on all 3 ecosystems
- `crates/orchestrator/tests/git_assist_no_attribution.rs`: composed commit subject + body contain no banned attribution strings
- `crates/orchestrator/tests/git_assist_bisect.rs`: merge commit preserves baby-step commits accessible to `git bisect`
- `crates/orchestrator/tests/auto_merge_guard.rs`: 101-LOC fixture PR blocks auto-merge; 50-LOC single-file unowned PR allows it
- `crates/cli/tests/pr_subcommand_dry_run.rs`: `elai pr open --dry-run` round-trip
- `tests/contracts/github_adapter_contract.rs`: feature flag respected; default build doesn't import `gh` CLI

## Files touched

- CREATE: `.elai_cc/crates/github_adapter/Cargo.toml` (~20 LOC)
- CREATE: `.elai_cc/crates/github_adapter/src/lib.rs` (~120 LOC — GhCliAdapter + RestApiAdapter + PrBodyComposer)
- CREATE: `.elai_cc/crates/github_adapter/src/review_watcher.rs` (~80 LOC — ReviewCommentWatcher loop, default OFF)
- CREATE: `.elai_cc/crates/orchestrator/src/git_assist.rs` (~150 LOC — compose_plan_merge + commit-cadence helpers)
- CREATE: `.elai_cc/crates/orchestrator/src/verify_package.rs` (~120 LOC — 3 ecosystem checkers)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+~60 LOC for 7 new action_type variants — bump SCHEMA_VERSION per memory project_b42_shipped.md trajectory)
- MODIFY: `.elai_cc/crates/orchestrator/src/agent_role.rs` (+1 variant `ExternalPrReviewer`, +1 line in routing)
- MODIFY: `.elai_cc/crates/orchestrator/src/auditor.rs` (+~30 LOC dispatch ExternalPrReviewer after PASS)
- MODIFY: `.elai_cc/crates/cli/src/main.rs` (+~40 LOC for `pr` subcommand tree)
- MODIFY: `.elai_cc/Cargo.toml` workspace members (+ github_adapter)
- CREATE: `workflow/audits/E-41-cross-family-agreement-2026-05-19.md`
- CREATE: `workflow/reasoning/E-41-decisions.md`
- OPTIONAL CREATE: `.elai_cc/crates/dashboard-v2/src/lib/windows/PrStatusPanel.svelte` (~120 LOC — gated behind B-26)

## Citations / Grounding

- arXiv:2507.02778 (Self-Correction Bench — 64.5% same-family blind-spot, "Wait" prefix cuts 89.3%) — grounds Task 2 cross-family requirement — VERIFY ID per memory feedback_paper_citation_verify
- arXiv:2603.26130 (SWE-PRBench, Mar 2026 — 8 frontier models 15-31% diff-only detection) — grounds Task 2 — VERIFY ID
- Faros AI Engineering Report 2026 (faros.ai/blog/ai-acceleration-whiplash-takeaways — incidents/PR +242.7% at high AI adoption) — grounds Task 8 auto-merge guard
- CodeRabbit state of AI vs human (coderabbit.ai/blog/state-of-ai-vs-human-code-generation-report — 1.7x more issues per AI PR) — grounds Task 8
- Univ. Waterloo early 2026 study (~25% AI output flawed) — grounds Task 8 — VERIFY via WebSearch before citing in body of merge commit
- snyk.io/articles/slopsquatting-mitigation-strategies (~20% AI-suggested pkg hallucinated, ~40% AI-imported deps vulnerable) — grounds Task 4
- semgrep.dev/blog/2026/operationalizing-ai-powered-detection (Mar 2026 — 8x TP, 50% less noise, 100+ AI rules) — grounds Task 3
- deployhq.com/git/committing-ai-generated-code — grounds Task 6 trailer-only attribution
- dev.to/kenimo49/why-just-squash-merge-no-longer-works-in-the-age-of-ai (AI PRs 3-10x bigger destroys bisect) — grounds Task 5 no-ff policy
- docs.github.com branch-protection — grounds Task 7 + Task 8 CODEOWNERS + stale-review dismissal
- github.com/Codium-ai/pr-agent (Qodo OSS) — fallback path if SaaS reviewer not selected — grounds Task 2 cost story
- C-40 PLANNED (C-SUMMARY_INDEX:633-638 — Devin Parity adapter) — grounds Task 1 surface

## Open questions / risks

- The retroactive Semgrep dry-run (report Open Question 3) needs to be sized first. If Semgrep would have blocked >20% of the last 100 commits, the soft-mode period needs extending to avoid false-positive fatigue burning out the gate.
- Cross-family reviewer agreement with Greptile/CodeRabbit (report Open Question 4) is the validation experiment. If the local 3B model κ < 0.5 against an industry reviewer on a 20-PR sample, ELAI cannot self-host the heterogeneous reviewer and must wire to a paid SaaS or a cloud Sonnet-as-Reviewer call. Outcome belongs in `workflow/audits/E-41-cross-family-agreement-2026-05-19.md`.
- Latency budget (report Open Question 2): if the external-reviewer round-trip adds >10 min to plan close, autonomous-mode users will disable it. Measure p50/p95 and set a `pr_external_review_timeout_ms` config knob.
- `gh` CLI is a shell-out; on Windows the default install path matters. Verify `gh` is on PATH at startup; fail with actionable error before any `pr` subcommand runs.
- C-40 overlap: if C-40 ships first, E-41 only adds Tasks 2-9 on top of its adapter primitive. Coordinate with the user — do not double-build the adapter.
- Bypass paths: anyone with write access can `git push --no-verify` past pre-commit hooks; the auto-merge guard must live in CI (server-side check), not just locally.

## Next

E-44 (durability) hardens the SAGA log that compensates an irreversible push — if E-41's `git push --force` ever lands (it shouldn't, per HARD RULES), E-44's compensation entry is the only safety net. E-45 (cross-language drift) feeds this PR CI: the `cargo test -p ts-bindings` + schema snapshot gate becomes a required PR status check before the heterogeneous reviewer is even invoked. C-40 should fold into this E-N as its surface is a strict superset; recommend deprecating C-40 as a separate planned phase once E-41 lands and noting the rollup in `B-PLAN-SUMMARY.md`.
