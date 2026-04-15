//! A-46 contract tests: BoundedString<N>, ContextTelescope, SurgicalTelescope,
//! PhaseEntry summary field, Slot 3 sizing constants.

use elai_core::bounded_string::{BoundedString, TooLong, WordTokenCounter};
use elai_core::telescope::{
    ContextTelescope, SurgicalTelescope, LABEL_OVERHEAD_TOKENS,
    TELESCOPE_VISION_MAX, TELESCOPE_CONCEPT_MAX, TELESCOPE_PHASE_MAX, TELESCOPE_STEP_MAX,
    SURGICAL_VISION_MAX, SURGICAL_BLAST_MAX, SURGICAL_STEP_MAX,
};
use elai_orchestrator::plan_skeleton::PhaseEntry;
use elai_orchestrator::context_budget::{
    ModelCapabilityProfile, FIXED_SLOT_TELESCOPE_TARGET, LARGE_SLOT_TELESCOPE, MICRO_SLOT_TELESCOPE,
};

// ---------------------------------------------------------------------------
// BoundedString<N> contracts
// ---------------------------------------------------------------------------

#[test]
fn bounded_string_accepts_exactly_n_tokens() {
    let counter = WordTokenCounter;
    // 5 tokens exactly
    let result = BoundedString::<5>::new("one two three four five", &counter);
    assert!(result.is_ok(), "exactly N tokens must succeed");
    assert_eq!(result.unwrap().as_ref(), "one two three four five");
}

#[test]
fn bounded_string_rejects_n_plus_one_tokens() {
    let counter = WordTokenCounter;
    let result = BoundedString::<5>::new("one two three four five six", &counter);
    assert!(result.is_err(), "N+1 tokens must fail");
    let err = result.unwrap_err();
    match err {
        TooLong { actual, limit } => {
            assert_eq!(actual, 6);
            assert_eq!(limit, 5);
        }
    }
}

#[test]
fn bounded_string_accepts_empty_string() {
    let counter = WordTokenCounter;
    let result = BoundedString::<5>::new("", &counter);
    assert!(result.is_ok(), "empty string (0 tokens) must succeed");
    assert_eq!(result.unwrap().as_ref(), "");
}

#[test]
fn bounded_string_serde_roundtrip() {
    let counter = WordTokenCounter;
    let bs = BoundedString::<10>::new("hello world", &counter).unwrap();
    let json = serde_json::to_string(&bs).unwrap();
    // Serializes as plain string
    assert_eq!(json, "\"hello world\"");
    // Deserializes without token check (trusts stored data)
    let deser: BoundedString<10> = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.as_ref(), "hello world");
}

// ---------------------------------------------------------------------------
// ContextTelescope contracts
// ---------------------------------------------------------------------------

#[test]
fn context_telescope_render_all_layers() {
    let counter = WordTokenCounter;
    let ct = ContextTelescope {
        project_vision: BoundedString::new("Build a coding assistant", &counter).unwrap(),
        concept_summary: Some(BoundedString::new("NLP module", &counter).unwrap()),
        phase_goal: Some(BoundedString::new("Implement tokenizer", &counter).unwrap()),
        step_context: Some(BoundedString::new("Add BPE support", &counter).unwrap()),
    };
    let rendered = ct.render();
    assert!(rendered.contains("Build a coding assistant"));
    assert!(rendered.contains("NLP module"));
    assert!(rendered.contains("Implement tokenizer"));
    assert!(rendered.contains("Add BPE support"));
}

#[test]
fn context_telescope_render_vision_only() {
    let counter = WordTokenCounter;
    let ct = ContextTelescope {
        project_vision: BoundedString::new("Build a coding assistant", &counter).unwrap(),
        concept_summary: None,
        phase_goal: None,
        step_context: None,
    };
    let rendered = ct.render();
    assert!(!rendered.is_empty(), "vision-only telescope must produce non-empty string");
    assert!(rendered.contains("Build a coding assistant"));
}

#[test]
fn context_telescope_token_estimate_bounded() {
    let counter = WordTokenCounter;
    let ct = ContextTelescope {
        project_vision: BoundedString::new("Build a coding assistant", &counter).unwrap(),
        concept_summary: Some(BoundedString::new("NLP module", &counter).unwrap()),
        phase_goal: Some(BoundedString::new("Implement tokenizer", &counter).unwrap()),
        step_context: Some(BoundedString::new("Add BPE support", &counter).unwrap()),
    };
    let est = ct.token_estimate();
    // Must be > 0 and <= sum of all max bounds + overhead
    assert!(est > 0);
    let max_possible = TELESCOPE_VISION_MAX + TELESCOPE_CONCEPT_MAX
        + TELESCOPE_PHASE_MAX + TELESCOPE_STEP_MAX + LABEL_OVERHEAD_TOKENS;
    assert!(est <= max_possible, "estimate {est} exceeds max {max_possible}");
}

