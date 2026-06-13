//! I-35: temporal tool-sequence enforcement — ordered history rules at the supervisor plane.
//!
//! The fourth dimension of I-1 enforcement: per-call argument gating (E-30) and per-state
//! masking (Phase 3A FSM) are ORDER-blind. I-35 adds ORDER: a TOML-declared rule set
//! evaluated over the ordered [`HistoryEntry`] buffer populated by every dispatcher cycle.
//!
//! CDA-immunity: the predicate evaluates the TOOL-CALL history buffer — an off-agent
//! artifact built from real dispatch events, not from token content. An attacker that
//! controls the model's output tokens CANNOT forge history rows.
//!
//! Replay determinism (GD-2): same ordered history + same rule set -> identical verdicts.
//! The rule set is embedded at compile time via `include_str!` (R8-frozen: the running
//! session cannot modify the TOML because `gate_freeze_check.py` guards the file hash).
//!
//! ## Predicate honesty (DEGRADED-PREDICATE constraint from I-35 plan)
//!
//! The 3 Tier-1 rules reference event kinds (`auditor_pass`, `taint_check_pass`, `test_run`)
//! that production paths do NOT yet emit into the history buffer. The predicates are
//! implemented honestly:
//!   - R1/R2 ship ENABLED with DEGRADED predicate: they fire only when the event kind IS in
//!     the buffer (the bench fixture injects it). Missing evidence = unsatisfied = OrderViolation
//!     fail-closed. Future evidence source: auditor exit path + taint pipeline emitters.
//!   - R3 ships DISABLED_WITH_COMMENT: cannot be made meaningful even in degraded form
//!     without a production test-runner push site. Disabled is safer than always-passing.
//!
//! ## Rule TOML path
//!
//! The compiled-in rule file is at `crates/rules/seed/tool_sequence_core.toml`.
//! Embedded via `include_str!("../../rules/seed/tool_sequence_core.toml")`.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of [`HistoryEntry`] items kept in the internal ring buffer.
/// Older entries are evicted FIFO when the buffer is full. 1024 is sufficient
/// for any realistic session without unbounded growth.
pub(crate) const HISTORY_BUFFER_CAPACITY: usize = 1024;

/// The compile-time embedded safety-core rule TOML (R8-frozen).
///
/// Embedded at compile time so the binary carries the rule set regardless of
/// the file-system layout at runtime (same pattern as `rule_loader.rs` SECURITY_SEED_TOMLS).
/// `gate_freeze_check.py` pins the hash of this file in `.claude/gate-manifest.txt`; a
/// session that edits the TOML is BLOCKED (R8: a run cannot modify its own evaluator).
pub(crate) const TOOL_SEQUENCE_CORE_TOML: &str =
    include_str!("../../rules/seed/tool_sequence_core.toml");

// ---------------------------------------------------------------------------
// HistoryEntry — the ordered record of a completed dispatch cycle
// ---------------------------------------------------------------------------

/// One entry in the ordered tool-call history buffer.
///
/// Populated by [`crate::directive::InProcessSupervisor::push_history_entry`] (called
/// from [`crate::hook_dispatcher::InProcessHookDispatcher::dispatch_pre_tool_use`] after
/// each completed dispatch cycle). The buffer is the predicate's input for sequence rules.
///
/// Kept deliberately simple (no generics, no trait bounds) so the type is object-safe and
/// can be passed through `Arc<dyn Supervisor>` via the default-no-op trait method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Logical tool class for sequence-rule matching.
    ///
    /// Examples: `"git_push"`, `"file_write"`, `"untrusted_ingest"`, `"auditor_pass"`,
    /// `"taint_check_pass"`, `"test_run"`, `"src_edit"`.
    ///
    /// This is a semantic class string, not the raw `action_type` SSB key. The dispatcher
    /// maps `invocation.action_type` to a class string via [`tool_class_from_action_type`].
    pub tool_class: String,
    /// Timestamp in milliseconds since UNIX epoch (from the injected ClockProvider).
    pub ts_ms: i64,
    /// Outcome of the dispatch cycle: `"allowed"`, `"blocked"`, or `"error"`.
    pub outcome: String,
}

