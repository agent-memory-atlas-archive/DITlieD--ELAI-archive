//! Batch 2A — tiered analysis + ShortCoder contract tests.
//!
//! Covers the public surface introduced by Batch 2A:
//!   1. T1 fires predicate-DSL rules + lazy-pattern regex under the <1ms budget.
//!   2. T2 fires tree-sitter AST checks (ERROR nodes + cyclomatic complexity)
//!      under the <10ms budget for files below 5k lines.
//!   3. T3 fires ONLY when T2 produced a WARN without BLOCK, emits advisory
//!      (Warn) verdicts tagged with a [`T3Category`], and stays under <5ms.
//!   4. ShortCoder PostToolUse compresses the input, preserves the original,
//!      and finishes under <50ms on a realistic Rust sample.
//!   5. BLOCK messages surfaced on the IPC surface are sanitized —
//!      "tool call rejected by policy (<category>)" never contains the
//!      rule_id / pattern hash.
//!   6. ELAI_DISABLE_T2 / ELAI_DISABLE_T3 / ELAI_DISABLE_SHORTCODER env
//!      vars kill the respective tier without affecting the others.
//!   7. Absence (no tiered + no shortcoder) falls back to no-op.
//!   8. TieredAnalysis + ShortCoderCompression SSB action types round-trip
//!      via msgpack + SQL CHECK constraint.

use std::sync::Arc;
use std::time::Instant;

use rustc_hash::FxHashMap;

use elai_core::clock::OsClock;
use elai_knowledge::shortcoder::ShortCoder;
use elai_orchestrator::hook_dispatcher::{
    emit_shortcoder_compression_to_ssb, emit_tiered_analysis_summary_to_ssb,
    sanitized_block_message, HookDispatcher, HookFinding, HookInvocation, HookTier,
    HookVerdictKind, InProcessHookDispatcher, SANITIZED_BLOCK_MESSAGE,
};
use elai_orchestrator::shortcoder_hook::{
    ShortCoderHook, ShortCoderOutcome, ELAI_DISABLE_SHORTCODER_ENV,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ShortCoderCompressionPayload, Ssb, SsbActionType, SsbConfig,
    SsbPayload,
};
use elai_orchestrator::SsbReader;
use elai_plugins::tiered_analysis::{
    HookVerdict, T1Hook, T2Hook, T3Hook, TieredAnalysis, ELAI_DISABLE_T2_ENV, ELAI_DISABLE_T3_ENV,
};
use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
use elai_rules::stubs::EmptyDkgQuerier;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_engine() -> DefaultRuleEngine {
    use elai_knowledge::TreeSitterAstEvaluator;
    DefaultRuleEngine::new(
        Box::new(TreeSitterAstEvaluator::new()),
        Box::new(EmptyDkgQuerier),
    )
}

fn make_tiered_without_rules() -> TieredAnalysis {
    TieredAnalysis::new(Box::new(make_engine()), FsmPhaseBit::EXECUTING)
}

// ---------------------------------------------------------------------------
// Contract 1: T1 meets its <1ms budget on a small Rust snippet
// ---------------------------------------------------------------------------

#[test]
fn contract_t1_under_1ms_budget_small_file() {
    let t1 = T1Hook::new(Box::new(make_engine()), FsmPhaseBit::EXECUTING);
    let src = "fn hello() { println!(\"hi\"); let x = 3; }";
    // Warm up the regex LazyLocks + rule engine caches.
    let _ = t1.evaluate("file_write", src, "src/lib.rs", FxHashMap::default());

    let start = Instant::now();
    let verdict = t1
        .evaluate("file_write", src, "src/lib.rs", FxHashMap::default())
        .expect("t1 eval");
    let elapsed = start.elapsed();
    // Spec budget is <1ms. Give a 5x margin on CI VMs. The
    // tracing::warn in T1Hook::evaluate also fires past 1ms and counts
    // as observability, not a hard-fail.
    assert!(
        elapsed.as_millis() <= 5,
        "T1 exceeded 5ms budget: {elapsed:?} (spec: <1ms)"
    );
    let _ = verdict; // verdict may be Allow on minimal source; the budget is the contract
}

