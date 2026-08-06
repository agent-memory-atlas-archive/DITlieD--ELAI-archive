//! Exhaustiveness lock for the four canonical dispatch enums (D-EXH / ADR-9).
//!
//! Phase 1.2 of the WIRING-FIX-ELAI plan originally specced "strip
//! `#[non_exhaustive]` from 4 dispatch enums". Grounding proved all four already
//! lack the attribute AND are already matched without a wildcard arm, so the
//! edit was a NO-OP. The load-bearing value is THIS regression test, which LOCKS
//! the four enums exhaustive so a future variant cannot be added without either
//! a compile error (round-trip exhaustive-match guard) or a test failure (the
//! explicit variant list + no-wildcard source-contract grep).
//!
//! The four enums + their CANONICAL dispatch sites (verified 2026-06-02):
//!   - `FsmSubstate::as_str`              src/fsm.rs   `impl FsmSubstate {`
//!   - `SkillEventPhase::as_str`          src/ssb.rs   `impl SkillEventPhase {`
//!   - `RouteDecisionEventKind::as_str`   src/ssb.rs   `impl RouteDecisionEventKind {`
//!   - `SsbActionType::as_sql_str`        src/ssb.rs   `/// SQL-compatible string for the CHECK constraint.`
//!
//! Two test families:
//!
//!   (A) ROUND-TRIP / exhaustive-match guard. For FsmSubstate, SkillEventPhase,
//!       and RouteDecisionEventKind an explicit `ALL_*` array enumerates every
//!       current variant; a private `*_match_guard` fn does a wildcard-free
//!       `match` whose every arm returns `()`. rustc requires exhaustivity, so
//!       ADDING a variant without updating the guard (and the array) is a hard
//!       COMPILE error pointing back here. Each variant is then driven through
//!       its dispatch fn and asserted to map to a NON-EMPTY, DISTINCT string,
//!       so a copy-paste dispatch arm (two variants -> same string) also fails.
//!       SsbActionType's round-trip is already locked by the sibling
//!       `action_type_lock.rs` (ALL_VARIANTS array + exhaustive_match_guard +
//!       golden); duplicating its 482-element array here would be redundant, so
//!       this file covers SsbActionType only with the (B) source-contract grep.
//!
//!   (B) SOURCE-CONTRACT / no-wildcard grep. Each canonical dispatch `match`
//!       block is read back as source text at test time (via std::fs, path
//!       relative to CARGO_MANIFEST_DIR) and asserted to contain NO wildcard arm
//!       (`_ =>`). A future wildcard would let a missing dispatch arm compile
//!       silently and mask an un-mapped variant; this grep makes that a FAIL.
//!       Each grep is anchored on the CANONICAL dispatch site. In particular
//!       RouteDecisionEventKind has a TEST-LOCAL match at ssb.rs ~29106 that DOES
//!       carry a wildcard (the build_test_payload arm); the anchor here is
//!       `impl RouteDecisionEventKind {`, never that test-local match. Likewise
//!       SsbActionType::as_sql_str is anchored on its doc-comment, never the
//!       build_test_payload `_ =>` arm at ssb.rs ~29422.
//!
//! Fail-closed: a missing source file or a stale anchor PANICS (hard FAIL),
//! never a silent skip (wiring-enforcement R1/R2: missing evidence = FAIL).
//!
//! stdlib-only; no strum, no new deps. Test code -> .unwrap()/.expect()/panic!
//! are permitted here.

use std::fs;
use std::path::PathBuf;

use elai_orchestrator::fsm::FsmSubstate;
use elai_orchestrator::ssb::{RouteDecisionEventKind, SkillEventPhase};

// ---------------------------------------------------------------------------
// Explicit variant arrays (the failing list IS the regression signal: a new
// variant must be appended here AND to the match-guard below, or the count
// assert / compile fails).
// ---------------------------------------------------------------------------

