D-4 — tripwire stack decisions (closing pass)
==============================================

Scope
-----
This document covers the closure pass for D-4 (tripwire stack). The D-4
substrate (5 detector free fns + TripwireMonitor + auto_revert +
TripwireFired / TemplateAutoReverted SSB variants) was already shipped at
SCHEMA_VERSION 70 in a prior session; the original cross-check report
flagged 11 gaps but several were stale (papers present, dashboard wired).
The 5 real gaps closed here are GAP-1..GAP-5 below.

DECISION-1 — SCHEMA bump 107 -> 108 (NOT plan's 49->50, NOT 69->70)
-------------------------------------------------------------------
The original D-4 plan was written when SCHEMA_VERSION was 48 and called
for a 49 -> 50 bump. Between plan-write and this closing pass, the
substrate already shipped at 69 -> 70 (carrying TripwireFired +
TemplateAutoReverted), and a concurrent C-38 landing pushed SCHEMA to
107 (pagerank + dkg-rebuild trio). This closing pass adds one new
variant (TripwireFiredConcurrent — GAP-2) so the actual atomic bump is
107 -> 108. The plan numbers are stale; the in-code value at
crates/orchestrator/src/ssb.rs SCHEMA_VERSION is the canonical truth.

The 9-tuple cascade for the bump is the same shape used by every prior
SCHEMA bump (D-3 closure, C-5, C-36, C-37, C-38):
  1. const SCHEMA_VERSION       107 -> 108
  2. enum SsbActionType          add TripwireFiredConcurrent arm
  3. as_sql_str() match          add "tripwire_fired_concurrent"
  4. from_str() match            add "tripwire_fired_concurrent" arm
  5. SQL CHECK allowlist         add 'tripwire_fired_concurrent'
  6. enum SsbPayload             add TripwireFiredConcurrent(TripwireFiredConcurrentPayload)
  7. action_type() match         add arm
  8. test_payload arm            add deterministic sample
  9. all_test_payloads list      add deterministic sample
  + action_type_lock.rs          TOTAL_VARIANTS 314 -> 315
                                 ALL_VARIANTS array append
                                 exhaustive_match_guard append
  + action_type_lock.golden      regen with 315 entries, alphabetical
                                 insertion of "tripwire_fired_concurrent"

DECISION-2 — Free-fn pattern + TripwireMonitor struct kept; no TripwireDetector trait
--------------------------------------------------------------------------------------
The D-4 plan blurb referenced a "TripwireDetector trait" implemented by
each of the 5 detectors. The shipped substrate uses a flat function
family (kl_divergence, pearson_correlation, iqr_outlier,
perplexity_anomaly + DdmState helper) plus a single TripwireMonitor
struct that composes them. The plan-level "trait" language is renamed in
the doc index to "function family + monitor" — the trait abstraction
was never needed because there is exactly one composition site
(TripwireMonitor::observe) and no downstream caller dispatches on
detector identity polymorphically. Removing the trait keeps the
detectors as pure functions whose grounding is auditable (each function
has #[must_use] + a docstring citing the paper section).

DECISION-3 — "~80% coverage" claim restated as "~70-90% per arxiv:2507.05619 §5.3"
----------------------------------------------------------------------------------
The original plan text cited arxiv:2507.05619 claiming "approximately
80% coverage of reward-hacking categories" for the multi-detector
ensemble. The actual numbers in §5.3 are a per-category breakdown:
  * 39.8% specification gaming  (cat 1)
  * 31.2% proxy optimisation    (cat 2)
  * 18.7% objective misalignment (cat 3)

Summed across two categories the cumulative coverage is 70.8%; across
three categories it is 89.7%. The "approximately 80%" claim is an
interpolation that does not appear verbatim in the paper. Where the
number is cited in code comments or operator-facing docs, it is restated
as "approximately 70-90% per arxiv:2507.05619 §5.3 (39.8% + 31.2% +
18.7%)" so the operator can audit the source. See elai_cc.toml
[tripwire] header comment for the canonical phrasing.

DECISION-4 — CorrelationDrop delta_rho threshold is an implementation choice
---------------------------------------------------------------------------
The plan asserted that the CorrelationDrop detector's delta_rho = 0.5
threshold was grounded in arxiv:2210.10760 (Gao et al, RM
over-optimisation). On re-reading the paper: §4 establishes the
proxy/gold reward correlation degradation pattern (which the
CorrelationDrop detector implements) but does NOT specify a numeric
threshold. 0.5 is a calibrated implementation choice tuned against the
ELAI baseline (the value below which proxy and gold disagree often
enough to warrant auto-revert).

In code comments and the toml section, the threshold is labelled:
  "implementation choice not directly grounded in arxiv:2210.10760;
   calibrated against ELAI baseline (see D-4-decisions.md DECISION-4)"

so a later operator does not falsely treat the number as paper-grounded.

DECISION-5 — Downstream consumers are D-26 only (not D-19, not D-27)
---------------------------------------------------------------------
The user-prompt narrative claimed three downstream consumers of the
tripwire SSB rows: D-19 (PRMBench), D-26 (concurrent-fire diagnostics),
and D-27 (canary observability). A grep sweep over the entire
crates/orchestrator/src + crates/dashboard-v2 tree returned:
  * D-26: actually consumes (the TripwireFiredConcurrent distinction is
    its motivation — branch on wire tag rather than join raw rows)
  * D-19: ZERO grep hits for tripwire/TripwireFired in the PRMBench
    runner, payload, or contract test
  * D-27: ZERO grep hits for tripwire in the canary observability path

D-19 and D-27 are NOT consumers. The D-4 dashboard wire surface
(commands_d4::get_recent_tripwires + TripwirePanel.svelte) is the sole
non-D-26 reader. Future work that touches D-19 or D-27 must not
inadvertently couple them to the tripwire stack without a corresponding
contract test landing here.

GAP closure summary
-------------------
GAP-1: SignalSource enum + payload field added.
  Files: crates/orchestrator/src/tripwire.rs (enum + impl + mapping),
         crates/orchestrator/src/ssb.rs (TripwireFiredPayload.source
         field with #[serde(default)] for legacy compat).
  No SCHEMA bump required for GAP-1 alone — the field is additive on
  the existing TripwireFired variant.

GAP-2: TripwireFiredConcurrent SSB variant added (SCHEMA 107 -> 108).
  Files: crates/orchestrator/src/ssb.rs (9-tuple cascade),
         crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS
         314 -> 315, ALL_VARIANTS append, exhaustive_match_guard arm),
         crates/orchestrator/tests/action_type_lock.golden (regen with
         "tripwire_fired_concurrent" inserted alphabetically).

GAP-3: [tripwire] section in elai_cc.toml.
  Files: elai_cc.toml (new L3-locked section with 8 keys),
         crates/cli/src/runtime_config.rs (TripwireConfigToml mirror +
         field on ElaiRuntimeConfig + accessor + to_tripwire_config
         converter mapping wire keys to the in-code TripwireConfig
         field names).

GAP-4: Contract tests.
  File: crates/contracts/tests/tripwire_contract.rs (8 tests covering
        SCHEMA floor, SignalSource serde, kind->source mapping, payload
        source field + legacy compat, TripwireFiredConcurrent
        round-trip + wire-tag resolution, [tripwire] section defaults
        parity with TripwireConfig::defaults(), DDM warning/drift
        mutual exclusion).

GAP-5: This decisions file (workflow/reasoning/D-4-decisions.md).
  Note: a similarly-named workflow/reasoning/D4-decisions.md exists from
  an unrelated 2026-05-05 Obscura session. The hyphenated form D-4 is
  the canonical D-plan naming.

Dashboard wire-level fix
------------------------
TripwireRow type is declared in:
  * crates/dashboard-v2/src-tauri/src/commands_d4.rs (Rust)
  * crates/dashboard-v2/src/lib/drawers/TripwirePanel.svelte (TS inline)

Per the exploration note, this is a hand-authored mirror (the dashboard
crate does NOT use ts-rs — types.ts is hand-authored and is the
canonical shared TS surface). The fix is to lift the TripwireRow TS type
into crates/dashboard-v2/src/lib/api/types.ts (shared location, parallel
to FsmSnapshot / PlanProgress / etc) and import it from TripwirePanel,
widening the kind union to admit the new "fired_concurrent" tag. This
matches the existing types.ts pattern and avoids introducing a new
crate-graph dependency (ts-rs would force a build-step + a generated TS
artifact path the dashboard does not currently consume).

Explicit non-scope
------------------
Out of scope per the directive:
  * The 5 existing detector free fns at tripwire.rs — already shipped.
  * The TripwireFired existing variant — only ADD source field (GAP-1
    additive extension); no rename, no field removal.
  * D-19 / D-27 — they don't consume tripwires per grep evidence.

References
----------
* Plan: docs/thefinalglance/D-plan/D-4_tripwire_stack.md
* Cross-check (stale rows present):
  workflow/sessions/s-25120/cross-check-recheck/D-4_tripwire_stack-report.md
* Papers (ingested):
  - workflow/papers/reward-hacking-detection-2507.05619-interface.md
  - workflow/papers/concept-drift-2004.05785-interface.md
  - workflow/papers/rm-overoptimization-2210.10760-interface.md
* Concurrent SCHEMA landing: C-38 (107) — workflow/reasoning/C-38-decisions.md
* Prior D-3 closure (105) — workflow/reasoning/D-3-decisions.md
