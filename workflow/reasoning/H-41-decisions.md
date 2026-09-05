H-41 decisions — live checker-coverage + checker-gap INSTRUMENT
date: 2026-06-08
session: s-10612

---

WHAT

H-41 instruments the already-shipped D-10/D-11 sound-checker-first preference
(grounding_mode.rs:157 Rule 5) by adding:

1. mode_a_coverage() — rolls up claim_routed SSB rows to compute the fraction
   of Factual/Computational claims that reached a Constructive (sound) checker.
   No new action_type variant; reuses shipped rows (GD-1).

2. CheckerGapDetected SSB beacon — emitted by route_claim_with_gap() (a thin
   wrapper over route_claim, keeping the existing signature intact, RISK-1
   mitigation) when a Factual/Computational claim falls to non-Constructive
   because no VerifierKind::Deterministic is in OrchContext.available_verifiers.

3. classify_complexity_taxon() — Xu arXiv:2401.11817 Table-3 heuristic. Returns
   NpCompleteOrCombinatorial / Undecidable / DecidableNoChecker / Unknown.
   SHORT OR AMBIGUOUS TEXT -> Unknown, never a guess (RISK-2).

4. checker_gap_backlog() — read-side roll-up over CheckerGapDetected rows,
   ranked by frequency x taxon_priority descending.

5. elai checker-gaps CLI + get_checker_gap_backlog / get_coverage_report Tauri
   commands + CheckerCoveragePanel.svelte frontend.

---

KEY DECISIONS

D-1: ROUTE_CLAIM SIGNATURE NOT CHANGED (RISK-1 MITIGATION)
route_claim() is a pub async fn with one production call site (task_runner.rs:11519).
Adding a parameter would break all callers. Decision: add route_claim_with_gap()
as a new pub async fn that wraps route_claim() and adds the gap-detect logic after
the routing decision. Existing callers remain unbroken; H-41 uses the wrapper.
task_runner.rs can migrate independently once the instrument is proven.

D-2: WIRE-DARK DEFAULT (R9)
ELAI_CHECKER_GAP_ENABLE env var required for gap emission. Wire-dark-allowlist.toml
row added (kill_switch type). Default-OFF so existing route_claim callers are
unaffected until the wrapper is adopted by task_runner.rs.

D-3: UNKNOWN NOT GUESS (RISK-2)
classify_complexity_taxon() returns Unknown for text < 10 chars or when no keyword
matches AND claim_type is not Factual/Computational. This is the correct posture:
under-ranking (Unknown = weight 1) is safer than over-ranking (NP-complete = 2)
for a heuristic classifier. Misclassification is an instrument flaw, not an
incorrect routing decision (the preference is already shipped, H-41 only measures).

D-4: COVERAGE = REACH NOT CORRECTNESS (GD-5)
mode_a_fraction measures the fraction of Factual/Computational claims that REACHED
a Constructive mode. It does NOT certify the checker's correctness — a checker bug
producing a false Pass is H-26/H-27's domain. This distinction is intentional and
is stated in the panel UI and decision doc.

D-5: INSTRUMENT NOT GATE
H-41 does not BLOCK a Mode-B fallback. It MEASURES it. A gate that forces refusal
on an uncovered NP-complete claim is a follow-on once the backlog/coverage numbers
justify the restriction. Forcing refusal prematurely degrades UX; measuring first
grounds the decision (GD-4).

---

EXPECTED TEST OUTCOME (contract test h41_checker_coverage_contract.rs)

schema_version_is_at_least_219                       PASS (219 >= 219)
checker_gap_detected_wire_tag_is_correct              PASS ("checker_gap_detected")
checker_gap_payload_action_type_round_trips           PASS (SsbActionType::CheckerGapDetected)
np_complete_keyword_yields_np_complete_taxon          PASS (NpCompleteOrCombinatorial, priority 2)
undecidable_keyword_yields_undecidable_taxon          PASS (Undecidable, priority 2)
short_claim_text_yields_unknown_not_guess             PASS (Unknown)
covered_factual_routes_constructive_no_gap            PASS (Constructive, 0 gap rows)
uncovered_factual_emits_checker_gap_row               PASS (non-Constructive, 1 gap row)
gap_not_emitted_when_kill_switch_off                  PASS (0 gap rows, default-OFF)
mode_a_fraction_matches_fixture                       PASS (3/4 = 0.75)

---

UNKNOWNS

- Taxon classifier keyword coverage is sparse. Real-world NP-complete claims
  may not use the exact trigger phrases. The Unknown fallback ensures no
  over-ranking; calibration against a labelled claim corpus is the follow-on.

- mode_a_fraction on a near-empty checker registry reads ~0. This is the
  CORRECT signal ("invest in checkers"), not a bug. Dashboard panel should
  communicate this context.

- The one production caller (task_runner.rs::dispatch_claim_routing) still uses
  route_claim directly. Switching it to route_claim_with_gap is the next step
  after the kill-switch is validated in staging.

---

GROUNDING

Kambhampati arXiv:2402.08115 S5: LLM self/peer critique collapses on reasoning;
sound external verifiers work. Mode-A coverage is the verification lever.

Xu arXiv:2401.11817 Table 3: NP-complete (SAT, SubsetSum, graph-colouring) and
undecidable (first-order entailment) task classes force LLM hallucination.
External solver or refusal is the only formally sound escape (Corollary 1 = RD-3).

RS-1 grep-confirmed (2026-06-04): grounding_mode.rs:159 Rule 5 ships the
constructive-when-possible preference. H-41 instruments it, does not re-implement.

SCHEMA_VERSION: 219, TOTAL_VARIANTS: 530, confirmed in ssb.rs + action_type_lock.rs.