/// Every `FsmSubstate` variant in declaration order (fsm.rs ~:125).
/// Adding a variant requires appending it here AND adding an arm to
/// `fsm_substate_match_guard` (compile error) AND to the `as_str` dispatch.
const ALL_FSM_SUBSTATES: &[FsmSubstate] = &[
    FsmSubstate::Framing,
    FsmSubstate::SleepCompute,
    FsmSubstate::Tdd,
    FsmSubstate::DarwinianGenerationEvaluating,
    FsmSubstate::Adversary,
    FsmSubstate::DispatchParallelBatch,
    FsmSubstate::DispatchExecuting,
    FsmSubstate::DispatchAuditing,
    FsmSubstate::DispatchMemorizing,
];

/// Every `SkillEventPhase` variant in declaration order (ssb.rs ~:21235).
const ALL_SKILL_EVENT_PHASES: &[SkillEventPhase] = &[
    SkillEventPhase::Loaded,
    SkillEventPhase::Invoked,
    SkillEventPhase::Failed,
];

/// Every `RouteDecisionEventKind` variant in declaration order (ssb.rs ~:24375).
const ALL_ROUTE_DECISION_KINDS: &[RouteDecisionEventKind] = &[
    RouteDecisionEventKind::CloudRouteChosen,
    RouteDecisionEventKind::TierModelSwapped,
    RouteDecisionEventKind::LocalRouteKept,
];

// ---------------------------------------------------------------------------
// Exhaustive-match guards. Every arm returns (); rustc requires exhaustivity.
// Adding a variant to the enum without an arm here is a HARD COMPILE ERROR that
// points back to this file. NO wildcard arm -- that is the whole point.
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn fsm_substate_match_guard(v: FsmSubstate) {
    match v {
        FsmSubstate::Framing => (),
        FsmSubstate::SleepCompute => (),
        FsmSubstate::Tdd => (),
        FsmSubstate::DarwinianGenerationEvaluating => (),
        FsmSubstate::Adversary => (),
        FsmSubstate::DispatchParallelBatch => (),
        FsmSubstate::DispatchExecuting => (),
        FsmSubstate::DispatchAuditing => (),
        FsmSubstate::DispatchMemorizing => (),
    }
}

#[allow(dead_code)]
fn skill_event_phase_match_guard(v: SkillEventPhase) {
    match v {
        SkillEventPhase::Loaded => (),
        SkillEventPhase::Invoked => (),
        SkillEventPhase::Failed => (),
    }
}

#[allow(dead_code)]
fn route_decision_kind_match_guard(v: RouteDecisionEventKind) {
    match v {
        RouteDecisionEventKind::CloudRouteChosen => (),
        RouteDecisionEventKind::TierModelSwapped => (),
        RouteDecisionEventKind::LocalRouteKept => (),
    }
}

// ---------------------------------------------------------------------------
// (A) ROUND-TRIP tests: every variant -> dispatch fn -> non-empty + distinct.
// ---------------------------------------------------------------------------

/// Drive every variant in `variants` through `dispatch`, assert each maps to a
/// non-empty string, and assert the mapping is injective (no two variants share
/// a wire label). `count_floor` is the count locked by the explicit array; a
/// drift between it and the array length is a separate hard FAIL so a removed
/// array row is also caught.
fn assert_roundtrip<T: Copy>(
    enum_name: &str,
    variants: &[T],
    count_floor: usize,
    dispatch: impl Fn(T) -> &'static str,
) {
    assert_eq!(
        variants.len(),
        count_floor,
        "{enum_name}: explicit variant array length ({}) drifted from the locked \
         count ({count_floor}). A variant was added/removed without updating both \
         the ALL_* array and the locked count in exhaustiveness_dispatch_lock.rs.",
        variants.len(),
    );

    let mut seen: Vec<&'static str> = Vec::with_capacity(variants.len());
    for &v in variants {
        let label = dispatch(v);
        assert!(
            !label.is_empty(),
            "{enum_name}: a variant dispatched to an EMPTY string -- dispatch fn \
             returned \"\" for one variant; every variant needs a real wire label.",
        );
        assert!(
            !seen.contains(&label),
            "{enum_name}: dispatch produced a DUPLICATE wire label '{label}' -- two \
             variants map to the same string (copy-paste dispatch arm?). The label \
             set must be injective.",
        );
        seen.push(label);
    }

    assert_eq!(
        seen.len(),
        count_floor,
        "{enum_name}: distinct-label count ({}) != locked variant count \
         ({count_floor}).",
        seen.len(),
    );
}