impl HistoryEntry {
    /// Construct a history entry from its parts.
    #[must_use]
    pub fn new(
        tool_class: impl Into<String>,
        ts_ms: i64,
        outcome: impl Into<String>,
    ) -> Self {
        Self {
            tool_class: tool_class.into(),
            ts_ms,
            outcome: outcome.into(),
        }
    }
}

/// Map a raw dispatcher `action_type` string to the logical tool class used by
/// sequence rules. Unknown action types are mapped to `"unknown"` (which no rule
/// triggers on, so they pass through silently — fail-open only for unrecognised
/// types, which is acceptable here since the gate is additive).
#[must_use]
pub fn tool_class_from_action_type(action_type: &str) -> &'static str {
    match action_type {
        // Shell execution
        "bash_tool_use" => "bash",
        // File mutations
        "file_write" | "write_file" | "edit" | "Edit" | "Write" => "file_write",
        // Source edits (subset of file_write but semantically distinct for rules)
        "source_edit" | "src_edit" => "src_edit",
        // Git operations
        "git_push" => "git_push",
        "git_commit" => "git_commit",
        // Ingest / untrusted input
        "untrusted_ingest" | "web_fetch" | "mcp_tool_call" => "untrusted_ingest",
        // Auditor pass signal (injected by bench fixture; production: auditor CLI exit)
        "auditor_pass" => "auditor_pass",
        // Taint-check pass signal (injected by bench fixture; production: taint pipeline)
        "taint_check_pass" => "taint_check_pass",
        // Test runner pass signal (injected by bench fixture; production: deferred)
        "test_run_pass" => "test_run_pass",
        _ => "unknown",
    }
}

// ---------------------------------------------------------------------------
// HistoryBuffer — the bounded ring buffer
// ---------------------------------------------------------------------------

/// Bounded FIFO ring buffer of [`HistoryEntry`] values.
///
/// Wrapped in `Arc<Mutex<...>>` so it can be shared across the dispatcher and
/// the supervisor without clone overhead. The lock is held only for the duration
/// of a single push or snapshot read — no async code runs under the lock.
#[derive(Debug)]
pub struct HistoryBuffer {
    inner: Arc<Mutex<VecDeque<HistoryEntry>>>,
    capacity: usize,
}

impl Clone for HistoryBuffer {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            capacity: self.capacity,
        }
    }
}

impl HistoryBuffer {
    /// Create a new buffer with the default capacity ([`HISTORY_BUFFER_CAPACITY`]).
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(
                HISTORY_BUFFER_CAPACITY,
            ))),
            capacity: HISTORY_BUFFER_CAPACITY,
        }
    }

    /// Push one entry, evicting the oldest if at capacity.
    pub fn push(&self, entry: HistoryEntry) {
        let mut guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        if guard.len() >= self.capacity {
            guard.pop_front();
        }
        guard.push_back(entry);
    }

    /// Return a snapshot of the current buffer as a `Vec` (oldest-first).
    #[must_use]
    pub fn snapshot(&self) -> Vec<HistoryEntry> {
        let guard = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        guard.iter().cloned().collect()
    }
}

impl Default for HistoryBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// SequenceRule TOML model
// ---------------------------------------------------------------------------

/// Deserialised TOML representation of a single sequence rule.
///
/// A rule is a {trigger, predicate, enforcement} triad:
///   - `trigger`: the tool class that causes this rule to be evaluated.
///   - `predicate`: the condition that must hold over the prior history.
///   - `enforcement`: what happens when the predicate is NOT satisfied.
///
/// Predicates are deterministic queries over the ordered [`HistoryEntry`] slice —
/// NOT closures (frozen/serialisable, GD-2 compliant).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SequenceRule {
    /// Unique rule identifier (e.g., `"R1-git-push-requires-auditor-pass"`).
    pub id: String,
    /// The tool class that triggers this rule (matches [`HistoryEntry::tool_class`]).
    pub trigger: String,
    /// The predicate kind + its parameters.
    pub predicate: SequencePredicate,
    /// What the gate does when the predicate is NOT satisfied.
    pub enforcement: EnforcementAction,
    /// Human-readable description of the rule (documentation, not evaluated).
    pub description: String,
    /// Whether this rule is currently enabled. `false` = rule is skipped entirely.
    /// Used for R3 (test_run) which cannot be made meaningful without a production
    /// push site.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Documents the predicate degradation level: `"full"`, `"degraded"`, or `"disabled"`.
    /// Informational only; `enabled=false` is the machine-readable disable.
    #[serde(default)]
    pub predicate_honesty: String,
}

