//! F-8 SCF — symbolic semantic conflict detector (self-mod pre-execution).
//!
//! A *symbolic* (LLM-free, pure, deterministic) detector that flags a
//! self-modification config variant which contradicts an active locked-key
//! invariant **before** archive admission. The detector is a guard SIGNAL,
//! not an accuracy claim: SCF's published 27.9% precision is a
//! simulation-only limitation (reliability LOW) and is a NON-target — it
//! never appears as an ELAI accuracy goal. That UNVERIFIED precision is
//! exactly why the policy is WARN-then-escalate (held-pending), never a hard
//! fail-closed BLOCK (see `workflow/reasoning/F-8-decisions.md`).
//!
//! This module is intentionally the *symbolic* half only. The detector
//! composes into:
//!   * the live `elai self-mod apply --proposal-fixture <json>` CLI path
//!     (the F-8 W-3 production entrypoint), and
//!   * the darwinian-evolve promotion loop at
//!     `external_benches.rs` (a documented GATED checkbox — that loop is
//!     RUNPOD-PENDING per E-7/E-9).
//!
//! Ground truth (re-grepped fresh, s-60824):
//!   * Locked keys come from [`crate::self_mod_safety::LOCKED_KEYS`] (the
//!     `pub const`) + [`crate::self_mod_safety::is_locked`] (`pub`) — NOT
//!     the private `locked_set`.
//!   * Tripwire quiescence is violated by ANY non-empty active set of the
//!     five [`crate::tripwire::TripwireKind`] variants — there is no
//!     `Divergence` variant.
//!   * [`crate::self_mod_types::ConfigVariant::config_blob`] is a
//!     serde_json object (`{key: value}`) — decoded the same way
//!     `osc::decode_flat_map` does.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::self_mod_safety::{is_locked, LOCKED_KEYS};
use crate::self_mod_types::{ConfigVariant, ProposedChange, StructuralChange};
use crate::ssb::{Ssb, SsbPayload, SymbolicConflictPayload};
use crate::tripwire::TripwireKind;

/// Agent id stamped on the F-8 symbolic-conflict SSB beacon. Mirrors the
/// per-gate agent-id convention used by the F-10 security-monotonicity gate
/// (`emit_security_class_regression_blocked`).
const SCF_AGENT_ID: &str = "symbolic_conflict_detector";

/// Superstep stamped on the F-8 symbolic-conflict SSB beacon. The detector is
/// a single-shot pre-admission gate (no superstep loop), so this is fixed at
/// `0` exactly like the F-10 gate beacon.
const SCF_SUPERSTEP: i64 = 0;

/// Length (hex chars) of the conflict fingerprint prefix. 16 hex chars =
/// 64 bits of the blake3 digest, enough to deduplicate conflict rows in the
/// SSB without carrying the full 64-char digest on every beacon.
pub const CONFLICT_FINGERPRINT_HEX_LEN: usize = 16;

/// Spend / capacity rails — a subset of [`LOCKED_KEYS`] whose semantics are
/// "resource limits the self-mod agent may never expand". A `Require`
/// intent naming one of these is a [`ConflictKind::ResourceContention`]
/// (the variant demands headroom on a rail it is forbidden to widen).
///
/// These names are validated against [`LOCKED_KEYS`] by the
/// `resource_rails_are_locked` contract test, so a future rename of a
/// locked key cannot silently desync this list.
pub const RESOURCE_RAIL_KEYS: &[&str] = &[
    "cloud_monthly_budget",
    "cloud_opus_allowed",
    "cloud_escalation_threshold",
];

// ---------------------------------------------------------------------------
// Intent algebra
// ---------------------------------------------------------------------------

/// The operation a self-mod intent performs against a config key. Closed
/// enum — every variant carries a distinct symbolic meaning the detector
/// reasons over. `Set`/`Raise`/`Lower` are *mutations*; `Require` is a
/// *precondition demand* (the variant needs the key to provide headroom).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentOp {
    /// Assign a new value to the key (the generic config-blob mutation).
    Set,
    /// Increase the key's numeric value.
    Raise,
    /// Decrease the key's numeric value.
    Lower,
    /// Demand that the key provide capacity / headroom (a precondition,
    /// not a mutation).
    Require,
}