#[test]
fn fsm_substate_roundtrip_is_locked() {
    // 5 variants per ADR-9 cross_file_contract ("FsmSubstate 5").
    assert_roundtrip("FsmSubstate", ALL_FSM_SUBSTATES, 5, FsmSubstate::as_str);

    // Spot-check the canonical labels so a rename of an existing arm (which the
    // injective check above would NOT catch) is surfaced too.
    assert_eq!(FsmSubstate::Framing.as_str(), "framing");
    assert_eq!(FsmSubstate::SleepCompute.as_str(), "sleep_compute");
    assert_eq!(FsmSubstate::Tdd.as_str(), "tdd");
    assert_eq!(
        FsmSubstate::DarwinianGenerationEvaluating.as_str(),
        "darwinian_generation_evaluating"
    );
    assert_eq!(FsmSubstate::Adversary.as_str(), "adversary");
}

#[test]
fn skill_event_phase_roundtrip_is_locked() {
    // 3 variants per ADR-9 cross_file_contract ("SkillEventPhase 3").
    assert_roundtrip(
        "SkillEventPhase",
        ALL_SKILL_EVENT_PHASES,
        3,
        |v| v.as_str(),
    );

    assert_eq!(SkillEventPhase::Loaded.as_str(), "loaded");
    assert_eq!(SkillEventPhase::Invoked.as_str(), "invoked");
    assert_eq!(SkillEventPhase::Failed.as_str(), "failed");
}

#[test]
fn route_decision_kind_roundtrip_is_locked() {
    // 3 variants per ADR-9 cross_file_contract ("RouteDecisionEventKind 3").
    assert_roundtrip(
        "RouteDecisionEventKind",
        ALL_ROUTE_DECISION_KINDS,
        3,
        |v| v.as_str(),
    );

    assert_eq!(
        RouteDecisionEventKind::CloudRouteChosen.as_str(),
        "cloud_route_chosen"
    );
    assert_eq!(
        RouteDecisionEventKind::TierModelSwapped.as_str(),
        "tier_model_swapped"
    );
    assert_eq!(
        RouteDecisionEventKind::LocalRouteKept.as_str(),
        "local_route_kept"
    );
}

// ---------------------------------------------------------------------------
// (B) SOURCE-CONTRACT tests: the canonical dispatch match block has NO wildcard.
// ---------------------------------------------------------------------------

/// Resolve a path under the orchestrator crate's `src/` at test time.
fn src_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join(rel)
}

