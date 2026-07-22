//! E-25 claude-code-architecture catalog slice -- the 9 high-leverage
//! Claude Code harness primitives ELAI ports as a substrate catalog,
//! alongside the 7-anti-pattern guard discriminants surfaced via
//! [`ClaudeCodeArchitectureEventKind`].
//!
//! Sibling slice: see [`crate::architecture::saas_archetype`] (E-11). This
//! module follows the same wire-dark substrate pattern: a canonical
//! taxonomy enum + a snake_case wire label + a Pattern-2 single-variant
//! SSB action_type (`ClaudeCodeArchitectureEvent`) with a per-primitive
//! discriminator on the payload. Downstream consumers (concrete compaction
//! ladder, skill templating engine, prompt-decision hook, opusplan
//! substate overlay, session-fork CLI, edit-checkpoint snapshot store,
//! bash-complexity / approval-fatigue / sandbox-template guards) ride on
//! top of this substrate and are tracked as future follow-ups in
//! `workflow/reasoning/E-25-decisions.md`.
//!
//! Scope-lean rationale:
//!   * Plan E-25_claude_code_architecture.md proposed 6 implementation
//!     modules + 7 anti-pattern guards + 3 dashboard panels. The dispatch
//!     re-cast E-25 as an architecture-reference CATALOG (mirrors E-11
//!     saas_archetype slice) so the substrate lands atomically and the
//!     concrete implementation modules can be reviewed individually as
//!     follow-up plans. NO dashboard panels, NO cli::fork module, NO
//!     edit_checkpoints.rs, NO compaction_ladder.rs in this commit.
//!   * Primitive 2 (deferred MCP tool descriptions) is DESCOPED entirely
//!     because the underlying lazy_schemas mechanism (E-22 Task 3 / C-35)
//!     is unshipped. Adding the catalog entry now would lie about ELAI's
//!     surface; the entry will be added in a follow-up once E-22 / C-35
//!     ships. See E-25-decisions.md DECISION-3.
//!   * The plan's `CompactionResult { messages, freed_tokens,
//!     tiers_applied, cache_preserved_bytes }` collides with the existing
//!     [`crate::compact::CompactionResult`] (different fields). The
//!     ladder-result struct is RENAMED to [`ClaudeCodeLadderResult`] in
//!     this catalog so both names coexist unambiguously.
//!
//! Wire-dark posture: the SSB emission surface
//! [`crate::ssb::SsbActionType::ClaudeCodeArchitectureEvent`] is gated by
//! the [`ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR`] environment variable. The
//! taxonomy enum + ladder-result struct + tier-id enum are always
//! available (in-process callers can consume them without touching the
//! SSB).
//!
//! Public surface:
//!   * [`ClaudeCodeArchPrimitive`] -- the 9-primitive taxonomy
//!   * [`ALL_CLAUDE_CODE_PRIMITIVES`] -- canonical iteration order
//!   * [`ClaudeCodeArchitectureEventKind`] -- 9-arm SSB discriminator
//!   * [`ClaudeCodeArchitectureEventPayload`] -- SSB payload struct
//!   * [`ClaudeCodeLadderResult`] -- renamed compaction-ladder return
//!   * [`CompactionTierId`] -- T1..T6 ladder-tier reference enum
//!   * [`ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR`] -- env var name
//!   * [`ssb_emission_enabled`] -- wire-dark gate
//!
//! Source grounding:
//!   * karanprasad.com/blog/how-claude-code-actually-works-reverse-
//!     engineering-512k-lines -- 6-tier compaction ladder + 98.4%
//!     deterministic infra split + cache_edits server-side parameter
//!   * code.claude.com/docs/en/how-claude-code-works -- official 6-tier
//!     compaction reference + checkpoints snapshot-before-edit +
//!     --fork-session / /branch
//!   * code.claude.com/docs/en/skills -- `!`cmd`` shell injection + live
//!     reload of SKILL.md
//!   * code.claude.com/docs/en/hooks -- `"type": "prompt"` LLM-decision
//!     hook handler
//!   * code.claude.com/docs/en/model-config -- opusplan alias (Opus for
//!     plan, Sonnet for execute)
//!   * github.com/VILA-Lab/Dive-into-Claude-Code -- pre-trust execution
//!     window + per-subcommand >50 bypass + approval-fatigue masking
//!   * infoq.com/news/2026/05/anthropic-claude-code-postmortem -- 6-week
//!     undetected quality regression (telemetry-guard rationale)
//!
//! All blog grounding is MEDIUM-reliability per source-tiers.md; the
//! code.claude.com/docs URLs are HIGH (official). Per CLAUDE.md
//! grounded-by-default rule, the WebFetch verification of each blog at
//! plan-entry time is recorded in
//! `workflow/sessions/s-25120/completion-drive/exploration/
//! E-25_claude_code_architecture-exploration.md`.

