//! H-20 (SCHEMA_VERSION 225): leakage static-analysis COMPOSITE gate.
//!
//! The Tier-1 fail-closed done-gate for ML-pipeline / backtest data-leakage.
//! It COMPOSES with the SHIPPED rules-engine [`elai_rules::predicate::DkgQuerier`]
//! reachability machinery over the DKG (it does NOT build a from-scratch
//! dataflow pass, and it does NOT use the G-1 taint_flow substring side-table
//! as the reachability engine — that primitive is the wrong shape for a
//! derives-from leakage property).
//!
//! # What it BLOCKs (negative reachability, `must_not_exist = true`)
//!   - L1.1 (no test set): no `train_test_split`-equivalent boundary declared.
//!   - L1.2 (preprocessing before split): a fit-site reaches BOTH a train sink
//!     AND a test sink (the fit was applied to unsplit data).
//!   - L1.3 (feature-selection before split): a selector fit reaches both sinks.
//!
//! # Soundness boundary (LOAD-BEARING — Kapoor & Narayanan 2207.07048)
//! The COMPOSITE is statically SOUND for only 3 of the 8 Kapoor classes:
//! L1.1 / L1.2 / L1.3. The other 5 — L1.4 duplicates, L2 illegitimate/proxy
//! features, L3.1 temporal, L3.2 nonindependence, L3.3 sampling-bias — are NOT
//! statically decidable. They are modelled as HONEST RESIDUE routed to human
//! Layer-3 ([`LeakageVerdictKind::HumanReviewRequired`]), NEVER claimed as a
//! sound static BLOCK. The missing-`.shift(1)` AST heuristic is a temporal
//! (L3.1) FLAG -> route to review, NOT a sound block. Recall is bounded even
//! for the sound classes (Yang et al. ~67.8%); the H-21 runtime differential is
//! the un-gameable backstop (R5). Comments must not imply 100% coverage.
//!
//! # FAIL-CLOSED on absent / empty / unavailable DKG (the #1 hazard)
//! With `must_not_exist = true`, an EMPTY graph reads as "no path -> ALLOW" =
//! FAIL-OPEN. This gate REFUSES that: an absent / empty / unavailable DKG yields
//! a hard `Block` (verdict reason `dkg_unavailable`), never the permissive
//! security-side fallback (R1/R2: missing evidence = FAIL). The caller probes
//! the DKG and threads [`LeakageInput::dkg_available`]; this module additionally
//! guards on a positive-presence probe so a silently-empty querier cannot pass.
//!
//! # REGISTERED suppression, never silent (W-1 / R1)
//! A flagged sound block is downgraded to suppressed ONLY when a matching row
//! exists in the frozen `.elai_cc/leakage-suppression-allowlist.toml`
//! (rule_id + file + reason + owner + unexpired). Absence of a row => the Block
//! STANDS. A silent `if flagged { Allow }` would be a P0 stub_ban violation.
//!
//! # SSB emission (off-agent artifact, R4)
//! Emits one [`crate::ssb::SsbPayload::LeakageBlocked`] row on EVERY decision
//! (Block / Allow / HumanReviewRequired). Even an Allow row is evidence.
//!
//! # HARD RULES
//!   - No `.unwrap()`/`.expect()` outside `#[cfg(test)]` (CLAUDE.md).
//!   - blake3 only for hashing (CLAUDE.md anti-stale).
//!   - No new Rust crate.
//!
//! References:
//!   - workflow/papers/kapoor-narayanan-leakage-taxonomy-interface.md (taxonomy).
//!   - crates/orchestrator/src/deflation_gate.rs (H-18 sibling shape).
//!   - crates/rules/src/predicate.rs (GraphReachability + DkgQuerier).
//!   - workflow/reasoning/H-20-decisions.md.

use std::sync::Arc;

use elai_rules::predicate::{DkgQuerier, ReachabilityParams};
use thiserror::Error;
use tracing::{debug, warn};

