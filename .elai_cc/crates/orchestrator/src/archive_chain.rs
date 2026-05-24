//! D-1: append-only ArchiveChain backing reasoning-template lifecycle.
//!
//! Storage format: JSON Lines (one [`ArchiveEntry`] per line) so the file is
//! line-diffable in git review (DGM §5 archive practice). The chain itself
//! has NO delete method — append-only by construction.
//!
//! Each entry carries its own `self_hash` + an optional `parent_hash`; the
//! [`ArchiveChain::verify_chain`] walk recomputes both at load time. A
//! tampered file (body mutated by hand, or a broken parent link) surfaces as
//! [`TemplateError::HashMismatch`] / [`TemplateError::ParentNotFound`] and the
//! orchestrator REFUSES to use the chain until the operator resolves it.

use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;

use crate::reasoning_template::{
    compute_content_hash, Blake3Hash, Domain, ReasoningTemplate, RetirementReason, TemplateError,
    TemplateId, TemplateRegistry, ArchiveStatus,
};
use crate::ssb::{ArchiveEntryAddedPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// D-8 — ArchiveVerifyError (typed verify failure modes)
// ---------------------------------------------------------------------------

/// D-8 (SCHEMA_VERSION 115) — per-entry verify failure mode surfaced by the
/// `elai archive verify` CLI driver + the `ArchiveVerifyResult` SSB row.
///
/// Each variant carries the entry index (0-based, line-aligned with the
/// underlying JSONL file) so dashboards / CI logs can highlight the broken
/// row directly. Hash strings are 64-char lowercase hex (Blake3); the
/// `MalformedJson` variant also carries the raw line text so operators can
/// see exactly what failed to parse without re-reading the file.
///
/// Lives in `orchestrator` (not `cli`) because the SSB payload type
/// references it — keeping it here lets the SSB schema cascade reference
/// the same canonical type the CLI walks.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArchiveVerifyError {
    /// Recomputed `self_hash` (from `(template_body, parent_hash)`) did not
    /// match the stored `self_hash` field. Lobotomy-attack signal — the
    /// template body was rewritten without recomputing the chain.
    #[error(
        "hash_mismatch at entry {entry_index} (template_id={template_id}): expected={expected} computed={computed}"
    )]
    HashMismatch {
        entry_index: u32,
        expected: String,
        computed: String,
        template_id: String,
    },
    /// `parent_hash` references a hash not present earlier in the chain.
    /// Either a forged parent reference or a chain-truncation attack.
    #[error(
        "broken_parent at entry {entry_index} (template_id={template_id}): parent_hash={parent_hash} not seen earlier"
    )]
    BrokenParent {
        entry_index: u32,
        parent_hash: String,
        template_id: String,
    },
    /// Two entries in the chain carry the same `self_hash`. Either an
    /// accidental double-append (chain integrity bug) or a deliberate
    /// hash-collision attempt (cryptographically improbable for blake3 but
    /// surfaced explicitly so the dashboard does not silently dedupe).
    #[error(
        "duplicate_hash at entry {second_at_index} (also seen at entry {first_at_index}): hash={hash}"
    )]
    DuplicateHash {
        hash: String,
        first_at_index: u32,
        second_at_index: u32,
    },
    /// JSONL deserialise failure on a single line. `entry_index` is the
    /// 0-based line index (counted across blank/whitespace skips so it
    /// corresponds to the run of *attempted* entry parses, not the raw
    /// file line number); `raw` is the full line text (truncated to
    /// `MALFORMED_JSON_RAW_MAX` bytes if longer); `parse_error` is the
    /// `serde_json` error message.
    #[error(
        "malformed_json at entry {entry_index}: {parse_error} (raw: {raw})"
    )]
    MalformedJson {
        entry_index: u32,
        raw: String,
        parse_error: String,
    },
    /// I/O failure reading the JSONL file (open / read line). Missing file
    /// becomes an empty chain via [`ArchiveChain::load`] / [`ArchiveChain::load_lenient`],
    /// so this is *only* "exists but cannot read".
    #[error("io error reading {path}: {source_msg}")]
    Io { path: String, source_msg: String },
}

/// Truncation ceiling for `MalformedJson::raw`. Keeps the SSB payload bounded
/// even when a tampered file contains multi-megabyte synthetic lines.
const MALFORMED_JSON_RAW_MAX: usize = 4096;

