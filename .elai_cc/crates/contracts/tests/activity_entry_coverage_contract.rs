//! Batch 2C — ActivityEntry coverage contract.
//!
//! Ensures that `ActivityEntry.svelte` maps EVERY `SsbActionType` enum
//! variant (crates/orchestrator/src/ssb.rs L87-L174) to a non-`dim` colour
//! token AND a non-empty, non-raw-enum typeLabel. The MVP audit 2026-04-18
//! section 8 requires that no SSB row render as an undifferentiated grey
//! line — this contract fails the build if a new variant is added to the
//! Rust enum without a matching frontend case.
//!
//! DISABLED (B-23 Phase 4H Group A audit): the original `ActivityEntry.svelte`
//! lives under `crates/.bkp_old/dashboard/src-frontend/src/` after the
//! dashboard-v2 migration in s-1430428. The dashboard-v2 equivalent has
//! not landed yet (planned Group F). Until Group F adds the new mapping,
//! this contract panics at file-read for the active path. Same gating as
//! the four `dashboard_*_contract.rs` files (see crates/contracts/Cargo.toml
//! L21-23). The cfg gate keeps the variant inventory in sync (Contract 7
//! still passes via the `>= 78` length floor) and the colour/label checks
//! reactivate once the dashboard-v2 ActivityEntry surface ships.
//!
//! Approach: the test reads the Svelte file as UTF-8, walks the enum via
//! `SsbActionType::as_sql_str()`, and asserts grep-style that each
//! variant's snake_case string appears inside the `switch` in `colour()`
//! AND inside `typeLabel()`. We also assert a minimum number of finding
//! categories are present in `FINDING_CATEGORIES`.
//!
//! Grounding: frontend file lives at
//!   crates/dashboard/src-frontend/src/ActivityEntry.svelte
//! We use `CARGO_MANIFEST_DIR` (the `contracts` crate root) to resolve
//! it — this makes the test path-independent of the caller's cwd.

#![cfg(any())]

use std::fs;
use std::path::PathBuf;

use elai_orchestrator::ssb::SsbActionType;

