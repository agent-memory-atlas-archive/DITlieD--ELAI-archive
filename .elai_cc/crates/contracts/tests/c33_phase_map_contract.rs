//! C-33 — Documentation + phase map reconciliation contract.
//!
//! Asserts the deliverables of C-33 are present:
//!   1. Both FailureCategory enums (tools side + core side) carry the C-33
//!      LazyCode / EmptyBody / FileSizeExceeded variants in lockstep.
//!   2. The detect_category regex classifier maps representative inputs to
//!      each new variant (ClassifyHandler IPC path).
//!   3. suggest_fix() returns a non-empty fix string for every new variant.
//!   4. migration_manifest.toml has been extended to >= 80 [[command]]
//!      entries (DETERMINISTIC_CHECK from the plan).
//!   5. The B-21 plan no longer carries the stale "deferred to C-22"
//!      wording for SSB action types.
//!   6. The B-39 plan reads REFINE_CAP = 3 (not cap = 2).
//!   7. B-0_INDEX.md contains SHIPPED-status blocks for B-37, B-38, B-40,
//!      B-41, B-42 and the SCHEMA_VERSION=63 progression line.
//!
//! Spec: docs/thefinalglance/C-plan/C-33_docs_phase_map_updates.md
//! Decisions: workflow/reasoning/C-33-decisions.md

use elai_core::tool_dispatcher::ToolFailureCategory;
use elai_ipc_types::ClassifyRequest;
use elai_tools::failure_taxonomy::{ClassifyHandler, DiagnosticResult, FailureCategory};
use elai_tools::ipc_registry::ToolHandler;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// 1. Enum lockstep — both enums expose 12 variants including the three
//    new C-33 names. Constructing each variant explicitly ensures a
//    compile-time failure if any variant is renamed or removed.
// ---------------------------------------------------------------------------

#[test]
fn failure_category_tools_side_has_twelve_variants_with_c33_additions() {
    // Round-trip every variant through serde so a rename anywhere breaks
    // the test (SCREAMING_SNAKE_CASE rule).
    let variants: [(FailureCategory, &str); 12] = [
        (FailureCategory::SyntaxError, "SYNTAX_ERROR"),
        (FailureCategory::TypeMismatch, "TYPE_MISMATCH"),
        (FailureCategory::UndefinedSymbol, "UNDEFINED_SYMBOL"),
        (FailureCategory::MissingImport, "MISSING_IMPORT"),
        (FailureCategory::LifetimeError, "LIFETIME_ERROR"),
        (FailureCategory::BorrowError, "BORROW_ERROR"),
        (FailureCategory::UnusedWarning, "UNUSED_WARNING"),
        (FailureCategory::TestAssertion, "TEST_ASSERTION"),
        (FailureCategory::RuntimePanic, "RUNTIME_PANIC"),
        (FailureCategory::LazyCode, "LAZY_CODE"),
        (FailureCategory::EmptyBody, "EMPTY_BODY"),
        (FailureCategory::FileSizeExceeded, "FILE_SIZE_EXCEEDED"),
    ];
    for (cat, expected) in variants {
        let json = serde_json::to_string(&cat).expect("serialize");
        let trimmed = json.trim_matches('"');
        assert_eq!(trimmed, expected, "rename for {cat:?}");
        let back: FailureCategory = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cat, back);
    }
}

#[test]
fn failure_category_core_side_lockstep_carries_c33_additions() {
    // The core-side enum mirrors the tools-side names verbatim. `as_str`
    // is the authoritative serialization for SSB-adjacent code paths.
    let pairs: [(ToolFailureCategory, &str); 12] = [
        (ToolFailureCategory::SyntaxError, "SYNTAX_ERROR"),
        (ToolFailureCategory::TypeMismatch, "TYPE_MISMATCH"),
        (ToolFailureCategory::UndefinedSymbol, "UNDEFINED_SYMBOL"),
        (ToolFailureCategory::MissingImport, "MISSING_IMPORT"),
        (ToolFailureCategory::LifetimeError, "LIFETIME_ERROR"),
        (ToolFailureCategory::BorrowError, "BORROW_ERROR"),
        (ToolFailureCategory::UnusedWarning, "UNUSED_WARNING"),
        (ToolFailureCategory::TestAssertion, "TEST_ASSERTION"),
        (ToolFailureCategory::RuntimePanic, "RUNTIME_PANIC"),
        (ToolFailureCategory::LazyCode, "LAZY_CODE"),
        (ToolFailureCategory::EmptyBody, "EMPTY_BODY"),
        (ToolFailureCategory::FileSizeExceeded, "FILE_SIZE_EXCEEDED"),
    ];
    for (cat, expected) in pairs {
        assert_eq!(cat.as_str(), expected, "as_str rename for {cat:?}");
    }
}

