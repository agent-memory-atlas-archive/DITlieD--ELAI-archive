//! E-1 — periodic chat-reflection loop that drives the long-term
//! memory layer (sibling to [`crate::chat_reflect`] — that owns the
//! per-turn 4-stage TRIAGE/REASONER/AUDITOR/SYNTHESIZE pipeline; this
//! owns the every-K-turns memory-evolution sweep).
//!
//! Per E-1 plan Task 5+6: after every `K` user turns the chat
//! orchestrator calls [`reflect`] which (a) summarizes the recent turn
//! window, (b) extracts new [`elai_memory::MemoryFact`] candidates via
//! the existing E-5 heuristic, (c) appends a session note to the
//! [`elai_memory::NoteStore`], (d) appends a one-line summary to the
//! [`elai_memory::CoreBlock`], (e) runs [`elai_memory::decay_tick`] on
//! the facts table. The orchestrator wraps the call in an
//! `SsbActionType::ChatReflectionCompleted` row when the function
//! returns `Ok`.
//!
//! Scope is deliberately Rust-pure: no LLM dispatch lives here. The
//! summary text is computed from the conversation slice via the
//! deterministic [`elai_memory::extract_facts`] heuristic + a
//! head-of-each-turn condensation (first ~120 chars of each user turn).
//! Future work — the orchestrator can swap [`summarize_window`] for an
//! LLM-backed implementation by passing a precomputed `summary`
//! string in [`ReflectConfig::override_summary`].

use std::time::SystemTime;

use elai_memory::core_block::CoreBlockStore;
use elai_memory::notes::NoteStore;
use elai_memory::{
    decay_tick, extract_facts, Conversation, CoreBlock, DecayTickStats, ExtractedFact,
    MemoryError, MemoryNote, Turn,
};
use thiserror::Error;

/// Default reflection cadence — every 4 user turns. Matches the
/// E-1 brief's observable_effect smoke ("triggers reflection at turn
/// 4 + 8").
pub const DEFAULT_REFLECT_EVERY_N_TURNS: usize = 4;

/// One-shot reflection configuration.
#[derive(Debug, Clone)]
pub struct ReflectConfig {
    /// User id (single-user installs use `"default"`).
    pub user_id: String,
    /// CoreBlock scope tag (`"global"` or `"project:<id>"`).
    pub scope_tag: String,
    /// Cap on how many turns the summarizer condenses into the note
    /// body. Bounded to avoid runaway bodies on long sessions.
    pub max_turns_per_note: usize,
    /// When `Some`, replaces the deterministic [`summarize_window`]
    /// output (orchestrator wires the LLM digest writer here).
    pub override_summary: Option<String>,
}

impl Default for ReflectConfig {
    fn default() -> Self {
        Self {
            user_id: "default".into(),
            scope_tag: "global".into(),
            max_turns_per_note: 32,
            override_summary: None,
        }
    }
}

/// Stats returned to the caller — surfaced via the
/// `ChatReflectionCompleted` SSB payload so the dashboard can render
/// per-reflection telemetry without re-querying the stores.
#[derive(Debug, Clone, PartialEq)]
pub struct ReflectResult {
    /// Stable id of the note row written this reflection. Empty when
    /// the conversation produced no insertable content.
    pub note_id: String,
    /// First ~80 chars of the note body — included so the SSB row is
    /// human-readable in the dashboard log view.
    pub note_preview: String,
    /// Number of [`ExtractedFact`] candidates the heuristic emitted.
    pub facts_extracted: usize,
    /// Decay-tick statistics (rows examined / updated / mean trust).
    pub decay: DecayTickStats,
    /// Bytes of the CoreBlock body after the post-reflection append.
    pub core_block_bytes: usize,
}

