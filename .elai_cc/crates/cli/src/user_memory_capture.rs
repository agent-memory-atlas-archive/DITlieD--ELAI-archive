//! H-31 (SCHEMA_VERSION 240): the `/remember` user-decision capture producer -- the FIRST
//! production caller of `elai_memory::dispatch_one`, activating the wire-dark bi-temporal
//! user-memory substrate into the LIVE `/remember` + `elai remember "<text>"` path.
//!
//! Flow (EXPLICIT-only -- no silent auto-write):
//!   1. Run the secret/PII reject FIRST (`elai_memory::classify`, the shipped `defense.rs` seam).
//!      A captured secret/credential is REJECTED, NEVER stored (both `Block` and `SkipSensitive`
//!      verdicts => not stored). Storing a value matching the reject is a stub_ban violation.
//!   2. Build a single `ExtractedFact` (a durable WORK `Preference`) from the user text. The
//!      shipped `extract_facts` heuristic is tried first (it yields stable `prefers_X` keys);
//!      free-form text that matches no pattern falls back to a deterministic blake3-keyed
//!      `remembered_<hash>` Preference so an arbitrary `/remember <text>` still captures.
//!   3. Call `elai_memory::dispatch_one` against the LIVE `FactStore` (the same handle convention
//!      the live tick uses: `memory_dir_path/memory_facts.sqlite`, `user_id="default"`,
//!      `scope="global"`). The dispatcher's reconcile gives ADD (no prior row) or
//!      UPDATE-via-`supersede` (a contradicting decision on the same key soft-closes the prior row,
//!      `old.valid_to_ms == new.valid_from_ms`, newest-wins; DELETE is BANNED for SSB replay).
//!   4. Read the returned `DispatchOp` and emit the off-agent `UserDecisionSuperseded` SSB row
//!      (`Add` => `Captured`, `Update` => `Superseded`), carrying the fact_id pair + fact_type +
//!      key -- NEVER the value (no secrets in the journal).
//!
//! Gated behind `ELAI_USER_MEMORY_ENABLE` (R9, default-OFF). When OFF, capture is a no-op that
//! reports the flag is disabled.
//!
//! CLAUDE.md HARD RULES honoured: NO `.unwrap()`/`.expect()` in this module (every fallible op
//! propagates a typed error); NO `SystemTime::now()` (the wall clock comes from the injected
//! `ClockProvider`); blake3 (NOT blake2/sha2) for the fallback key + evidence fingerprint;
//! `.try_into()` for narrowing; named constants (no magic numbers); DELETE never issued.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::paths::{memory_dir_path, sessions_dir_path};
use elai_memory::{
    classify, dispatch_one, DefenseVerdict, DispatchContext, DispatchOp, ExtractedFact, FactStore,
    FactType, Provenance, Scope, TrustTier,
};
use elai_orchestrator::ssb::{
    Ssb, SsbConfig, SsbPayload, UserDecisionEventKind, UserDecisionSupersededPayload,
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// The R9 default-OFF kill-switch / experiment flag. When unset or not `"1"`, capture is a no-op.
pub(crate) const USER_MEMORY_FLAG: &str = "ELAI_USER_MEMORY_ENABLE";

/// The canonical user id for the local single-user store (mirrors the live tick convention at
/// `live_cli.rs` dispatch_chat_memory_reflect: `user_id="default"`). `pub(crate)` so the
/// retrieval-into-prompt path (report_fns) keys the same user.
pub(crate) const DEFAULT_USER_ID: &str = "default";

/// The canonical scope tag for `/remember` facts (global, cross-project; mirrors the live tick
/// `scope_tag="global"`).
const DEFAULT_SCOPE_TAG: &str = "global";

/// Project id stamped on the emitted SSB rows.
const CAPTURE_PROJECT_ID: &str = "elai-user-memory";

/// Session id stamped on the emitted SSB rows.
const CAPTURE_SESSION_ID: &str = "elai-user-memory-capture";

/// SSB source label for the capture producer (the `agent_id` of the emitted row).
const CAPTURE_SSB_SOURCE: &str = "cli.user_memory_capture";

/// Initial trust score for a newly captured user-direct preference. The user typed it explicitly
/// (the highest-trust provenance), so it starts at the top of the `[0,1]` band.
const INITIAL_TRUST_SCORE: f32 = 1.0;

/// Confidence for a fallback (heuristic-miss) captured preference. The user was explicit, so it is
/// high but below the `i prefer X over Y` heuristic's 0.85.
const FALLBACK_CONFIDENCE: f32 = 0.9;

/// Hex prefix length of the blake3 fallback key slug (keeps the key short + stable).
const FALLBACK_KEY_HEX_LEN: usize = 16;

/// The SSB superstep for fire-and-forget capture rows (single-shot; not a stepped run).
const CAPTURE_SUPERSTEP: i64 = 0;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the `/remember` capture producer.
#[derive(Debug, Error)]
pub(crate) enum CaptureError {
    /// The text was empty after trimming.
    #[error("/remember requires non-empty text")]
    EmptyText,
    /// The text matched the secret/PII reject -- NEVER stored.
    #[error("/remember rejected: the text matches a secret/PII pattern ({0}) and was NOT stored")]
    SecretRejected(String),
    /// I/O error creating the store/SSB directory.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    /// FactStore open / dispatch error.
    #[error("memory store error: {0}")]
    Memory(#[from] elai_memory::MemoryError),
    /// SSB initialization error.
    #[error("SSB initialization error: {0}")]
    Ssb(String),
}

/// The outcome of a capture, returned to the caller for human/JSON rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CaptureOutcome {
    /// The flag was OFF -- capture is a no-op.
    Disabled,
    /// A new preference was captured (ADD). Carries the new fact key.
    Captured {
        /// The captured fact's key.
        key: String,
    },
    /// A contradicting preference superseded a prior one (UPDATE via soft-close). Carries the key.
    Superseded {
        /// The superseded/replaced fact's key.
        key: String,
    },
    /// The candidate matched an existing row with no change (NOOP). Carries the key + reason.
    Noop {
        /// The matched fact's key.
        key: String,
        /// Snake_case reason tag from the dispatcher.
        reason: String,
    },
}

/// True iff the R9 flag is set to `"1"` (default-OFF).
#[must_use]
pub(crate) fn user_memory_enabled() -> bool {
    std::env::var(USER_MEMORY_FLAG).is_ok_and(|v| v == "1")
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Capture a `/remember <text>` user decision into the live bi-temporal FactStore + emit the
/// off-agent `UserDecisionSuperseded` SSB row.
///
/// Returns a [`CaptureOutcome`] describing the dispatcher decision. The clock is injected (NOT
/// `SystemTime::now()`). When `ELAI_USER_MEMORY_ENABLE` is not `"1"`, returns
/// [`CaptureOutcome::Disabled`] without touching the store.
pub(crate) fn capture_remember(text: &str) -> Result<CaptureOutcome, CaptureError> {
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    capture_remember_with_clock(text, &clock)
}

/// Clock-injected core (testable without `SystemTime`). All wall-clock reads go through `clock`.
pub(crate) fn capture_remember_with_clock(
    text: &str,
    clock: &Arc<dyn ClockProvider>,
) -> Result<CaptureOutcome, CaptureError> {
    if !user_memory_enabled() {
        return Ok(CaptureOutcome::Disabled);
    }

    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(CaptureError::EmptyText);
    }

    // STEP 2: build the candidate. (We build it BEFORE the secret reject so the reject runs over
    // the exact serialized candidate the dispatcher would store -- key + value haystack.)
    let candidate = build_candidate(trimmed);

    // STEP 1 (runs over the built candidate): secret/PII reject FIRST. A captured secret/credential
    // is REJECTED, NEVER stored. Both Block (attack) and SkipSensitive (policy) => not stored.
    match classify(&candidate) {
        DefenseVerdict::Allow => {}
        DefenseVerdict::Block { reason } | DefenseVerdict::SkipSensitive { reason } => {
            return Err(CaptureError::SecretRejected(reason));
        }
    }

    // Open the live FactStore (same convention as the live tick: memory_dir/memory_facts.sqlite).
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mem_dir = memory_dir_path(&project_root);
    std::fs::create_dir_all(&mem_dir)?;
    let facts_path = mem_dir.join("memory_facts.sqlite");
    let mut facts = FactStore::open(&facts_path)?;

    let now_ms: u128 = clock.now_ms();
    let ctx = DispatchContext {
        user_id: DEFAULT_USER_ID.to_string(),
        project_id: None,
        scope: Scope::Global,
        provenance: Provenance::user_direct(DEFAULT_SCOPE_TAG),
        trust_tier: TrustTier::T0UserDirect,
        initial_trust_score: INITIAL_TRUST_SCORE,
        now_ms,
    };

    // STEP 3: dispatch. ADD (no prior row) or UPDATE-via-supersede (contradicting decision on the
    // same key soft-closes the prior row; DELETE never issued).
    let op = dispatch_one(&mut facts, &candidate, &ctx)?;

    // STEP 4: emit the off-agent SSB row from the DispatchOp.
    let (outcome, payload) = op_to_outcome_and_payload(&op, &candidate);
    emit_ssb_row(clock, &project_root, payload);

    Ok(outcome)
}

// ---------------------------------------------------------------------------
// Candidate construction
// ---------------------------------------------------------------------------

/// Build a single `ExtractedFact` from the user text. Tries the shipped `extract_facts` heuristic
/// first (stable `prefers_X` keys); falls back to a deterministic blake3-keyed `remembered_<hash>`
/// Preference for free-form text that matches no pattern.
///
/// H-31 W-1 (topic-key for the two-option contradiction shape): the shipped `extract_facts`
/// heuristic keys an `"i prefer X over Y"` statement on the STANCE (`prefers_X_over_Y`), so the
/// genuine contradiction `"i prefer Y over X"` keys differently (`prefers_Y_over_X`) and ADDs a
/// second row instead of SUPERSEDING the prior. To make a real contradiction reach the supersede
/// arm, this producer re-keys the `prefer_over` shape ONLY onto the UNORDERED topic pair
/// (`topic_{a}_vs_{b}` with the two option slugs sorted lexicographically). The two contradicting
/// statements then collapse to the SAME key with a DIFFERENT value (the raw text + which option is
/// preferred flip), so the second `/remember` drives `dispatch_one`'s UPDATE-via-`supersede` arm
/// and emits a `Superseded` row. This is the narrowest sound rewrite: it touches ONLY the
/// two-named-option shape (where the topic is unambiguous -- exactly the pair of named options) and
/// leaves the single-option (`prefers_X`) and the free-form blake3 fallback keys UNCHANGED, so two
/// unrelated decisions can never false-merge. Residual (documented in H-31-decisions.md): a
/// single-option flip (`"i prefer rust"` then `"i prefer go"`) keeps distinct keys and does NOT
/// supersede -- a missed supersede is preferred over a false one that drops a distinct decision.
fn build_candidate(text: &str) -> ExtractedFact {
    use elai_memory::{extract_facts, Conversation, Turn};

    let conv = Conversation {
        session_id: CAPTURE_SESSION_ID.to_string(),
        turns: vec![Turn {
            message_id: "remember".to_string(),
            role: "user".to_string(),
            text: text.to_string(),
        }],
    };
    // extract_facts only errors on an empty session_id (we always pass a non-empty one), so an
    // error here is impossible; fall through to the deterministic fallback if it ever changes.
    if let Ok(mut facts) = extract_facts(&conv) {
        if let Some(mut first) = facts.drain(..).next() {
            rekey_prefer_over_to_topic(&mut first);
            return first;
        }
    }
    fallback_candidate(text)
}

/// Re-key an `"i prefer X over Y"` extracted fact onto the UNORDERED topic pair so a contradicting
/// `"i prefer Y over X"` collides on the same key and SUPERSEDES (H-31 W-1). Detection is exact: the
/// shipped heuristic stamps `value["matched"] == "prefer_over"` and carries string `subject` +
/// `alternative` fields (extractor.rs `prefer_over` arm). When BOTH option strings are present the
/// key is rewritten to `topic_{a}_vs_{b}` where `a`,`b` are the two slugged options sorted
/// lexicographically (so the order the user named them in does not matter). The `value` is left
/// intact (it still carries which option is preferred), so the contradiction is a CHANGED value on
/// the SAME key -- the supersede arm, not a NOOP. Any other shape (single-option `prefers_X`,
/// `dislikes_X`, or the fallback) is left UNTOUCHED -- no over-merge of unrelated facts.
fn rekey_prefer_over_to_topic(fact: &mut ExtractedFact) {
    if fact.value.get("matched").and_then(|m| m.as_str()) != Some("prefer_over") {
        return;
    }
    let subject = fact.value.get("subject").and_then(|s| s.as_str());
    let alternative = fact.value.get("alternative").and_then(|s| s.as_str());
    let (Some(subject), Some(alternative)) = (subject, alternative) else {
        return;
    };
    let mut options = [topic_slug(subject), topic_slug(alternative)];
    if options[0].is_empty() || options[1].is_empty() {
        return;
    }
    options.sort();
    fact.key = format!("topic_{}_vs_{}", options[0], options[1]);
}

/// Normalize an option string into a stable snake_case slug for the topic key: lowercased,
/// non-alphanumeric runs collapsed to a single underscore, leading/trailing underscores trimmed.
/// Deterministic and content-addressed (mirrors the extractor's own `slug` so the topic key is
/// stable across sessions). No new dependency.
fn topic_slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_under = false;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_under = false;
        } else if !prev_under {
            out.push('_');
            prev_under = true;
        }
    }
    out.trim_matches('_').to_string()
}

