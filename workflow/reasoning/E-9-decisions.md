E-9 decisions — agent-eval benchmarks substrate (completion-drive ship)

Session: s-25120 completion-drive
SCHEMA bump: 164 -> 165 (atomic +1 single-variant Pattern-2 17th application)
TOTAL_VARIANTS bump: 437 -> 438

DECISION-1: Pattern-2 single SsbActionType variant + Phase discriminator
What: ONE new SsbActionType `BenchmarkRunObserved` carrying `BenchmarkRunObservedPayload`
with a 5-arm `BenchPhase` discriminator: Loaded / Started / ItemEvaluated / Completed /
Aborted. Plan as-written proposed 12 separate variants (SweBenchLiveRunCompleted +
LiveCodeBenchRunCompleted + Tau2BenchRunCompleted + BfclV3RunCompleted +
GaiaRunCompleted + ClawsBenchSplitCompleted + SycophancyProbeCompleted +
MathFloorsCompleted + HleCeiling Tracked + DeepPlanningCompleted + DsCpCoverageMeasured
+ BenchPowerCalibrated + LowPowerNoConclusion).
Why: 17th application of Pattern-2 per E-7 DarwinianGenerationCompleted + E-8
PromptOptimizationCompleted + E-10 CriticDecisionEvent + E-43 SkillSystemEvent
precedent. Single-variant collapses 12 * 10-site cascade = 120 cascade sites to 1 *
10-site cascade. The suite_name is a String payload field, NOT an action-type
distinguisher. Power-calibration (required_n vs observed_n) lives as Option<usize>
payload fields, low_power flag as bool, p_value as Option<f64>. Suite-specific
fields (model_effect_pp / harness_effect_pp for ClawsBench, coverage_gap_pp for
DS-CP) ride as Option<f64> with `None` default — forward-compat envelope per D-30
CalibrationBootstrap precedent.
Grounded by: ssb.rs:5489 DarwinianGenerationCompleted (Pattern-2 13th), ssb.rs:5601
SkillSystemEvent (Pattern-2 16th), context-package interface map + exploration
gap-2-recommendation.

DECISION-2: Promotion-gate reuse, side-by-side over replacement
What: `orchestrator::external_benches::registry` calls into existing
`orchestrator::promotion_gate::two_gate_promote(candidate, descriptor, archive, cfg)`
from E-7 substrate when a model/harness change requests promotion. The 11-suite
verdict + paired-bootstrap p map to `PromotionConfig.tau` margin; `SignificanceVerdict::Regression`
short-circuits before two_gate_promote is invoked. No fork of promotion logic.
Why: promotion_gate.rs:103 exists (E-7 batch2; cross-check report MISSING claim is
STALE per context-package HIGH-1). Reuse over re-implement. DS-CP validator lives
side-by-side at orchestrator/src/ds_cp_validator.rs with existing
orchestrator/src/conformal_calibration.rs (compute_tau / decide / TauUpdater)
unchanged — DS-CP ADDs reweighted-proximity CoverageReport, does NOT replace plain
conformal. Same pattern: prmbench_adversarial.py wraps existing
orchestrator/src/benchmarks/prmbench_runner.rs (D-19 SHIPPED), does NOT replace.

