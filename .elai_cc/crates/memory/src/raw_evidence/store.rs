//! Raw-evidence record + append-only, bounded, namespace-scoped store.
//!
//! This is the immutable provenance layer beneath the extracted-fact layer.
//! See [`super`] for the architectural overview and grounding.

use crate::trust::Provenance;

/// Maximum records per namespace before FIFO compaction kicks in. Bounds the
/// append-only attack surface (plan open-question: "raw-evidence growth ...
/// needs explicit compaction/namespaces + scoped forensic query").
pub const MAX_RECORDS_PER_NAMESPACE: usize = 4096;

/// An immutable raw-evidence record — the provenance layer beneath the
/// extracted-fact layer. Append-only: once inserted, a record is never
/// mutated or individually deleted. The `record_id` is a blake3 16-hex
/// fingerprint of `(namespace, pointer, raw_content)`, computed at
/// construction time.
///
/// `provenance` carries the G-1 taint tier — the raw evidence remembers
/// WHERE it came from (file / web / session / sandbox) and at what trust
/// tier, so a poisoned extracted fact cannot launder its provenance through
/// the raw-evidence tier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEvidenceRecord {
    /// blake3 16-hex stable ID of `(namespace, pointer, raw_content)`.
    pub record_id: String,
    /// Namespace key (e.g. `repo:branch`, `project:<id>`, `session:<id>`).
    /// Scopes compaction + forensic query.
    pub namespace: String,
    /// The evidence pointer string (mirrors [`crate::fact::MemoryFact::evidence_pointers`]
    /// entries: `dkg:fn@file:line`, `sem:fn@commit:sig`, `sess-123`, etc.).
    pub pointer: String,
    /// The raw, un-extracted content (the full source text / message body /
    /// artifact snippet). This is the "one detail extraction dropped".
    pub raw_content: String,
    /// G-1 taint provenance — source + trust tier of the raw evidence.
    pub provenance: Provenance,
    /// Wall-clock ms when the record was ingested. Append-only: never changes.
    pub ingested_at_ms: u128,
}

impl RawEvidenceRecord {
    /// Construct a new [`RawEvidenceRecord`], computing the blake3
    /// `record_id` from `(namespace, pointer, raw_content)`.
    pub fn new(
        namespace: impl Into<String>,
        pointer: impl Into<String>,
        raw_content: impl Into<String>,
        provenance: Provenance,
        ingested_at_ms: u128,
    ) -> Self {
        let ns: String = namespace.into();
        let ptr: String = pointer.into();
        let raw: String = raw_content.into();
        let mut hasher = blake3::Hasher::new();
        hasher.update(ns.as_bytes());
        hasher.update(b"\x00");
        hasher.update(ptr.as_bytes());
        hasher.update(b"\x00");
        hasher.update(raw.as_bytes());
        let record_id: String = hasher
            .finalize()
            .to_hex()
            .chars()
            .take(16)
            .collect();
        Self {
            record_id,
            namespace: ns,
            pointer: ptr,
            raw_content: raw,
            provenance,
            ingested_at_ms,
        }
    }
}

/// In-memory append-only raw-evidence store. Holds [`RawEvidenceRecord`]s
/// keyed by namespace. When a namespace exceeds
/// [`MAX_RECORDS_PER_NAMESPACE`], the oldest records are evicted (FIFO
/// compaction) — the bound that addresses the plan's "raw-evidence growth"
/// open question.
///
/// The store is append-only: [`RawEvidenceStore::insert`] adds records;
/// [`RawEvidenceStore::compact_namespace`] is the only shrinking operation
/// and evicts the oldest records down to `keep` (default: half the cap).
/// Individual records are never mutated or deleted.
#[derive(Debug, Default)]
pub struct RawEvidenceStore {
    records: Vec<RawEvidenceRecord>,
}

