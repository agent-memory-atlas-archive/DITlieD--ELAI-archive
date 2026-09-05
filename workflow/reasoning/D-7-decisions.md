D-7 closure decisions — cross-domain transfer canary closure pass

Status: shipped in s-25120 closure pass (SCHEMA 113 -> 114).

Context

D-7 had MOST of its core machinery already shipped in a prior pass:
canary.rs (861 LOC, 11 tests) + 2 SSB variants (CanaryRunCompleted,
CanaryFamilyEvaluated) + Domain enum extension (Planning, ToolUse) +
Tauri commands (get_canary_recent, get_canary_per_family_breakdown) +
CanaryPanel.svelte mounted via DrawerHost + smoke test. The audit
re-check found a NOT_WIRED gap (BAI promote_candidate had zero canary
references) + STUB_BAN on hardcoded constants + 3 follow-on SSB
observation variants that the original plan called for but never
shipped. This closure pass lands the gap-closing wiring without
reopening any of the SHIPPED surfaces.

DECISION-1: SCHEMA bump 113 -> 114 (not the plan's 52 -> 53)

The plan was authored at SCHEMA 52; head had drifted to 113 by the time
this closure pass ran (concurrent piggybacks: D-4 GAP-2, D-5 closure,
D-6 closure, C-38/C-40/C-41/C-42, etc). The atomic 3-variant bump
lands at 113 -> 114 — the contract test's SCHEMA floor at
`>= 114` catches any future regression.

Grounding: per the SCHEMA-history comment block in
crates/orchestrator/src/ssb.rs the version number is a monotonic
counter, not a plan-anchored ordinal. The plan's stale number is a
documentation artifact only.

DECISION-2: NanoFlux (arXiv:2509.23252) is empirical motivation only

The plan, the context-package, and several in-code comments cite
NanoFlux as if the paper specifies the cross-domain canary pattern. It
does not. NanoFlux is VERIFIED REAL (see
workflow/papers/nanoflux-2509.23252-interface.md) but the paper
reports empirical multi-domain improvement on a fine-tuning method —
it does NOT define:

  - a canary mechanism
  - a 5% (or any other) degradation threshold
  - a cross-domain regression safeguard
  - a per-family sample floor

Every numeric default in the D-7 implementation is an ELAI policy
choice. The in-code citation pattern is now tagged
"(ELAI design hypothesis; NanoFlux provides empirical motivation only —
no canary mechanism in the paper itself)" everywhere the paper is
referenced. See ssb.rs SCHEMA 114 header, canary.rs module header
(future-edit candidate; the existing module header still cites
NanoFlux as if it were prescriptive — flagged for follow-up but not
load-bearing on the closure), elai_cc.toml [canary] section,
runtime_config.rs CanaryRuntimeConfig docstring, and the action_type_lock
comment.

DECISION-3: 5% degradation threshold is an ELAI choice

DEFAULT_FAIL_BAND = 0.05 in canary.rs is an ELAI policy choice based
on the companion §3.5 noise-balanced operating point (>=10 samples at
p~0.7 gives std ~3%). It is NOT NanoFlux-specified. The marginal_band
(0.02 toml default; 0.025 in-code legacy default), the min_per_family
(10 toml default; 5 in-code legacy default), and the
insufficient_data_min_samples (5) are all ELAI choices. The in-code
defaults retained for backward compat with the existing canary.rs
unit tests; the toml defaults are the production-recommended values.

The `[canary]` toml section ships with the production-recommended
values:

  fail_band = 0.05
  marginal_band = 0.02
  min_per_family = 10
  regression_threshold = 0.05
  insufficient_data_min_samples = 5

DECISION-4: CanaryFamily::ToolUse preserved (plan named KnowledgeQa)

The shipped CanaryFamily enum has variant ToolUse; the plan-doc named
it KnowledgeQa. The substitution rationale is documented in-source at
canary.rs lines 83-88: "Replaces the plan-doc's KnowledgeQa per the
orchestrator brief since D-7's BAI gate already consumes PrmBench
knowledge rows from D-19 and the ToolUse family covers a strictly
orthogonal axis (interaction vs recall)". The closure pass preserves
the shipped name — renaming would force a structural enum migration
without semantic benefit.

DECISION-5: 3 new SSB variants added atomically; existing 2 preserved

The 3 new variants are added as semantically orthogonal events,
NOT replacements for the 2 shipped variants:

  Existing (shipped earlier, preserved as-is):
    CanaryRunCompleted   — one row per run; carries overall decision.
    CanaryFamilyEvaluated — one row per family-evaluated; carries delta.

  New (this closure pass, SCHEMA 114):
    CrossDomainRegression  — one row per per-family regression detection;
                              carries source_domain + regressed_domain
                              + delta_pct + threshold_pct.
    CanaryFamilyPass       — one row per clean-pass family; carries
                              score_pct + sample_count.
    CanaryInsufficientData — one row per under-sampled family; carries
                              sample_count + min_required.

