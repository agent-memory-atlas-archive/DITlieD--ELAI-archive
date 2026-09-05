D-28 — Adversarial example farming for PPM retraining (Adv-RM arXiv:2504.06141)

Session: s-25120 (completion-drive closeout pass; D-28 host-side scaffolding)
Plan: docs/thefinalglance/D-plan/D-28_adversarial_example_farming.md
Sibling closures: D-19 (PRMBench v1 runner) -> D-23..D-27 (sibling benches + dashboard surface). D-28 is the FIRST data-PRODUCING runner under crates/orchestrator/src/farm/ (sibling of the data-EVALUATING runners under crates/orchestrator/src/benchmarks/).

Substrate state at session start: D-2 SidecarPpmHook trait + MockSidecarPpmHook + HttpSidecarPpmHook shipped (no surface modifications this pass). D-19 PRMBenchRunner + GatingVerdict 2-arm closed enum shipped (we reuse GatingVerdict). SCHEMA_VERSION at session-start head = 144 (D-25). This pass bumps to 145 atomically.

Single-variant SSB precedent: D-19 / D-23 / D-24 / D-25 each shipped exactly ONE new SsbActionType variant per closure pass + end-of-run-only observation (no "_started" row). D-28 follows the same shape.

DECISION-1 — SCHEMA bump 144 -> 145 (atomic +1). Single new SsbActionType variant `AdversarialFarmRunCompleted` + single new SsbPayload variant carrying `AdversarialFarmRunCompletedPayload`. Producer-side: `crate::farm::adv_rm::AdvRmFarmer::run` emits exactly one row at end-of-run on the Ok return path (mirrors D-19/D-23/D-24/D-25 single-variant precedent). Lockstep cascade: enum / SQL CHECK / as_sql_str / from_str / SsbPayload / action_type() / all_test_payloads() / action_type_lock.golden (8th lockstep site) + variant tests. action_type_lock.rs TOTAL_VARIANTS 417 -> 418 (+1).
Grounding: crates/orchestrator/src/ssb.rs:1776 (SCHEMA_VERSION const = 145 post-edit); D-25-decisions.md DECISION-1 (precedent atomic +1 bump on a single-variant farm/bench closure pass).

DECISION-2 — Exactly ONE new SsbActionType variant `AdversarialFarmRunCompleted` (NOT 2). Plan-doc considered an additional `AdversarialFarmRunStarted` open-of-run row for liveness telemetry. REJECTED per D-19/D-23/D-24/D-25 precedent (end-of-run-only observation); the dashboard surface polls the SSB stream on a cadence anyway, and the runner's tracing::info! "adv-rm farm run starting" record satisfies operator-side visibility without polluting the SSB row stream with a transient open-event the dashboard would have to filter. Multi-variant alternative DEFERRED (would require a 2nd SSB variant + payload + dashboard handler).
Grounding: D-19-decisions.md DECISION-2 (2-arm closed GatingVerdict, no "_started" row); D-23-decisions.md (single SycophancyBenchRunCompleted variant precedent); D-24-decisions.md DECISION-1; D-25-decisions.md DECISION-1.