use serde::{Deserialize, Serialize};

/// Canonical claude-code-architecture primitive taxonomy. Nine variants
/// covering the high-leverage harness primitives ELAI ports, minus the
/// deferred MCP tool-description primitive that is descoped pending the
/// E-22 Task 3 / C-35 lazy_schemas substrate (see module-level doc).
///
/// snake_case wire discriminants are LOCKED. Adding a 10th primitive
/// requires bumping the catalog HERE and adding a matching arm to
/// [`ClaudeCodeArchitectureEventKind`] AND updating
/// `crates/contracts/tests/` coverage when those follow-up contracts
/// land.
///
/// snake_case wire discriminants:
///   * `compaction_ladder` -- 6-tier compaction ladder (Budget Reduction
///     / Snip / Microcompact / Context Collapse / Auto-Compact /
///     Narrative-Summary) layered above [`crate::compact::compact_session`].
///   * `skill_templating` -- `!`cmd`` shell-injection blocks in SKILL.md
///     body + filesystem-watched live reload via `notify` crate.
///   * `prompt_handler` -- LLM-decision hook handler (Allow / Deny / Ask)
///     called via the per-role runtime (defaults to local Qwen 3B sidecar).
///   * `opusplan_overlay` -- substate-overlay on
///     [`crate::runtime_orchestrator::RuntimeOrchestrator`] routing the
///     `assistant_primary` role to a planner-runtime during planning
///     FSM states and an executor-runtime during execution states.
///   * `session_fork` -- `elai resume --fork <new_session_id>` clones
///     SESSION_HANDOFF + memory-bank + FSM state into a new session
///     directory; original session untouched; `forked_from` metadata
///     recorded.
///   * `edit_checkpoints` -- pre-edit snapshot store under
///     `.elai_cc/.session-checkpoints/{session_id}/{seq}/{path}` with
///     blake3 integrity hashing and `elai checkpoint rewind --steps N`
///     restore path.
///   * `bash_complexity_guard` -- BLOCK Bash commands with >50
///     subcommands (rejects the Claude Code per-subcommand parsing
///     safety-bypass anti-pattern per VILA-Lab analysis).
///   * `approval_fatigue_guard` -- if auto-approval rate exceeds 80%
///     over a rolling 50-decision window, emit telemetry + force
///     explicit user reconfirmation on the next prompt.
///   * `sandbox_template_guard` -- if microsandbox is unavailable when
///     a skill-template `!`cmd`` block executes, emit telemetry warning
///     and apply the `.elai_cc/.elai/state/` process-jail fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeCodeArchPrimitive {
    /// 6-tier compaction ladder (Primitive 1).
    CompactionLadder,
    /// `!`cmd`` skill templating + live reload (Primitive 3).
    SkillTemplating,
    /// LLM-decision hook handler (Primitive 4).
    PromptHandler,
    /// opusplan plan-vs-execute substate overlay (Primitive 5).
    OpusplanOverlay,
    /// Filesystem-level session fork (Primitive 6a).
    SessionFork,
    /// Pre-edit snapshot store + rewind (Primitive 6b).
    EditCheckpoints,
    /// Bash >50-subcommand BLOCK guard (Primitive 7b).
    BashComplexityGuard,
    /// Auto-approval-rate >80% reconfirmation guard (Primitive 7c).
    ApprovalFatigueGuard,
    /// Skill-template unsandboxed-fallback warning guard (Primitive 7
    /// sandbox warn sub-clause).
    SandboxTemplateGuard,
}

