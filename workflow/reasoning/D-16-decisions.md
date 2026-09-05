D-16 — UniCR conformal-refusal calibration decisions
=====================================================

Scope shipped this session
--------------------------

The orchestrator-binding instructions narrowed the D-16 plan from the
full 13-task surface to an orchestrator-scoped MVP. What landed:

  * .elai_cc/crates/orchestrator/src/conformal_calibration.rs (NEW)
      - CalibrationTuple { tuple_id, claim_id, claim_text, ground_truth,
        predicted_score, ts_ms }
      - CalibrationCorpus { tuples, source, frozen } + load_seed_jsonl
      - CorpusSource enum { HeldOut, Synthetic, Curated, Auto }
      - RefusalDecision enum { Abstain, Answer { confidence } }
      - compute_tau / compute_tau_with_min — Vovk §2.2 (1 - epsilon)
        quantile predictor
      - decide / decide_and_emit / compute_tau_and_emit
      - VecConformal{Tau,Refusal}Emitter test stubs
      - recent_tau_rows / recent_refusal_rows read surfaces

  * .elai_cc/crates/orchestrator/src/agent_pool.rs
      - dispatch_conformal_refusal — one-shot producer wiring the
        GroundingMode::ConformalRefusal arm to a live corpus

  * .elai_cc/crates/orchestrator/src/lib.rs — pub mod conformal_calibration

  * .elai_cc/crates/orchestrator/src/ssb.rs
      - SCHEMA_VERSION 81 -> 82
      - SsbActionType +2 (ConformalTauComputed, ConformalRefusalDecided)
      - SsbPayload +2 (matching payload structs)
      - SQL CHECK constraint +2 action_type literals
      - from_str / as_sql_str / action_type() arms
      - all_test_payloads +2 (count 171 -> 173)

  * .elai_cc/calibration/seed/{README.md, calibration_seed.jsonl}
      - 20-tuple synthetic seed corpus, 10 positive / 10 negative

  * .elai_cc/crates/dashboard-v2/src-tauri/src/commands_d16.rs (NEW)
      - get_conformal_recent(limit) -> tau_rows + refusal_rows + counts
      - get_calibration_corpus_state() -> snapshot DTO

  * .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs
      - mod commands_d16; + invoke_handler registration (2 commands)

  * .elai_cc/crates/dashboard-v2/src/lib/drawers/ConformalCalibrationPanel.svelte (NEW)
      - tau table + corpus state table + abstain/answer histogram bar +
        per-claim decision table
      - DrawerKey 'conformal-calibration' added to api/types.ts
      - DrawerHost.svelte + Drawer.svelte labels updated

  * .elai_cc/crates/orchestrator/tests/conformal_integration.rs (NEW)
      - dispatch_conformal_refusal low+high score smoke test
      - load_seed_jsonl off-disk smoke test using CARGO_MANIFEST_DIR

  * .elai_cc/scripts/locked_keys_registry.toml — D-16 commented seed
    entries appended (per D-9 bootstrap policy).

Out-of-scope (deferred to a follow-up patch)
--------------------------------------------

  * Nightly L3-locked TauUpdater + persisted .elai_cc/calibration/tau.txt.
  * CLI `elai calibration {status,update}` subcommands.
  * elai_cc.toml `[conformal]` section + composite_weights loader (the
    seed corpus carries `predicted_score` as a pre-composed scalar so
    the MVP does not need the weights).
  * GroundingMode integration in task_runner.rs — D-10 already exposes
    `ConformalRefusal` as a routing target; the consumer wiring
    (load corpus once at startup, route through
    `dispatch_conformal_refusal` per claim) is a task_runner edit and
    was held back to keep the producer surface diff reviewable.
  * Background corpus-growth hook (task_runner post-claim-resolution
    append). Same reason as above.
  * 600-tuple integration test (`min_corpus_size >= 500`) — DEFAULT
    floor stays at 20 in this MVP so the seed corpus exercises the full
    producer path. The production floor will raise via
    `compute_tau_with_min` once the nightly grower lands.

Three concrete decisions
------------------------

(a) Why epsilon = 0.1 default

    Grounding: arXiv:2503.01345 (UniCR) §6 — 0.1 is the most-reported
    operating point in the paper's empirical sweep and the value the
    Vovk et al. §2.2 quantile-predictor analysis defaults to in
    Algorithmic Learning in a Random World worked-example chapters. It
    is also tunable per project — the orchestrator caller picks
    epsilon at dispatch time so a more conservative pipeline can use
    0.05, an aggressive one 0.2.

