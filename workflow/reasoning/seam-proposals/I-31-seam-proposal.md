# I-31 GRPO flywheel — seam-design proposal (DOC-ONLY)

session: opuswalk-I-31-PLAN-1786170946-b2ec01  date: 2026-08-08
actor: synthesizer-elai on posthog-fable-5  role: PLAN  writes: this doc only
base: worktree HEAD a2744e5 (3727 tracked files; same HEAD as /home/user/teikoku/elai main —
verified `git log --oneline -1` on both this session). Every seam below verified against this
tree with grep/sed/read this session, not from plan memory.
plan: docs/thefinalglance/I-plan/I-31_grpo_flywheel.md (147 ln, read in full)
NOTE: /home/user/teikoku/conductor/opus_walk/evidence/I-31/ is permission-denied to this worker
(touch probe failed: "Permission denied", same as the I-30 run). The complete work product
therefore lives in THIS file, the one allowed write. Section 6 carries the L2/L3 digest a
scoped-mount consumer needs.

## 1. Queue reason + gate evidence restatement

Queue reason (TaskSpec verbatim): "genuine unshipped BUILD (GRPO flywheel / GrpoHarness absent)
but plan_gate_check exit 1 (missing EXPECTED_OBSERVATION/FORK/ABORT/VERIFY...); refine-first +
design needed."

Re-run this session (off plan file at HEAD):
`python3 .claude/tools/plan_gate_check.py --file docs/thefinalglance/I-plan/I-31_grpo_flywheel.md --json`
=> verdict FAIL, exit 1. Per point:

- P1 EXPECTED_OBSERVATION present — fail (count 0; content exists as lowercase
  `observable_effect:` at plan ln 120, wrong header shape)
