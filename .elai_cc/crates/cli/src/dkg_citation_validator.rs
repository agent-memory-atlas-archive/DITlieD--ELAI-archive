//! H-32 — concrete [`elai_memory::CitationValidator`] backed by the on-disk
//! `SymbolGraphCache` (crates/knowledge). Lives in crates/cli to avoid a
//! memory → knowledge crate edge.
//!
//! The validator resolves citations via two complementary legs:
//!   1. **Name-index existence** (`ensure_name_index`): checks that the symbol
//!      name still appears in the graph at all (primary, fast).
//!   2. **Line-range overlap** (`get_or_load_payload` + `find_symbol_at`): for
//!      each `SymbolRef` whose file matches the citation's file, loads the
//!      per-file node payload and checks whether any `SymbolNode` spans the
//!      cited line.
//!
//! `SymbolRef` carries only (`file`, `id`) — no line number. The line-range
//! check is therefore done via the payload + `find_symbol_at` (R5 note: the
//! name-index leg is the primary existence evidence; the line check is
//! secondary).
//!
//! Fail-closed throughout: any cache open/load error → `Validity::Unknown`.
//! Symbol not found in name index → `Validity::Stale` (was resolvable at
//! write time; no longer is). Index load error → `Validity::Unknown`.

use elai_core::paths::dkg_db_path;
use elai_knowledge::symbol_graph::{
    cache::SymbolGraphCache,
    entry_points::find_symbol_at,
    sharding::SYMBOL_FILE_LRU_SIZE,
};
use elai_memory::{CitationPointer, CitationValidator, MemoryFact, Validity};

/// DKG-backed citation validator. Constructed once per
/// `assemble_user_memory_section` call (name-index load is ~1-5 ms on SSD
/// per the cache docs; the cost is bounded to the small top-K slice).
///
/// The `CitationValidator` trait takes `&self`; we work around the mutable
/// cache by opening a fresh `SymbolGraphCache` per `validate` call. This is
/// safe because `SymbolGraphCache::new` opens the DB read-only and the
/// shard/payload loads are idempotent.
pub struct DkgCitationValidator {
    /// Absolute path to the on-disk symbol graph DB.
    db_path: std::path::PathBuf,
    /// Project identifier (derives from project root path).
    project_id: String,
}

impl DkgCitationValidator {
    /// Open the on-disk symbol graph at the DKG db path derived from
    /// `project_root`. Returns `None` when the DB does not exist or fails to
    /// open — the caller should treat this as no validator (validity gate
    /// bypassed, Tier-A facts excluded via `Validity::Unknown` in
    /// `revalidate_citation`).
    pub fn open(project_root: &std::path::Path) -> Option<Self> {
        let db_path = dkg_db_path(project_root);
        if !db_path.exists() {
            return None;
        }
        // Probe: verify the cache opens before claiming the validator exists.
        let project_id = elai_core::project_id_from_path(project_root);
        match SymbolGraphCache::new(&db_path, &project_id, SYMBOL_FILE_LRU_SIZE) {
            Ok(_) => Some(Self { db_path, project_id }),
            Err(e) => {
                tracing::debug!(
                    error = %e,
                    db_path = %db_path.display(),
                    "H-32: DkgCitationValidator probe open failed (validity gate will be bypassed)"
                );
                None
            }
        }
    }
}

impl CitationValidator for DkgCitationValidator {
    fn validate(&self, fact: &MemoryFact) -> Validity {
        // Find the first parseable dkg: pointer. If none, Unknown (fail-closed).
        let citation = match fact
            .evidence_pointers
            .iter()
            .find_map(|p| CitationPointer::parse(p))
        {
            Some(c) => c,
            None => return Validity::Unknown,
        };

        // Open a fresh cache per call (read-only; idempotent shard loads).
        let mut cache =
            match SymbolGraphCache::new(&self.db_path, &self.project_id, SYMBOL_FILE_LRU_SIZE) {
                Ok(c) => c,
                Err(e) => {
                    tracing::debug!(error = %e, "H-32: cache open failed → Unknown");
                    return Validity::Unknown;
                }
            };

        // Leg 1: name-index existence.
        let name_index = match cache.ensure_name_index() {
            Ok(idx) => idx,
            Err(e) => {
                tracing::debug!(error = %e, "H-32: name-index load failed → Unknown");
                return Validity::Unknown;
            }
        };

        let refs = match name_index.get(&citation.symbol_name) {
            Some(r) if !r.is_empty() => r.clone(),
            _ => {
                tracing::debug!(
                    symbol = %citation.symbol_name,
                    "H-32: symbol absent from name_index → Stale"
                );
                return Validity::Stale;
            }
        };

        // Leg 2: check that at least one ref maps to the cited file.
        let file_match: Vec<_> = refs
            .iter()
            .filter(|r| r.file == citation.file)
            .collect();

        if file_match.is_empty() {
            tracing::debug!(
                symbol = %citation.symbol_name,
                cited_file = %citation.file,
                "H-32: symbol exists but not in cited file → Stale"
            );
            return Validity::Stale;
        }

        // Leg 3: load the file's SymbolNodes and verify the cited line is still
        // spanned by a symbol. If the payload is absent (file removed), Stale.
        let nodes = match cache.get_or_load_payload(&citation.file) {
            Ok(Some(payload)) => payload.symbols.clone(),
            Ok(None) => {
                tracing::debug!(
                    file = %citation.file,
                    "H-32: file payload absent from index → Stale"
                );
                return Validity::Stale;
            }
            Err(e) => {
                tracing::debug!(
                    error = %e,
                    file = %citation.file,
                    "H-32: file payload load error → Unknown"
                );
                return Validity::Unknown;
            }
        };

        if find_symbol_at(&nodes, citation.line).is_some() {
            Validity::Valid
        } else {
            tracing::debug!(
                symbol = %citation.symbol_name,
                file = %citation.file,
                line = citation.line,
                "H-32: symbol in file but line no longer covered → Stale"
            );
            Validity::Stale
        }
    }
}
