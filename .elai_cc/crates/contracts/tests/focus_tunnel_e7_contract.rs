//! B-28 E-7 — SpeculativeFileRouter library-tier contract.
//!
//! Pins the public surface of [`elai_orchestrator::focus_tunnel::file_router`]:
//!   * `SpeculativeFileRouter::new(max_micro_steps)` constructs with a counter.
//!   * `route(target_file, content, edit_region, model_output, language)`
//!     returns a `RouteDecision` discriminating Proceed / NeedsMicroStep /
//!     MicroStepCapExhausted / ParseSkipped.
//!   * False-negative bias: unsupported language or parse error → ParseSkipped
//!     (orchestrator treats as Proceed equivalent).
//!   * Cap-of-2 enforcement: after `record_micro_step_completion()` twice,
//!     a third undefined-symbol detection returns MicroStepCapExhausted.
//!   * Counter is per-instance: two routers operate independently (no global).
//!   * `record_micro_step_completion()` only increments; it does not reset
//!     the counter (cap is per-baby-step lifetime — caller constructs a fresh
//!     router for each baby step to reset).
//!   * Defensive bail: when the heuristic path equals target_file (e.g. the
//!     model self-references the file being written), router returns
//!     ParseSkipped rather than NeedsMicroStep on the same file.
//!   * Symbol detection ignores Rust/Python keywords + primitive type names +
//!     self/Self/cls + std prefix markers.
//!   * `file_router_stats_for_payloads` returns zero stats today (locked API
//!     for the future batched SSB bump; mirrors E-5a / E-6 precedent).
//!
//! Spec: workflow/papers/focus-tunnel-design-interface.md L53-58, L441-490.
//! Locked decisions: workflow/sessions/s-27436/context-package-E7.md §12-§16.

use std::path::{Path, PathBuf};

use elai_orchestrator::focus_tunnel::file_router::{
    file_router_stats_for_payloads, FileRouterStats, MicroStep, MicroStepType, RouteDecision,
    SpeculativeFileRouter,
};
use elai_orchestrator::focus_tunnel::EditRegion;
use elai_orchestrator::ssb::{MicroStepCreatedPayload, SsbActionType, SsbPayload};
use elai_plugins::import_extractor::ImportLanguage;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const DEFAULT_CAP: usize = 2;

fn rust_target_path() -> PathBuf {
    PathBuf::from("crates/foo/src/bar.rs")
}

fn python_target_path() -> PathBuf {
    PathBuf::from("python/foo/bar.py")
}

fn fake_region() -> EditRegion {
    EditRegion {
        start_line: 1,
        end_line: 1,
    }
}

// ---------------------------------------------------------------------------
// 1. Rust: undefined function reference → NeedsMicroStep
// ---------------------------------------------------------------------------

