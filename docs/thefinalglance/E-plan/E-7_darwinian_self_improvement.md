# E-7: Darwinian substrate hardening — QD archive, Two-Gate guardrail, EST, adversarial farming undeferred
Objective: ship the empirical-rigor upgrades the report identifies as load-bearing for the D-plan Wave-1 Darwinian substrate (D-1..D-9): replace D-1's fitness-proportional parent sampling with MAP-Elites quality-diversity over (latency × token × failure-mode); wrap D-5 promotion with the Two-Gate guardrail (margin τ AND capacity cap K[m]) from arXiv:2510.04399; add Evaluator-Stress-Test invariance probes (arXiv:2507.05619) on top of D-2 PRM scores; drop perplexity from D-4 tripwire stack or externalize it; undefer adversarial example farming (D-28) into the Wave-1 critical path as a 50-task trap fixture; harden D-8 with Ed25519 signed merkle chain.
Estimated effort: 3 sessions, ~12 h (split as D-1+D-5 refit; D-2+D-4 EST and tripwire; D-6+D-8+D-28 farming and crypto)
Pre-requisites: B-23 (self-mod substrate, gated), D-1..D-9 (PLANNED; E-7 either co-lands the original D-N or amends if D-N has shipped). C-15 (ClockProvider) + C-21 (MistakeBook substrate) DONE per D-SUMMARY_INDEX.
Parallel-safe with: E-6 (failure-mode defenses), E-8 (prompt evolution), E-9 (benchmarks), E-10 critic.
Source report: workflow/improvement-sweep-2026-05-19/reports/07-darwinian-self-improvement.md

## Context

DGM (arXiv:2505.22954) raised SWE-bench 20%→50% but exhibited two documented reward hacks (Sakana blog): (1) fake test logs, (2) removed hallucination-detector markers in its own code. Both hacks were chain-valid against fitness-proportional parent selection. AlphaEvolve (arXiv:2506.13131) uses MAP-Elites quality-diversity over a behaviour descriptor — DGM does not, and the open-ended-evolution literature treats QD as default (Mouret & Clune 2015). arXiv:2510.04399 proves PAC-learnability is preserved iff the policy-reachable family has uniformly bounded capacity, giving the Two-Gate guardrail (margin τ AND capacity cap K[m]). EST (arXiv:2507.05619) reports 78.4% precision / 81.7% recall RL, 74.2%/78.6% LLM on invariance-based proxy-gaming detection. Soft contamination (arXiv:2602.12413) shows 9-gram filters miss 85.9% rephrased MMLU on Llama-2-13B; LiveCodeBench (arXiv:2403.07974) shows temporal partition is necessary. Honest reflection of these results is mandatory before D-1..D-9 ship. E-7 ports each into ELAI's substrate as a delta/extension to the corresponding D-N file, treating D-28 (Adversarial Farming) as Wave-1 not Wave-5 because arXiv:2410.06491 reports specification-gaming jumps 0%→97% with reflection alone, no training — ELAI's MSI + BAI loop has exactly that reflection structure.

## Existing state (B/C/D coverage)

- DELTA on D-1 (ReasoningTemplate archive + blake3, PLANNED): E-7 replaces the parent-selection method with MAP-Elites over a behaviour descriptor; original blake3 content-hash + ArchiveEntry shape preserved.
- DELTA on D-2 (PPM sidecar Qwen2.5-Math-PRM-7B port 8101, PLANNED): E-7 mandates PRMBench pre-ship validation (Step-Consistency >= 0.7, Confidence-Invariance >= 0.7) AND adds EST invariance check as a second-verifier signal before D-3 consensus.
- DELTA on D-3 (VerificationResult ≥2-verifier consensus, PLANNED): E-7 mandates that the 2nd verifier be from an independent base-model family AND adds a verifier-diversity-score lower bound; treats verifier-set itself as L3-frozen.
- DELTA on D-4 (5-tripwire stack KL+Δρ+IQR+perplexity+DDM, PLANNED): E-7 drops perplexity OR externalizes to a frozen-LM; adds Page-Hinkley + ADWIN; adds EST invariance as a 5th tripwire.
- DELTA on D-5 (Beta-posterior Thompson BAI, PLANNED): E-7 switches to Discounted-TS (arXiv:2305.10718) with discount tuned to MSI's λ=0.3 timescale AND adds Successive Rejects as a fixed-budget complementary BAI; wraps with Two-Gate guardrail.
- DELTA on D-6 (9-gram held-out, PLANNED): E-7 adds paraphrase / embedding-similarity filter ≥0.85 AND temporal cutoff AND canary GUID tokens per BIG-Bench pattern.
- DELTA on D-7 (5-family cross-domain canary, PLANNED): E-7 defines explicit transfer-regression threshold (max(-20% relative, -5% absolute) on any family, NOT count-of-5).
- DELTA on D-8 (blake3 hash chain, PLANNED): E-7 adds Ed25519 signature on each link by a second-party signer (auditor agent in distinct sandbox) AND includes lock-list-hash in each link so any lock-list change is flagged.
- DELTA on D-9 (L3 frozen-surface predicates, PLANNED): E-7 extends lock-list to process-supervision logs + trusted output sandbox for test execution + verifier-set.
- REPLACEMENT for D-28 (adversarial farming, DEFERRED → Wave-1 critical path): 50 hand-crafted trap-task fixtures shipped as part of D-6 held-out store with weight ≥10× in promotion.