/// Deterministic fallback candidate: a `Preference` whose key is `remembered_<blake3-prefix>` over
/// the trimmed text so the same `/remember <text>` always reconciles to the same row (NOOP on a
/// repeat, Superseded only when the text changes for that key -- which, being content-derived,
/// means a genuinely different statement gets its own row; the heuristic path is what produces the
/// contradiction-on-same-key Superseded behaviour for `i prefer X` style statements).
fn fallback_candidate(text: &str) -> ExtractedFact {
    let hash = blake3::hash(text.as_bytes()).to_hex();
    let suffix = &hash.as_str()[..FALLBACK_KEY_HEX_LEN.min(hash.len())];
    ExtractedFact {
        fact_type: FactType::Preference,
        key: format!("remembered_{suffix}"),
        value: serde_json::json!({ "raw": text, "matched": "remember_fallback" }),
        confidence: FALLBACK_CONFIDENCE,
        evidence: vec![CAPTURE_SESSION_ID.to_string(), "remember".to_string()],
    }
}

// ---------------------------------------------------------------------------
// DispatchOp -> outcome + SSB payload
// ---------------------------------------------------------------------------

/// Map a `DispatchOp` + the candidate into a `CaptureOutcome` and the off-agent SSB payload.
/// `Add` => `Captured`, `Update` => `Superseded`, `Delete`/`Noop` => `Noop` outcome (tombstones are
/// not produced by `/remember`, so `Delete` maps to a Noop-shaped outcome for completeness).
fn op_to_outcome_and_payload(
    op: &DispatchOp,
    candidate: &ExtractedFact,
) -> (CaptureOutcome, SsbPayload) {
    let fact_type = candidate.fact_type.as_str().to_string();
    let key = candidate.key.clone();
    let evidence_blake3 = blake3::hash(
        format!("{DEFAULT_USER_ID}:{}:{}", fact_type, key).as_bytes(),
    )
    .to_hex()
    .to_string();

    match op {
        DispatchOp::Add { fact_id } => (
            CaptureOutcome::Captured { key: key.clone() },
            SsbPayload::UserDecisionSuperseded(UserDecisionSupersededPayload {
                kind: UserDecisionEventKind::Captured,
                old_fact_id: String::new(),
                new_fact_id: fact_id.clone(),
                fact_type,
                key,
                evidence_blake3,
            }),
        ),
        DispatchOp::Update {
            old_fact_id,
            new_fact_id,
        } => (
            CaptureOutcome::Superseded { key: key.clone() },
            SsbPayload::UserDecisionSuperseded(UserDecisionSupersededPayload {
                kind: UserDecisionEventKind::Superseded,
                old_fact_id: old_fact_id.clone(),
                new_fact_id: new_fact_id.clone(),
                fact_type,
                key,
                evidence_blake3,
            }),
        ),
        DispatchOp::Delete { fact_id } => (
            CaptureOutcome::Noop {
                key: key.clone(),
                reason: "soft_closed".to_string(),
            },
            SsbPayload::UserDecisionSuperseded(UserDecisionSupersededPayload {
                kind: UserDecisionEventKind::Superseded,
                old_fact_id: fact_id.clone(),
                new_fact_id: String::new(),
                fact_type,
                key,
                evidence_blake3,
            }),
        ),
        DispatchOp::Noop { reason, .. } => (
            CaptureOutcome::Noop {
                key: key.clone(),
                reason: reason.clone(),
            },
            SsbPayload::UserDecisionSuperseded(UserDecisionSupersededPayload {
                kind: UserDecisionEventKind::Captured,
                old_fact_id: String::new(),
                new_fact_id: String::new(),
                fact_type,
                key,
                evidence_blake3,
            }),
        ),
    }
}