#[test]
fn rust_undefined_function_emits_micro_step() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    let target = rust_target_path();
    let target_content = "pub fn defined_one() { let _ = 1; }\n";
    let model_output = "pub fn caller() { process_foo(42); }\n";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Rust,
    );
    match decision {
        RouteDecision::NeedsMicroStep(MicroStep {
            path,
            type_hint,
            reason,
        }) => {
            // Path heuristic: parent_dir(target) + snake_case(name) + .rs
            assert_eq!(
                path,
                PathBuf::from("crates/foo/src/process_foo.rs"),
                "path heuristic should produce parent_dir/snake_case(name).rs",
            );
            assert!(
                matches!(type_hint, MicroStepType::Function | MicroStepType::Module),
                "snake_case identifier in call position → Function (Module also accepted as conservative)",
            );
            assert!(
                reason.contains("process_foo"),
                "reason must name the undefined symbol; got: {reason}",
            );
        }
        other => panic!("expected NeedsMicroStep, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 2. Rust: all symbols defined → Proceed
// ---------------------------------------------------------------------------

#[test]
fn rust_all_defined_returns_proceed() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    let target = rust_target_path();
    let target_content = "pub fn helper() {}\npub struct Widget;\n";
    let model_output = "pub fn caller() { helper(); let _w: Widget; }\n";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Rust,
    );
    match decision {
        RouteDecision::Proceed { written_path } => {
            assert_eq!(written_path, target);
        }
        other => panic!("expected Proceed, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 3. Rust: malformed model output → ParseSkipped
// ---------------------------------------------------------------------------

#[test]
fn rust_malformed_model_output_returns_parse_skipped() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    let target = rust_target_path();
    let target_content = "pub fn helper() {}\n";
    // Severely malformed: unterminated brace + raw garbage tokens.
    let model_output = "pub fn ((( !! ~~~ %%%";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Rust,
    );
    // ParseSkipped is the locked false-negative-bias return value here.
    // The router does NOT spuriously emit NeedsMicroStep on parse failure.
    assert!(
        matches!(
            decision,
            RouteDecision::ParseSkipped { .. } | RouteDecision::Proceed { .. }
        ),
        "malformed model output → ParseSkipped (preferred) or Proceed (acceptable false-negative); got {decision:?}",
    );
    // If ParseSkipped, the reason must be a non-empty diagnostic string.
    if let RouteDecision::ParseSkipped { reason } = &decision {
        assert!(!reason.is_empty(), "ParseSkipped reason must be non-empty");
    }
}

// ---------------------------------------------------------------------------
// 4. Python: undefined module reference → NeedsMicroStep with .py
// ---------------------------------------------------------------------------

#[test]
fn python_undefined_helper_emits_micro_step() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    let target = python_target_path();
    let target_content = "def existing():\n    return 1\n";
    let model_output = "def caller():\n    return helper_module()\n";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Python,
    );
    match decision {
        RouteDecision::NeedsMicroStep(MicroStep { path, reason, .. }) => {
            assert_eq!(
                path,
                PathBuf::from("python/foo/helper_module.py"),
                "Python path heuristic: parent_dir/<name>.py (case-preserving)",
            );
            assert!(
                reason.contains("helper_module"),
                "reason names symbol; got: {reason}",
            );
        }
        other => panic!("expected NeedsMicroStep, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 5. Unsupported language (Java) → ParseSkipped (false-negative bias)
// ---------------------------------------------------------------------------

#[test]
fn unsupported_language_returns_parse_skipped() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    let target = PathBuf::from("java/src/Foo.java");
    let target_content = "class Foo { void bar() {} }";
    let model_output = "class Foo { void bar() { unknown_thing(); } }";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Java,
    );
    match decision {
        RouteDecision::ParseSkipped { reason } => {
            assert!(
                reason.contains("language") || reason.contains("unsupported"),
                "ParseSkipped reason must mention the language; got: {reason}",
            );
        }
        other => panic!("expected ParseSkipped for unsupported language, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 6. Cap-of-2 ordering: emit, record, emit, record, exhausted
// ---------------------------------------------------------------------------

#[test]
fn cap_of_two_after_two_completions_returns_exhausted() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    let target = rust_target_path();
    let target_content = "pub fn defined() {}\n";

    // First call: undefined `process_a` → NeedsMicroStep
    let first = router.route(
        &target,
        target_content,
        Some(fake_region()),
        "fn caller() { process_a(); }\n",
        ImportLanguage::Rust,
    );
    assert!(
        matches!(first, RouteDecision::NeedsMicroStep(_)),
        "first call must emit NeedsMicroStep; got {first:?}",
    );
    router.record_micro_step_completion();
    assert_eq!(router.micro_steps_consumed(), 1);

    // Second call: undefined `process_b` → NeedsMicroStep
    let second = router.route(
        &target,
        target_content,
        Some(fake_region()),
        "fn caller() { process_b(); }\n",
        ImportLanguage::Rust,
    );
    assert!(
        matches!(second, RouteDecision::NeedsMicroStep(_)),
        "second call must emit NeedsMicroStep; got {second:?}",
    );
    router.record_micro_step_completion();
    assert_eq!(router.micro_steps_consumed(), 2);

    // Third call: undefined `process_c` → MicroStepCapExhausted
    let third = router.route(
        &target,
        target_content,
        Some(fake_region()),
        "fn caller() { process_c(); }\n",
        ImportLanguage::Rust,
    );
    match third {
        RouteDecision::MicroStepCapExhausted {
            count,
            last_undefined,
        } => {
            assert_eq!(count, 2, "count must reflect completions; got {count}");
            assert_eq!(
                last_undefined, "process_c",
                "last_undefined must name the symbol that tripped the cap",
            );
        }
        other => panic!("expected MicroStepCapExhausted, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 7. Same-file path collision → ParseSkipped (defensive bail)
// ---------------------------------------------------------------------------

#[test]
fn same_file_path_collision_returns_parse_skipped() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    // The undefined symbol's snake_case form will be `bar`. parent_dir is
    // `crates/foo/src`. Heuristic path becomes `crates/foo/src/bar.rs` —
    // which IS the target_file. Defensive bail: ParseSkipped.
    let target = PathBuf::from("crates/foo/src/bar.rs");
    let target_content = "pub fn defined() {}\n";
    let model_output = "fn caller() { bar(); }\n";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Rust,
    );
    match decision {
        RouteDecision::ParseSkipped { reason } => {
            assert!(
                reason.contains("self") || reason.contains("same") || reason.contains("collision"),
                "reason must mention the collision; got: {reason}",
            );
        }
        other => panic!("expected ParseSkipped (defensive bail), got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 8. record_micro_step_completion() does NOT reset the counter
// ---------------------------------------------------------------------------

#[test]
fn record_completion_only_increments_never_resets() {
    let mut router = SpeculativeFileRouter::new(5);
    assert_eq!(router.micro_steps_consumed(), 0);
    router.record_micro_step_completion();
    assert_eq!(router.micro_steps_consumed(), 1);
    router.record_micro_step_completion();
    assert_eq!(router.micro_steps_consumed(), 2);
    router.record_micro_step_completion();
    assert_eq!(router.micro_steps_consumed(), 3);
    // Counter never drops; remaining must decrement monotonically.
    assert_eq!(router.micro_steps_remaining(), 2);
}

// ---------------------------------------------------------------------------
// 9. file_router_stats_for_payloads returns zero stats today
// ---------------------------------------------------------------------------

#[test]
fn file_router_stats_zero_today() {
    // Empty slice path.
    let empty: Vec<elai_orchestrator::ssb::SsbPayload> = Vec::new();
    let stats: FileRouterStats = file_router_stats_for_payloads(&empty);
    assert_eq!(stats, FileRouterStats::default());
    assert_eq!(stats.total_micro_steps_created, 0);
    assert_eq!(stats.total_cap_exhausted, 0);
}

// ---------------------------------------------------------------------------
// 10. Symbol detection ignores keywords / primitives / self
// ---------------------------------------------------------------------------

#[test]
fn ignores_rust_keywords_and_primitives() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    let target = rust_target_path();
    let target_content = "pub fn helper() {}\n";
    // All identifiers referenced here are keywords (`let`, `mut`, `pub`),
    // primitives (`i32`, `String`, `bool`, `usize`), or `self`. None should
    // be flagged as undefined.
    let model_output = "pub fn caller() { let mut x: i32 = 0; let s: String = String::new(); let b: bool = false; let n: usize = 0; helper(); }\n";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Rust,
    );
    assert!(
        matches!(decision, RouteDecision::Proceed { .. }),
        "keyword/primitive-only references must not emit NeedsMicroStep; got {decision:?}",
    );
}

#[test]
fn ignores_python_keywords_and_primitives() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    let target = python_target_path();
    let target_content = "def existing():\n    return 1\n";
    // Only Python keywords / builtins / `self`. existing() is defined.
    let model_output = "def caller(self):\n    x: int = 0\n    y: str = 'hi'\n    z: bool = True\n    return existing()\n";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Python,
    );
    assert!(
        matches!(decision, RouteDecision::Proceed { .. }),
        "keyword/primitive-only references must not emit NeedsMicroStep; got {decision:?}",
    );
}

// ---------------------------------------------------------------------------
// 11. Two routers maintain independent counters (no global state)
// ---------------------------------------------------------------------------

#[test]
fn distinct_routers_have_independent_counters() {
    let mut router_a = SpeculativeFileRouter::new(DEFAULT_CAP);
    let mut router_b = SpeculativeFileRouter::new(DEFAULT_CAP);
    router_a.record_micro_step_completion();
    router_a.record_micro_step_completion();
    assert_eq!(router_a.micro_steps_consumed(), 2);
    assert_eq!(router_a.micro_steps_remaining(), 0);
    // router_b must be unaffected.
    assert_eq!(router_b.micro_steps_consumed(), 0);
    assert_eq!(router_b.micro_steps_remaining(), DEFAULT_CAP);
    // And router_b can still emit micro-steps after router_a is full.
    let target = rust_target_path();
    let decision = router_b.route(
        &target,
        "pub fn helper() {}\n",
        Some(fake_region()),
        "fn caller() { undefined_fn(); }\n",
        ImportLanguage::Rust,
    );
    assert!(
        matches!(decision, RouteDecision::NeedsMicroStep(_)),
        "router_b must be unaffected by router_a's saturation; got {decision:?}",
    );
}

// ---------------------------------------------------------------------------
// 12. Determinism: same inputs yield the same RouteDecision
// ---------------------------------------------------------------------------

#[test]
fn route_is_deterministic_on_identical_inputs() {
    let target: &Path = Path::new("crates/foo/src/bar.rs");
    let target_content = "pub fn defined() {}\n";
    let model_output = "fn caller() { undefined_helper(); }\n";

    let mut router_a = SpeculativeFileRouter::new(DEFAULT_CAP);
    let mut router_b = SpeculativeFileRouter::new(DEFAULT_CAP);
    let dec_a = router_a.route(
        target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Rust,
    );
    let dec_b = router_b.route(
        target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Rust,
    );
    assert_eq!(
        dec_a, dec_b,
        "identical inputs across two routers must yield identical decisions",
    );
}

// ---------------------------------------------------------------------------
// 13. Rust `use foo::Bar;` makes `Bar` defined (no spurious micro-step).
// ---------------------------------------------------------------------------

#[test]
fn test_use_simple_import_not_undefined() {
    // SPEC_DRIFT fix: a `use foo::Bar;` statement in target_file MUST add
    // `Bar` to the defined-set so model_output referencing `Bar` does NOT
    // trigger a spurious NeedsMicroStep and burn the cap-of-2 budget.
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    let target = rust_target_path();
    let target_content = "use foo::Bar;\nfn x() {}\n";
    let model_output = "fn caller() { let _b: Bar = make(); }\n";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Rust,
    );
    // The first undefined SHOULD now be `make` (not `Bar`). If routes still
    // emits NeedsMicroStep, it must target `make`, not `Bar` — otherwise the
    // fix has not landed.
    match decision {
        RouteDecision::Proceed { .. } => {
            // Acceptable — both Bar and make may be classified as defined
            // depending on the over-collection of the walker.
        }
        RouteDecision::NeedsMicroStep(MicroStep { reason, .. }) => {
            assert!(
                !reason.contains("`Bar`"),
                "use-imported `Bar` must NOT be flagged undefined; got: {reason}",
            );
        }
        other => panic!(
            "expected Proceed or NeedsMicroStep(make), got {other:?} (Bar should be defined via use_declaration)",
        ),
    }
}

// ---------------------------------------------------------------------------
// 14. Rust `use foo::{Bar, Baz};` makes both names defined.
// ---------------------------------------------------------------------------

#[test]
fn test_use_list_import_not_undefined() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    let target = rust_target_path();
    let target_content = "use foo::{Bar, Baz};\nfn x() {}\n";
    let model_output = "fn caller() { let _b: Bar; let _z: Baz; }\n";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Rust,
    );
    match decision {
        RouteDecision::Proceed { .. } => {}
        RouteDecision::NeedsMicroStep(MicroStep { reason, .. }) => {
            panic!(
                "use-list imports `Bar`/`Baz` must both be defined; got micro-step reason: {reason}",
            );
        }
        other => panic!("expected Proceed, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 15. Rust `use foo::Bar as Baz;` — only `Baz` is the local binding.
// ---------------------------------------------------------------------------

#[test]
fn test_use_as_clause_alias_is_local_binding() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    let target = rust_target_path();
    let target_content = "use foo::Bar as Baz;\nfn x() {}\n";
    // Reference the alias — must NOT be flagged undefined.
    let model_output = "fn caller() { let _b: Baz; }\n";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Rust,
    );
    match decision {
        RouteDecision::Proceed { .. } => {}
        RouteDecision::NeedsMicroStep(MicroStep { reason, .. }) => {
            assert!(
                !reason.contains("`Baz`"),
                "alias `Baz` must be defined; got: {reason}",
            );
        }
        other => panic!("expected Proceed (Baz is the alias binding), got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 16. Rust nested use list — `use foo::{Bar, baz::{Qux}};` defines Bar and Qux.
// ---------------------------------------------------------------------------

#[test]
fn test_use_nested_list_import_not_undefined() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    let target = rust_target_path();
    let target_content = "use foo::{Bar, baz::{Qux}};\nfn x() {}\n";
    let model_output = "fn caller() { let _b: Bar; let _q: Qux; }\n";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Rust,
    );
    match decision {
        RouteDecision::Proceed { .. } => {}
        RouteDecision::NeedsMicroStep(MicroStep { reason, .. }) => {
            panic!(
                "nested use list must define both `Bar` and `Qux`; got micro-step reason: {reason}",
            );
        }
        other => panic!("expected Proceed, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 17. ITEM 3 — Self-collision detection works across path separators.
// ---------------------------------------------------------------------------
//
// The defensive bail at L266-273 of file_router.rs compares the suggested
// path against the target_file via PathBuf equality. PathBuf::from preserves
// the separator the caller used; on Windows the separator is '\\' and on
// POSIX it is '/'. Both forms must trip the self-collision guard. We build
// the target_file using std::path::PathBuf::from with the platform-native
// separator via Path components, so the test is identical cross-platform.
//
// Cross-platform note: writing literal `\\` in the test would only exercise
// the Windows behaviour; PathBuf canonicalises components, so we let the
// stdlib choose the separator and assert the ParseSkipped fires regardless.

#[test]
fn test_self_collision_detected_across_separators() {
    let mut router = SpeculativeFileRouter::new(DEFAULT_CAP);
    // Build target using OS-native components — PathBuf::from on Windows uses
    // '\\', on POSIX uses '/'. The undefined symbol's snake_case form is
    // `bar`, parent_dir is `crates/foo/src`, so the heuristic path collides
    // with target. Defensive bail: ParseSkipped.
    let target: PathBuf = ["crates", "foo", "src", "bar.rs"].iter().collect();
    let target_content = "pub fn defined() {}\n";
    let model_output = "fn caller() { bar(); }\n";
    let decision = router.route(
        &target,
        target_content,
        Some(fake_region()),
        model_output,
        ImportLanguage::Rust,
    );
    match decision {
        RouteDecision::ParseSkipped { reason } => {
            assert!(
                reason.contains("self") || reason.contains("same") || reason.contains("collision"),
                "reason must mention the collision; got: {reason}",
            );
        }
        other => panic!(
            "expected ParseSkipped (self-collision via OS-native separator), got {other:?}",
        ),
    }
}

// ---------------------------------------------------------------------------
// B-28 WIRING-1 — MicroStepCreated SSB round-trip + aggregator wired
// ---------------------------------------------------------------------------

/// `SsbActionType::MicroStepCreated` and its payload must round-trip via
/// msgpack and report the matching action_type.
#[test]
fn micro_step_created_ssb_round_trip() {
    let payload = SsbPayload::MicroStepCreated(MicroStepCreatedPayload {
        parent_step_id: 42,
        created_path: "crates/foo/src/bar.rs".into(),
        type_hint: "Module".into(),
        reason: "undefined_symbol".into(),
        count: 1,
    });

    assert_eq!(payload.action_type(), SsbActionType::MicroStepCreated);
    assert_eq!(payload.action_type().as_sql_str(), "micro_step_created");

    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, payload, "msgpack round-trip must preserve payload");
}

/// `file_router_stats_for_payloads` aggregates real `MicroStepCreated`
/// rows by type_hint + total + cap_exhausted (the cap_exhausted flag is
/// encoded as `reason == "cap_exhausted"`).
#[test]
fn file_router_stats_aggregates_real_rows() {
    let rows: Vec<SsbPayload> = vec![
        SsbPayload::MicroStepCreated(MicroStepCreatedPayload {
            parent_step_id: 1,
            created_path: "crates/a/src/lib.rs".into(),
            type_hint: "Module".into(),
            reason: "undefined_symbol".into(),
            count: 1,
        }),
        SsbPayload::MicroStepCreated(MicroStepCreatedPayload {
            parent_step_id: 1,
            created_path: "crates/a/src/utils.rs".into(),
            type_hint: "Function".into(),
            reason: "undefined_symbol".into(),
            count: 1,
        }),
        SsbPayload::MicroStepCreated(MicroStepCreatedPayload {
            parent_step_id: 2,
            created_path: "crates/a/src/types.rs".into(),
            type_hint: "Struct".into(),
            reason: "cap_exhausted".into(),
            count: 0,
        }),
    ];
    let stats: FileRouterStats = file_router_stats_for_payloads(&rows);
    assert_eq!(stats.total_micro_steps_created, 2);
    assert_eq!(stats.total_cap_exhausted, 1);
    assert_eq!(stats.per_type_hint.get("Module").copied().unwrap_or(0), 1);
    assert_eq!(stats.per_type_hint.get("Function").copied().unwrap_or(0), 1);
}