- P2 ON_FAIL pairs every move — pass
- P3 FORK trigger or FORKS: none — fail
- P4 RECON_NEEDED / GROUNDING coverage — pass (## Grounding at plan ln 125)
- P5 ABORT conditions exist — fail (substance exists in EXCLUDED ln 44-48 + stub_ban ln 109-116)
- P6 VERIFY runs with off-agent pass artifact — fail ("no VERIFY/VERIFICATION marker found")
- P7 REDTEAM block records an attack — fail
- P9 vertical slice names PRODUCTION_HOOK/WIRING_CONTRACT — pass (## WIRING_CONTRACT ln 89)
- P10 INTEGRATION_SMOKE per unit cadence — fail (8 unit headings, 0 smoke markers)

Queue premise CONFIRMED, both halves: the build is genuinely unshipped (section 2a) and the
plan genuinely fails the gate. Same legacy-shape failure profile as sibling I-30
(P1/P3/P5/P6/P7/P10). K-27 forbids executing a plan_gate-failing plan (D-8 refine-before-execute,
K-27 plan ln 16 cites the same rule for legacy-format files).

## 2. Real code evidence (verified file:line at a2744e5)

Paths worktree-relative under `.elai_cc/` unless noted.

### 2a. GrpoHarness / grpo absence (the unshipped BUILD, confirmed)

- `git grep -in "GrpoHarness" HEAD` => 0 hits, whole tree.
- `crates/training/` directory ABSENT (`ls .elai_cc/crates/` this session; 23 crates, no
  training). The E-55 contract test itself documents absence as the correct pre-unlock state
  (no_dpo_no_grpo_until_unlock.rs:110-113).
- Every `grpo` mention in HEAD (~40 hits) is a doc-comment, plan text, or the contract test —
  zero `fn`/`struct`/`mod` grpo symbols. Corroborated independently by the earlier conductor
  wiring report (conductor/state/wiring-reports/I-31.md): "NOT wired
  and NOT implemented — by design" (its SSB line pins have since drifted; mine below are fresh).
- ELAI-FEATURE-MAP.md:1863-1867: "GRPO FLYWHEEL [PLANNED(I-31), LAST + BLOCKING GUARDS] ...
  NOT-BUILT [D-026 GPU, PENDING]; never mark shipped."

### 2b. THE STRUCTURAL BLOCK: E-55 contract test bans the Tier-1 artifact BY FILENAME

This is the finding that separates I-31 from I-30, and the plan file does not mention it.

- `crates/contracts/tests/no_dpo_no_grpo_until_unlock.rs` — test fn
  `no_dpo_ppo_grpo_dapo_kto_module_until_unlock` (:93) walks `crates/orchestrator/src/` and
  `crates/training/` (scan_dirs :103-106) and FAILS on any *.rs filename containing a
  `PROHIBITED_TOKENS` prefix `["dpo_", "ppo_", "grpo_", "dapo_", "kto_"]` (:48), assert
  :143-148: "HARD-BLOCKED until rl_weight_update_unlocked SSB row emitted."
- I-31 Tier-1 ships `crates/training/grpo_harness.rs` (plan ln 68). That filename matches
  `grpo_` inside a scanned dir => the contract test fails the moment the file lands. The plan's
  W-2 claim (ln 7-9: lane stays behind the darwinian-evolve allowlist row,
  wire-dark-allowlist.toml:117-121 — row verified present) does NOT clear this: the E-55 test
  is a filename scan, not an env/allowlist gate.
- The unlock signal exists but has never fired and CANNOT fire today:
  - `RewardLadderEventKind::RlWeightUpdateUnlocked` variant ssb/payloads.rs:12025, wire string
    "rl_weight_update_unlocked" :12041; `SsbActionType::RewardLadderEvent` action_type.rs:2621,
    wire string :5502. Wire-format WIRED, production emit site: NONE (grep: zero emit sites in
    src outside doc-comments).
  - The unlock evaluator `check_unlock` (rl/rl_weight_update_deferred.rs:110) is pure and
    fully implemented but DARK: callers are its own `#[cfg(test)]` block only (:233/:247/:262).
    The module doc's "the orchestrator wires these from live state" is not realized in code.
  - Unlock conditions (E-55-decisions.md DECISION-7, ln 65-76, constants verified in code
    :36/:40/:46): C1 BAI saturated (3 stable sessions, an arm ≥20 pulls with mean ≥0.6),
    C2 ORM coverage ≥0.80, C3 VRAM headroom ≥4.0GB. C3 is PHYSICALLY UNMET on this box: the
    module doc (:11-14) records the 12GB ceiling leaves <2GB headroom and requires a 2x3090
    upgrade or 24GB rent — this is exactly the D-026 GPU gate the FEATURE-MAP stamps on I-31.
- Editing/relaxing the contract test to admit grpo_harness.rs is a frozen-gate edit
  (safety-frozen-gate; preamble R-3: never edit the gates that judge your own output). Renaming
  the harness to dodge the filename scan (e.g. train_harness.rs) evades the gate's intent and
  must be recorded as a REDTEAM-forbidden move in the refined plan, not used.

### 2c. IC-7 dependency: I-30 NOT landed (hard sequencing)

- IC-7 = "SFT before GRPO" (I-0_INDEX.md:586; I-31 plan ln 2 "binds ID-6 | IC-7 | depends
  I-30 + H-43/H-44/H-28 + I-24"; ln 8 "GRPO is LAST by doctrine").
- I-30 state at HEAD: `SftExhaust|sft_exhaust` symbol grep over crates => 0 hits;
  `git log --all --grep="I-30"` => 0 commits. I-30 sits at BUILD-WITH-SEAM *after its own
  K-27 refine* per workflow/reasoning/seam-proposals/I-30-seam-proposal.md §4 (this walk,
  earlier unit). So I-31 is ≥2 stages behind: I-30 refine → I-30 Tier-1 land → I-31 eligible.
- I-31 consumes I-30's SFT init checkpoint AND its JSONL negative pairs (plan ln 100, ln 22-24);
  neither artifact class exists yet.
- Backlog card workflow/reasoning/backlog-drain/cards-IJKL.md:128 agrees: "state: dark ...
  blocked on H-43/44/28 LIVE + RunPod + darwinian allowlist"; :359 "I-30 (dark, no training
  crate) blocks I-31/J-50".

### 2d. Blocking prerequisites 1-7 (plan ln 28-38): ALL PRESENT as modules; liveness is executor-time evidence

| # | Prerequisite | Module evidence (verified this session) |
|---|---|---|
| 1 | H-43 execution-grounded fitness | execution_fitness.rs (ExecutionFitness :156, FrozenCorpus :118); production callers incl. darwin_admit_loop.rs, clade_metaproductivity.rs, evaluator_abi.rs. CAVEAT: IS-2 `derived_alpha` still EXISTS at darwin_admit_loop.rs:285 as the length-proxy fallback; :854 documents it is REPLACED by parsed eval-log outcome when parseable. "IS-2 retired confirmed" is therefore conditional — the refined plan must pin "harness consumes the execution-outcome path only" and keep the plan's own negative grep (ln 110: rg derived_alpha|blake3(config over grpo_harness.rs == 0). |
| 2 | H-44 behavioral traps | trap_behavioral.rs present (apply_isomorphic_rename :135, frozen_corpus_hash :364); old IS-3 keyword grep still at trap_tasks.rs:122 (retired-as-anchor, per plan). |
| 3 | I-24 PRM canaries | canary_injector.rs (CanaryPair :108, CanaryInjectorConfig::load :162, is_biased :232); consumed by verifier_stack.rs + ssb payload/action_type. "PrmCanaryEvent rows present in a LIVE scoring run" remains runtime evidence the executor must capture. |
| 4 | H-28 reward-tampering gate + R8 freeze | reward_tampering.rs (classify_reward_tampering :180, RewardTamperingOutcome :116, is_hard_block :147); CLI surface reward_tamper_cmd.rs wired at main.rs:303-311; .claude/tools/gate_freeze_check.py present. |
| 5 | H-17 trial-N + H-18 deflation | trial_ledger.rs (open :190, trial_count :332); deflation_gate.rs (compute_dsr_tier1 :214). |
| 6 | D-6 held-out corpus | holdout_budget.rs (classify_holdout_query :319, HoldoutBudgetLedger::open :433). |
| 7 | D-7 cross-domain canary | canary.rs present (cross_domain refs); plus gate_self_validation.rs (GateId::name :110, is_pass :199). |

Presence is NOT the plan's bar — the plan demands each be LIVE in a running loop, verified by
the executor before EXECUTING. That check stays in the refined plan as executor deliverable 0.

### 2e. Reward-endpoint dependency reality (plan self-contradiction to fix at refine)

Plan ln 42-44 claims the endpoint is "in-process (no external dep ...; a plain Axum/Actix
handler)". Reality: axum/actix appear in NO production manifest — grep count 0 in
crates/orchestrator/Cargo.toml and crates/cli/Cargo.toml. The only axum mention in any
Cargo.toml is a comment in crates/contracts/Cargo.toml:98-99 explaining the repo precedent:
"Uses tokio::net directly; no hyper/axum server dep needed." An Axum handler IS a new dep and
triggers the plan's own anti-slopsquat + user-confirm protocol (ln 40-44). Tier-1 (fixture
harness, in-process call) needs NO network transport at all. Record as FORK (section 3).

### 2f. Seam inventory for the eventual build (real hooks, for the post-unlock executor)

- CLI production entrypoint: `pub(crate) enum CliAction` crates/cli/src/args.rs:75 + dispatch
  `CliAction::Rra =>` crates/cli/src/main.rs:800-801 — the same rra_cmd family pattern the
  I-30 proposal names; `elai train grpo --init <sft-checkpoint>` (plan reachable_from ln 105)
  lands as a new variant + parse arm + `train_cmd.rs` sibling. NOTE: the filename ban (2b)
  covers crates/orchestrator/src + crates/training only — a cli module is scan-exempt, but the
  harness itself belongs in crates/training and therefore waits for unlock.
- Unlock-evaluator wiring (the TRUE critical path, and it is E-55.1 work, not I-31 work):
  wire `check_unlock` into a task_runner post-task hook exactly where I-28 wired credit
  dispatch — `dispatch_dag_credit(...)` call at task_runner.rs:12497, fn at :14007 (the
  established post-task dispatch pattern). Live signal sources all exist:
  C1 `ArmPosterior` bai.rs:84 (n_pulls :88, mean :105) from D-5 BAI;
  C2 rl/reward_fusion.rs (FusionWeights :57, score :121) coverage fraction;
  C3 VRAM probe precedent python/elai_inference/server.py:1350-1355 (`props.total_memory`).
  On all_three_true: emit RewardLadderEvent(RlWeightUpdateUnlocked) via payloads.rs:12025 +
  UnlockTicket::from_status (rl_weight_update_deferred.rs:144). Env gate already defined:
  ENV_AGENTIC_RL_ENABLE rl/mod.rs:79, agentic_rl_enabled() :84 (currently dark outside rl/).
- SSB event seam for GrpoRewardComputed (plan ln 134): new variant via ssb/payloads.rs +
  ssb/action_type.rs + golden relock crates/orchestrator/tests/action_type_lock.rs — the
  executor TaskSpec must LIST the golden in its allowed write set (same flag the I-30
  proposal raised). PLAN DRIFT: plan ln 134 pins "ssb.rs:2981 / action_type_lock.rs:601" —
  stale; the ssb module split into ssb/{action_type,payloads,schema,...}.rs and the lock test
  lives under tests/. Also darwin_admit_loop.rs:202 → :285 (derived_alpha), trap_tasks.rs:123
  → :122. Re-pin at refine (W-9).
- W-2 lane gate: wire-dark-allowlist.toml:117-121 darwinian-evolve row — present as plan claims.
- NEW pre-gate the plan omits: FEATURE-MAP:1866 adds "Pre-gated by L-24 FD-1..FD-3 (diversity
  guard on the training exhaust)". L-24 plan ln 62-64 (leg D): FD-1 train on MOVES not
  survivors; FD-2 lambda-filtered labels; FD-3 evaluator-id+version-hash edge on every dataset
  row — contract-first, code lands inside I-30/I-31 at D-026 unlock. The refined I-31 must add
  L-24 FD-1..FD-3 to consumes_symbols/stub_ban.

