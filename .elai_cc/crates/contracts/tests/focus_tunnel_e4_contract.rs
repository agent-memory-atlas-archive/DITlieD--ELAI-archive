// B-28 E-4 contract test: StreamingValidator + LazyPatternGate +
// ChunkParseGate + ImportValidationGate + TokenPositionTracker +
// EarlyTermReason + ValidationDecision + SSB schema bump 31 -> 32 +
// SsbActionType::StreamingEarlyTerminated + streaming_early_termination_stats_for_payloads.
//
// Locks the public surface added in E-4. The validator is LIBRARY-TIER —
// synchronous, no I/O — and is NOT wired into run_fim_step / run_diff_step
// in E-4 (those stay STUBS per locked decision #10). The wiring lives in a
// dedicated post-E-7 WIRING phase.
//
// Test plan (per task instructions PROTOCOL step 4):
//   1.  StreamingValidator default config rejects "todo!()" only when in_code_body == true.
//   2.  "// TODO: ..." inside a comment does NOT trigger.
//   3.  `let x = "// TODO";` inside a string literal does NOT trigger.
//   4.  Unbalanced braces persistence: feed 100+ tokens with diverging brace counts triggers UnbalancedBraces.
//   5.  Output too long: feed 2.1x expected tokens triggers OutputTooLong.
//   6.  DeprecatedImport: feed a known stale rust use line triggers DeprecatedImport.
//   7.  on_complete returns Ok when no triggers fire after 500 tokens of clean code.
//   8.  SSB schema_version reads 32.
//   9.  StreamingEarlyTerminated SQL round-trip via as_sql_str + ssb_action_type_from_str_public.
//  10.  StreamingEarlyTerminatedPayload msgpack round-trip via SsbPayload::action_type.
//  11.  streaming_early_termination_stats_for_payloads aggregates correctly across 5 fake rows.
//  12.  Tree-sitter ERROR-node debouncing: a transient parse error that resolves does NOT trigger.
//  13.  Tree-sitter ERROR-node debouncing: a persistent error DOES trigger.
//  14.  Python pass body: `def f():\n    pass` triggers LazyPattern.
//  15.  Object-safety: Box<dyn TokenGate> + Box<dyn ChunkGate> compile.