/// Failure modes — all propagated as a single typed error.
#[derive(Debug, Error)]
pub enum ReflectError {
    /// Backing store error.
    #[error("memory store error: {0}")]
    Store(#[from] MemoryError),
    /// Caller passed an empty conversation — the reflection loop
    /// refuses to write a no-op note row.
    #[error("empty conversation")]
    EmptyConversation,
}

/// Run one reflection pass on the supplied `conv` window.
///
/// Orchestrated by `task_runner.rs::on_chat_turn_committed` (or the
/// CLI's equivalent) — the caller decides cadence via
/// [`should_reflect`].
///
/// Side effects (all confined to the two store handles passed in):
///   * one new row in `memory_notes` (skipped when summary is empty).
///   * `core_blocks` upserted with the appended summary fragment.
///   * `memory_facts` decay-tick applied with `now_ms`.
///
/// Returns a [`ReflectResult`] for the SSB beacon. The caller emits the
/// `ChatReflectionCompleted` payload — this function does NOT touch SSB
/// directly (separation: memory-layer purity, SSB-emission is owned by
/// the orchestrator boundary).
pub fn reflect(
    conv: &Conversation,
    notes_store: &NoteStore,
    core_store: &CoreBlockStore,
    facts_store: &elai_memory::FactStore,
    cfg: &ReflectConfig,
    now_ms: u128,
) -> Result<ReflectResult, ReflectError> {
    if conv.turns.is_empty() {
        return Err(ReflectError::EmptyConversation);
    }

    // 1) Build the summary body.
    let summary = match &cfg.override_summary {
        Some(s) => s.clone(),
        None => summarize_window(conv, cfg.max_turns_per_note),
    };

    // 2) Extract fact candidates from the same window — orchestrator
    // hooks the actual FactStore::insert via Mem0 dispatcher; the
    // count is what we surface in telemetry.
    let extracted: Vec<ExtractedFact> = extract_facts(conv).map_err(ReflectError::Store)?;

    // 3) Persist the note row.
    let note = build_note(&summary, &extracted, conv, now_ms);
    notes_store.insert(&note)?;

    // 4) Append to the CoreBlock — head-truncates on overflow.
    let core: CoreBlock = core_store.append(
        &cfg.user_id,
        &cfg.scope_tag,
        &note.body,
        now_ms,
    )?;

    // 5) Run the decay tick on the facts table.
    // H-33: pass empty pin set (chat_memory_reflect does not have access
    // to the SSB viewer; fail-toward-decay is the correct posture here).
    let decay = decay_tick(facts_store, &std::collections::HashSet::new(), now_ms)?;

    Ok(ReflectResult {
        note_id: note.note_id,
        note_preview: preview(&note.body, 80),
        facts_extracted: extracted.len(),
        decay,
        core_block_bytes: core.content.len(),
    })
}

/// Cadence predicate — returns `true` when `turn_index` (0-based) is a
/// reflection boundary. Used by the orchestrator's chat-turn-committed
/// hook to decide whether to call [`reflect`].
pub fn should_reflect(turn_index: usize, every_n: usize) -> bool {
    if every_n == 0 {
        return false;
    }
    turn_index > 0 && turn_index.is_multiple_of(every_n)
}

/// Deterministic head-of-each-turn condenser. Caps each turn at
/// ~120 chars and the overall window at `max_turns`.
pub fn summarize_window(conv: &Conversation, max_turns: usize) -> String {
    let take = conv.turns.len().min(max_turns.max(1));
    let mut out = String::new();
    for turn in conv.turns.iter().take(take) {
        let role = if turn.role == "user" { "U" } else { "A" };
        let head = take_head(&turn.text, 120);
        out.push_str(role);
        out.push_str(": ");
        out.push_str(head.trim());
        out.push('\n');
    }
    // Trim trailing newline so downstream consumers see clean content.
    while out.ends_with('\n') {
        out.pop();
    }
    out
}

fn take_head(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut cut = max_bytes;
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }
    &s[..cut]
}

fn preview(s: &str, max_bytes: usize) -> String {
    take_head(s, max_bytes).to_owned()
}

