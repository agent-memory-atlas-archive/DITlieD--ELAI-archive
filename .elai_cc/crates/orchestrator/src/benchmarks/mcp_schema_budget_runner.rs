//! H-34: MCP-schema token-budget MEASURE-FIRST runner.
//!
//! Computes the per-turn MCP-schema token baseline from a compile-time
//! embedded fixture corpus (mirror of `seccodeplt_runner.rs` structure:
//! `include_str!`, `corpus_blake3`, seeded determinism, no numeric literals
//! for computed results). Fully deterministic and offline (true W-5).
//!
//! # What this measures (honestly)
//!
//! The corpus represents a realistic multi-server MCP tool catalog (20 tools
//! across local-rag / pod-lifecycle / notion). For each corpus entry the
//! `schema_tokens` field approximates the prompt-token cost of including that
//! tool's full JSON schema in a turn. The runner computes:
//!
//! * `baseline_schema_tokens_per_turn` — cost when ALL tools are injected
//!   eagerly (no defer-load gate).
//! * `injected_schema_tokens_per_turn` — cost when ONLY the always-floor set
//!   (tools with `floor: true`) is injected (the defer-load ON-path minimum).
//! * `reduction_tokens` — `baseline - injected` (must be > 0 for a non-degenerate run).
//! * `deferred_resolves` — count of non-floor tools that would be deferred.
//!
//! Both token counts are COMPUTED from the corpus, never literals.
//!
//! # Honesty
//!
//! This is an OFFLINE approximation using corpus-estimated token counts.
//! The live multi-server measurement (real tokenizer, real RunPod sessions
//! with multiple connected MCP servers) is a benchmark-class deferral:
//! it requires a live RunPod environment and is tracked under H-34.
//!
//! # Wiring discipline
//!
//! - blake3 for corpus content hash (anti-stale rule).
//! - thiserror for errors; tracing for diagnostics.
//! - serde for the on-disk corpus + the report.
//! - No `.unwrap()` / `.expect()` outside `#[cfg(test)]`.
//! - Corpus embedded via `include_str!` (filesystem-independent).
//! - Cap assertion: `injected_schema_tokens_per_turn <= INJECTED_CAP_TOKENS`.
//!   The runner WARNS (does not panic) if the floor-only set would exceed the cap;
//!   the `run()` method returns `Err` if the assertion is violated after WARN.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};

use crate::determinism::ELAI_SEED_ENV;

// ---------------------------------------------------------------------------
// Embedded corpus
// ---------------------------------------------------------------------------

/// The MCP schema-budget fixture corpus, embedded at compile time.
/// One JSON array of [`McpToolEntry`] objects. Filesystem-independent.
const CORPUS_JSON: &str =
    include_str!("mcp_schema_budget_fixtures/corpus.json");

/// Suite label recorded in the report + the eventual SSB beacon.
pub const SUITE_LABEL: &str = "mcp_schema_budget";

/// Named cap constant: the maximum allowed `injected_schema_tokens_per_turn`
/// when the defer-load floor-only path is active. Must be <= 500 tokens
/// (per H-34 spec: always-injected slice stays under a 500-token cap).
/// Named const per CLAUDE.md "no magic numbers" rule.
pub const INJECTED_CAP_TOKENS: u64 = 500;

// ---------------------------------------------------------------------------
// Corpus entry
// ---------------------------------------------------------------------------

/// One MCP tool entry in the fixture corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpToolEntry {
    /// MCP server name (e.g. `"local-rag"`).
    pub server_name: String,
    /// Tool name within the server (e.g. `"mcp_search_context"`).
    pub tool_name: String,
    /// Fully-qualified tool name (e.g. `"local-rag__mcp_search_context"`).
    pub qualified_name: String,
    /// Natural-language description of the tool.
    pub description: String,
    /// Approximate prompt-token cost of including this tool's full JSON schema.
    /// COMPUTED from the corpus fixture; not a benchmark target literal.
    pub schema_tokens: u64,
    /// `true` = this tool is in the always-floor set and is ALWAYS injected.
    /// `false` = this tool is deferred unless selected by keyword or vector retrieval.
    pub floor: bool,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum McpSchemaBudgetError {
    /// The embedded corpus JSON failed to deserialise.
    #[error("mcp_schema_budget corpus malformed json: {0}")]
    MalformedCorpus(String),
    /// The corpus is empty (would make every rate degenerate).
    #[error("mcp_schema_budget corpus is empty")]
    EmptyCorpus,
    /// `ELAI_SEED` was set but not parseable as a u64.
    #[error("mcp_schema_budget invalid ELAI_SEED '{value}': {detail}")]
    InvalidSeed { value: String, detail: String },
    /// The injected floor-only set exceeds the named cap.
    #[error(
        "mcp_schema_budget injected_schema_tokens_per_turn ({injected}) \
         exceeds INJECTED_CAP_TOKENS ({cap}): trim the floor set"
    )]
    CapExceeded { injected: u64, cap: u64 },
    /// The corpus produced a degenerate result (no reduction from defer-load).
    #[error(
        "mcp_schema_budget baseline ({baseline}) must exceed injected ({injected}): \
         floor set is identical to the full catalog or corpus has no non-floor tools"
    )]
    NonDegenerate { baseline: u64, injected: u64 },
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