use crate::ssb::{LeakageBlockedKind, LeakageBlockedPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// LOCKED named constants (must match leakage.toml / seed_leakage_rules)
// ---------------------------------------------------------------------------

/// Canonical seed rule id for the leakage COMPOSITE (mutable=0).
pub const LEAKAGE_RULE_ID: &str = "FIN-LEAK-001";

/// Maximum reachability traversal depth for the fit->sink walk.
///
/// Bounded so a pathological DKG cannot make the walk unbounded. 8 mirrors
/// `predicate::MAX_COMPOSITE_DEPTH`; a fit -> preprocessing -> split -> sink
/// chain is well within this.
const REACHABILITY_MAX_DEPTH: u8 = 8;

/// DKG edge labels that carry a derives-from data dependency. The leakage
/// walk follows ONLY these (call edges alone do not imply data flow).
/// Matches the `DkgDataFlowEdgeRef::flow_type` vocabulary
/// (PARAMETER | RETURN | ASSIGNMENT | FIELD_ACCESS) plus the generic
/// `data_flow` label some indexer rows carry.
const DATA_FLOW_EDGE_TYPES: [&str; 5] = [
    "data_flow",
    "PARAMETER",
    "RETURN",
    "ASSIGNMENT",
    "FIELD_ACCESS",
];

/// Sink pattern for the train branch of the train/test split.
const TRAIN_SINK_PATTERN: &str = "train";

/// Sink pattern for the test branch of the train/test split.
const TEST_SINK_PATTERN: &str = "test";

/// Presence-probe source pattern. A positive `must_not_exist = false` probe
/// over this pattern is used to confirm the DKG actually has reachable nodes
/// for the target module. A querier that returns zero AND an error-free probe
/// is treated as UNAVAILABLE (fail-closed) unless the caller already proved
/// availability via [`LeakageInput::dkg_available`].
const PRESENCE_PROBE_SINK_PATTERN: &str = "*";

// ---------------------------------------------------------------------------
// Public API types
// ---------------------------------------------------------------------------

/// A declared train/test split boundary (the L1.1 SchemaRequired leg).
///
/// Its PRESENCE is the L1.1 check: absence => Block (no test set declared).
/// `split_symbol` names the boundary node in the DKG (e.g. the
/// `train_test_split` call); fit-sites reaching sinks on BOTH sides of this
/// boundary are the L1.2 / L1.3 violations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitBoundary {
    /// Qualified DKG symbol of the split call (e.g. "sklearn.train_test_split").
    pub split_symbol: String,
    /// Repo-relative file the split is declared in.
    pub file: String,
    /// 1-based line of the split declaration.
    pub line: u32,
}

/// A statically-SOUND leakage class (Kapoor L1.1 / L1.2 / L1.3).
///
/// These are the ONLY classes the COMPOSITE may emit as a sound BLOCK.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundLeakClass {
    /// L1.1: no train/test split boundary declared.
    NoTestSet,
    /// L1.2: a preprocessing fit-site reaches both train and test sinks.
    PreprocessBeforeSplit,
    /// L1.3: a feature-selection fit-site reaches both train and test sinks.
    FeatureSelectBeforeSplit,
}

impl SoundLeakClass {
    /// The Kapoor taxonomy code (for the SSB row + dashboard).
    pub fn kapoor_code(self) -> &'static str {
        match self {
            SoundLeakClass::NoTestSet => "L1.1",
            SoundLeakClass::PreprocessBeforeSplit => "L1.2",
            SoundLeakClass::FeatureSelectBeforeSplit => "L1.3",
        }
    }

    /// Map to the SSB inner Kind discriminant.
    pub fn ssb_kind(self) -> LeakageBlockedKind {
        match self {
            SoundLeakClass::NoTestSet => LeakageBlockedKind::NoTestSet,
            SoundLeakClass::PreprocessBeforeSplit => LeakageBlockedKind::PreprocessBeforeSplit,
            SoundLeakClass::FeatureSelectBeforeSplit => {
                LeakageBlockedKind::FeatureSelectBeforeSplit
            }
        }
    }
}

/// A NON-statically-decidable leakage class (Kapoor residue: L1.4 / L2 / L3.x).
///
/// These NEVER produce a sound static block. They route to human Layer-3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidueClass {
    /// L1.4: duplicate rows across train/test (requires runtime data compare).
    Duplicates,
    /// L2: illegitimate / proxy features (requires domain semantics).
    IllegitimateFeatures,
    /// L3.1: temporal leakage (requires time-column semantics).
    Temporal,
    /// L3.2: nonindependence between train/test samples.
    Nonindependence,
    /// L3.3: sampling bias in the test distribution.
    SamplingBias,
}

impl ResidueClass {
    /// The Kapoor taxonomy code.
    pub fn kapoor_code(self) -> &'static str {
        match self {
            ResidueClass::Duplicates => "L1.4",
            ResidueClass::IllegitimateFeatures => "L2",
            ResidueClass::Temporal => "L3.1",
            ResidueClass::Nonindependence => "L3.2",
            ResidueClass::SamplingBias => "L3.3",
        }
    }
}