## 3. What the K-27 refine must add (mechanical) — and why refine alone does NOT unblock

Gate-greening is mostly re-blocking existing substance, same as I-30:
- P1: promote `observable_effect:` (ln 120-124) to a line-start EXPECTED_OBSERVATION block.
- P5: EXCLUDED (ln 44-48) + stub_ban (ln 109-116) re-stated as ABORT conditions, plus two new
  ones from this proposal: ABORT if no_dpo_no_grpo_until_unlock.rs fails on the changeset;
  ABORT if no recorded-session rl_weight_update_unlocked row exists at execute time.
- P6: VERIFY block with runnable lines: `cargo test -p contracts no_dpo_no_grpo_until_unlock`
  and `cargo test -p contracts i31_grpo_flywheel_contract` (plain `cargo` — Depot shim, build
  law) + `python3 .claude/tools/gate_freeze_check.py` over the reward-endpoint path + off-agent
  pass artifact convention.
- P7: REDTEAM block, attacks: (i) rename-to-evade the E-55 filename scan (FORBIDDEN — evading
  a frozen gate); (ii) seeded reward-hack candidate must yield PromotionBlocked (already the
  plan's NEGATIVE row ln 124); (iii) harness edits reward-endpoint/gate code (R8 — caught by
  gate_freeze_check.py); (iv) canary-divergence gaming (I-24 ScorerNeutralized skip, ln 26-28).
