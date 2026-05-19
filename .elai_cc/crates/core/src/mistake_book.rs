//! MistakeBook: structured constraint store for externalized error memory.
//!
//! NOT a narrative log. Stores structured constraints (forbidden diffs,
//! failure categories, root causes) injected into prompts as <=150 tokens.
//! Per baby step. Max 5 entries, oldest-evicted on overflow.
//!
//! Research grounding: Code-A1 (arXiv:2603.15611) MistakeBook mechanism,
//! CTIM-Rover finding (episodic text degrades small models 42%->31-40%),
//! GISclaw+OpenSage (external JSON state, max 150 token injection).

use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum mistake entries retained per baby step. Oldest evicted on overflow.
pub const MAX_ENTRIES_PER_STEP: usize = 5;

/// Maximum tokens for the injected CONSTRAINTS block.
pub const MAX_INJECTION_TOKENS: usize = 150;

/// Length of the short diff hash prefix shown in prompts (hex chars).
pub const DIFF_HASH_SHORT_LEN: usize = 12;

// ---------------------------------------------------------------------------
// DiffHash — blake3 hash of a failed patch for forbidden_diffs deduplication
// ---------------------------------------------------------------------------

/// Blake3 hash of a failed diff/patch. Used for forbidden_diffs deduplication.
/// Stored as raw 32 bytes; displays as hex.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DiffHash([u8; 32]);

impl DiffHash {
    /// Compute blake3 hash of a diff string.
    pub fn from_diff(diff: &str) -> Self {
        Self(*blake3::hash(diff.as_bytes()).as_bytes())
    }

    /// Compute blake3 hash of raw bytes.
    pub fn from_bytes(data: &[u8]) -> Self {
        Self(*blake3::hash(data).as_bytes())
    }

    /// Full hex representation (64 chars).
    pub fn to_hex(&self) -> String {
        blake3::Hash::from_bytes(self.0).to_hex().to_string()
    }

    /// Short hex prefix for prompt injection (DIFF_HASH_SHORT_LEN chars).
    pub fn to_short_hex(&self) -> String {
        self.to_hex()[..DIFF_HASH_SHORT_LEN].to_string()
    }

    /// Raw 32-byte array.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for DiffHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_short_hex())
    }
}

// ---------------------------------------------------------------------------
// FailureCategory — from FailureTaxonomy (Layer 1)
// ---------------------------------------------------------------------------

/// Coarse failure categories from the FailureTaxonomy (reasoning Layer 1).
/// Each variant maps to a deterministic diagnosis path.
///
/// C-21 Task 10 (Q6 decision): added T1Block / T2Block / PreToolBlock to
/// distinguish gate-driven failures from compiler / test / logic failures.
/// Allows downstream MistakeBook readers to bucket "block" mistakes
/// without parsing root_cause_json.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureCategory {
    CompileError,
    TestFailure,
    TypeMismatch,
    MissingImport,
    UndefinedSymbol,
    BorrowCheck,
    LifetimeError,
    LazyCode,
    EmptyBody,
    FileSizeExceeded,
    RuntimePanic,
    LogicError,
    Timeout,
    Unknown,
    /// C-21 Task 10 — Tier 1 pre-tool-use hook fired BLOCK (rule
    /// evaluator / lazy-code / size cap caught the action before the
    /// agent could run). Source string in synthetic Failed reason:
    /// "T1_BLOCK".
    T1Block,
    /// C-21 Task 10 — Tier 2 hook fired BLOCK (deprecated AST imports,
    /// banned syntax, etc.). Source string: "T2_BLOCK".
    T2Block,
    /// C-21 Task 10 — generic PreToolUse hook fired BLOCK without a
    /// specific tier discriminant (e.g. irreversible classifier,
    /// non-T1/T2 PreToolUse rule). Source string: "PRE_TOOL_BLOCK".
    PreToolBlock,
}