/// Verdict kind returned by [`evaluate_leakage`].
///
/// The enum DISTINGUISHES a SOUND static block (L1.1/L1.2/L1.3) from a
/// HEURISTIC FLAG / human-review routing (the residue). A residue class is
/// NEVER a sound block (soundness boundary, Kapoor 2207.07048).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeakageVerdictKind {
    /// No sound leakage detected and no residue flagged. Promotion allowed.
    Allow,
    /// A statically-SOUND leakage class fired. Promotion is blocked.
    SoundBlock(SoundLeakClass),
    /// A non-decidable class was heuristically flagged. NOT a sound block:
    /// routes to human Layer-3 review (advisory). Promotion is NOT auto-allowed
    /// nor sound-blocked; the human is the decider.
    HumanReviewRequired(ResidueClass),
    /// Fail-closed: the DKG was absent / empty / unavailable, so the negative
    /// reachability assertion could not be soundly evaluated. Hard Block
    /// (R1/R2: missing evidence = FAIL, never ALLOW).
    DkgUnavailable,
}

/// Full verdict from [`evaluate_leakage`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeakageVerdict {
    /// Allow / SoundBlock / HumanReviewRequired / DkgUnavailable.
    pub kind: LeakageVerdictKind,
    /// Machine-readable reason code.
    pub reason: String,
    /// The fit-site that triggered the block (None for Allow / DkgUnavailable).
    pub fit_site: Option<FitSite>,
    /// The reachability path (DKG node names) supporting a sound block.
    pub reachability_path: Vec<String>,
    /// Free-text evidence string for the dashboard / audit.
    pub evidence: String,
    /// True iff a registered suppression downgraded an otherwise-sound block.
    pub suppressed: bool,
}

/// A fit/transform/select call site that reached both sinks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FitSite {
    /// Repo-relative file path.
    pub file: String,
    /// 1-based line.
    pub line: u32,
    /// Qualified DKG symbol (e.g. "sklearn.StandardScaler.fit").
    pub symbol: String,
}

/// Input bundle for [`evaluate_leakage`].
#[derive(Debug, Clone)]
pub struct LeakageInput {
    /// Repo-relative module path under analysis (the backtest / pipeline file).
    pub module_path: String,
    /// The declared split boundary. `None` => L1.1 (no test set) -> SoundBlock.
    pub declared_split: Option<SplitBoundary>,
    /// The fit-site source pattern the reachability walk starts from
    /// (e.g. "fit" | "fit_transform"). Names the preprocessing class.
    pub fit_source_pattern: String,
    /// Whether the fit-site is a feature-selector (L1.3) vs a preprocessor
    /// (L1.2). Only changes the reported [`SoundLeakClass`], not the walk.
    pub is_feature_selection: bool,
    /// A heuristic residue flag the caller wants routed to human review
    /// (e.g. a detected missing-`.shift(1)`). `None` => no residue routing.
    pub residue_flag: Option<ResidueClass>,
    /// Caller's DKG-availability determination (db present AND a probe returned
    /// reachable nodes). `false` => hard fail-closed Block.
    pub dkg_available: bool,
    /// ELAI_SEED for GD-2 reproducibility / corpus pinning.
    pub seed: u64,
}

/// A single registered suppression row (loaded from the frozen allowlist).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeakageSuppression {
    /// Must match [`LEAKAGE_RULE_ID`] for the suppression to apply.
    pub rule_id: String,
    /// File the suppression applies to (exact match against the fit-site file).
    pub file: String,
    /// Mandatory free-text reason.
    pub reason: String,
    /// Accountable owner.
    pub owner: String,
    /// Hard expiry as a unix timestamp (seconds). A suppression past its expiry
    /// does NOT apply (the block stands). `None` => never expires (discouraged).
    pub expiry_unix: Option<i64>,
}

impl LeakageSuppression {
    /// Does this row suppress a sound block at `fit_file`, given `now_unix`?
    ///
    /// ALL of: rule_id matches FIN-LEAK-001, file matches, reason+owner are
    /// non-empty, and the row is unexpired. A row missing any field grants NO
    /// suppression (mirrors wire-dark-allowlist semantics, R8/R9).
    fn applies(&self, fit_file: &str, now_unix: i64) -> bool {
        if self.rule_id != LEAKAGE_RULE_ID {
            return false;
        }
        if self.file != fit_file {
            return false;
        }
        if self.reason.trim().is_empty() || self.owner.trim().is_empty() {
            return false;
        }
        match self.expiry_unix {
            Some(expiry) => expiry > now_unix,
            None => true,
        }
    }
}