fn build_note(
    summary: &str,
    extracted: &[ExtractedFact],
    conv: &Conversation,
    now_ms: u128,
) -> MemoryNote {
    // Tags = the union of fact-types extracted; keywords = a small
    // bag-of-keys from the extracted candidates. Both feed the PPR
    // seed builder.
    let tags: Vec<String> = extracted
        .iter()
        .map(|e| e.fact_type.as_str().to_owned())
        .collect();
    let mut keywords: Vec<String> = extracted.iter().map(|e| e.key.clone()).collect();
    if keywords.is_empty() {
        // Fallback — pull a handful of bare-word tokens from the last
        // user turn so the note is at least retrievable by topic.
        if let Some(last_user) = conv.turns.iter().rev().find(|t| t.role == "user") {
            keywords.extend(naive_keywords(&last_user.text, 6));
        }
    }
    let links: Vec<String> = Vec::new();
    let body = if summary.is_empty() {
        format!("(session={}; no extractable content)", conv.session_id)
    } else {
        summary.to_owned()
    };
    MemoryNote {
        note_id: MemoryNote::new_id(),
        scope: "session".into(),
        tags,
        keywords,
        context: format!("session={}", conv.session_id),
        body,
        links,
        fact_type: "summary".into(),
        created_at_ms: now_ms,
        last_accessed_ms: now_ms,
        valid_to_ms: None,
        trust_score: 0.6,
    }
}

/// Lightweight keyword extractor — splits on whitespace + punctuation,
/// drops short / stopword-ish tokens, returns up to `cap` items.
fn naive_keywords(text: &str, cap: usize) -> Vec<String> {
    const STOPWORDS: &[&str] = &[
        "the", "and", "for", "with", "this", "that", "from", "are", "but", "you", "your",
        "have", "was", "were", "what", "which", "when", "how", "into", "out", "all", "any",
        "can", "will", "would", "should", "i", "we", "they", "to", "of", "on", "in", "is",
        "it", "be", "as", "at", "by", "or", "if", "so",
    ];
    let mut seen: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
    let mut out = Vec::new();
    for raw in text.split(|c: char| !c.is_ascii_alphanumeric()) {
        let tok = raw.to_ascii_lowercase();
        if tok.len() < 3 {
            continue;
        }
        if STOPWORDS.contains(&tok.as_str()) {
            continue;
        }
        if !seen.insert(tok.clone()) {
            continue;
        }
        out.push(tok);
        if out.len() >= cap {
            break;
        }
    }
    out
}