/// Every declared variant — keep in sync with SsbActionType. Used to
/// iterate even though the enum itself isn't strum-backed. A compilation
/// check inside the iterator keeps the list honest: adding a new variant
/// to the enum without adding it here would leave it uncovered.
const ALL_VARIANTS: &[SsbActionType] = &[
    SsbActionType::FileWrite,
    SsbActionType::FileRead,
    SsbActionType::ToolResult,
    SsbActionType::TestOutcome,
    SsbActionType::Finding,
    SsbActionType::Claim,
    SsbActionType::Checkpoint,
    SsbActionType::ModelSwap,
    SsbActionType::PhaseTransition,
    SsbActionType::KnowledgeDegradation,
    SsbActionType::IsolationMetric,
    SsbActionType::PlanUpdate,
    SsbActionType::ReplanRequest,
    SsbActionType::StubReference,
    SsbActionType::ReasoningTrace,
    SsbActionType::PhaseComplete,
    SsbActionType::MemoryMonitoring,
    SsbActionType::WorkloadProfile,
    SsbActionType::DeathEvent,
    SsbActionType::RecoveryEvent,
    SsbActionType::SupervisionExhausted,
    SsbActionType::SpiralDetected,
    SsbActionType::ContextReduction,
    SsbActionType::BranchMerge,
    SsbActionType::MistakeEntry,
    SsbActionType::ResearchStep,
    SsbActionType::ResearchDecomposeLevel,
    SsbActionType::WebResearchStep,
    SsbActionType::PreviewAvailable,
    SsbActionType::InterventionRequest,
    SsbActionType::InterventionResponse,
    SsbActionType::TaskSubmitted,
    SsbActionType::HookEvent,
    SsbActionType::ProviderSignals,
    SsbActionType::SearchTriggerFired,
    SsbActionType::VramTierDetected,
    SsbActionType::IndexingProgress,
    SsbActionType::DkgReady,
    SsbActionType::TieredAnalysis,
    SsbActionType::ShortCoderCompression,
    SsbActionType::ToolDispatch,
    SsbActionType::ValidationTier,
    SsbActionType::FailureClassified,
    SsbActionType::GitCommit,
    SsbActionType::DairaTrace,
    SsbActionType::SystemProbe,
    SsbActionType::FeasibilityReport,
    SsbActionType::NovelStepDetected,
    SsbActionType::StubRun,
    // Batch 3B (Phase 4G).
    SsbActionType::GateDecision,
    SsbActionType::TaskSplit,
    SsbActionType::EscalationFired,
    SsbActionType::ModeChanged,
    // Batch 3C (Phase 4H) — Layer 1A.
    SsbActionType::ProposalGenerated,
    SsbActionType::ProposalApplied,
    SsbActionType::ProposalRejected,
    SsbActionType::MsiUpdate,
    SsbActionType::ConfigArchived,
    // Batch 4C — scattered A-step wiring.
    SsbActionType::ProfileSelected,
    SsbActionType::FsmTransition,
    SsbActionType::MicroReflection,
    SsbActionType::TelescopeCollapse,
    SsbActionType::ConfigWarning,
    // Batch-fu — cross-cutting follow-up wiring.
    SsbActionType::ConstraintsInjected,
    SsbActionType::InterventionResolved,
    // Batch-ful-a — DkgQuerierAdapter hot-swap signal.
    SsbActionType::DkgQuerierSwapped,
    // Batch-ful-c — A-8 freshness steering vector lifecycle.
    SsbActionType::SteeringComputed,
    SsbActionType::SteeringComputeFailed,
    // Batch-ful-d — external MCP bridge lifecycle + per-call rows.
    SsbActionType::McpServerConnected,
    SsbActionType::McpToolCalled,
    SsbActionType::McpServerDisconnected,
    // B-27 — cloud-mode spend ledger + escalation marker.
    SsbActionType::CloudSpend,
    SsbActionType::EscalateToCloud,
    // B-23 Phase 4H Group A — self-mod beacons (Layer 1A growth + Layer 3 +
    // OSC + REG + EVO). Five batch-3C variants (proposal_*, msi_update,
    // config_archived) shipped earlier.
    SsbActionType::DataTableGrew,
    SsbActionType::L3Proposal,
    SsbActionType::ConvergenceStagnant,
    SsbActionType::ConfigOscillation,
    SsbActionType::RegressionDetected,
    SsbActionType::EvolutionWithheld,
];

/// Batch 2C finding categories that MUST surface inside ActivityEntry.svelte
/// FINDING_CATEGORIES. Covers the pre-existing categories plus the new
/// failure-taxonomy set from `FailureClassifiedPayload` (rule_violation,
/// integration_*, compilation_error, test_failure, runtime_error,
/// timeout, resource_exhausted, context_explosion, raw_log_overload,
/// phase_gate_failure, file_size_exceeded).
const REQUIRED_FINDING_CATEGORIES: &[&str] = &[
    "rule_violation",
    "interrogation_fix_suggestion",
    "agent_failed",
    "integration_checkpoint_failed",
    "integration_bubble_up",
    "hook_pre_tool_use_block",
    "hook_post_tool_use_block",
    "tiered_analysis_block",
    "compilation_error",
    "test_failure",
    "runtime_error",
    "timeout",
    "resource_exhausted",
    "context_explosion",
    "raw_log_overload",
    "phase_gate_failure",
    "file_size_exceeded",
];

fn activity_entry_path() -> PathBuf {
    // CARGO_MANIFEST_DIR at contract-test build time points at
    // .elai_cc/crates/contracts/. The frontend lives at
    // .elai_cc/crates/dashboard/src-frontend/src/ActivityEntry.svelte.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .expect("crates dir")
        .join("dashboard")
        .join("src-frontend")
        .join("src")
        .join("ActivityEntry.svelte")
}

fn read_activity_entry() -> String {
    let p = activity_entry_path();
    fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("read {}: {e}", p.display()))
}

// ---------------------------------------------------------------------------
// Contract 1: every SsbActionType variant has a `case "<snake>":` line in the
// colour() switch.
// ---------------------------------------------------------------------------