fn truncate_raw_line(raw: String) -> String {
    if raw.len() <= MALFORMED_JSON_RAW_MAX {
        return raw;
    }
    // Truncate on a char boundary to keep the JSON valid UTF-8.
    let mut idx = MALFORMED_JSON_RAW_MAX;
    while idx > 0 && !raw.is_char_boundary(idx) {
        idx -= 1;
    }
    let mut out = String::with_capacity(idx + 16);
    out.push_str(&raw[..idx]);
    out.push_str("...<truncated>");
    out
}

// ---------------------------------------------------------------------------
// StagedEvalResult
// ---------------------------------------------------------------------------

/// Per-template staged evaluation metrics. Populated by D-5 (BAI Thompson
/// promotion) + D-7 (cross-domain canary). D-1 only carries the shape so the
/// archive entry on disk has space for the future fields.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StagedEvalResult {
    pub stage_10_pass: u32,
    pub stage_10_total: u32,
    pub stage_50_pass: u32,
    pub stage_50_total: u32,
    pub stage_200_pass: u32,
    pub stage_200_total: u32,
    /// Per-domain delta from the baseline. Filled by D-7 canary; empty in
    /// D-1 entries.
    pub cross_domain_deltas: Vec<(Domain, f32)>,
}

// ---------------------------------------------------------------------------
// ArchiveEntry
// ---------------------------------------------------------------------------

/// One row in the archive chain. The `self_hash` is computed from
/// `(template.template_body, parent_hash)` and stored explicitly so the
/// chain walk is `O(N)` line-by-line rather than rebuilding parent
/// resolution per row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveEntry {
    pub self_hash: Blake3Hash,
    pub parent_hash: Option<Blake3Hash>,
    pub template: ReasoningTemplate,
    pub staged_eval_result: Option<StagedEvalResult>,
    pub promoted_at_ms: Option<i64>,
    pub retired_at_ms: Option<i64>,
    pub retirement_reason: Option<RetirementReason>,
}

impl ArchiveEntry {
    /// Construct a root entry (no parent) for a newly-introduced template.
    /// `self_hash` is computed from the template body. The caller is
    /// responsible for setting `template.content_hash` to match.
    #[must_use]
    pub fn new_root(template: ReasoningTemplate) -> Self {
        let self_hash = compute_content_hash(template.template_body.as_str(), None);
        Self {
            self_hash,
            parent_hash: None,
            template,
            staged_eval_result: None,
            promoted_at_ms: None,
            retired_at_ms: None,
            retirement_reason: None,
        }
    }

    /// Construct a child entry with a known parent hash.
    #[must_use]
    pub fn new_child(template: ReasoningTemplate, parent_hash: Blake3Hash) -> Self {
        let self_hash =
            compute_content_hash(template.template_body.as_str(), Some(&parent_hash));
        Self {
            self_hash,
            parent_hash: Some(parent_hash),
            template,
            staged_eval_result: None,
            promoted_at_ms: None,
            retired_at_ms: None,
            retirement_reason: None,
        }
    }
}

// ---------------------------------------------------------------------------
// ArchiveChain
// ---------------------------------------------------------------------------

/// Append-only chain of [`ArchiveEntry`] rows. Lives on disk as
/// `.elai_cc/archive/reasoning-templates.jsonl` (one entry per line).
///
/// Concurrency: D-1 ships a single-writer assumption (the migration tool +
/// the future D-5 BAI processor are the only writers). Cross-process locking
/// is deferred to D-5/D-8 — the spec marks `fs2` as optional and we did not
/// add a new dep this session. See `notes` in the return summary.
#[derive(Debug, Clone)]
pub struct ArchiveChain {
    storage_path: PathBuf,
    entries: Vec<ArchiveEntry>,
}