- P10: per-unit INTEGRATION_SMOKE or n/a for the 8 headings.
- P3 FORKS to record: (i) E-55 DECISION-9 ladder says the post-unlock rung is
  "SFT-on-rollouts → KTO+RLVR → maybe DAPO → never DPO" (E-55-decisions.md:91-99) while I-31
  picks GRPO (plan ln 44-46 rejects PPO only) — the newer I-plan supersedes on variant choice
  but the reconciliation with DECISION-9 must be RECORDED, it currently is not; (ii) reward
  endpoint transport: in-process fn call (Tier-1, zero dep) vs tokio::net raw (repo precedent)
  vs axum new-dep (anti-slopsquat + owner approval) — lean: in-process for Tier-1, decide
  transport only at Tier-2; (iii) unlock-evaluator wiring placement: task_runner post-task
  hook (dispatch_dag_credit pattern :12497) vs session-runtime — lean: task_runner, matches
  I-28 precedent; (iv) does unlock-wiring ship as E-55.1 (its own unit) or as I-31 slice 0 —
  lean: E-55.1, it is not GRPO work and is useful regardless.
- Drift re-pins per 2f; add L-24 FD-1..FD-3 pre-gate per 2f.

BUT: unlike I-30 (where refine alone made the unit executable), a green I-31 plan is still
unbuildable, because the blocks are structural, not doc-shape:
1. E-55 contract test blocks the Tier-1 filename until an rl_weight_update_unlocked row exists
   in a recorded session (2b) — and the evaluator that could emit it is dark with C3 physically
   false (D-026 GPU: needs 2x3090 or 24GB rent — an OWNER hardware/spend decision).