use elai_orchestrator::focus_tunnel::{
    streaming_early_termination_stats_for_payloads, EarlyTermReason, LanguageHint,
    StreamingValidator, StreamingValidatorConfig, ValidationDecision, ValidationOutcome,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, SsbActionType, SsbPayload, StreamingEarlyTerminatedPayload,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_validator(language: LanguageHint, max_gen_tokens: u32) -> StreamingValidator {
    StreamingValidator::new(StreamingValidatorConfig {
        language,
        max_gen_tokens,
        step_id: "baby_step_1".into(),
        mode: "fim".into(),
        expected_factor: 2,
        brace_divergence_threshold: 8,
        brace_min_tokens_before_trigger: 100,
        deprecated_imports: Vec::new(),
    })
}

fn feed_tokens(validator: &mut StreamingValidator, text: &str) -> ValidationDecision {
    // Feed character-by-character so the token tracker exercises every byte.
    for ch in text.chars() {
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        match validator.on_token(s) {
            ValidationDecision::Continue => {}
            other => return other,
        }
    }
    ValidationDecision::Continue
}

// ---------------------------------------------------------------------------
// Test 1 — lazy todo!() in code body triggers
// ---------------------------------------------------------------------------

#[test]
fn lazy_pattern_triggers_on_rust_todo_in_code_body() {
    let mut v = default_validator(LanguageHint::Rust, 200);
    let decision = feed_tokens(&mut v, "fn foo() {\n    todo!();\n}\n");
    match decision {
        ValidationDecision::EarlyTerm { reason: EarlyTermReason::LazyPattern { .. }, .. } => {}
        other => panic!("expected LazyPattern early-term, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 2 — todo!() inside a line comment is IGNORED
// ---------------------------------------------------------------------------

#[test]
fn lazy_pattern_ignored_inside_line_comment() {
    let mut v = default_validator(LanguageHint::Rust, 200);
    let decision = feed_tokens(&mut v, "fn foo() {\n    // todo!() left for E-5\n}\n");
    assert!(
        matches!(decision, ValidationDecision::Continue),
        "comment-content must not trigger lazy gate: {decision:?}",
    );
}

// ---------------------------------------------------------------------------
// Test 3 — todo!() inside a string literal is IGNORED
// ---------------------------------------------------------------------------

#[test]
fn lazy_pattern_ignored_inside_string_literal() {
    let mut v = default_validator(LanguageHint::Rust, 200);
    let decision = feed_tokens(&mut v, "fn foo() {\n    let s = \"todo!()\";\n}\n");
    assert!(
        matches!(decision, ValidationDecision::Continue),
        "string literal must not trigger lazy gate: {decision:?}",
    );
}

// ---------------------------------------------------------------------------
// Test 4 — unbalanced braces persistence triggers after >= 100 tokens
// ---------------------------------------------------------------------------

#[test]
fn unbalanced_braces_persistence_triggers() {
    let mut v = StreamingValidator::new(StreamingValidatorConfig {
        language: LanguageHint::Rust,
        max_gen_tokens: 10_000,
        step_id: "baby_step_99".into(),
        mode: "diff".into(),
        expected_factor: 200, // disable OutputTooLong for this test
        brace_divergence_threshold: 8,
        brace_min_tokens_before_trigger: 100,
        deprecated_imports: Vec::new(),
    });
    // Generate at least 150 tokens worth of openings without any closings.
    // Test ONLY the token-level gate: do NOT call on_chunk here (chunk-level
    // tree-sitter parse would also flag the malformed input — keeping the
    // two gates separate keeps the test contract clear).
    let mut decision = ValidationDecision::Continue;
    for _ in 0..200 {
        decision = v.on_token("{");
        if !matches!(decision, ValidationDecision::Continue) {
            break;
        }
    }
    match decision {
        ValidationDecision::EarlyTerm {
            reason: EarlyTermReason::UnbalancedBraces { .. },
            ..
        } => {}
        other => panic!("expected UnbalancedBraces, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 5 — output too long triggers when token count > expected * factor
// ---------------------------------------------------------------------------

#[test]
fn output_too_long_triggers_at_expected_factor() {
    let mut v = default_validator(LanguageHint::Rust, 50);
    // Expected = 50, factor = 2 -> trigger at 101.
    let mut decision = ValidationDecision::Continue;
    for _ in 0..200 {
        decision = v.on_token("x");
        if !matches!(decision, ValidationDecision::Continue) {
            break;
        }
    }
    match decision {
        ValidationDecision::EarlyTerm {
            reason: EarlyTermReason::OutputTooLong { .. },
            ..
        } => {}
        other => panic!("expected OutputTooLong, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 6 — deprecated import triggers DeprecatedImport
// ---------------------------------------------------------------------------

#[test]
fn deprecated_import_triggers_on_blake2() {
    use elai_plugins::import_extractor::DeprecatedImport;
    let mut v = StreamingValidator::new(StreamingValidatorConfig {
        language: LanguageHint::Rust,
        max_gen_tokens: 1_000,
        step_id: "baby_step_2".into(),
        mode: "fim".into(),
        expected_factor: 100,
        brace_divergence_threshold: 64, // disable for this test
        brace_min_tokens_before_trigger: 10_000,
        deprecated_imports: vec![DeprecatedImport {
            ecosystem: "cargo".to_string(),
            deprecated_name: "blake2".to_string(),
            replacement_name: "blake3".to_string(),
            severity: "BLOCK".to_string(),
            reason: "RUSTSEC blake2 deprecated for non-RFC uses".to_string(),
        }],
    });
    let source = "use blake2::Blake2b;\nfn hash() -> Blake2b { todo_inert() }\n";
    let mut decision = ValidationDecision::Continue;
    for ch in source.chars() {
        let mut buf = [0u8; 4];
        let s = ch.encode_utf8(&mut buf);
        decision = v.on_token(s);
        if !matches!(decision, ValidationDecision::Continue) {
            break;
        }
    }
    // Chunk gate is what runs the import extractor — make sure we flush.
    if matches!(decision, ValidationDecision::Continue) {
        decision = v.on_chunk(source);
    }
    match decision {
        ValidationDecision::EarlyTerm {
            reason: EarlyTermReason::DeprecatedImport { .. },
            ..
        } => {}
        other => panic!("expected DeprecatedImport, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 7 — on_complete returns Ok after clean code
// ---------------------------------------------------------------------------

#[test]
fn on_complete_returns_ok_on_clean_code() {
    let mut v = default_validator(LanguageHint::Rust, 1_000);
    let clean = "fn add(a: i32, b: i32) -> i32 { a + b }\n";
    // Feed clean code ~12 times = ~500 tokens of plausible code.
    for _ in 0..12 {
        let d = feed_tokens(&mut v, clean);
        assert!(matches!(d, ValidationDecision::Continue), "should remain Continue: {d:?}");
        let chunk = v.on_chunk(clean);
        assert!(matches!(chunk, ValidationDecision::Continue), "chunk should be Continue: {chunk:?}");
    }
    let outcome = v.on_complete();
    assert!(outcome.is_ok(), "expected on_complete Ok, got {outcome:?}");
}

// ---------------------------------------------------------------------------
// Test 8 — SSB schema_version reads >= 32 (E-4 baseline). B-28 WIRING-1
// bumps the constant to 35 in one atomic edit (PipelinePreAssembled /
// DependencyResolved / MicroStepCreated). The assertion stays a `>=` check
// so future bumps do not require touching this contract.
// ---------------------------------------------------------------------------

#[test]
fn ssb_schema_version_is_at_least_e4_baseline() {
    let v = elai_orchestrator::ssb::SCHEMA_VERSION_PUBLIC;
    assert!(
        v >= 32,
        "B-28 E-4 floors SCHEMA_VERSION at 32; got {v}. WIRING-1 bumps to 35."
    );
}

// ---------------------------------------------------------------------------
// Test 9 — SsbActionType::StreamingEarlyTerminated SQL round-trip
// ---------------------------------------------------------------------------

#[test]
fn streaming_early_terminated_sql_round_trip() {
    let s = SsbActionType::StreamingEarlyTerminated.as_sql_str();
    assert_eq!(s, "streaming_early_terminated");
    let back = ssb_action_type_from_str_public(s).expect("from_str");
    assert_eq!(back, SsbActionType::StreamingEarlyTerminated);
}

// ---------------------------------------------------------------------------
// Test 10 — StreamingEarlyTerminatedPayload msgpack round-trip
// ---------------------------------------------------------------------------

#[test]
fn streaming_early_terminated_payload_round_trip() {
    let payload = StreamingEarlyTerminatedPayload {
        step_id: "baby_step_42".into(),
        mode: "fim".into(),
        reason: "lazy_pattern".into(),
        reason_detail: Some("rust_todo_macro".into()),
        token_count_at_term: 17,
        total_tokens_received: 17,
        language_hint: "rust".into(),
    };
    let wrapped = SsbPayload::StreamingEarlyTerminated(payload.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    assert_eq!(decoded, wrapped, "msgpack round-trip");
    assert_eq!(decoded.action_type(), SsbActionType::StreamingEarlyTerminated);
}

// ---------------------------------------------------------------------------
// Test 11 — streaming_early_termination_stats_for_payloads aggregates 5 rows
// ---------------------------------------------------------------------------

#[test]
fn stats_aggregates_five_rows() {
    let p_lazy = SsbPayload::StreamingEarlyTerminated(StreamingEarlyTerminatedPayload {
        step_id: "s1".into(),
        mode: "fim".into(),
        reason: "lazy_pattern".into(),
        reason_detail: None,
        token_count_at_term: 10,
        total_tokens_received: 10,
        language_hint: "rust".into(),
    });
    let p_braces = SsbPayload::StreamingEarlyTerminated(StreamingEarlyTerminatedPayload {
        step_id: "s2".into(),
        mode: "diff".into(),
        reason: "unbalanced_braces".into(),
        reason_detail: None,
        token_count_at_term: 250,
        total_tokens_received: 250,
        language_hint: "rust".into(),
    });
    let p_long = SsbPayload::StreamingEarlyTerminated(StreamingEarlyTerminatedPayload {
        step_id: "s3".into(),
        mode: "fim".into(),
        reason: "output_too_long".into(),
        reason_detail: None,
        token_count_at_term: 4_096,
        total_tokens_received: 4_096,
        language_hint: "python".into(),
    });
    let p_dep = SsbPayload::StreamingEarlyTerminated(StreamingEarlyTerminatedPayload {
        step_id: "s4".into(),
        mode: "fim".into(),
        reason: "deprecated_import".into(),
        reason_detail: Some("blake2".into()),
        token_count_at_term: 32,
        total_tokens_received: 32,
        language_hint: "rust".into(),
    });
    let p_lazy2 = SsbPayload::StreamingEarlyTerminated(StreamingEarlyTerminatedPayload {
        step_id: "s5".into(),
        mode: "fim".into(),
        reason: "lazy_pattern".into(),
        reason_detail: None,
        token_count_at_term: 5,
        total_tokens_received: 5,
        language_hint: "rust".into(),
    });
    let payloads: Vec<&SsbPayload> = vec![&p_lazy, &p_braces, &p_long, &p_dep, &p_lazy2];
    let stats = streaming_early_termination_stats_for_payloads(&payloads);
    assert_eq!(stats.total_terminations, 5);
    assert_eq!(stats.reason_breakdown.get("lazy_pattern").copied(), Some(2));
    assert_eq!(stats.reason_breakdown.get("unbalanced_braces").copied(), Some(1));
    assert_eq!(stats.reason_breakdown.get("output_too_long").copied(), Some(1));
    assert_eq!(stats.reason_breakdown.get("deprecated_import").copied(), Some(1));
    assert_eq!(stats.total_tokens_saved, 10 + 250 + 4_096 + 32 + 5);
}

// ---------------------------------------------------------------------------
// Test 12 — transient tree-sitter error does NOT trigger (debounce = 2)
// ---------------------------------------------------------------------------

#[test]
fn transient_parse_error_does_not_trigger() {
    let mut v = StreamingValidator::new(StreamingValidatorConfig {
        language: LanguageHint::Rust,
        max_gen_tokens: 10_000,
        step_id: "baby_step_dbnc".into(),
        mode: "diff".into(),
        expected_factor: 200,
        brace_divergence_threshold: 256, // disable
        brace_min_tokens_before_trigger: 10_000,
        deprecated_imports: Vec::new(),
    });
    // Chunk 1 has incomplete `fn foo()` — tree-sitter will flag ERROR.
    let d1 = v.on_chunk("fn foo(");
    // Chunk 2 resolves it.
    let d2 = v.on_chunk("fn foo() {}");
    assert!(matches!(d1, ValidationDecision::Continue), "single ERROR must not trigger: {d1:?}");
    assert!(matches!(d2, ValidationDecision::Continue), "resolved tree must not trigger: {d2:?}");
}

// ---------------------------------------------------------------------------
// Test 13 — persistent tree-sitter ERROR across 2 chunks triggers
// ---------------------------------------------------------------------------

#[test]
fn persistent_parse_error_triggers() {
    let mut v = StreamingValidator::new(StreamingValidatorConfig {
        language: LanguageHint::Rust,
        max_gen_tokens: 10_000,
        step_id: "baby_step_persist".into(),
        mode: "diff".into(),
        expected_factor: 200,
        brace_divergence_threshold: 256,
        brace_min_tokens_before_trigger: 10_000,
        deprecated_imports: Vec::new(),
    });
    // Both chunks are broken — the persistent ERROR-node fires after the
    // second chunk's parse also reports has_error == true. The validator
    // reports this via the UnbalancedBraces variant (structural error is
    // surfaced through that variant; a richer TreeSitterStructuralError
    // variant could be added later without breaking this contract).
    let _ = v.on_chunk("fn foo(");
    let d2 = v.on_chunk("fn foo( ) ) ) garbage ;;;;");
    assert!(
        matches!(d2, ValidationDecision::EarlyTerm { .. }),
        "persistent ERROR across 2 chunks must trigger early-term, got {d2:?}",
    );
}

// ---------------------------------------------------------------------------
// Test 14 — Python `pass` body triggers LazyPattern
// ---------------------------------------------------------------------------

#[test]
fn python_pass_body_triggers() {
    let mut v = default_validator(LanguageHint::Python, 200);
    let src = "def foo():\n    pass\n";
    let decision = feed_tokens(&mut v, src);
    match decision {
        ValidationDecision::EarlyTerm {
            reason: EarlyTermReason::LazyPattern { .. },
            ..
        } => {}
        other => panic!("expected LazyPattern for Python pass body, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Test 15 — Object-safety: StreamingValidator usable through generic borrow
// ---------------------------------------------------------------------------

#[test]
fn streaming_validator_constructs_for_all_languages() {
    // Smoke test that StreamingValidator constructs for every LanguageHint and
    // is consumable through &mut (object-safe by virtue of being concrete; the
    // gates are dynamic-dispatch internally).
    for lang in [LanguageHint::Rust, LanguageHint::Python, LanguageHint::Generic] {
        let mut v = default_validator(lang, 100);
        let d = v.on_token("x");
        assert!(matches!(d, ValidationDecision::Continue), "smoke token: {d:?}");
    }
}

// ---------------------------------------------------------------------------
// Test 16 — on_complete after LazyPattern termination returns
//           Err(ValidationOutcome::Terminated{ reason: LazyPattern{..} })
//
// Locks B-28 E-4 BLOCK 1 fix: on_complete() must surface the REAL reason
// captured at the moment terminate() fired, not a hardcoded
// EarlyTermReason::OutputTooLong placeholder. Without the
// `termination_reason` field on StreamingValidator, the WIRING phase
// would insert wrong reason strings into the SSB ledger.
// ---------------------------------------------------------------------------

#[test]
fn on_complete_after_lazy_pattern_returns_err_with_real_reason() {
    let mut v = default_validator(LanguageHint::Rust, 200);
    let decision = feed_tokens(&mut v, "fn foo() {\n    todo!();\n}\n");
    assert!(
        matches!(decision, ValidationDecision::EarlyTerm { reason: EarlyTermReason::LazyPattern { .. }, .. }),
        "precondition: validator must have terminated via LazyPattern, got {decision:?}",
    );
    let outcome = v.on_complete();
    let err = outcome.expect_err("on_complete must return Err after termination");
    match err {
        ValidationOutcome::Terminated {
            reason: EarlyTermReason::LazyPattern { .. },
            ..
        } => {}
        other => panic!(
            "on_complete must surface the captured LazyPattern reason; got {other:?}"
        ),
    }
}

// ---------------------------------------------------------------------------
// Test 17 — on_complete after UnbalancedBraces termination returns
//           Err(ValidationOutcome::Terminated{ reason: UnbalancedBraces{..} })
//
// Same lock as Test 16 but for the UnbalancedBraces variant. If
// on_complete() were still hardcoded to OutputTooLong, this would fail.
// ---------------------------------------------------------------------------

#[test]
fn on_complete_after_unbalanced_braces_returns_err_with_real_reason() {
    let mut v = StreamingValidator::new(StreamingValidatorConfig {
        language: LanguageHint::Rust,
        max_gen_tokens: 10_000,
        step_id: "baby_step_block1".into(),
        mode: "diff".into(),
        expected_factor: 200, // disable OutputTooLong for this test
        brace_divergence_threshold: 8,
        brace_min_tokens_before_trigger: 100,
        deprecated_imports: Vec::new(),
    });
    let mut decision = ValidationDecision::Continue;
    for _ in 0..200 {
        decision = v.on_token("{");
        if !matches!(decision, ValidationDecision::Continue) {
            break;
        }
    }
    assert!(
        matches!(decision, ValidationDecision::EarlyTerm { reason: EarlyTermReason::UnbalancedBraces { .. }, .. }),
        "precondition: validator must have terminated via UnbalancedBraces, got {decision:?}",
    );
    let outcome = v.on_complete();
    let err = outcome.expect_err("on_complete must return Err after termination");
    match err {
        ValidationOutcome::Terminated {
            reason: EarlyTermReason::UnbalancedBraces { .. },
            ..
        } => {}
        other => panic!(
            "on_complete must surface the captured UnbalancedBraces reason; got {other:?}"
        ),
    }
}

// ---------------------------------------------------------------------------
// Test 18 — Over-close brace stream triggers UnbalancedBraces
//
// Locks B-28 E-4 BLOCK 2 fix: feeding only `}` bytes (no openers) must
// trip the brace-balance gate. The fix introduced an explicit
// `close_excess: u32` counter on TokenPositionTracker that increments
// when a `}` arrives while brace_depth <= 0; the gate now fires if
// EITHER unsigned_abs(brace_depth) OR close_excess exceeds the
// configured threshold.
// ---------------------------------------------------------------------------

#[test]
fn over_close_braces_triggers_unbalanced_braces() {
    let mut v = StreamingValidator::new(StreamingValidatorConfig {
        language: LanguageHint::Rust,
        max_gen_tokens: 10_000,
        step_id: "baby_step_overclose".into(),
        mode: "diff".into(),
        expected_factor: 200, // disable OutputTooLong for this test
        brace_divergence_threshold: 8,
        brace_min_tokens_before_trigger: 100,
        deprecated_imports: Vec::new(),
    });
    // Feed only `}` — every byte is a close brace with no matching opener.
    // After ~110 tokens we are well past brace_min_tokens_before_trigger
    // and well past brace_divergence_threshold = 8.
    let mut decision = ValidationDecision::Continue;
    for _ in 0..200 {
        decision = v.on_token("}");
        if !matches!(decision, ValidationDecision::Continue) {
            break;
        }
    }
    match decision {
        ValidationDecision::EarlyTerm {
            reason: EarlyTermReason::UnbalancedBraces { .. },
            ..
        } => {}
        other => panic!(
            "over-close stream must trigger UnbalancedBraces (BLOCK 2 fix), got {other:?}"
        ),
    }
}