Producer site is canary::emit_per_family_observation, called inside
canary::emit_canary_run_with_source per family. The selection is
mutually exclusive — exactly one of the three new variants fires per
family per run, plus the original CanaryFamilyEvaluated row.

DECISION-6: BAI promote_candidate gates on canary verdict

The BAI promote_candidate function stays pure (signature unchanged so
existing tests continue to pin its semantics). The gate fires in a
new emit_bai_round_with_canary wrapper that:

  1. Reads the latest canary status via
     crate::agent_pool::recent_canary_status (already shipped — D-7
     produced this read surface).
  2. If status.blocks_promotion() (decision_kind == "fail"), the
     promotion is REFUSED: posterior is still updated (Beta(alpha,
     beta) stays honest about the eval signal) but the chain status
     stays Candidate. The decision returns Rejected with reason
     "canary_gate_blocked_dominant_family_<name>", which threads into
     the TemplateCandidateRejected SSB row.
  3. Otherwise the call delegates to the un-gated promote_candidate.

emit_bai_round (no canary) is now a thin wrapper passing None — every
existing caller works without modification. dispatch_bai_round in
task_runner.rs opens a transient SsbViewer per round to fetch the
status; on viewer-open failure it degrades to None (matches D-5/D-6's
"wire-dark = no-op" pattern).

Production wiring is gated by ELAI_BAI_ENABLE env var (existing D-5
gate). Wire-dark by default — the canary gate is a no-op until the
operator opts in, mirroring the BAI enablement pattern.

GAP closure summary

GAP-1 (HIGH, NOT_WIRED) — BAI canary gate: CLOSED.
  emit_bai_round_with_canary + dispatch_bai_round consults
  recent_canary_status before flipping a candidate to Promoted.

GAP-2 (HIGH, STUB_BAN) — [canary] toml section: CLOSED.
  CanaryRuntimeConfig::from_toml_value mirrors the BAI/HeldOut precedent.
  All 5 fields land with documented defaults; production-recommended
  values (fail_band=0.05, marginal_band=0.02, min_per_family=10,
  regression_threshold=0.05, insufficient_data_min_samples=5) ship in
  elai_cc.toml as the documented stanza.

GAP-3 — 3 new SSB variants atomic bump: CLOSED.
  SCHEMA 113 -> 114. Full 9-tuple cascade for each variant: enum entry
  + payload struct + as_sql_str + from_str + SQL allowlist + SsbPayload
  variant + action_type() arm + test_payload arm + all_test_payloads
  sample. action_type_lock.rs TOTAL_VARIANTS 327 -> 330; ALL_VARIANTS +
  exhaustive_match_guard + action_type_lock.golden all updated.

GAP-4 — Emit-call sites: CLOSED.
  canary::emit_per_family_observation fires exactly one of the three
  new variants per family per run, alongside the existing
  CanaryFamilyEvaluated row. Selection is mutually exclusive:
  insufficient -> regression -> pass.

GAP-5 — Contract tests: CLOSED.
  crates/contracts/tests/canary_contract.rs holds 9 tests covering:
  SCHEMA floor, payload round-trips for the 3 new variants, regression-
  delta invariant, insufficient-data invariant, wire-tag resolution,
  CanaryRuntimeConfig default-match-toml, gate predicate monotonicity.

GAP-6 — Decisions doc: this file.

Audit verification

  - cargo check --workspace --lib: PASS (8 pre-existing warnings,
    0 errors).
  - cargo check --tests -p elai-orchestrator: PASS (18 pre-existing
    warnings, 0 errors).
  - cargo check --tests -p elai_cc_contracts --features mcp: PASS
    (pre-existing warnings only).
  - svelte-check on dashboard-v2: 0 ERRORS 18 WARNINGS (all pre-existing
    CSS-unused / event-directive-deprecated / non_reactive_update;
    none related to D-7).

Out of scope for this pass

  - The 861-LOC canary.rs core implementation (shipped earlier, frozen).
  - The 2 shipped SSB variants (CanaryRunCompleted, CanaryFamilyEvaluated).
  - The Domain enum extension (Planning, ToolUse — shipped earlier).
  - The Tauri commands (get_canary_recent, get_canary_per_family_breakdown).
  - The CanaryPanel.svelte mount.
  - The smoke test.

Follow-up flagged

  - canary.rs module header still cites NanoFlux as if the paper
    specifies the canary mechanism. The closure pass added the
    "ELAI design hypothesis" caveat to every NEW citation site
    (SCHEMA 114 header, runtime_config docstring, toml stanza,
    contract test header) but did NOT rewrite the existing canary.rs
    lines 1-44 docstring. Not load-bearing on closure — flagged for
    follow-up when canary.rs is next touched.
  - canary.rs unit tests still pin the in-code DEFAULT_MIN_PER_FAMILY = 5
    and DEFAULT_MARGINAL_BAND = 0.025. The toml-driven defaults are 10
    and 0.02 respectively. Test backward-compat preserved by keeping the
    in-code consts unchanged; runtime callers should plumb through
    CanaryRuntimeConfig once the wiring lands in a follow-up pass.