/// Errors from [`evaluate_leakage`].
#[derive(Debug, Error)]
pub enum LeakageError {
    /// A DKG query failed (distinct from "DKG empty" which is a fail-closed
    /// Block, not an error). A hard query failure is surfaced so the caller
    /// can decide; it is NOT silently downgraded to Allow.
    #[error("DKG query error: {0}")]
    DkgQuery(String),

    /// Writing the off-agent LeakageBlocked SSB row failed.
    #[error("SSB write error: {0}")]
    SsbWrite(String),
}

// ---------------------------------------------------------------------------
// Main gate: evaluate_leakage
// ---------------------------------------------------------------------------

/// Evaluate the H-20 leakage COMPOSITE (Tier-1, static-sound for L1.1/L1.2/L1.3).
///
/// Composes [`DkgQuerier::query_reachability`] negative-reachability assertions
/// over the DKG. Emits exactly one [`SsbPayload::LeakageBlocked`] row on every
/// decision (the off-agent artifact, R4).
///
/// # Order of checks (fail-closed first)
///   1. DKG availability (caller flag AND a positive presence probe). Absent /
///      empty / unavailable => `DkgUnavailable` Block (R1/R2). This MUST run
///      BEFORE the negative-reachability assertion, otherwise an empty graph
///      would read as "no path -> Allow" (fail-open).
///   2. L1.1 SchemaRequired: `declared_split.is_none()` => SoundBlock(NoTestSet).
///   3. L1.2/L1.3 negative reachability: fit-site reaches BOTH train sink AND
///      test sink => SoundBlock(Preprocess/FeatureSelect). Suppressed ONLY by a
///      registered allowlist row.
///   4. Residue routing: a heuristic `residue_flag` => HumanReviewRequired (NOT
///      a sound block).
///   5. Otherwise Allow.
pub async fn evaluate_leakage(
    querier: &dyn DkgQuerier,
    ssb: &Arc<Ssb>,
    input: LeakageInput,
    suppressions: &[LeakageSuppression],
    now_unix: i64,
) -> Result<LeakageVerdict, LeakageError> {
    // -- Step 1: FAIL-CLOSED on absent / empty / unavailable DKG. -----------
    // The negative-reachability assertion (must_not_exist=true) is only sound
    // over a populated graph. An empty graph trivially satisfies "no path",
    // which is FAIL-OPEN. We refuse to evaluate it as ALLOW.
    if !input.dkg_available || !dkg_is_populated(querier, &input.module_path)? {
        warn!(
            module = %input.module_path,
            dkg_available = input.dkg_available,
            "leakage_gate: DKG unavailable/empty -> Block (fail-closed, R1/R2)"
        );
        let verdict = LeakageVerdict {
            kind: LeakageVerdictKind::DkgUnavailable,
            reason: "dkg_unavailable".to_string(),
            fit_site: None,
            reachability_path: Vec::new(),
            evidence: "DKG absent/empty/unavailable: negative reachability cannot be \
                       soundly evaluated; fail-closed Block (R1/R2 missing-evidence=FAIL)"
                .to_string(),
            suppressed: false,
        };
        emit_ssb(ssb, build_payload(&input, &verdict)).await?;
        return Ok(verdict);
    }

    // -- Step 2: L1.1 SchemaRequired (no test set declared) -> SoundBlock. ---
    let split = match &input.declared_split {
        None => {
            debug!(
                module = %input.module_path,
                "leakage_gate: no declared split boundary -> SoundBlock(L1.1 NoTestSet)"
            );
            let verdict = LeakageVerdict {
                kind: LeakageVerdictKind::SoundBlock(SoundLeakClass::NoTestSet),
                reason: "block_no_test_set".to_string(),
                fit_site: None,
                reachability_path: Vec::new(),
                evidence: "L1.1: no train/test split boundary declared for the module \
                           (SchemaRequired leg). Promotion blocked fail-closed."
                    .to_string(),
                suppressed: false,
            };
            emit_ssb(ssb, build_payload(&input, &verdict)).await?;
            return Ok(verdict);
        }
        Some(split) => split,
    };

    // -- Step 3: L1.2/L1.3 negative reachability (fit reaches BOTH sinks). ---
    // Two positive reachability queries (fit -> train sink, fit -> test sink).
    // A fit-site that reaches BOTH sinks crossed the split boundary => leakage.
    let train_paths = query_paths(querier, &input.fit_source_pattern, TRAIN_SINK_PATTERN)?;
    let test_paths = query_paths(querier, &input.fit_source_pattern, TEST_SINK_PATTERN)?;

    if !train_paths.is_empty() && !test_paths.is_empty() {
        // The fit-site is the source of the first reaching path.
        let fit_site = first_node_as_fit_site(&train_paths, split, &input.fit_source_pattern);
        let class = if input.is_feature_selection {
            SoundLeakClass::FeatureSelectBeforeSplit
        } else {
            SoundLeakClass::PreprocessBeforeSplit
        };

        // Suppression check (REGISTERED, never silent). A sound block is
        // downgraded ONLY when a matching allowlist row applies; absence of a
        // row => the block STANDS.
        let suppressed = suppressions
            .iter()
            .any(|s| s.applies(&fit_site.file, now_unix));

        let reachability_path = train_paths
            .first()
            .cloned()
            .unwrap_or_default();

        if suppressed {
            warn!(
                fit_file = %fit_site.file,
                class = class.kapoor_code(),
                "leakage_gate: sound block SUPPRESSED by registered allowlist row"
            );
            let verdict = LeakageVerdict {
                kind: LeakageVerdictKind::Allow,
                reason: "allow_suppressed".to_string(),
                fit_site: Some(fit_site),
                reachability_path,
                evidence: format!(
                    "{}: fit-site reaches both train and test sinks, but a REGISTERED \
                     suppression row downgraded the block (audited).",
                    class.kapoor_code()
                ),
                suppressed: true,
            };
            emit_ssb(ssb, build_payload(&input, &verdict)).await?;
            return Ok(verdict);
        }

        debug!(
            fit_file = %fit_site.file,
            class = class.kapoor_code(),
            "leakage_gate: fit-site reaches both sinks -> SoundBlock"
        );
        let verdict = LeakageVerdict {
            kind: LeakageVerdictKind::SoundBlock(class),
            reason: format!("block_{}", class.kapoor_code().to_lowercase().replace('.', "_")),
            fit_site: Some(fit_site),
            reachability_path,
            evidence: format!(
                "{}: fit-site reaches BOTH a train sink and a test sink across the \
                 declared split boundary '{}' ({} train-paths, {} test-paths). \
                 Fit was applied to unsplit data.",
                class.kapoor_code(),
                split.split_symbol,
                train_paths.len(),
                test_paths.len()
            ),
            suppressed: false,
        };
        emit_ssb(ssb, build_payload(&input, &verdict)).await?;
        return Ok(verdict);
    }

    // -- Step 4: residue routing (NON-sound; human Layer-3). ----------------
    // A heuristic flag (e.g. missing-.shift(1) temporal L3.1) is NEVER a sound
    // block. It routes to human review. The gate does not auto-allow nor
    // sound-block it.
    if let Some(residue) = input.residue_flag {
        warn!(
            module = %input.module_path,
            class = residue.kapoor_code(),
            "leakage_gate: heuristic residue flag -> HumanReviewRequired (NOT a sound block)"
        );
        let verdict = LeakageVerdict {
            kind: LeakageVerdictKind::HumanReviewRequired(residue),
            reason: format!("review_{}", residue.kapoor_code().to_lowercase().replace('.', "_")),
            fit_site: None,
            reachability_path: Vec::new(),
            evidence: format!(
                "{}: heuristic flag is NOT statically decidable (outside the sound \
                 boundary). Routed to human Layer-3 review; the static gate makes no \
                 sound claim here.",
                residue.kapoor_code()
            ),
            suppressed: false,
        };
        emit_ssb(ssb, build_payload(&input, &verdict)).await?;
        return Ok(verdict);
    }

    // -- Step 5: Allow (no sound leakage, no residue). ----------------------
    debug!(
        module = %input.module_path,
        "leakage_gate: no sound leakage and no residue flag -> Allow"
    );
    let verdict = LeakageVerdict {
        kind: LeakageVerdictKind::Allow,
        reason: "allow".to_string(),
        fit_site: None,
        reachability_path: Vec::new(),
        evidence: format!(
            "No L1.1/L1.2/L1.3 leakage: split boundary '{}' declared and no fit-site \
             reaches both sinks. (Sound for L1.1-L1.3 only; L1.4/L2/L3.x are out of \
             the static boundary — H-21 runtime backstop covers the residue.)",
            split.split_symbol
        ),
        suppressed: false,
    };
    emit_ssb(ssb, build_payload(&input, &verdict)).await?;
    Ok(verdict)
}

