# K-3: PR/CI evidence emitter — render ELAI's per-hunk evidence into GitHub check-runs + PR comments over the WIRED github_pr adapter (THE distribution leg)

STATUS: PLANNED (WAVE KW-2; BUILD-renderer over a WIRED adapter; KD-2/KD-5; the single highest-leverage distribution leg)

Objective: the PR is the supervision surface the market standardized on (KS-3 reports 16/18); ELAI's
evidence renders only in its own CLI/dashboard today. Devin/Factory post OPINION/SCREENSHOT evidence;
nobody posts deterministic, hash-bound, replayable evidence — that is the open slot. This session BUILDS
a RENDERER that emits ELAI's per-hunk evidence into GitHub check-runs + PR comments over the ALREADY-WIRED
github_pr adapter (KG-2/KC-2): gate verdicts (F-1/F-2/F-9), F-4 stdout-hash-bound test outcomes, G-5 DSR
+ vouched/unvouched bands, H-52 anchored findings (REUSE its resolver for comment placement — hallucinated
line numbers become structurally impossible), the G-8 attestation digest + an offline-verify line, and a
J-16 blast-radius cone summary when available. Hard boundary: it is a RENDERER — it re-implements neither
the github_pr adapter nor any evidence source; a hunk with NO evidence renders UNVOUCHED, never silently
clean (R1). It renders ELAI's OWN-run evidence first; the A-4-wrapped foreign-agent evidence when A-4 lands.

Estimated effort: ~1.5 sessions (~6-7 h). In-process render + a live check-run POST against a fixture
repo (the github_pr adapter already does the GitHub I/O — KG-2).
Pre-requisites: the github_pr module WIRED (KG-2 — review.rs, body_composer.rs, comment_watcher.rs,
adapter_trait.rs, gh_cli.rs; CLI pr_review_cmd.rs; Tauri commands_c40.rs; state github.svelte.ts). G-5
review_surface + H-52 finding anchor (KG-3). F-4 stdout-hash binding, G-8 attestation, J-16 cone (when
shipped). The RRA evidence manifest as the closed-set render source.
Parallel-safe with: K-2, K-4, K-6, K-9 (disjoint files). NOT parallel with edits to github_pr internals.
Source: K-MASTER-BRIEF §3.2 K-3 [KS-1]; KS-3 reports 16/18; grounding map KG-2/KC-2, KD-2/KD-5. Doctrine: K-0_INDEX.md.

## Context
Why this matters: distribution is ELAI's weakest moat (#1, KS-2 §1.7); attaching the evidence to the
surface that already won (the PR) is the leverage. Mechanism: read the RRA evidence manifest for the
PR's hunks as a CLOSED SET, render each evidence kind into a GitHub check-run + PR comments via the
github_pr adapter; the H-52 resolver places comments at real anchors (no hallucinated line numbers); a
hunk with no evidence renders explicitly UNVOUCHED. Honesty (KD-2): the Tauri dashboard stays the DEEP
view; the PR rendering is the meet-the-buyer view. Time-boxed under KD-5 (spec-first-mover; GitHub is the
most credible absorber — KS-2 §1.10).

## Existing state (A/B..J coverage)
- KG-2/KC-2 github_pr WIRED: crates/orchestrator/src/github_pr/{review.rs, body_composer.rs,
  comment_watcher.rs, adapter_trait.rs, gh_cli.rs, semgrep_gate.rs, auto_merge_guard.rs, types.rs};
  CLI crates/cli/src/{pr_review_cmd.rs, github_pr_cmd.rs}; Tauri commands_c40.rs; frontend
  github.svelte.ts; contracts e41_ci_cd_pr_review_contract.rs + c40_github_pr_adapter_contract.rs.
- KG-3 G-5 review_surface.rs (DSR + vouched/unvouched) + H-52 h52_finding_anchor_contract.rs (the
  resolver). score_review_surface / review_anchor symbol names PLAN-asserted; re-grep at edit (W-7).
- F-4 stdout-hash-bound test outcomes; G-8 attestation digest + `elai verify-attestation` (KG-7); J-16
  blast-radius cone (J-16 BlastRadiusEvent — when shipped).
- A-4 I-40 `elai wrap` PLANNED (KC-4) — the foreign-agent evidence source; K-3 does NOT wait on it.