/// Convenience: produce a `Turn` carrying `now_ms`-keyed id. Used by
/// the orchestrator's per-message wrapper.
pub fn turn_with_clock(role: &str, text: &str, message_id: &str) -> Turn {
    let _ = SystemTime::now(); // entropy-allow: result discarded, appears vestigial (Turn below carries no timestamp field and this fn has no caller found in the codebase); left as-is per scope, flagged for separate dead-code review
    Turn {
        message_id: message_id.into(),
        role: role.into(),
        text: text.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_memory::FactStore;

    fn fixture_conv(session: &str, n_user_turns: usize) -> Conversation {
        let mut turns = Vec::new();
        for i in 0..n_user_turns {
            turns.push(Turn {
                message_id: format!("m{i}u"),
                role: "user".into(),
                text: format!("i prefer blake3 over sha256 for hashing in turn {i}"),
            });
            turns.push(Turn {
                message_id: format!("m{i}a"),
                role: "assistant".into(),
                text: "ack".into(),
            });
        }
        Conversation {
            session_id: session.into(),
            turns,
        }
    }

    #[test]
    fn should_reflect_cadence() {
        assert!(!should_reflect(0, 4));
        assert!(!should_reflect(1, 4));
        assert!(!should_reflect(3, 4));
        assert!(should_reflect(4, 4));
        assert!(should_reflect(8, 4));
        assert!(!should_reflect(5, 4));
        assert!(!should_reflect(4, 0));
    }

    #[test]
    fn summarize_window_caps_per_turn_bytes() {
        let long_text = "x".repeat(500);
        let conv = Conversation {
            session_id: "s".into(),
            turns: vec![Turn {
                message_id: "m".into(),
                role: "user".into(),
                text: long_text,
            }],
        };
        let sum = summarize_window(&conv, 4);
        // 120-byte head + 3-byte "U: " prefix.
        assert!(sum.len() <= 200, "got {} bytes", sum.len());
        assert!(sum.starts_with("U: "));
    }

    #[test]
    fn reflect_smoke_round_trip() {
        let notes = NoteStore::open_in_memory().expect("notes");
        let core = CoreBlockStore::open_in_memory().expect("core");
        let facts = FactStore::open_in_memory().expect("facts");
        let conv = fixture_conv("s-test", 2);
        let cfg = ReflectConfig::default();

        let r = reflect(&conv, &notes, &core, &facts, &cfg, 1_700_000_000_000)
            .expect("reflect");
        assert!(!r.note_id.is_empty());
        assert!(r.facts_extracted >= 1, "should extract blake3 preference");
        assert!(r.core_block_bytes > 0);

        // The note is queryable.
        let listed = notes.list_recent(10).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].note_id, r.note_id);

        // The CoreBlock is non-empty + decay applied trust.
        let cb = core
            .get(&cfg.user_id, &cfg.scope_tag)
            .expect("get")
            .expect("some");
        assert!(!cb.content.is_empty());
    }

    #[test]
    fn reflect_two_consecutive_turns_appends_to_core() {
        let notes = NoteStore::open_in_memory().expect("notes");
        let core = CoreBlockStore::open_in_memory().expect("core");
        let facts = FactStore::open_in_memory().expect("facts");
        let cfg = ReflectConfig::default();

        let conv1 = fixture_conv("s-1", 1);
        let _ = reflect(&conv1, &notes, &core, &facts, &cfg, 1_000).expect("r1");
        let after1 = core
            .get(&cfg.user_id, &cfg.scope_tag)
            .expect("g1")
            .expect("s1");

        let conv2 = fixture_conv("s-2", 1);
        let _ = reflect(&conv2, &notes, &core, &facts, &cfg, 2_000).expect("r2");
        let after2 = core
            .get(&cfg.user_id, &cfg.scope_tag)
            .expect("g2")
            .expect("s2");

        assert!(
            after2.content.as_bytes().len() >= after1.content.as_bytes().len(),
            "core block must accumulate (or head-truncate but not shrink below first)",
        );
        assert!(after2.edits > after1.edits);
    }

    #[test]
    fn reflect_empty_conversation_errors() {
        let notes = NoteStore::open_in_memory().expect("notes");
        let core = CoreBlockStore::open_in_memory().expect("core");
        let facts = FactStore::open_in_memory().expect("facts");
        let conv = Conversation {
            session_id: "s".into(),
            turns: vec![],
        };
        let r = reflect(&conv, &notes, &core, &facts, &ReflectConfig::default(), 1);
        assert!(matches!(r, Err(ReflectError::EmptyConversation)));
    }

    #[test]
    fn override_summary_takes_precedence() {
        let notes = NoteStore::open_in_memory().expect("notes");
        let core = CoreBlockStore::open_in_memory().expect("core");
        let facts = FactStore::open_in_memory().expect("facts");
        let conv = fixture_conv("s", 1);
        let cfg = ReflectConfig {
            override_summary: Some("LLM-supplied digest".into()),
            ..ReflectConfig::default()
        };
        let _ = reflect(&conv, &notes, &core, &facts, &cfg, 1).expect("reflect");
        let listed = notes.list_recent(1).expect("list");
        assert_eq!(listed[0].body, "LLM-supplied digest");
    }
}