DECISION-3: 7 paper cites grounded, 3 unresolved/removed
GROUNDED (7 — paper-ingestor verified per grounding doc):
  - arXiv:2509.16941 SWE-Bench Pro (RAG-indexed, 41 chunks project:paper-swe-bench-pro,
    primary live-eval target replacing plan's SWE-bench-Live Verified mention)
  - arXiv:2505.23419 SWE-bench-Live (locally cached .claude/papers/2505.23419.html;
    monthly-refreshed, contamination-resistant)
  - arXiv:2511.22173 RefineBench (RAG-indexed, project:paper-refinebench; self-refinement
    ceiling Gemini 2.5 Pro 31.3% baseline)
  - arXiv:2604.05172 ClawsBench (verified Apr 2026; scaffold +39-63pp dominates 10pp
    model effect)
  - arXiv:2311.12983 GAIA (verified Nov 2023; human 92% vs GPT-4+plugins 15%; Sonnet
    4.5 leads 74.6%)
  - arXiv:2601.18137 DeepPlanning (locally cached .claude/papers/2601.18137.html;
    GPT-5.2-high 44.6% multi-day ceiling)
  - arXiv:2506.07982 tau2 (Sierra Research dual-control conversational eval; SOTA
    0.862 retail / 0.993 telecom Anthropic)
SUPPORTING (paper-ingestor verified but used as live-eval ground only, not for
code-grounding numbers):
  - arXiv:2403.07974 LiveCodeBench (interface map already in workflow/papers/)
  - arXiv:2501.03124 PRMBench (interface map already in workflow/papers/)
  - arXiv:2310.13548 Sharma sycophancy (interface map already in workflow/papers/)
  - arXiv:2406.07791 LLM-as-judge position bias (paper-ingestor VERIFIED; substitutes
    plan's arXiv:2410.15393 unverifiable cite)
  - arXiv:2306.05685 LLM-as-judge MT-Bench (foundational; substitutes plan's
    arXiv:2504.03846 unverifiable cite)
  - arXiv:2502.03461 GSM8K-Platinum methodology + gradientscience.org/gsm8k-platinum
    blog + madrylab/gsm8k-platinum HF dataset (110/219 corrected items grounding)
UNRESOLVED — DOCUMENTED AS REMOVED, NOT CITED IN CODE COMMENTS:
  - "DS-CP arXiv:2510.05566" — plan's exact citation does NOT resolve. Searches
    returned DS-1000 (2211.11501), DSCodeBench (2505.15621), DA-Code (2410.07331),
    DSBench (ICLR 2025). DS-CP module ships as ELAI design hypothesis with
    closest-substitute placeholder; no arXiv ID embedded in code comments. Live
    coverage-gap measurement RUNPOD-PENDING.
  - focused.io/lab/agent-benchmarks-measure-the-harness — blog URL does NOT resolve.
    The "scaffold dominates" framing is grounded on arXiv:2604.05172 ClawsBench
    directly; the unresolved blog cite is REMOVED from code comments.
  - MetaMate — no arXiv ID resolves; REMOVED from plan-citation set entirely (plan
    citation list did not include it; mentioned in grounding doc as unresolved).
Why: per memory:feedback_paper_citation_verify and CLAUDE.md anti-stale rule,
unverifiable cites cannot ground claims. Following E-7 DECISION-3 precedent (REMOVED
arXiv:2602.12413 ZERO_RELEVANCE) — unresolved cites are documented and dropped.

DECISION-4: ENV gate + cfg feature dual-lock
Gate: `ELAI_EVAL_BENCHMARK_ENABLE=1` env var required for live eval dispatch;
scaffolding stays in-tree always. Optional: composes on existing
`cfg(feature = "darwinian-evolve")` only WHERE the gate calls into E-7 promotion_gate
machinery. The 3 new Rust modules (external_benches / bench_significance /
ds_cp_validator) are NOT cfg-feature-gated (always compiled, callable; emission
gated). Mirrors E-7 + E-8 pattern: substrate compiled, live loops RUNPOD-PENDING.

DECISION-5: 7 of 11 Python runners ship as scaffolding ONLY (RUNPOD-PENDING markers)
Live model evals (SWE-Bench Pro, SWE-bench-Live, LiveCodeBench v6, tau2-bench,
BFCL v3, GAIA L1+L2, DeepPlanning) require remote GPU + dataset downloads + token
budget on the sidecar host. Per session prompt §RUNPOD-AWARENESS the scaffolding
ships fully (Python runner files, pyproject.toml, CLI subcommand, SSB emission
points) — every live-eval call site carries `# RUNPOD-PENDING: <command>` marker.
ClawsBench split + sycophancy probes + math_floors + HLE ceiling are also
RUNPOD-PENDING for live runs; the FIXTURE-driven contract tests run host-side
deterministically.

DECISION-6: symbol_graph anchor retarget
Plan §Reads cites `.elai_cc/crates/orchestrator/src/symbol_graph.rs` — file does
NOT exist at that path. The orchestrator/src/lib.rs:647 `pub mod symbol_graph`
declaration resolves to a dir or module index that doesn't ship a single-file
counterpart. Actual symbol_graph code lives at `crates/knowledge/src/symbol_graph/`
(8 files including store.rs, types.rs, mod.rs). RESOLUTION: SWE-Bench Pro / RefineBench
regression target documented as `crates/knowledge/src/symbol_graph/` directory
anchor; bench runners do not import a specific symbol_graph type — they only
emit observations against the `run_subcommand::RunArgs` pipeline.

DECISION-7: Pattern-2 ENV gate naming
`ELAI_EVAL_BENCHMARK_ENABLE` — concrete prefix per E7_FSM_SUBSTATE_COLLISION_RENAME
gotcha (anti-collision; not bare `ELAI_BENCHMARK_ENABLE` which would shadow other
benchmark runners). No new FsmSubstate added (benchmarks run under existing AUDITING
state without a new substate marker per exploration DECISION-7).

DECISION-8: CLI surface
Subcommand: `elai eval run --benchmark=<NAME> [--harness-vary] [--p-target=0.05]
[--list] [--json]`. Lives at crates/cli/src/eval_cmd.rs (NEW). Routes through
new `CliAction::EvalRun` variant added to args.rs after `BenchAblation`. NOT
`elai bench external` (plan's name) — `eval` is a top-level new subcommand to
keep separate from the 10 existing `bench *` D-19..D-25 runners (which are Rust
runners; the 11 new external suites are Python runners shelling out per
DECISION-5).

DECISION-9: Dashboard panel deferred per session prompt LANDING note
Session prompt §LANDING: "NO dashboard panel (E-9 not in dashboard list)". This
deviates from plan §Task 16 (ExternalBenchPanel.svelte 4x3 grid). Rationale per
session prompt: the 11-suite grid is wide + nontrivial UI; the SSB stream already
surfaces every BenchmarkRunObserved beacon via existing read-only Activity surface
+ get_optimizer_run-style command pattern, so the panel is not load-bearing for
verdict surfaces. Deferred to a follow-on E-N. The Tauri command +
`get_external_bench_history` are NOT added in this commit per LANDING.

DECISION-10: Anti-slopsquatting Python deps
.elai_cc/benches/external/pyproject.toml deps: httpx, orjson, scipy, hypothesis,
datasets (HF). All verified to exist on pypi.org. Pinned versions documented
inline with last-published date.
  - httpx 0.27.* — pypi.org/project/httpx/ verified; standard async HTTP client.
  - orjson 3.10.* — pypi.org/project/orjson/ verified; per CLAUDE.md anti-stale
    over stdlib json.
  - scipy 1.13.* — pypi.org/project/scipy/ verified; bootstrap + ANOVA stats.
  - hypothesis 6.* — pypi.org/project/hypothesis/ verified; property tests.
  - datasets 2.* — pypi.org/project/datasets/ verified; HF dataset loader.

DECISION-11: Anti-LLM-as-judge bias rotation
ENFORCED: `judge_rotation_contract.rs` contract test asserts every LLM-as-judge
call surface (HLE, sycophancy probes, ClawsBench split when LLM-judged) rotates
option positions + ID tokens per call. Grounded on arXiv:2406.07791 (Judging the
Judges position bias) + arXiv:2306.05685 (MT-Bench foundational). The unverifiable
arXiv:2410.15393 + arXiv:2504.03846 cites from plan are REMOVED from code comments;
replaced by these two grounded substitutes.

DECISION-12 (audit follow-up s-25120 batch4-fixer): promotion-gate composition wired
through `external_benches::promote_after_bench_completed` (NEW). The function lives
behind `#[cfg(feature = "darwinian-evolve")]` matching the gating on
`promotion_gate.rs` (per DECISION-4). Callers invoke it AFTER the BenchPhase::Completed
SSB row is emitted. The candidate ArchiveEntry uses a zero `template_blake3` for the
benchmark-promotion bridge — the canonical template hash flows in via descriptor when
the live RUNPOD path lands (per-axis capacity breakdown per arXiv:2510.04399 §Theorem
11 remains RUNPOD-PENDING; the cell-count proxy in `two_gate_promote` ships unchanged).
No benchmark_runner.rs module exists yet — wiring is exposed as a public function on
the external_benches surface so future runners + CLI consumers can compose.