## Reads (mandatory before coding)
1. _grounding/K-GROUNDING-MAP.md (KG-2/KG-3/KC-2, KD-2/KD-5).
2. crates/orchestrator/src/github_pr/{adapter_trait.rs, body_composer.rs, comment_watcher.rs, review.rs}
   (the adapter surface this renders over) + crates/cli/src/pr_review_cmd.rs (the entrypoint).
3. crates/orchestrator/src/review_surface.rs (G-5 DSR + bands) + the H-52 finding-anchor resolver
   (h52_finding_anchor_contract.rs target) — re-grep the resolver symbol at edit (W-7).
4. The RRA evidence manifest shape (G-2; rra_cmd.rs) — the closed-set render source.
5. F-4 / G-8 / J-16 surfaces for the digest + cone summary.

## Tasks (WAVE KW-2 — the renderer)
1. Build the evidence-manifest reader: from a PR's RRA evidence manifest, assemble a closed set of
   per-hunk evidence records (verdict, stdout-hash outcome, DSR band, anchored findings, attestation
   digest, cone summary). ~120 LOC.
2. Build the check-run + PR-comment renderer over github_pr's body_composer/comment paths: one check-run
   summarizing gate verdicts + the attestation digest + the offline-verify line; per-hunk comments
   placed via the H-52 resolver. A hunk with no evidence -> a UNVOUCHED comment (never omitted, never
   clean). ~180-240 LOC. RENDERER ONLY — no new adapter, no new GitHub I/O.
3. Wire it behind the existing `elai pr-review` entrypoint (pr_review_cmd.rs) as an evidence-emit mode;
   prefer-reuse the github_pr / review_surface / attestation SSB rows the evidence already owns
   (net 0..+1; else +PrEvidenceEmitted — Kind: CheckRunPosted / HunkVouched / HunkUnvouched).
4. NEGATIVE arm: a `--post-unvouched-as-clean` flag MUST be rejected/fail (the closed-set render makes
   an out-of-evidence "clean" comment structurally impossible).
5. frontend_pair: surface the emitted check-run status on github.svelte.ts (decide at edit, W-4).
6. workflow/reasoning/K-3-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - the evidence-manifest reader + the check-run/PR-comment renderer (in/under github_pr/)
  - the `elai pr-review` evidence-emit mode (pr_review_cmd.rs)
  - +PrEvidenceEmitted SSB OR reuse of the github_pr/review_surface/attestation rows (decide at edit)
consumes_symbols:
  - github_pr::adapter_trait + body_composer + comment_watcher (KG-2 — the adapter; RENDER over it,
    re-implement nothing; KC-2)
  - review_surface (G-5 DSR + bands) + the H-52 finding-anchor resolver (comment placement)
  - F-4 stdout-hash outcomes, G-8 attestation digest, J-16 cone summary (when present)
  - the RRA evidence manifest (G-2) — the closed-set render source
reachable_from:
  - `elai pr-review <pr>` in evidence-emit mode -> github_pr adapter -> a real GitHub check-run
    [PRODUCTION CLI + GitHub check-run]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO re-implementation of the github_pr adapter or any evidence source (RENDERER only; GD-1)
  - NO silently-clean hunk — a no-evidence hunk renders UNVOUCHED (R1)
  - NO open-set/free-text evidence comment (closed-set render from the manifest)
  - NO hallucinated line number — comments placed via the H-52 resolver only
  - NO .unwrap()/.expect() on the prod render path
frontend_pair:
  the emitted check-run status on crates/dashboard-v2/src/lib/state/github.svelte.ts (decide at edit,
  W-4) — the dashboard is the DEEP view, the PR is the buyer view (KD-2).
