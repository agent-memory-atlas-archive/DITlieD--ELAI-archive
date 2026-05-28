//! E-8: LPO `<edit>` / `<frozen>` tag enforcement on BabyStepSpec blocks.
//!
//! Per arXiv:2504.20355 (LPO, "Localized Prompt Optimization"): only the
//! spans wrapped in `<edit>...</edit>` are optimizer-mutable; spans wrapped
//! in `<frozen>...</frozen>` are immutable. ELAI applies this discipline
//! at the BabyStepSpec block level. The 9 block-kinds split as:
//!
//!   editable: TASK, GROUNDING, PRE_GATE, TDD, ROLE_PROMPT_BODY
//!   frozen:   DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, EXIT_CRITERIA
//!
//! This module owns:
//!   * [`BlockKind`] -- closed taxonomy of 9 block kinds.
//!   * [`BabyStepSpec`] -- typed value object carrying one
//!     `(BlockKind, body)` tuple per block. Constructed by the
//!     babystep-author agent; consumed by the optimizer admit path.
//!   * [`EditableSpans`] / [`FrozenSpans`] -- typed projection of the
//!     spec into the two LPO categories.
//!   * [`editable_spans`] -- the projection function. Pure / total /
//!     no I/O.
//!   * [`assert_no_frozen_edit`] -- predicate the optimizer admit path
//!     calls before accepting a mutation; refuses any rewrite whose
//!     diff overlaps a frozen span.
//!
//! Why a focused new module instead of editing baby_step.rs:
//!   Per workflow/reasoning/E-8-decisions.md DECISION-4, the file
//!   `crates/orchestrator/src/baby_step_spec.rs` named by E-8 plan
//!   task 2 does NOT exist (only `grammars/baby_step_spec.ebnf`). The
//!   LPO span-level discipline is a pure-text predicate over closed
//!   block-kinds, so a focused new module is ~150 LOC and self-
//!   contained -- no upstream dep on the EBNF grammar / parser.
//!
//! No `.unwrap()` / `.expect()` outside test code (CLAUDE.md hard rule).

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Closed taxonomy of the 9 BabyStepSpec block-kinds.
///
/// The first 5 are LPO-editable (role-prompt body + task description +
/// grounding context + pre-gate notes + TDD scaffold). The last 4 are
/// LPO-frozen (deterministic checks + acceptance criteria + integration
/// notes + exit criteria) -- mutating these would let the optimizer
/// game the verification surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockKind {
    /// `TASK:` block -- editable.
    Task,
    /// `GROUNDING:` block -- editable.
    Grounding,
    /// `PRE_GATE:` block -- editable.
    PreGate,
    /// `TDD:` scaffold block -- editable.
    Tdd,
    /// Role-prompt body / few-shot examples -- editable.
    RolePromptBody,
    /// `DETERMINISTIC_CHECK:` block -- FROZEN.
    DeterministicCheck,
    /// `ACCEPTANCE:` block -- FROZEN.
    Acceptance,
    /// `INTEGRATION:` block -- FROZEN.
    Integration,
    /// `EXIT_CRITERIA:` block -- FROZEN.
    ExitCriteria,
}

impl BlockKind {
    /// `true` IFF the block is LPO-editable.
    #[must_use]
    pub fn is_editable(&self) -> bool {
        matches!(
            self,
            Self::Task
                | Self::Grounding
                | Self::PreGate
                | Self::Tdd
                | Self::RolePromptBody
        )
    }

    /// `true` IFF the block is LPO-frozen.
    #[must_use]
    pub fn is_frozen(&self) -> bool {
        !self.is_editable()
    }

    /// Snake-case wire tag.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Grounding => "grounding",
            Self::PreGate => "pre_gate",
            Self::Tdd => "tdd",
            Self::RolePromptBody => "role_prompt_body",
            Self::DeterministicCheck => "deterministic_check",
            Self::Acceptance => "acceptance",
            Self::Integration => "integration",
            Self::ExitCriteria => "exit_criteria",
        }
    }

    /// All 9 block-kinds. Used by tests + the LPO span-enumeration
    /// surface.
    #[must_use]
    pub const fn all() -> [Self; 9] {
        [
            Self::Task,
            Self::Grounding,
            Self::PreGate,
            Self::Tdd,
            Self::RolePromptBody,
            Self::DeterministicCheck,
            Self::Acceptance,
            Self::Integration,
            Self::ExitCriteria,
        ]
    }
}

/// One LPO span -- a `(kind, body)` tuple. Bodies are stored as owned
/// `String` so the projection survives transport across the
/// orchestrator/optimizer boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecBlock {
    pub kind: BlockKind,
    pub body: String,
}

/// Typed BabyStepSpec carrying one [`SpecBlock`] per block-kind. The
/// vec is ordered; consumers iterate in declaration order.
///
/// Why not a `HashMap<BlockKind, String>`: babystep-author may emit
/// multiple blocks of the same kind (e.g. two GROUNDING blocks) when
/// the task draws on heterogeneous sources. Vec preserves order +
/// multiplicity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BabyStepSpec {
    pub blocks: Vec<SpecBlock>,
}

impl BabyStepSpec {
    /// Construct from a list of `(kind, body)` tuples.
    #[must_use]
    pub fn from_blocks(blocks: Vec<SpecBlock>) -> Self {
        Self { blocks }
    }
}

/// LPO-editable projection of a [`BabyStepSpec`]. Spans the optimizer
/// is permitted to mutate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditableSpans {
    pub spans: Vec<SpecBlock>,
}