/// All 9 [`ClaudeCodeArchPrimitive`] discriminants in declaration order.
/// Iteration surface for tests + future catalog traversal sites.
pub const ALL_CLAUDE_CODE_PRIMITIVES: &[ClaudeCodeArchPrimitive] = &[
    ClaudeCodeArchPrimitive::CompactionLadder,
    ClaudeCodeArchPrimitive::SkillTemplating,
    ClaudeCodeArchPrimitive::PromptHandler,
    ClaudeCodeArchPrimitive::OpusplanOverlay,
    ClaudeCodeArchPrimitive::SessionFork,
    ClaudeCodeArchPrimitive::EditCheckpoints,
    ClaudeCodeArchPrimitive::BashComplexityGuard,
    ClaudeCodeArchPrimitive::ApprovalFatigueGuard,
    ClaudeCodeArchPrimitive::SandboxTemplateGuard,
];

impl ClaudeCodeArchPrimitive {
    /// Canonical snake_case wire label. Stable across versions; used in
    /// SSB payloads + workflow/papers/ slug stems if a per-primitive
    /// research note ships.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::CompactionLadder => "compaction_ladder",
            Self::SkillTemplating => "skill_templating",
            Self::PromptHandler => "prompt_handler",
            Self::OpusplanOverlay => "opusplan_overlay",
            Self::SessionFork => "session_fork",
            Self::EditCheckpoints => "edit_checkpoints",
            Self::BashComplexityGuard => "bash_complexity_guard",
            Self::ApprovalFatigueGuard => "approval_fatigue_guard",
            Self::SandboxTemplateGuard => "sandbox_template_guard",
        }
    }

    /// Inverse of [`ClaudeCodeArchPrimitive::as_snake_case`]. Returns
    /// `None` for unknown discriminants so SSB consumers can fail closed
    /// on bad input.
    #[must_use]
    pub fn from_snake_case(s: &str) -> Option<Self> {
        match s {
            "compaction_ladder" => Some(Self::CompactionLadder),
            "skill_templating" => Some(Self::SkillTemplating),
            "prompt_handler" => Some(Self::PromptHandler),
            "opusplan_overlay" => Some(Self::OpusplanOverlay),
            "session_fork" => Some(Self::SessionFork),
            "edit_checkpoints" => Some(Self::EditCheckpoints),
            "bash_complexity_guard" => Some(Self::BashComplexityGuard),
            "approval_fatigue_guard" => Some(Self::ApprovalFatigueGuard),
            "sandbox_template_guard" => Some(Self::SandboxTemplateGuard),
            _ => None,
        }
    }

    /// Human-readable display label for CLI / future dashboard surfaces.
    /// Stable because downstream hand-authored mirrors key on these.
    #[must_use]
    pub fn display_label(self) -> &'static str {
        match self {
            Self::CompactionLadder => "Compaction Ladder",
            Self::SkillTemplating => "Skill Templating",
            Self::PromptHandler => "Prompt Handler",
            Self::OpusplanOverlay => "Opusplan Overlay",
            Self::SessionFork => "Session Fork",
            Self::EditCheckpoints => "Edit Checkpoints",
            Self::BashComplexityGuard => "Bash Complexity Guard",
            Self::ApprovalFatigueGuard => "Approval Fatigue Guard",
            Self::SandboxTemplateGuard => "Sandbox Template Guard",
        }
    }
}