impl FailureCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CompileError => "compile_error",
            Self::TestFailure => "test_failure",
            Self::TypeMismatch => "type_mismatch",
            Self::MissingImport => "missing_import",
            Self::UndefinedSymbol => "undefined_symbol",
            Self::BorrowCheck => "borrow_check",
            Self::LifetimeError => "lifetime_error",
            Self::LazyCode => "lazy_code",
            Self::EmptyBody => "empty_body",
            Self::FileSizeExceeded => "file_size_exceeded",
            Self::RuntimePanic => "runtime_panic",
            Self::LogicError => "logic_error",
            Self::Timeout => "timeout",
            Self::Unknown => "unknown",
            Self::T1Block => "t1_block",
            Self::T2Block => "t2_block",
            Self::PreToolBlock => "pre_tool_block",
        }
    }
}

impl fmt::Display for FailureCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// MistakeEntry — single externalized error record
// ---------------------------------------------------------------------------

/// A single mistake record stored in SSB per baby step.
/// Structured constraint, not narrative.
///
/// CALLERS MUST NOT include variable names, identifier names, or code snippets
/// in any field (root_cause_json, action_taken, etc.). Episodic text with
/// variable names degrades small-model performance 42%→31-40% per CTIM-Rover
/// findings (arXiv:2603.15611). Only structured action codes are allowed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MistakeEntry {
    /// Which retry attempt produced this mistake (1-indexed).
    pub attempt_number: u32,
    /// Blake3 hash of the failed patch (for forbidden_diffs deduplication).
    pub diff_hash: DiffHash,
    /// Coarse failure category from FailureTaxonomy.
    pub failure_category: FailureCategory,
    /// Line number where the fault was localized (if available).
    pub faulty_line: Option<u32>,
    /// Structured root cause from micro-reflection (Layer 2), JSON string.
    /// Contains action field only, never variable names or code snippets.
    pub root_cause_json: Option<String>,
    /// What corrective action was taken: "decomposed", "templated", "branched", etc.
    pub action_taken: String,
    /// Unix timestamp (seconds) for ordering. Oldest evicted on overflow.
    pub timestamp: u64,
}

// ---------------------------------------------------------------------------
// ForbiddenDiffs — separate Vec<DiffHash> concept for cycle prevention
// ---------------------------------------------------------------------------

/// Explicit collection of forbidden diff hashes. Prevents regenerating
/// known-failed patches across retries. Injected as separate
/// "Do NOT produce patch {hash}" lines in the CONSTRAINTS block.
///
/// This is a distinct concept from MistakeEntry: a MistakeEntry carries
/// structured diagnosis metadata, while ForbiddenDiffs is the pure blocklist.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForbiddenDiffs {
    hashes: Vec<DiffHash>,
}

impl ForbiddenDiffs {
    pub fn new() -> Self {
        Self { hashes: Vec::new() }
    }

    /// Collect forbidden diffs from a set of MistakeEntries.
    pub fn from_entries(entries: &[MistakeEntry]) -> Self {
        Self {
            hashes: entries.iter().map(|e| e.diff_hash.clone()).collect(),
        }
    }

    /// Add a new diff hash (deduplicates).
    pub fn add(&mut self, hash: DiffHash) {
        if !self.hashes.iter().any(|h| h == &hash) {
            self.hashes.push(hash);
        }
    }

    /// Check if a diff is forbidden.
    pub fn contains(&self, hash: &DiffHash) -> bool {
        self.hashes.iter().any(|h| h == hash)
    }

    /// Number of forbidden diffs tracked.
    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    /// Whether the forbidden list is empty.
    pub fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }

    /// Get all forbidden diff hashes.
    pub fn hashes(&self) -> &[DiffHash] {
        &self.hashes
    }

    /// Format as injection lines for the CONSTRAINTS block.
    /// Each line: "Do NOT produce patch {hash_short}."
    pub fn format_injection(&self) -> String {
        if self.hashes.is_empty() {
            return String::new();
        }
        let mut out = String::new();
        for hash in &self.hashes {
            out.push_str(&format!(
                "- Do NOT produce patch {}.\n",
                hash.to_short_hex()
            ));
        }
        let summary: Vec<String> = self.hashes.iter().map(|h| h.to_short_hex()).collect();
        out.push_str(&format!("- Forbidden patterns: {}\n", summary.join(", ")));
        out
    }
}

// ---------------------------------------------------------------------------
// Prompt injection formatter
// ---------------------------------------------------------------------------