impl IntentOp {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            IntentOp::Set => "set",
            IntentOp::Raise => "raise",
            IntentOp::Lower => "lower",
            IntentOp::Require => "require",
        }
    }

    /// True when the op *mutates* the key (vs merely demanding headroom).
    /// `Require` is the only non-mutating op.
    #[must_use]
    pub fn is_mutation(&self) -> bool {
        matches!(self, IntentOp::Set | IntentOp::Raise | IntentOp::Lower)
    }
}

/// One symbolic condition: a `key`, the `op` applied to it, and the
/// associated `value` (stringified — the detector reasons symbolically over
/// keys + ops, not over numeric ranges).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IntentCondition {
    pub key: String,
    pub op: IntentOp,
    pub value: String,
}

impl IntentCondition {
    /// Construct a condition. Takes owned/`Into<String>` for ergonomic call
    /// sites in both production and tests.
    pub fn new(key: impl Into<String>, op: IntentOp, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            op,
            value: value.into(),
        }
    }
}

/// The symbolic intent extracted from a self-mod proposal: what the variant
/// *asserts* (the mutations it performs) and what it *requires* (the
/// preconditions it demands of the process).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolicIntent {
    /// Mutations the variant performs on config keys.
    pub asserts: Vec<IntentCondition>,
    /// Preconditions the variant demands (resource headroom, capacity).
    pub requires: Vec<IntentCondition>,
}

impl SymbolicIntent {
    /// True when the intent has neither asserts nor requires — a no-op
    /// proposal that can never conflict.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.asserts.is_empty() && self.requires.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Process model
// ---------------------------------------------------------------------------

/// A snapshot of the live process invariants the detector checks an intent
/// against. Built by [`current_process_model`] from
/// [`crate::self_mod_safety::LOCKED_KEYS`] + the active tripwire set.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessModel {
    /// Snapshot of the locked-key set (owned copy of [`LOCKED_KEYS`]).
    pub locked_keys: Vec<String>,
    /// True when ANY tripwire is currently active (quiescence violated).
    pub tripwires_active: bool,
}

impl ProcessModel {
    /// True when `key` is in this model's locked set. Mirrors
    /// [`crate::self_mod_safety::is_locked`] but reads the captured snapshot
    /// so the detector is a pure function of its inputs.
    #[must_use]
    pub fn is_locked_key(&self, key: &str) -> bool {
        self.locked_keys.iter().any(|k| k == key)
    }
}

// ---------------------------------------------------------------------------
// Conflict types
// ---------------------------------------------------------------------------

/// The three symbolic conflict families (SCF IntentNode pre/postcondition
/// shape). Closed enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictKind {
    /// The intent directly mutates a locked-key invariant (a config key the
    /// self-mod agent may never modify). The canonical "you cannot touch a
    /// locked key" conflict.
    Contradictory,
    /// The intent demands headroom on a resource rail the agent is
    /// forbidden to expand (a `Require` on a locked spend/capacity key).
    ResourceContention,
    /// The intent mutates a locked-key-governed parameter while the process
    /// is NOT quiescent (a tripwire is active) — the causal precondition
    /// (quiescence before any governed mutation) is violated.
    CausalViolation,
}

impl ConflictKind {
    /// Snake-case wire label (used by the SSB beacon in a later slice).
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            ConflictKind::Contradictory => "contradictory",
            ConflictKind::ResourceContention => "resource_contention",
            ConflictKind::CausalViolation => "causal_violation",
        }
    }
}

/// A single detected conflict. `fingerprint` is a deterministic blake3
/// prefix over `(kind, key, detail)` so identical conflicts dedupe across
/// runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolicConflict {
    pub kind: ConflictKind,
    pub key: String,
    pub detail: String,
    pub fingerprint: String,
}

