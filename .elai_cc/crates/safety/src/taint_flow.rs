//! G-1: harness-wide taint propagation (data-plane defense).
//!
//! Lights up the WIRE-DARK provenance/capability-gate data-plane. Untrusted
//! content (web body, MCP tool output, external file read, sandbox stdout)
//! is registered into a process-local, content-keyed side-table at its
//! ingest site; at a destructive sink the argv is looked up against the
//! side-table and, if it CONTAINS the untrusted content VERBATIM as a
//! substring (>= `MIN_TAINT_MATCH_LEN` bytes), the SHIPPED
//! [`crate::capability_gate::allow_tool_with_origin`] gate rejects the call.
//!
//! HONEST SCOPE (GD-5): this is a verbatim-substring content match within ONE
//! process -- NOT a derives-from dataflow analysis. Transformed / summarized /
//! paraphrased untrusted content, cross-process sinks, and non-McpToolRegistry
//! MCP transports are out of scope.
//!
//! Threading model (G-1 DD1): SIDE-TABLE, not a `Provenanced<String>` type
//! threaded through the IPC dispatch boundary. The dispatch crosses a
//! process boundary that serializes bytes; a typed wrapper cannot cross
//! without an IPC-frame serde change + every dispatcher signature. Instead we
//! mirror the already-shipped F-3 `capability_flow.rs` slot pattern: an
//! `OnceLock<Arc<..>>` shared by the ingest producer and the sink enforcer,
//! keyed by the blake3 prefix of the registered content.
//!
//! Dependency-screened join (G-1 DD3 / GD-4, LOAD-BEARING): the join is
//! NOT run-global. An argv is tainted ONLY by the registered ingests whose
//! content it actually contains VERBATIM as a substring (the substring
//! screen). A clean CLI / first-party arg matches no registered untrusted
//! ingest and stays
//! Trusted. RTBAS (arXiv:2502.08966, VERIFIED) shows a run-global join
//! collapses agent utility to 22-33% on the Slack suite via over-tainting;
//! the dependency-screened join retains 90%+. The content-key side-table is
//! inherently dependency-screened: only argv carrying the untrusted bytes is
//! tainted, not every subsequent sink call.
//!
//! Default-ON (G-1 W-2): the side-table exists from first access; there is NO
//! enable flag. The only opt-out is the `ELAI_TAINT_PROPAGATE_DISABLE`
//! kill-switch read by the SINK enforcer (not this module) -- disarming
//! removes the gate, it does not flip a default-off terminal state.
//!
//! Reconciliation (G-1 DD7): `crate::provenance` (CaMeL-lite) is the canonical
//! typed label; this module composes with `capability_gate` + `prompt_armor`
//! (the `seed_from_verdict` bridge). It does NOT introduce a third taint
//! primitive -- it is the join + content-keyed registry over the existing
//! `Provenance` type.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use blake3::Hasher;

use crate::prompt_armor::ClassifierVerdict;
use crate::provenance::{Provenance, TrustLevel};

/// Number of leading blake3 bytes mixed into the side-table key. Keying on a
/// bounded prefix keeps the map key cheap while staying collision-safe via
/// blake3; the full content is stored for the substring dependency-screen.
const CONTENT_KEY_HASH_BYTES: usize = 8;

/// Maximum number of distinct untrusted ingests retained in one process's
/// side-table. Bounds memory for a long-running orchestrator; the OLDEST
/// entries are evicted first. Fail-closed posture is preserved: an
/// evicted-then-referenced ingest is simply not found, and an argv that no
/// longer matches any LIVE untrusted ingest is genuinely no longer carrying
/// that taint.
const MAX_TRACKED_INGESTS: usize = 512;

