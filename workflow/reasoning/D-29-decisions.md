D-29 hierarchical_bai_research — DECISIONS LOG

Plan: F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-29_hierarchical_bai_research.md
Session: s-25120 (completion-drive cluster, plan 25 of 26)
SCHEMA bump: 145 -> 146
TOTAL_VARIANTS bump: 418 -> 419
New SsbActionType: HierarchicalBaiRoundCompleted

Plan body is marked DEFERRED; the s-25120 HANDOFF §6 explicitly overrides
to FULL BUILD with mitigations for the 6 explorer gaps. This doc records
every load-bearing decision so the audit + memory-writer agents can pin
the cluster cleanly.

DECISION-1 — Single SSB variant + atomic SCHEMA bump (pattern-1 + pattern-2)
Plan body hinted at two-row emission (per-family hyperposterior + per-arm
draw). REJECTED per D-19/D-23/D-24/D-25/D-28 single-variant precedent.
Shipped one new variant HierarchicalBaiRoundCompleted carrying both the
mu / kappa hyperposterior AND the full per-arm shrunk Beta summary list
in a single payload (the dashboard tooling that already parses
BaiArmUpdated keeps working; HierarchicalBaiRoundCompleted is the
additive hierarchical context per integration-explorer SEE-ALSO
recommendation).

Files touched (10-tuple cascade + extras):
  1. orchestrator/src/ssb.rs SCHEMA_VERSION 145 -> 146.
  2. orchestrator/src/ssb.rs SQL CHECK allowlist + new 'hierarchical_bai_round_completed' string entry.
  3. orchestrator/src/ssb.rs SsbActionType enum + new HierarchicalBaiRoundCompleted variant.
  4. orchestrator/src/ssb.rs SsbActionType::as_sql_str arm.
  5. orchestrator/src/ssb.rs ssb_action_type_from_str_public arm.
  6. orchestrator/src/ssb.rs SsbPayload enum + new HierarchicalBaiRoundCompleted(payload) variant.
  7. orchestrator/src/ssb.rs SsbPayload::action_type() arm.
  8. orchestrator/src/ssb.rs default_payload_for_type arm (msgpack roundtrip support fixture).
  9. orchestrator/src/ssb.rs new ArmPosteriorSummary struct + HierarchicalBaiRoundCompletedPayload struct.
 10. orchestrator/src/ssb.rs SsbPayload msgpack tail fixture variant entry.
 11. orchestrator/tests/action_type_lock.rs TOTAL_VARIANTS 418 -> 419 + ALL_VARIANTS append + exhaustive_match_guard arm.
 12. orchestrator/tests/action_type_lock.golden header + alphabetical insertion between held_out_task_used and hook_event; all subsequent ordinals shift +1.

DECISION-2 — Paper labels (pattern-5)
Plan body line 22-23 cites variance-BAI 2106.10417 and asserts a
"30-50% regret reduction" headline. Paper-ingestor verification confirms:
  * arXiv:2111.06929 "Hierarchical Bayesian Bandits" (Hong, Kveton,
    Zaheer, Ghavamzadeh — AISTATS 2022). HIGH-reliability SUPPORTING
    EVIDENCE — provides the exact Bernoulli/Beta hyperprior shape
    `theta_i ~ Beta(mu*kappa, (1-mu)*kappa)` used by D-29 (paper §2
    Bernoulli/Beta variant). HOWEVER the paper analyses regret
    minimization, NOT best-arm identification. Cited as SUPPORTING
    EVIDENCE only; the Bayes regret guarantees do NOT carry over to
    BAI (paper interface file L144-148 explicitly notes this).
  * arXiv:2106.10417 "Variance-Dependent Best Arm Identification" (Lu,
    Tao, Zhang — UAI 2021). HIGH-reliability DIRECT GROUNDING for the
    BAI side (variance-dependent sample complexity H_var; paper §5.1).
    Already ingested at D-5; reused here.
  * The plan's "30-50% regret reduction" claim is UNGROUNDED for the
    BAI regime (neither cited paper quotes that number for BAI). Tagged
    as supporting-only / UNGROUNDED in the plan and recalibrated here
    to "live baseline RUNPOD-PENDING" in the SSB payload docstring.

DECISION-3 — Free-fn + struct over trait (pattern-3)
Hierarchical layer is `HierarchicalBaiState` (struct) +
`hierarchical_arm_select` / `hierarchical_update` (free fns) +
`SelectionBasis` (enum) + `FamilyHyperprior` (struct). NO Trait
abstractions. Single concrete impl mirrors D-5's Thompson / promote_candidate
shape.

