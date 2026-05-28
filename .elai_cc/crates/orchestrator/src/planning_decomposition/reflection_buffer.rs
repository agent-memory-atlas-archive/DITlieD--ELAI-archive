//! Reflexion-style per-node reflection buffer (arXiv:2303.11366).
//!
//! Plan task 3 of E-46: each tree node (Concept / Phase / Step /
//! BabyStep) carries a `Vec<Reflection>` of textual reflections buffered
//! separately from the authoritative audit log. The TreeNavigator
//! prepends recent reflections to the agent prompt when re-dispatching
//! the same level (executor, auditor, decomposer).
//!
//! Storage shape: in-memory `FxHashMap<NodeKey, ReflectionBuffer>` keyed
//! by a free-form node identifier string supplied by the caller. The
//! buffer is sync + Send + Sync — no async needed at the buffer layer.
//! Producers serialize the buffer state through the
//! [`crate::planning_decomposition::producers::emit_reflection_appended`]
//! SSB row.
//!
//! Cap + eviction: when [`ReflectionBuffer::push`] is called and the
//! buffer is at [`MAX_REFLECTIONS_PER_NODE`] entries, the oldest entry
//! (front of the deque) is dropped first (LRU on creation order). Eviction
//! is silent — no SSB row — because the audit log retains the canonical
//! record.
//!
//! Trust tier: every reflection is born at `TrustTier::T1` per plan task
//! 3 docstring (codebase-derived, ELAI-internal distillation, not user
//! direct (T0) or third-party (T2/T3)). C-48 SafeHarbor primitives mark
//! the SSB row with the same tier.
//!
//! Anti-stale-training: blake3 ONLY for any content hashing (no
//! blake2/sha2); the `source_event_id` field is a `blake3_hex` slice of
//! the originating event payload so the buffer can de-dup repeated
//! cascade events for the same root cause without pulling in `uuid`.
//!
//! No `.unwrap()` outside `#[cfg(test)]`. No `unimplemented!()`. No
//! silent fallback on hash failure — `push` validates inputs up-front
//! and returns `Err(ReflectionError)`.

use std::collections::VecDeque;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

/// Default cap on reflections retained per node (per plan task 3 +
/// open-question line 215 default). Surface in
/// `elai_cc.toml` `[planning.reflection_buffer] cap_per_node` to override.
pub const MAX_REFLECTIONS_PER_NODE: usize = 8;

/// Minimum reflection text length. Empty or whitespace-only reflections
/// are rejected — the buffer is meant to carry distilled prose, not noise.
pub const MIN_REFLECTION_TEXT_LEN: usize = 4;

/// Trust tier marker on a single reflection. Mirrors C-48 SafeHarbor
/// `TrustTier` shape but kept local so this module does not depend on the
/// C-48 crate at compile time (the dependency goes the other direction:
/// C-48 row writers re-import this enum or map between identical labels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReflectionTrustTier {
    /// User-direct content (verbatim user input).
    T0,
    /// Codebase-derived distillation (default for ELAI-internal reflections).
    T1,
    /// Third-party retrieved content (web, paper-ingestor).
    T2,
    /// Untrusted / unverified.
    T3,
}

impl ReflectionTrustTier {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::T0 => "t0",
            Self::T1 => "t1",
            Self::T2 => "t2",
            Self::T3 => "t3",
        }
    }
}

/// One reflection entry. Carries the prose distillation + a content-derived
/// `source_event_id` (blake3-hex of the originating event payload) + a
/// creation timestamp + the trust tier.
///
/// Equality: two reflections with the same `source_event_id` are considered
/// duplicates by [`ReflectionBuffer::push`] (de-dup at push time).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reflection {
    /// The reflection prose (max ~2 KiB for SSB payload sanity).
    pub text: String,
    /// Blake3-hex of the originating event payload. Used for de-dup so a
    /// retry that hits the same cascade event does not double-buffer the
    /// same reflection.
    pub source_event_id: String,
    /// Wall-clock UNIX seconds when this reflection was created.
    pub created_at_unix: u64,
    /// Trust tier (default T1 for ELAI-internal reflections).
    pub trust_tier: ReflectionTrustTier,
}

