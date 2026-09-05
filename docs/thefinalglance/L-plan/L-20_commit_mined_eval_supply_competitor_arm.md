# L-20: commit-mined eval task supply + competitor-arm paired harness — the buffbench steal (measurement machinery, not agent machinery)

STATUS: PLANNED (L-T4 WAVE 1; compressed form, steal track; nature EXTENSION+BUILD-instrument)

- Track L-T4 (MEASURED-COMPETITOR / EVAL-SUPPLY) | lever GROUNDED/AUDITABLE | grounding STRONG (codebuff source read 2026-07-03 + feature-map anchors, PLAN-asserted file:lines re-grep at edit per W-7) | nature EXTENSION(J-56/E-54) + BUILD-instrument | effort M-L | WAVE 1 | depends: J-56 corpus verbs (or ships their first production caller), the WIRED agent-CLI bridges

- Premise: codebuff's only genuinely stealable assets are in evals/ — buffbench mines HARD tasks from a repo's real commit history (real diff = ground truth, contamination-resistant by construction) and runs COMPETITOR CLIs as arms on the same tasks under one envelope. ELAI already ships the hard half of the competitor leg (the vendor-CLI bridges) and the governance half of the supply leg (E-54 corpus + J-56 verbs); L-20 is the composition. This is the task-SUPPLY + comparison-ARM leg the GTM "believe-the-number" requirement (project_gtm_playbook_2026) has no unit for.

## Steal/build
Three legs, one file (fewest-files; all three live in the bench/eval subsystem 1.13).