impl SymbolicConflict {
    /// Construct a conflict, computing its fingerprint from
    /// `(kind, key, detail)`.
    pub fn new(kind: ConflictKind, key: impl Into<String>, detail: impl Into<String>) -> Self {
        let key = key.into();
        let detail = detail.into();
        let fingerprint = conflict_fingerprint(kind, &key, &detail);
        Self {
            kind,
            key,
            detail,
            fingerprint,
        }
    }
}

// ---------------------------------------------------------------------------
// Fingerprint
// ---------------------------------------------------------------------------

/// Deterministic conflict fingerprint: the first
/// [`CONFLICT_FINGERPRINT_HEX_LEN`] hex chars of
/// `blake3(kind | key | detail)`. blake3 per the project HARD RULE
/// (never blake2 / sha2 for non-RFC uses).
#[must_use]
pub fn conflict_fingerprint(kind: ConflictKind, key: &str, detail: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(kind.as_str().as_bytes());
    hasher.update(b"|");
    hasher.update(key.as_bytes());
    hasher.update(b"|");
    hasher.update(detail.as_bytes());
    let hex = hasher.finalize().to_hex();
    hex.as_str()
        .chars()
        .take(CONFLICT_FINGERPRINT_HEX_LEN)
        .collect()
}

// ---------------------------------------------------------------------------
// Tripwire quiescence
// ---------------------------------------------------------------------------

/// Quiescence is violated when ANY tripwire is active. Per F-8-decisions.md
/// this is `!active.is_empty()` over the five [`TripwireKind`] variants —
/// NOT a count of three specific kinds. The exhaustive match below exists
/// purely to make the five-variant coverage explicit at compile time: if a
/// sixth kind is ever added the match (and thus this contract) breaks
/// loudly, forcing a deliberate review of quiescence semantics.
#[must_use]
pub fn tripwire_quiescent_violated(active: &[TripwireKind]) -> bool {
    // Compile-time coverage assertion over all five variants. This produces
    // no runtime cost (the closure is only invoked per element) but pins the
    // detector to the closed TripwireKind enum so a new variant forces a
    // review here.
    let _coverage = |k: &TripwireKind| -> bool {
        match k {
            TripwireKind::KlDivergence
            | TripwireKind::CorrelationDrop
            | TripwireKind::IqrOutlier
            | TripwireKind::Perplexity
            | TripwireKind::Ddm => true,
        }
    };
    active.iter().any(|k| _coverage(k))
}

// ---------------------------------------------------------------------------
// Process-model construction
// ---------------------------------------------------------------------------

/// Build the live [`ProcessModel`] from the canonical safety surface:
/// [`LOCKED_KEYS`] (the `pub const`) plus the set of currently-active
/// tripwires. `is_locked` is referenced here (not the private `locked_set`)
/// to keep the model in lock-step with the dispatch guard.
#[must_use]
pub fn current_process_model(active_tripwires: &[TripwireKind]) -> ProcessModel {
    // Defensive: build the snapshot from LOCKED_KEYS but verify each entry
    // round-trips through the public `is_locked` predicate so the snapshot
    // can never drift from the dispatch guard's view of "locked".
    let locked_keys: Vec<String> = LOCKED_KEYS
        .iter()
        .filter(|k| is_locked(k))
        .map(|k| (*k).to_string())
        .collect();
    ProcessModel {
        locked_keys,
        tripwires_active: tripwire_quiescent_violated(active_tripwires),
    }
}

// ---------------------------------------------------------------------------
// Intent extraction
// ---------------------------------------------------------------------------

/// Decode a `config_blob` into its flat key set. Mirrors
/// `osc::decode_flat_map`:
///   * empty blob  -> empty map (fresh archive, no asserts),
///   * JSON object -> the object's keys,
///   * anything else (non-object / undecodable) -> empty map (the variant
///     carries no symbolically-addressable keys; nothing to conflict on).
fn decode_blob_keys(blob: &[u8]) -> BTreeMap<String, String> {
    if blob.is_empty() {
        return BTreeMap::new();
    }
    let value: serde_json::Value = match serde_json::from_slice(blob) {
        Ok(v) => v,
        Err(_) => return BTreeMap::new(),
    };
    match value {
        serde_json::Value::Object(map) => map
            .into_iter()
            .map(|(k, v)| (k, value_to_string(&v)))
            .collect(),
        _ => BTreeMap::new(),
    }
}