/// Surface a propose-on-repeated-correction proposal (ProposalSurfaced) WITHOUT writing a fact.
/// Emits the off-agent SSB row only; the row is the only thing emitted unprompted. Returns the
/// emitted payload's key for the caller's surface text.
pub(crate) fn surface_proposal(clock: &Arc<dyn ClockProvider>, fact_type: &str, key: &str) {
    if !user_memory_enabled() {
        return;
    }
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let evidence_blake3 = blake3::hash(format!("{DEFAULT_USER_ID}:{fact_type}:{key}").as_bytes())
        .to_hex()
        .to_string();
    let payload = SsbPayload::UserDecisionSuperseded(UserDecisionSupersededPayload {
        kind: UserDecisionEventKind::ProposalSurfaced,
        old_fact_id: String::new(),
        new_fact_id: String::new(),
        fact_type: fact_type.to_string(),
        key: key.to_string(),
        evidence_blake3,
    });
    emit_ssb_row(clock, &project_root, payload);
}

/// Render a [`CaptureOutcome`] for the CLI (human text or `--json`).
pub(crate) fn print_capture_outcome(outcome: &CaptureOutcome, json: bool) {
    if json {
        let (status, key, reason) = match outcome {
            CaptureOutcome::Disabled => ("disabled", String::new(), String::new()),
            CaptureOutcome::Captured { key } => ("captured", key.clone(), String::new()),
            CaptureOutcome::Superseded { key } => ("superseded", key.clone(), String::new()),
            CaptureOutcome::Noop { key, reason } => ("noop", key.clone(), reason.clone()),
        };
        let out = serde_json::json!({
            "remember": {
                "status": status,
                "key": key,
                "reason": reason,
                "flag": USER_MEMORY_FLAG,
                "ssb_action_type": "user_decision_superseded",
            }
        });
        match serde_json::to_string_pretty(&out) {
            Ok(s) => println!("{s}"),
            Err(e) => eprintln!("remember: JSON render failed: {e}"),
        }
        return;
    }
    match outcome {
        CaptureOutcome::Disabled => {
            println!(
                "remember: {USER_MEMORY_FLAG} is not enabled (default-OFF). Set {USER_MEMORY_FLAG}=1 to capture."
            );
        }
        CaptureOutcome::Captured { key } => {
            println!("remember: captured a new preference (key={key}).");
            println!("  UserDecisionSuperseded SSB row: action_type='user_decision_superseded' kind='captured'.");
        }
        CaptureOutcome::Superseded { key } => {
            println!("remember: superseded a prior preference (key={key}); the old row was soft-closed (not deleted).");
            println!("  UserDecisionSuperseded SSB row: action_type='user_decision_superseded' kind='superseded'.");
        }
        CaptureOutcome::Noop { key, reason } => {
            println!("remember: no change (key={key}, reason={reason}).");
        }
    }
}

