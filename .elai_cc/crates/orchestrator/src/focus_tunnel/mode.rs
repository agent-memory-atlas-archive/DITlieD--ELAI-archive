//! ModeSelector — deterministic FIM / DIFF / STRUCTURED routing for the
//! focus-tunnel path (B-28 E-3).
//!
//! Spec: `workflow/papers/focus-tunnel-design-interface.md`
//!   * L25-28: ModeSelector surface (StepSpec + ModelCapabilities + ExecutionMode
//!     -> GenerationMode).
//!   * L84-90: 5-rule precedence table (first match wins).
//!   * L137-138, L208-210: preferred_mode is advisory; orchestrator has FINAL SAY.
//!   * L211-212: mode_distribution metric MUST be tracked in SSB (we expose
//!     [`mode_distribution_for_payloads`] for the read-side aggregation).
//!
//! Two enums named "execution mode" exist in the orchestrator:
//!   * [`crate::cloud_mode::ExecMode`] (`Local | Cloud | Auto`) — PROVIDER routing.
//!   * [`crate::context_budget::ExecutionMode`] (`Interactive | Autonomous`) —
//!     BUDGET failure routing.
//! Rule 1 in the focus-tunnel spec refers to the former; this module wires
//! against [`crate::cloud_mode::ExecMode`] directly to avoid aliasing.
//!
//! No live FIM / DIFF execution lives here — see
//! [`crate::focus_tunnel::orchestrator`] for the per-mode wrappers and
//! [`crate::task_runner::drive_task`] for the integration site (one
//! [`ModeSelector::select`] call immediately before the executor dispatch
//! at the `baby_step_executor.execute(exec_ctx)` callsite).

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::cloud_mode::ExecMode;
use crate::context_budget::ModelFeatures;
use crate::research_tree::BabyStepSpec;
use crate::ssb::SsbPayload;

// ---------------------------------------------------------------------------
// Public value types
// ---------------------------------------------------------------------------

/// One of three generation modes the focus-tunnel routes between.
///
/// Serialized as snake_case strings (matching [`crate::ssb::SsbActionType`]
/// convention) so SSB rows and dashboard JSON share the wire form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationMode {
    /// Fill-In-the-Middle: raw code emission between Qwen FIM sentinels.
    /// Requires `ModelFeatures::supports_fim == true` and a single contiguous
    /// edit region.
    Fim,
    /// Unified-diff output (multi-region). Parsed by
    /// [`crate::focus_tunnel::DiffParser`] and replayed via
    /// [`crate::focus_tunnel::replay_hunks`].
    Diff,
    /// Default JSON-tool-call output. Mandatory for create_file actions,
    /// cloud execution, and any FIM/DIFF fallback.
    Structured,
}

/// Which deterministic rule from the spec's L84-90 table fired for this
/// decision. Captured in SSB so the auditor can verify the first-match-wins
/// ordering and the dashboard can render "rule: cloud_override" rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectionRule {
    /// Rule 1 (spec L85): `execution_mode == CLOUD` -> STRUCTURED.
    CloudOverride,
    /// Rule 2 (spec L86): `step.type == "create_file"` -> STRUCTURED.
    CreateFileOverride,
    /// Rule 3 (spec L87-88): edit_existing AND edit_region.is_some() AND
    /// `model.supports_fim` -> FIM.
    FimAllConditionsMet,
    /// Rule 4 (spec L89): edit_existing AND changes_count > 1 -> DIFF.
    MultiRegionDiff,
    /// Rule 5 (spec L90): else -> STRUCTURED.
    StructuredFallback,
    /// Tie-break promotion: when Rule 5 would fire AND a `preferred_mode`
    /// hint is present AND the hint is reachable under the active features
    /// (FIM requires supports_fim, DIFF requires multi-region capability),
    /// promote the mode to the hint. Captured as a distinct rule so the
    /// auditor sees the promotion event explicitly.
    HintedByPreferredMode,
}

/// Outcome of one [`ModeSelector::select`] call.
///
/// `reason` is a static label suitable for SSB / tracing; we keep it
/// `'static` so the value type stays `Copy`-friendly for the hot path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeSelection {
    pub mode: GenerationMode,
    pub rule_fired: SelectionRule,
    pub reason: &'static str,
    /// The advisory `preferred_mode` value present on the `StepContext` at
    /// decision time, if any. Preserved so the dashboard can render
    /// "designer hinted FIM, orchestrator chose STRUCTURED (rule_2)".
    pub preferred_was: Option<GenerationMode>,
}