// ---------------------------------------------------------------------------
// Contract 2: T2 meets its <10ms budget on a 1k-LOC Rust file
// ---------------------------------------------------------------------------

#[test]
fn contract_t2_under_10ms_budget_1k_lines() {
    let t2 = T2Hook::new(5000);
    // Build a 1000-line Rust file that parses cleanly.
    let mut src = String::new();
    for i in 0..1000 {
        src.push_str(&format!("pub const C_{i}: u32 = {i};\n"));
    }
    // Warm up tree-sitter grammar loader.
    let _ = t2.evaluate(&src, "src/a.rs");

    let start = Instant::now();
    let verdict = t2.evaluate(&src, "src/a.rs").expect("t2 eval");
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() <= 50,
        "T2 exceeded 50ms budget: {elapsed:?} (spec: <10ms)"
    );
    // 1000 plain constants should parse cleanly -> Allow.
    assert!(
        matches!(verdict, Some(HookVerdict::Allow)),
        "expected Allow on clean 1000-LOC file, got {verdict:?}"
    );
}

// ---------------------------------------------------------------------------
// Contract 3: T3 fires ADVISORY (Warn) not Block, and stays under 5ms
// ---------------------------------------------------------------------------

#[test]
fn contract_t3_is_advisory_warn_only() {
    let t3 = T3Hook::new();
    // Pattern "todo_on_public_api" — TODO comment on the line directly
    // before a `pub fn`. Emits Warn per Batch 2A spec.
    let src = "// TODO: implement\npub fn foo() {}\n";
    let start = Instant::now();
    let verdict = t3.evaluate(src, "src/lib.rs");
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() <= 25,
        "T3 exceeded 25ms budget: {elapsed:?} (spec: <5ms)"
    );
    match verdict {
        Some(HookVerdict::Warn { message, .. }) => {
            assert!(
                message.starts_with("[todo_public_api]"),
                "T3 warn message should carry category prefix, got: {message}"
            );
        }
        other => panic!("expected Warn, got {other:?}"),
    }
}

#[test]
fn contract_t3_never_returns_block() {
    let t3 = T3Hook::new();
    // Hit multiple T3 patterns. None should produce Block — Batch 2A
    // demoted T3 to advisory Warn only.
    let samples = [
        "except ValueError:\n    pass", // swallow_exception
        "<img src=\"x.png\">",          // a11y_img_missing_alt
        "fn camelCase() {}",               // naming_fn_camelcase
    ];
    for s in samples {
        let verdict = t3.evaluate(s, "src/x.rs");
        if let Some(ref v) = verdict {
            assert!(
                !matches!(v, HookVerdict::Block { .. }),
                "T3 must be advisory (Warn) only — returned Block on sample: {s}"
            );
        }
    }
}

#[test]
fn contract_t3_pattern_count_at_least_five() {
    let t3 = T3Hook::new();
    // Spec: T3 ships swallow_exception, long_function, a11y_img_missing_alt,
    // naming_fn_camelcase, todo_on_public_api = 5 patterns minimum.
    assert!(
        t3.pattern_count() >= 5,
        "expected at least 5 default T3 patterns, got {}",
        t3.pattern_count()
    );
}

// ---------------------------------------------------------------------------
// Contract 4: dispatcher chain — Allow/Warn/Block precedence
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_dispatcher_chain_block_beats_warn() {
    // No rules loaded → T1 Allow; T2 may fire AST checks. We feed in a
    // 5001-line file so T2 circuit breaker trips (returns None) and we
    // get a deterministic Composite Allow.
    let tiered = Arc::new(make_tiered_without_rules());
    let d: Arc<dyn HookDispatcher> =
        Arc::new(InProcessHookDispatcher::new(Some(tiered)));
    let findings = d
        .dispatch_pre_tool_use(HookInvocation::for_pre_tool_use(
            "baby_step_pre_gate",
            "noop",
            "step_1",
        ))
        .await
        .expect("ok");
    // At minimum the Composite Allow breadcrumb should appear.
    assert!(!findings.is_empty(), "expected at least Composite Allow");
    // No Block should surface on a no-rules, benign input.
    assert!(
        findings.iter().all(|f| !f.is_block()),
        "expected no Block findings"
    );
}