/// Format a set of MistakeEntries into the CONSTRAINTS block for prompt injection.
/// Output is structured data, not narrative. Max `MAX_INJECTION_TOKENS` tokens
/// (approximated as ~4 chars per token).
///
/// Format:
/// ```text
/// CONSTRAINTS (from previous attempts):
/// - Do NOT produce patch {hash}. (failed: {category})
/// - Line {line}: {action} (from diagnosis)
/// - Forbidden patterns: {hash1}, {hash2}, ...
/// ```
pub fn format_constraints(entries: &[MistakeEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    // ~4 chars per token, budget = MAX_INJECTION_TOKENS * 4
    let char_budget = MAX_INJECTION_TOKENS * 4;
    let mut out = String::with_capacity(char_budget);
    out.push_str("CONSTRAINTS (from previous attempts):\n");

    // Per-entry constraint lines
    for entry in entries {
        let line = format!(
            "- Do NOT produce patch {}. (failed: {})\n",
            entry.diff_hash.to_short_hex(),
            entry.failure_category,
        );
        if out.len() + line.len() > char_budget {
            break;
        }
        out.push_str(&line);

        // Add localization hint if available
        if let (Some(line_num), Some(root_cause)) = (entry.faulty_line, entry.root_cause_json.as_deref()) {
            let hint = format!("- Line {line_num}: {root_cause}\n");
            if out.len() + hint.len() <= char_budget {
                out.push_str(&hint);
            }
        }
    }

    // Forbidden patterns summary line
    let hashes: Vec<String> = entries.iter().map(|e| e.diff_hash.to_short_hex()).collect();
    let forbidden = format!("- Forbidden patterns: {}\n", hashes.join(", "));
    if out.len() + forbidden.len() <= char_budget {
        out.push_str(&forbidden);
    }

    out
}

/// Check if a diff hash already exists in the entries (deduplication).
pub fn contains_diff(entries: &[MistakeEntry], hash: &DiffHash) -> bool {
    entries.iter().any(|e| &e.diff_hash == hash)
}

/// Insert a new entry, evicting the oldest if at capacity.
/// Returns the evicted entry if overflow occurred.
pub fn insert_with_eviction(entries: &mut Vec<MistakeEntry>, entry: MistakeEntry) -> Option<MistakeEntry> {
    // Deduplicate by diff_hash
    if contains_diff(entries, &entry.diff_hash) {
        return None;
    }

    if entries.len() >= MAX_ENTRIES_PER_STEP {
        // Evict oldest (lowest timestamp)
        let oldest_idx = entries
            .iter()
            .enumerate()
            .min_by_key(|(_, e)| e.timestamp)
            .map(|(i, _)| i)
            .unwrap_or(0);
        let evicted = entries.remove(oldest_idx);
        entries.push(entry);
        Some(evicted)
    } else {
        entries.push(entry);
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(attempt: u32, diff: &str, ts: u64) -> MistakeEntry {
        MistakeEntry {
            attempt_number: attempt,
            diff_hash: DiffHash::from_diff(diff),
            failure_category: FailureCategory::CompileError,
            faulty_line: Some(42),
            root_cause_json: Some("missing semicolon".to_string()),
            action_taken: "templated".to_string(),
            timestamp: ts,
        }
    }

    #[test]
    fn diff_hash_deterministic() {
        let h1 = DiffHash::from_diff("--- a/foo.rs\n+++ b/foo.rs\n-old\n+new");
        let h2 = DiffHash::from_diff("--- a/foo.rs\n+++ b/foo.rs\n-old\n+new");
        assert_eq!(h1, h2);
        assert_eq!(h1.to_hex().len(), 64);
        assert_eq!(h1.to_short_hex().len(), DIFF_HASH_SHORT_LEN);
    }

    #[test]
    fn diff_hash_different_inputs() {
        let h1 = DiffHash::from_diff("patch-A");
        let h2 = DiffHash::from_diff("patch-B");
        assert_ne!(h1, h2);
    }

    #[test]
    fn format_constraints_empty() {
        assert_eq!(format_constraints(&[]), "");
    }

    #[test]
    fn format_constraints_single_entry() {
        let entries = vec![make_entry(1, "diff-1", 100)];
        let out = format_constraints(&entries);
        assert!(out.starts_with("CONSTRAINTS (from previous attempts):"));
        assert!(out.contains("Do NOT produce patch"));
        assert!(out.contains("compile_error"));
        assert!(out.contains("Line 42"));
        assert!(out.contains("Forbidden patterns:"));
    }

    #[test]
    fn format_constraints_within_token_budget() {
        let entries: Vec<_> = (0..MAX_ENTRIES_PER_STEP)
            .map(|i| make_entry(i as u32, &format!("diff-{i}"), i as u64))
            .collect();
        let out = format_constraints(&entries);
        // ~4 chars per token
        let approx_tokens = out.len() / 4;
        assert!(
            approx_tokens <= MAX_INJECTION_TOKENS + 10, // small margin for rounding
            "injection too large: ~{approx_tokens} tokens, budget {MAX_INJECTION_TOKENS}"
        );
    }

    #[test]
    fn insert_with_eviction_under_capacity() {
        let mut entries = Vec::new();
        let evicted = insert_with_eviction(&mut entries, make_entry(1, "d1", 100));
        assert!(evicted.is_none());
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn insert_with_eviction_at_capacity() {
        let mut entries: Vec<_> = (0..MAX_ENTRIES_PER_STEP)
            .map(|i| make_entry(i as u32, &format!("d{i}"), (i + 1) as u64))
            .collect();
        assert_eq!(entries.len(), MAX_ENTRIES_PER_STEP);

        let evicted = insert_with_eviction(&mut entries, make_entry(99, "new-diff", 100));
        assert!(evicted.is_some());
        let evicted = evicted.unwrap();
        // Should have evicted the one with lowest timestamp (1)
        assert_eq!(evicted.timestamp, 1);
        assert_eq!(entries.len(), MAX_ENTRIES_PER_STEP);
    }

    #[test]
    fn insert_with_eviction_deduplicates() {
        let mut entries = vec![make_entry(1, "same-diff", 100)];
        let evicted = insert_with_eviction(&mut entries, make_entry(2, "same-diff", 200));
        assert!(evicted.is_none());
        assert_eq!(entries.len(), 1); // no duplicate added
    }

    #[test]
    fn contains_diff_works() {
        let entries = vec![make_entry(1, "target", 100)];
        let target_hash = DiffHash::from_diff("target");
        let other_hash = DiffHash::from_diff("other");
        assert!(contains_diff(&entries, &target_hash));
        assert!(!contains_diff(&entries, &other_hash));
    }

    #[test]
    fn failure_category_roundtrip() {
        let cat = FailureCategory::BorrowCheck;
        let json = serde_json::to_string(&cat).unwrap();
        assert_eq!(json, "\"borrow_check\"");
        let back: FailureCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cat);
    }

    #[test]
    fn forbidden_diffs_from_entries() {
        let entries = vec![
            make_entry(1, "diff-a", 100),
            make_entry(2, "diff-b", 200),
        ];
        let fd = ForbiddenDiffs::from_entries(&entries);
        assert_eq!(fd.len(), 2);
        assert!(fd.contains(&DiffHash::from_diff("diff-a")));
        assert!(fd.contains(&DiffHash::from_diff("diff-b")));
        assert!(!fd.contains(&DiffHash::from_diff("diff-c")));
    }

    #[test]
    fn forbidden_diffs_deduplicates() {
        let mut fd = ForbiddenDiffs::new();
        let hash = DiffHash::from_diff("same");
        fd.add(hash.clone());
        fd.add(hash.clone());
        assert_eq!(fd.len(), 1);
    }

    #[test]
    fn forbidden_diffs_format_injection() {
        let entries = vec![make_entry(1, "diff-x", 100)];
        let fd = ForbiddenDiffs::from_entries(&entries);
        let output = fd.format_injection();
        assert!(output.contains("Do NOT produce patch"));
        assert!(output.contains("Forbidden patterns:"));
    }

    #[test]
    fn forbidden_diffs_empty() {
        let fd = ForbiddenDiffs::new();
        assert!(fd.is_empty());
        assert_eq!(fd.format_injection(), "");
    }

    #[test]
    fn mistake_entry_serde_roundtrip() {
        let entry = make_entry(3, "test-diff", 999);
        let bytes = serde_json::to_vec(&entry).unwrap();
        let back: MistakeEntry = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back, entry);
    }
}