LEG A — `elai eval mine-commits` (the supply). Port the pick-commits.ts pipeline shape, deterministic-first: (1) walk git history of a target repo (own repo or a user-pointed one), DETERMINISTIC pre-filter (merge/revert/deps/lockfile/docs-only/formatting-only/mass-rename OUT; floor+ceiling on files-changed/insertions; failure_taxonomy.py categories reused for typing); (2) an OPTIONAL advisory LLM screen for "HARD" selection (codebuff's reject-list prompt is the reference) — ADVISORY tier per TD-3: it RANKS candidates, it never fabricates a row, and a row's admissibility is the deterministic filter's verdict; (3) emit J-56 corpus rows: {task_prompt derived from commit message + touched-surface summary, base_sha (parent tree = workspace), reference_diff (blake3-hashed), category, provenance}. Rows flow through `elai eval ingest` (J-56) and the E-54 13-gram contamination filter (public-repo rows only; private-repo rows are contamination-resistant by construction — that asymmetry is the point and is journaled per row).

LEG B — competitor-arm runner (the comparison). A bench arm kind CompetitorArm{backend} over the SHIPPED agent-CLI bridges: claude_code / codex_cli / opencode / gemini_cli spawn through cli_common::spawn_isolated_with_event (cli_common.rs:330, WIRED, Tier-C) with their existing hand-parsed stream-json legs — do NOT write a second spawner (GD-1). Each task runs in a disposable workspace (J-80 pool when it lands; a plain temp worktree meanwhile), ELAI arm + N competitor arms on the SAME row set, identical envelope (task prompt, workspace, wall-clock cap, journaled cost per E-49 $/CTS shape). Scoring: deterministic legs are the GATE — build green, scoped tests green where the row carries them, reference-diff file-set overlap (structural, DKG/AST-level, advisory-weighted), completion-vs-abstention; the buffbench 0-10 LLM judge is ADVISORY-tier only, journaled, never load-bearing (TD-3; Section 4 LLM-judge-on-a-gate stays rejected). Statistics per I-38: >=5 seeds, paired, pass^k floor. OUTPUT DISCIPLINE (binding): an internal paired delta under a published reproducible envelope (GD-2 recomputable under ELAI_SEED where the arm permits; competitor arms are honestly non-seed-reproducible — journaled as such, MD/KD claims discipline); NEVER a leaderboard-rank claim (GD-3; SWE-bench-headline DROPPED precedent); any public number gates on KD-1. Competitor arms run with the vendor's own permissive flag INSIDE the disposable workspace only; the ELAI arm always runs its normal gates (measuring ELAI-with-gates vs competitor-without is the honest product framing and is stated in the envelope).

LEG C — bench lessons rows (observe-only). Port the lessons-extractor.ts shape: per FAILED arm-task, an advisory {what_went_wrong, what_should_have_been_done} row extracted from the trace + deterministic score deltas, journaled for J-10 insight backprop / I-12 curator / VCG J-96 consumption. H-31 firewall: lessons rows are NEVER injected into the auditor's context; K-19 SF-1 binds any promotion (execution-grounded outcomes only, never "the lesson was retrieved").

EXCLUDED (verdicts in workflow/reasoning/steal-codebuff-2026-07-03.md): the 0-10 judge as a gate (S10 REJECT), set_messages history mutation (S11), the agent store (S12), handleSteps user code agents (S13), ads/streaks (S15), the repo-claimed 61%-vs-53% number (S18 — design-rationale only, never cited).

## Existing ground truth (all PLAN-asserted from ELAI-FEATURE-MAP.md 2026-07-03; executor re-greps per W-7)
- agent-CLI bridges WIRED: cli_common.rs:330 spawn_isolated_with_event; claude_code.rs:68/90 stream-json parse; contract-tested; Tier-C (map 1.9 AGENT-CLI BRIDGE TRANSPORT).
- J-56 PLANNED: `elai eval ingest` / `elai eval run --scope subset|full`; +EvalCorpusEvent (RowIngested/RowRejected/SuiteRun); E-54 corpus DARK (ingest_task no CLI entrypoint; 13-gram filter; per-family shards).
- G-6 SAFEDELTA BENCH-PENDING (runner shape canonical, do NOT fork); I-38 QDELTA PLANNED (>=5-seed paired floor); I-27 bench_convergence runner shape; K-15/J-107 both REUSE the same runner family (GD-1 precedent).
- E-49 $/CTS metric PARTIAL; J-80 worktree pool PLANNED; failure_taxonomy.py WIRED (.claude/tools, 13 categories).

## Integration
- crates/. . . benchmarks/ competitor_arm_runner.rs + commit_miner (mirror the K-15/J-107 runner pattern over the I-27 shape; args_bench.rs arm) + the `elai eval mine-commits` CLI verb beside J-56's verbs.
- frontend_pair: a paired-arm tile on the bench/audit surface (decide at edit, W-4) or justified N/A while the dashboard bench panel family (E-49) is PARTIAL.

## WIRING_CONTRACT
produces_symbols:
  - benchmarks/commit_miner (deterministic filter + row emitter) and the `elai eval mine-commits` CLI verb
  - benchmarks/competitor_arm_runner.rs (CompetitorArm kind over the cli_common bridge spawn seam)
  - the bench-lessons observe-only emitter (leg C)
consumes_symbols:
  - crates/providers cli_common::spawn_isolated_with_event + the claude_code/codex_cli/opencode parse legs (the spawn/parse machinery — do NOT re-implement, GD-1)
  - J-56 EvalCorpusEvent verbs + E-54 contamination filter + failure_taxonomy categories (row admission)
  - I-38 paired-stats floor + I-27/K-15 runner shape + E-49 cost capture (scoring + envelope)
reachable_from:
  - `elai eval mine-commits` and `elai bench competitor` CLI verbs (real production entrypoints; a #[cfg(test)] caller never satisfies this, R7)
stub_ban:
  - NO second CLI spawner; NO LLM verdict on row admissibility or arm scoring; NO leaderboard-rank string in any output; NO silent skip of an unavailable vendor CLI (named FAIL, R1); NO .unwrap() on prod paths; blake3 only
frontend_pair: decide at edit (paired-arm tile) or justified N/A per above.
observable_effect:
  trigger: "elai eval mine-commits --repo <path> --max 20 && elai bench competitor --arms elai,claude_code --rows <subset> --seeds 5"
  expect_ssb: "EvalCorpusEvent Kind=RowIngested rows carrying base_sha + reference_diff blake3; SuiteRun (or +Kind CompetitorArmRun) rows per arm with cost + deterministic score legs; prefer-reuse, net 0..+1 (decide at edit, re-grep LIVE baseline per W-9)"
  expect_artifact: "a paired-delta report artifact recomputable from the journal (GD-2). NEGATIVE ARMS: a merge/docs-only commit is filtered OUT deterministically (row never minted); an absent vendor CLI fails the arm with a named reason, never a silent skip; a leaderboard-rank formatted string in the report = the contract test FAILS"

## Grounding
- codebuff source (local, read 2026-07-03): evals/buffbench/pick-commits.ts (the HARD-commit screen + reject list), runners/claude.ts (spawn `claude -p --output-format stream-json`, cost capture), judge.ts (advisory-only reference), lessons-extractor.ts (leg C shape), setup-test-repo.ts. Apache-2.0; shapes ported, no code copied.
- ELAI anchors: feature map 1.9/1.13 entries quoted above; steal ledger checked (leaderboard ban GD-3, LLM-judge-on-a-gate rejected, batch-racing IC-1 untouched — arms are sequential-or-capped, never racing).
- The repo-claimed 61%/53% is tier-2 and appears nowhere in ELAI output.

## COMPOSES-WITH
J-56 (corpus + verbs), E-54 (filter/shards), G-6/I-38/I-27/K-15/J-107 (runner family + stats floor — REUSE, do not fork), agent-CLI bridges (B-37 family), E-49 ($/CTS), J-80 (workspace pool), J-10/I-12/J-96 (leg C consumers), K-19 SF-1 + H-31 (leg C discipline), M-5/K-1 (where a public number would ship — KD-1 gated).

End of L-20.
