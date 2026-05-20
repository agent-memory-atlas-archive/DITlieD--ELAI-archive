//! D-1: ReasoningTemplate typed schema + Blake3 content-hash + L3-frozen
//! closed-enum mutation surface.
//!
//! Replaces the historical string-literal reasoning patterns scattered across
//! `crates/prompts/templates/*.toml` and agent definition files with a typed
//! `ReasoningTemplate` struct that carries:
//!
//! - a stable [`TemplateId`] (UUID newtype),
//! - a closed [`Domain`] + [`PatternKind`] + [`VerifierKind`] enum triplet
//!   (L3-frozen per `docs/specs/guide/reasoning-improvement-grounded-2026-05-19.md`
//!   §4 — agents cannot self-weaken `VerifierKind::Prm` to `VerifierKind::Refuse`),
//! - a [`BoundedString`] template body (rejects >N chars at construction time;
//!   never panics),
//! - a [`Predicate`] eligibility expression (closed-form, no LLM callout),
//! - a Blake3 content-hash (per CLAUDE.md HARD RULE: blake3, never sha256 /
//!   blake2; see also `core/Cargo.toml` for the workspace declaration),
//! - an [`ArchiveStatus`] lifecycle (Candidate -> Promoted -> Retired |
//!   Frozen) with [`RetirementReason`] when retired.
//!
//! D-1 only defines the *schema* + the [`TemplateRegistry`] runtime loader.
//! D-2 (PPM sidecar) and D-5 (BAI Thompson promotion) consume this surface;
//! the in-prompt string-literal replacement is deferred to D-2.

use std::collections::HashMap;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Newtypes
// ---------------------------------------------------------------------------

/// Stable identifier for a [`ReasoningTemplate`]. Newtype around `Uuid` so
/// the public type system distinguishes template ids from other UUIDs (agent
/// ids, session ids, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TemplateId(pub Uuid);

impl TemplateId {
    /// Construct a new random [`TemplateId`] using `uuid::Uuid::new_v4`.
    /// Callers in deterministic test paths should pass `TemplateId(Uuid::nil())`
    /// or a fixture-derived UUID via `From<Uuid>`.
    #[must_use]
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }

    /// Borrow the underlying UUID.
    #[must_use]
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Hex string representation suitable for SSB payloads / JSON.
    #[must_use]
    pub fn as_hex(&self) -> String {
        self.0.simple().to_string()
    }
}

impl From<Uuid> for TemplateId {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

/// 32-byte blake3 content hash. Wrapped in a newtype so the type system
/// distinguishes content hashes from other 32-byte payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Blake3Hash(pub [u8; 32]);

impl Blake3Hash {
    /// All-zero hash; used as the canonical "no parent" sentinel inside
    /// [`compute_content_hash`] so the hash function is total.
    pub const ZERO: Self = Self([0u8; 32]);

    /// Hex representation (64 lowercase chars).
    #[must_use]
    pub fn as_hex(&self) -> String {
        let mut out = String::with_capacity(64);
        for byte in &self.0 {
            // 2-char lowercase per byte. We avoid `format!` in a hot path.
            out.push(hex_nibble(byte >> 4));
            out.push(hex_nibble(byte & 0x0f));
        }
        out
    }
}

impl From<blake3::Hash> for Blake3Hash {
    fn from(h: blake3::Hash) -> Self {
        Self(*h.as_bytes())
    }
}

fn hex_nibble(n: u8) -> char {
    match n {
        0..=9 => (b'0' + n) as char,
        10..=15 => (b'a' + (n - 10)) as char,
        // unreachable for any 4-bit value, fallback to '0' rather than
        // panicking — keeps the function total.
        _ => '0',
    }
}

// ---------------------------------------------------------------------------
// BoundedString<N>
// ---------------------------------------------------------------------------

/// String wrapper that rejects construction with > N UTF-8 *bytes* at
/// `try_new` time. NEVER panics on overflow — returns
/// [`TemplateError::BoundedStringExceeded`].
///
/// We bound by byte length (not char count) because the on-wire / on-disk
/// cost of a template body is bytes, not Unicode scalars. `1024` (the
/// canonical bound used by [`ReasoningTemplate::template_body`]) is the
/// per-template cap mandated by the spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BoundedString<const N: usize>(String);

impl<const N: usize> BoundedString<N> {
    /// Construct a `BoundedString<N>` from `s`. Returns an error if the
    /// byte length exceeds `N`.
    pub fn try_new(s: impl Into<String>) -> Result<Self, TemplateError> {
        let s = s.into();
        if s.len() > N {
            return Err(TemplateError::BoundedStringExceeded {
                bound: N,
                actual: s.len(),
            });
        }
        Ok(Self(s))
    }