impl RawEvidenceStore {
    /// Construct an empty raw-evidence store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a raw-evidence record. If the record's namespace is at
    /// capacity, the oldest record(s) in that namespace are evicted first
    /// (FIFO compaction).
    pub fn insert(&mut self, record: RawEvidenceRecord) {
        let ns = record.namespace.clone();
        let ns_count = self
            .records
            .iter()
            .filter(|r| r.namespace == ns)
            .count();
        if ns_count >= MAX_RECORDS_PER_NAMESPACE {
            self.compact_namespace(&ns, MAX_RECORDS_PER_NAMESPACE / 2);
        }
        self.records.push(record);
    }

    /// Evict the oldest records in `namespace` down to `keep` records.
    /// Returns the number of records evicted. This is the only shrinking
    /// operation — individual records are never deleted.
    pub fn compact_namespace(&mut self, namespace: &str, keep: usize) -> usize {
        let ns_count = self.namespace_count(namespace);
        if ns_count <= keep {
            return 0;
        }
        let ns_indices: Vec<usize> = self
            .records
            .iter()
            .enumerate()
            .filter(|(_, r)| r.namespace == namespace)
            .map(|(i, _)| i)
            .collect();
        let to_evict = ns_count - keep;
        // Evict the oldest `to_evict` records in this namespace (lowest indices).
        let evict_set: Vec<usize> = ns_indices.into_iter().take(to_evict).collect();
        // Remove from highest index to lowest to keep indices valid.
        let mut evict_sorted = evict_set.clone();
        evict_sorted.sort_unstable_by(|a, b| b.cmp(a));
        for idx in evict_sorted {
            self.records.remove(idx);
        }
        tracing::info!(
            target = "memory.raw_evidence",
            namespace = %namespace,
            evicted = to_evict,
            kept = keep,
            "J-21: raw-evidence namespace compacted (FIFO)"
        );
        to_evict
    }

    /// Query raw-evidence records by namespace whose `pointer` matches any
    /// of the given `evidence_pointers`. Returns matching records in
    /// insertion order (oldest first). This is the escalation read path —
    /// deterministic, no LLM.
    pub fn query_by_pointers(
        &self,
        namespace: &str,
        evidence_pointers: &[String],
    ) -> Vec<&RawEvidenceRecord> {
        self.records
            .iter()
            .filter(|r| r.namespace == namespace)
            .filter(|r| evidence_pointers.iter().any(|p| p == &r.pointer))
            .collect()
    }

    /// Returns the total number of records across all namespaces.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns `true` if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns the number of records in `namespace`.
    pub fn namespace_count(&self, namespace: &str) -> usize {
        self.records
            .iter()
            .filter(|r| r.namespace == namespace)
            .count()
    }