/// Minimum byte-length an untrusted ingest's content must have before its
/// verbatim-substring screen ([`TaintedContext::lookup_for_argv`]) is allowed
/// to taint a sink argv (G-1 GD-4 short-string FP fix, s-56068 B2a).
///
/// Rationale: the dependency screen is a verbatim substring match. A
/// pathologically short untrusted ingest (e.g. a 1-2 char `from_tool_output`
/// like "ok", or a lone "-m") would substring-match nearly ANY benign argv
/// (`git commit -m 'ok done'`) and over-taint a clean first-party call into a
/// false BLOCK. RTBAS (arXiv:2502.08966) shows run-global over-tainting
/// collapses agent utility to 22-33%; a too-coarse substring screen reintroduces
/// the same failure at the per-ingest level. Below this threshold the substring
/// screen is too coarse to be evidence of a real data-flow dependency, so a
/// sub-threshold ingest is NOT used to taint -- this fails OPEN only for content
/// too short to encode a destructive instruction. Any REAL injected payload
/// (a shell one-liner `rm -rf /path`, a URL, a multi-word instruction) far
/// exceeds 8 bytes, so the genuine block path is unweakened: the verbatim
/// screen still fires for content of length >= `MIN_TAINT_MATCH_LEN`.
const MIN_TAINT_MATCH_LEN: usize = 8;

/// blake3 prefix of `content` as a `u64` side-table key. Mirrors the
/// `TextSource::label_blake3_prefix` shape (blake3, little-endian first 8
/// bytes). blake3 is the ONLY hash used here (CLAUDE.md HARD RULE; no legacy
/// hash families).
fn content_key(content: &str) -> u64 {
    let mut h = Hasher::new();
    h.update(content.as_bytes());
    let out = h.finalize();
    let bytes = out.as_bytes();
    let mut acc = [0u8; CONTENT_KEY_HASH_BYTES];
    acc.copy_from_slice(&bytes[..CONTENT_KEY_HASH_BYTES]);
    u64::from_le_bytes(acc)
}

/// One registered untrusted ingest: its raw content (for the substring
/// dependency-screen at the sink) and the provenance label it carries.
#[derive(Debug, Clone)]
struct TrackedIngest {
    content: String,
    provenance: Provenance,
}

/// Mutable inner state behind the side-table mutex.
#[derive(Debug, Default)]
struct TaintState {
    map: HashMap<u64, TrackedIngest>,
    /// Insertion-ordered keys for FIFO eviction past `MAX_TRACKED_INGESTS`.
    fifo: Vec<u64>,
}

/// Process-local content-keyed taint side-table.
///
/// The ingest producer ([`TaintedContext::register`]) and the sink enforcer
/// ([`TaintedContext::lookup_for_argv`]) share ONE instance via the
/// process-global [`tainted_context`] slot, so a taint registered at a web /
/// tool-output / file / sandbox ingest is visible to the destructive-sink
/// dispatch within the same process.
#[derive(Debug, Default)]
pub struct TaintedContext {
    state: Mutex<TaintState>,
}

impl TaintedContext {
    /// Construct an empty side-table. Used by the process-global slot and by
    /// tests that want an isolated instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an untrusted (or `Recallable`) ingest into the side-table,
    /// keyed by its content.
    ///
    /// A `Trusted` provenance is intentionally NOT stored: it can never cause
    /// a gate rejection and storing it would bloat the table. Only
    /// `Recallable` / `Untrusted` ingests are tracked -- the labels the gate
    /// (Untrusted) or a higher HITL layer (Recallable) cares about.
    ///
    /// If the same content was already registered, its provenance is joined
    /// (most-restrictive wins) so re-registration never DROPS taint.
    pub fn register(&self, content: &str, provenance: Provenance) {
        if matches!(provenance.trust, TrustLevel::Trusted) {
            return;
        }
        let key = content_key(content);
        // Poisoned lock: refuse to silently lose taint, but do not panic on
        // the hot ingest path. A missing registration leaves the sink
        // conservative (only matched content taints an argv), so a no-op here
        // never fails OPEN past what an absent ingest already implies.
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        match state.map.get_mut(&key) {
            Some(existing) => {
                let joined = join_screened(&[&existing.provenance, &provenance]);
                existing.provenance = joined;
            }
            None => {
                if state.fifo.len() >= MAX_TRACKED_INGESTS {
                    let evict = state.fifo.remove(0);
                    state.map.remove(&evict);
                }
                state.fifo.push(key);
                state.map.insert(
                    key,
                    TrackedIngest {
                        content: content.to_owned(),
                        provenance,
                    },
                );
            }
        }
    }