/// E-25 Pattern-2 24th application -- fire-point discriminator carried by
/// [`ClaudeCodeArchitectureEventPayload`].
///
/// Collapses 9 originally-proposed SSB variants (one per primitive in
/// [`ClaudeCodeArchPrimitive`]) into one [`crate::ssb::SsbActionType`]
/// variant + one 9-arm discriminator per the Pattern-2 cascade discipline
/// established by E-7 (DarwinianGenerationCompleted) and reinforced by
/// E-55 (RewardLadderEvent) / E-11 (SaasPriorArtEvent) / E-22
/// (GithubTrendingEvent) / E-23 (DeveloperDiscourseEvent).
///
/// Per-arm fire-point semantics:
///   * `CompactionLadderTierApplied` -- a future compaction-ladder run
///     applied one of the 6 tiers and freed N tokens; carries
///     `tier_id` (T1..T6) + freed_tokens + wall_ms in `details_json`.
///   * `SkillTemplateExecuted` -- a SKILL.md `!`cmd`` block was scanned
///     and its stdout substituted before the skill content reached the
///     model.
///   * `PromptDecisionMade` -- the LLM-decision hook returned
///     Allow / Deny(reason) / Ask(reason); discriminant + reason in
///     `details_json`.
///   * `OpusplanOverlayActivated` -- a runtime-substate overlay rerouted
///     `assistant_primary` to a planner-runtime or executor-runtime;
///     `details_json` carries fsm_state + role + chosen runtime label.
///   * `SessionForked` -- `elai resume --fork` cloned a session into a
///     new directory; `details_json` carries
///     `forked_from` + new session id + memory-bank byte count.
///   * `EditCheckpointSnapshotted` -- pre-edit snapshot written under
///     `.elai_cc/.session-checkpoints/`; carries blake3 prefix + path.
///   * `BashComplexityBlocked` -- a Bash command with >50 subcommands
///     was rejected; carries subcommand count.
///   * `ApprovalFatigueDetected` -- rolling auto-approval rate exceeded
///     the configured threshold; carries rate + window size + threshold.
///   * `SandboxTemplateWarned` -- a skill-template `!`cmd`` block ran
///     without microsandbox; jail fallback applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaudeCodeArchitectureEventKind {
    /// Compaction-ladder tier applied + freed tokens recorded.
    CompactionLadderTierApplied,
    /// Skill `!`cmd`` block scanned + substituted into the skill content.
    SkillTemplateExecuted,
    /// LLM-decision hook returned Allow / Deny / Ask.
    PromptDecisionMade,
    /// Runtime substate overlay rerouted a role to planner / executor.
    OpusplanOverlayActivated,
    /// `elai resume --fork` cloned a session.
    SessionForked,
    /// Pre-edit snapshot persisted with blake3 hash.
    EditCheckpointSnapshotted,
    /// Bash command rejected for >50 subcommands.
    BashComplexityBlocked,
    /// Auto-approval rate exceeded the configured threshold.
    ApprovalFatigueDetected,
    /// Skill-template `!`cmd`` block ran outside microsandbox.
    SandboxTemplateWarned,
}