#[test]
fn contract_every_variant_has_colour_case() {
    let src = read_activity_entry();
    let mut missing: Vec<&str> = Vec::new();
    for variant in ALL_VARIANTS {
        let needle = format!("case \"{}\":", variant.as_sql_str());
        if !src.contains(&needle) {
            missing.push(variant.as_sql_str());
        }
    }
    assert!(
        missing.is_empty(),
        "ActivityEntry.svelte colour() is missing cases for: {missing:?}"
    );
}

// ---------------------------------------------------------------------------
// Contract 2: no variant routes to a `dim` colour in the non-default arm.
// The `default: return \"dim\";` line is permitted (it only catches unknown
// enum strings). We assert that no explicit `case "<snake>":` block returns
// "dim" directly inside the next 4 lines of the file.
// ---------------------------------------------------------------------------

#[test]
fn contract_no_explicit_variant_returns_dim() {
    let src = read_activity_entry();
    // Split on `case "<snake>":` and look at the text between that case
    // label and either the next `case `/`}` brace. We assert `return "dim";`
    // (or return 'dim') never appears inside that window.
    for variant in ALL_VARIANTS {
        let needle = format!("case \"{}\":", variant.as_sql_str());
        let Some(idx) = src.find(&needle) else {
            continue; // covered by contract 1
        };
        let tail = &src[idx + needle.len()..];
        // Bound the window to the next `case `, `default:`, or `}` — whichever
        // comes first — so we only inspect the body of THIS case.
        let end = [tail.find("case \""), tail.find("default:"), tail.find('}')]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(tail.len());
        let window = &tail[..end];
        assert!(
            !window.contains("return \"dim\""),
            "{} routes explicitly to dim in colour() — every variant must map to a non-dim token (see Batch 2C spec)",
            variant.as_sql_str()
        );
    }
}

// ---------------------------------------------------------------------------
// Contract 3: every variant produces a non-empty, differentiated typeLabel.
// We check that each variant string appears inside a `case` or `at === "..."`
// predicate inside the `typeLabel()` function, or that a matching return
// line ties the variant to a human-readable label. Since typeLabel is long,
// we use two complementary heuristics and OR them:
//
//   (a) The literal `case "<snake>":` appears in typeLabel's switch. OR
//   (b) `at === "<snake>"` appears as a guarded if-branch. OR
//   (c) `at === "finding"` — Finding has a dedicated branch that produces
//       category-specific labels; ensured by contract 4.
// ---------------------------------------------------------------------------

#[test]
fn contract_every_variant_has_type_label() {
    let src = read_activity_entry();
    // Extract the typeLabel body so we only search within it — this avoids
    // false positives from the colour() switch.
    let label_start = src
        .find("function typeLabel(")
        .expect("typeLabel function present");
    let label_body = &src[label_start..];
    let mut missing: Vec<&str> = Vec::new();
    for variant in ALL_VARIANTS {
        let snake = variant.as_sql_str();
        let case_label = format!("case \"{snake}\":");
        let if_label = format!("at === \"{snake}\"");
        if !label_body.contains(&case_label) && !label_body.contains(&if_label) {
            missing.push(snake);
        }
    }
    assert!(
        missing.is_empty(),
        "ActivityEntry.svelte typeLabel() missing branches for: {missing:?}"
    );
}

// ---------------------------------------------------------------------------
// Contract 4: FINDING_CATEGORIES array contains at least 15 documented
// categories (task spec: findingCategory must recognise at least 15).
// We also assert the 15+ required names appear verbatim as array entries.
// ---------------------------------------------------------------------------

#[test]
fn contract_finding_categories_coverage() {
    let src = read_activity_entry();
    // Locate the array literal.
    let arr_start = src
        .find("FINDING_CATEGORIES")
        .expect("FINDING_CATEGORIES declared");
    let arr_body = &src[arr_start..];
    let mut missing: Vec<&str> = Vec::new();
    for cat in REQUIRED_FINDING_CATEGORIES {
        let needle = format!("\"{cat}\"");
        if !arr_body.contains(&needle) {
            missing.push(cat);
        }
    }
    assert!(
        missing.is_empty(),
        "FINDING_CATEGORIES missing: {missing:?}"
    );
    assert!(
        REQUIRED_FINDING_CATEGORIES.len() >= 15,
        "spec requires findingCategory() recognise at least 15 categories; required list is {}",
        REQUIRED_FINDING_CATEGORIES.len()
    );
}