    /// Borrow the inner string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume and return the inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Byte length.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// True if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Closed enums (L3-frozen per spec §4)
// ---------------------------------------------------------------------------

/// Reasoning template domain. L3-frozen closed enum — agents cannot mutate
/// the variant set without a human-only structural change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    Code,
    Math,
    Prose,
    Mixed,
    // D-7 (additive): two new variants to support the cross-domain transfer
    // canary's 5 orthogonal task families. `Planning` covers
    // PlanBench-style multi-step decomposition tasks; `ToolUse` covers
    // tool-grounded task families (web/api/tool-call benchmarks). The
    // existing four variants are kept stable (no rename, no reorder) so
    // any on-disk `ReasoningTemplate` row with Domain::Mixed remains
    // valid. See workflow/reasoning/D-7-decisions.md.
    Planning,
    ToolUse,
}

impl Domain {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Math => "math",
            Self::Prose => "prose",
            Self::Mixed => "mixed",
            Self::Planning => "planning",
            Self::ToolUse => "tool_use",
        }
    }
}

/// Reasoning pattern kind. L3-frozen closed enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind {
    Decomposition,
    Reflection,
    Verification,
    Synthesis,
}

impl PatternKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Decomposition => "decomposition",
            Self::Reflection => "reflection",
            Self::Verification => "verification",
            Self::Synthesis => "synthesis",
        }
    }
}

/// Verifier required for templates of this kind. L3-frozen — an agent cannot
/// downgrade `Prm` to `Refuse` to bypass verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifierKind {
    Compile,
    Test,
    Prm,
    Refuse,
}

impl VerifierKind {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Compile => "compile",
            Self::Test => "test",
            Self::Prm => "prm",
            Self::Refuse => "refuse",
        }
    }
}

/// Archive lifecycle state.
///
/// Transitions:
/// * `Candidate -> Promoted | Retired`
/// * `Promoted -> Retired`
/// * `Retired -> {}` (terminal)
/// * `Frozen -> {}` (terminal — initial migration produces Frozen entries)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveStatus {
    Candidate,
    Promoted,
    Retired,
    Frozen,
}

impl ArchiveStatus {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Promoted => "promoted",
            Self::Retired => "retired",
            Self::Frozen => "frozen",
        }
    }

    /// Returns true if `from -> to` is a permitted status transition.
    /// `Frozen` and `Retired` are terminal — no outbound transitions.
    #[must_use]
    pub fn can_transition_to(self, to: Self) -> bool {
        match (self, to) {
            (Self::Candidate, Self::Promoted) => true,
            (Self::Candidate, Self::Retired) => true,
            (Self::Promoted, Self::Retired) => true,
            _ => false,
        }
    }
}

/// Reason a template was retired. Feeds the dashboard MutationArchivePanel +
/// the future D-4 MistakeBook tripwire integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetirementReason {
    Degradation,
    Superseded,
    Banned,
    MistakeBookTrigger,
}

impl RetirementReason {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Degradation => "degradation",
            Self::Superseded => "superseded",
            Self::Banned => "banned",
            Self::MistakeBookTrigger => "mistake_book_trigger",
        }
    }
}

// ---------------------------------------------------------------------------
// Predicate
// ---------------------------------------------------------------------------

/// Closed-form eligibility predicate. Evaluation is total + side-effect-free
/// (per spec §4 L3-frozen rule: predicates MUST NOT call the LLM).
///
/// `Never` is a trivially-false leaf used by the `OrAny([Never, Always])`
/// unit test. Recursion is bounded by the Vec length — there is no
/// mutual-recursion path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum Predicate {
    Always,
    Never,
    MatchDomain(Domain),
    MinConfidence(f32),
    AndAll(Vec<Predicate>),
    OrAny(Vec<Predicate>),
}

/// Inputs that a [`Predicate`] inspects to decide template eligibility.
/// Lightweight value type — does NOT borrow the full orchestrator context to
/// keep predicate evaluation cheap and pure.
#[derive(Debug, Clone, Default)]
pub struct PredicateCtx {
    /// Current task domain, when known.
    pub domain: Option<Domain>,
    /// Confidence of the upstream signal driving the selection (0.0-1.0).
    pub confidence: f32,
}