impl ClaudeCodeArchitectureEventKind {
    /// Canonical snake_case wire label. Stable across versions.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CompactionLadderTierApplied => "compaction_ladder_tier_applied",
            Self::SkillTemplateExecuted => "skill_template_executed",
            Self::PromptDecisionMade => "prompt_decision_made",
            Self::OpusplanOverlayActivated => "opusplan_overlay_activated",
            Self::SessionForked => "session_forked",
            Self::EditCheckpointSnapshotted => "edit_checkpoint_snapshotted",
            Self::BashComplexityBlocked => "bash_complexity_blocked",
            Self::ApprovalFatigueDetected => "approval_fatigue_detected",
            Self::SandboxTemplateWarned => "sandbox_template_warned",
        }
    }

    /// Reverse-map a wire label to the matching kind. Returns `None`
    /// for unknown labels so SSB readers can fail closed.
    #[must_use]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "compaction_ladder_tier_applied" => Some(Self::CompactionLadderTierApplied),
            "skill_template_executed" => Some(Self::SkillTemplateExecuted),
            "prompt_decision_made" => Some(Self::PromptDecisionMade),
            "opusplan_overlay_activated" => Some(Self::OpusplanOverlayActivated),
            "session_forked" => Some(Self::SessionForked),
            "edit_checkpoint_snapshotted" => Some(Self::EditCheckpointSnapshotted),
            "bash_complexity_blocked" => Some(Self::BashComplexityBlocked),
            "approval_fatigue_detected" => Some(Self::ApprovalFatigueDetected),
            "sandbox_template_warned" => Some(Self::SandboxTemplateWarned),
            _ => None,
        }
    }

    /// Map a fire-point kind back to the [`ClaudeCodeArchPrimitive`] it
    /// belongs to. Used by downstream consumers that want the primitive
    /// label without re-deriving it from the kind.
    #[must_use]
    pub fn primitive(self) -> ClaudeCodeArchPrimitive {
        match self {
            Self::CompactionLadderTierApplied => ClaudeCodeArchPrimitive::CompactionLadder,
            Self::SkillTemplateExecuted => ClaudeCodeArchPrimitive::SkillTemplating,
            Self::PromptDecisionMade => ClaudeCodeArchPrimitive::PromptHandler,
            Self::OpusplanOverlayActivated => ClaudeCodeArchPrimitive::OpusplanOverlay,
            Self::SessionForked => ClaudeCodeArchPrimitive::SessionFork,
            Self::EditCheckpointSnapshotted => ClaudeCodeArchPrimitive::EditCheckpoints,
            Self::BashComplexityBlocked => ClaudeCodeArchPrimitive::BashComplexityGuard,
            Self::ApprovalFatigueDetected => ClaudeCodeArchPrimitive::ApprovalFatigueGuard,
            Self::SandboxTemplateWarned => ClaudeCodeArchPrimitive::SandboxTemplateGuard,
        }
    }
}

/// E-25 (SCHEMA_VERSION 172) -- payload for
/// [`crate::ssb::SsbActionType::ClaudeCodeArchitectureEvent`].
///
/// Wire-dark default; emission gated by
/// [`ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR`]. Carries the 9-arm discriminator
/// + the canonical primitive label + an optional session id + a
/// free-form `details_json` so each kind's per-firing scalars (tier id,
/// freed tokens, decision reason, fsm-state label, blake3 prefix,
/// subcommand count, approval rate, jail path) can ride along without
/// per-kind payload structs (Pattern-2 single-variant scope-lean
/// rationale; mirrors `RewardLadderEventPayload` / `SaasPriorArtEventPayload`
/// shape exactly).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaudeCodeArchitectureEventPayload {
    /// Fire-point discriminator. See [`ClaudeCodeArchitectureEventKind`].
    pub kind: ClaudeCodeArchitectureEventKind,
    /// Canonical primitive label this fire-point belongs to. Derivable
    /// from [`ClaudeCodeArchitectureEventKind::primitive`] but persisted
    /// for cheap SSB filters.
    pub primitive: ClaudeCodeArchPrimitive,
    /// Optional canonical session id (the substrate emits per-session
    /// rather than per-task; some primitives such as
    /// `ApprovalFatigueDetected` aggregate across sessions and may set
    /// this to `None`).
    pub session_id: Option<String>,
    /// Free-form per-kind context (e.g. tier_id, freed_tokens,
    /// decision_reason, fsm_state, runtime_label, forked_from,
    /// blake3_prefix, subcommand_count, approval_rate). Serialized as
    /// JSON so future kinds can extend without a schema bump.
    pub details_json: serde_json::Value,
    /// Fire-point wall-clock (millis since UNIX_EPOCH).
    pub ts_ms: i64,
}