// ---------------------------------------------------------------------------
// SSB emission (fire-and-forget, mirrors the live tick pattern)
// ---------------------------------------------------------------------------

/// Open the SSB at `<sessions>/ssb.db` and write the payload on a current-thread runtime. Both
/// `Ssb::new` (which `tokio::spawn`s the write-loop and so REQUIRES a running reactor) and
/// `write_payload` run INSIDE `block_on` so the spawn has a reactor. Failure to emit is logged,
/// NOT fatal -- a transient SSB hiccup must not deny the user their captured preference (the
/// FactStore write already committed).
fn emit_ssb_row(clock: &Arc<dyn ClockProvider>, project_root: &PathBuf, payload: SsbPayload) {
    let sessions_dir = sessions_dir_path(project_root);
    if let Err(e) = std::fs::create_dir_all(&sessions_dir) {
        tracing::warn!(error = %e, dir = %sessions_dir.display(), "mkdir sessions_dir failed (SSB emit skipped)");
        return;
    }
    let db_path = sessions_dir.join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: CAPTURE_PROJECT_ID.to_string(),
        session_id: CAPTURE_SESSION_ID.to_string(),
    };
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            tracing::warn!(error = %e, "tokio runtime build failed (capture row not emitted)");
            return;
        }
    };
    let clock = Arc::clone(clock);
    // Ssb::new spawns the write-loop, so it MUST be constructed inside the reactor. The write is
    // awaited before the Ssb drops, so the row is durably persisted before shutdown.
    rt.block_on(async move {
        let ssb = match Ssb::new(cfg, clock) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(error = %e, "SSB init failed (capture row not emitted)");
                return;
            }
        };
        if let Err(e) = ssb
            .write_payload(CAPTURE_SSB_SOURCE, CAPTURE_SUPERSTEP, payload, Vec::new())
            .await
        {
            tracing::warn!(error = %e, "SSB write_payload failed (capture row not emitted)");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;

    use elai_core::clock::LogicalInstant;

    /// Process-wide lock serializing the CWD-mutating live-producer tests (set_current_dir is
    /// process-global, like env vars). Other in-module tests do not touch the CWD.
    static CWD_LOCK: Mutex<()> = Mutex::new(());

    /// Test guard: serialize flag-mutating tests so they do not race the process env.
    fn with_flag<T>(value: Option<&str>, body: impl FnOnce() -> T) -> T {
        let prev = std::env::var(USER_MEMORY_FLAG).ok();
        match value {
            Some(v) => std::env::set_var(USER_MEMORY_FLAG, v),
            None => std::env::remove_var(USER_MEMORY_FLAG),
        }
        let out = body();
        match prev {
            Some(p) => std::env::set_var(USER_MEMORY_FLAG, p),
            None => std::env::remove_var(USER_MEMORY_FLAG),
        }
        out
    }

    /// Deterministic stepping clock for the live-producer test: every `now_ms()` returns a fresh,
    /// strictly-increasing millisecond so two `capture_remember` calls get DISTINCT timestamps
    /// (the producer reads `now_ms()`, which does not advance `SeededClock`'s tick). No SystemTime.
    struct SteppingClock {
        ms: AtomicU64,
    }

    impl SteppingClock {
        fn new(start_ms: u64) -> Self {
            Self {
                ms: AtomicU64::new(start_ms),
            }
        }
    }

    impl ClockProvider for SteppingClock {
        fn now(&self) -> LogicalInstant {
            LogicalInstant::from_tick(self.ms.load(Ordering::Relaxed))
        }
        fn elapsed_since(&self, _mark: LogicalInstant) -> std::time::Duration {
            std::time::Duration::ZERO
        }
        fn system_now(&self) -> std::time::SystemTime {
            std::time::SystemTime::UNIX_EPOCH
                + std::time::Duration::from_millis(self.ms.load(Ordering::Relaxed))
        }
        fn instant_now(&self) -> std::time::Instant {
            std::time::Instant::now()
        }
        fn now_ms(&self) -> u128 {
            // Strictly-increasing: return the current value then advance for the next call.
            u128::from(self.ms.fetch_add(1, Ordering::Relaxed))
        }
    }

    #[test]
    fn fallback_candidate_is_deterministic_preference() {
        let a = fallback_candidate("always run the auditor before commit");
        let b = fallback_candidate("always run the auditor before commit");
        assert_eq!(a.key, b.key, "same text => same key (blake3 deterministic)");
        assert_eq!(a.fact_type, FactType::Preference);
        assert!(a.key.starts_with("remembered_"));
        assert!(!a.evidence.is_empty(), "evidence must be non-empty for dispatch_one");
    }

    #[test]
    fn build_candidate_uses_heuristic_when_it_matches() {
        let c = build_candidate("i prefer tracing over log");
        assert_eq!(c.fact_type, FactType::Preference);
        // H-31 W-1: the prefer_over shape is re-keyed onto the UNORDERED topic pair
        // (sorted: log < tracing) so a contradiction collides on the same key.
        assert_eq!(c.key, "topic_log_vs_tracing");
    }

    #[test]
    fn build_candidate_single_option_prefer_is_unchanged() {
        // The narrowest-sound rewrite touches ONLY the two-named-option shape; a single-option
        // "i prefer X" keeps its distinct stance key (no over-merge of unrelated facts).
        let c = build_candidate("i prefer rust");
        assert_eq!(c.key, "prefers_rust");
    }

    #[test]
    fn contradicting_prefer_over_collide_on_one_topic_key() {
        // The load-bearing W-1 property at the producer level: two CONTRADICTING two-option
        // statements on the SAME topic map to the SAME key but a DIFFERENT value, so dispatch_one
        // sees a same-key changed-value => UPDATE => supersede (NOT a second ADD).
        let a = build_candidate("i prefer tracing over log");
        let b = build_candidate("i prefer log over tracing");
        assert_eq!(a.key, b.key, "contradiction collides on one topic key");
        assert_eq!(a.key, "topic_log_vs_tracing");
        assert_ne!(a.value, b.value, "the value differs (which option is preferred flips)");
    }

    #[test]
    fn secret_value_is_rejected_not_stored() {
        // A force-push directive is a Block; an api key is a SkipSensitive. Both => SecretRejected.
        let block = build_candidate("always use git push --force on main");
        match classify(&block) {
            DefenseVerdict::Block { .. } => {}
            other => panic!("expected Block for force-push, got {other:?}"),
        }
        let key = build_candidate("my token is sk-ant-abcd1234efgh5678");
        match classify(&key) {
            DefenseVerdict::SkipSensitive { .. } => {}
            other => panic!("expected SkipSensitive for api key, got {other:?}"),
        }
    }

    #[test]
    fn capture_is_noop_when_flag_off() {
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        with_flag(None, || {
            let out = capture_remember_with_clock("i prefer rust over go", &clock)
                .expect("capture");
            assert_eq!(out, CaptureOutcome::Disabled);
        });
    }

    #[test]
    fn empty_text_is_error_when_flag_on() {
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        with_flag(Some("1"), || {
            let err = capture_remember_with_clock("   ", &clock).unwrap_err();
            assert!(matches!(err, CaptureError::EmptyText));
        });
    }

    #[test]
    fn add_op_maps_to_captured_kind() {
        let cand = fallback_candidate("always run the auditor before commit");
        let op = DispatchOp::Add {
            fact_id: "new-1".to_string(),
        };
        let (outcome, payload) = op_to_outcome_and_payload(&op, &cand);
        assert!(matches!(outcome, CaptureOutcome::Captured { .. }));
        match payload {
            SsbPayload::UserDecisionSuperseded(p) => {
                assert_eq!(p.kind, UserDecisionEventKind::Captured);
                assert_eq!(p.new_fact_id, "new-1");
                assert!(p.old_fact_id.is_empty(), "Captured carries no old_fact_id");
                assert_eq!(p.fact_type, "preference");
                // The payload struct has NO value field by construction (key + fact_type + ids +
                // blake3 only) -- no secret can reach the journal.
                assert!(!p.evidence_blake3.is_empty(), "carries a blake3 evidence fingerprint");
            }
            other => panic!("expected UserDecisionSuperseded, got {other:?}"),
        }
    }

    #[test]
    fn update_op_maps_to_superseded_kind_with_both_ids() {
        let cand = fallback_candidate("skip the auditor");
        let op = DispatchOp::Update {
            old_fact_id: "old-1".to_string(),
            new_fact_id: "new-2".to_string(),
        };
        let (outcome, payload) = op_to_outcome_and_payload(&op, &cand);
        assert!(matches!(outcome, CaptureOutcome::Superseded { .. }));
        match payload {
            SsbPayload::UserDecisionSuperseded(p) => {
                assert_eq!(p.kind, UserDecisionEventKind::Superseded);
                assert_eq!(p.old_fact_id, "old-1");
                assert_eq!(p.new_fact_id, "new-2");
            }
            other => panic!("expected UserDecisionSuperseded, got {other:?}"),
        }
    }

    /// H-31 W-1 LOAD-BEARING: drive the REAL `capture_remember` producer (NOT a synthetic hardcoded
    /// key) with two CONTRADICTING two-option statements on the SAME topic and assert the SECOND
    /// supersedes the first -- `CaptureOutcome::Superseded`, with the bi-temporal soft-close
    /// invariant `old.valid_to_ms == new.valid_from_ms` proven against the LIVE FactStore the
    /// producer wrote, and the old row NOT deleted. This is the live-wire proof the audit W-1
    /// flagged as missing: before the topic-rekey the live CLI could only ever ADD two rows.
    #[test]
    fn live_producer_contradiction_supersedes_with_soft_close_invariant() {
        let _cwd = CWD_LOCK.lock().expect("cwd lock");
        let tmp = tempfile::tempdir().expect("tempdir");
        let prev_cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("set cwd");

        // A stepping clock so the two captures land at DISTINCT, strictly-increasing ms.
        let clock: Arc<dyn ClockProvider> = Arc::new(SteppingClock::new(1_700_000_000_000));

        let (first_key, second_key) = with_flag(Some("1"), || {
            let first = capture_remember_with_clock("i prefer tracing over log", &clock)
                .expect("first capture");
            let second = capture_remember_with_clock("i prefer log over tracing", &clock)
                .expect("second capture");
            let CaptureOutcome::Captured { key: k1 } = first else {
                panic!("first contradicting statement must CAPTURE (ADD), got {first:?}");
            };
            let CaptureOutcome::Superseded { key: k2 } = second else {
                panic!(
                    "second contradicting statement on the SAME topic must SUPERSEDE, got {second:?}"
                );
            };
            (k1, k2)
        });

        // Both collapse onto ONE unordered topic key (the live-wire collision the rekey creates).
        assert_eq!(first_key, second_key, "both map to the same topic key");
        assert_eq!(first_key, "topic_log_vs_tracing");

        // Off-agent proof against the LIVE store the producer wrote: exactly one CURRENT row, the
        // prior is SOFT-CLOSED (not deleted), and old.valid_to_ms == new.valid_from_ms.
        let facts_path = memory_dir_path(tmp.path()).join("memory_facts.sqlite");
        let store = FactStore::open(&facts_path).expect("open live store");
        let current = store
            .list_current(DEFAULT_USER_ID, Some(Scope::Global), Some(FactType::Preference), 16)
            .expect("list_current");
        let closed = store
            .list_including_closed(DEFAULT_USER_ID, Some(FactType::Preference), 16)
            .expect("list_including_closed");

        let current_for_topic: Vec<_> = current
            .iter()
            .filter(|f| f.key == "topic_log_vs_tracing")
            .collect();
        assert_eq!(
            current_for_topic.len(),
            1,
            "exactly ONE current row for the topic after supersede (newest-wins)"
        );
        let fresh = current_for_topic[0];
        let stale = closed
            .iter()
            .find(|f| f.key == "topic_log_vs_tracing" && f.valid_to_ms.is_some())
            .expect("the prior row is soft-closed (present via include-closed, NOT deleted)");
        assert_eq!(
            stale.valid_to_ms,
            Some(fresh.valid_from_ms),
            "BI-TEMPORAL INVARIANT: old.valid_to_ms == new.valid_from_ms (soft-close, no delete)"
        );

        std::env::set_current_dir(prev_cwd).expect("restore cwd");
    }
}