/// Aggregate report — one per [`McpSchemaBudgetRunner::run`] invocation.
///
/// All token counts are COMPUTED from the corpus fixture, never literals.
/// The contract asserts `baseline > injected > 0`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpSchemaBudgetReport {
    /// Suite label ([`SUITE_LABEL`]).
    pub suite: String,
    /// Baseline schema tokens per turn (all tools injected eagerly).
    /// COMPUTED: `sum(tool.schema_tokens for all tools in corpus)`.
    pub baseline_schema_tokens_per_turn: u64,
    /// Injected schema tokens per turn after defer-load (floor-only).
    /// COMPUTED: `sum(tool.schema_tokens for floor=true tools)`.
    pub injected_schema_tokens_per_turn: u64,
    /// Reduction in tokens: `baseline - injected` (signed; negative = regression).
    pub reduction_tokens: i64,
    /// Number of non-floor tools that would be deferred.
    pub deferred_resolves: u32,
    /// Total tools in the corpus.
    pub total_tools: u32,
    /// Tools in the always-floor set.
    pub floor_tools: u32,
    /// Deterministic RNG seed used (`ELAI_SEED`, default 0).
    pub seed: u64,
    /// blake3 hex of the canonical corpus bytes (provenance / staleness pin).
    pub corpus_blake3: String,
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// MCP schema-budget runner. Construction parses + validates the embedded
/// corpus once; [`McpSchemaBudgetRunner::run`] is a pure, deterministic
/// scoring pass (no I/O, no async).
#[derive(Debug)]
pub struct McpSchemaBudgetRunner {
    tools: Vec<McpToolEntry>,
    corpus_blake3: String,
    seed: u64,
}

impl McpSchemaBudgetRunner {
    /// Construct from the embedded corpus, reading `ELAI_SEED` (default 0) for
    /// the recorded seed. Returns `Err` on a malformed / empty corpus or an
    /// unparseable seed.
    pub fn from_embedded() -> Result<Self, McpSchemaBudgetError> {
        let seed = read_seed()?;
        Self::from_json_with_seed(CORPUS_JSON, seed)
    }

    /// Construct from explicit corpus JSON bytes + an explicit seed. Used by
    /// calibration / unit tests; production uses [`Self::from_embedded`].
    pub fn from_json_with_seed(json: &str, seed: u64) -> Result<Self, McpSchemaBudgetError> {
        let tools: Vec<McpToolEntry> = serde_json::from_str(json)
            .map_err(|e| McpSchemaBudgetError::MalformedCorpus(e.to_string()))?;
        if tools.is_empty() {
            return Err(McpSchemaBudgetError::EmptyCorpus);
        }
        let mut hasher = Hasher::new();
        hasher.update(json.as_bytes());
        let corpus_blake3 = hasher.finalize().to_hex().to_string();
        Ok(Self {
            tools,
            corpus_blake3,
            seed,
        })
    }

    /// Number of tools in the corpus.
    #[must_use]
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// The recorded seed.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// blake3 hex of the canonical corpus bytes.
    #[must_use]
    pub fn corpus_blake3(&self) -> &str {
        &self.corpus_blake3
    }

    /// Borrow the parsed tools (for calibration assertions).
    #[must_use]
    pub fn tools(&self) -> &[McpToolEntry] {
        &self.tools
    }