DECISION-4 — ELAI_BAI_MODE env-gate (pattern-4)
String-valued env var (NOT boolean) so future modes ("hierarchical_3level"
etc.) slot in without a fresh env name. Default value (unset OR unknown)
returns BaiMode::Flat — the D-5 baseline runs untouched in production.
Both gates required for hierarchical path: ELAI_BAI_ENABLE=1 (outer
opt-in) AND ELAI_BAI_MODE=hierarchical (mode selector). Wire-dark by
default for the entire stack.

DECISION-5 — Zero new Rust deps (pattern-7)
The Metropolis-within-Gibbs sampler is hand-rolled. Reuses
`crate::bai::sample_beta` (Marsaglia-Tsang gamma-ratio Beta sampler;
D-5 already in tree) for the inner Beta draws. The mu / kappa proposal
distributions use Gaussian random-walk on mu (sigma=0.05) and
log-Gaussian random-walk on kappa (sigma=0.5 in log-space) — both via
the same Box-Muller normal pipeline as bai.rs:182 `next_standard_normal`.
Log-Gamma via Lanczos approximation (Numerical Recipes 2007 §6.1) is
new local code (no `statrs` / `special` dep). All math goes through
`elai_core::SeededRng` so determinism contracts hold.

DECISION-6 — Single FamilyId(0) default in task_runner dispatch
The hierarchical dispatch in `task_runner::dispatch_bai_round` assigns
ALL arms to FamilyId(0) until a routing map ships. Rationale: the
runtime does not yet hold a TemplateId -> family routing table (the
"5 families per spec §3.7 Q3" is design intent, not yet shipped). The
single-family configuration is mathematically equivalent to a flat Beta
posterior with a Bayes-update toward the hyperprior — it does not break
correctness, it just removes the cross-family signal until a routing
map is wired. Logged as RUNPOD-PENDING.

DECISION-7 — RUNPOD-PENDING items (documented, NOT blocking)
  a. Live flat-vs-hierarchical regret baseline on the synthetic 5x10
     fixture over 1000 rounds. The math is CPU-bound (~50 arms x 100
     Gibbs iter = O(K * 100) per round; <1ms on a dev box) BUT the
     baseline measurement needs a runtime session producing 1000
     verdicts. Host-side scaffolding shipped; runtime measurement
     deferred.
  b. Path A vs Path B determination. Plan §3 says "regret <= flat-BAI
     regret on synthetic" decides Path A. With (a) RUNPOD-PENDING, the
     decision is also deferred. Pre-committed threshold: hierarchical
     regret >= 5% lower than flat-BAI regret with bootstrap CI not
     crossing zero. Logged here so hindsight-reasoning is impossible.
  c. Family routing map (DECISION-6 above). Currently all arms land in
     FamilyId(0); the integration test exercises 5 distinct families
     directly via the fixture but the production dispatch does not.

DECISION-8 — Gibbs configuration constants (research-tested defaults)
  * MAX_GIBBS_ITERATIONS = 100 (Gelman BDA3 §11.5 standard recommendation;
    R-hat < 1.1 reaches well before 100 on conjugate-adjacent posteriors).
  * GIBBS_BURN_IN = 20 (20% of MAX; leaves 80 effective samples).
  * KAPPA_PRIOR_SHAPE / RATE = 2.0 / 0.5 (Gelman BDA3 §5.3 weakly-informative
    Gamma; mean=4.0, var=8.0).
  * MIN_KAPPA / MAX_KAPPA = 1e-3 / 1.0e6 (numerical clamps; Beta density
    underflow at kappa < 1e-3 and irrelevant concentration beyond 1e6).
  * MU_FLOOR = 1e-6 (avoids Beta(0, K) singularity in the proposal).
These constants are pinned by contract tests
`kappa_prior_constants_pinned_at_weakly_informative_defaults`.

DECISION-9 — File layout
Plan §"Files touched" line 92 names `bai_hierarchical.rs` (sibling) AND
§1 spec line 48 names `python/bai_hierarchical/` (Python pkg). Decision:
  * Rust sibling at `orchestrator/src/bai_hierarchical.rs` (not
    `orchestrator/src/bai/hierarchical.rs`). The latter would force a
    refactor of `bai.rs` into `bai/mod.rs`; integration-explorer flagged
    this as gratuitous (CLAUDE.md "no unsolicited refactoring"). The
    prompt manifest's "OR src/lib.rs" alternative is honoured —
    `pub mod bai_hierarchical;` lives in lib.rs next to `pub mod bai;`.
  * Python sidecar module NOT shipped. The hand-rolled Rust Gibbs runs
    in O(K * 100) per round (<1ms on dev box at K=50); the Python path
    was a research-grade artifact from when the plan presumed RunPod for
    Gibbs. No Python deps added (pattern-7).

