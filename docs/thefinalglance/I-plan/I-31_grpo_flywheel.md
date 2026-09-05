# I-31: grpo flywheel — execution-grounded rewards, blocking ledger guards, evaluator outside the write boundary

- Track 17 — OPEN-WEIGHT FLYWHEEL | lever SELF-IMPROVEMENT | grounding STRONG | nature BUILD | effort XL | WAVE 5 | LAST | binds ID-6 | IC-7 | depends I-30 + H-43/H-44/H-28 + I-24
- Premise: a self-improving loop whose evaluator it can influence is the one failure that discredits the
  entire trust claim (H-plan Track 7's convergence theorem: without an un-gameable stronger-than-generator
  filter, the loop collapses). I-31 runs GRPO on execution-grounded rewards only, behind blocking guards that
  are not optional. the lane stays behind the existing darwinian-evolve kill-switched allowlisted gate (W-2
  satisfied by the shipped allowlist row). GRPO is LAST by doctrine (IC-7). DPO fine-tuning is E-55
  HARD-AVOID and is not re-litigated here.

FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule).

## Steal/build
GRPO mechanism: group relative policy optimization (GRPO) generates K completions per prompt, scores them
with the reward endpoint, normalizes within-group, updates the policy. the reward endpoint is a typed local
gRPC/HTTP handler ON THE SUPERVISOR SIDE (I-1) — the trained policy cannot reach the reward code (the
supervisor owns the endpoint address; the policy only sees the reward value returned). reward sources:
compile gate (pass/fail, from I-17 L1), test gate (pass count / total, from I-17 L1), mutation gate
(CI-cleared score from I-20), null-test gate (I-8's halting-bound abort signal), format conformance penalty
(XGrammar conformance score, additive). no neural reward model — execution gates only (VD-1: execution-grounded
fitness). KL penalty against mode collapse: KL(policy || I-30 SFT init checkpoint) with a configurable
coefficient; when KL exceeds the threshold the update step is skipped. negative-trajectory masking: I-30's
JSONL negative pairs (fluent-but-vetoed) are injected as masked negative examples (the policy is penalized
for reproducing them). IS-2 retired: derived_alpha = blake3(config bytes) is the old byte-hash fitness;
I-31 never uses it — IS-2 was the placeholder H-43 replaced. I-24 canaries: the scoring stream carries
AST-preserving canary pairs; a run where the canary pair's scores diverge by >threshold triggers a
ScorerNeutralized event and the update step is SKIPPED for that batch. BLOCKING prerequisites (not
advisories — each must be LIVE in the codebase before I-31 executes, verified by the executor):
  1. H-43 execution-grounded fitness LIVE: no byte-hash anywhere in the loop (IS-2 retired confirmed).
  2. H-44 behavioral traps LIVE: score_candidate_for_traps (IS-3 replaced by H-44's behavioral detector).
  3. I-24 canaries in the scoring stream: PrmCanaryEvent rows present in a live scoring run.
  4. H-28 reward-tampering gate + the evaluator/harness/thresholds in the R8 frozen set (the reward endpoint
     code + the gate frozen; gate_freeze_check.py passes on the reward endpoint path).
  5. H-17 trial-N persistence + H-18 deflation gate: every promotion candidate has a persisted trial count
     and passes deflation before the checkpoint is loaded by the production policy.
  6. D-6 held-out eval corpus: the promotion gate uses held-out data the training loop never sees.
  7. D-7 cross-domain canary: the promotion gate includes a cross-domain stress test.
the live multi-night training run is a separately-gated RunPod checkbox (W-10 Tier-2): the deterministic
fixture harness ships first and exercises the real reward endpoint path, the canary injection, and the KL
penalty computation on a frozen small-fixture policy. NEW DEPS: the RL training stack (e.g. TRL, veRL, or a
custom trainer for GRPO) is a new dep — anti-slopsquat protocol is MANDATORY: verify crate/package name +
repo URL + last-published date + license, USER-CONFIRMED before any import. the reward endpoint server is
in-process (no external dep for the endpoint itself; the endpoint is a plain Axum/Actix handler on the
supervisor side). EXCLUDED: DPO fine-tuning (E-55 HARD-AVOID — not re-litigated); PPO (GRPO is the chosen
variant per the research corpus; PPO's value-network overhead is not justified for this scale); byte-hash
fitness anywhere in the loop (IS-2 retired, H-43's replacement is mandatory); evaluator inside the loop's
write boundary (R8 — the reward endpoint and all gate code is frozen read-only to the training run).

## Existing ground truth
- IS-2: derived_alpha = blake3(config bytes) at darwin_admit_loop.rs:202 — the OLD byte-hash fitness
  placeholder. H-43 replaced it; I-31 inherits H-43's execution-grounded fitness. IS-2 is retired; I-31
  must confirm H-43 is LIVE before executing (blocking prerequisite 1 above).
- IS-3: score_candidate_for_traps keyword grep at trap_tasks.rs:123 — the OLD keyword-grep trap. H-44
  replaced it with behavioral detection. I-31 composes H-44; IS-3 is retired as I-31's anchor (blocking
  prerequisite 2 above).
- IS-4: config_archive.rs (archival + parent selection) — the LoRA checkpoint is a config-class artifact per ID-6; I-31 uses the same archive for checkpoint lineage.
- IS-16: H-17 trial_ledger.rs, H-18 deflation_gate.rs, H-28 gate files (reward-tampering gate — PLAN-asserted; executor re-greps per W-7), D-6 holdout_budget.rs — the blocking gate chain.
- IS-19: wiring tooling + R8 frozen-gate (reward endpoint + gate code frozen; gate_freeze_check.py is the enforcement tool).
- I-30 (BLOCKING prerequisite): the SFT init checkpoint I-31 starts from; I-30's JSONL negative pairs.
- I-24 (BLOCKING prerequisite): the PRM canary anti-gaming system in the scoring stream.
- H-43..H-48 (Track 7 spine — BLOCKING): execution-grounded fitness (H-43), behavioral traps (H-44), worst-case scoring (H-45), clade selection (H-46), eval firewall (H-47), bandit on real reward (H-48). compose, never re-author.
- D-5/D-29 bandit (composes): the bandit that governs exploration of policy variants; I-31 feeds it reward signals.
- E-55 SFT-on-rollouts (KTO/RLVR notes inherited): the plan-level context; DPO HARD-AVOID stands.
- darwinian-evolve kill-switched allowlist row in wire-dark-allowlist.toml: the W-2 gate I-31 stays behind.

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/training/grpo_harness.rs (~200 LOC — reward endpoint (Axum handler on supervisor side), KL penalty
  computation, canary injection hook, negative-trajectory masking, GrpoRewardComputed SSB emission)
- gate_freeze_check.py run over the reward endpoint path + gate files (R8 enforcement at ship time)
- BLOCKING prerequisite checklist in the decision doc (each prerequisite verified by executor before EXECUTING)
- contract test crates/contracts/tests/i31_grpo_flywheel_contract.rs (fixture: a seeded reward-hack attempt
  is HARD-BLOCKED by H-28 gate; a promotion candidate without trial-N is rejected; byte-hash fitness absent)
- decision doc workflow/reasoning/I-31-decisions.md

DEFER (Tier-2 / gated — RunPod checkbox; user approval required for RL dep):
- RL training stack dep (anti-slopsquat verify + user-confirm crate name/version/license first)
- live multi-night GRPO training run (RunPod; requires approved dep + live reward endpoint)
- checkpoint promotion through the full IS-16 ledger chain (requires a training run to have produced a checkpoint)
- online bandit integration with D-5/D-29 (requires volume from live training runs)

## Integration
- CREATE: crates/training/grpo_harness.rs (~200 LOC)
- CREATE: crates/training/ workspace crate (if absent — PLAN-asserted; executor re-greps per W-7; if the crate exists, MODIFY it)
- MODIFY: wire-dark-allowlist.toml (reuse the darwinian-evolve row; verify it covers the GRPO lane or add a promotion row per R9)
- seams from the map: Track 17 anchor — IS-2/IS-3 (via H-43/H-44) + IS-16 + D-6 + E-55
- frontend_pair: flywheel status panel (admits: checkpoints entering the ledger chain; blocks: H-28 rejections; KL trend per training episode). cross_lang_diff.py covers Tauri -> Svelte. Justified: the panel is the W-6 surface; the trust claim depends on operator visibility of the blocking guard state.

## WIRING_CONTRACT
produces_symbols:
  - crates/training/grpo_harness.rs::GrpoHarness (reward endpoint handler + KL penalty + canary hook + masking)
  - crates/training/grpo_harness.rs::RewardEndpoint (Axum handler, supervisor-side — policy cannot reach it)

consumes_symbols:
  - H-43 execution-grounded fitness (the LIVE replacement for IS-2 byte-hash; BLOCKING — must be confirmed LIVE before executing) — GD-1 / VD-1
  - H-44 behavioral traps (LIVE replacement for IS-3 keyword-grep; BLOCKING) — GD-1
  - H-28 reward-tampering gate (gate frozen in R8 set; gate_freeze_check.py verifies) — GD-1 / ID-6
  - I-24 PRM canary anti-gaming (canary injection hook in the scoring stream; BLOCKING) — GD-1
  - IS-16 gate files (trial_ledger.rs / deflation_gate.rs / holdout_budget.rs / gate_self_validation.rs — the full ID-6 promotion chain) — ID-6
  - I-30 SFT init checkpoint + negative pairs (the policy init point and the masked negatives) — GD-1
  - D-5/D-29 bandit (reward signal consumer; I-31 feeds it per-episode reward; bandit governs exploration) — GD-1
  - E-55 KTO/RLVR notes (inherited policy constraints; DPO HARD-AVOID) — GD-1

reachable_from:
  - `elai train grpo --init <sft-checkpoint>` (CLI entrypoint, Tier-2 RunPod gated)  [PRODUCTION training surface]
  - Tier-1 contract test exercises RewardEndpoint directly (in-process); the Tier-2 production path is the R7 evidence
  - GrpoRewardComputed rows in SSB are the off-agent artifacts; H-28 gate produces PromotionBlocked rows as the anti-gaming evidence

stub_ban:
  - NO byte-hash fitness anywhere in the loop (IS-2 retired; the contract test greps the harness for 'derived_alpha' or 'blake3(config' — MUST count 0)
  - NO evaluator (reward endpoint + gate code) inside the loop's write boundary — R8 enforced by gate_freeze_check.py
  - NO DPO fine-tuning (E-55 HARD-AVOID — not re-litigated; DPO is structurally absent from the training pipeline)
  - NO promotion without the full IS-16 ledger chain (trial-N + deflation + held-out + D-7 canary — PromotionLedgered row must cite all four)
  - NO RL dep added without anti-slopsquat verification + user approval first
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2

frontend_pair:
  flywheel status panel (admits / blocks / KL trend). GrpoRewardComputed(RewardServed / PromotionBlocked / PromotionLedgered) rows feed the panel. cross_lang_diff.py covers Tauri -> Svelte. Justified: the blocking guard state is the trust claim's user-visible evidence; W-6 is not waivable.

observable_effect:
  trigger: "(Tier-1) python .claude/tools/wiring_contract_check.py --bn-file ... --run-trigger (contract test + gate_freeze_check.py); (Tier-2, RunPod) elai train grpo --init <sft-checkpoint> --fixtures <fixture-set>"
  expect_ssb: "GrpoRewardComputed(RewardServed) rows citing execution gate verdicts (each row links to a TriLayerVerdict / MutationBudgetedRun row); GrpoRewardComputed(PromotionBlocked) for a seeded reward-hack candidate (H-28 fires, journal-verified); GrpoRewardComputed(PromotionLedgered) for a genuine improvement (cites trial-N + deflation + held-out + D-7 canary row ids)"
  expect_artifact: "NEGATIVE: byte-hash fitness structurally absent (contract test rg 'derived_alpha|blake3.config' grpo_harness.rs — MUST count 0); NEGATIVE: a seeded harness-edit fixture is HARD-BLOCKED by H-28 with PromotionBlocked (not PromotionLedgered — the gate must reject the tampered checkpoint); NEGATIVE: a promotion without a complete IS-16 ledger chain does not produce a PromotionLedgered row"

## Grounding
- IS-2 (retired by H-43), IS-3 (retired by H-44), IS-4 (archival precedent), IS-16 (H-gate files), IS-19 (wiring tooling + R8).
- H-43..H-48 (Track 7 verification spine): the blocking prerequisites and the convergence theorem grounding.
- IC-7 (mandatory sequencing: SFT first, GRPO LAST; no DPO).
- ID-6 (weights-are-configs): every checkpoint promotes through IS-16; evaluator outside write boundary (R8).
- VD-1..VD-5 (H-plan Track 7 delta family): execution-grounded fitness (VD-1), behavioral detectors (VD-2), worst-case scoring (VD-3), stronger-than-generator verifier (VD-4), undecidable residue (VD-5) — all bind here.
- TD-1..TD-5 (H-plan Track 7 companion): persisted N (TD-1), frozen evaluator (TD-2), LLM-is-advisory (TD-3), verify-the-verifiers (TD-4), honest residue (TD-5) — all bind here.
- W-2 (kill-switch posture): the darwinian-evolve allowlisted lane is the W-2 mechanism; I-31 inherits it.
- E-55 HARD-AVOID (DPO): inherited; not re-grounded here.
- SSB: +GrpoRewardComputed (Kind: RewardServed / PromotionBlocked / PromotionLedgered). baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).
- arXiv IDs: GRPO algorithm (WebSearch-confirm the primary citation before headlining; the research corpus cites it but the specific arXiv ID must be verified before use as a headline number per feedback_paper_citation_verify).

## COMPOSES-WITH
- H-43..H-48 (Track 7 loop spine): do NOT re-author these. they are the spine; I-31 is the consumer. H-43 provides execution-grounded fitness; H-44 provides behavioral traps; H-45 worst-case scoring; H-46 clade selection; H-47 eval firewall; H-48 bandit on real reward.
- I-30 SFT init checkpoint + negative pairs: do NOT re-implement the dataset pipeline; consume I-30's output.
- I-24 PRM canaries: do NOT re-implement canary injection; hook the existing injection point in the L2 scoring stream.
- H-28 reward-tampering gate: do NOT re-implement the gate; call it as the promotion guard; its code is frozen (R8).
- IS-16 gate files (H-17/H-18/D-6 etc.): do NOT re-implement ledger logic; the harness calls these gates at promotion time.
- D-5/D-29 bandit: do NOT re-implement the bandit; feed it the per-episode reward signal.
- darwinian-evolve allowlist row: do NOT create a new gate mechanism; the existing W-2 allowlist row covers the lane.
- E-55 (KTO/RLVR policy context): do NOT re-derive the policy constraints; inherit them verbatim.

End of I-31.