    /// Run the budget computation and produce the report.
    ///
    /// Deterministic: same corpus => byte-identical report. Both token counts
    /// are computed from `schema_tokens` in the corpus, never literals.
    ///
    /// Returns `Err(CapExceeded)` if the floor-only injected set exceeds
    /// [`INJECTED_CAP_TOKENS`] (after emitting a `tracing::warn`).
    ///
    /// Returns `Err(NonDegenerate)` if `baseline <= injected` (no reduction —
    /// the corpus must have at least one non-floor tool).
    pub fn run(&self) -> Result<McpSchemaBudgetReport, McpSchemaBudgetError> {
        let total_tools = u32::try_from(self.tools.len()).unwrap_or(u32::MAX);

        // Baseline: all tools injected eagerly.
        let baseline_schema_tokens_per_turn: u64 =
            self.tools.iter().map(|t| t.schema_tokens).sum();

        // Floor-only: only tools with floor=true are injected per turn.
        let floor_tools_list: Vec<&McpToolEntry> =
            self.tools.iter().filter(|t| t.floor).collect();
        let floor_tools = u32::try_from(floor_tools_list.len()).unwrap_or(u32::MAX);
        let injected_schema_tokens_per_turn: u64 =
            floor_tools_list.iter().map(|t| t.schema_tokens).sum();
        let deferred_resolves = total_tools.saturating_sub(floor_tools);

        info!(
            total_tools,
            floor_tools,
            baseline_schema_tokens_per_turn,
            injected_schema_tokens_per_turn,
            deferred_resolves,
            "mcp_schema_budget: starting budget computation",
        );

        // Cap check: warn then fail if the floor-only set exceeds the named cap.
        if injected_schema_tokens_per_turn > INJECTED_CAP_TOKENS {
            warn!(
                injected = injected_schema_tokens_per_turn,
                cap = INJECTED_CAP_TOKENS,
                "mcp_schema_budget: floor-only injected set exceeds cap; trim ALWAYS_AVAILABLE",
            );
            return Err(McpSchemaBudgetError::CapExceeded {
                injected: injected_schema_tokens_per_turn,
                cap: INJECTED_CAP_TOKENS,
            });
        }

        // Non-degeneracy: baseline must exceed injected (there must be non-floor tools).
        if baseline_schema_tokens_per_turn <= injected_schema_tokens_per_turn {
            return Err(McpSchemaBudgetError::NonDegenerate {
                baseline: baseline_schema_tokens_per_turn,
                injected: injected_schema_tokens_per_turn,
            });
        }

        let reduction_tokens = i64::try_from(baseline_schema_tokens_per_turn)
            .unwrap_or(i64::MAX)
            .saturating_sub(
                i64::try_from(injected_schema_tokens_per_turn).unwrap_or(i64::MAX),
            );

        let report = McpSchemaBudgetReport {
            suite: SUITE_LABEL.to_string(),
            baseline_schema_tokens_per_turn,
            injected_schema_tokens_per_turn,
            reduction_tokens,
            deferred_resolves,
            total_tools,
            floor_tools,
            seed: self.seed,
            corpus_blake3: self.corpus_blake3.clone(),
        };

        info!(
            baseline = report.baseline_schema_tokens_per_turn,
            injected = report.injected_schema_tokens_per_turn,
            reduction = report.reduction_tokens,
            deferred = report.deferred_resolves,
            "mcp_schema_budget: run complete",
        );

        Ok(report)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read `ELAI_SEED` (default 0). Errors only when the var is set but not a
/// valid u64 — an empty / unset var is the default-0 path.
fn read_seed() -> Result<u64, McpSchemaBudgetError> {
    match std::env::var(ELAI_SEED_ENV) {
        Ok(raw) if !raw.trim().is_empty() => {
            raw.trim()
                .parse::<u64>()
                .map_err(|e| McpSchemaBudgetError::InvalidSeed {
                    value: raw.clone(),
                    detail: e.to_string(),
                })
        }
        _ => Ok(0u64),
    }
}

// ---------------------------------------------------------------------------
// Tests + calibration
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn runner() -> McpSchemaBudgetRunner {
        McpSchemaBudgetRunner::from_json_with_seed(CORPUS_JSON, 0)
            .expect("embedded corpus parses")
    }

    #[test]
    fn corpus_has_at_least_ten_tools() {
        let r = runner();
        assert!(
            r.tool_count() >= 10,
            "corpus must have >=10 tools, got {}",
            r.tool_count()
        );
    }

    #[test]
    fn corpus_has_floor_and_non_floor_tools() {
        let r = runner();
        let floor_count = r.tools().iter().filter(|t| t.floor).count();
        let non_floor_count = r.tools().iter().filter(|t| !t.floor).count();
        assert!(floor_count > 0, "corpus must have >=1 floor tool");
        assert!(
            non_floor_count > 0,
            "corpus must have >=1 non-floor tool (required for non-degenerate run)"
        );
    }

    #[test]
    fn every_tool_has_required_fields() {
        let r = runner();
        for t in r.tools() {
            assert!(!t.server_name.is_empty(), "server_name required");
            assert!(!t.tool_name.is_empty(), "tool_name required");
            assert!(!t.qualified_name.is_empty(), "qualified_name required");
            assert!(
                t.schema_tokens > 0,
                "schema_tokens must be > 0 for tool {}",
                t.qualified_name
            );
        }
    }

    #[test]
    fn qualified_names_are_unique() {
        let r = runner();
        let mut names: Vec<&str> = r.tools().iter().map(|t| t.qualified_name.as_str()).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len(), "qualified_names must be unique");
    }