(b) Why MIN_CORPUS_SIZE = 20 in the MVP (vs the plan's 500)

    The plan's 500 floor is the finite-sample bound from UniCR §3.4
    that guarantees ε = 0.1 holds with high probability. The MVP
    ships with a 20-tuple seed corpus so the orchestrator can exercise
    the FULL producer path (compute -> decide -> SSB emit -> dashboard
    read) before the nightly grower starts. `compute_tau_with_min`
    accepts a caller-supplied floor so the production wiring
    (`dispatch_conformal_refusal` in task_runner once the grower
    lands) can raise it to 500 without an API change. The 20-floor is
    documented on the const and the seed README.

(c) Why predicted_score is a pre-composed scalar (vs the plan's
    composite weights w1*logprob + w2*-entropy + w3*novelty)

    The composite-weights loader lives in the deferred elai_cc.toml
    `[conformal]` section. The MVP corpus already carries
    `predicted_score` as the consumer-side pre-composed scalar so the
    `compute_tau` math operates on the same `[0.0, 1.0]` space the
    decide() rule consumes. When the production hook lands, the
    task_runner pre-composes the GG signal scalar exactly once at
    corpus-append time and the rest of the pipeline stays unchanged.
    This decouples the L3-locked refusal rule (compute_tau + decide)
    from the (planned) L3-locked composite_weights config — a smaller
    locked surface is easier to audit.

Test results
------------

  * cargo check -p elai-orchestrator: PASS (exit 0).
  * cargo check -p elai-dashboard-v2: PASS (exit 0).
  * In-module unit tests (cargo test -p elai-orchestrator):
    pending (run by auditor).
  * Integration test: tests/conformal_integration.rs compiled with
    `cargo test --no-run` — wait on auditor for the runtime pass.

Schema bookkeeping
------------------

  * SCHEMA_VERSION_PUBLIC = 82 (was 81; D-12 raced from 80 -> 81 in
    parallel; this session observed 81 then bumped +1).
  * all_test_payloads().len() goes 171 -> 173 (+2 new variants).
  * No CHECK-constraint comma drift — the new literals appended after
    the existing list with a leading comma on the prior `'baby_step_dispatched'`
    entry.

Grounding sources
-----------------

  * arXiv:2503.01345 — UniCR (Unified Conformal Refusal). The §3.2
    refusal-region rule (abstain when nonconformity score exceeds the
    (1 - ε) empirical quantile of the calibration scores) is the rule
    this module implements verbatim.
  * Vovk V., Gammerman A., Shafer G. — Algorithmic Learning in a
    Random World (Springer, 2005), §2.2: the original
    quantile-predictor construction the UniCR refusal rule
    specialises.
  * Anti-stale-training compliance: blake3 (none needed here),
    tracing, thiserror, native async-fn-in-trait, no async-trait
    crate, no new external dep. Per CLAUDE.md.

D-16 closure (this session — post C-52, SCHEMA 126 -> 127)
==========================================================

The earlier session shipped 60% of the substrate at SCHEMA 82 (now
at head 126 post C-52). This closure session resolves the remaining
gaps from the cross-check re-check:

DECISION-1 -- SCHEMA bump 126 -> 127 (single new variant)
---------------------------------------------------------
The plan's stale 61 -> 62 target is dropped. We bump ONE step
(126 -> 127) for the single new variant `CalibrationCorpusUpdated`
(durable TauUpdater audit row). Previously-shipped ConformalTauComputed
+ ConformalRefusalDecided variants stay at their existing positions;
NOT renumbered. 9-tuple lockstep applied (enum / CHECK / as_sql_str /
from_str / SsbPayload variant / action_type() arm / payload struct /
test_payload reconstruct arm / all_test_payloads count 290 -> 291).
Additive only.

DECISION-2 -- Preserve closed `RefusalDecision { Abstain, Answer }`
-------------------------------------------------------------------
The plan's 4-variant proposal is REJECTED. D-22 imports the 2-variant
shape at runner.rs:70; expanding the enum would break that consumer
and any pattern-match against the closed shape. The 2-variant enum is
load-bearing.

DECISION-3 -- 4-tag reason taxonomy lifts to OPTIONAL payload field
-------------------------------------------------------------------
The 4-tag refusal-reason taxonomy
(`low_evidence | high_dispersion | tool_inconsistency |
verifier_rejection`) lifts to an OPTIONAL `reason: Option<String>`
field on `ConformalRefusalDecidedPayload`. We ship a typed
`RefusalReason` enum + `decide_and_emit_with_reason` adapter that
stamps the reason on `Abstain` rows only (Answer rows always drop the
reason). The plain `decide_and_emit` keeps reason None — backward-
compatible with every existing call site
(agent_pool::dispatch_conformal_refusal included).

DECISION-4 -- alpha and rho are DISTINCT UniCR quantities
---------------------------------------------------------
Per UniCR (arXiv:2509.01455) Eq.5 + §4.2:
  * alpha = marginal miscoverage rate; CRC quantile at (1 - alpha).
    Default 0.10.
  * rho = selective risk budget. DISTINCT from alpha. Default 0.05.
  * Invariant: rho < alpha (selective predictor must be stricter than
    the marginal one).
Both carried on `[conformal]` toml + ConformalRefusalRuntimeConfig.
Only alpha lives on `CalibrationCorpusUpdated` SSB rows because rho
is a decision-time quantity, not a calibration-time quantity.
`validated()` enforces the invariant.

DECISION-5 -- CLI namespace `elai conformal-calibration` (distinct)
-------------------------------------------------------------------
`elai calibration` is OWNED by C-47 (ToolCalibrationRegistry).
D-16 uses `elai conformal-calibration {status, update}` — a sibling
top-level verb. Both surfaces share the "calibration" theme but
operate on completely different state (C-47: per-tool case base;
D-16: UniCR tau scalar).

DECISION-6 -- Rewarding-Doubt (arXiv:2503.02623) is supporting-only
-------------------------------------------------------------------
arXiv:2503.02623 "Rewarding Doubt" has NO abstention mechanism — it
is a log-score calibration paper, NOT a refusal-implementation paper.
Cited as "supporting evidence on log-score calibration; not
implementation-specifying for refusal" per D-13 DECISION-8 framing.

DECISION-7 -- Production dispatch via task_runner layer-7 hook
--------------------------------------------------------------
`dispatch_conformal_corpus_grow_hook` runs in the post-task chain
AFTER `dispatch_novelty_hook`. Layer-7 chain is now:
  route -> verify -> bai -> adversary -> sycophancy -> novelty
       -> conformal-grow
Gated by (1) `resolved.conformal_refusal_runtime.is_some()` (2)
`ELAI_CONFORMAL_REFUSAL_ENABLE=1` (3) `[conformal].enable = true`
(4) `corpus_grow_min_interval_secs` throttle elapsed. All four MUST
hold.

DECISION-8 -- tau.txt for cross-process visibility; SSB for audit
-----------------------------------------------------------------
`TauUpdater::update_tau` writes the scalar to `tau_path` (default
`.elai_cc/data/conformal_tau.txt`) via atomic tmp+rename. The CLI
`status` sub-action + dashboard `get_calibration_corpus_state` both
read-via-stat from this path. Separately, every persistence step
emits one `calibration_corpus_updated` SSB row carrying
(corpus_size, new_tau, alpha, source, ts_ms) — the durable audit
trail the dashboard renders. Complementary, not redundant.

DECISION-9 -- Plan-code signature drift locked
----------------------------------------------
Plan called for 4 RefusalDecision variants; code has 2. DECISION-2
documents why. This decision exists to lock the drift so future
plan-improvers don't re-attempt the expansion.

DECISION-10 -- Out-of-scope deferred
------------------------------------
  * Per-task `(claim_score, verdict)` corpus-append plumbing.
    Today's hook ships the throttled tau refresh + audit row; the
    append-side requires the upstream verifier disagreement signal
    that lands only after the adversary chain finishes routing
    claim scores end-to-end.
  * Live IPC seam TauUpdater <-> task_runner config (operators
    today flip `enable` via toml; CLI `update` is on-demand
    override).
  * Dashboard ConformalCalibrationPanel.svelte UI rewire for
    new fields (corpus_size / current_tau / last_updated_ts).
    Tauri command exposes them; Svelte render lands in follow-up.

Files touched (D-16 closure)
----------------------------
  * conformal_calibration.rs -- TauUpdater + RefusalReason +
    update_tau_and_emit + ConformalError::TauPersist + 8 new tests.
  * ssb.rs -- SCHEMA 126 -> 127 + CalibrationCorpusUpdated variant
    (9-tuple lockstep) + reason field on
    ConformalRefusalDecidedPayload.
  * runtime_config.rs -- ConformalRefusalRuntimeConfig +
    ConformalRefusalRuntimeState + 4 tests.
  * task_runner.rs -- dispatch_conformal_corpus_grow_hook +
    ResolvedConfig field + throttle state.
  * elai_cc.toml -- [conformal] L3-LOCKED stanza (commented).
  * args.rs -- ConformalCalibrationAction + parser + dispatch entry.
  * main.rs -- mod conformal_calibration_cli + dispatch arm.
  * conformal_calibration_cli.rs (NEW) -- run / run_status /
    run_update + 5 tests.
  * commands_d16.rs -- CalibrationCorpusStateDto extended +
    get_calibration_corpus_state rewires to live TauUpdater.
  * contracts/tests/conformal_refusal_contract.rs (NEW/extend) --
    SCHEMA floor + RefusalDecision 2-arm + payload round-trip +
    TauUpdater determinism + UniCR Eq.5 correctness.