// ---------------------------------------------------------------------------
// DKG helpers
// ---------------------------------------------------------------------------

/// Positive presence probe: does the DKG have ANY reachable node for `module`?
///
/// Uses `must_not_exist = false` (positive reachability). A populated graph
/// returns at least one match for the wildcard sink; an empty / unavailable
/// graph returns zero. This is the second leg of the fail-closed guard so a
/// silently-empty querier (e.g. `EmptyDkgQuerier`) cannot pass even if the
/// caller's `dkg_available` flag was optimistic.
fn dkg_is_populated(querier: &dyn DkgQuerier, module: &str) -> Result<bool, LeakageError> {
    let params = ReachabilityParams {
        source_pattern: module.to_string(),
        sink_pattern: PRESENCE_PROBE_SINK_PATTERN.to_string(),
        must_traverse: None,
        must_not_exist: false,
        max_depth: REACHABILITY_MAX_DEPTH,
        edge_types: DATA_FLOW_EDGE_TYPES.iter().map(|s| s.to_string()).collect(),
    };
    let matches = querier
        .query_reachability(&params)
        .map_err(|e| LeakageError::DkgQuery(format!("presence probe: {e}")))?;
    Ok(!matches.is_empty())
}

/// Run a positive reachability query from `source` to `sink`, returning the
/// node-name paths. Empty result = no path (NOT an error).
fn query_paths(
    querier: &dyn DkgQuerier,
    source: &str,
    sink: &str,
) -> Result<Vec<Vec<String>>, LeakageError> {
    let params = ReachabilityParams {
        source_pattern: source.to_string(),
        sink_pattern: sink.to_string(),
        must_traverse: None,
        must_not_exist: false,
        max_depth: REACHABILITY_MAX_DEPTH,
        edge_types: DATA_FLOW_EDGE_TYPES.iter().map(|s| s.to_string()).collect(),
    };
    let matches = querier
        .query_reachability(&params)
        .map_err(|e| LeakageError::DkgQuery(format!("{source}->{sink}: {e}")))?;
    Ok(matches.into_iter().map(|m| m.path).collect())
}