    #[test]
    fn report_is_non_degenerate_baseline_gt_injected() {
        let r = runner();
        let rep = r.run().expect("clean run");
        assert!(
            rep.baseline_schema_tokens_per_turn > 0,
            "baseline must be > 0"
        );
        assert!(
            rep.injected_schema_tokens_per_turn > 0,
            "injected must be > 0 (floor set is non-empty)"
        );
        assert!(
            rep.baseline_schema_tokens_per_turn > rep.injected_schema_tokens_per_turn,
            "baseline ({}) must exceed injected ({}) — defer-load must reduce tokens",
            rep.baseline_schema_tokens_per_turn,
            rep.injected_schema_tokens_per_turn
        );
        assert!(
            rep.reduction_tokens > 0,
            "reduction_tokens ({}) must be > 0",
            rep.reduction_tokens
        );
    }

    #[test]
    fn injected_is_below_cap() {
        let r = runner();
        let rep = r.run().expect("clean run");
        assert!(
            rep.injected_schema_tokens_per_turn <= INJECTED_CAP_TOKENS,
            "injected ({}) must be <= cap ({INJECTED_CAP_TOKENS})",
            rep.injected_schema_tokens_per_turn
        );
    }

    #[test]
    fn run_is_deterministic() {
        let r = runner();
        let a = r.run().expect("run a");
        let b = r.run().expect("run b");
        assert_eq!(a, b, "two runs must be byte-identical");
    }

    #[test]
    fn corpus_blake3_is_64_hex_chars() {
        let r = runner();
        assert_eq!(r.corpus_blake3().len(), 64, "blake3 hex digest is 64 chars");
        assert!(r.corpus_blake3().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn empty_corpus_is_rejected() {
        match McpSchemaBudgetRunner::from_json_with_seed("[]", 0) {
            Err(McpSchemaBudgetError::EmptyCorpus) => {}
            other => panic!("expected EmptyCorpus, got {other:?}"),
        }
    }

    #[test]
    fn malformed_corpus_is_rejected() {
        match McpSchemaBudgetRunner::from_json_with_seed("{not an array", 0) {
            Err(McpSchemaBudgetError::MalformedCorpus(_)) => {}
            other => panic!("expected MalformedCorpus, got {other:?}"),
        }
    }

    #[test]
    fn seed_is_recorded_in_report() {
        let r = McpSchemaBudgetRunner::from_json_with_seed(CORPUS_JSON, 42)
            .expect("parse");
        let rep = r.run().expect("run");
        assert_eq!(rep.seed, 42, "seed must round-trip into the report");
        assert_eq!(rep.suite, SUITE_LABEL);
    }

    #[test]
    fn report_internal_consistency() {
        let r = runner();
        let rep = r.run().expect("run");
        // reduction_tokens == baseline - injected
        let expected_reduction = i64::try_from(rep.baseline_schema_tokens_per_turn)
            .unwrap()
            .saturating_sub(i64::try_from(rep.injected_schema_tokens_per_turn).unwrap());
        assert_eq!(
            rep.reduction_tokens, expected_reduction,
            "reduction_tokens must equal baseline - injected"
        );
        // floor_tools + deferred_resolves == total_tools
        assert_eq!(
            rep.floor_tools + rep.deferred_resolves,
            rep.total_tools,
            "floor_tools + deferred_resolves must equal total_tools"
        );
    }
}