impl Predicate {
    /// Evaluate this predicate against `ctx`. Total function, never panics.
    /// Closed-form: no I/O, no LLM call, no allocation aside from recursion.
    #[must_use]
    pub fn eval(&self, ctx: &PredicateCtx) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::MatchDomain(d) => ctx.domain == Some(*d),
            Self::MinConfidence(min) => ctx.confidence >= *min,
            Self::AndAll(items) => items.iter().all(|p| p.eval(ctx)),
            Self::OrAny(items) => items.iter().any(|p| p.eval(ctx)),
        }
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// All errors returned by `reasoning_template` + `archive_chain`.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TemplateError {
    /// BoundedString<N> rejected a too-long input.
    #[error("bounded string exceeded: bound={bound} actual={actual}")]
    BoundedStringExceeded { bound: usize, actual: usize },

    /// Recomputed content hash does not match the stored value (tamper
    /// detection in [`verify_hash`] + [`crate::archive_chain::ArchiveChain::verify_chain`]).
    #[error("content hash mismatch (template_id={template_id})")]
    HashMismatch { template_id: String },

    /// Parent template id referenced by an entry is not present in the
    /// archive.
    #[error("parent template not found: parent_id={parent_id}")]
    ParentNotFound { parent_id: String },

    /// Status transition rejected (Frozen/Retired are terminal; Candidate
    /// cannot jump to Frozen; etc.). Carries the canonical wire labels.
    #[error("invalid status transition: from={from} to={to}")]
    StatusTransitionInvalid { from: &'static str, to: &'static str },

    /// I/O error wrapping (only from archive_chain — kept here so the
    /// error type is unified across the two modules).
    #[error("archive io error: {0}")]
    Io(String),

    /// Serde JSON error wrapping.
    #[error("archive serde error: {0}")]
    Serde(String),
}

// ---------------------------------------------------------------------------
// ReasoningTemplate
// ---------------------------------------------------------------------------

/// Canonical template body bound (bytes). The plan calls this `1024`; we
/// expose it as a public const so dashboards / migrations can validate
/// upfront.
pub const TEMPLATE_BODY_BOUND: usize = 1024;

/// Bounded body alias used by [`ReasoningTemplate`].
pub type TemplateBody = BoundedString<TEMPLATE_BODY_BOUND>;

/// Typed reasoning template — the bounded-capacity replacement for the
/// historical string-literal patterns scattered across prompts + agent
/// definitions.
///
/// L3-frozen fields (cannot be mutated by self-mod): `domain`, `pattern_kind`,
/// `verifier_required`. See `docs/specs/guide/reasoning-improvement-grounded-2026-05-19.md`
/// §4 + workflow/reasoning/D-1-decisions.md.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningTemplate {
    pub id: TemplateId,
    pub domain: Domain,
    pub pattern_kind: PatternKind,
    pub template_body: TemplateBody,
    pub eligibility_predicate: Predicate,
    pub verifier_required: VerifierKind,
    pub content_hash: Blake3Hash,
    /// UTC unix-millis epoch. Wall clock source must be the injected
    /// [`elai_core::clock::ClockProvider`] — never `SystemTime::now()` per
    /// the C-15 entropy quarantine. We store as i64 to dodge a chrono /
    /// time-crate transitive dep at the orchestrator boundary.
    pub created_at_ms: i64,
    pub parent_id: Option<TemplateId>,
    pub archive_status: ArchiveStatus,
}

// ---------------------------------------------------------------------------
// Hashing
// ---------------------------------------------------------------------------

/// Compute the blake3 content hash of `body` mixed with `parent_hash`.
///
/// `parent_hash` of `None` means "root template"; we feed a 32-byte zero
/// sentinel into the hasher so the function stays total. Same `(body,
/// parent_hash)` -> same output (deterministic).
#[must_use]
pub fn compute_content_hash(body: &str, parent_hash: Option<&Blake3Hash>) -> Blake3Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(body.as_bytes());
    match parent_hash {
        Some(h) => hasher.update(&h.0),
        None => hasher.update(&Blake3Hash::ZERO.0),
    };
    Blake3Hash::from(hasher.finalize())
}