observable_effect:
  trigger: "elai pr-review <fixture-pr> --emit-evidence"
  expect_ssb: "the github_pr/review_surface/attestation rows the evidence owns (or PrEvidenceEmitted:
    CheckRunPosted + per-hunk HunkVouched/HunkUnvouched), recomputable from the journal (GD-2)"
  expect_artifact: "a real check-run on the fixture PR whose every claim joins a journal row; an
    out-of-evidence comment is structurally impossible; the --post-unvouched-as-clean broken arm FAILS (R2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-3_pr_ci_evidence_emitter.md --run-trigger
rg -n "emit.evidence|evidence.emit" crates/cli/src/pr_review_cmd.rs    # the mode exists on the real entrypoint
rg -n "UNVOUCHED|unvouched" crates/orchestrator/src/github_pr/         # the honest no-evidence render
rg -c "post.unvouched.as.clean|unvouched_as_clean" crates/orchestrator/src/github_pr/  # the broken arm is rejected, not implemented as pass
python .claude/tools/cross_lang_diff.py --changed crates/orchestrator/src/github_pr crates/dashboard-v2/src/lib/state/github.svelte.ts  # frontend_pair drift
test -f workflow/reasoning/K-3-decisions.md
```

## ACCEPTANCE
- [ ] KD-2: the evidence renders in the PR (check-run + comments); the dashboard stays the deep view.
- [ ] R1: a no-evidence hunk renders UNVOUCHED; missing evidence is never silently clean.
- [ ] GD-1: RENDERER only — github_pr adapter + every evidence source re-used, not re-implemented.
- [ ] H-52 resolver places every comment (no hallucinated line numbers).
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator, -p cli; svelte-check if frontend_pair.
- [ ] SSB net 0..+1, sequential bump from live 236/545.

## WIRING DONE-GATE (clause by clause)
- W-1: done = a real check-run posts on a fixture PR + its claims join journal rows.
- W-2: N/A (the emit mode is invoked, not a default-OFF flag) — IF gated, register in wire-dark-allowlist
  with a promotion plan. W-3: N/A.
- W-4: frontend_pair = github.svelte.ts check-run status (or W-6-justified absence). W-5: live GitHub
  check-run trigger. W-7: github_pr + review_surface + H-52 resolver re-grepped at edit.
- GD-1/GD-2: as ACCEPTANCE. KD-2/KD-5: the PR rendering + the time-box.

## Tests to add
- crates/contracts/tests/k3_pr_evidence_emitter_contract.rs (~120 LOC): (a) a fixture PR with full
  evidence renders a check-run + per-hunk vouched comments; (b) a hunk with NO evidence renders
  UNVOUCHED (the honest case); (c) comment placement uses the H-52 resolver (anchored, never raw line);
  (d) every rendered claim has a journal row (GD-2); (e) --post-unvouched-as-clean is rejected (R2).

## Files touched
- CREATE: crates/orchestrator/src/github_pr/evidence_emitter.rs (~220 LOC, the renderer)
- MODIFY: crates/cli/src/pr_review_cmd.rs (~40 LOC: the --emit-evidence mode)
- MODIFY: crates/orchestrator/src/ssb.rs + tests/action_type_lock.rs (IF +PrEvidenceEmitted; sequential)
- MODIFY: crates/dashboard-v2/src/lib/state/github.svelte.ts (~30 LOC: check-run status, if frontend_pair)
- CREATE: crates/contracts/tests/k3_pr_evidence_emitter_contract.rs (~120 LOC)
- CREATE: workflow/reasoning/K-3-decisions.md (~30 LOC)

## Citations / Grounding
- KS-3 reports 16/18: the PR is the standardized supervision surface; deterministic/cryptographic/
  replayable PR evidence is the unfilled slot; vendors post observability (screenshots/logs), not
  attestation. [EXTERNAL]
- KS-2 §1.7/§1.10: distribution is the weak moat; 12-24mo window; GitHub the most credible absorber. [EXTERNAL]
- ELAI ground truth: KG-2 (github_pr module WIRED), KG-3 (review_surface + H-52 resolver), KG-7 (RRA +
  verify-attestation), KC-2 (renderer over a wired adapter).
- CLAUDE.md: GD-1 (compose), R1 (fail-closed evidence), H-52 resolver reuse.

## Open questions / risks
- Surface scope: GitHub-only v1 (check-runs API) is the recommendation (where the market is); GitLab is
  demand-gated (brief §7). Author GitHub-only; note GitLab as a later rider.
- github_pr internal coupling: confirm the body_composer/comment_watcher seams are stable enough to
  render over without forking; if not, surface to the user before adding adapter surface.
- A-4 dependency: foreign-agent evidence needs A-4 (`elai wrap`); K-3 ships own-run evidence first and
  consumes A-4's manifest when it lands (KC-4) — do not block on A-4.

## Next
This is the distribution leg — it puts ELAI's deterministic, hash-bound, replayable evidence where the
buyer supervises (the PR). It pairs with A-4 (the wrapped foreign-agent evidence source) and A-5 (the
review-time study measuring whether the rendered evidence saves review minutes). After it, the cockpit
(A-5) is the deep view, the PR is the wedge.

End of K-3.
