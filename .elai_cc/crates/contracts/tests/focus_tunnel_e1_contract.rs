// B-28 E-1 contract test: FIM prompt assembly + ModelFeatures.supports_fim.
//
// Locks the public surface added in E-1:
//   1. ModelFeatures.supports_fim: bool (compile-time field check via construction).
//   2. Factory defaults: 4 Qwen factories -> supports_fim=true; cloud_200k -> false;
//      ModelFeatures::none() -> false.
//   3. FimPromptAssembler::assemble produces a prompt containing all three Qwen
//      sentinels in the spec-locked order: prefix < suffix < middle (byte indices).
//   4. Budget invariant: when prefix+suffix+max_gen exceeds context_window,
//      assemble truncates the suffix and flags `expected_suffix_truncated=true`.
//   5. scaffold::fim::strip_fim_artifacts round-trip: trailing <|fim_pad|> /
//      <|endoftext|> echoes are removed (no behavioural change introduced by
//      orchestrator code; this is the deterministic contract E-1 reuses).

use elai_orchestrator::context_budget::{ModelCapabilityProfile, ModelFeatures};
use elai_orchestrator::focus_tunnel::{
    FimContext, FimPromptAssembler, FimPromptError,
};
use scaffold::fim::{
    strip_fim_artifacts, QWEN_END_OF_TEXT, QWEN_FIM_MIDDLE, QWEN_FIM_PAD, QWEN_FIM_PREFIX,
    QWEN_FIM_SUFFIX,
};

#[test]
fn model_features_none_has_supports_fim_false() {
    let f = ModelFeatures::none();
    assert!(
        !f.supports_fim,
        "ModelFeatures::none() must default supports_fim=false; cloud and unknown models fall back to STRUCTURED"
    );
}

#[test]
fn qwen_factories_enable_fim() {
    let cases: [(&str, ModelCapabilityProfile); 4] = [
        ("micro_7b_12gb", ModelCapabilityProfile::micro_7b_12gb()),
        ("micro_3b_12gb", ModelCapabilityProfile::micro_3b_12gb()),
        ("large_32b_48gb", ModelCapabilityProfile::large_32b_48gb()),
        ("micro_8gb", ModelCapabilityProfile::micro_8gb()),
    ];
    for (name, profile) in cases {
        assert!(
            profile.features.supports_fim,
            "{name} (Qwen2.5-Coder family) must report supports_fim=true"
        );
    }
}

#[test]
fn cloud_profile_disables_fim() {
    let profile = ModelCapabilityProfile::cloud_200k();
    assert!(
        !profile.features.supports_fim,
        "cloud_200k must report supports_fim=false (focus-tunnel-design L112-114: cloud models do not support FIM)"
    );
}

#[test]
fn assemble_emits_three_sentinels_in_locked_order() {
    let assembler = FimPromptAssembler::new(/*context_window*/ 20_000);
    let ctx = FimContext {
        file_path: "src/main.rs".to_string(),
        prefix_text: "fn main() {\n    // before\n".to_string(),
        suffix_text: "\n    // after\n}\n".to_string(),
        hint_text: Some("complete the body".to_string()),
        prefix_tokens: 16,
        suffix_tokens: 16,
        max_gen_tokens: 256,
        context_window: 20_000,
    };

    let out = assembler
        .assemble(&ctx)
        .expect("budget fits, assemble should succeed");

    let prefix_idx = out
        .prompt
        .find(QWEN_FIM_PREFIX)
        .expect("prompt must contain <|fim_prefix|>");
    let suffix_idx = out
        .prompt
        .find(QWEN_FIM_SUFFIX)
        .expect("prompt must contain <|fim_suffix|>");
    let middle_idx = out
        .prompt
        .find(QWEN_FIM_MIDDLE)
        .expect("prompt must contain <|fim_middle|>");

    assert!(
        prefix_idx < suffix_idx && suffix_idx < middle_idx,
        "Qwen FIM sentinel byte ordering locked: prefix({prefix_idx}) < suffix({suffix_idx}) < middle({middle_idx})"
    );
    assert!(
        !out.expected_suffix_truncated,
        "budget fits comfortably; suffix must NOT be truncated"
    );
}

#[test]
fn oversize_budget_triggers_suffix_truncation() {
    // 1000 lines of suffix to force the 50-line truncation gate.
    let huge_suffix: String = (0..1000)
        .map(|i| format!("// suffix line {i}\n"))
        .collect();

    let assembler = FimPromptAssembler::new(/*context_window*/ 1024);
    let ctx = FimContext {
        file_path: "src/big.rs".to_string(),
        prefix_text: "fn f() {}\n".to_string(),
        suffix_text: huge_suffix,
        hint_text: None,
        prefix_tokens: 100,
        suffix_tokens: 4000,
        max_gen_tokens: 512,
        context_window: 1024,
    };

    let out = assembler
        .assemble(&ctx)
        .expect("truncation path must succeed, not error");
    assert!(
        out.expected_suffix_truncated,
        "oversize prefix+suffix+max_gen must flip expected_suffix_truncated=true"
    );

    // Truncated suffix in the emitted prompt must be at most 50 lines worth of
    // the original suffix between the suffix and middle markers.
    let suffix_start = out
        .prompt
        .find(QWEN_FIM_SUFFIX)
        .expect("suffix sentinel present");
    let middle_start = out
        .prompt
        .find(QWEN_FIM_MIDDLE)
        .expect("middle sentinel present");
    let suffix_region = &out.prompt[suffix_start + QWEN_FIM_SUFFIX.len()..middle_start];
    let suffix_line_count = suffix_region.lines().count();
    assert!(
        suffix_line_count <= 50,
        "truncation cap is 50 lines; got {suffix_line_count}"
    );
}

#[test]
fn impossible_budget_returns_error() {
    // prefix_tokens alone exceeds context_window -> hard error, not silent truncate.
    let assembler = FimPromptAssembler::new(/*context_window*/ 512);
    let ctx = FimContext {
        file_path: "src/x.rs".to_string(),
        prefix_text: "// pretend this is huge\n".to_string(),
        suffix_text: "// suffix\n".to_string(),
        hint_text: None,
        prefix_tokens: 1024,
        suffix_tokens: 16,
        max_gen_tokens: 16,
        context_window: 512,
    };
    let err = assembler
        .assemble(&ctx)
        .expect_err("prefix alone exceeds window; must surface FimPromptError, not silently truncate");
    assert!(
        matches!(err, FimPromptError::ContextOverflow { .. }),
        "expected ContextOverflow variant"
    );
}

#[test]
fn strip_fim_artifacts_removes_trailing_sentinels() {
    // Roundtrip lock: scaffold::strip_fim_artifacts is the contract E-1 reuses
    // for cleaning model output after /v1/completions, since the sidecar does
    // not honour `stop` today (scaffold/src/fim.rs L25-27).
    let raw = format!(
        "let x = 1;\nlet y = 2;\n{pad}{eos}",
        pad = QWEN_FIM_PAD,
        eos = QWEN_END_OF_TEXT
    );
    let cleaned = strip_fim_artifacts(&raw);
    assert!(!cleaned.contains(QWEN_FIM_PAD));
    assert!(!cleaned.contains(QWEN_END_OF_TEXT));
    assert!(cleaned.contains("let x = 1;"));
    assert!(cleaned.contains("let y = 2;"));
}