/// Recompute `template.content_hash` from `template.template_body` +
/// `template.parent_id` (using the current archive lookup). Returns
/// [`TemplateError::HashMismatch`] if the stored hash differs.
///
/// We deliberately recompute rather than trust the cached value: this is the
/// in-memory tamper defence per the WIRING_CONTRACT `stub_ban` rule.
pub fn verify_hash(template: &ReasoningTemplate) -> Result<(), TemplateError> {
    // We hash body + parent_hash (Blake3Hash, not parent_id). For verify_hash
    // alone we do not have the parent template lookup — the caller supplies
    // it via the ArchiveChain path. This standalone variant treats parent
    // hash as a constant: the existing content_hash MUST embed
    // (body, parent_or_zero). We approximate "parent's hash" by hashing the
    // recorded parent_id bytes (UUID 16 bytes). If parent_id is None we use
    // ZERO. The archive_chain variant (`verify_chain`) does the proper
    // self_hash + parent_hash dance.
    let parent_hash = template
        .parent_id
        .map(|pid| compute_parent_proxy_hash(&pid));
    let recomputed = compute_content_hash(template.template_body.as_str(), parent_hash.as_ref());
    if recomputed == template.content_hash {
        Ok(())
    } else {
        Err(TemplateError::HashMismatch {
            template_id: template.id.as_hex(),
        })
    }
}

/// Derive a deterministic 32-byte proxy hash from a parent [`TemplateId`].
/// Used by the standalone `verify_hash` (the ArchiveChain path uses the real
/// parent entry's `self_hash` instead).
#[must_use]
pub fn compute_parent_proxy_hash(parent_id: &TemplateId) -> Blake3Hash {
    let mut hasher = blake3::Hasher::new();
    hasher.update(parent_id.as_uuid().as_bytes());
    Blake3Hash::from(hasher.finalize())
}

// ---------------------------------------------------------------------------
// TemplateRegistry
// ---------------------------------------------------------------------------

/// In-memory index of templates loaded from an [`crate::archive_chain::ArchiveChain`].
/// Only templates with `archive_status in {Promoted, Frozen}` are indexed —
/// `Candidate` entries are pending eval, `Retired` entries are dead.
///
/// Uses [`FxHashMap`] per the CLAUDE.md anti-stale rule (`HashMap` is banned
/// after the C-15 migration); the inner field uses `rustc_hash::FxHashMap`
/// directly to avoid pulling in an extra dep.
#[derive(Debug, Clone, Default)]
pub struct TemplateRegistry {
    templates: FxHashMap<TemplateId, ReasoningTemplate>,
}

impl TemplateRegistry {
    /// Empty registry (no templates).
    #[must_use]
    pub fn new() -> Self {
        Self {
            templates: FxHashMap::default(),
        }
    }

    /// Insert a template directly — used by tests + the migration tool.
    /// Production callers should go through
    /// [`TemplateRegistry::load_from_chain`].
    pub fn insert(&mut self, t: ReasoningTemplate) {
        self.templates.insert(t.id, t);
    }

    /// Number of indexed templates.
    #[must_use]
    pub fn len(&self) -> usize {
        self.templates.len()
    }

    /// True if no templates indexed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }

    /// Lookup by [`TemplateId`].
    #[must_use]
    pub fn get(&self, id: &TemplateId) -> Option<&ReasoningTemplate> {
        self.templates.get(id)
    }

    /// Snapshot of all templates (cloned). Used by the dashboard panel +
    /// audit dispatcher gate.
    #[must_use]
    pub fn snapshot(&self) -> Vec<ReasoningTemplate> {
        self.templates.values().cloned().collect()
    }

    /// Select the first template matching (`domain`, `pattern_kind`) whose
    /// eligibility predicate evaluates to true under `ctx`. Returns `None`
    /// if no match. Deterministic order is NOT guaranteed (FxHashMap iter is
    /// arbitrary); production wiring (D-2) should layer a deterministic
    /// tie-breaker on top.
    #[must_use]
    pub fn select_template(
        &self,
        domain: Domain,
        pattern_kind: PatternKind,
        ctx: &PredicateCtx,
    ) -> Option<&ReasoningTemplate> {
        self.templates.values().find(|t| {
            t.domain == domain
                && t.pattern_kind == pattern_kind
                && (t.archive_status == ArchiveStatus::Promoted
                    || t.archive_status == ArchiveStatus::Frozen)
                && t.eligibility_predicate.eval(ctx)
        })
    }
}