// ---------------------------------------------------------------------------
// SurgicalTelescope contracts
// ---------------------------------------------------------------------------

#[test]
fn surgical_telescope_render_three_layers() {
    let counter = WordTokenCounter;
    let st = SurgicalTelescope {
        project_vision: BoundedString::new("Build a coding assistant", &counter).unwrap(),
        blast_radius: Some(BoundedString::new("affects tokenizer and parser modules", &counter).unwrap()),
        step_task: Some(BoundedString::new("Fix off-by-one in BPE merge", &counter).unwrap()),
    };
    let rendered = st.render();
    assert!(rendered.contains("Build a coding assistant"));
    assert!(rendered.contains("affects tokenizer and parser modules"));
    assert!(rendered.contains("Fix off-by-one in BPE merge"));
}

#[test]
fn surgical_telescope_token_estimate_within_range() {
    let counter = WordTokenCounter;
    let st = SurgicalTelescope {
        project_vision: BoundedString::new("short", &counter).unwrap(),
        blast_radius: Some(BoundedString::new("small", &counter).unwrap()),
        step_task: Some(BoundedString::new("fix", &counter).unwrap()),
    };
    let est = st.token_estimate();
    // Minimal input: must be well under 600
    assert!(est <= 600, "minimal surgical telescope estimate {est} should be <= 600");
    assert!(est > 0);
}

// ---------------------------------------------------------------------------
// PhaseEntry summary field contracts
// ---------------------------------------------------------------------------

#[test]
fn phase_entry_serde_roundtrip_without_summary() {
    let entry = PhaseEntry::new("phase-1", "Build the thing").unwrap();
    let toml_str = toml::to_string(&entry).unwrap();
    // summary field must not appear in serialized output when None
    assert!(!toml_str.contains("summary"), "None summary must not serialize");
    let deser: PhaseEntry = toml::from_str(&toml_str).unwrap();
    assert!(deser.summary.is_none());
}

#[test]
fn phase_entry_serde_roundtrip_with_summary() {
    let mut entry = PhaseEntry::new("phase-1", "Build the thing").unwrap();
    entry.summary = Some("A brief summary of this phase".to_string());
    let toml_str = toml::to_string(&entry).unwrap();
    assert!(toml_str.contains("summary"), "Some summary must serialize");
    let deser: PhaseEntry = toml::from_str(&toml_str).unwrap();
    assert_eq!(deser.summary.as_deref(), Some("A brief summary of this phase"));
}

// ---------------------------------------------------------------------------
// Slot 3 sizing constants contracts
// ---------------------------------------------------------------------------

#[test]
fn large_slot_telescope_is_1000() {
    assert_eq!(LARGE_SLOT_TELESCOPE, 1000);
}

#[test]
fn micro_slot_telescope_is_400() {
    assert_eq!(MICRO_SLOT_TELESCOPE, 400);
}

#[test]
fn all_model_config_telescope_targets_at_least_1050() {
    for (name, profile) in [
        ("micro_7b", ModelCapabilityProfile::micro_7b_12gb()),
        ("micro_3b", ModelCapabilityProfile::micro_3b_12gb()),
        ("large_32b", ModelCapabilityProfile::large_32b_48gb()),
        ("micro_8gb", ModelCapabilityProfile::micro_8gb()),
    ] {
        assert!(
            profile.fixed_slot_targets[2] >= FIXED_SLOT_TELESCOPE_TARGET,
            "{name} fixed_slot_targets[2] = {} must be >= {}",
            profile.fixed_slot_targets[2],
            FIXED_SLOT_TELESCOPE_TARGET,
        );
    }
}

// ---------------------------------------------------------------------------
// Named constants contracts
// ---------------------------------------------------------------------------

#[test]
fn telescope_constants_match_spec() {
    assert_eq!(TELESCOPE_VISION_MAX, 200);
    assert_eq!(TELESCOPE_CONCEPT_MAX, 150);
    assert_eq!(TELESCOPE_PHASE_MAX, 100);
    assert_eq!(TELESCOPE_STEP_MAX, 100);
    assert_eq!(SURGICAL_VISION_MAX, 200);
    assert_eq!(SURGICAL_BLAST_MAX, 200);
    assert_eq!(SURGICAL_STEP_MAX, 200);
}