/// LPO-frozen projection of a [`BabyStepSpec`]. Spans the optimizer
/// must NOT mutate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrozenSpans {
    pub spans: Vec<SpecBlock>,
}

/// Project the spec into editable + frozen halves. Total / pure / no
/// allocation aside from the two Vec clones.
#[must_use]
pub fn editable_spans(spec: &BabyStepSpec) -> EditableSpans {
    let spans = spec
        .blocks
        .iter()
        .filter(|b| b.kind.is_editable())
        .cloned()
        .collect();
    EditableSpans { spans }
}

/// Symmetric -- frozen projection.
#[must_use]
pub fn frozen_spans(spec: &BabyStepSpec) -> FrozenSpans {
    let spans = spec
        .blocks
        .iter()
        .filter(|b| b.kind.is_frozen())
        .cloned()
        .collect();
    FrozenSpans { spans }
}

/// LPO violation: optimizer attempted to mutate a frozen block.
#[derive(Debug, Error, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LpoViolation {
    /// Optimizer proposed a mutation whose target block is FROZEN.
    #[error("frozen block `{kind}` cannot be edited (LPO arXiv:2504.20355)")]
    FrozenBlockMutated { kind: String },
    /// Optimizer proposed a mutation against a block-kind that was
    /// removed from the spec between project + admit.
    #[error("block `{kind}` not present in original spec")]
    BlockMissing { kind: String },
}

/// Predicate: refuse mutation IFF its target is a frozen block.
///
/// Returns `Ok(())` when `target_kind` is editable AND present in
/// `spec`. Returns `Err(LpoViolation::FrozenBlockMutated)` when
/// `target_kind` is frozen, regardless of presence. Returns
/// `Err(LpoViolation::BlockMissing)` when `target_kind` is editable
/// but absent.
pub fn assert_no_frozen_edit(
    spec: &BabyStepSpec,
    target_kind: BlockKind,
) -> Result<(), LpoViolation> {
    if target_kind.is_frozen() {
        return Err(LpoViolation::FrozenBlockMutated {
            kind: target_kind.as_str().to_owned(),
        });
    }
    let present = spec.blocks.iter().any(|b| b.kind == target_kind);
    if !present {
        return Err(LpoViolation::BlockMissing {
            kind: target_kind.as_str().to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_spec() -> BabyStepSpec {
        BabyStepSpec::from_blocks(
            BlockKind::all()
                .iter()
                .map(|k| SpecBlock {
                    kind: *k,
                    body: format!("body-{}", k.as_str()),
                })
                .collect(),
        )
    }

    #[test]
    fn deterministic_check_block_is_frozen() {
        assert!(BlockKind::DeterministicCheck.is_frozen());
        assert!(!BlockKind::DeterministicCheck.is_editable());
    }

    #[test]
    fn all_four_frozen_blocks_identified() {
        let spec = full_spec();
        let frozen = frozen_spans(&spec);
        let kinds: Vec<BlockKind> = frozen.spans.iter().map(|s| s.kind).collect();
        assert!(kinds.contains(&BlockKind::DeterministicCheck));
        assert!(kinds.contains(&BlockKind::Acceptance));
        assert!(kinds.contains(&BlockKind::Integration));
        assert!(kinds.contains(&BlockKind::ExitCriteria));
        assert_eq!(frozen.spans.len(), 4);
    }

    #[test]
    fn all_five_editable_blocks_identified() {
        let spec = full_spec();
        let editable = editable_spans(&spec);
        let kinds: Vec<BlockKind> = editable.spans.iter().map(|s| s.kind).collect();
        assert!(kinds.contains(&BlockKind::Task));
        assert!(kinds.contains(&BlockKind::Grounding));
        assert!(kinds.contains(&BlockKind::PreGate));
        assert!(kinds.contains(&BlockKind::Tdd));
        assert!(kinds.contains(&BlockKind::RolePromptBody));
        assert_eq!(editable.spans.len(), 5);
    }

    #[test]
    fn editable_block_passes_no_frozen_assertion() {
        let spec = full_spec();
        let r = assert_no_frozen_edit(&spec, BlockKind::Task);
        assert!(r.is_ok(), "Task is editable; mutation must be permitted");
    }

    #[test]
    fn frozen_block_fails_no_frozen_assertion() {
        let spec = full_spec();
        let r = assert_no_frozen_edit(&spec, BlockKind::DeterministicCheck);
        match r {
            Err(LpoViolation::FrozenBlockMutated { kind }) => {
                assert_eq!(kind, "deterministic_check");
            }
            other => panic!("expected FrozenBlockMutated, got {other:?}"),
        }
    }

    #[test]
    fn missing_block_fails_with_block_missing() {
        let spec = BabyStepSpec::from_blocks(vec![SpecBlock {
            kind: BlockKind::Task,
            body: "x".into(),
        }]);
        let r = assert_no_frozen_edit(&spec, BlockKind::Grounding);
        match r {
            Err(LpoViolation::BlockMissing { kind }) => assert_eq!(kind, "grounding"),
            other => panic!("expected BlockMissing, got {other:?}"),
        }
    }

    #[test]
    fn editable_plus_frozen_covers_all_nine_kinds() {
        let spec = full_spec();
        let e = editable_spans(&spec);
        let f = frozen_spans(&spec);
        assert_eq!(e.spans.len() + f.spans.len(), BlockKind::all().len());
    }

    #[test]
    fn as_str_round_trip_per_kind() {
        for k in BlockKind::all() {
            let tag = k.as_str();
            assert!(!tag.is_empty());
        }
    }
}