DECISION-10 — Single-row vs 3-row SSB emission (integration-explorer §"Open question 6")
Plan body ambiguous between "ONE row (HierarchicalBaiRoundCompleted)"
and "THREE rows preserving the flat-path sequence". Decision: 4-row
sequence (preserve all 3 D-5 flat rows; emit the new hierarchical
aggregate AS A 4TH SIBLING ROW). The flat dispatch runs FIRST
(BaiArmUpdated + TemplatePromoted/TemplateCandidateRejected emitted via
the canary-gated path); the hierarchical aggregate is emitted AFTER as a
separate writer call. Dashboard tooling that already parses
BaiArmUpdated keeps working; HierarchicalBaiRoundCompleted is the
additive hierarchical signal.

DECISION-11 — Canary gate composition (D-7 preserved)
The hierarchical layer composes ABOVE the D-7 canary gate, NOT in place
of it. The flat-path `emit_bai_round_with_canary` runs first (with its
existing canary suppression of promotion); the hierarchical aggregate
row reports the canary outcome via `canary_gate_passed: bool` and via
the `selection_basis` field (CanaryOverride when blocked).

DECISION-12 — Provenance for synthetic fixtures (pattern-10)
The integration test's SyntheticNonIidBandit fixture carries
`Provenance::ElaiSynthetic` semantically (the data is generated by the
test itself; no real ELAI task data is touched). The fixture is bound
to a deterministic seed (`20260526`) and the second
`fixture_is_deterministic_under_fixed_seed` test pins this.

VERIFICATION (cargo check passes; no test runs per memory feedback
`feedback_no_cargo_test_unless_permitted`):
  * cargo check -p elai-orchestrator --lib: green (10 unrelated warnings)
  * cargo check -p elai-orchestrator --tests: green
  * cargo check -p elai_cc_contracts --tests: green
  * cargo check -p elai-cli: green

KNOWN-RESIDUAL (open after mitigations):
  * GAP-2 / GAP-3 are RUNPOD-PENDING per DECISION-7.
  * GAP-6 family routing map is RUNPOD-PENDING per DECISION-6 (the
    single-FamilyId(0) wiring is correctness-preserving but loses the
    cross-family signal until a routing map ships).
  * Other 4 GAPs (paper, env-gate, dispatch, hierarchical state) are
    addressed by this build.

DECISION-7b CLARIFICATION (2026-06-06, s-plan-audit follow-up)
This entry supersedes the parts of DECISION-6 / DECISION-7c that the code
has since outrun. Both the hierarchical-vs-flat resolution AND the
family-routing gap are restated here with the ground truth grepped from
the current tree, so neither the stale audit status note
(workflow/sessions/s-plan-audit-2026-06-04/plan-status/D/D-29_*.md) nor
the original DECISION-6 premise can be cited going forward.