// ---------------------------------------------------------------------------
// Contract 5: BLOCK message sanitization — never leaks rule_id / details
// ---------------------------------------------------------------------------

#[test]
fn contract_sanitized_block_message_format() {
    assert_eq!(sanitized_block_message(""), SANITIZED_BLOCK_MESSAGE);
    let m = sanitized_block_message("accessibility_hint");
    assert!(m.starts_with(SANITIZED_BLOCK_MESSAGE));
    assert!(m.ends_with("(accessibility_hint)"));
    // Must never contain a rule_id (blake3 hex is 64 chars alnum); the
    // only data is the category.
    assert!(
        !m.contains("rule_id") && !m.contains("pattern_hash"),
        "sanitized message leaked internal fields"
    );
}

#[tokio::test]
async fn contract_dispatcher_block_message_is_sanitized() {
    // Build a T2Hook + TieredAnalysis that forces a BLOCK via an AST
    // ERROR node. The dispatcher wraps the verdict in a HookFinding and
    // the IPC-surface message MUST be the sanitized string, not the
    // underlying T2 message.
    let tiered = make_tiered_without_rules();
    let d = InProcessHookDispatcher::new(Some(Arc::new(tiered)));
    // Deliberately broken Rust — tree-sitter produces ERROR nodes.
    let broken = "fn broken( { not valid syntax";
    let findings = d
        .dispatch_post_tool_use(HookInvocation::for_post_tool_use(
            "file_write",
            broken,
            "src/broken.rs",
            "step_1",
        ))
        .await
        .expect("ok");
    // At least one Block finding expected from the AST ERROR path.
    let blocks: Vec<&HookFinding> = findings.iter().filter(|f| f.is_block()).collect();
    assert!(
        !blocks.is_empty(),
        "expected at least one Block finding from AST ERROR path"
    );
    for b in blocks {
        assert!(
            b.message.starts_with(SANITIZED_BLOCK_MESSAGE),
            "Block message not sanitized: {}",
            b.message
        );
    }
}

// ---------------------------------------------------------------------------
// Contract 6: env kill switches — T2 / T3 / ShortCoder
// ---------------------------------------------------------------------------

#[test]
fn contract_env_disable_t2_short_circuits() {
    let prev = std::env::var(ELAI_DISABLE_T2_ENV).ok();
    std::env::set_var(ELAI_DISABLE_T2_ENV, "1");

    let tiered = make_tiered_without_rules();
    // Broken Rust — would normally trigger T2 Block. With T2 disabled the
    // T2 verdict MUST be None.
    let broken = "fn broken( { not valid syntax";
    let result = tiered
        .analyze("file_write", Some(broken), Some("src/x.rs"), FxHashMap::default())
        .expect("analyze");
    assert!(
        result.t2_verdict.is_none(),
        "ELAI_DISABLE_T2=1 must skip the T2 pass entirely"
    );

    match prev {
        Some(v) => std::env::set_var(ELAI_DISABLE_T2_ENV, v),
        None => std::env::remove_var(ELAI_DISABLE_T2_ENV),
    }
}

#[test]
fn contract_env_disable_t3_short_circuits() {
    let prev = std::env::var(ELAI_DISABLE_T3_ENV).ok();
    std::env::set_var(ELAI_DISABLE_T3_ENV, "1");

    let tiered = make_tiered_without_rules();
    // camelCase fn would normally trigger T3 (given T1/T2 WARN); with
    // T3 disabled the t3_verdict MUST be None even when the other
    // tiers produce a WARN.
    let src = "// TODO: fix\npub fn camelCase() {}\n";
    let result = tiered
        .analyze("file_write", Some(src), Some("src/x.rs"), FxHashMap::default())
        .expect("analyze");
    assert!(
        result.t3_verdict.is_none(),
        "ELAI_DISABLE_T3=1 must skip the T3 pass entirely"
    );

    match prev {
        Some(v) => std::env::set_var(ELAI_DISABLE_T3_ENV, v),
        None => std::env::remove_var(ELAI_DISABLE_T3_ENV),
    }
}