2. IC-7: I-30 Tier-1 not landed (2c) — hard predecessor, itself pre-refine.
3. Owner scheduling: ML-infra tail parked (cards-IJKL.md:95 "RunPod/flywheel tail
   (I-16/I-30/I-31) blocked"); owner UI hold additionally parks the plan's frontend_pair
   flywheel panel (plan ln 86-87, 117-118) — no dashboard-primary work; the panel sequences
   behind the hold in the slice plan, W-6 discharged at Tier-2, not deleted.

## 4. Verdict

**DEFER** — gates, in blocking order:
1. E-55 unlock contract: crates/contracts/tests/no_dpo_no_grpo_until_unlock.rs (:48/:103-106/
   :143-148) until RewardLadderEvent(RlWeightUpdateUnlocked) is emitted in a recorded session;
   today check_unlock (rl_weight_update_deferred.rs:110) has zero production callers and C3
   (VRAM ≥4GB headroom, DECISION-7) is false on this hardware => D-026 GPU owner gate.
2. IC-7: I-30 SFT Tier-1 must land first; I-30 is itself at refine-pending (its own proposal §4).
3. K-27: plan_gate_check exit 1 => refine-first (section 3) before ANY execute.

Sanctioned path TO the unlock (recommend queueing, none of it is I-31 code):
(a) run the I-31 K-27 refine now (doc-only, cheap, section 3 — so the plan is execute-ready
    the day the gates clear); (b) queue E-55.1 unlock-wiring as its own unit: check_unlock
    into the task_runner post-task hook (:12497 pattern) fed by bai.rs:84 posteriors +
    reward_fusion coverage + the server.py:1350 VRAM probe, emitting the unlock row when true
    — this is E-55-sanctioned, touches no grpo_ filename, and turns the unlock from a dead
    letter into a live gate; (c) surface the D-026 GPU decision (2x3090 / 24GB rent / RunPod
    W-10 checkbox) to the owner — no unlock is possible without it. Do NOT: edit or rename
    around the contract test; start crates/training/grpo_harness.rs pre-unlock; do
    dashboard-primary panel work under the UI hold.

NOT SUPERSEDED: FEATURE-MAP keeps I-31 PLANNED(:1863), backlog card keeps it dark-blocked
(:128), L-24 names it as future consumer — no successor unit exists.

## 5. No-invention attestation

Every citation above was grepped/read this session at a2744e5: the contract test and its
token list/scan dirs/assert, the dark check_unlock and its test-only callers, the SSB
variant + wire string lines, the seven prerequisite modules, the CLI dispatch family, the
task_runner dispatch hook, the BAI/reward_fusion/VRAM signal sources, the allowlist row, the
FEATURE-MAP stamp, E-55 DECISION-7/8/9, L-24 leg D, and both plan_gate_check runs (I-31 this
session; I-30 per its proposal). Nothing is asserted from plan memory.

## 6. L2/L3 digest for a scoped-mount consumer (evidence-dir substitute)

Binding facts a worktree-scoped executor cannot re-read from the repo root:
- Unit: I-31, Track 17 OPEN-WEIGHT FLYWHEEL, WAVE 5, LAST by doctrine; binds ID-6 + IC-7;
  depends I-30 + H-43/H-44/H-28 + I-24 (I-SUMMARY_INDEX.md:573-585). DPO HARD-AVOID forever
  (E-55 DECISION-8); GRPO-vs-KTO variant fork must be reconciled with DECISION-9 at refine.
- Contract: produces crates/training/grpo_harness.rs::GrpoHarness + ::RewardEndpoint
  (supervisor-side; policy never reaches reward code); consumes H-43 execution fitness, H-44
  traps, H-28 gate (R8-frozen), I-24 canaries, IS-16 ledger chain (trial_ledger/deflation_gate/
  holdout_budget/gate_self_validation), I-30 checkpoint + negative pairs, D-5/D-29 bandit,
  L-24 FD-1..FD-3 row contract. Reachable from `elai train grpo --init <sft-checkpoint>`
  (Tier-2 RunPod-gated); Tier-1 is the fixture harness + contract test.
- Acceptance observation: GrpoRewardComputed rows RewardServed (cites execution-gate verdicts) /
  PromotionBlocked (seeded reward-hack, H-28 fires) / PromotionLedgered (cites trial-N +
  deflation + held-out + D-7 canary row ids). NEGATIVE: rg 'derived_alpha|blake3(config'
  over grpo_harness.rs == 0; tampered-harness fixture => PromotionBlocked; incomplete ledger
  chain => no PromotionLedgered row.
- Hard blocks to re-verify at execute time: no_dpo_no_grpo_until_unlock.rs green WITH the
  unlock row present in a recorded session; H-43/H-44/H-28/I-24 LIVE (not just present);
  I-30 Tier-1 landed; darwinian-evolve allowlist row covers the lane; gate_freeze_check.py
  green over reward-endpoint path; plan_gate_check exit 0 on the refined plan.

VERDICT: DEFER — gate: E-55 rl_weight_update_unlocked contract (no_dpo_no_grpo_until_unlock.rs)
+ D-026 GPU owner decision + IC-7 (I-30 first) + K-27 refine; queue the doc-only refine and the
E-55.1 unlock-wiring unit now, build no grpo_ file pre-unlock.

END. This worker does not certify DONE; the off-agent verifier and opus controller own that.