    /// Look up the joined provenance for a destructive-sink argument.
    ///
    /// The DEPENDENCY SCREEN (GD-4): an argument is tainted ONLY by the
    /// registered ingests whose content is a substring of `argv` (i.e. the
    /// argument actually contains / derives from that untrusted ingest). The
    /// join is taken over ONLY those matched ingests -- NOT over the whole
    /// table. An argument matching no registered untrusted ingest returns
    /// `None` (it is a clean, first-party / CLI argument).
    ///
    /// Returns `Some(joined_provenance)` when at least one untrusted ingest is
    /// data-flow-relevant to this argument, else `None`.
    pub fn lookup_for_argv(&self, argv: &str) -> Option<Provenance> {
        let state = self.state.lock().ok()?;
        let relevant: Vec<&Provenance> = state
            .map
            .values()
            // GD-4 short-string FP guard (s-56068 B2a): only an ingest whose
            // content is at least `MIN_TAINT_MATCH_LEN` bytes is allowed to
            // taint via the verbatim-substring screen. A shorter ingest would
            // substring-match benign argv and over-taint; it cannot encode a
            // destructive instruction, so skipping it fails open only for noise.
            .filter(|ing| {
                ing.content.len() >= MIN_TAINT_MATCH_LEN
                    && argv.contains(ing.content.as_str())
            })
            .map(|ing| &ing.provenance)
            .collect();
        if relevant.is_empty() {
            return None;
        }
        Some(join_screened(&relevant))
    }

    /// Number of currently-tracked ingests. Test/diagnostic only.
    pub fn tracked_len(&self) -> usize {
        self.state.lock().map(|s| s.map.len()).unwrap_or(0)
    }

    /// Clear the table. Used by tests to isolate a fresh table; production
    /// never clears (FIFO eviction bounds it).
    pub fn clear(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.map.clear();
            state.fifo.clear();
        }
    }
}

/// Process-global taint side-table slot.
///
/// Mirrors the F-3 `capability_flow::committed_slot` shape: the ingest sites
/// and the sink enforcer all obtain the SAME `Arc<TaintedContext>` so a taint
/// registered at ingest is visible at the sink within the same process.
///
/// Default-ON: the slot is created empty on first access; there is NO env-gate
/// guarding its creation (W-2). The kill-switch lives at the SINK, not here.
static TAINTED_CONTEXT: OnceLock<Arc<TaintedContext>> = OnceLock::new();

/// Return the process-global taint side-table, creating it on first call.
pub fn tainted_context() -> Arc<TaintedContext> {
    TAINTED_CONTEXT
        .get_or_init(|| Arc::new(TaintedContext::new()))
        .clone()
}

/// Dependency-screened join: the element-wise MOST-RESTRICTIVE join over the
/// supplied `relevant` provenances ONLY (GD-4).
///
/// `relevant` MUST already be the dependency-screened subset (the ingests the
/// in-flight value actually derives from) -- this function does NOT itself
/// screen; it joins what it is given. Trust dominates on the lattice
/// `Trusted < Recallable < Untrusted` ([`TrustLevel`] `Ord`, proven by
/// `provenance::tests::trust_level_ordering_stable`); the returned provenance
/// carries the `source` of the element holding the maximum trust level (the
/// dominating, most-restrictive ingest), so the blocked-sink SSB row can name
/// the offending origin.
///
/// An empty `relevant` slice yields a `Trusted` user-prompt provenance (a
/// value derived from nothing untrusted is trusted). Callers that need a
/// fail-closed default on UNKNOWN provenance handle the `None` case at the
/// lookup layer, not here: an empty relevant set means "provably no untrusted
/// dependency", which is genuinely Trusted, not unknown.
pub fn join_screened(relevant: &[&Provenance]) -> Provenance {
    let mut dominant: Option<&Provenance> = None;
    for p in relevant {
        match dominant {
            None => dominant = Some(p),
            Some(cur) if p.trust > cur.trust => dominant = Some(p),
            Some(_) => {}
        }
    }
    match dominant {
        Some(p) => (*p).clone(),
        None => Provenance::user_prompt(),
    }
}