/// E-25 reference enum -- the 6 compaction-ladder tier ids per
/// karanprasad.com reverse-engineering of Claude Code's 512K-LOC
/// harness. Persisted in [`ClaudeCodeArchitectureEventPayload::details_json`]
/// as the `tier_id` field under
/// [`ClaudeCodeArchitectureEventKind::CompactionLadderTierApplied`]
/// fire-points. The concrete ladder implementation is a follow-up plan;
/// the enum lives here so the contract sketch is stable now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompactionTierId {
    /// T1 -- whitespace strip + dedupe identical tool calls.
    BudgetReduction,
    /// T2 -- drop oldest tool-result payloads >2KB while keeping
    /// summaries.
    Snip,
    /// T3 -- summarize 3 oldest user/assistant pairs into one bullet
    /// list per pair.
    Microcompact,
    /// T4 -- fold all tool results older than 5 turns into a single
    /// "older work" summary.
    ContextCollapse,
    /// T5 -- invoke sidecar with a "summarize preserving X" prompt.
    AutoCompact,
    /// T6 -- last-resort full re-narrate by sidecar.
    NarrativeSummary,
}

impl CompactionTierId {
    /// Canonical snake_case wire label.
    #[must_use]
    pub fn as_snake_case(self) -> &'static str {
        match self {
            Self::BudgetReduction => "budget_reduction",
            Self::Snip => "snip",
            Self::Microcompact => "microcompact",
            Self::ContextCollapse => "context_collapse",
            Self::AutoCompact => "auto_compact",
            Self::NarrativeSummary => "narrative_summary",
        }
    }

    /// Stable ordinal in the cheapest-first ladder traversal order. T1
    /// = 1, T6 = 6. Downstream consumers (concrete ladder impl + future
    /// dashboard tier-visualization panel) key on this.
    #[must_use]
    pub fn ordinal(self) -> u8 {
        match self {
            Self::BudgetReduction => 1,
            Self::Snip => 2,
            Self::Microcompact => 3,
            Self::ContextCollapse => 4,
            Self::AutoCompact => 5,
            Self::NarrativeSummary => 6,
        }
    }
}

/// All 6 [`CompactionTierId`] discriminants in canonical cheapest-first
/// ladder traversal order. Iteration surface for future ladder
/// implementation + tests.
pub const ALL_COMPACTION_TIERS: &[CompactionTierId] = &[
    CompactionTierId::BudgetReduction,
    CompactionTierId::Snip,
    CompactionTierId::Microcompact,
    CompactionTierId::ContextCollapse,
    CompactionTierId::AutoCompact,
    CompactionTierId::NarrativeSummary,
];

/// E-25 reference struct -- the rename of the plan's proposed
/// `orchestrator::compaction_ladder::CompactionResult` to avoid the hard
/// name collision with the existing [`crate::compact::CompactionResult`]
/// (different fields, different consumer surface). The plan's struct
/// shape is preserved verbatim; only the type name changes.
///
/// Concrete compaction-ladder impl is a follow-up; this struct lives in
/// the catalog so the public contract is locked from the substrate
/// commit forward.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeCodeLadderResult {
    /// Post-ladder messages serialized as a JSON value so this catalog
    /// struct does NOT depend on `crate::providers::types::Message` (the
    /// substrate is meant to be referenceable by contract-only callers).
    /// The concrete ladder implementation will adapt
    /// `Vec<providers::types::Message>` to / from this JSON shape.
    pub messages_json: serde_json::Value,
    /// Total tokens freed across all applied tiers.
    pub freed_tokens: u64,
    /// Ordered list of tier ids the ladder applied (cheapest-first;
    /// stops at the first tier that drops below the target budget).
    pub tiers_applied: Vec<CompactionTierId>,
    /// First-N bytes of the prompt (system + CLAUDE.md + first user
    /// turn) preserved across the ladder run to maximize prompt-cache
    /// hit probability on the remote sidecar / cloud Sonnet path.
    /// Static-mode reads emit a `compaction_cache_preserved` fire-point
    /// when this field stays unchanged across the ladder.
    pub cache_preserved_bytes: u64,
}

/// Environment-variable name controlling wire-dark SSB emission for
/// E-25 claude-code-architecture fire-points. Set to `1` to enable.
///
/// Defaults to OFF to keep the catalog substrate dark until concrete
/// downstream consumers ship.
pub const ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR: &str = "ELAI_CLAUDE_CODE_ARCH_ENABLE";