/// Stringify a JSON value for the symbolic `value` slot. The detector
/// reasons over keys + ops, so the value is informational only; we keep a
/// compact representation rather than the full nested JSON.
fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => "null".to_string(),
        other => other.to_string(),
    }
}

/// Build a [`SymbolicIntent`] from a [`ConfigVariant`]. Each key in the
/// variant's `config_blob` becomes a `Set` assertion. If a blob key matches
/// a known resource rail it ALSO yields a `Require` condition (the variant
/// demands headroom on that rail) so the resource-contention rule can fire.
#[must_use]
pub fn intent_from_config_variant(cv: &ConfigVariant) -> SymbolicIntent {
    let keys = decode_blob_keys(&cv.config_blob);
    let mut asserts = Vec::with_capacity(keys.len());
    let mut requires = Vec::new();
    for (key, value) in keys {
        if RESOURCE_RAIL_KEYS.contains(&key.as_str()) {
            requires.push(IntentCondition::new(key.clone(), IntentOp::Require, value.clone()));
        }
        asserts.push(IntentCondition::new(key, IntentOp::Set, value));
    }
    SymbolicIntent { asserts, requires }
}

/// Build a [`SymbolicIntent`] from a Layer-3 [`StructuralChange`]. A
/// structural change asserts a `Set` on the config key implied by its
/// `change_type` label. Provided for completeness so the same detector can
/// screen L3 proposals; L3 execution is HUMAN-ONLY, so this only ever feeds
/// the *signal*, never an autonomous mutation.
#[must_use]
pub fn intent_from_structural_change(sc: &StructuralChange) -> SymbolicIntent {
    let key = sc.change_type.as_str().to_string();
    SymbolicIntent {
        asserts: vec![IntentCondition::new(key, IntentOp::Set, sc.description.clone())],
        requires: Vec::new(),
    }
}