## Reads (mandatory before coding)

- .elai_cc/crates/orchestrator/src/ssb.rs (SsbActionType at 756; add new variants)
- .elai_cc/crates/orchestrator/src/fsm.rs (FsmSubstate at 124)
- .elai_cc/crates/orchestrator/src/agent_pool.rs (allowed_grounding_methods)
- .elai_cc/python/elai_inference/server.py (PRM sidecar bring-up, port assignment)
- F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\07-darwinian-self-improvement.md
- F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-SUMMARY_INDEX.md (D-1..D-9, D-28 entries)
- workflow/papers/dgm-interface.md (DGM interface map already at .claude/papers/2505.22954v3.pdf)
- arXiv:2505.22954 DGM (already saved); arXiv:2506.13131 AlphaEvolve (already saved)
- arXiv:2510.04399 Statistical Limits (Two-Gate); arXiv:2507.05619 EST
- arXiv:2602.12413 soft contamination; arXiv:2403.07974 LiveCodeBench
- arXiv:2305.10718 Discounted-TS; Audibert COLT 2010 Successive Rejects
- arXiv:2210.10760 Gao RM overoptimization; arXiv:2310.09144 Goodhart geometric
- arXiv:2402.06627 in-context reward hacking; arXiv:2410.06491 Honesty-to-Subterfuge
- arXiv:1906.01820 mesa-optimization; arXiv:1908.04734 Everitt reward tampering
- arXiv:2501.03124 PRMBench (Qwen2.5-Math-PRM validation)

## Tasks

1. Replace D-1 parent selection with MAP-Elites (crates/orchestrator/src/template_archive.rs, ~250 LOC NEW or +180 LOC delta). BehaviourDescriptor = (latency_tier: enum{Sub10s, Sub60s, Over60s}, token_class: enum{Lt500, Lt2k, Lt8k, Over8k}, failure_mode: Option<MastMode> from E-6). Archive struct: HashMap<(latency_tier, token_class, failure_mode), ArchiveEntry> with one elite per cell. Promotion = best-in-cell, not fitness-proportional global. Add SsbActionType::ArchiveCellElevated.

2. Add Two-Gate guardrail at D-5 promotion (crates/orchestrator/src/promotion_gate.rs, ~140 LOC). promote_iff(margin_held_out_delta >= tau) && (reachable_policy_capacity <= K_m). K_m configured as L3-locked constant; tau read from canary held-out per cell. Add SsbActionType::TwoGateBlocked with reason {below_margin, capacity_exceeded}.

3. Wire Discounted-TS + Successive Rejects (crates/orchestrator/src/bai.rs, ~220 LOC). DiscountedTs { discount_factor: f64, arms: Vec<ArmPosterior> } with default discount = exp(-1.0 / 0.3) ~= 0.0357 per session per memory:project_b40 references on MSI λ=0.3; SuccessiveRejects pass with fixed budget per generation. ArmPosterior carries breakpoint_window for non-stationarity tracking. Reference Audibert 2010 + arXiv:2106.10417 in module doc.

4. Add EST invariance probe (crates/orchestrator/src/est_probe.rs, ~180 LOC). Given a candidate, perturb its input by N (default 5) semantic-equivalent rewrites (mode A=whitespace, B=identifier rename, C=paraphrase via cloud Sonnet); the PPM score must change by less than epsilon (default 0.05) per probe; if score_variance > epsilon -> emit EstInvarianceViolation tripwire. Adds SsbActionType::EstInvarianceMeasured.