/// Seed / escalate a provenance from an E-37 PromptArmor classifier verdict
/// (G-1 DD7 composition with E-37).
///
/// A non-`Clean` verdict (`Suspect` / `Malicious`) forces the label to
/// `Untrusted`, preserving the ingest's `source` so the origin is still named.
/// A `Clean` verdict returns `base` unchanged. This lets an ingest site that
/// already classified its content escalate a label the path-screen marked
/// `Trusted` (e.g. a tracked source file whose CONTENT nonetheless trips the
/// injection classifier) to `Untrusted` -- fail-closed on the classifier
/// signal, never the reverse.
pub fn seed_from_verdict(verdict: &ClassifierVerdict, base: Provenance) -> Provenance {
    if verdict.is_clean() {
        base
    } else {
        Provenance {
            source: base.source,
            trust: TrustLevel::Untrusted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::TextSource;
    use std::path::PathBuf;

    fn web(url: &str) -> Provenance {
        Provenance::from_web(url)
    }

    #[test]
    fn join_empty_is_trusted() {
        let j = join_screened(&[]);
        assert!(j.is_trusted());
    }

    #[test]
    fn join_untrusted_dominates_trusted() {
        let t = Provenance::user_prompt();
        let u = web("https://attacker.example");
        let j = join_screened(&[&t, &u]);
        assert!(j.is_untrusted());
        assert_eq!(j.source.label(), "web");
    }

    #[test]
    fn join_recallable_below_untrusted() {
        let r = Provenance::from_memory("memory-bank/x.md");
        let u = web("https://e.com");
        let j = join_screened(&[&r, &u]);
        assert!(j.is_untrusted());
    }

    #[test]
    fn join_recallable_only_is_recallable_not_untrusted() {
        let r = Provenance::from_memory("memory-bank/x.md");
        let j = join_screened(&[&r]);
        assert!(!j.is_untrusted());
        assert!(!j.is_trusted());
        assert_eq!(j.trust, TrustLevel::Recallable);
    }

    #[test]
    fn register_then_lookup_matches_substring() {
        let ctx = TaintedContext::new();
        ctx.register("rm -rf /important", web("https://attacker.example/payload"));
        let p = ctx
            .lookup_for_argv("git commit -m 'apply: rm -rf /important now'")
            .expect("argv deriving from untrusted ingest must be tainted");
        assert!(p.is_untrusted());
        assert_eq!(p.source.label(), "web");
    }

    #[test]
    fn lookup_unrelated_argv_is_none_dependency_screen() {
        // GD-4: an untrusted ingest does NOT taint an unrelated, non-data-
        // dependent argv. This proves the join is NOT run-global.
        let ctx = TaintedContext::new();
        ctx.register("rm -rf /important", web("https://attacker.example/payload"));
        let unrelated = ctx.lookup_for_argv("git commit -m 'feature: add login button'");
        assert!(
            unrelated.is_none(),
            "an unrelated clean argv must NOT inherit an untrusted ingest's taint"
        );
    }

    #[test]
    fn short_ingest_does_not_overtaint_benign_argv() {
        // GD-4 short-string FP guard (s-56068 B2a): a pathologically short
        // untrusted ingest (below MIN_TAINT_MATCH_LEN) must NOT taint a benign
        // argv that happens to contain it as a substring. Without the guard,
        // "ok" would substring-match the commit message and over-taint.
        let ctx = TaintedContext::new();
        ctx.register("ok", web("https://attacker.example/short"));
        let benign = ctx.lookup_for_argv("git commit -m 'looks ok to me'");
        assert!(
            benign.is_none(),
            "a sub-threshold untrusted ingest must not over-taint a benign argv"
        );
        // A real injected payload (>= MIN_TAINT_MATCH_LEN bytes) still taints.
        ctx.register("rm -rf /workspace", web("https://attacker.example/long"));
        let blocked = ctx
            .lookup_for_argv("git commit -m 'apply: rm -rf /workspace now'")
            .expect("a sufficiently long untrusted ingest must still taint");
        assert!(blocked.is_untrusted());
    }

    #[test]
    fn min_taint_match_len_boundary_is_eight() {
        // Pin the threshold: exactly MIN_TAINT_MATCH_LEN bytes taints; one
        // byte shorter does not. Locks the named constant so a future tweak is
        // a deliberate, test-visible change.
        let ctx = TaintedContext::new();
        let at_threshold = "a".repeat(MIN_TAINT_MATCH_LEN); // 8 bytes
        let below = "a".repeat(MIN_TAINT_MATCH_LEN - 1); // 7 bytes
        ctx.register(&below, web("https://e.com/below"));
        assert!(
            ctx.lookup_for_argv(&format!("x {below} y")).is_none(),
            "{} bytes is below threshold and must not taint",
            MIN_TAINT_MATCH_LEN - 1
        );
        ctx.register(&at_threshold, web("https://e.com/at"));
        assert!(
            ctx.lookup_for_argv(&format!("x {at_threshold} y")).is_some(),
            "{MIN_TAINT_MATCH_LEN} bytes is at threshold and must taint"
        );
    }

    #[test]
    fn trusted_ingest_is_not_tracked() {
        let ctx = TaintedContext::new();
        ctx.register("first-party source line", Provenance::user_prompt());
        assert_eq!(ctx.tracked_len(), 0);
        assert!(ctx.lookup_for_argv("first-party source line").is_none());
    }

    #[test]
    fn re_register_joins_not_drops() {
        let ctx = TaintedContext::new();
        ctx.register("payload-x", Provenance::from_memory("k"));
        ctx.register("payload-x", web("https://e.com"));
        let p = ctx.lookup_for_argv("do payload-x").expect("tracked");
        assert!(p.is_untrusted(), "re-register must join to most-restrictive");
    }

    #[test]
    fn seed_from_verdict_clean_is_passthrough() {
        let base = Provenance::from_file_screened(
            PathBuf::from(".elai_cc/crates/safety/src/lib.rs"),
            std::path::Path::new(".elai_cc"),
        );
        assert!(base.is_trusted());
        let out = seed_from_verdict(&ClassifierVerdict::Clean, base.clone());
        assert!(out.is_trusted());
        assert_eq!(out, base);
    }

    #[test]
    fn seed_from_verdict_malicious_forces_untrusted() {
        let base = Provenance::user_prompt();
        let v = ClassifierVerdict::Malicious {
            reason: "injection".into(),
            patterns_matched: vec!["ignore all previous".into()],
        };
        let out = seed_from_verdict(&v, base);
        assert!(out.is_untrusted());
    }

    #[test]
    fn seed_from_verdict_suspect_forces_untrusted_preserves_source() {
        let base = Provenance::from_tool_output("http://localhost:8200", "search");
        let v = ClassifierVerdict::Suspect {
            reason: "suspicious".into(),
            patterns_matched: vec![],
        };
        let out = seed_from_verdict(&v, base);
        assert!(out.is_untrusted());
        assert_eq!(out.source.label(), "tool_output");
    }

    #[test]
    fn fifo_eviction_bounds_table() {
        let ctx = TaintedContext::new();
        for i in 0..(MAX_TRACKED_INGESTS + 10) {
            ctx.register(&format!("ingest-{i}"), web(&format!("https://e.com/{i}")));
        }
        assert!(ctx.tracked_len() <= MAX_TRACKED_INGESTS);
        assert!(ctx.lookup_for_argv("use ingest-0 here").is_none());
        let recent = MAX_TRACKED_INGESTS + 5;
        assert!(ctx
            .lookup_for_argv(&format!("use ingest-{recent} here"))
            .is_some());
    }

    #[test]
    fn process_global_slot_is_shared() {
        let a = tainted_context();
        let b = tainted_context();
        a.clear();
        a.register("global-shared-payload", web("https://e.com"));
        assert!(b
            .lookup_for_argv("contains global-shared-payload inside")
            .is_some());
        a.clear();
    }

    #[test]
    fn content_key_is_blake3_stable() {
        let k1 = content_key("hello world");
        let k2 = content_key("hello world");
        let k3 = content_key("hello worlD");
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }

    #[test]
    fn tracked_source_label_is_preserved_through_join() {
        let f = Provenance::from_file(PathBuf::from("/etc/passwd")); // untrusted
        let s = Provenance::from_sandbox("box-9");
        let j = join_screened(&[&f, &s]);
        assert!(j.is_untrusted());
        assert!(matches!(
            j.source,
            TextSource::File { .. } | TextSource::SandboxOutput { .. }
        ));
    }
}