    /// Returns a slice of all records (for forensic query / debugging).
    pub fn records(&self) -> &[RawEvidenceRecord] {
        &self.records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const T: u128 = 1_700_000_000_000;
    const DKG_PTR: &str = "dkg:validate_jwt@crates/auth/src/lib.rs:42";

    #[test]
    fn raw_evidence_record_computes_stable_id() {
        let r1 = RawEvidenceRecord::new(
            "repo:main",
            DKG_PTR,
            "fn validate_jwt(token: &str) -> Result<Claims, Error>",
            Provenance::codebase_derived("crates/auth/src/lib.rs"),
            T,
        );
        let r2 = RawEvidenceRecord::new(
            "repo:main",
            DKG_PTR,
            "fn validate_jwt(token: &str) -> Result<Claims, Error>",
            Provenance::codebase_derived("crates/auth/src/lib.rs"),
            T,
        );
        assert_eq!(r1.record_id, r2.record_id);
        assert_eq!(r1.record_id.len(), 16);
    }

    #[test]
    fn raw_evidence_record_id_differs_on_content() {
        let r1 = RawEvidenceRecord::new(
            "repo:main", DKG_PTR, "content A",
            Provenance::codebase_derived("f.rs"), T,
        );
        let r2 = RawEvidenceRecord::new(
            "repo:main", DKG_PTR, "content B",
            Provenance::codebase_derived("f.rs"), T,
        );
        assert_ne!(r1.record_id, r2.record_id);
    }

    #[test]
    fn raw_evidence_record_id_differs_on_namespace() {
        let r1 = RawEvidenceRecord::new(
            "repo:main", DKG_PTR, "content",
            Provenance::codebase_derived("f.rs"), T,
        );
        let r2 = RawEvidenceRecord::new(
            "repo:feature", DKG_PTR, "content",
            Provenance::codebase_derived("f.rs"), T,
        );
        assert_ne!(r1.record_id, r2.record_id);
    }

    #[test]
    fn store_insert_and_query_by_pointers() {
        let mut store = RawEvidenceStore::new();
        let r1 = RawEvidenceRecord::new(
            "ns1", DKG_PTR, "raw content about jwt",
            Provenance::codebase_derived("f.rs"), T,
        );
        let r2 = RawEvidenceRecord::new(
            "ns1", "sess-123", "session message",
            Provenance::user_direct("cli"), T + 1000,
        );
        store.insert(r1);
        store.insert(r2);

        let ptrs = vec![DKG_PTR.to_string()];
        let matched = store.query_by_pointers("ns1", &ptrs);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].pointer, DKG_PTR);
        assert_eq!(matched[0].raw_content, "raw content about jwt");
    }

    #[test]
    fn store_query_filters_by_namespace() {
        let mut store = RawEvidenceStore::new();
        store.insert(RawEvidenceRecord::new(
            "ns1", DKG_PTR, "ns1 content",
            Provenance::codebase_derived("f.rs"), T,
        ));
        store.insert(RawEvidenceRecord::new(
            "ns2", DKG_PTR, "ns2 content",
            Provenance::codebase_derived("f.rs"), T,
        ));

        let ptrs = vec![DKG_PTR.to_string()];
        let matched = store.query_by_pointers("ns1", &ptrs);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].namespace, "ns1");
    }

    #[test]
    fn store_compact_namespace_evicts_oldest() {
        let mut store = RawEvidenceStore::new();
        for i in 0..10 {
            store.insert(RawEvidenceRecord::new(
                "ns1",
                format!("ptr-{i}"),
                format!("content-{i}"),
                Provenance::codebase_derived("f.rs"),
                T + i as u128,
            ));
        }
        assert_eq!(store.namespace_count("ns1"), 10);
        let evicted = store.compact_namespace("ns1", 3);
        assert_eq!(evicted, 7);
        assert_eq!(store.namespace_count("ns1"), 3);
        // Oldest 7 evicted; remaining are ptr-7, ptr-8, ptr-9.
        let remaining: Vec<&str> = store
            .records()
            .iter()
            .filter(|r| r.namespace == "ns1")
            .map(|r| r.pointer.as_str())
            .collect();
        assert!(remaining.contains(&"ptr-7"));
        assert!(remaining.contains(&"ptr-8"));
        assert!(remaining.contains(&"ptr-9"));
        assert!(!remaining.contains(&"ptr-0"));
    }

    #[test]
    fn store_compact_noop_when_under_keep() {
        let mut store = RawEvidenceStore::new();
        store.insert(RawEvidenceRecord::new(
            "ns1", "ptr", "content",
            Provenance::codebase_derived("f.rs"), T,
        ));
        let evicted = store.compact_namespace("ns1", 10);
        assert_eq!(evicted, 0);
        assert_eq!(store.namespace_count("ns1"), 1);
    }

    #[test]
    fn store_len_and_is_empty() {
        let mut store = RawEvidenceStore::new();
        assert!(store.is_empty());
        store.insert(RawEvidenceRecord::new(
            "ns1", "ptr", "content",
            Provenance::codebase_derived("f.rs"), T,
        ));
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());
    }
}