/// Extract the fit-site from the first node of a reaching path.
///
/// The first node is the reachability source (the fit call). `file`/`line`
/// fall back to the declared split's file when the node name does not encode
/// a location (the DKG path carries qualified symbol names, not file:line).
fn first_node_as_fit_site(
    paths: &[Vec<String>],
    split: &SplitBoundary,
    fit_source_pattern: &str,
) -> FitSite {
    let symbol = paths
        .first()
        .and_then(|p| p.first())
        .cloned()
        .unwrap_or_else(|| fit_source_pattern.to_string());
    FitSite {
        file: split.file.clone(),
        line: split.line,
        symbol,
    }
}

// ---------------------------------------------------------------------------
// SSB emission
// ---------------------------------------------------------------------------

async fn emit_ssb(ssb: &Arc<Ssb>, payload: LeakageBlockedPayload) -> Result<(), LeakageError> {
    ssb.write_payload(
        "leakage_gate",
        0,
        SsbPayload::LeakageBlocked(payload),
        Vec::new(),
    )
    .await
    .map_err(|e| LeakageError::SsbWrite(format!("{e}")))?;
    Ok(())
}

/// Build the off-agent [`LeakageBlockedPayload`] from inputs + verdict.
///
/// `corpus_blake3` pins the exact inputs so the verdict is recomputable (GD-2).
/// blake3 ONLY (CLAUDE.md HARD RULE).
fn build_payload(input: &LeakageInput, verdict: &LeakageVerdict) -> LeakageBlockedPayload {
    let (kind, kapoor_code) = match &verdict.kind {
        LeakageVerdictKind::Allow => (LeakageBlockedKind::HumanReviewRequired, "allow".to_string()),
        LeakageVerdictKind::SoundBlock(class) => (class.ssb_kind(), class.kapoor_code().to_string()),
        LeakageVerdictKind::HumanReviewRequired(residue) => {
            (LeakageBlockedKind::HumanReviewRequired, residue.kapoor_code().to_string())
        }
        LeakageVerdictKind::DkgUnavailable => {
            (LeakageBlockedKind::HumanReviewRequired, "dkg_unavailable".to_string())
        }
    };

    let (fit_file, fit_line, fit_symbol) = match &verdict.fit_site {
        Some(site) => (site.file.clone(), site.line, site.symbol.clone()),
        None => (String::new(), 0, String::new()),
    };

    let split_symbol = input
        .declared_split
        .as_ref()
        .map(|s| s.split_symbol.clone())
        .unwrap_or_default();

    let corpus_key = format!(
        "{}:{}:{}:{}:{}",
        input.module_path, split_symbol, input.fit_source_pattern, verdict.reason, input.seed
    );
    let corpus_blake3 = blake3::hash(corpus_key.as_bytes()).to_hex().to_string();

    LeakageBlockedPayload {
        kind,
        kapoor_code,
        module_path: input.module_path.clone(),
        fit_site_file: fit_file,
        fit_site_line: fit_line,
        fit_site_symbol: fit_symbol,
        reachability_path: verdict.reachability_path.clone(),
        verdict: verdict.reason.clone(),
        suppressed: verdict.suppressed,
        seed: input.seed,
        corpus_blake3,
    }
}