/// Build a [`SymbolicIntent`] from a structured [`ProposedChange`]. An
/// `UpdateKey` is a `Set` assertion on its `key`; an `InsertRow` names the
/// target table as a `Set` (data-table appends are screened too, though
/// they rarely touch locked keys).
#[must_use]
pub fn intent_from_proposed_change(change: &ProposedChange) -> SymbolicIntent {
    let condition = match change {
        ProposedChange::UpdateKey {
            key, new_value, ..
        } => IntentCondition::new(key.clone(), IntentOp::Set, new_value.clone()),
        ProposedChange::InsertRow { table, .. } => {
            IntentCondition::new(table.clone(), IntentOp::Set, String::new())
        }
    };
    SymbolicIntent {
        asserts: vec![condition],
        requires: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Detector
// ---------------------------------------------------------------------------

/// The symbolic conflict detector. Pure, deterministic, LLM-free. Returns
/// every conflict the `intent` raises against the `model`. An empty result
/// means the variant is clean and may proceed to admission.
///
/// Rules (each independent; a single intent can raise several):
///   1. [`ConflictKind::Contradictory`] — any *mutating* `assert`
///      (`Set`/`Raise`/`Lower`) on a locked key.
///   2. [`ConflictKind::CausalViolation`] — any mutating `assert` on a
///      locked key WHILE `model.tripwires_active` (quiescence required
///      before touching a governed parameter).
///   3. [`ConflictKind::ResourceContention`] — any `Require` on a locked
///      resource rail (the variant demands headroom on a rail it cannot
///      expand).
#[must_use]
pub fn detect_conflicts(intent: &SymbolicIntent, model: &ProcessModel) -> Vec<SymbolicConflict> {
    let mut conflicts = Vec::new();

    for cond in &intent.asserts {
        if !model.is_locked_key(&cond.key) {
            continue;
        }
        if cond.op.is_mutation() {
            // Rule 1: locked-key mutation is always contradictory.
            conflicts.push(SymbolicConflict::new(
                ConflictKind::Contradictory,
                cond.key.clone(),
                format!(
                    "intent op '{}' mutates locked key '{}'",
                    cond.op.as_str(),
                    cond.key
                ),
            ));
            // Rule 2: the same mutation during active drift also violates
            // the quiescence precondition.
            if model.tripwires_active {
                conflicts.push(SymbolicConflict::new(
                    ConflictKind::CausalViolation,
                    cond.key.clone(),
                    format!(
                        "locked key '{}' mutated while a tripwire is active (quiescence required)",
                        cond.key
                    ),
                ));
            }
        }
    }

    // Rule 3: a Require on a locked resource rail.
    for cond in &intent.requires {
        if cond.op == IntentOp::Require
            && model.is_locked_key(&cond.key)
            && RESOURCE_RAIL_KEYS.contains(&cond.key.as_str())
        {
            conflicts.push(SymbolicConflict::new(
                ConflictKind::ResourceContention,
                cond.key.clone(),
                format!(
                    "intent requires headroom on locked resource rail '{}'",
                    cond.key
                ),
            ));
        }
    }

    conflicts
}

// ---------------------------------------------------------------------------
// SSB emission (WARN-then-escalate)
// ---------------------------------------------------------------------------

/// Emit one [`SsbPayload::SymbolicConflictDetected`] beacon per detected
/// conflict.
///
/// This is the WARN side of WARN-then-escalate: every flagged conflict becomes
/// a durable SSB row (the held-pending record). It NEVER blocks — a write
/// failure is logged and swallowed (the gate is observability, not a hard
/// fail), exactly mirroring [`crate::security_monotonicity::
/// emit_security_class_regression_blocked`]. Each row's `fingerprint` is the
/// conflict's own deterministic blake3 prefix so identical conflicts dedupe
/// across runs; `candidate_id` correlates every row of one screening pass.
///
/// `ssb` MUST be a live handle constructed INSIDE a tokio runtime
/// (`Ssb::new` spawns a write-loop — see gotchas:110). Returns the number of
/// beacons successfully written.
pub async fn emit_symbolic_conflict_detected(
    ssb: &Ssb,
    conflicts: &[SymbolicConflict],
    candidate_id: &str,
    fsm_substate: &str,
) -> usize {
    let mut written = 0usize;
    for conflict in conflicts {
        let payload = SsbPayload::SymbolicConflictDetected(SymbolicConflictPayload {
            fingerprint: conflict.fingerprint.clone(),
            kind: conflict.kind.as_str().to_string(),
            key: conflict.key.clone(),
            detail: conflict.detail.clone(),
            candidate_id: candidate_id.to_string(),
            fsm_substate: fsm_substate.to_string(),
        });
        match ssb
            .write_payload(SCF_AGENT_ID, SCF_SUPERSTEP, payload, Vec::new())
            .await
        {
            Ok(_) => written += 1,
            Err(e) => {
                tracing::error!(
                    error = %e,
                    candidate_id,
                    key = %conflict.key,
                    kind = %conflict.kind.as_str(),
                    "symbolic_conflict: write SymbolicConflictDetected failed"
                );
            }
        }
    }
    written
}

/// Screen one [`ConfigVariant`] for symbolic conflicts and, on any conflict,
/// emit the WARN-then-escalate beacon(s) and report the candidate HELD PENDING.
///
/// This is the reusable compose unit shared by:
///   * the `elai self-mod apply --proposal-fixture <json>` CLI entrypoint (the
///     F-8 W-3 production entrypoint), and
///   * the darwinian-evolve promotion seam
///     (`external_benches::promote_after_bench_completed`).
///
/// POLICY: WARN-then-escalate. The return value is `true` when conflicts were
/// found (the candidate is HELD PENDING — the caller must NOT apply / admit it,
/// but also MUST NOT hard fail-closed). `false` means the variant is clean and
/// may proceed to admission. The beacons themselves are the held-pending
/// record; this function never panics and never blocks promotion of OTHER
/// candidates.
///
/// `ssb` MUST be a live handle constructed inside a tokio runtime
/// (gotchas:110).
pub async fn screen_config_variant_for_conflicts(
    ssb: &Ssb,
    cv: &ConfigVariant,
    active_tripwires: &[TripwireKind],
    fsm_substate: &str,
) -> bool {
    let intent = intent_from_config_variant(cv);
    if intent.is_empty() {
        return false;
    }
    let model = current_process_model(active_tripwires);
    let conflicts = detect_conflicts(&intent, &model);
    if conflicts.is_empty() {
        return false;
    }
    emit_symbolic_conflict_detected(ssb, &conflicts, &cv.variant_id, fsm_substate).await;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json_blob(pairs: &[(&str, serde_json::Value)]) -> Vec<u8> {
        let map: serde_json::Map<String, serde_json::Value> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), v.clone()))
            .collect();
        serde_json::to_vec(&serde_json::Value::Object(map)).unwrap_or_default()
    }

    fn variant(blob: Vec<u8>) -> ConfigVariant {
        ConfigVariant {
            variant_id: "v-test".to_string(),
            parent_id: None,
            score: 0.0,
            children_count: 0,
            config_blob: blob,
            created_at: 0,
            pipeline_outcomes: Vec::new(),
            pipeline_mode: crate::self_mod_layer1a::PipelineMode::Surgical,
            status: crate::self_mod_types::VariantStatus::Active,
        }
    }

    #[test]
    fn clean_variant_yields_no_conflicts() {
        let cv = variant(json_blob(&[(
            "freshness_half_life",
            serde_json::json!(7.0),
        )]));
        let intent = intent_from_config_variant(&cv);
        let model = current_process_model(&[]);
        assert!(detect_conflicts(&intent, &model).is_empty());
    }

    #[test]
    fn locked_key_mutation_is_contradictory() {
        let cv = variant(json_blob(&[("score_weights", serde_json::json!("tampered"))]));
        let intent = intent_from_config_variant(&cv);
        let model = current_process_model(&[]);
        let conflicts = detect_conflicts(&intent, &model);
        assert!(conflicts
            .iter()
            .any(|c| c.kind == ConflictKind::Contradictory && c.key == "score_weights"));
    }

    #[test]
    fn locked_mutation_during_tripwire_adds_causal_violation() {
        let cv = variant(json_blob(&[("auditor_config", serde_json::json!("x"))]));
        let intent = intent_from_config_variant(&cv);
        let model = current_process_model(&[TripwireKind::Ddm]);
        let conflicts = detect_conflicts(&intent, &model);
        assert!(conflicts
            .iter()
            .any(|c| c.kind == ConflictKind::CausalViolation));
    }

    #[test]
    fn require_on_resource_rail_is_contention() {
        let cv = variant(json_blob(&[(
            "cloud_monthly_budget",
            serde_json::json!(99999),
        )]));
        let intent = intent_from_config_variant(&cv);
        let model = current_process_model(&[]);
        let conflicts = detect_conflicts(&intent, &model);
        assert!(conflicts
            .iter()
            .any(|c| c.kind == ConflictKind::ResourceContention));
    }

    #[test]
    fn fingerprint_is_deterministic_16_hex() {
        let a = conflict_fingerprint(ConflictKind::Contradictory, "score_weights", "d");
        let b = conflict_fingerprint(ConflictKind::Contradictory, "score_weights", "d");
        assert_eq!(a, b);
        assert_eq!(a.len(), CONFLICT_FINGERPRINT_HEX_LEN);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn tripwire_quiescence_any_nonempty() {
        assert!(!tripwire_quiescent_violated(&[]));
        assert!(tripwire_quiescent_violated(&[TripwireKind::Perplexity]));
    }
}