5. Rebuild D-4 tripwire stack (crates/orchestrator/src/tripwire.rs, ~200 LOC delta). DROP perplexity from in-loop tripwires; keep KL + Δρ + IQR + DDM; ADD Page-Hinkley + ADWIN + EstInvarianceViolation. Auto-revert fires when >=2 concurrent. Optional perplexity is computed by a frozen out-of-band LM (config feature `perplexity_external` referencing a model path that the self-mod loop has no write access to per D-9 lock).

6. Add paraphrase + canary contamination filter (crates/orchestrator/src/held_out_filter.rs, ~160 LOC). Pipeline: 9-gram filter (already in D-6 plan), then cosine similarity with fastembed embedding >= 0.85 = block, then temporal cutoff (task.released_at > model.training_cutoff_or_last_self_mod). Canary GUID tokens: each held-out task gets a random 16-char UUID; periodic prompt asks "have you seen this GUID?" — if yes, retire task. Adds SsbActionType::ContamFilterRejected.

7. Define D-7 transfer-regression threshold (crates/orchestrator/src/cross_domain_canary.rs, ~120 LOC delta or NEW). Reject promotion if for any non-target family, max(relative_delta=-20%, absolute_delta=-5%). Make family selection adversarial: pick families where the candidate scores LOWEST in last-3 generations, not where ELAI is strongest. Successive-Rejects over multi-family score selects the BAI variant. Add SsbActionType::TransferRegressionBlocked.