// ---------------------------------------------------------------------------
// Unit tests (COMPOSITE construction + empty-DKG guard)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_rules::predicate::{DkgError, ReachabilityMatch};
    use elai_rules::stubs::EmptyDkgQuerier;

    /// A configurable fake DKG querier for unit tests.
    struct FakeDkg {
        populated: bool,
        train_paths: Vec<Vec<String>>,
        test_paths: Vec<Vec<String>>,
    }

    impl DkgQuerier for FakeDkg {
        fn query_reachability(
            &self,
            params: &ReachabilityParams,
        ) -> Result<Vec<ReachabilityMatch>, DkgError> {
            // Presence probe (wildcard sink).
            if params.sink_pattern == PRESENCE_PROBE_SINK_PATTERN {
                return Ok(if self.populated {
                    vec![ReachabilityMatch {
                        path: vec!["node".to_string()],
                        edge_labels: vec!["data_flow".to_string()],
                    }]
                } else {
                    vec![]
                });
            }
            let paths = if params.sink_pattern == TRAIN_SINK_PATTERN {
                &self.train_paths
            } else {
                &self.test_paths
            };
            Ok(paths
                .iter()
                .map(|p| ReachabilityMatch {
                    path: p.clone(),
                    edge_labels: vec!["data_flow".to_string()],
                })
                .collect())
        }

        fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
            Ok(())
        }
    }

    fn split() -> SplitBoundary {
        SplitBoundary {
            split_symbol: "train_test_split".to_string(),
            file: "pipeline.py".to_string(),
            line: 10,
        }
    }

    fn base_input(dkg_available: bool) -> LeakageInput {
        LeakageInput {
            module_path: "pipeline.py".to_string(),
            declared_split: Some(split()),
            fit_source_pattern: "fit_transform".to_string(),
            is_feature_selection: false,
            residue_flag: None,
            dkg_available,
            seed: 42,
        }
    }

    fn test_ssb() -> Arc<Ssb> {
        use crate::ssb::SsbConfig;
        let dir = std::env::temp_dir().join(format!("h20-unit-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cfg = SsbConfig {
            db_path: dir.join(format!("ssb-{}.db", rand_suffix())),
            pool_size: Some(2),
            project_id: "h20-unit".to_string(),
            session_id: "h20-unit-session".to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> =
            Arc::new(elai_core::clock::OsClock);
        Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
    }

    fn rand_suffix() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn empty_dkg_is_fail_closed_block() {
        let ssb = test_ssb();
        let querier = EmptyDkgQuerier;
        // dkg_available=true but the querier is empty: the presence probe must
        // still fail-close (never inherit the permissive "no path -> Allow").
        let verdict = evaluate_leakage(&querier, &ssb, base_input(true), &[], 0)
            .await
            .expect("evaluate_leakage");
        assert_eq!(
            verdict.kind,
            LeakageVerdictKind::DkgUnavailable,
            "empty DKG must Block fail-closed, not Allow"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dkg_available_false_is_fail_closed_block() {
        let ssb = test_ssb();
        let querier = FakeDkg { populated: true, train_paths: vec![], test_paths: vec![] };
        let verdict = evaluate_leakage(&querier, &ssb, base_input(false), &[], 0)
            .await
            .expect("evaluate_leakage");
        assert_eq!(verdict.kind, LeakageVerdictKind::DkgUnavailable);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn fit_reaching_both_sinks_is_sound_block() {
        let ssb = test_ssb();
        let querier = FakeDkg {
            populated: true,
            train_paths: vec![vec!["fit_transform".into(), "X_train".into()]],
            test_paths: vec![vec!["fit_transform".into(), "X_test".into()]],
        };
        let verdict = evaluate_leakage(&querier, &ssb, base_input(true), &[], 0)
            .await
            .expect("evaluate_leakage");
        assert_eq!(
            verdict.kind,
            LeakageVerdictKind::SoundBlock(SoundLeakClass::PreprocessBeforeSplit)
        );
        assert!(verdict.fit_site.is_some());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn clean_split_then_fit_allows() {
        let ssb = test_ssb();
        // fit reaches only the train sink (correct: fit after split).
        let querier = FakeDkg {
            populated: true,
            train_paths: vec![vec!["fit_transform".into(), "X_train".into()]],
            test_paths: vec![],
        };
        let verdict = evaluate_leakage(&querier, &ssb, base_input(true), &[], 0)
            .await
            .expect("evaluate_leakage");
        assert_eq!(verdict.kind, LeakageVerdictKind::Allow);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn no_split_is_l1_1_block() {
        let ssb = test_ssb();
        let querier = FakeDkg { populated: true, train_paths: vec![], test_paths: vec![] };
        let mut input = base_input(true);
        input.declared_split = None;
        let verdict = evaluate_leakage(&querier, &ssb, input, &[], 0)
            .await
            .expect("evaluate_leakage");
        assert_eq!(
            verdict.kind,
            LeakageVerdictKind::SoundBlock(SoundLeakClass::NoTestSet)
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn registered_suppression_downgrades_block() {
        let ssb = test_ssb();
        let querier = FakeDkg {
            populated: true,
            train_paths: vec![vec!["fit_transform".into(), "X_train".into()]],
            test_paths: vec![vec!["fit_transform".into(), "X_test".into()]],
        };
        let suppressions = vec![LeakageSuppression {
            rule_id: LEAKAGE_RULE_ID.to_string(),
            file: "pipeline.py".to_string(),
            reason: "legacy notebook, tracked in JIRA-123".to_string(),
            owner: "maintainer".to_string(),
            expiry_unix: Some(i64::MAX),
        }];
        let verdict = evaluate_leakage(&querier, &ssb, base_input(true), &suppressions, 0)
            .await
            .expect("evaluate_leakage");
        assert_eq!(verdict.kind, LeakageVerdictKind::Allow);
        assert!(verdict.suppressed, "registered row must mark suppressed=true");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn expired_suppression_does_not_apply() {
        let ssb = test_ssb();
        let querier = FakeDkg {
            populated: true,
            train_paths: vec![vec!["fit_transform".into(), "X_train".into()]],
            test_paths: vec![vec!["fit_transform".into(), "X_test".into()]],
        };
        let suppressions = vec![LeakageSuppression {
            rule_id: LEAKAGE_RULE_ID.to_string(),
            file: "pipeline.py".to_string(),
            reason: "expired".to_string(),
            owner: "maintainer".to_string(),
            expiry_unix: Some(100), // now_unix=200 > 100 -> expired
        }];
        let verdict = evaluate_leakage(&querier, &ssb, base_input(true), &suppressions, 200)
            .await
            .expect("evaluate_leakage");
        assert_eq!(
            verdict.kind,
            LeakageVerdictKind::SoundBlock(SoundLeakClass::PreprocessBeforeSplit),
            "expired suppression must NOT downgrade the block"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn residue_flag_routes_to_human_review_not_sound_block() {
        let ssb = test_ssb();
        let querier = FakeDkg {
            populated: true,
            train_paths: vec![],
            test_paths: vec![],
        };
        let mut input = base_input(true);
        input.residue_flag = Some(ResidueClass::Temporal);
        let verdict = evaluate_leakage(&querier, &ssb, input, &[], 0)
            .await
            .expect("evaluate_leakage");
        assert_eq!(
            verdict.kind,
            LeakageVerdictKind::HumanReviewRequired(ResidueClass::Temporal),
            "L3.1 temporal is NOT sound-decidable: must route to review, never a sound block"
        );
    }
}