impl ArchiveChain {
    /// Construct an in-memory chain bound to `storage_path`. Does NOT read
    /// disk — use [`ArchiveChain::load`] for that.
    #[must_use]
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            entries: Vec::new(),
        }
    }

    /// Load the chain from disk. Missing file = empty chain (Ok). Malformed
    /// JSON = [`TemplateError::Serde`]. I/O error = [`TemplateError::Io`].
    pub fn load(storage_path: PathBuf) -> Result<Self, TemplateError> {
        let mut chain = Self::new(storage_path.clone());
        if !storage_path.exists() {
            return Ok(chain);
        }
        let file =
            std::fs::File::open(&storage_path).map_err(|e| TemplateError::Io(e.to_string()))?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.map_err(|e| TemplateError::Io(e.to_string()))?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: ArchiveEntry =
                serde_json::from_str(&line).map_err(|e| TemplateError::Serde(e.to_string()))?;
            chain.entries.push(entry);
        }
        Ok(chain)
    }

    /// D-8 (SCHEMA_VERSION 115) — lenient loader for `elai archive verify`.
    ///
    /// Returns a populated [`ArchiveChain`] paired with a list of
    /// [`ArchiveVerifyError::MalformedJson`] rows, one per JSONL line that
    /// failed to parse. Unlike [`ArchiveChain::load`] (which short-circuits
    /// on the first parse error so production callers never operate on a
    /// partial chain), this variant keeps walking so the verify driver can
    /// report every parse failure in one pass.
    ///
    /// Semantics:
    /// * Missing file -> Ok((empty chain, empty errors)).
    /// * I/O failure during open or line read -> Err([`ArchiveVerifyError::Io`]).
    /// * Empty / whitespace-only lines are skipped without incrementing the
    ///   entry index (matches [`ArchiveChain::load`]).
    /// * Lines that fail to deserialise are recorded as MalformedJson and
    ///   skipped; subsequent lines continue to be parsed.
    /// * Lines that deserialise successfully are appended to `chain.entries`.
    ///
    /// `entry_index` on MalformedJson rows tracks the position among
    /// *attempted parses* (i.e. non-blank lines), so it lines up 1:1 with
    /// the index a successful load would have used for the entry. This is
    /// the index the dashboard / CI log highlights.
    pub fn load_lenient(
        storage_path: PathBuf,
    ) -> Result<(Self, Vec<ArchiveVerifyError>), ArchiveVerifyError> {
        let mut chain = Self::new(storage_path.clone());
        let mut errors: Vec<ArchiveVerifyError> = Vec::new();
        if !storage_path.exists() {
            return Ok((chain, errors));
        }
        let path_str = storage_path.to_string_lossy().into_owned();
        let file = std::fs::File::open(&storage_path).map_err(|e| ArchiveVerifyError::Io {
            path: path_str.clone(),
            source_msg: e.to_string(),
        })?;
        let reader = BufReader::new(file);
        let mut entry_cursor: u32 = 0;
        for line in reader.lines() {
            let line = line.map_err(|e| ArchiveVerifyError::Io {
                path: path_str.clone(),
                source_msg: e.to_string(),
            })?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<ArchiveEntry>(&line) {
                Ok(entry) => {
                    chain.entries.push(entry);
                }
                Err(parse_err) => {
                    errors.push(ArchiveVerifyError::MalformedJson {
                        entry_index: entry_cursor,
                        raw: truncate_raw_line(line),
                        parse_error: parse_err.to_string(),
                    });
                }
            }
            entry_cursor = entry_cursor.saturating_add(1);
        }
        Ok((chain, errors))
    }

    /// On-disk path this chain reads/writes.
    #[must_use]
    pub fn storage_path(&self) -> &Path {
        &self.storage_path
    }

    /// Total entry count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True if no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// All entries, in append order.
    #[must_use]
    pub fn entries(&self) -> &[ArchiveEntry] {
        &self.entries
    }

    /// Most-recent N entries (in append order). Used by
    /// `get_archive_recent` Tauri command.
    #[must_use]
    pub fn recent(&self, n: usize) -> Vec<ArchiveEntry> {
        let start = self.entries.len().saturating_sub(n);
        self.entries[start..].to_vec()
    }

    /// Find an entry by template id.
    #[must_use]
    pub fn find_by_template_id(
        &self,
        id: &crate::reasoning_template::TemplateId,
    ) -> Option<&ArchiveEntry> {
        self.entries.iter().find(|e| e.template.id == *id)
    }

    /// Append `entry` to the in-memory + on-disk chain (O_APPEND only — never
    /// rewrite).
    ///
    /// We deliberately fsync via `BufWriter::flush` + `File::sync_all` so a
    /// crash mid-append leaves the file in one of two states: pre-append or
    /// fully-appended. Partial JSON lines would break `load()`; we forbid
    /// them.
    pub fn append(&mut self, entry: ArchiveEntry) -> Result<(), TemplateError> {
        // Ensure parent dir exists. Created lazily so a fresh repo doesn't
        // need to pre-create `.elai_cc/archive/`.
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| TemplateError::Io(e.to_string()))?;
        }

        let line = serde_json::to_string(&entry)
            .map_err(|e| TemplateError::Serde(e.to_string()))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.storage_path)
            .map_err(|e| TemplateError::Io(e.to_string()))?;
        writeln!(file, "{line}").map_err(|e| TemplateError::Io(e.to_string()))?;
        file.sync_all()
            .map_err(|e| TemplateError::Io(e.to_string()))?;

        self.entries.push(entry);
        Ok(())
    }

    /// Walk the chain, recomputing `self_hash` per entry from
    /// `(template_body, parent_hash)`, and verifying every parent_hash
    /// resolves to a previously-seen entry.
    ///
    /// Returns the index of the first broken entry on failure (so the dash
    /// can highlight the row), wrapped in a `TemplateError`.
    pub fn verify_chain(&self) -> Result<(), TemplateError> {
        let mut seen_hashes: HashSet<Blake3Hash> = HashSet::new();
        for (idx, entry) in self.entries.iter().enumerate() {
            // Parent reachability.
            if let Some(parent_hash) = entry.parent_hash {
                if !seen_hashes.contains(&parent_hash) {
                    warn!(
                        index = idx,
                        template_id = entry.template.id.as_hex(),
                        "ArchiveChain::verify_chain: parent hash not seen earlier in chain"
                    );
                    return Err(TemplateError::ParentNotFound {
                        parent_id: entry.template.id.as_hex(),
                    });
                }
            }
            // Self-hash recomputation.
            let recomputed = compute_content_hash(
                entry.template.template_body.as_str(),
                entry.parent_hash.as_ref(),
            );
            if recomputed != entry.self_hash {
                warn!(
                    index = idx,
                    template_id = entry.template.id.as_hex(),
                    "ArchiveChain::verify_chain: self_hash mismatch"
                );
                return Err(TemplateError::HashMismatch {
                    template_id: entry.template.id.as_hex(),
                });
            }
            seen_hashes.insert(entry.self_hash);
        }
        Ok(())
    }

    /// D-5: in-memory mutator used by [`crate::bai::promote_candidate`] when
    /// a Beta-posterior Thompson sample fires the promotion rule.
    ///
    /// Locates the entry by `template_id`, flips
    /// `template.archive_status` from `Candidate` to `Promoted`, attaches
    /// the StagedEvalResult, and sets `promoted_at_ms`. Returns `true` on
    /// success, `false` if no entry with that id exists in the in-memory
    /// chain.
    ///
    /// IMPORTANT: this mutates the in-memory entry only. Persisting the
    /// status flip to disk is the responsibility of the wiring layer
    /// (`crate::bai::emit_bai_round`), which records the promotion by
    /// appending a new chain entry rather than rewriting the JSONL line —
    /// preserving the D-1 append-only invariant.
    pub fn set_entry_promoted(
        &mut self,
        template_id: TemplateId,
        eval: StagedEvalResult,
        promoted_at_ms: i64,
    ) -> bool {
        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.template.id == template_id)
        {
            entry.template.archive_status = ArchiveStatus::Promoted;
            entry.staged_eval_result = Some(eval);
            entry.promoted_at_ms = Some(promoted_at_ms);
            true
        } else {
            false
        }
    }

    /// Build a [`TemplateRegistry`] from this chain. Entries with
    /// `archive_status in {Promoted, Frozen}` are indexed; `Candidate` +
    /// `Retired` are skipped.
    #[must_use]
    pub fn build_registry(&self) -> TemplateRegistry {
        let mut reg = TemplateRegistry::new();
        for entry in &self.entries {
            match entry.template.archive_status {
                ArchiveStatus::Promoted | ArchiveStatus::Frozen => {
                    reg.insert(entry.template.clone());
                }
                ArchiveStatus::Candidate | ArchiveStatus::Retired => {}
            }
        }
        reg
    }
}