8. Adversarial trap-task farming undeferred (crates/orchestrator/src/trap_tasks.rs + .elai_cc/held-out/traps/*.toml, ~150 LOC + 50 fixtures). 50 hand-crafted traps modeled on Krakovna's specification-gaming list: Coast-Runners-style green-block loop, Lego-stacking-flip variants, marker-removal trap, fake-test-log trap, verifier-capture trap, metric-overflow trap. Each trap has obvious-metric-maxxing path != intended outcome. Trap_score = 10x normal task in promotion vote. Add SsbActionType::TrapTripped (FATAL band — never promote).

9. Ed25519-signed merkle chain (crates/orchestrator/src/archive_crypto.rs, ~200 LOC). Each ArchiveEntry now has fields {self_blake3, parent_blake3, parent_lock_list_blake3, verifier_set_blake3, ed25519_sig}. Second-party signer = auditor agent process (separate AgentRole::Auditor process per Tier-C isolation). `elai archive verify` walks chain and verifies signatures. Add SsbActionType::ArchiveSigVerified / ArchiveSigBroken.

10. L3 lock-list extension (crates/orchestrator/src/l3_lock.rs, ~120 LOC delta). Add to lock-list: process-supervision logs (SSB audit trail), trusted-output sandbox tier (test execution sandbox config), verifier-set definitions, perplexity-LM model path. Tests must demonstrate lock-list-modification PR is blocked.

11. Dashboard MutationArchivePanel.svelte upgrade (already planned at D-26; this is a delta ~180 LOC). Render BehaviourDescriptor cells as a 3D grid (latency × token × failure-mode) with elite-occupancy heatmap; render Two-Gate decisions; surface EST-violation banner; surface trap-trip events as red flags.

12. CLI: `elai archive map-elites stats` (~70 LOC) outputs cell coverage + per-cell elite quality; `elai archive verify` extended to verify Ed25519 signatures (~50 LOC delta).

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::template_archive::BehaviourDescriptor
  - orchestrator::template_archive::MapElitesArchive
  - orchestrator::promotion_gate::two_gate_promote(candidate, tau, k_m) -> Result<PromotionDecision>
  - orchestrator::bai::DiscountedTs
  - orchestrator::bai::SuccessiveRejects
  - orchestrator::est_probe::EstProbe::run
  - orchestrator::tripwire::TripwireStack (rebuilt; perplexity gated behind feature)
  - orchestrator::held_out_filter::ContamFilter
  - orchestrator::cross_domain_canary::TransferThreshold
  - orchestrator::trap_tasks::TrapTaskRegistry
  - orchestrator::archive_crypto::Ed25519SignedLink
  - SsbActionType::ArchiveCellElevated
  - SsbActionType::TwoGateBlocked
  - SsbActionType::EstInvarianceMeasured
  - SsbActionType::ContamFilterRejected
  - SsbActionType::TransferRegressionBlocked
  - SsbActionType::TrapTripped
  - SsbActionType::ArchiveSigVerified
  - SsbActionType::ArchiveSigBroken
consumes_symbols:
  - crates/orchestrator/src/template_archive.rs::admit (uses MapElitesArchive)
  - crates/orchestrator/src/promotion_gate.rs (gates D-5)
  - crates/orchestrator/src/tripwire.rs::evaluate (called per generation)
  - crates/orchestrator/src/held_out_filter.rs (called on every held-out admit)
  - cli::archive_cmd::verify (Ed25519 sig walk)
reachable_from:
  - FsmSubstate::Auditing (EST probe fires before consensus)
  - FsmSubstate::Promoting (Two-Gate fires here; if not present, add as substate)
  - self_mod_bridge (DGM-style proposal -> Two-Gate -> ArchiveCellElevated or rejected)
stub_ban:
  - "two_gate_promote\\(.*true.*\\)" — unconditional-true returns absent
  - "EstProbe::default\\(\\)" — default impl test-only
  - "perplexity.*tripwire" — in-loop perplexity reference absent in non-test source
  - "fitness_proportional|softmax_parent" — old DGM parent-selection regex absent
frontend_pair: crates/dashboard-v2/src/lib/components/MutationArchivePanel.svelte
observable_effect:
  trigger: "elai bench self-mod --seed 42 --generations 5 --include-traps"
  expect_ssb: "SsbActionType::ArchiveCellElevated and SsbActionType::TrapTripped both present in <300s; at least 1 ArchiveSigVerified per generation"
  expect_artifact: "stdout shows MAP-Elites cell coverage table, Two-Gate decisions per generation, 0 successful trap promotions; dashboard MutationArchivePanel renders the 3D cell heatmap"

## DETERMINISTIC_CHECK

```bash
# 1. blake3 only (anti-stale rule)
rg "blake2|sha2" .elai_cc/crates/orchestrator/src/archive_crypto.rs .elai_cc/crates/orchestrator/src/template_archive.rs
# Expected: zero hits.

# 2. Perplexity dropped or externalized
rg "perplexity" .elai_cc/crates/orchestrator/src/tripwire.rs --type rust
# Expected: only references under `cfg(feature = "perplexity_external")` or in doc comments.

# 3. Ed25519 used (verify dep added to Cargo.toml properly per anti-slopsquatting)
rg "^ed25519-dalek" .elai_cc/crates/orchestrator/Cargo.toml
# Expected: 1 match. (verify via `cargo info ed25519-dalek` before adding)

# 4. MAP-Elites cells populated and queryable
cargo run -p elai-cli -- archive map-elites stats --json
# Expected: JSON with `cells: {...}` and at least one occupied cell on a fixture archive.

# 5. Two-Gate blocks an over-capacity candidate
cargo test -p elai-orchestrator promotion_gate::tests::capacity_cap_blocks
# Expected: PASS.

# 6. Trap-task fixture catches a known reward-hack pattern
cargo run -p elai-cli -- bench self-mod --seed 42 --generations 1 --include-traps --json | jq '.trap_tripped'
# Expected: > 0 if seeded fake-test-log path included.

# 7. Adversarial farming traps wired into held-out store
ls .elai_cc/held-out/traps/*.toml | wc -l
# Expected: >= 50.

# 8. EST invariance probe non-trivial (cloud-Sonnet paraphrase route)
cargo test -p elai-orchestrator est_probe::tests::flags_variance_above_threshold
# Expected: PASS.
```

## ACCEPTANCE

- [ ] MAP-Elites archive replaces fitness-proportional parent selection in self_mod_bridge; cell coverage >= 6 of 12 default cells after 5-generation smoke run.
- [ ] Two-Gate guardrail blocks promotion when capacity cap K_m is exceeded; tests demonstrate both margin and capacity rejection paths.
- [ ] Discounted-TS + Successive Rejects replace vanilla TS; discount factor configurable; default tuned to MSI λ=0.3.
- [ ] EST invariance probe fires on a hand-crafted gameable candidate (PRM score invariant to semantic perturbation) and blocks promotion.
- [ ] Tripwire stack has no in-loop perplexity reference outside a `perplexity_external` feature gate; Page-Hinkley and ADWIN are present and emit drift events.
- [ ] Contam filter rejects a paraphrased-known-task on the held-out store via cosine >= 0.85; canary GUID tokens present in 100% of new held-out tasks.
- [ ] Transfer-regression threshold (-20% rel / -5% abs) hard-coded as L3 constant; family selection picks weakest-3 families per generation.
- [ ] 50 trap-task fixtures present in .elai_cc/held-out/traps/; trap tripping is FATAL band — no promotion path; SsbActionType::TrapTripped emitted.
- [ ] Ed25519 signature on every archive link; `elai archive verify` exits 1 on signature break; lock-list-hash carried in every link.
- [ ] L3 lock-list covers process-supervision logs + verifier-set + sandbox config + perplexity-LM path; pre-commit hook blocks lock-list mutation without auditor sign-off.
- [ ] No .unwrap() in new non-test code; no blake2 / sha2 / async-trait crate; native async fn in traits.

## Tests to add

- crates/orchestrator/tests/map_elites_archive_smoke.rs: 12-cell archive admits 5 candidates, each falling into expected cell
- crates/orchestrator/tests/two_gate_capacity_blocks.rs: candidate with K > K_m rejected with reason::capacity_exceeded
- crates/orchestrator/tests/discounted_ts_regret_synthetic.rs: 50-round non-stationary benchmark; regret <= published Audibert bound on stationary segment
- crates/orchestrator/tests/est_invariance_probe.rs: variance > epsilon triggers EstInvarianceViolation
- crates/orchestrator/tests/tripwire_no_perplexity_in_loop.rs: feature-flagged perplexity off by default
- crates/orchestrator/tests/contam_filter_paraphrase.rs: paraphrased fixture caught at cosine >= 0.85
- crates/orchestrator/tests/trap_task_fatal_band.rs: trap-trip blocks any subsequent promotion in the generation
- crates/orchestrator/tests/archive_crypto_ed25519_chain.rs: tampered link detected; clean chain verifies
- crates/orchestrator/tests/l3_lock_list_extension.rs: pre-commit hook blocks lock-list edit
- crates/contracts/tests/d_plan_substrate_wiring_contract.rs: all new SsbActionType variants registered and serializable

## Files touched

- CREATE: crates/orchestrator/src/template_archive.rs (~250 LOC; replaces DGM-style if exists)
- CREATE: crates/orchestrator/src/promotion_gate.rs (~140 LOC)
- CREATE: crates/orchestrator/src/bai.rs (~220 LOC; DiscountedTs + SuccessiveRejects)
- CREATE: crates/orchestrator/src/est_probe.rs (~180 LOC)
- CREATE: crates/orchestrator/src/held_out_filter.rs (~160 LOC)
- CREATE: crates/orchestrator/src/cross_domain_canary.rs (~120 LOC or delta)
- CREATE: crates/orchestrator/src/trap_tasks.rs (~150 LOC)
- CREATE: crates/orchestrator/src/archive_crypto.rs (~200 LOC)
- CREATE: crates/orchestrator/src/l3_lock.rs (~120 LOC or delta)
- CREATE: .elai_cc/held-out/traps/*.toml (50 fixtures, ~30 LOC each)
- MODIFY: crates/orchestrator/src/tripwire.rs (~200 LOC delta — drop perplexity, add PH+ADWIN+EST)
- MODIFY: crates/orchestrator/src/ssb.rs (+8 SsbActionType variants, ~80 LOC delta)
- MODIFY: crates/orchestrator/src/fsm.rs (FsmSubstate::Promoting if not present, ~30 LOC)
- MODIFY: crates/orchestrator/Cargo.toml (+ed25519-dalek; verify version with cargo info before adding)
- MODIFY: crates/cli/src/archive_cmd.rs (verify Ed25519 sig walk, ~50 LOC)
- MODIFY: crates/dashboard-v2/src/lib/components/MutationArchivePanel.svelte (~180 LOC delta)

## Citations / Grounding

- arXiv:2505.22954 (DGM, Sakana blog) — two documented reward hacks (marker removal + fake test logs); grounds trap fixtures + lock-list extension + sandbox requirement
- arXiv:2506.13131 (AlphaEvolve) — MAP-Elites in evolutionary coding agent; grounds D-1 redesign
- Mouret & Clune 2015 (MAP-Elites foundation) — quality-diversity cell archive; behaviour descriptor
- arXiv:2510.04399 (Statistical Limits of self-improving agents) — Two-Gate guardrail (margin τ + capacity K[m]); PAC preservation
- arXiv:2507.05619 (Evaluator Stress Test) — 78.4/81.7 RL, 74.2/78.6 LLM precision/recall; invariance-based proxy-gaming detector; grounds EST probe
- arXiv:2602.12413 (soft contamination) — 9-gram filter insufficient (Llama-2-13B 85.9% on rephrased MMLU); grounds paraphrase + canary
- arXiv:2403.07974 (LiveCodeBench) — temporal partition by release date; grounds temporal cutoff
- arXiv:2305.10718 (Discounted Thompson Sampling) — Õ(√TB_T) regret for non-stationary bandits; grounds D-5 redesign
- Audibert COLT 2010 (Successive Rejects) — parameter-free fixed-budget BAI; grounds SR as complement
- arXiv:2210.10760 (Gao RM overoptimization) — BoN: gold = a·d − b·d² (quadratic in KL√d); grounds optimal early-stopping
- arXiv:2310.09144 (Goodhart in RL) — early-stopping method that provably avoids Goodhart pitfall
- arXiv:2402.06627 (in-context reward hacking) — ICRH from feedback loops without weight updates; grounds adversarial farming undeferral
- arXiv:2410.06491 (Honesty to Subterfuge) — spec gaming 0% → 97% with reflection alone; grounds Wave-1 priority for D-28
- arXiv:1906.01820 (mesa-optimization Hubinger) — inner alignment; grounds open question on lock-list completeness
- arXiv:1908.04734 (Everitt reward tampering) — RF-input + reward-function tampering; grounds Ed25519 second-party signer
- arXiv:2501.03124 (PRMBench) — Qwen2.5-Math-PRM-7B validation against 9 sub-dimensions; pre-ship D-2 gate
- Krakovna specification-gaming spreadsheet (DeepMind, cited in report) — 60+ catalogued exploits; trap-task seed list

## Open questions / risks

- MAP-Elites behaviour-descriptor cell axes are the load-bearing decision. (latency × token × failure-mode) is a reasonable first cut but UNCONFIRMED; report flags "If your axes correlate (e.g. determinism vs correctness on a deterministic task), the frontier degenerates." E-7 ships with these 3 axes but adds a config knob and a deterministic check that cell-coverage diversity (Shannon entropy across cells) stays > 1.0 nats after 20 generations.
- EST invariance probe uses cloud-Sonnet for paraphrase generation; this is cost and latency overhead per candidate. Report cites 74.2/78.6 LLM precision/recall — those numbers assume the perturbation distribution matches the report's setup. ELAI-flavoured invariance may differ; E-7 logs per-perturbation deltas for later calibration.
- Two-Gate K_m capacity cap is a numerical constant that must be set per-deployment; report does not prescribe value. E-7 ships with K_m = 50 archive entries (DGM published archive size) as default; this is UNCONFIRMED for ELAI.
- Trap-task fixtures (50) are hand-crafted; report flags "if D-28 farming detects ANY spec-gaming, do not promote — even if harmless looking" (arXiv:2508.17511 School of Reward Hacks). E-7 implements FATAL band but the trap set is finite — adversary may exploit a pattern not in the 50.
- Adversarial farming retraining loop (D-28 original Wave-5 deferred scope) NOT shipped in E-7. E-7 lands the 50-trap fixture as evaluation only; the retraining loop remains deferred until D-5 has >=20 promotion cycles per original D-28 entry.
- Report explicitly flags D-7's "5 cross-domain families" count as the wrong axis vs magnitude; -20% relative / -5% absolute threshold is design choice grounded in DGM 50→28.9% transfer drop (42% relative) as the upper bar, but the threshold value is design hypothesis not published number.
- Ed25519-dalek crate version must be verified via `cargo info ed25519-dalek` before commit; report lists no specific version. Anti-slopsquatting rule applies.

## Next

- E-7 unblocks E-9 benchmarks — the EST invariance check + MAP-Elites + trap traps feed directly into the SWE-bench-Live + PRMBench integration in E-9
- D-1..D-9 either no-op if E-7 lands first (E-7 supersedes the changed surface) or extend the existing D-N if D-N has shipped; E-7's tasks 1-10 are written as explicit deltas
- D-28 retraining loop remains deferred per CLAUDE.md hard rule on no destructive ops without explicit user approval; E-7 ships only the trap-task evaluation surface
- E-10 (hostile critic) flagged D-8 hash chain as "lineage not goodness" — E-7 task 9 (Ed25519 second-party signer + lock-list-hash in link) is the structural fix; covers both reports