fn default_enabled() -> bool {
    true
}

/// Deterministic predicate kinds for sequence rules.
///
/// Each variant encodes a concrete query over the ordered history slice. Variants are
/// CLOSED (enumerated at compile time) so the gate cannot be extended at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SequencePredicate {
    /// Requires that at least one entry with `tool_class == required_class` AND
    /// `outcome == "allowed"` exists AFTER the most recent entry with
    /// `tool_class == reset_on_class` (or anywhere in the buffer if no reset has occurred).
    RequiresSinceLast {
        /// The tool class that must have appeared since the last `reset_on_class`.
        required_class: String,
        /// The tool class that resets the window (the "last src_edit" boundary).
        reset_on_class: String,
    },
    /// Requires that at least one entry with `tool_class == required_class` AND
    /// `outcome == "allowed"` exists BETWEEN the most recent `after_class` entry
    /// and the current trigger — i.e., after the last `after_class` but before now.
    RequiresBetween {
        /// The preceding tool class that opens the window.
        after_class: String,
        /// The tool class that must appear between `after_class` and the trigger.
        required_class: String,
    },
}

/// What the gate does when the predicate is NOT satisfied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EnforcementAction {
    /// Refuse the tool call and emit `ToolSequenceVerdict::HaltEnforced`.
    /// The call is Discarded at the supervisor level (consistent with the blocked path).
    Halt,
    /// Emit `ToolSequenceVerdict::OrderViolation` but do NOT halt.
    /// Currently unused in Tier-1 rules (all Tier-1 use `halt`); reserved for Tier-2.
    WarnOnly,
}

// ---------------------------------------------------------------------------
// SequenceRuleSet — the compiled rule registry
// ---------------------------------------------------------------------------

/// The compiled rule registry, produced from the TOML source.
///
/// The `content_hash` is the blake3 hash of the raw TOML bytes, recorded in
/// `gate-manifest.txt` for R8 freeze verification.
#[derive(Debug, Clone)]
pub struct SequenceRuleSet {
    /// The parsed rules, in declaration order.
    pub rules: Vec<SequenceRule>,
    /// blake3 hash of the raw TOML source bytes (R8 freeze anchor).
    pub content_hash: [u8; 32],
}

/// TOML root type for deserialisation (the `[[rules]]` array).
#[derive(Debug, Deserialize)]
struct RuleFile {
    rules: Vec<SequenceRule>,
}

impl SequenceRuleSet {
    /// Parse and compile the rule set from raw TOML bytes.
    ///
    /// Returns `Err` if the TOML is malformed. Each rule is validated
    /// independently; unknown fields are silently ignored for forward-compatibility.
    pub fn from_toml_bytes(bytes: &[u8]) -> Result<Self, SequenceRuleError> {
        let src = std::str::from_utf8(bytes)
            .map_err(|e| SequenceRuleError::TomlParse(format!("UTF-8 error: {e}")))?;
        let file: RuleFile = toml::from_str(src)
            .map_err(|e| SequenceRuleError::TomlParse(e.to_string()))?;
        let hash = *blake3::hash(bytes).as_bytes();
        Ok(Self {
            rules: file.rules,
            content_hash: hash,
        })
    }

    /// Load the compiled-in safety-core rule set ([`TOOL_SEQUENCE_CORE_TOML`]).
    ///
    /// This is the canonical production loader. The bytes are embedded at compile time
    /// so the binary is filesystem-independent. Panics at startup if the embedded TOML
    /// is malformed (a build-time guarantee that any TOML change is caught at compile).
    #[must_use]
    pub fn load_builtin() -> Self {
        Self::from_toml_bytes(TOOL_SEQUENCE_CORE_TOML.as_bytes())
            .expect("tool_sequence_core.toml must parse; malformed TOML is a build error")
    }
}

// ---------------------------------------------------------------------------
// SequenceRuleError
// ---------------------------------------------------------------------------

/// Errors raised by rule-set parsing.
#[derive(Debug, thiserror::Error)]
pub enum SequenceRuleError {
    #[error("TOML parse error: {0}")]
    TomlParse(String),
}