// ---------------------------------------------------------------------------
// 2. detect_category routes the C-33 patterns to the new variants. We
//    drive the IPC path (ClassifyHandler) so the regex classifier AND the
//    serde round-trip are both exercised.
// ---------------------------------------------------------------------------

fn classify(text: &str) -> FailureCategory {
    let req = ClassifyRequest {
        error_output: text.to_string(),
        file_path: None,
    };
    let bytes = postcard::to_allocvec(&req).expect("postcard encode");
    let handler = ClassifyHandler::new();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("rt");
    let result = rt.block_on(handler.execute(&bytes)).expect("handler ok");
    let diag: DiagnosticResult = serde_json::from_str(&result.output).expect("diag json");
    diag.category
}

#[test]
fn lazy_code_pattern_routes_to_lazy_code_variant() {
    assert_eq!(classify("// TODO: implement"), FailureCategory::LazyCode);
    assert_eq!(classify("placeholder body"), FailureCategory::LazyCode);
    assert_eq!(classify("unimplemented!()"), FailureCategory::LazyCode);
    assert_eq!(classify("todo!()"), FailureCategory::LazyCode);
}

#[test]
fn empty_body_pattern_routes_to_empty_body_variant() {
    assert_eq!(classify("empty function body in fn foo"), FailureCategory::EmptyBody);
    assert_eq!(classify("error: no statements in block"), FailureCategory::EmptyBody);
}

#[test]
fn file_size_exceeded_pattern_routes_to_file_size_exceeded_variant() {
    assert_eq!(
        classify("file exceeds 800 lines: src/big.rs"),
        FailureCategory::FileSizeExceeded
    );
    assert_eq!(
        classify("FILE_SIZE gate fired"),
        FailureCategory::FileSizeExceeded
    );
}

// ---------------------------------------------------------------------------
// 3. suggest_fix exposes a non-empty string for every new variant. We use
//    serde_json round-trip on a synthesised DiagnosticResult because
//    suggest_fix is private; instead we drive the classifier with input
//    that lands on each variant and check fix_instruction.
// ---------------------------------------------------------------------------

fn fix_for(text: &str) -> String {
    let req = ClassifyRequest {
        error_output: text.to_string(),
        file_path: None,
    };
    let bytes = postcard::to_allocvec(&req).expect("postcard");
    let handler = ClassifyHandler::new();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("rt");
    let result = rt.block_on(handler.execute(&bytes)).expect("handler ok");
    let diag: DiagnosticResult = serde_json::from_str(&result.output).expect("diag");
    diag.fix_instruction
}

#[test]
fn suggest_fix_returns_non_empty_string_for_each_new_variant() {
    let lazy = fix_for("TODO: real impl");
    assert!(!lazy.is_empty());
    assert!(lazy.to_lowercase().contains("placeholder")
        || lazy.to_lowercase().contains("todo")
        || lazy.to_lowercase().contains("lazy"));

    let empty = fix_for("empty function body");
    assert!(!empty.is_empty());
    assert!(empty.to_lowercase().contains("implement") || empty.to_lowercase().contains("body"));

    let size = fix_for("file exceeds maximum");
    assert!(!size.is_empty());
    assert!(size.to_lowercase().contains("split") || size.to_lowercase().contains("max-lines"));
}

// ---------------------------------------------------------------------------
// 4. migration_manifest.toml has been extended.
// ---------------------------------------------------------------------------