// ---------------------------------------------------------------------------
// D-8 — full-walk verify_archive (collect-all + DuplicateHash detection)
// ---------------------------------------------------------------------------

/// D-8 — collect-all archive verify report. Mirrors the CLI driver's
/// in-memory return shape so the contract test, the CLI, and any
/// future programmatic consumer share one canonical walker.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ArchiveVerifyReport {
    pub archive_path: String,
    pub total_entries: u32,
    pub verified: u32,
    pub errors: Vec<ArchiveVerifyError>,
    pub passed: bool,
    pub fail_fast: bool,
}

/// D-8 — full-walk archive verify. Uses [`ArchiveChain::load_lenient`] to
/// surface MalformedJson without short-circuiting, walks every entry
/// (recomputing self_hash + checking parent reachability + detecting
/// duplicate self_hash across entries), and returns a populated
/// [`ArchiveVerifyReport`].
///
/// `fail_fast=true` short-circuits on the first error (matches CI mode);
/// `fail_fast=false` collects every error encountered (dev mode).
///
/// `Err` is reserved for infrastructural failures (path-unreadable I/O).
/// Tamper / mismatch / duplicate / malformed-JSON findings are encoded
/// inside `report.errors`.
pub fn verify_archive(
    archive_path: &std::path::Path,
    fail_fast: bool,
) -> Result<ArchiveVerifyReport, ArchiveVerifyError> {
    use std::collections::HashMap;

    let path_str = archive_path.to_string_lossy().into_owned();

    let (chain, parse_errors) = ArchiveChain::load_lenient(archive_path.to_path_buf())?;
    let total_entries = u32::try_from(chain.len()).unwrap_or(u32::MAX);

    let mut errors: Vec<ArchiveVerifyError> = Vec::new();
    for err in parse_errors {
        errors.push(err);
        if fail_fast {
            return Ok(ArchiveVerifyReport {
                archive_path: path_str,
                total_entries,
                verified: 0,
                errors,
                passed: false,
                fail_fast,
            });
        }
    }

    if chain.is_empty() && errors.is_empty() {
        return Ok(ArchiveVerifyReport {
            archive_path: path_str,
            total_entries: 0,
            verified: 0,
            errors,
            passed: true,
            fail_fast,
        });
    }

    let mut verified: u32 = 0;
    let mut seen_hash_to_idx: HashMap<String, u32> = HashMap::new();

    for (idx, entry) in chain.entries().iter().enumerate() {
        let entry_index = u32::try_from(idx).unwrap_or(u32::MAX);
        let template_id = entry.template.id.as_hex();
        let self_hash_hex = entry.self_hash.as_hex();

        if let Some(&first_idx) = seen_hash_to_idx.get(&self_hash_hex) {
            errors.push(ArchiveVerifyError::DuplicateHash {
                hash: self_hash_hex.clone(),
                first_at_index: first_idx,
                second_at_index: entry_index,
            });
            if fail_fast {
                break;
            }
        }

        if let Some(parent_hash) = entry.parent_hash {
            let parent_hex = parent_hash.as_hex();
            if !seen_hash_to_idx.contains_key(&parent_hex) {
                errors.push(ArchiveVerifyError::BrokenParent {
                    entry_index,
                    parent_hash: parent_hex,
                    template_id: template_id.clone(),
                });
                if fail_fast {
                    break;
                }
                seen_hash_to_idx
                    .entry(self_hash_hex)
                    .or_insert(entry_index);
                continue;
            }
        }

        let recomputed = compute_content_hash(
            entry.template.template_body.as_str(),
            entry.parent_hash.as_ref(),
        );
        if recomputed != entry.self_hash {
            errors.push(ArchiveVerifyError::HashMismatch {
                entry_index,
                expected: self_hash_hex.clone(),
                computed: recomputed.as_hex(),
                template_id,
            });
            if fail_fast {
                break;
            }
        } else {
            verified = verified.saturating_add(1);
        }

        seen_hash_to_idx
            .entry(self_hash_hex)
            .or_insert(entry_index);
    }

    let passed = errors.is_empty();
    Ok(ArchiveVerifyReport {
        archive_path: path_str,
        total_entries,
        verified,
        errors,
        passed,
        fail_fast,
    })
}