DECISION-3 — UNGROUNDED "+5pp acceptance gate" claim in plan-doc REJECTED. Recalibrated to +1.0pp paper-grounded floor (rounded UP to integer-pp from paper's +0.7pp overall RewardBench delta). Plan-doc and earlier draft cited "+5pp" as the PRMBench acceptance gate; that number does NOT appear in arXiv:2504.06141. The paper's actually-reported deltas (§5):
  - +0.7pp overall RewardBench (the conservative "what does the technique do on average" number)
  - +2.86pp Chat-Hard subcategory maximum (the strongest single-category effect)
  - Round-3+ yields near-zero new examples (saturation at 2 rounds)
  - >~1000 examples per round HURTS performance (diversity saturation)
ELAI floor pinned at 1.0pp (DEFAULT_PAPER_FLOOR_PP = 1.0 in adv_rm.rs:93). This is +0.7 rounded up to integer-pp to stay conservative without overclaiming. Module-level docstring + adv_rm.rs:386-391 explicit reject the +5pp value; with_paper_floor_pp() exists for caller overrides but documents the rejection.
Grounding: arXiv:2504.06141 §5 Table 2 (overall delta) + §3.2 (Z>1.96 95th-pct filter) + §5 saturation discussion. WebSearch verified (see workflow/reasoning/D-28-grounding.md for arXiv ID confirmation). Per memory rule feedback_paper_citation_verify: arXiv:2504.06141 = "Adversarial Examples for Reward Models" (verified title via WebSearch).

DECISION-4 — Live PPM fine-tune held at 1 epoch (paper §5 explicit "single epoch"). Plan-doc claimed "3-epoch fine-tune"; REJECTED. The paper's adversarial training run is explicitly described as single-epoch (§5: "after one epoch of additional training on the adversarial corpus"). Multi-epoch alternative would risk overfitting on the small adversarial corpus (1000 examples/round * 2 rounds = ~2000 total examples max) — the paper's design rationale is that the K>=2 ensemble uncertainty signal selects HARD examples, so even one epoch is sufficient to move the boundary. ppm_finetune.sh `EPOCHS=1` constant (RUNPOD-PENDING). Multi-epoch override deferred to a future plan (would need explicit grounding for the deviation).
Grounding: arXiv:2504.06141 §5.

DECISION-5 — K>=2 ensemble fallback for K=1 ELAI PPM singleton. The paper requires a K>=2 reward-model ensemble for the inter-model uncertainty std signal driving the Z>1.96 filter. ELAI has K=1 (Qwen2.5-Math-PRM-7B singleton — see project_qwen3b_baseline + project_b40_shipped memory entries; D-2 PpmClient construction). Fallback: use PPM response's `score_var` (per-step score variance) as the K=1 stand-in for K>=2 ensemble std. The semantic is "the proxy>>gold gap pattern" (D-4 rm-overoptimization style): when step scores diverge widely AND the mean is well above 0.5, the K=1 PPM is signaling "I have low confidence about this set of steps". This is a grounded HYPOTHESIS not a paper-direct claim — documented as such in adv_rm.rs:49-60 module docstring + workflow/reasoning/D-28-grounding.md. The single-PPM Z formula: `Z = (mean - 0.5) / sqrt(var)`; std=0 returns Z=0 to avoid div-by-zero. Production K=2 ensemble alternative DEFERRED — would require a second PPM checkpoint + second sidecar process + dual-port plumbing on top of D-2's HttpSidecarPpmHook.
Grounding: arXiv:2504.06141 §3.3 (K>=2 ensemble design); D-4-decisions.md (rm-overoptimization proxy>>gold pattern) — substrate precedent for K=1 fallback signals.

DECISION-6 — "PPM" project term. ELAI's local nomenclature for the Process Reward Model surface (Qwen2.5-Math-PRM-7B verifier) is "PPM" (Process Preference / PRM client wrapper). All D-28 module names + log targets + endpoint paths use "ppm" prefix to match D-2 substrate: `crate::sidecar_ppm_hook::SidecarPpmHook` trait, `PpmScoreRequest`, `PpmScoreResponse`, `PpmMode`, `/v1/ppm/score` HTTP endpoint, `ppm_health` CLI subcommand. The Python live-loop script is `python/elai_inference/ppm_adversarial.py` (not `prm_adversarial.py`). The fine-tune shell script is `workflow/remote-setup/ppm_finetune.sh`. No "PRM" namespace introduced (the paper's "PRM" is preserved in references to arXiv:2501.03124 PRMBench specifically; everywhere else uses ELAI's local "PPM" term for consistency with D-2).
Grounding: crates/orchestrator/src/sidecar_ppm_hook.rs (D-2 trait surface).

DECISION-7 — Z>1.96 + 1000-sample cap + 2-round max are HARD invariants. Pinned as constants in adv_rm.rs:
  - DEFAULT_Z_THRESHOLD = 1.96 (paper §3.2: 95th percentile two-sided normal CDF; 1.96 is the canonical statistics constant for that interval)
  - DEFAULT_MAX_SAMPLES = 1000 (paper §5 sweet spot)
  - MAX_SAMPLES_CAP = 1500 (saturation guard — paper §5 shows perf degradation above ~1000-1500; we allow up to 1500 to give callers headroom but no further)
  - DEFAULT_MAX_ROUNDS = 2 (paper §5 round-3 plateau)
  - MAX_ROUNDS_CAP = 2 (HARD ceiling — the with_max_rounds builder validates this; round 3+ is REJECTED at runtime per InvalidMaxRounds error)
Validation lives in AdvRmFarmer::validate() called at AdvRmFarmer::run() entry; out-of-range values surface typed FarmError variants (InvalidMaxSamples / InvalidMaxRounds / InvalidZThreshold / InvalidPaperFloor) at the parser boundary instead of mid-run.
Grounding: arXiv:2504.06141 §3.2 + §5.

DECISION-8 — D-2 + D-19 surfaces FROZEN this pass. SidecarPpmHook trait (sidecar_ppm_hook.rs:214), PpmScoreRequest, PpmScoreResponse, PpmMode, PpmHookError closed enum, MockSidecarPpmHook, HttpSidecarPpmHook — ZERO modifications. AdvRmFarmer is generic over `H: SidecarPpmHook` (adv_rm.rs:334 struct generic; consumes the trait surface as-is). D-19's GatingVerdict 2-arm closed enum (Pass/Fail) reused unchanged (adv_rm.rs:79 import); the report's `verdict` field is the same enum the PRMBench runner emits. No new error variants on D-2; no new fields on PpmScoreRequest/Response. This is the "no-surface-modifications" precedent D-19/D-23/D-24/D-25 set — additive plug-in to existing trait surfaces.
Grounding: crates/orchestrator/src/sidecar_ppm_hook.rs (D-2 surface); crates/orchestrator/src/benchmarks/prmbench_runner.rs (D-19 GatingVerdict).

DECISION-9 — RUNPOD-PENDING markers for all live-PPM components. Three RUNPOD-PENDING components ship in this pass as scaffolding only:
  - python/elai_inference/ppm_adversarial.py — live PPM scoring loop. Marked `# RUNPOD-PENDING: bash workflow/remote-setup/start-ppm-sidecar.sh` at the live-invocation site. Default `--mock` mode runs against the local mock for CI smoke; `--live` mode requires the remote PPM sidecar to be reachable on port 8101.
  - workflow/remote-setup/ppm_finetune.sh — live Bradley-Terry fine-tune script. Every live `ssh ... torchrun ...` line is `# RUNPOD-PENDING:` prefixed and commented out. The SSH host is a placeholder `${PPM_FT_HOST:-pythondev}` (the canonical remote sidecar host per CLAUDE.md "Remote sidecar (canonical)" block); the actual fine-tune kickoff is operator-driven once a 24GB+ training-class GPU is available.
  - dual `bench_prmbench v1` -> `bench_prmbench v2` runs that compute the live `prmbench_delta_pp`. The host-side AdvRmFarmer::run sets `prmbench_delta_pp = 0.0` (verdict = Fail) until the CLI driver or a future patch overlays the live delta from a real PPM-retrain cycle. The shipped scaffolding CANNOT self-declare a paper-grounded pass without the live retrain — this is INTENTIONAL.
Grounding: CLAUDE.md "Remote sidecar (canonical)" block (pythondev host details); memory rule project_qwen3b_baseline (24GB target hardware).

DECISION-10 — No new dashboard surfaces this pass. The dashboard's PrmBenchPanel.svelte (shipped in D-19/D-27) ALREADY renders prmbench v1/v2 rows. The dashboard polls the existing prmbench rows for the v2-v1 delta — no new Tauri command, no new component, no new drawer. D-28 emits the AdversarialFarmRunCompleted SSB row carrying `samples_farmed + rounds + prmbench_delta_pp + verdict + paper_floor_pp + corpus_blake3 + provenance_counts + corpus_path`; the existing dashboard surface consumes the prmbench delta INDIRECTLY by re-running the benchmark after the live PPM retrain. The new SSB row is operator-observable via existing recent-rows tables but no panel-level UI is authored this pass.
Grounding: D-27-decisions.md DECISION-2 (composite Tauri command + existing PrmBenchPanel.svelte); D-19 sibling shipped PrmBenchPanel as a drawer-mounted view.

DECISION-11 — D-19..D-27 consolidation precedent codified: closure passes ship exactly the surface required to honour the wiring contract; multi-variant SSB schemas, additional dashboard panels, and "stretch goal" components are DEFERRED to follow-up plans (e.g. D-28.1 if needed). The single-variant + scaffolding-first + zero-new-dashboards posture mirrors D-19 (PRMBench shipped 1 variant + 1 dashboard view), D-23 (Sycophancy 1 variant + reused refusal panel), D-24 (Canary 1 variant + reused canary panel), D-25 (Tamper 1 variant + reused tamper view), D-26 (Archive dashboard composite Tauri command + 0 new SSB variants), D-27 (Mode-routing dashboard composite + 0 new SSB variants). D-28's posture is "1 new SSB variant + 0 new dashboards + 3 RUNPOD-PENDING scripts" — the same proportional shape as the data-EVALUATING benches, applied to the FIRST data-PRODUCING runner.
Grounding: workflow/reasoning/D-19..D-27-decisions.md sibling chain.

KNOWN-RESIDUAL (recorded; deferred):
  - KR-1 Live PPM fine-tune wall-clock measurement (workflow/remote-setup/ppm_finetune.sh shipped as scaffold; real torchrun execution + checkpoint upload pipeline is RUNPOD-PENDING; the AdvRmFarmer::run report's prmbench_delta_pp stays at 0.0 until that lands).
  - KR-2 K=2 ensemble PPM checkpoint pair (would replace the K=1 fallback per DECISION-5; would also remove the proxy>>gold-gap caveat from adv_rm.rs:49-60).
  - KR-3 PrmBenchPanel.svelte v1-vs-v2 delta-renderer tweak to surface the AdversarialFarmRunCompleted SSB row INLINE (currently the dashboard just shows prmbench v1 + v2 side-by-side; an explicit "Adv-RM farm produced this corpus" attribution would land alongside KR-1).
  - KR-4 Live python ppm_adversarial.py mode against the production PPM (currently `--mock` mode is the CI default; `--live` mode shipped as scaffold + RUNPOD-PENDING marker only).
  - KR-5 Per-round candidate-step PERTURBATION (paper §3.2 adversarial-generation step; the host-side scaffolding re-uses the same seed set across rounds, which is the K=1 simplification matching DECISION-5; a real generation loop would land alongside KR-1).

OUT-OF-SCOPE for this pass:
  - The full arXiv:2504.06141 §3.3 K-ensemble reward-model wiring (would require a second 7B PPM checkpoint + dual-port sidecar + a `Vec<SidecarPpmHook>` aggregator; deferred per DECISION-5).
  - New dashboard panel for the adv-rm farm run report (deferred per DECISION-10).
  - benchmarks/adv_rm/corpus/ pre-seeded with hand-authored adversarial-corpus rows (the shipped state is empty + a README documenting the provenance contract; live corpus rows are written by ppm_adversarial.py at runtime).