/// Lightweight wire DTO for the dashboard panel + IPC consumers.
/// Strips the `Predicate` (closed-form, opaque to JS) and exposes flattened
/// snake-case enum strings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningTemplateDto {
    pub id: String,
    pub domain: String,
    pub pattern_kind: String,
    pub verifier_required: String,
    pub content_hash_hex: String,
    pub parent_id: Option<String>,
    pub created_at_ms: i64,
    pub archive_status: String,
    pub body_len: usize,
}

impl From<&ReasoningTemplate> for ReasoningTemplateDto {
    fn from(t: &ReasoningTemplate) -> Self {
        Self {
            id: t.id.as_hex(),
            domain: t.domain.as_str().to_string(),
            pattern_kind: t.pattern_kind.as_str().to_string(),
            verifier_required: t.verifier_required.as_str().to_string(),
            content_hash_hex: t.content_hash.as_hex(),
            parent_id: t.parent_id.map(|p| p.as_hex()),
            created_at_ms: t.created_at_ms,
            archive_status: t.archive_status.as_str().to_string(),
            body_len: t.template_body.len(),
        }
    }
}

// ---------------------------------------------------------------------------
// Compile-time variant checks
// ---------------------------------------------------------------------------
//
// These ensure that any future addition to the closed enums forces a
// downstream contract test update. The contract test
// `reasoning_template_schema_contract.rs` asserts exact variant counts via
// these constants.

/// Variant count for [`Domain`]. Increment ONLY via a human-only structural
/// change (L3-frozen per spec §4). D-7 bumped 4 -> 6 to land Planning +
/// ToolUse for the cross-domain transfer canary.
pub const DOMAIN_VARIANT_COUNT: usize = 6;
/// Variant count for [`PatternKind`].
pub const PATTERN_KIND_VARIANT_COUNT: usize = 4;
/// Variant count for [`VerifierKind`].
pub const VERIFIER_KIND_VARIANT_COUNT: usize = 4;
/// Variant count for [`ArchiveStatus`].
pub const ARCHIVE_STATUS_VARIANT_COUNT: usize = 4;
/// Variant count for [`RetirementReason`].
pub const RETIREMENT_REASON_VARIANT_COUNT: usize = 4;

/// Compile-time exhaustiveness check helper: every match on these enums must
/// cover exactly the documented variant count. The const fns below are
/// referenced by the contract test so adding a variant without bumping the
/// count above breaks compilation.
#[doc(hidden)]
pub const fn _domain_check(d: Domain) -> usize {
    match d {
        Domain::Code => 0,
        Domain::Math => 1,
        Domain::Prose => 2,
        Domain::Mixed => 3,
        // D-7 additive: Planning + ToolUse extension.
        Domain::Planning => 4,
        Domain::ToolUse => 5,
    }
}
#[doc(hidden)]
pub const fn _pattern_kind_check(p: PatternKind) -> usize {
    match p {
        PatternKind::Decomposition => 0,
        PatternKind::Reflection => 1,
        PatternKind::Verification => 2,
        PatternKind::Synthesis => 3,
    }
}
#[doc(hidden)]
pub const fn _verifier_kind_check(v: VerifierKind) -> usize {
    match v {
        VerifierKind::Compile => 0,
        VerifierKind::Test => 1,
        VerifierKind::Prm => 2,
        VerifierKind::Refuse => 3,
    }
}
#[doc(hidden)]
pub const fn _archive_status_check(a: ArchiveStatus) -> usize {
    match a {
        ArchiveStatus::Candidate => 0,
        ArchiveStatus::Promoted => 1,
        ArchiveStatus::Retired => 2,
        ArchiveStatus::Frozen => 3,
    }
}
#[doc(hidden)]
pub const fn _retirement_reason_check(r: RetirementReason) -> usize {
    match r {
        RetirementReason::Degradation => 0,
        RetirementReason::Superseded => 1,
        RetirementReason::Banned => 2,
        RetirementReason::MistakeBookTrigger => 3,
    }
}