fn elai_root() -> PathBuf {
    // contracts crate lives at .elai_cc/crates/contracts; the manifest lives
    // at .elai_cc/migration_manifest.toml.
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn migration_manifest_has_at_least_eighty_command_entries() {
    let path = elai_root().join("migration_manifest.toml");
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let count = body
        .lines()
        .filter(|l| l.trim() == "[[command]]")
        .count();
    assert!(
        count >= 80,
        "migration_manifest.toml only has {count} [[command]] entries; \
         C-33 requires >= 80 (existing 78 extended to cover the SLASH_COMMAND_SPECS \
         catalog)."
    );
    // C-33 extension entries — verify a handful by name.
    assert!(body.contains("name = \"Ask\""), "missing Ask entry");
    assert!(body.contains("name = \"GraphMermaid\""), "missing GraphMermaid entry");
    assert!(body.contains("name = \"Measure\""), "missing Measure entry");
    assert!(body.contains("name = \"Pipeline\""), "missing Pipeline entry");
    assert!(body.contains("name = \"WiringAudit\""), "missing WiringAudit entry");
}

// ---------------------------------------------------------------------------
// 5/6/7. Plan-file reconciliation. The plan files live under
//        docs/thefinalglance/B-plan/ which is two levels above the
//        elai_cc/ workspace root (D:\.ELAI_workflow\docs vs D:\.ELAI_workflow\.elai_cc).
// ---------------------------------------------------------------------------

fn docs_root() -> PathBuf {
    elai_root()
        .parent()
        .expect("workflow root")
        .join("docs")
        .join("thefinalglance")
        .join("B-plan")
}

/// Read a plan file, returning empty string if it is gitignored / missing.
/// Tests that rely on plan content `assert!` against the content; absence
/// is treated as a soft skip with a warn print rather than a hard failure,
/// because docs/ may be excluded from the working tree on some checkouts.
fn read_plan(name: &str) -> Option<String> {
    let p = docs_root().join(name);
    std::fs::read_to_string(&p).ok()
}

#[test]
fn b39_plan_uses_refine_cap_three() {
    let Some(body) = read_plan("B-39_chat_reflect_pipeline.md") else {
        eprintln!("WARN: B-39 plan file not present; skipping content check");
        return;
    };
    assert!(
        body.contains("REFINE_CAP = 3") || body.contains("cap = 3") || body.contains("cap=3"),
        "B-39 plan does not record REFINE_CAP=3 after C-33 reconciliation"
    );
    // The stale "cap = 2" line MUST be gone (only on the REFINE line; the
    // unrelated "Total pipeline budget cap = 20K" line stays).
    let stale_lines: Vec<&str> = body
        .lines()
        .filter(|l| l.contains("REFINE") && l.contains("cap = 2"))
        .collect();
    assert!(
        stale_lines.is_empty(),
        "Stale REFINE cap=2 line still present in B-39 plan: {stale_lines:?}"
    );
}

#[test]
fn b21_plan_lists_shipped_ssb_action_type_names() {
    let Some(body) = read_plan("B-21_phase_4F_feasibility_probe.md") else {
        eprintln!("WARN: B-21 plan file not present; skipping content check");
        return;
    };
    // The four B-21-era names plus the two C-22 atomic additions must all
    // appear in the WIRING_CONTRACT block.
    for name in [
        "system_probe",
        "feasibility_report",
        "novel_step_detected",
        "stub_run",
        "alternative_explored",
        "feasibility_escalated",
    ] {
        assert!(
            body.contains(name),
            "B-21 WIRING_CONTRACT missing SSB action type `{name}` after C-33 reconciliation"
        );
    }
    // The stale "deferred to C-22" wording for feasibility_escalated must
    // not survive the C-33 rewrite (C-22 already shipped it).
    assert!(
        !body.contains("deferred to C-22"),
        "B-21 plan still carries stale 'deferred to C-22' wording"
    );
}

#[test]
fn b0_index_records_b37_through_b42_status_blocks_and_schema_progression() {
    let Some(body) = read_plan("B-0_INDEX.md") else {
        eprintln!("WARN: B-0_INDEX.md not present; skipping content check");
        return;
    };
    for tag in ["B-37 SHIPPED", "B-38 SHIPPED", "B-40 SHIPPED", "B-41 SHIPPED", "B-42 SHIPPED"] {
        assert!(
            body.contains(tag),
            "B-0_INDEX missing `{tag}` status block after C-33"
        );
    }
    assert!(
        body.contains("SCHEMA_VERSION=63") || body.contains("SCHEMA_VERSION 63"),
        "B-0_INDEX missing SCHEMA_VERSION=63 progression line after C-33"
    );
}