// ---------------------------------------------------------------------------
// FSM gate helper (consumer of verify_chain)
// ---------------------------------------------------------------------------

/// FSM / audit dispatcher gate consumed before template-promotion.
/// Returns `Ok(())` only if the chain on disk is internally consistent.
/// Production callers (D-5 BAI processor, audit dispatcher) MUST call this
/// before promoting a candidate; on failure the dashboard MutationArchivePanel
/// surfaces the broken row + the operator manually resolves it.
///
/// Implemented as a free function so the wiring layer can call it without
/// holding the orchestrator's task_runner state.
pub fn pre_promotion_chain_gate(chain: &ArchiveChain) -> Result<(), TemplateError> {
    chain.verify_chain()
}

/// Producer: append `entry` to `chain` AND emit an `ArchiveEntryAdded` SSB
/// row. The orchestrator path (task_runner / agent_pool / D-5 BAI processor)
/// calls this rather than [`ArchiveChain::append`] directly so the dashboard
/// MutationArchivePanel sees live updates without polling the JSONL file.
///
/// SSB write errors are logged + swallowed (the chain append already
/// succeeded on disk — we never re-throw to caller because that would
/// leave the on-disk + in-memory state mismatched with the visible SSB
/// surface). The append error IS propagated.
pub async fn append_and_emit_ssb(
    chain: &mut ArchiveChain,
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    entry: ArchiveEntry,
) -> Result<(), TemplateError> {
    // 1. Capture the metadata BEFORE move into chain.append.
    let template_id = entry.template.id.as_hex();
    let content_hash = entry.self_hash.as_hex();
    let parent_hash = entry.parent_hash.map(|h| h.as_hex());
    let domain = entry.template.domain.as_str().to_string();
    let pattern_kind = entry.template.pattern_kind.as_str().to_string();
    let archive_status = entry.template.archive_status.as_str().to_string();
    let path_for_files = chain.storage_path().to_string_lossy().into_owned();

    // 2. Append (propagates I/O / serde errors).
    chain.append(entry)?;

    // 3. Emit SSB row. Best-effort — errors logged, not returned.
    let payload = SsbPayload::ArchiveEntryAdded(ArchiveEntryAddedPayload {
        template_id,
        content_hash,
        parent_hash,
        domain,
        pattern_kind,
        archive_status,
    });
    if let Err(e) = ssb
        .write_payload(agent_id, superstep, payload, vec![path_for_files])
        .await
    {
        warn!(error = %e, "archive_chain::append_and_emit_ssb: SSB write failed");
    }
    Ok(())
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reasoning_template::{
        compute_content_hash, ArchiveStatus, Blake3Hash, Domain, PatternKind, Predicate,
        ReasoningTemplate, TemplateBody, TemplateId, VerifierKind,
    };
    use tempfile::TempDir;

    fn mk_template(body: &str, status: ArchiveStatus) -> ReasoningTemplate {
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

    fn jsonl_path(dir: &TempDir) -> PathBuf {
        dir.path().join("archive.jsonl")
    }

    #[test]
    fn append_then_load_round_trips() {
        let dir = TempDir::new().expect("tempdir");
        let path = jsonl_path(&dir);
        let mut chain = ArchiveChain::new(path.clone());
        let t = mk_template("hello", ArchiveStatus::Frozen);
        let entry = ArchiveEntry::new_root(t.clone());
        chain.append(entry.clone()).expect("append");

        // Round-trip.
        let reloaded = ArchiveChain::load(path).expect("load");
        assert_eq!(reloaded.len(), 1);
        assert_eq!(reloaded.entries[0].template.id, t.id);
        assert_eq!(reloaded.entries[0].self_hash, entry.self_hash);
    }

    #[test]
    fn verify_chain_ok_on_clean_chain() {
        let dir = TempDir::new().expect("tempdir");
        let mut chain = ArchiveChain::new(jsonl_path(&dir));
        let t1 = mk_template("root", ArchiveStatus::Frozen);
        let root = ArchiveEntry::new_root(t1);
        let root_hash = root.self_hash;
        chain.append(root).expect("append root");

        let t2 = mk_template("child", ArchiveStatus::Candidate);
        let child = ArchiveEntry::new_child(t2, root_hash);
        chain.append(child).expect("append child");

        chain.verify_chain().expect("clean verify");
    }

    #[test]
    fn verify_chain_detects_tampered_body() {
        let dir = TempDir::new().expect("tempdir");
        let mut chain = ArchiveChain::new(jsonl_path(&dir));
        let t = mk_template("clean", ArchiveStatus::Frozen);
        let mut entry = ArchiveEntry::new_root(t);
        // Tamper: mutate body but leave self_hash stale.
        entry.template.template_body = TemplateBody::try_new("tampered").expect("fits");
        chain.entries.push(entry);
        match chain.verify_chain() {
            Err(TemplateError::HashMismatch { .. }) => {}
            other => panic!("expected HashMismatch, got {other:?}"),
        }
    }

    #[test]
    fn verify_chain_detects_broken_parent_link() {
        let dir = TempDir::new().expect("tempdir");
        let mut chain = ArchiveChain::new(jsonl_path(&dir));
        let t = mk_template("orphan", ArchiveStatus::Candidate);
        // Fake parent_hash that was never appended.
        let fake_parent = Blake3Hash([42u8; 32]);
        let entry = ArchiveEntry::new_child(t, fake_parent);
        chain.entries.push(entry);
        match chain.verify_chain() {
            Err(TemplateError::ParentNotFound { .. }) => {}
            other => panic!("expected ParentNotFound, got {other:?}"),
        }
    }

    #[test]
    fn build_registry_skips_candidate_and_retired() {
        let dir = TempDir::new().expect("tempdir");
        let mut chain = ArchiveChain::new(jsonl_path(&dir));
        let cand = mk_template("c", ArchiveStatus::Candidate);
        let prom = mk_template("p", ArchiveStatus::Promoted);
        let frozen = mk_template("f", ArchiveStatus::Frozen);
        let retired = mk_template("r", ArchiveStatus::Retired);
        chain.append(ArchiveEntry::new_root(cand)).expect("a1");
        chain.append(ArchiveEntry::new_root(prom.clone())).expect("a2");
        chain.append(ArchiveEntry::new_root(frozen.clone())).expect("a3");
        chain.append(ArchiveEntry::new_root(retired)).expect("a4");

        let reg = chain.build_registry();
        assert_eq!(reg.len(), 2);
        assert!(reg.get(&prom.id).is_some());
        assert!(reg.get(&frozen.id).is_some());
    }

    #[test]
    fn recent_n_returns_tail_in_order() {
        let dir = TempDir::new().expect("tempdir");
        let mut chain = ArchiveChain::new(jsonl_path(&dir));
        for i in 0..5 {
            let t = mk_template(&format!("body-{i}"), ArchiveStatus::Frozen);
            chain.append(ArchiveEntry::new_root(t)).expect("append");
        }
        let tail = chain.recent(3);
        assert_eq!(tail.len(), 3);
        assert_eq!(tail[0].template.template_body.as_str(), "body-2");
        assert_eq!(tail[2].template.template_body.as_str(), "body-4");
    }

    #[test]
    fn recent_n_with_n_greater_than_len_returns_all() {
        let dir = TempDir::new().expect("tempdir");
        let mut chain = ArchiveChain::new(jsonl_path(&dir));
        let t = mk_template("only", ArchiveStatus::Frozen);
        chain.append(ArchiveEntry::new_root(t)).expect("append");
        let all = chain.recent(100);
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn load_missing_file_is_ok_empty() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("does-not-exist.jsonl");
        let chain = ArchiveChain::load(path).expect("load missing");
        assert!(chain.is_empty());
    }

    #[test]
    fn pre_promotion_chain_gate_returns_ok_on_clean() {
        let dir = TempDir::new().expect("tempdir");
        let mut chain = ArchiveChain::new(jsonl_path(&dir));
        let t = mk_template("body", ArchiveStatus::Frozen);
        chain.append(ArchiveEntry::new_root(t)).expect("append");
        pre_promotion_chain_gate(&chain).expect("gate ok");
    }

    /// observable_effect smoke: drop an ArchiveEntry, call verify_chain(), and
    /// assert get_archive_recent's underlying chain.recent() returns the row.
    /// This is the integration test required by the WIRING_CONTRACT's
    /// observable_effect block.
    #[test]
    fn observable_effect_archive_entry_visible_via_recent() {
        let dir = TempDir::new().expect("tempdir");
        let mut chain = ArchiveChain::new(jsonl_path(&dir));
        let t = mk_template("smoke body", ArchiveStatus::Frozen);
        let template_id = t.id;
        let entry = ArchiveEntry::new_root(t);
        chain.append(entry).expect("append");

        // verify_chain returns Ok (clean chain).
        chain.verify_chain().expect("verify clean");

        // recent(10) — the underlying call get_archive_recent makes — returns
        // the inserted row.
        let rows = chain.recent(10);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].template.id, template_id);
        assert_eq!(rows[0].template.template_body.as_str(), "smoke body");
        assert_eq!(rows[0].template.archive_status, ArchiveStatus::Frozen);
    }
}
