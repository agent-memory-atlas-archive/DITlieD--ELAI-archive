# K-11: Human-viewable evidence-artifact render — make ELAI's machine evidence reviewable in the PR

STATUS: PLANNED (WAVE KW-2; M; BUILD-renderer; EXTENSION(K-3/E-41/G-5/G-2) — render existing evidence, author no new evidence machinery)

Objective: ELAI's trust spine produces MACHINE-grade evidence (RRA bundle G-2, SSB rows, F-4 stdout_hash, coverage hits, G-5 DSR, H-52 anchored findings, the visual-ui-loop screenshots it ALREADY captures) but has NO human-viewable RENDER of that evidence attached to a PR/review. K-3 (PR/CI evidence emitter) renders per-hunk verdicts into GitHub check-runs + PR comments over the WIRED github_pr adapter; K-11 EXTENDS K-3 with a human-viewable EVIDENCE-ARTIFACT section — screenshots / logs / RRA-summary / test-evidence rendered into the PR body + the G-5 reviewer cockpit, risk-tiered by the G-5 DSR so a reviewer SEES the change met intent and reviews only by risk tier. The grounded gap is the steal-kunchen C2.2 EXTENSION(2): no-mistakes' TEST stage records artifacts[{path|url|content}] (screenshots/videos/logs) rendered into the PR body (## Testing / Evidence); the DEMAND side is the video's "eng-manager not diff-reviewer" thesis (low-risk diffs never hand-reviewed because the pipeline caught it). ELAI HAS the evidence; it lacks the human RENDER. HARD BOUNDARY: author NO new evidence machinery — render RRA/F-4/G-5/H-52/visual-ui-loop artifacts that already exist. GD-1: do NOT re-implement github_pr, RRA, G-5 DSR, or H-52 anchoring. KD-2 (meet-the-buyer-in-the-PR). Source: workflow/reasoning/steal-kunchen-agentic-tools-2026-06-24.md C2.2 + SYNTHESIS TIER-1(2).

Estimated effort: ~1 session (~M). In-process render at PR-emit time; GitHub API leg is network-gated (W-10 split: deterministic render fixture now, live check-run post separately gated).
Pre-requisites (re-grep live per W-7): K-3 PR/CI evidence emitter (the check-run + PR-comment path); E-41 github_pr — crates/orchestrator/src/github_pr/review.rs::{post_review_comment, build_review_argv, ReviewCommentKind} (codegraph-confirmed live); G-5 REVIEWER COCKPIT + DSR (review_surface, lines-needing-attention/total); G-2 RRA (rra.rs build_rra/verify_rra); F-4 TEST-INTEGRITY (stdout_hash on TestOutcomePayload); H-52 FINDING ANCHORING (review_anchor.rs verbatim-snippet resolver); visual-ui-loop (.claude/tools/visual/ screenshots, WIRED).
Parallel-safe with: K-12, J-80, J-81, J-82 (disjoint files).
Source: steal-kunchen C2.2/C2.3 + the META "demand-side" tie. Doctrine: K-0_INDEX.md (KD-1/KD-2/KD-5).

## Context
Why this matters: the market's supervision surface is the PR, not a bespoke cockpit (KD-2). A reviewer who can SEE the evidence (a screenshot proving the UI renders, a test log proving the path ran, the RRA digest proving reproducibility, the G-5 DSR proving how much actually needs attention) reviews by risk tier instead of reading every diff — the exact "AI-validated so the human only reviews risky diffs" workflow practitioners demonstrably want (steal-kunchen C3.3 META; AUTHOR claim, market-validation not an ELAI number). The mechanism is a render, not a model: assemble the evidence ELAI already journals into a deterministic human-viewable section. A hunk with NO backing artifact renders UNVOUCHED, never silently clean (the K-3 invariant, extended to the human surface). The cloud lane's reproducibility claim stays action-stream (GD-5), never "byte-identical tokens".

## Existing state (B/C/D/E/F/G/H/I coverage — W-7 anchors, executor re-greps)
- K-3 PR/CI evidence emitter [PLANNED] — the check-run + PR-comment renderer this EXTENDS (K-0_INDEX §"THE 9 SESSIONS").
- crates/orchestrator/src/github_pr/review.rs::{post_review_comment, build_review_argv, ReviewCommentKind} [WIRED] — the PR-comment poster (E-41 `elai pr-review`; C-40 RestApiAdapter deferred).
- G-5 REVIEWER COCKPIT + DSR [WIRED-static] — per-hunk proof tokens + lines-needing-attention/total; review_surface_scored never triggered live (the risk-tier signal K-11 renders).
- G-2 RRA [WIRED] — rra.rs portable msgpack+zstd bundle (the reproducible-run digest K-11 renders a human summary of).
- F-4 TEST-INTEGRITY [WIRED] — blake3 stdout_hash on TestOutcomePayload (the "this test actually ran" proof).
- H-52 FINDING ANCHORING [PARTIAL, uncommitted] — review_anchor.rs verbatim-snippet→hunk resolver (anchors the rendered findings).
- visual-ui-loop (.claude/tools/visual/) [WIRED] — ui-diff.mjs/ax-scaffold.mjs capture the screenshots K-11 attaches.

## Reads (mandatory before coding)
1. _grounding/K-GROUNDING-MAP.md (KC corrections override the brief); the K-3 detail file.
2. workflow/reasoning/steal-kunchen-agentic-tools-2026-06-24.md C2.2 + SYNTHESIS TIER-1(2) + the no-mistakes pipeline (test/pr stages) reference at docs/.source/no-mistakes.
3. crates/orchestrator/src/github_pr/review.rs (the poster to extend); the G-5 review_surface module; rra.rs (the bundle to summarize); review_anchor.rs.
4. F-0_INDEX W-1..W-10; G-0 GD-1..GD-5; K-0 KD-1/KD-2/KD-5; CLAUDE.md (no .unwrap on prod, blake3-only, worker-git-read-only).

## Tasks (WAVE KW-2 — render evidence into the PR/cockpit)
1. EvidenceArtifactSet assembler (~80 LOC): collect, for a changeset, the existing artifacts — RRA digest (G-2), F-4 outcome rows, G-5 DSR + vouched/unvouched bands, H-52 anchored findings, visual-ui-loop screenshot refs. PURE composition over journaled rows; NO new capture. A hunk with no artifact → UNVOUCHED entry (never omitted).
2. Human-viewable renderer (~70 LOC): emit a markdown ## Evidence section (RRA summary + per-test evidence links + collapsible logs + screenshot refs) into the github_pr PR-comment body via post_review_comment, risk-tiered by the G-5 DSR (high-attention hunks expand, low-risk collapse). DO NOT copy lavish's author-brand design bundle; DO NOT mutate git (worker-git-read-only).
3. Wire into the K-3 emit path (~20 LOC): K-3's check-run posting calls the K-11 renderer; the evidence section ships with the per-hunk verdicts in the SAME emit. `elai pr-review --emit-evidence` flag (default-ON behind ELAI_PR_EVIDENCE_RENDER_DISABLE per W-2).
4. workflow/reasoning/K-11-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/github_pr/evidence_render.rs — EvidenceArtifactSet assembler + human-viewable renderer (the ## Evidence section)
  - crates/cli — `elai pr-review --emit-evidence` flag wiring (default-ON, kill-switch ELAI_PR_EVIDENCE_RENDER_DISABLE)
consumes_symbols:
  - github_pr/review.rs::post_review_comment / ReviewCommentKind (the existing poster — RENDER through it, never re-implement)
  - rra.rs::{build_rra,verify_rra} (RRA digest source); review_surface (G-5 DSR + bands); review_anchor.rs (H-52 anchors); TestOutcomePayload (F-4 stdout_hash); .claude/tools/visual/ screenshots
reachable_from:
  - crates/cli `elai pr-review --emit-evidence` → github_pr post path (PRODUCTION CLI)
  - the K-3 check-run posting path → EvidenceArtifactSet → post_review_comment
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO new evidence-capture machinery (render ONLY existing RRA/F-4/G-5/H-52/visual-ui-loop artifacts)
  - NO re-implementation of github_pr, rra, review_surface, or review_anchor (GD-1 P0)
  - NO silently-clean hunk: a hunk with no backing artifact renders UNVOUCHED
  - NO git mutation by any worker; NO lavish author-brand design bundle; NO .unwrap()/.expect() on prod paths
frontend_pair:
  the G-5 reviewer cockpit gains the same EvidenceArtifactSet view (Tauri read of the rendered section) — frontend_pair via the cockpit; cross_lang_diff.py PASS. (PR-body render itself is W-6 N/A — GitHub is the surface.)
observable_effect:
  trigger: "elai pr-review --emit-evidence --pr <n>  (deterministic fixture leg: render to stdout; live check-run post RunPod/network-gated, W-10)"
  expect_ssb: "prefer-reuse github_pr / G-5 review_surface rows; else +1 EvidenceArtifactRendered (Pattern-2; Kind: SectionRendered / HunkVouched / HunkUnvouched)"
  expect_artifact: "a ## Evidence section carrying the RRA digest + test-evidence links + screenshot refs; NEGATIVE arm: a hunk with no backing artifact MUST render UNVOUCHED (never omitted, never clean)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-11_human_viewable_evidence_render.md --run-trigger
rg "EvidenceArtifactSet|UNVOUCHED" .elai_cc/crates/orchestrator/src/github_pr/evidence_render.rs   # render + unvouched path present
rg -c "fn post_review_comment" .elai_cc/crates/orchestrator/src/github_pr/evidence_render.rs        # MUST be 0 (consumed, not re-implemented)
test -f workflow/reasoning/K-11-decisions.md
```

## ACCEPTANCE
- [ ] KD-2: the evidence renders into the PR (check-run/comment); the cockpit is the DEEP view, never the only view.
- [ ] KD-1: every rendered number carries its measured/journal source or an [EXTERNAL]/[DESIGN] tag.
- [ ] GD-1: github_pr/rra/review_surface/review_anchor CONSUMED, not re-implemented.
- [ ] W-2: render ships default-ON behind ELAI_PR_EVIDENCE_RENDER_DISABLE (kill-switch, not enable-flag).
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator (+cli). No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = `elai pr-review --emit-evidence` run-trigger renders the section (not cargo-green).
- W-2: kill-switch ELAI_PR_EVIDENCE_RENDER_DISABLE; no default-OFF enable-flag.
- W-3: renders REAL journaled artifacts, not a summary-only port.
- W-4: frontend_pair = cockpit evidence view; cross_lang_diff PASS.
- W-5/W-10: live check-run post is the network-gated checkbox; deterministic render fixture is the in-process leg.
- W-6: PR-body render is the GitHub surface (cockpit covers the dashboard item).
- W-7: every anchor re-grepped at edit.
- W-9: SSB prefer-reuse; any bump named from the LIVE baseline (~267/577), never hard-coded.
- GD-5: cloud reproducibility stays action-stream; the render says so.

## Tests to add
- crates/contracts/tests/k11_evidence_render_contract.rs (~LOC): (a) EvidenceArtifactSet assembles RRA+F-4+G-5+H-52+screenshot refs from journaled rows; (b) a hunk with no artifact renders UNVOUCHED; (c) the renderer calls post_review_comment (does not re-implement it); (d) risk-tier collapse/expand follows the G-5 DSR; (e) no git mutation occurs.

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/github_pr/evidence_render.rs (~150 LOC)
- MODIFY: .elai_cc/crates/cli (the pr-review subcommand: --emit-evidence flag) (~20 LOC)
- MODIFY: .elai_cc/crates/dashboard-v2 (cockpit evidence view) (~40 LOC) + Tauri read pair
- CREATE: crates/contracts/tests/k11_evidence_render_contract.rs (~70 LOC)
- CREATE: workflow/reasoning/K-11-decisions.md

## Citations / Grounding
- steal-kunchen-agentic-tools-2026-06-24.md C2.2 EXTENSION(2) (no-mistakes test/pr stages; ## Testing/Evidence render) + C3.3 META demand-side ("eng-manager not diff-reviewer", AUTHOR claim — market-validation, not an ELAI number, HD-1/ID-5).
- ELAI ground truth: github_pr/review.rs post_review_comment/ReviewCommentKind (WIRED); G-5 review_surface; rra.rs; review_anchor.rs (H-52); visual-ui-loop (.claude). All re-grepped at edit (W-7).
- CLAUDE.md: blake3-only, no .unwrap on prod, worker-git-read-only, no-removal-without-confirmation.

## Open questions / risks
- GitHub check-run vs PR-comment size limits for inline screenshots — render refs/links, not embedded blobs; decide at authoring against the live API limit (WebSearch the current limit, KD-1).
- DOCTRINE-TENSION (recorded, NOT in K-11 scope): no-mistakes' PR-babysit-to-merge auto-fix loop tensions with one-commit-at-end + worker-git-read-only; the on-wedge slice is poll-CI + surface, human merges, NO autonomous force-push — surfaced to the user, not auto-authored here.
- lavish overlapping-text elementFromPoint occlusion probe is a separate rider to visual-ui-loop.md, not K-11.

## Next
K-11 lands the distribution leg's human face (the buyer sees the evidence in the PR). Pairs with K-3 (the verdict renderer it extends) and A-5 (the G-5 review-time study measures the win). After K-11 + K-3, the K-T2 wedge — "vendor-neutral replayable evidence where supervision happens" — is renderable end-to-end.

End of K-11.
