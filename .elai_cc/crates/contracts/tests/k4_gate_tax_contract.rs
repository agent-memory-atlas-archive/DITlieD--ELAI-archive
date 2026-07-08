//! K-4 — composed gate-tax budget contract tests.
//!
//! Covers the properties `crates/orchestrator/src/benchmarks/gate_tax_runner.rs`'s
//! own `#[cfg(test)]` module cannot assert from inside the producing crate
//! (RECON/ADV items from the K-4 v2 plan + this dispatch's contract):
//!   (a) Leg B composes `Σ(unit_cost × fire_count)` from a FIXTURE journal
//!       via the pure `compose_gate_tax` function.
//!   (b) the in-scope-coding gate set is `evidence::all_gates()`-derived, not
//!       a literal list divorced from the shipped registry.
//!   (c) the RIDER-b coverage table fails loud (ABORT-2b) on an injected
//!       untagged gate — the anti-silent-narrowing guard (ADV-5).
//!   (d) same-seed byte-identity (GD-2) on the RunnerPresent path.
//!   (e) a dry journal (this crate's test cwd carries no `sessions/ssb.db`)
//!       reports `RunnerStatus::RunnerPresent`, never a fabricated 0-tax
//!       `Measured` pass (ADV-1, the K-18a lesson).
//!   (f) SP2: `audit_gate_tax_ms` is populated (MOVE 4b) and is a DISTINCT
//!       field, never summed with `coding_task_gate_tax_ms` (rider a).
//!   (g) SP2: `evaluate_ceiling_breach` fails closed while
//!       `CODING_TASK_GATE_TAX_CEILING_MS` is CEILING-PENDING (ABORT-5).
//!   (h) SP2 (MOVE 6): a deliberately spammy `over_block_rate` trips the
//!       breach path.

use elai_orchestrator::benchmarks::gate_tax_runner::{
    build_coverage_table, compose_gate_tax, evaluate_ceiling_breach, run_gate_tax,
    GateCoverageTag, GateTaxError, RunnerStatus,
};
use elai_orchestrator::evidence;

/// (a) Leg B composes `Σ(unit_cost × fire_count)` from a FIXTURE journal —
/// `compose_gate_tax` is a pure, deterministic function of its inputs (GD-2's
/// recompute leg), independent of any live microbench or journal read.
#[test]
fn leg_b_composes_from_a_fixture_journal() {
    let fixture_journal = vec![
        ("knowledge_readiness".to_string(), 0.25, 4u64),
        ("grounding_sweep".to_string(), 1.5, 2u64),
        ("anti_fabrication".to_string(), 0.1, 10u64),
    ];
    let (total, top) = compose_gate_tax(&fixture_journal);
    let expected = 0.25 * 4.0 + 1.5 * 2.0 + 0.1 * 10.0;
    assert!((total - expected).abs() < f64::EPSILON, "got {total}, expected {expected}");
    assert_eq!(top.len(), 3, "all 3 contributors ranked (<= TOP_CONTRIBUTORS_N)");
    assert_eq!(top[0].gate_id, "grounding_sweep", "3.0ms is the highest single contribution");
}

/// (b) the in-scope-coding gate set on a real report resolves against the
/// SHIPPED `evidence::all_gates()` registry — never a hand list divorced
/// from the real gate implementations (FIX-1's telemetry-derived mandate).
#[test]
fn in_scope_coding_gates_resolve_against_the_real_evidence_registry() {
    let report = run_gate_tax(999).expect("run_gate_tax");
    let all_gate_names: Vec<&'static str> = evidence::all_gates().iter().map(|g| g.name()).collect();

    let coding_gate_ids: Vec<&str> = report
        .coverage
        .iter()
        .filter(|entry| entry.tag == GateCoverageTag::InScopeCoding)
        .map(|entry| entry.gate_id.as_str())
        .collect();

    assert_eq!(coding_gate_ids.len(), 11, "the 11 settled in-scope-coding FSM gates");
    for gate_id in coding_gate_ids {
        assert!(
            all_gate_names.contains(&gate_id),
            "in-scope-coding gate '{gate_id}' must resolve against evidence::all_gates()"
        );
    }
    assert_eq!(report.n_untagged, 0, "RIDER-b: a successful report always has 0 untagged");
}

/// (c) / ADV-5: an injected gate_id NOT in the canonical registry must FAIL
/// the coverage build (ABORT-2b) — an in-scope gate can never quietly fall
/// out of the composed number without the run itself failing.
#[test]
fn coverage_table_fails_loud_on_an_untagged_gate_injection() {
    let ids = vec![
        "knowledge_readiness".to_string(),
        "totally_untagged_synthetic_gate_xyz".to_string(),
    ];
    let result = build_coverage_table(&ids);
    match result {
        Err(GateTaxError::UntaggedManifestGate { gate_id }) => {
            assert_eq!(gate_id, "totally_untagged_synthetic_gate_xyz");
        }
        other => panic!("expected UntaggedManifestGate, got {other:?}"),
    }
}