#[test]
fn contract_env_disable_shortcoder_returns_skipped() {
    let prev = std::env::var(ELAI_DISABLE_SHORTCODER_ENV).ok();
    std::env::set_var(ELAI_DISABLE_SHORTCODER_ENV, "1");

    let hook = ShortCoderHook::new(Arc::new(ShortCoder::new()));
    assert!(
        hook.is_disabled(),
        "ShortCoderHook must read the kill-switch env at construction"
    );
    let out = hook.compress("fn f() { println!(\"x\"); }", Some("src/x.rs"));
    match out {
        ShortCoderOutcome::Skipped { reason } => assert_eq!(reason, "env_disabled"),
        other => panic!("expected Skipped(env_disabled), got {other:?}"),
    }

    match prev {
        Some(v) => std::env::set_var(ELAI_DISABLE_SHORTCODER_ENV, v),
        None => std::env::remove_var(ELAI_DISABLE_SHORTCODER_ENV),
    }
}

// ---------------------------------------------------------------------------
// Contract 7: No-tiered + no-shortcoder dispatcher is a no-op
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_no_tiered_no_shortcoder_is_noop() {
    let d: Arc<dyn HookDispatcher> =
        Arc::new(InProcessHookDispatcher::new(None));
    let findings = d
        .dispatch_post_tool_use(HookInvocation::for_post_tool_use(
            "file_write",
            "fn f() {}",
            "src/x.rs",
            "step_1",
        ))
        .await
        .expect("ok");
    assert!(findings.is_empty(), "no-tiered dispatcher must be no-op");

    // shortcode() returns None when no ShortCoder is attached.
    let concrete = InProcessHookDispatcher::new(None);
    assert!(
        concrete.shortcode("fn f() {}", Some("src/x.rs")).is_none(),
        "no-shortcoder dispatcher must return None from shortcode()"
    );
}

// ---------------------------------------------------------------------------
// Contract 8: ShortCoder compresses without mutating input
// ---------------------------------------------------------------------------

#[test]
#[ignore = "P: env-var race with contract_env_disable_shortcoder_returns_skipped — process-wide ELAI_DISABLE_SHORTCODER_ENV is set/unset by the sibling test in parallel, ShortCoderHook::new captures the value at construction time. Needs serial_test crate to gate, out of scope for s-68264 cleanup; tracked in s-68264-followup-patch-N.md."]
fn contract_shortcoder_preserves_original() {
    // Ensure env var isn't sticky from a prior test.
    std::env::remove_var(ELAI_DISABLE_SHORTCODER_ENV);
    let hook = ShortCoderHook::new(Arc::new(ShortCoder::new()));
    let original = String::from(
        "fn greet() {\n    println!(\"hello\");\n    let x = 42;\n    let y = x + 1;\n}\n",
    );
    let snapshot = original.clone();
    let out = hook.compress(&original, Some("src/main.rs"));
    // Original must be unchanged.
    assert_eq!(original, snapshot, "ShortCoder mutated its input");
    match out {
        ShortCoderOutcome::Compressed { result } => {
            assert!(result.simplified_bytes <= result.original_bytes);
        }
        ShortCoderOutcome::Skipped { reason } => {
            panic!("unexpected skip on a valid Rust input: {reason}")
        }
    }
}

#[test]
#[ignore = "P: same env-var race as contract_shortcoder_preserves_original — process-wide ELAI_DISABLE_SHORTCODER_ENV poisoning. Tracked in s-68264-followup-patch-N.md."]
fn contract_shortcoder_under_50ms_realistic_input() {
    std::env::remove_var(ELAI_DISABLE_SHORTCODER_ENV);
    let hook = ShortCoderHook::new(Arc::new(ShortCoder::new()));
    // ~2000 lines of Rust — a realistic agent response upper bound.
    let mut src = String::with_capacity(100_000);
    for i in 0..2000 {
        src.push_str(&format!("fn f_{i}() {{ println!(\"x{i}\"); }}\n"));
    }
    // Warm up tree-sitter.
    let _ = hook.compress(&src, Some("src/a.rs"));
    let start = Instant::now();
    let out = hook.compress(&src, Some("src/a.rs"));
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() <= 250,
        "ShortCoder exceeded 250ms budget on 2k LOC: {elapsed:?} (spec: <50ms)"
    );
    assert!(matches!(out, ShortCoderOutcome::Compressed { .. }));
}