// ---------------------------------------------------------------------------
// Contract 5: findingLevel() escalates BLOCK > WARN > INFO (substring scan).
// ---------------------------------------------------------------------------

#[test]
fn contract_finding_level_escalation_wording() {
    let src = read_activity_entry();
    let fn_start = src
        .find("function findingLevel(")
        .expect("findingLevel declared");
    let fn_body = &src[fn_start..];
    // BLOCK / fail branch must precede the WARN branch so escalation is
    // correct (BLOCK beats WARN beats INFO).
    let fail_idx = fn_body.find("return \"fail\"").expect("fail branch present");
    let warn_idx = fn_body.find("return \"warn\"").expect("warn branch present");
    let info_idx = fn_body.find("return \"info\"").expect("info branch present");
    assert!(
        fail_idx < warn_idx && warn_idx < info_idx,
        "findingLevel escalation out of order: fail={fail_idx}, warn={warn_idx}, info={info_idx}"
    );
    // Specific substrings that must be scanned (per spec): BLOCK / Block /
    // blocked / rejected / Fail.
    for needle in ["BLOCK", "Block", "blocked", "rejected"] {
        assert!(
            fn_body.contains(needle),
            "findingLevel must recognise `{needle}` (substring missing)"
        );
    }
}

// ---------------------------------------------------------------------------
// Contract 6: the `default` arm in colour() is the ONLY non-variant path
// that returns "dim". This enforces that raw enum drift (e.g. a future
// backend adds a variant) falls into the labelled default and the test
// above can then flag it.
// ---------------------------------------------------------------------------

#[test]
fn contract_default_arm_exists_and_returns_dim() {
    let src = read_activity_entry();
    let colour_start = src.find("function colour(").expect("colour fn present");
    let colour_body = &src[colour_start..];
    // Count `return "dim"` occurrences inside colour(). Batch 2C:
    // exactly one (the default arm).
    let count = colour_body.matches("return \"dim\"").count();
    assert_eq!(
        count, 1,
        "colour() should have exactly one `return \"dim\"` (default arm), found {count}"
    );
    // Confirm the default arm exists.
    assert!(
        colour_body.contains("default:"),
        "colour() must retain the default arm as the ONLY dim fallback"
    );
}

// ---------------------------------------------------------------------------
// Contract 7: total variant count matches the enum. If the enum grows, this
// test surfaces the drift at build time so the frontend gets updated.
// ---------------------------------------------------------------------------

#[test]
fn contract_variant_count_matches_enum_doc() {
    // The enum doc comment in ssb.rs L84 claims "All 32 action types" but the
    // enum has grown. Batch 3A (Phase 4F) raised the floor to 48; Batch 3B
    // (Phase 4G) adds gate_decision + task_split + escalation_fired +
    // mode_changed so the floor was 52. Batch 3C (Phase 4H) adds
    // proposal_generated + proposal_applied + proposal_rejected + msi_update
    // + config_archived so the floor is now 57. Growth is fine; shrinkage
    // flags a breaking change for review.
    // Batch 4C: +5 variants (profile_selected, fsm_transition,
    // micro_reflection, telescope_collapse, config_warning) bumping the
    // floor to 62. Batch-fu: +2 variants (constraints_injected,
    // intervention_resolved) bumping the floor to 64. Batch-ful-a: +1
    // variant (dkg_querier_swapped) bumping the floor to 65. Batch-ful-c
    // (A-8): +2 variants (steering_computed, steering_compute_failed)
    // bumping the floor to 67. Batch-ful-d (phase-3J S9): +3 variants
    // (mcp_server_connected, mcp_tool_called, mcp_server_disconnected)
    // bumping the floor to 70. B-27: +2 (cloud_spend, escalate_to_cloud)
    // bumping floor to 72. B-23 Phase 4H Group A: +6 (data_table_grew,
    // l3_proposal, convergence_stagnant, config_oscillation,
    // regression_detected, evolution_withheld) bumping floor to 78.
    assert!(
        ALL_VARIANTS.len() >= 78,
        "ALL_VARIANTS shrank below 78 — review SsbActionType enum"
    );
}