impl Reflection {
    /// Construct a fresh `T1` reflection. The `source_event_payload` is
    /// hashed with blake3 to derive `source_event_id` so that the same
    /// event payload always yields the same id without taking on a `uuid`
    /// crate dep.
    ///
    /// Returns `Err(ReflectionError::TextTooShort)` if `text` (after
    /// trim) is shorter than [`MIN_REFLECTION_TEXT_LEN`].
    pub fn new_t1(
        text: String,
        source_event_payload: &[u8],
        created_at_unix: u64,
    ) -> Result<Self, ReflectionError> {
        if text.trim().len() < MIN_REFLECTION_TEXT_LEN {
            return Err(ReflectionError::TextTooShort {
                got: text.trim().len(),
                min: MIN_REFLECTION_TEXT_LEN,
            });
        }
        let source_event_id = blake3::hash(source_event_payload).to_hex().to_string();
        Ok(Self {
            text,
            source_event_id,
            created_at_unix,
            trust_tier: ReflectionTrustTier::T1,
        })
    }
}

/// Per-node reflection buffer. Bounded `VecDeque` keeping the cap-N most
/// recent reflections in creation order (oldest at the front).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ReflectionBuffer {
    /// In-order reflections; front = oldest.
    pub entries: VecDeque<Reflection>,
    /// Per-buffer cap (default [`MAX_REFLECTIONS_PER_NODE`]).
    pub cap: usize,
}

impl ReflectionBuffer {
    /// New empty buffer with default cap.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            cap: MAX_REFLECTIONS_PER_NODE,
        }
    }

    /// New empty buffer with a custom cap. `cap == 0` is rejected (would
    /// reject every push) and degrades to default.
    #[must_use]
    pub fn with_cap(cap: usize) -> Self {
        let cap = if cap == 0 {
            warn!(target: "planning.reflection", "cap=0 invalid; using default");
            MAX_REFLECTIONS_PER_NODE
        } else {
            cap
        };
        Self {
            entries: VecDeque::new(),
            cap,
        }
    }

    /// Append a reflection. If the buffer is at capacity, the oldest entry
    /// is evicted first (silent). Duplicates by `source_event_id` are NOT
    /// pushed; the existing entry remains in place. Returns `true` if the
    /// reflection was appended, `false` if it was a duplicate.
    pub fn push(&mut self, reflection: Reflection) -> bool {
        if self.entries.iter().any(|r| r.source_event_id == reflection.source_event_id) {
            debug!(
                target: "planning.reflection",
                source_event_id = %reflection.source_event_id,
                "dedup skip (existing entry retained)"
            );
            return false;
        }
        if self.entries.len() >= self.cap {
            let evicted = self.entries.pop_front();
            if let Some(ev) = &evicted {
                debug!(
                    target: "planning.reflection",
                    evicted_source_event_id = %ev.source_event_id,
                    "evicted oldest at cap"
                );
            }
        }
        self.entries.push_back(reflection);
        true
    }

    /// Number of entries currently buffered.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True iff the buffer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Borrowed slice over the entries in oldest-to-newest order.
    pub fn iter(&self) -> impl Iterator<Item = &Reflection> {
        self.entries.iter()
    }

    /// Render the entries as a multi-line prefix block suitable for
    /// prepending to an agent prompt (per plan task 3 §41 "prepend a
    /// 'Recent reflections on this node:' section to the agent prompt
    /// before re-dispatching the same level").
    ///
    /// Returns the empty string if the buffer is empty so callers can
    /// safely concatenate unconditionally.
    #[must_use]
    pub fn render_prompt_prefix(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        let mut out = String::from("Recent reflections on this node:\n");
        for (i, r) in self.entries.iter().enumerate() {
            out.push_str(&format!("  [{}] ({}) {}\n", i, r.trust_tier.as_str(), r.text.trim()));
        }
        out
    }
}

/// Multi-node buffer registry — maps free-form node-key strings to per-node
/// [`ReflectionBuffer`]. The registry itself is sync `&mut self` mutation;
/// concurrent callers wrap it in a `tokio::sync::Mutex` or
/// `parking_lot::Mutex` (caller choice).
#[derive(Debug, Clone, Default)]
pub struct ReflectionRegistry {
    nodes: FxHashMap<String, ReflectionBuffer>,
    /// Per-buffer cap to apply when a buffer is created for a node not yet
    /// present in the registry.
    pub default_cap: usize,
}