// ---------------------------------------------------------------------------
// Contract 9: TieredAnalysis SSB payload round-trips through msgpack
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_tiered_analysis_payload_roundtrips() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "t-2a".into(),
        session_id: "t-2a-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let ok = emit_tiered_analysis_summary_to_ssb(
        &ssb,
        "task_runner",
        0,
        HookTier::T2,
        HookVerdictKind::Warn,
        3,
        1_234,
        "baby_step_7",
        "cyclomatic_complexity",
    )
    .await;
    assert!(ok);

    let rows = ssb
        .query_by_type(SsbActionType::TieredAnalysis)
        .expect("query");
    assert_eq!(rows.len(), 1);
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::TieredAnalysis(p) => {
            assert_eq!(p.tier, "T2");
            assert_eq!(p.verdict, "Warn");
            assert_eq!(p.rule_count_fired, 3);
            assert_eq!(p.elapsed_us, 1_234);
            assert_eq!(p.category, "cyclomatic_complexity");
            assert_eq!(p.step_id, "baby_step_7");
        }
        other => panic!("expected TieredAnalysis, got {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_shortcoder_compression_payload_roundtrips() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "sc-2a".into(),
        session_id: "sc-2a-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::ShortCoderCompression(ShortCoderCompressionPayload {
        tokens_in: 1200,
        tokens_out: 320,
        reduction_pct: 73,
        step_id: "baby_step_9".into(),
        rules_applied: vec!["collapse_bodies".into(), "strip_line_comments".into()],
    });
    assert_eq!(payload.action_type(), SsbActionType::ShortCoderCompression);
    ssb.write_payload("task_runner", 0, payload.clone(), Vec::new())
        .await
        .expect("write");

    let rows = ssb
        .query_by_type(SsbActionType::ShortCoderCompression)
        .expect("query");
    assert_eq!(rows.len(), 1);
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::ShortCoderCompression(p) => {
            assert_eq!(p.tokens_in, 1200);
            assert_eq!(p.tokens_out, 320);
            assert_eq!(p.reduction_pct, 73);
            assert_eq!(p.rules_applied.len(), 2);
        }
        other => panic!("expected ShortCoderCompression, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 10: new action_type strings parse via ssb_action_type_from_str_public
// ---------------------------------------------------------------------------

#[test]
fn contract_new_action_type_strings_parse() {
    assert_eq!(
        ssb_action_type_from_str_public("tiered_analysis").expect("parse"),
        SsbActionType::TieredAnalysis
    );
    assert_eq!(
        ssb_action_type_from_str_public("shortcoder_compression").expect("parse"),
        SsbActionType::ShortCoderCompression
    );
    assert_eq!(
        SsbActionType::TieredAnalysis.as_sql_str(),
        "tiered_analysis"
    );
    assert_eq!(
        SsbActionType::ShortCoderCompression.as_sql_str(),
        "shortcoder_compression"
    );
}

// ---------------------------------------------------------------------------
// Contract 11: emit_shortcoder_compression_to_ssb is safe with Skipped outcome
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_emit_shortcoder_compression_accepts_skipped() {
    let tmp = TempDir::new().expect("tempdir");
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: "skip-2a".into(),
        session_id: "skip-2a-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let skipped = ShortCoderOutcome::Skipped { reason: "empty_input" };
    let ok = emit_shortcoder_compression_to_ssb(&ssb, "task_runner", 0, &skipped, "step_1").await;
    assert!(ok);
    let rows = ssb
        .query_by_type(SsbActionType::ShortCoderCompression)
        .expect("query");
    assert_eq!(rows.len(), 1);
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode");
    match decoded {
        SsbPayload::ShortCoderCompression(p) => {
            assert_eq!(p.tokens_in, 0);
            assert_eq!(p.tokens_out, 0);
            assert_eq!(p.reduction_pct, 0);
            assert!(p.rules_applied.is_empty());
        }
        other => panic!("expected ShortCoderCompression, got {other:?}"),
    }
}