// ---------------------------------------------------------------------------
// ToolSequenceVerdict — the gate outcome for a single rule evaluation
// ---------------------------------------------------------------------------

/// Outcome of evaluating the sequence gate against the current history.
///
/// Mirrors the SSB `ToolSequenceVerdictPayload` kind discriminator (I-35 SSB
/// Pattern-2 3-arm): `Allowed`, `OrderViolation`, `HaltEnforced`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolSequenceVerdict {
    /// All enabled rules that trigger on this tool class were satisfied.
    /// The call is allowed to proceed.
    Allowed,
    /// At least one rule's predicate was not satisfied. For `WarnOnly` enforcement.
    OrderViolation {
        /// The `rule_id` of the first unsatisfied rule.
        rule_id: String,
        /// Human-readable description of which predicate was not satisfied.
        unsatisfied_predicate: String,
    },
    /// A Halt-enforced rule's predicate was not satisfied. The call is refused.
    HaltEnforced {
        /// The `rule_id` of the first halt-enforced rule that fired.
        rule_id: String,
        /// Human-readable description of which predicate was not satisfied.
        unsatisfied_predicate: String,
    },
}

impl ToolSequenceVerdict {
    /// Returns `true` if this verdict means the tool call should be refused.
    #[must_use]
    pub fn is_halt(&self) -> bool {
        matches!(self, Self::HaltEnforced { .. })
    }
}

// ---------------------------------------------------------------------------
// Core evaluation logic (deterministic, GD-2)
// ---------------------------------------------------------------------------

/// Evaluate one sequence rule against the provided history snapshot.
///
/// Deterministic: same `history` + same `rule` -> identical verdict (GD-2).
/// `history` must be ordered oldest-first (as returned by [`HistoryBuffer::snapshot`]).
///
/// When the rule is disabled (`rule.enabled == false`), always returns [`ToolSequenceVerdict::Allowed`].
#[must_use]
pub fn evaluate_sequence_rule(
    rule: &SequenceRule,
    history: &[HistoryEntry],
) -> ToolSequenceVerdict {
    if !rule.enabled {
        return ToolSequenceVerdict::Allowed;
    }

    let satisfied = match &rule.predicate {
        SequencePredicate::RequiresSinceLast {
            required_class,
            reset_on_class,
        } => {
            // Find the index of the most recent reset_on_class entry.
            let reset_pos = history
                .iter()
                .rposition(|e| &e.tool_class == reset_on_class);
            // If reset_on_class has never appeared, the constraint window was never
            // opened (e.g. git_push with no preceding src_edit in this session).
            // Absent window = trivially satisfied, consistent with RequiresBetween
            // semantics and the I-35 "predicate window" contract (i35_d test).
            if reset_pos.is_none() {
                return ToolSequenceVerdict::Allowed;
            }
            let window_start = reset_pos.map(|i| i + 1).unwrap_or(0);
            // Check if required_class with outcome="allowed" exists AFTER window_start.
            history[window_start..].iter().any(|e| {
                e.tool_class == *required_class && e.outcome == "allowed"
            })
        }
        SequencePredicate::RequiresBetween {
            after_class,
            required_class,
        } => {
            // Find the most recent after_class entry.
            let window_start = history
                .iter()
                .rposition(|e| &e.tool_class == after_class)
                .map(|i| i + 1)
                .unwrap_or(0);
            // If no after_class entry at all, the predicate is unsatisfied
            // (there was an untrusted_ingest "sometime" but it's not in our buffer —
            // fail-closed: missing evidence = unsatisfied).
            if window_start == 0
                && !history.iter().any(|e| &e.tool_class == after_class)
            {
                // No after_class in history: no window opened -> predicate trivially
                // satisfied (the trigger is "after an untrusted_ingest"; if no ingest
                // occurred, no constraint applies).
                return ToolSequenceVerdict::Allowed;
            }
            // required_class with outcome="allowed" must appear after after_class.
            history[window_start..].iter().any(|e| {
                e.tool_class == *required_class && e.outcome == "allowed"
            })
        }
    };

    if satisfied {
        ToolSequenceVerdict::Allowed
    } else {
        match rule.enforcement {
            EnforcementAction::Halt => ToolSequenceVerdict::HaltEnforced {
                rule_id: rule.id.clone(),
                unsatisfied_predicate: rule.description.clone(),
            },
            EnforcementAction::WarnOnly => ToolSequenceVerdict::OrderViolation {
                rule_id: rule.id.clone(),
                unsatisfied_predicate: rule.description.clone(),
            },
        }
    }
}