/// The two action verbs the spec recognises on a step. `Unknown` is the
/// fallback for ambiguous task text — treated as `EditExisting` by Rule 5
/// after every more-specific rule misses (conservative; preserves
/// STRUCTURED unless FIM preconditions are independently met).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepAction {
    EditExisting,
    CreateFile,
    Unknown,
}

/// The `[start_line, end_line)` region a FIM-mode edit will splice into.
/// Spec L132 makes this MANDATORY for FIM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditRegion {
    pub start_line: usize,
    pub end_line: usize,
}

/// Input to [`ModeSelector::select`]. Built locally by
/// [`build_step_context_from`] from a [`BabyStepSpec`] using task-text
/// heuristics — keeps `mode.rs` decoupled from research_tree.rs schema drift.
///
/// `changes_count` defaults to 1 unless the heuristic finds multi-region
/// signals (e.g. "edit foo.rs and bar.rs", "in three places").
/// `preferred_mode` is the optional designer hint per spec L137-138.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepContext {
    pub action: StepAction,
    pub edit_region: Option<EditRegion>,
    pub changes_count: usize,
    pub preferred_mode: Option<GenerationMode>,
}

impl StepContext {
    /// Empty conservative default: unknown action, no region, 1 change,
    /// no hint. Falls through to STRUCTURED via Rule 5 for any model.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            action: StepAction::Unknown,
            edit_region: None,
            changes_count: 1,
            preferred_mode: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Selector
// ---------------------------------------------------------------------------

/// Stateless 5-rule selector. Construct once per task (cheap), call
/// [`ModeSelector::select`] per baby step.
#[derive(Debug, Default, Clone, Copy)]
pub struct ModeSelector;

impl ModeSelector {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Apply the spec's 5-rule precedence table.
    ///
    /// Evaluated in order, first match wins:
    ///   1. `exec == Cloud` -> STRUCTURED (rule [`SelectionRule::CloudOverride`]).
    ///   2. `step.action == CreateFile` -> STRUCTURED
    ///      (rule [`SelectionRule::CreateFileOverride`]).
    ///   3. edit_existing + edit_region.is_some() + supports_fim -> FIM
    ///      (rule [`SelectionRule::FimAllConditionsMet`]).
    ///   4. edit_existing + changes_count > 1 -> DIFF
    ///      (rule [`SelectionRule::MultiRegionDiff`]).
    ///   5. else -> STRUCTURED, with possible
    ///      [`SelectionRule::HintedByPreferredMode`] promotion when
    ///      `preferred_mode` is set AND reachable under the active features.
    #[must_use]
    pub fn select(
        &self,
        step: &StepContext,
        features: &ModelFeatures,
        exec: ExecMode,
    ) -> ModeSelection {
        let preferred = step.preferred_mode;

        // Rule 1 — CLOUD forces STRUCTURED, EXCEPT on an EditExisting step where DIFF
        // is the output-token-cheaper mode (H-3). CreateFile + non-edit cloud steps
        // still force Structured (the model can't emit a file path in DIFF; Rule 2).
        // The relax lets cloud EditExisting fall through to the FIM/DIFF rules.
        if matches!(exec, ExecMode::Cloud)
            && !matches!(step.action, StepAction::EditExisting)
        {
            return ModeSelection {
                mode: GenerationMode::Structured,
                rule_fired: SelectionRule::CloudOverride,
                reason: "rule_1_cloud_execution",
                preferred_was: preferred,
            };
        }

        // Rule 2 — create_file is always STRUCTURED (the model never
        // emits a file path in FIM/DIFF; spec L323).
        if matches!(step.action, StepAction::CreateFile) {
            return ModeSelection {
                mode: GenerationMode::Structured,
                rule_fired: SelectionRule::CreateFileOverride,
                reason: "rule_2_create_file",
                preferred_was: preferred,
            };
        }

        let is_edit = matches!(
            step.action,
            StepAction::EditExisting | StepAction::Unknown
        );

        // Rule 3 — edit_existing + region + supports_fim.
        if is_edit
            && matches!(step.action, StepAction::EditExisting)
            && step.edit_region.is_some()
            && features.supports_fim
            && step.changes_count <= 1
        {
            return ModeSelection {
                mode: GenerationMode::Fim,
                rule_fired: SelectionRule::FimAllConditionsMet,
                reason: "rule_3_fim_eligible",
                preferred_was: preferred,
            };
        }

        // Rule 4 — multi-region edit -> DIFF.
        if is_edit
            && matches!(step.action, StepAction::EditExisting)
            && step.changes_count > 1
        {
            return ModeSelection {
                mode: GenerationMode::Diff,
                rule_fired: SelectionRule::MultiRegionDiff,
                reason: "rule_4_multi_region",
                preferred_was: preferred,
            };
        }

        // Rule 5 — fallback. Optionally promote to preferred_mode when the
        // hint is reachable. B-28 WIRING-1 item D: hint promotion is GATED
        // on a concrete known action (`EditExisting` / `CreateFile`) — when
        // the action is `Unknown`, the orchestrator MUST fall through to
        // STRUCTURED regardless of the hint, because the more-specific
        // rules (Rule 2/3/4) never get a chance to gate it. Spec L84-90:
        // hint is advisory only; orchestrator retains final say.
        let action_allows_promotion = matches!(
            step.action,
            StepAction::EditExisting | StepAction::CreateFile
        );
        if action_allows_promotion {
            if let Some(hint) = preferred {
                let reachable = match hint {
                    GenerationMode::Structured => true,
                    GenerationMode::Fim => {
                        features.supports_fim && step.edit_region.is_some()
                    }
                    GenerationMode::Diff => step.changes_count > 1,
                };
                if reachable && !matches!(hint, GenerationMode::Structured) {
                    return ModeSelection {
                        mode: hint,
                        rule_fired: SelectionRule::HintedByPreferredMode,
                        reason: "rule_5_hint_promotion",
                        preferred_was: preferred,
                    };
                }
            }
        }

        ModeSelection {
            mode: GenerationMode::Structured,
            rule_fired: SelectionRule::StructuredFallback,
            reason: "rule_5_structured_fallback",
            preferred_was: preferred,
        }
    }
}

// ---------------------------------------------------------------------------
// Heuristic — derive a StepContext from a BabyStepSpec
// ---------------------------------------------------------------------------

/// Build a [`StepContext`] from a [`BabyStepSpec`] using only the task-text
/// today (no `edit_region` or `changes_count` fields exist on the spec yet —
/// see workflow/sessions/s-27436/context-package-E3.md ITEM 1).
///
/// Conservative defaults:
///   * `action`: `CreateFile` when the task text contains "create" + "file"
///     / "new file" cues; otherwise `EditExisting`. Empty / "make it work"
///     style tasks map to `EditExisting` so Rule 5 lands on STRUCTURED.
///   * `edit_region`: always `None` in E-3. Future spec amendments to
///     BabyStepSpec can populate it; until then FIM is unreachable from
///     this heuristic alone (and that is the documented E-3 behaviour).
///   * `changes_count`: 1 by default; bumped to N if the task text mentions
///     "multiple files" / "in N places" / "and" between filenames. Kept
///     simple — DIFF mode rarely fires from this heuristic, matching the
///     "DIFF unreachable in E-3 by default" decision in B-28-decisions.md.
///   * `preferred_mode`: `None` (BabyStepSpec.preferred_mode is deferred
///     to a follow-on phase per CLAIMED_FILES restriction; see decisions.md).
#[must_use]
pub fn build_step_context_from(spec: &BabyStepSpec) -> StepContext {
    let task = spec.task.to_ascii_lowercase();
    let action = classify_step_action(&task);
    let changes_count = if mentions_multiple_regions(&task) {
        2
    } else {
        1
    };
    StepContext {
        action,
        // B-28 WIRING-2 fix BLOCK 2b: forward the spec's `edit_region`
        // verbatim. When `Some`, FIM Rule 3 becomes reachable; when `None`
        // the conservative fallback path (Rule 5 -> STRUCTURED) is
        // preserved. The heuristic no longer manufactures regions —
        // callers must populate the field on BabyStepSpec to unlock FIM.
        edit_region: spec.edit_region,
        changes_count,
        // B-28 WIRING-1 item B: forward the spec's advisory `preferred_mode`
        // hint into the StepContext. Rule 5 still gates promotion on a
        // concrete `EditExisting` / `CreateFile` action (item D guard).
        preferred_mode: spec.preferred_mode,
    }
}

fn classify_step_action(lower_task: &str) -> StepAction {
    if lower_task.is_empty() {
        return StepAction::Unknown;
    }
    // Explicit creation cues. We require both a creation verb AND a "file"
    // / "module" / "crate" noun so phrasing like "create a test for foo()"
    // does not get mis-routed away from edit_existing.
    let creation_verbs = ["create", "add a new", "scaffold", "new file"];
    let creation_targets = ["file", "module", "crate", "package", "directory"];
    let has_verb = creation_verbs.iter().any(|v| lower_task.contains(v));
    let has_target = creation_targets.iter().any(|t| lower_task.contains(t));
    if has_verb && has_target {
        return StepAction::CreateFile;
    }
    // Explicit edit cue. "edit", "update", "fix", "refactor", "modify"
    // all map to EditExisting.
    let edit_verbs = ["edit", "update", "fix", "refactor", "modify", "rewrite"];
    if edit_verbs.iter().any(|v| lower_task.contains(v)) {
        return StepAction::EditExisting;
    }
    // Conservative default: EditExisting. Rule 5 lands on STRUCTURED.
    StepAction::EditExisting
}

fn mentions_multiple_regions(lower_task: &str) -> bool {
    // Crude but deterministic. "in N places" / "multiple regions" /
    // "and ... and" / "across files" are the cues. Avoids false positives
    // on "and" used as a normal conjunction by requiring an explicit
    // multi-region keyword.
    let cues = [
        "in multiple places",
        "in several places",
        "multiple regions",
        "across files",
        "throughout the file",
        "two places",
        "three places",
    ];
    cues.iter().any(|c| lower_task.contains(c))
}

// ---------------------------------------------------------------------------
// Read-side aggregation — mode_distribution
// ---------------------------------------------------------------------------

/// Aggregate a slice of [`SsbPayload`] references into a count-per-mode map.
///
/// Filters to [`SsbPayload::GenerationModeSelected`] rows only — other
/// variants are silently skipped so callers can pass an unfiltered SSB
/// payload stream. The returned map only contains modes with at least
/// one observation (callers fill 0s for missing keys).
///
/// Reader-side aggregation per the locked decision in
/// workflow/sessions/s-27436/context-package-E3.md ITEM 6: no extra in-memory
/// counter on `TaskRunner` — every SSB row carries the mode, so counting is
/// downstream-only.
#[must_use]
pub fn mode_distribution_for_payloads(
    payloads: &[&SsbPayload],
) -> FxHashMap<GenerationMode, u32> {
    let mut out: FxHashMap<GenerationMode, u32> = FxHashMap::default();
    for payload in payloads {
        if let SsbPayload::GenerationModeSelected(p) = payload {
            if let Some(mode) = parse_generation_mode_str(&p.mode) {
                *out.entry(mode).or_insert(0) += 1;
            }
        }
    }
    out
}

/// Parse the SSB wire-form mode string back into a [`GenerationMode`].
/// `None` on any unrecognised string so partially-corrupt SSB rows do not
/// poison the aggregation.
#[must_use]
pub fn parse_generation_mode_str(s: &str) -> Option<GenerationMode> {
    match s {
        "fim" => Some(GenerationMode::Fim),
        "diff" => Some(GenerationMode::Diff),
        "structured" => Some(GenerationMode::Structured),
        _ => None,
    }
}

/// SSB wire-form string for a [`GenerationMode`]. Inverse of
/// [`parse_generation_mode_str`]. Used by the task_runner emit site so the
/// payload field stays a `String` (avoids cycles with `ssb.rs`).
#[must_use]
pub fn generation_mode_str(mode: GenerationMode) -> &'static str {
    match mode {
        GenerationMode::Fim => "fim",
        GenerationMode::Diff => "diff",
        GenerationMode::Structured => "structured",
    }
}