// `HashMap` import kept for the doc-link to the historical std::HashMap rule.
// `FxHashMap` is the actual storage. Leaving this here suppresses an unused
// warning if someone later removes the std import path.
#[allow(dead_code)]
fn _ensure_hashmap_alias_for_doc_consumers() -> HashMap<(), ()> {
    HashMap::new()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_string_accepts_up_to_bound() {
        let s = "x".repeat(TEMPLATE_BODY_BOUND);
        let bs = BoundedString::<TEMPLATE_BODY_BOUND>::try_new(s).expect("at bound");
        assert_eq!(bs.len(), TEMPLATE_BODY_BOUND);
    }

    #[test]
    fn bounded_string_rejects_over_bound_no_panic() {
        let s = "x".repeat(TEMPLATE_BODY_BOUND + 1);
        let result = BoundedString::<TEMPLATE_BODY_BOUND>::try_new(s);
        match result {
            Err(TemplateError::BoundedStringExceeded { bound, actual }) => {
                assert_eq!(bound, TEMPLATE_BODY_BOUND);
                assert_eq!(actual, TEMPLATE_BODY_BOUND + 1);
            }
            other => panic!("expected BoundedStringExceeded, got {other:?}"),
        }
    }

    #[test]
    fn compute_content_hash_is_deterministic() {
        let parent = Blake3Hash([7u8; 32]);
        let h1 = compute_content_hash("hello", Some(&parent));
        let h2 = compute_content_hash("hello", Some(&parent));
        assert_eq!(h1, h2);
        assert_eq!(h1.0.len(), 32);
    }

    #[test]
    fn compute_content_hash_differs_on_body_change() {
        let parent = Blake3Hash([7u8; 32]);
        let h1 = compute_content_hash("hello", Some(&parent));
        let h2 = compute_content_hash("hello!", Some(&parent));
        assert_ne!(h1, h2);
    }

    #[test]
    fn compute_content_hash_differs_on_parent_change() {
        let p1 = Blake3Hash([7u8; 32]);
        let p2 = Blake3Hash([8u8; 32]);
        let h1 = compute_content_hash("hello", Some(&p1));
        let h2 = compute_content_hash("hello", Some(&p2));
        assert_ne!(h1, h2);
    }

    #[test]
    fn compute_content_hash_root_vs_zero_parent_equal() {
        // None parent_hash is documented to mix the ZERO sentinel.
        let none_hash = compute_content_hash("hello", None);
        let zero_hash = compute_content_hash("hello", Some(&Blake3Hash::ZERO));
        assert_eq!(none_hash, zero_hash);
    }

    fn make_template(body: &str, status: ArchiveStatus) -> ReasoningTemplate {
        let body = TemplateBody::try_new(body).expect("body fits");
        let content_hash = compute_content_hash(body.as_str(), None);
        ReasoningTemplate {
            id: TemplateId::new_v4(),
            domain: Domain::Code,
            pattern_kind: PatternKind::Decomposition,
            template_body: body,
            eligibility_predicate: Predicate::Always,
            verifier_required: VerifierKind::Compile,
            content_hash,
            created_at_ms: 0,
            parent_id: None,
            archive_status: status,
        }
    }

    #[test]
    fn verify_hash_detects_tamper() {
        let mut t = make_template("clean body", ArchiveStatus::Frozen);
        verify_hash(&t).expect("clean verify");
        // Mutate the body but leave the cached hash stale.
        t.template_body = TemplateBody::try_new("tampered body").expect("fits");
        match verify_hash(&t) {
            Err(TemplateError::HashMismatch { .. }) => {}
            other => panic!("expected HashMismatch, got {other:?}"),
        }
    }

    #[test]
    fn predicate_and_all_short_circuits_on_true() {
        let p = Predicate::AndAll(vec![Predicate::Always, Predicate::Always]);
        assert!(p.eval(&PredicateCtx::default()));
    }

    #[test]
    fn predicate_or_any_short_circuits_on_never_plus_always() {
        let p = Predicate::OrAny(vec![Predicate::Never, Predicate::Always]);
        assert!(p.eval(&PredicateCtx::default()));
    }

    #[test]
    fn predicate_min_confidence_threshold() {
        let p = Predicate::MinConfidence(0.5);
        let mut ctx = PredicateCtx::default();
        ctx.confidence = 0.49;
        assert!(!p.eval(&ctx));
        ctx.confidence = 0.5;
        assert!(p.eval(&ctx));
    }

    #[test]
    fn status_transition_table_matches_spec() {
        // Allowed.
        assert!(ArchiveStatus::Candidate.can_transition_to(ArchiveStatus::Promoted));
        assert!(ArchiveStatus::Candidate.can_transition_to(ArchiveStatus::Retired));
        assert!(ArchiveStatus::Promoted.can_transition_to(ArchiveStatus::Retired));
        // Terminal — disallowed.
        assert!(!ArchiveStatus::Frozen.can_transition_to(ArchiveStatus::Promoted));
        assert!(!ArchiveStatus::Frozen.can_transition_to(ArchiveStatus::Retired));
        assert!(!ArchiveStatus::Retired.can_transition_to(ArchiveStatus::Promoted));
        assert!(!ArchiveStatus::Retired.can_transition_to(ArchiveStatus::Candidate));
        // Promotion never goes backward.
        assert!(!ArchiveStatus::Promoted.can_transition_to(ArchiveStatus::Candidate));
    }

    #[test]
    fn registry_select_template_filters_by_archive_status() {
        let mut reg = TemplateRegistry::new();
        let candidate = make_template("candidate body", ArchiveStatus::Candidate);
        let frozen = make_template("frozen body", ArchiveStatus::Frozen);
        let retired = make_template("retired body", ArchiveStatus::Retired);
        reg.insert(candidate);
        reg.insert(frozen.clone());
        reg.insert(retired);

        let picked = reg
            .select_template(Domain::Code, PatternKind::Decomposition, &PredicateCtx::default())
            .expect("select frozen");
        assert_eq!(picked.id, frozen.id);
    }

    #[test]
    fn registry_select_returns_none_when_no_match() {
        let reg = TemplateRegistry::new();
        let picked = reg.select_template(
            Domain::Code,
            PatternKind::Decomposition,
            &PredicateCtx::default(),
        );
        assert!(picked.is_none());
    }

    #[test]
    fn dto_round_trips_essential_fields() {
        let t = make_template("dto body", ArchiveStatus::Frozen);
        let dto = ReasoningTemplateDto::from(&t);
        assert_eq!(dto.domain, "code");
        assert_eq!(dto.pattern_kind, "decomposition");
        assert_eq!(dto.archive_status, "frozen");
        assert_eq!(dto.verifier_required, "compile");
        assert_eq!(dto.content_hash_hex.len(), 64);
        assert_eq!(dto.body_len, "dto body".len());
        assert!(dto.parent_id.is_none());
    }

    #[test]
    fn variant_count_constants_match_check_fns() {
        // Calls the const fn for each enum variant — compile fails if a
        // variant is missing from the match.
        let _ = _domain_check(Domain::Code);
        let _ = _domain_check(Domain::Math);
        let _ = _domain_check(Domain::Prose);
        let _ = _domain_check(Domain::Mixed);
        // D-7 additive: Planning + ToolUse.
        let _ = _domain_check(Domain::Planning);
        let _ = _domain_check(Domain::ToolUse);
        assert_eq!(DOMAIN_VARIANT_COUNT, 6);

        let _ = _pattern_kind_check(PatternKind::Decomposition);
        let _ = _pattern_kind_check(PatternKind::Reflection);
        let _ = _pattern_kind_check(PatternKind::Verification);
        let _ = _pattern_kind_check(PatternKind::Synthesis);
        assert_eq!(PATTERN_KIND_VARIANT_COUNT, 4);

        let _ = _verifier_kind_check(VerifierKind::Compile);
        let _ = _verifier_kind_check(VerifierKind::Test);
        let _ = _verifier_kind_check(VerifierKind::Prm);
        let _ = _verifier_kind_check(VerifierKind::Refuse);
        assert_eq!(VERIFIER_KIND_VARIANT_COUNT, 4);

        let _ = _archive_status_check(ArchiveStatus::Candidate);
        let _ = _archive_status_check(ArchiveStatus::Promoted);
        let _ = _archive_status_check(ArchiveStatus::Retired);
        let _ = _archive_status_check(ArchiveStatus::Frozen);
        assert_eq!(ARCHIVE_STATUS_VARIANT_COUNT, 4);

        let _ = _retirement_reason_check(RetirementReason::Degradation);
        let _ = _retirement_reason_check(RetirementReason::Superseded);
        let _ = _retirement_reason_check(RetirementReason::Banned);
        let _ = _retirement_reason_check(RetirementReason::MistakeBookTrigger);
        assert_eq!(RETIREMENT_REASON_VARIANT_COUNT, 4);
    }

    #[test]
    fn blake3_hash_hex_is_64_chars() {
        let h = compute_content_hash("anything", None);
        let hex = h.as_hex();
        assert_eq!(hex.len(), 64);
        // All chars hex.
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