impl ReflectionRegistry {
    /// Fresh empty registry with default per-node cap.
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: FxHashMap::default(),
            default_cap: MAX_REFLECTIONS_PER_NODE,
        }
    }

    /// Registry with a custom per-node cap.
    #[must_use]
    pub fn with_default_cap(cap: usize) -> Self {
        let cap = if cap == 0 { MAX_REFLECTIONS_PER_NODE } else { cap };
        Self {
            nodes: FxHashMap::default(),
            default_cap: cap,
        }
    }

    /// Push a reflection into the buffer for `node_key`. Creates the buffer
    /// on first touch. Returns the bool from [`ReflectionBuffer::push`].
    pub fn push(&mut self, node_key: &str, reflection: Reflection) -> bool {
        let cap = self.default_cap;
        let buf = self
            .nodes
            .entry(node_key.to_owned())
            .or_insert_with(|| ReflectionBuffer::with_cap(cap));
        buf.push(reflection)
    }

    /// Borrow the buffer for `node_key`, returning the empty default if
    /// not present (does NOT insert).
    pub fn get(&self, node_key: &str) -> Option<&ReflectionBuffer> {
        self.nodes.get(node_key)
    }

    /// Number of distinct nodes with at least one buffered reflection.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ReflectionError {
    #[error("reflection text too short ({got} < {min} chars after trim)")]
    TextTooShort { got: usize, min: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(text: &str, seed: &[u8]) -> Reflection {
        Reflection::new_t1(text.to_string(), seed, 1_700_000_000).unwrap()
    }

    #[test]
    fn reflection_new_t1_short_text_rejected() {
        let res = Reflection::new_t1("hi".into(), b"x", 1);
        assert!(res.is_err());
        match res.unwrap_err() {
            ReflectionError::TextTooShort { got, min } => {
                assert_eq!(got, 2);
                assert_eq!(min, MIN_REFLECTION_TEXT_LEN);
            }
        }
    }

    #[test]
    fn reflection_new_t1_long_text_ok() {
        let r = mk("a reflection on the prior cascade event", b"evt-1");
        assert_eq!(r.trust_tier, ReflectionTrustTier::T1);
        // blake3 of "evt-1" is deterministic; just check it's 64 hex chars.
        assert_eq!(r.source_event_id.len(), 64);
    }

    #[test]
    fn buffer_push_appends_in_order() {
        let mut buf = ReflectionBuffer::new();
        buf.push(mk("first reflection here", b"a"));
        buf.push(mk("second reflection here", b"b"));
        let texts: Vec<&str> = buf.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(texts, vec!["first reflection here", "second reflection here"]);
    }

    #[test]
    fn buffer_dedup_by_source_event_id() {
        let mut buf = ReflectionBuffer::new();
        assert!(buf.push(mk("first reflection", b"evt-X")));
        assert!(!buf.push(mk("DUPLICATE attempt", b"evt-X")));
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.iter().next().unwrap().text, "first reflection");
    }

    #[test]
    fn buffer_evicts_oldest_at_cap() {
        let mut buf = ReflectionBuffer::with_cap(3);
        buf.push(mk("first reflection text", b"a"));
        buf.push(mk("second reflection text", b"b"));
        buf.push(mk("third reflection text", b"c"));
        buf.push(mk("fourth reflection text", b"d"));
        assert_eq!(buf.len(), 3);
        let texts: Vec<&str> = buf.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(texts, vec![
            "second reflection text",
            "third reflection text",
            "fourth reflection text",
        ]);
    }

    #[test]
    fn buffer_with_cap_zero_falls_back_to_default() {
        let buf = ReflectionBuffer::with_cap(0);
        assert_eq!(buf.cap, MAX_REFLECTIONS_PER_NODE);
    }

    #[test]
    fn render_prompt_prefix_empty_is_blank() {
        let buf = ReflectionBuffer::new();
        assert_eq!(buf.render_prompt_prefix(), "");
    }

    #[test]
    fn render_prompt_prefix_non_empty_has_header() {
        let mut buf = ReflectionBuffer::new();
        buf.push(mk("a useful reflection", b"a"));
        let pfx = buf.render_prompt_prefix();
        assert!(pfx.starts_with("Recent reflections on this node:"));
        assert!(pfx.contains("a useful reflection"));
        assert!(pfx.contains("(t1)"));
    }

    #[test]
    fn registry_push_creates_buffer_on_first_touch() {
        let mut reg = ReflectionRegistry::new();
        assert!(reg.push("node-a", mk("first push reflection", b"a")));
        assert_eq!(reg.node_count(), 1);
        assert_eq!(reg.get("node-a").unwrap().len(), 1);
    }

    #[test]
    fn registry_independent_buffers_per_node() {
        let mut reg = ReflectionRegistry::with_default_cap(2);
        reg.push("node-a", mk("a reflection 1", b"a1"));
        reg.push("node-a", mk("a reflection 2", b"a2"));
        reg.push("node-a", mk("a reflection 3", b"a3"));
        reg.push("node-b", mk("b reflection 1", b"b1"));
        assert_eq!(reg.get("node-a").unwrap().len(), 2);
        assert_eq!(reg.get("node-b").unwrap().len(), 1);
    }
}