/// SSB wire-form string for a [`SelectionRule`].
#[must_use]
pub fn selection_rule_str(rule: SelectionRule) -> &'static str {
    match rule {
        SelectionRule::CloudOverride => "cloud_override",
        SelectionRule::CreateFileOverride => "create_file_override",
        SelectionRule::FimAllConditionsMet => "fim_all_conditions_met",
        SelectionRule::MultiRegionDiff => "multi_region_diff",
        SelectionRule::StructuredFallback => "structured_fallback",
        SelectionRule::HintedByPreferredMode => "hinted_by_preferred_mode",
    }
}

/// SSB wire-form string for the active execution mode at decision time.
#[must_use]
pub fn exec_mode_str(exec: ExecMode) -> &'static str {
    match exec {
        ExecMode::Local => "local",
        ExecMode::Cloud => "cloud",
        ExecMode::Auto => "auto",
    }
}

// ---------------------------------------------------------------------------
// Unit tests — invariants the selector must always satisfy.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn features(supports_fim: bool) -> ModelFeatures {
        let mut f = ModelFeatures::none();
        f.supports_fim = supports_fim;
        f
    }

    #[test]
    fn rule_precedence_first_match_wins() {
        // CLOUD + every other rule's preconditions still routes STRUCTURED.
        let step = StepContext {
            action: StepAction::EditExisting,
            edit_region: Some(EditRegion { start_line: 1, end_line: 2 }),
            changes_count: 5,
            preferred_mode: Some(GenerationMode::Diff),
        };
        let out = ModeSelector::new().select(&step, &features(true), ExecMode::Cloud);
        assert_eq!(out.rule_fired, SelectionRule::CloudOverride);
        assert_eq!(out.mode, GenerationMode::Structured);
    }

    #[test]
    fn auto_execution_treated_as_local_for_routing() {
        // ExecMode::Auto means "start local, escalate on retry". From the
        // ModeSelector's standpoint at the decision instant that is Local.
        let step = StepContext {
            action: StepAction::EditExisting,
            edit_region: Some(EditRegion { start_line: 10, end_line: 20 }),
            changes_count: 1,
            preferred_mode: None,
        };
        let out = ModeSelector::new().select(&step, &features(true), ExecMode::Auto);
        assert_eq!(out.mode, GenerationMode::Fim);
        assert_eq!(out.rule_fired, SelectionRule::FimAllConditionsMet);
    }

    #[test]
    fn hint_promotion_only_when_reachable() {
        // Hint=FIM but no edit_region -> not reachable, Rule 5 -> STRUCTURED.
        let step = StepContext {
            action: StepAction::EditExisting,
            edit_region: None,
            changes_count: 1,
            preferred_mode: Some(GenerationMode::Fim),
        };
        let out = ModeSelector::new().select(&step, &features(true), ExecMode::Local);
        assert_eq!(out.mode, GenerationMode::Structured);
        assert_eq!(out.rule_fired, SelectionRule::StructuredFallback);
    }

    #[test]
    fn hint_promotion_when_reachable_diff() {
        // Hint=DIFF with multi-region edit but no region info: should still
        // not fire FIM, but DIFF is unreachable too (changes_count<=1).
        // Outcome: STRUCTURED, rule=StructuredFallback.
        let step = StepContext {
            action: StepAction::EditExisting,
            edit_region: None,
            changes_count: 1,
            preferred_mode: Some(GenerationMode::Diff),
        };
        let out = ModeSelector::new().select(&step, &features(true), ExecMode::Local);
        assert_eq!(out.mode, GenerationMode::Structured);
        assert_eq!(out.rule_fired, SelectionRule::StructuredFallback);
    }

    #[test]
    fn wire_form_strings_are_stable() {
        // Wire-form strings are part of the SSB CHECK constraint contract.
        // Lock them here so any rename forces a deliberate schema bump.
        assert_eq!(generation_mode_str(GenerationMode::Fim), "fim");
        assert_eq!(generation_mode_str(GenerationMode::Diff), "diff");
        assert_eq!(
            generation_mode_str(GenerationMode::Structured),
            "structured"
        );
        assert_eq!(
            selection_rule_str(SelectionRule::CloudOverride),
            "cloud_override"
        );
        assert_eq!(exec_mode_str(ExecMode::Local), "local");
        assert_eq!(exec_mode_str(ExecMode::Cloud), "cloud");
        assert_eq!(exec_mode_str(ExecMode::Auto), "auto");
    }

    #[test]
    fn parse_generation_mode_str_rejects_unknown() {
        assert_eq!(parse_generation_mode_str("fim"), Some(GenerationMode::Fim));
        assert_eq!(
            parse_generation_mode_str("structured"),
            Some(GenerationMode::Structured)
        );
        assert_eq!(parse_generation_mode_str("FIM"), None);
        assert_eq!(parse_generation_mode_str("unknown_mode"), None);
        assert_eq!(parse_generation_mode_str(""), None);
    }

    /// B-28 WIRING-1 item D inline regression: Unknown action MUST NOT
    /// receive Rule 5 hint promotion even when the hint is reachable.
    #[test]
    fn rule_5_guard_blocks_unknown_action_with_reachable_hint() {
        let selector = ModeSelector::new();
        let mut features = ModelFeatures::none();
        features.supports_fim = true;
        let step = StepContext {
            action: StepAction::Unknown,
            edit_region: Some(EditRegion {
                start_line: 1,
                end_line: 4,
            }),
            changes_count: 1,
            preferred_mode: Some(GenerationMode::Fim),
        };
        let out = selector.select(&step, &features, ExecMode::Local);
        assert_eq!(out.mode, GenerationMode::Structured);
        assert_eq!(out.rule_fired, SelectionRule::StructuredFallback);
    }
}