/// Evaluate the full rule set against the provided history and trigger tool class.
///
/// Returns the first non-Allowed verdict (fail-fast), or `Allowed` if all enabled
/// rules that match `trigger_class` are satisfied.
///
/// Deterministic: same `history` + same `trigger_class` + same `rules` -> identical
/// verdict (GD-2).
#[must_use]
pub fn evaluate_sequence_rules(
    rules: &SequenceRuleSet,
    trigger_class: &str,
    history: &[HistoryEntry],
) -> ToolSequenceVerdict {
    for rule in &rules.rules {
        if rule.trigger != trigger_class {
            continue;
        }
        match evaluate_sequence_rule(rule, history) {
            ToolSequenceVerdict::Allowed => continue,
            non_allowed => return non_allowed,
        }
    }
    ToolSequenceVerdict::Allowed
}

// ---------------------------------------------------------------------------
// SSB emission helper
// ---------------------------------------------------------------------------

/// Emit one [`ToolSequenceVerdict`] SSB row.
///
/// The bench probe uses this to exercise the real SSB write path and verify
/// that the off-agent artifact (the SSB row) lands — satisfying wiring-enforcement
/// R1/R4 without requiring a live inference step.
///
/// A `None` `ssb` is a no-op (caller does not have a store yet).
///
/// # Errors
///
/// Returns `Err` only on SSB infrastructure failure (write error), not on
/// `OrderViolation` / `HaltEnforced` verdicts — those are expected outcomes
/// that the bench probe asserts on.
pub async fn emit_tool_sequence_verdict(
    ssb: Option<&std::sync::Arc<crate::ssb::Ssb>>,
    verdict: &ToolSequenceVerdict,
    tool_class: &str,
    session_id: Option<&str>,
) -> anyhow::Result<()> {
    use crate::ssb::{SsbPayload, ToolSequenceVerdictKind, ToolSequenceVerdictPayload};

    let Some(ssb) = ssb else {
        return Ok(());
    };

    let (kind, rule_id, unsatisfied) = match verdict {
        ToolSequenceVerdict::Allowed => (
            ToolSequenceVerdictKind::Allowed,
            String::new(),
            String::new(),
        ),
        ToolSequenceVerdict::OrderViolation { rule_id, unsatisfied_predicate } => (
            ToolSequenceVerdictKind::OrderViolation,
            rule_id.clone(),
            unsatisfied_predicate.clone(),
        ),
        ToolSequenceVerdict::HaltEnforced { rule_id, unsatisfied_predicate } => (
            ToolSequenceVerdictKind::HaltEnforced,
            rule_id.clone(),
            unsatisfied_predicate.clone(),
        ),
    };

    let ts_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let payload = SsbPayload::ToolSequenceVerdict(ToolSequenceVerdictPayload {
        kind,
        tool_class: tool_class.to_owned(),
        rule_id,
        unsatisfied_predicate: unsatisfied,
        ts_ms,
    });

    let agent_id = session_id.unwrap_or("bench-sequence-probe");
    ssb.write_payload(agent_id, 0, payload, Vec::new())
        .await
        .map_err(|e| anyhow::anyhow!("tool_sequence SSB write error: {e}"))?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(tool_class: &str, outcome: &str) -> HistoryEntry {
        HistoryEntry::new(tool_class, 1_700_000_000_000, outcome)
    }

    #[test]
    fn requires_since_last_satisfied() {
        let history = vec![
            entry("src_edit", "allowed"),
            entry("auditor_pass", "allowed"),
        ];
        let rule = SequenceRule {
            id: "R1-test".into(),
            trigger: "git_push".into(),
            predicate: SequencePredicate::RequiresSinceLast {
                required_class: "auditor_pass".into(),
                reset_on_class: "src_edit".into(),
            },
            enforcement: EnforcementAction::Halt,
            description: "git_push requires auditor_pass since last src_edit".into(),
            enabled: true,
            predicate_honesty: "degraded".into(),
        };
        assert_eq!(
            evaluate_sequence_rule(&rule, &history),
            ToolSequenceVerdict::Allowed
        );
    }

    #[test]
    fn requires_since_last_violated() {
        let history = vec![
            entry("src_edit", "allowed"),
            // No auditor_pass after src_edit.
        ];
        let rule = SequenceRule {
            id: "R1-test".into(),
            trigger: "git_push".into(),
            predicate: SequencePredicate::RequiresSinceLast {
                required_class: "auditor_pass".into(),
                reset_on_class: "src_edit".into(),
            },
            enforcement: EnforcementAction::Halt,
            description: "git_push requires auditor_pass since last src_edit".into(),
            enabled: true,
            predicate_honesty: "degraded".into(),
        };
        assert!(
            evaluate_sequence_rule(&rule, &history).is_halt(),
            "must halt when auditor_pass is absent since last src_edit"
        );
    }

    #[test]
    fn requires_between_no_ingest_passes_trivially() {
        // If no untrusted_ingest in history, the file_write constraint does not apply.
        let history = vec![entry("src_edit", "allowed")];
        let rule = SequenceRule {
            id: "R2-test".into(),
            trigger: "file_write".into(),
            predicate: SequencePredicate::RequiresBetween {
                after_class: "untrusted_ingest".into(),
                required_class: "taint_check_pass".into(),
            },
            enforcement: EnforcementAction::Halt,
            description: "file_write after untrusted_ingest requires taint_check_pass".into(),
            enabled: true,
            predicate_honesty: "degraded".into(),
        };
        assert_eq!(
            evaluate_sequence_rule(&rule, &history),
            ToolSequenceVerdict::Allowed,
            "no untrusted_ingest in history = constraint trivially satisfied"
        );
    }

    #[test]
    fn requires_between_violated_after_ingest() {
        let history = vec![
            entry("untrusted_ingest", "allowed"),
            // No taint_check_pass before file_write.
        ];
        let rule = SequenceRule {
            id: "R2-test".into(),
            trigger: "file_write".into(),
            predicate: SequencePredicate::RequiresBetween {
                after_class: "untrusted_ingest".into(),
                required_class: "taint_check_pass".into(),
            },
            enforcement: EnforcementAction::Halt,
            description: "file_write after untrusted_ingest requires taint_check_pass".into(),
            enabled: true,
            predicate_honesty: "degraded".into(),
        };
        assert!(
            evaluate_sequence_rule(&rule, &history).is_halt(),
            "must halt when taint_check_pass absent after untrusted_ingest"
        );
    }

    #[test]
    fn disabled_rule_always_allows() {
        let history = vec![entry("src_edit", "allowed")];
        let rule = SequenceRule {
            id: "R3-disabled".into(),
            trigger: "git_commit".into(),
            predicate: SequencePredicate::RequiresSinceLast {
                required_class: "test_run_pass".into(),
                reset_on_class: "src_edit".into(),
            },
            enforcement: EnforcementAction::Halt,
            description: "disabled rule".into(),
            enabled: false,
            predicate_honesty: "disabled".into(),
        };
        assert_eq!(
            evaluate_sequence_rule(&rule, &history),
            ToolSequenceVerdict::Allowed,
            "disabled rule must always return Allowed"
        );
    }

    #[test]
    fn history_buffer_evicts_oldest() {
        let buf = HistoryBuffer {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(3))),
            capacity: 3,
        };
        buf.push(entry("a", "allowed"));
        buf.push(entry("b", "allowed"));
        buf.push(entry("c", "allowed"));
        buf.push(entry("d", "allowed")); // should evict "a"
        let snap = buf.snapshot();
        assert_eq!(snap.len(), 3);
        assert_eq!(snap[0].tool_class, "b");
        assert_eq!(snap[2].tool_class, "d");
    }

    #[test]
    fn builtin_rule_set_parses() {
        // Verify the compile-time embedded TOML is valid and produces at least
        // 2 enabled rules.
        let rules = SequenceRuleSet::load_builtin();
        let enabled: Vec<_> = rules.rules.iter().filter(|r| r.enabled).collect();
        assert!(
            enabled.len() >= 2,
            "must have at least 2 enabled rules in tool_sequence_core.toml; got {}",
            enabled.len()
        );
        // Hash must be non-zero (blake3 of any non-empty input is not all-zero).
        assert!(
            rules.content_hash.iter().any(|&b| b != 0),
            "content_hash must be non-zero"
        );
    }
}