/// (d) / GD-2: two independent `run_gate_tax` calls at the same seed must be
/// byte-identical on the RunnerPresent path (fixture-driven, no live
/// wall-clock microbench). This crate's test cwd carries no
/// `sessions/ssb.db`, so the RunnerPresent branch is what real test
/// execution exercises here — asserted explicitly rather than assumed.
#[test]
fn same_seed_runner_present_reports_are_byte_identical() {
    let r1 = run_gate_tax(4242).expect("run 1");
    let r2 = run_gate_tax(4242).expect("run 2");
    assert_eq!(
        r1.runner_status,
        RunnerStatus::RunnerPresent,
        "contracts-crate test cwd carries no sessions/ssb.db (dry journal by construction)"
    );
    assert_eq!(r2.runner_status, RunnerStatus::RunnerPresent);

    let j1 = serde_json::to_string(&r1).expect("serialise r1");
    let j2 = serde_json::to_string(&r2).expect("serialise r2");
    assert_eq!(j1, j2, "same-seed RunnerPresent reports must be byte-identical (GD-2)");
}

/// (e) / ADV-1 (the K-18a lesson): a dry journal must report
/// `RunnerStatus::RunnerPresent`, never fabricate a `Measured` composition
/// over zero real fire-count data, and never silently report a 0.0 tax as a
/// pass.
#[test]
fn dry_journal_never_reports_measured_or_a_silent_zero_tax() {
    let report = run_gate_tax(1).expect("run_gate_tax");
    assert_eq!(report.runner_status, RunnerStatus::RunnerPresent);
    assert!(
        report.coding_task_gate_tax_ms > 0.0,
        "RunnerPresent must compose a non-zero fixture tax, never a silent 0.0 pass"
    );
    assert_eq!(
        report.task_class_label, "fixture",
        "RunnerPresent's task_class_label is always the fixture proxy"
    );
    // rider a: the two headline numbers are separate fields, never summed.
    // SP2: audit_gate_tax_ms is now MEASURED (MOVE 4b) — never negative, but
    // no longer asserted == 0.0 (that was the SP1 placeholder).
    assert!(report.audit_gate_tax_ms >= 0.0, "MOVE 4b: measured, never negative");
    assert!(report.audit_unbounded, "CEILING-PENDING[K-4]: no confirmed audit ceiling yet");
}

/// (f) SP2 / MOVE 4b: `audit_gate_tax_ms` is populated and is a field
/// independent of `coding_task_gate_tax_ms` — two different seeds (which
/// vary the CODING leg's fixture path not at all in practice, but exercise
/// two independent `run_gate_tax` calls) yield the SAME audit number,
/// because the per-audit leg is a process-memoized machine/checker-set
/// property (see `gate_tax_audit_leg.rs`'s module doc), never derived from
/// or summed with the coding number.
#[test]
fn audit_gate_tax_is_populated_and_independent_of_the_coding_task_number() {
    let r1 = run_gate_tax(11).expect("run 1");
    let r2 = run_gate_tax(22).expect("run 2");
    assert!(r1.audit_gate_tax_ms >= 0.0, "MOVE 4b: measured, never negative");
    assert_eq!(
        r1.audit_gate_tax_ms, r2.audit_gate_tax_ms,
        "rider a: the per-audit leg is independent of the coding-task seed/composition"
    );
}

/// (g) SP2 / MOVE 3 / ABORT-5: `CODING_TASK_GATE_TAX_CEILING_MS` is
/// CEILING-PENDING[K-4] (no user-confirmed value); every real report's
/// `coding_task_gate_tax_ms` is proven > 0.0
/// (`run_gate_tax_never_reports_zero_tax_as_a_silent_pass`), so the breach
/// evaluation must ALWAYS trip while pending — fail-closed, never a silent
/// pass.
#[test]
fn evaluate_ceiling_breach_fails_closed_while_coding_ceiling_is_pending() {
    let report = run_gate_tax(33).expect("run");
    let verdict = evaluate_ceiling_breach(&report);
    assert!(verdict.is_some(), "ABORT-5: CEILING-PENDING must fail closed, never a silent pass");
    assert!(verdict.unwrap().contains("coding_task_gate_tax_ms"));
}

/// (h) SP2 / MOVE 6: a deliberately spammy `over_block_rate` (synthetic,
/// hand-set on an otherwise real report) trips the breach path distinctly
/// from the coding-ceiling reason.
#[test]
fn evaluate_ceiling_breach_trips_on_a_spammy_over_block_rate() {
    let mut report = run_gate_tax(44).expect("run");
    report.coding_task_gate_tax_ms = 0.0; // isolate the over-block condition
    report.over_block_rate = 0.2; // MOVE 6: a deliberately spammy synthetic value
    let verdict = evaluate_ceiling_breach(&report);
    assert!(verdict.is_some(), "MOVE 6: any over-block on the benign corpus must trip the breach path");
    assert!(verdict.unwrap().contains("over_block_rate"));
}