/// Read `rel_src` (under src/), find `anchor` (the line that introduces the
/// canonical dispatch impl/fn), then capture the FIRST `match` block after the
/// anchor by tracking brace depth from the `match`'s opening `{` to its
/// matching `}`. Returns the source text of just that match block (arms only).
///
/// Fail-closed: a missing file, a missing anchor, or a missing `match`/brace
/// PANICS -- these are stale-anchor / moved-code signals, not skips.
fn extract_match_block(rel_src: &str, anchor: &str, enum_name: &str) -> String {
    let path = src_path(rel_src);
    let text = fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{enum_name}: cannot read canonical dispatch source {}: {e}. The \
             exhaustiveness lock cannot verify a file it cannot read (fail-closed).",
            path.display(),
        )
    });

    let anchor_idx = text.find(anchor).unwrap_or_else(|| {
        panic!(
            "{enum_name}: anchor {anchor:?} not found in {}. The dispatch site \
             moved or was renamed -- update the anchor in \
             exhaustiveness_dispatch_lock.rs (fail-closed, stale anchor).",
            path.display(),
        )
    });

    let after_anchor = &text[anchor_idx..];
    let match_rel = after_anchor.find("match ").unwrap_or_else(|| {
        panic!(
            "{enum_name}: no `match` found after anchor {anchor:?} in {}. The \
             canonical dispatch is no longer a `match` -- review by hand.",
            path.display(),
        )
    });

    // From the `match` keyword, find its opening brace then walk to the matching
    // close brace by depth. The first `{` after `match ` opens the arm block.
    let from_match = &after_anchor[match_rel..];
    let open_rel = from_match.find('{').unwrap_or_else(|| {
        panic!(
            "{enum_name}: `match` after anchor {anchor:?} in {} has no opening \
             brace -- malformed source.",
            path.display(),
        )
    });

    let bytes = from_match.as_bytes();
    let mut depth: i32 = 0;
    let mut close_idx: Option<usize> = None;
    for (i, &b) in bytes.iter().enumerate().skip(open_rel) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    close_idx = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close_idx.unwrap_or_else(|| {
        panic!(
            "{enum_name}: unbalanced braces in the `match` after anchor \
             {anchor:?} in {} -- could not find the closing brace.",
            path.display(),
        )
    });

    // Return the arm body (between the opening and closing braces).
    from_match[open_rel + 1..close].to_string()
}

/// Assert the canonical dispatch match block for `enum_name` contains no
/// wildcard arm. A wildcard (`_ =>`) would let a missing dispatch arm compile
/// silently and mask an un-mapped variant -- which is exactly what this lock
/// exists to prevent.
fn assert_no_wildcard_arm(rel_src: &str, anchor: &str, enum_name: &str) {
    let block = extract_match_block(rel_src, anchor, enum_name);
    assert!(
        !block.contains("_ =>"),
        "{enum_name}: the CANONICAL dispatch match block (anchored on {anchor:?} \
         in src/{rel_src}) contains a wildcard arm (`_ =>`). A wildcard masks a \
         missing dispatch arm for a future variant, defeating the exhaustiveness \
         lock. Remove the wildcard and add an explicit arm per variant.\n\
         --- captured block ---\n{block}",
    );
}

#[test]
fn fsm_substate_dispatch_has_no_wildcard() {
    // Canonical: impl FsmSubstate { fn as_str ... } at fsm.rs ~:175-187.
    assert_no_wildcard_arm("fsm.rs", "impl FsmSubstate {", "FsmSubstate");
}

#[test]
fn skill_event_phase_dispatch_has_no_wildcard() {
    // Canonical: impl SkillEventPhase { fn as_str ... } at ssb.rs ~:21244.
    assert_no_wildcard_arm("ssb.rs", "impl SkillEventPhase {", "SkillEventPhase");
}

#[test]
fn route_decision_kind_dispatch_has_no_wildcard() {
    // Canonical: impl RouteDecisionEventKind { fn as_str ... } at ssb.rs ~:24393.
    // NOT the test-local build_test_payload match at ssb.rs ~:29106 (which DOES
    // carry a wildcard by design); the anchor below targets the impl block only.
    assert_no_wildcard_arm(
        "ssb.rs",
        "impl RouteDecisionEventKind {",
        "RouteDecisionEventKind",
    );
}

#[test]
fn ssb_action_type_sql_dispatch_has_no_wildcard() {
    // Canonical: impl SsbActionType { fn as_sql_str ... } at ssb.rs ~:8366-9131.
    // Anchored on the unique doc-comment that immediately precedes as_sql_str so
    // we capture the as_sql_str match (and NOT the build_test_payload match with
    // its `_ => SsbPayload::Finding(...)` wildcard arm at ssb.rs ~:29422).
    assert_no_wildcard_arm(
        "ssb.rs",
        "/// SQL-compatible string for the CHECK constraint.",
        "SsbActionType::as_sql_str",
    );
}