A. PATH A vs PATH B IS RUNPOD-PENDING (the headline of this clarification).
   Plan §3 closure forks on a single comparison: hierarchical regret <=
   flat-BAI regret on the synthetic benchmark decides Path A (productize);
   equal-or-worse decides Path B (infeasibility note + concrete blocker).
   That comparison number has NEVER been measured. Status: RUNPOD-PENDING.
   It is NOT resolvable host-side because no flat-vs-hierarchical regret
   trace exists in metrics/ or workflow/ (grep returned zero rows). Faking
   a number to force a Path is BANNED (CLAUDE.md grounded-by-default +
   executor no-fabricated-measurements). The fork stays OPEN until the run.

   THE EXACT BENCHMARK THAT RESOLVES IT (pre-committed so hindsight is
   impossible):
     * Fixture: the synthetic non-IID bandit, 5 families x 10 arms = 50
       arms (plan §2). Within-family arms correlated, across-family
       independent. Deterministic seed 20260526 (DECISION-12; pinned by
       `fixture_is_deterministic_under_fixed_seed` /
       `selection_is_deterministic_under_fixed_seed` in
       crates/contracts/tests/hierarchical_bai_contract.rs).
     * Horizon: 1000 rounds (plan §2 "over 1000 pulls").
     * Metric: cumulative simple regret of flat BAI (D-5
       crates/orchestrator/src/bai.rs) vs hierarchical BAI
       (crates/orchestrator/src/bai_hierarchical.rs), each driven from the
       SAME seeded fixture so the only difference is the estimator.
     * Compute: a RunPod CPU session that actually executes 1000 verdict
       rounds end-to-end. The per-round math is sub-millisecond at K=50
       (O(K * MAX_GIBBS_ITERATIONS), MAX_GIBBS_ITERATIONS=100 per
       DECISION-8); the RUNPOD requirement is a real 1000-round RUN that
       produces 1000 HierarchicalBaiRoundCompleted SSB rows, not raw FLOPs.
     * Decision rule (pinned at DECISION-7b above, restated verbatim):
       Path A iff hierarchical regret is >= 5% lower than flat-BAI regret
       AND the bootstrap CI on the regret DELTA does not cross zero.
       Otherwise Path B with >=1 concrete blocker named.
   Where the resolving number lands when measured: append a PATH-RESOLVED
   entry to THIS file (flat regret, hierarchical regret, delta, CI, Path
   chosen) and, if Path A, register the ELAI_BAI_MODE=hierarchical
   promotion in wire-dark-allowlist.toml. Until then the production default
   stays BaiMode::Flat (runtime_config.rs:423 returns Flat on unset/unknown
   ELAI_BAI_MODE), so this RUNPOD-PENDING fork ships ZERO risk to prod.

B. FAMILY-ROUTING GAP — CORRECTED. DECISION-6 / DECISION-7c recorded that
   `task_runner::dispatch_bai_round` assigned ALL arms to FamilyId(0) until
   a routing map shipped. That premise is now STALE. Ground truth grepped
   2026-06-06 from crates/orchestrator/src/task_runner.rs (the comment
   block tags it "D-29 RUNPOD-COMPLETE 2026-05-29"):
     * Per-arm assignment loop (task_runner.rs ~10307-10313) calls
       crate::bai_hierarchical::family_id_for_template_or_mixed(&chain,
       arm.template_id) for EACH arm — real TemplateId -> Domain ->
       FamilyId routing through the live ArchiveChain, not a FamilyId(0)
       blanket.
     * The aggregate-row family (~10318) and the per-arm SSB stamp
       (~10383, comment "per-arm family stamp now derived from the
       production routing map (was hard-coded 0)") use the same resolver.
     * The resolver (bai_hierarchical.rs:183 family_id_for_template_or_mixed)
       maps through family_id_for_domain (a closed 6-family enum:
       Code/Math/Prose/Mixed/Planning/ToolUse) and falls back to
       FAMILY_ID_MIXED — NOT FamilyId(0) — only on cold-start (template not
       yet in the chain). FAMILY_ID_MIXED carries the weakly-informative
       Beta(2,2) hyperprior, the least-bad unknown-domain default.
   So the cross-family signal IS active in production dispatch; the only
   residual is the cold-start fallback to FAMILY_ID_MIXED for arms whose
   template has not yet been appended to the archive chain. The bare
   "all FamilyId(0)" gap from DECISION-6 is CLOSED. The remaining hardcoded
   FamilyId(0) sites are TEST FIXTURES ONLY (bai_hierarchical.rs ~743-778,
   assign_arm(.., FamilyId(0)) inside #[cfg(test)] unit tests) and do not
   touch the production path. NOTE: even with routing live, the PRODUCTION
   benefit of multi-family pooling is only OBSERVABLE once item A is run;
   correct routing without a measured regret delta is necessary-not-
   sufficient for Path A.

OFF-AGENT ARTIFACT for this clarification (not the agent's own prose):
  * the existing contract crates/contracts/tests/hierarchical_bai_contract.rs
    asserts SsbActionType::HierarchicalBaiRoundCompleted round-trips
    (as_sql_str / ssb_action_type_from_str_public + JSON + msgpack) and
    that SCHEMA_VERSION_PUBLIC >= 146 — i.e. the off-agent SSB row this
    feature emits is real and version-locked.
  * the production dispatch in task_runner.rs (dispatch_bai_round, the
    real post-consensus entrypoint at ~10173, reached from ~9879) is the
    code that emits that row and now carries the real family routing.
  Neither artifact was authored by this clarification; both are grepped
  ground truth. What this clarification does NOT and CANNOT assert is the
  regret number (item A) — that is the genuine RUNPOD-PENDING residual.