/// Returns true iff the E-25 SSB emission surface is wire-enabled via
/// the canonical env var. Default-off keeps the substrate dark per
/// wire-dark posture established by E-11 / E-22 / E-23 / E-43 / E-55.
#[must_use]
pub fn ssb_emission_enabled() -> bool {
    std::env::var(ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR)
        .map(|v| v == "1")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_taxonomy_count_is_locked() {
        // Exactly 9 primitives. Bumping requires updating ssb.rs
        // `ClaudeCodeArchitectureEventKind` lockstep AND
        // workflow/reasoning/E-25-decisions.md.
        assert_eq!(ALL_CLAUDE_CODE_PRIMITIVES.len(), 9);
    }

    #[test]
    fn primitive_round_trips_snake_case() {
        for &p in ALL_CLAUDE_CODE_PRIMITIVES {
            let label = p.as_snake_case();
            let round_tripped = ClaudeCodeArchPrimitive::from_snake_case(label)
                .expect("snake_case round-trip");
            assert_eq!(round_tripped, p);
        }
    }

    #[test]
    fn primitive_labels_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for &p in ALL_CLAUDE_CODE_PRIMITIVES {
            assert!(seen.insert(p.as_snake_case()), "duplicate label");
        }
        assert_eq!(seen.len(), 9);
    }

    #[test]
    fn primitive_display_labels_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for &p in ALL_CLAUDE_CODE_PRIMITIVES {
            assert!(seen.insert(p.display_label()), "duplicate display label");
        }
        assert_eq!(seen.len(), 9);
    }

    #[test]
    fn primitive_from_snake_case_rejects_unknown() {
        assert!(ClaudeCodeArchPrimitive::from_snake_case("deferred_mcp_schemas").is_none());
        assert!(ClaudeCodeArchPrimitive::from_snake_case("").is_none());
        assert!(ClaudeCodeArchPrimitive::from_snake_case("compactionladder").is_none());
    }

    #[test]
    fn event_kind_count_matches_primitive_count() {
        // 9 fire-points; one per primitive. Pattern-2 single-variant
        // discipline locks this 1:1 mapping.
        let kinds = [
            ClaudeCodeArchitectureEventKind::CompactionLadderTierApplied,
            ClaudeCodeArchitectureEventKind::SkillTemplateExecuted,
            ClaudeCodeArchitectureEventKind::PromptDecisionMade,
            ClaudeCodeArchitectureEventKind::OpusplanOverlayActivated,
            ClaudeCodeArchitectureEventKind::SessionForked,
            ClaudeCodeArchitectureEventKind::EditCheckpointSnapshotted,
            ClaudeCodeArchitectureEventKind::BashComplexityBlocked,
            ClaudeCodeArchitectureEventKind::ApprovalFatigueDetected,
            ClaudeCodeArchitectureEventKind::SandboxTemplateWarned,
        ];
        assert_eq!(kinds.len(), ALL_CLAUDE_CODE_PRIMITIVES.len());
    }

    #[test]
    fn event_kind_primitive_mapping_is_consistent() {
        let kinds = [
            ClaudeCodeArchitectureEventKind::CompactionLadderTierApplied,
            ClaudeCodeArchitectureEventKind::SkillTemplateExecuted,
            ClaudeCodeArchitectureEventKind::PromptDecisionMade,
            ClaudeCodeArchitectureEventKind::OpusplanOverlayActivated,
            ClaudeCodeArchitectureEventKind::SessionForked,
            ClaudeCodeArchitectureEventKind::EditCheckpointSnapshotted,
            ClaudeCodeArchitectureEventKind::BashComplexityBlocked,
            ClaudeCodeArchitectureEventKind::ApprovalFatigueDetected,
            ClaudeCodeArchitectureEventKind::SandboxTemplateWarned,
        ];
        let mut seen_primitives = std::collections::BTreeSet::new();
        for k in &kinds {
            let p = k.primitive();
            assert!(seen_primitives.insert(p.as_snake_case()),
                "duplicate primitive mapping for kind {:?}", k);
        }
        assert_eq!(seen_primitives.len(), 9);
    }

    #[test]
    fn event_kind_round_trips_str() {
        let kinds = [
            ClaudeCodeArchitectureEventKind::CompactionLadderTierApplied,
            ClaudeCodeArchitectureEventKind::SkillTemplateExecuted,
            ClaudeCodeArchitectureEventKind::PromptDecisionMade,
            ClaudeCodeArchitectureEventKind::OpusplanOverlayActivated,
            ClaudeCodeArchitectureEventKind::SessionForked,
            ClaudeCodeArchitectureEventKind::EditCheckpointSnapshotted,
            ClaudeCodeArchitectureEventKind::BashComplexityBlocked,
            ClaudeCodeArchitectureEventKind::ApprovalFatigueDetected,
            ClaudeCodeArchitectureEventKind::SandboxTemplateWarned,
        ];
        for k in &kinds {
            let s = k.as_str();
            let round_tripped =
                ClaudeCodeArchitectureEventKind::from_str(s).expect("str round-trip");
            assert_eq!(round_tripped, *k);
        }
    }

    #[test]
    fn compaction_tier_taxonomy_is_locked() {
        assert_eq!(ALL_COMPACTION_TIERS.len(), 6);
        for (i, t) in ALL_COMPACTION_TIERS.iter().enumerate() {
            assert_eq!(t.ordinal() as usize, i + 1);
        }
    }

    #[test]
    fn compaction_tier_labels_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for &t in ALL_COMPACTION_TIERS {
            assert!(seen.insert(t.as_snake_case()), "duplicate tier label");
        }
        assert_eq!(seen.len(), 6);
    }

    #[test]
    fn payload_round_trips_msgpack() {
        let payload = ClaudeCodeArchitectureEventPayload {
            kind: ClaudeCodeArchitectureEventKind::CompactionLadderTierApplied,
            primitive: ClaudeCodeArchPrimitive::CompactionLadder,
            session_id: Some("s-e25-test".into()),
            details_json: serde_json::json!({
                "tier_id": "snip",
                "freed_tokens": 1234u64,
                "wall_ms": 7
            }),
            ts_ms: 1_700_000_172_001,
        };
        let bytes = rmp_serde::to_vec_named(&payload).expect("serialize");
        let deserialized: ClaudeCodeArchitectureEventPayload =
            rmp_serde::from_slice(&bytes).expect("deserialize");
        assert_eq!(deserialized, payload);
    }

    #[test]
    fn ladder_result_round_trips_msgpack() {
        let result = ClaudeCodeLadderResult {
            messages_json: serde_json::json!([
                {"role": "user", "content": "hi"},
                {"role": "assistant", "content": "hello"}
            ]),
            freed_tokens: 9000,
            tiers_applied: vec![CompactionTierId::BudgetReduction, CompactionTierId::Snip],
            cache_preserved_bytes: 4096,
        };
        let bytes = rmp_serde::to_vec_named(&result).expect("serialize");
        let deserialized: ClaudeCodeLadderResult =
            rmp_serde::from_slice(&bytes).expect("deserialize");
        assert_eq!(deserialized, result);
    }

    #[test]
    fn env_gate_defaults_off() {
        // SAFETY: only this test mutates this var; cargo test default
        // threading per-crate is single-process so env mutation is OK
        // for the unset / set / unset roundtrip below.
        let prior = std::env::var(ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR).ok();
        std::env::remove_var(ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR);
        assert!(!ssb_emission_enabled(), "default-off invariant");
        std::env::set_var(ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR, "1");
        assert!(ssb_emission_enabled(), "explicit-1 enables");
        std::env::set_var(ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR, "0");
        assert!(!ssb_emission_enabled(), "explicit-0 keeps dark");
        match prior {
            Some(v) => std::env::set_var(ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR, v),
            None => std::env::remove_var(ELAI_CLAUDE_CODE_ARCH_ENABLE_VAR),
        }
    }
}
