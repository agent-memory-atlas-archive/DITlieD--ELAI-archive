// I-36: per-(role, backend) compiled-prompt artifact store.
//
// SYNOPSIS
// --------
// After an offline optimizer (GEPA / MIPROv2-class) runs on the cloud lane,
// it produces a compiled variant — a plain-string replacement for the assembled
// system prompt for a given (role, backend) pair. This module stores, versions,
// and looks up those variants. The substitution happens in the prompt-routing
// layer (providers::anthropic_api::complete) AFTER I-32 role_contracts apply.
//
// TIER-1 (ship now):
//   - CompiledPromptVariant (the artifact)
//   - CompiledPromptStore   (BTreeMap + msgpack + atomic-write, mirrors C-25)
//   - lookup()              (production seam: called from report_fns)
//   - rollback_on_regression()
//
// TIER-2 (DEFER behind ELAI_PROMPT_COMPILE_DISABLE):
//   - promote_variant()     — WIRE-DARK[I-36]: requires Tier-2 ledger threading
//   - run_transfer_gap_fixture() — WIRE-DARK[I-36]: requires live-model budget
//
// CONTRACT INVARIANTS (stub_ban):
//   - No .unwrap()/.expect() on production paths (only in #[cfg(test)] bodies)
//   - blake3 only (NEVER blake2/sha2) for the variant hash
//   - No CoT scaffold under extended thinking in compiled strings (enforced by
//     has_cot_scaffold(): the deliberative-hygiene grep)
//   - No local-lane self-optimization (landmine #12); optimizer is cloud-lane only
//   - No promotion bypassing ID-6 ledger gates (promote_variant is WIRE-DARK[I-36])
//
// STORAGE: mirrors CompiledPipelineStore (compiled_pipeline.rs:669) exactly —
//   BTreeMap (B-15 HashMap ban), rmp_serde msgpack, atomic tempfile+rename,
//   standard_path under project_root/.elai/cache/, load-on-corrupt-returns-empty.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ssb::{PromptCompiledKind, PromptCompiledPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// Constants — named, no magic numbers.
// ---------------------------------------------------------------------------

const COMPILED_PROMPT_CACHE_DIR: &str = ".elai/cache";
const COMPILED_PROMPT_CACHE_FILE: &str = "compiled_prompts.msgpack";

/// A compiled prompt variant for one (role, backend) pair.
///
/// Produced offline by the cloud-lane optimizer (GEPA / MIPROv2-class); stored
/// as a plain-string artifact. Call sites do NOT refactor; the store swaps the
/// assembled system-prompt string at the adapter boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledPromptVariant {
    /// Snake-case role name (e.g. `"executor"`, `"researcher"`).
    pub role: String,
    /// Backend name (e.g. `"anthropic_api"`, `"sidecar"`).
    pub backend: String,
    /// The compiled system-prompt string — the plain-text replacement.
    ///
    /// DELIBERATIVE-HYGIENE CONTRACT (stub_ban): this string MUST NOT contain
    /// an explicit step-by-step CoT scaffold when the variant is intended for
    /// extended-thinking roles (landmine #6: Opus 4.7+ auto-interleaves thinking;
    /// explicit scaffolds are redundant and may suppress native reasoning). Use
    /// `has_cot_scaffold()` to verify before storing.
    pub compiled_text: String,
    /// blake3 hex digest (full 64-char) of `compiled_text`. NEVER blake2/sha2.
    pub variant_hash: String,
    /// ID-6 promotion ledger gate pass count at store time.
    pub ledger_pass_count: u32,
    /// Fixture pass-rate of the compiled variant on the target backend (GD-2,
    /// recomputable under ELAI_SEED). `None` when not yet measured (Tier-2).
    #[serde(default)]
    pub fixture_pass_rate: Option<f32>,
    /// Source pass-rate (the base variant on the same backend) at store time.
    /// `None` when not yet measured.
    #[serde(default)]
    pub source_pass_rate: Option<f32>,
    /// Unix ms timestamp when this variant was promoted.
    pub promoted_at_ms: i64,
    /// Last time this variant was returned from `lookup()`. Not serialized for
    /// cache-hit logic: updated in memory only (eviction tracks `promoted_at_ms`).
    #[serde(skip)]
    pub last_used_at_ms: i64,
}

impl CompiledPromptVariant {
    /// Compute the canonical blake3 hex digest for a compiled text string.
    ///
    /// blake3 ONLY. Anti-stale-training hard rule (CLAUDE.md).
    #[must_use]
    pub fn hash_text(text: &str) -> String {
        let mut h = Hasher::new();
        h.update(text.as_bytes());
        h.finalize().to_hex().to_string()
    }

    /// Deliberative-hygiene grep: returns `true` if `text` contains an
    /// explicit step-by-step CoT scaffold that would be redundant / harmful
    /// under extended thinking (landmine #6, I-0).
    ///
    /// The check is a static string scan — intentionally conservative.
    /// A `true` result from this function means the compiled text violates
    /// the stub_ban contract and MUST NOT be stored.
    #[must_use]
    pub fn has_cot_scaffold(text: &str) -> bool {
        let lower = text.to_ascii_lowercase();
        // Explicit numbered step patterns that indicate a CoT scaffold.
        let markers = [
            "step 1:",
            "step 2:",
            "step 3:",
            "step 4:",
            "step 5:",
            "1. think",
            "1. reason",
            "first, think through",
            "let's think step by step",
            "think step by step",
            "let me reason step by step",
            "<thinking>",
        ];
        markers.iter().any(|m| lower.contains(m))
    }

    /// BTreeMap key for this variant.
    #[must_use]
    pub fn store_key(&self) -> (String, String) {
        (self.role.clone(), self.backend.clone())
    }
}

/// Errors from [`CompiledPromptStore`] operations.
#[derive(Debug, Error)]
pub enum CompiledPromptStoreError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("msgpack encode error: {0}")]
    Encode(#[from] rmp_serde::encode::Error),
    #[error("msgpack decode error: {0}")]
    Decode(#[from] rmp_serde::decode::Error),
    #[error("cot scaffold detected in compiled text for role={role} backend={backend}")]
    CotScaffoldDetected { role: String, backend: String },
    #[error("variant hash mismatch: stored={stored} recomputed={recomputed}")]
    HashMismatch { stored: String, recomputed: String },
}

/// Per-(role, backend) compiled-prompt artifact store.
///
/// Mirrors [`CompiledPipelineStore`](crate::compiled_pipeline::CompiledPipelineStore)
/// exactly: BTreeMap (B-15 HashMap ban), rmp_serde msgpack encoding, atomic
/// tempfile+rename persistence, load-on-corrupt-returns-empty (NEVER panic).
///
/// The store is DEFAULT-ON: `lookup()` returns a compiled variant when one
/// has been promoted for the (role, backend) pair. Kill-switch
/// `ELAI_PROMPT_COMPILE_DISABLE` suppresses all substitutions (operations
/// become no-ops; store is not loaded from disk). See allowlist row.
#[derive(Debug)]
pub struct CompiledPromptStore {
    cache_path: PathBuf,
    entries: BTreeMap<(String, String), CompiledPromptVariant>,
}

impl CompiledPromptStore {
    /// Construct a fresh in-memory store. Does NOT touch disk; call
    /// `load_from_disk()` after construction to populate from the cache file.
    #[must_use]
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            cache_path,
            entries: BTreeMap::new(),
        }
    }

    /// Resolve the standard cache path under `project_root/.elai/cache/compiled_prompts.msgpack`.
    #[must_use]
    pub fn standard_path(project_root: &Path) -> PathBuf {
        project_root
            .join(COMPILED_PROMPT_CACHE_DIR)
            .join(COMPILED_PROMPT_CACHE_FILE)
    }

    /// Read-only accessor for tests / diagnostics.
    #[must_use]
    pub fn entries(&self) -> &BTreeMap<(String, String), CompiledPromptVariant> {
        &self.entries
    }

    /// Return true if the kill-switch `ELAI_PROMPT_COMPILE_DISABLE` is set to `"1"`.
    ///
    /// When true, all lookup/substitution operations are no-ops and the store
    /// is not loaded from disk. The kill-switch covers Tier-1 substitution (pass-through)
    /// and Tier-2 optimizer runs equally. See wire-dark-allowlist.toml.
    #[must_use]
    pub fn kill_switch_active() -> bool {
        std::env::var("ELAI_PROMPT_COMPILE_DISABLE")
            .map(|v| v == "1")
            .unwrap_or(false)
    }

    /// Construct and load from the standard path under `project_root`.
    ///
    /// Returns an empty store if the kill-switch is active. On any decode
    /// error, emits a tracing warn and returns empty (NEVER panic, stub_ban).
    #[must_use]
    pub fn load_standard(project_root: &Path) -> Self {
        if Self::kill_switch_active() {
            tracing::debug!(
                target = "compiled_prompt_store",
                "ELAI_PROMPT_COMPILE_DISABLE=1; prompt-variant substitution disabled"
            );
            return Self::new(Self::standard_path(project_root));
        }
        let path = Self::standard_path(project_root);
        let mut store = Self::new(path);
        store.load_from_disk();
        store
    }

    /// Load entries from disk. On any decode error, emits a warning and
    /// uses an empty store (NEVER panic). Missing file is treated as empty.
    pub fn load_from_disk(&mut self) {
        if !self.cache_path.exists() {
            return;
        }
        let bytes = match fs::read(&self.cache_path) {
            Ok(b) => b,
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    path = %self.cache_path.display(),
                    "compiled_prompt_store: read failed; using empty store"
                );
                self.entries.clear();
                return;
            }
        };
        let decoded: Result<BTreeMap<(String, String), CompiledPromptVariant>, _> =
            rmp_serde::from_slice(&bytes);
        match decoded {
            Ok(map) => {
                self.entries = map;
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    path = %self.cache_path.display(),
                    "compiled_prompt_store: corrupt msgpack; using empty store"
                );
                self.entries.clear();
            }
        }
    }

    /// Persist entries to disk atomically (tempfile + rename).
    /// Creates parent directories if absent.
    pub fn persist_to_disk(&self) -> Result<(), CompiledPromptStoreError> {
        if let Some(parent) = self.cache_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let bytes = rmp_serde::to_vec_named(&self.entries)?;
        let tmp_path = self.cache_path.with_extension("msgpack.tmp");
        fs::write(&tmp_path, &bytes)?;
        fs::rename(&tmp_path, &self.cache_path)?;
        Ok(())
    }

    /// Look up a promoted compiled variant for `(role, backend)`.
    ///
    /// Returns `None` when:
    ///   - the kill-switch is active
    ///   - no promoted variant exists for this (role, backend) pair
    ///   - the stored variant hash does not match the compiled text (integrity check)
    ///
    /// This is the production seam called from `report_fns::build_system_prompt_for_role`
    /// (after I-32 role_contracts apply) and from `anthropic_api::complete` (via
    /// CompletionConfig::compiled_system_override).
    #[must_use]
    pub fn lookup(&self, role: &str, backend: &str) -> Option<&CompiledPromptVariant> {
        if Self::kill_switch_active() {
            return None;
        }
        let key = (role.to_owned(), backend.to_owned());
        let variant = self.entries.get(&key)?;
        // Integrity check: recompute hash to detect silent corruption.
        let recomputed = CompiledPromptVariant::hash_text(&variant.compiled_text);
        if recomputed != variant.variant_hash {
            tracing::warn!(
                role = role,
                backend = backend,
                stored_hash = %variant.variant_hash,
                recomputed_hash = %recomputed,
                "compiled_prompt_store: hash mismatch; dropping corrupt variant"
            );
            return None;
        }
        Some(variant)
    }

    /// Insert a new promoted variant into the store (in memory only).
    /// Call `persist_to_disk()` after to save.
    ///
    /// Returns an error if the CoT scaffold check fails (stub_ban) or if the
    /// provided `variant_hash` does not match the compiled text.
    pub fn insert(
        &mut self,
        variant: CompiledPromptVariant,
    ) -> Result<(), CompiledPromptStoreError> {
        // Deliberative-hygiene: no CoT scaffold under extended thinking.
        if CompiledPromptVariant::has_cot_scaffold(&variant.compiled_text) {
            return Err(CompiledPromptStoreError::CotScaffoldDetected {
                role: variant.role.clone(),
                backend: variant.backend.clone(),
            });
        }
        // Hash integrity.
        let recomputed = CompiledPromptVariant::hash_text(&variant.compiled_text);
        if recomputed != variant.variant_hash {
            return Err(CompiledPromptStoreError::HashMismatch {
                stored: variant.variant_hash.clone(),
                recomputed,
            });
        }
        let key = variant.store_key();
        self.entries.insert(key, variant);
        Ok(())
    }

    /// Remove the variant for `(role, backend)` from the in-memory store.
    /// Callers should call `persist_to_disk()` after.
    pub fn remove(&mut self, role: &str, backend: &str) {
        let key = (role.to_owned(), backend.to_owned());
        self.entries.remove(&key);
    }

    /// Return a summary of all stored (role, backend) pairs and their variant hashes.
    /// Used by the Tauri badge command.
    ///
    /// The Tauri badge (commands_i36.rs::get_compiled_variants) reads SSB rows
    /// directly via the SsbViewer rather than calling this method (so the badge
    /// works even after a restart with no in-memory store). This method is the
    /// complement for callers that already have the in-memory store loaded and
    /// want the lightweight status vec without a DB round-trip.
    // WIRE-DARK[I-36]: direct-store accessor for `elai compiled-status` CLI (I-36-compiled-status-cli). SSB surface wired. See stubs.md row I-36-variant-status.
    #[must_use]
    pub fn variant_status(&self) -> Vec<VariantStatus> {
        self.entries
            .iter()
            .map(|((role, backend), v)| VariantStatus {
                role: role.clone(),
                backend: backend.clone(),
                variant_hash: v.variant_hash[..16.min(v.variant_hash.len())].to_owned(),
                fixture_pass_rate: v.fixture_pass_rate,
                promoted_at_ms: v.promoted_at_ms,
            })
            .collect()
    }
}

/// Summary of one stored compiled variant (returned to the dashboard).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VariantStatus {
    pub role: String,
    pub backend: String,
    /// First 16 hex chars of the blake3 hash.
    pub variant_hash: String,
    pub fixture_pass_rate: Option<f32>,
    pub promoted_at_ms: i64,
}

// ---------------------------------------------------------------------------
// promote_variant — WIRE-DARK[I-36]
// ---------------------------------------------------------------------------
// Tier-2: routing through darwin two_gate_promote requires threading
// CompiledPromptVariant into the PromotionConfig/DescriptorEntry bridge,
// which is a non-trivial structural change. Deferred to I-36-promotion.
// The function signature is published for contract-test discoverability;
// the body returns an error until the Tier-2 bridge is built.

/// Attempt to promote a candidate compiled variant through the ID-6 ledger gates.
///
/// # WIRE-DARK[I-36]
///
/// The full darwin two_gate_promote bridge is Tier-2 DEFER. This function
/// currently returns an error with instructions. When I-36-promotion lands,
/// the body is replaced with the real ledger routing (see stubs.md row I-36-promote).
///
/// For Tier-1 use `CompiledPromptStore::insert()` directly after manual verification.
// WIRE-DARK[I-36]: Tier-2 darwin two_gate_promote bridge deferred to I-36-promotion.
pub fn promote_variant(
    _candidate: &CompiledPromptVariant,
    _ssb: Option<&std::sync::Arc<Ssb>>,
) -> Result<(), PromoteError> {
    Err(PromoteError::Tier2Deferred)
}

/// Errors from [`promote_variant`].
#[derive(Debug, Error)]
pub enum PromoteError {
    #[error(
        "promote_variant is WIRE-DARK[I-36] (Tier-2 DEFER). \
         Use CompiledPromptStore::insert() for manual Tier-1 promotion. \
         See workflow/sessions/s-107b022a/stubs.md row I-36-promote."
    )]
    Tier2Deferred,
    #[error("cot scaffold detected; variant rejected by deliberative-hygiene check")]
    CotScaffoldRejected,
}

// ---------------------------------------------------------------------------
// rollback_on_regression
// ---------------------------------------------------------------------------

/// Compare `new_pass_rate` against the stored `source_pass_rate` for `(role, backend)`.
///
/// If the compiled variant regresses the fixture floor (new_pass_rate < source_pass_rate),
/// remove the variant from the store, persist, and return `RollbackOutcome::Rolled`.
/// Otherwise return `RollbackOutcome::Retained`. Missing variants return `RollbackOutcome::NoVariant`.
///
/// Per the WIRING_CONTRACT: emits a `PromptCompiled(Rolledback)` SSB row when a regression is
/// detected and `ssb` is `Some`. The SSB emit is best-effort (log on failure, never propagate).
pub fn rollback_on_regression(
    store: &mut CompiledPromptStore,
    role: &str,
    backend: &str,
    new_pass_rate: f32,
    ssb: Option<&std::sync::Arc<Ssb>>,
    ts_ms: i64,
) -> RollbackOutcome {
    let source_rate = match store.lookup(role, backend) {
        None => return RollbackOutcome::NoVariant,
        Some(v) => v.source_pass_rate.unwrap_or(0.0),
    };

    if new_pass_rate < source_rate {
        let variant_hash = store
            .lookup(role, backend)
            .map(|v| v.variant_hash.clone())
            .unwrap_or_default();
        store.remove(role, backend);
        if let Err(e) = store.persist_to_disk() {
            tracing::warn!(
                error = %e,
                "compiled_prompt_store: rollback persist failed"
            );
        }
        if let Some(s) = ssb {
            let payload = SsbPayload::PromptCompiled(PromptCompiledPayload {
                kind: PromptCompiledKind::Rolledback,
                role: role.to_owned(),
                backend: backend.to_owned(),
                variant_hash,
                fixture_delta: new_pass_rate - source_rate,
                ts_ms,
            });
            // Best-effort SSB emit: rollback_on_regression is sync; if we are
            // inside a tokio runtime, spawn a detached task for the async write.
            // If no runtime is available (unit tests, bench contexts) skip silently.
            let ssb_arc = std::sync::Arc::clone(s);
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    if let Err(e) = ssb_arc
                        .write_payload("compiled_prompt_store", 0, payload, Vec::new())
                        .await
                    {
                        tracing::warn!(
                            error = %e,
                            "compiled_prompt_store: SSB Rolledback emit failed"
                        );
                    }
                });
            }
        }
        RollbackOutcome::Rolled {
            source_rate,
            new_pass_rate,
        }
    } else {
        RollbackOutcome::Retained
    }
}

/// Outcome of [`rollback_on_regression`].
#[derive(Debug, Clone, PartialEq)]
pub enum RollbackOutcome {
    /// No variant was stored for this (role, backend).
    NoVariant,
    /// Variant was retained (new_pass_rate >= source_pass_rate).
    Retained,
    /// Variant was removed due to regression.
    Rolled {
        source_rate: f32,
        new_pass_rate: f32,
    },
}

// ---------------------------------------------------------------------------
// Transfer-gap measurement harness stub — WIRE-DARK[I-36]
// ---------------------------------------------------------------------------

/// Result of a source-vs-compiled fixture comparison (GD-2, recomputable under ELAI_SEED).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferGapResult {
    pub role: String,
    pub backend: String,
    pub variant_hash: String,
    /// Fixture pass-rate with the source (uncompiled) prompt on the target backend.
    pub source_pass_rate: f32,
    /// Fixture pass-rate with the compiled variant on the target backend.
    pub compiled_pass_rate: f32,
    /// compiled_pass_rate - source_pass_rate (positive = improvement).
    pub delta: f32,
    /// ELAI_SEED value used for this measurement run.
    pub seed: u64,
}

/// Run a transfer-gap fixture comparison for one (role, backend) pair.
///
/// # WIRE-DARK[I-36]
///
/// Live model inference is budget-gated (Tier-2). This function returns
/// `None` unconditionally until I-36-promotion wires the live measurement
/// harness. See stubs.md row I-36-transfer-gap.
// WIRE-DARK[I-36]: live model fixture runner deferred to I-36-promotion.
pub fn run_transfer_gap_fixture(
    _role: &str,
    _backend: &str,
    _variant: &CompiledPromptVariant,
    _seed: u64,
) -> Option<TransferGapResult> {
    // WIRE-DARK[I-36]: live model fixture runner deferred to I-36-promotion.
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_variant(role: &str, backend: &str, text: &str) -> CompiledPromptVariant {
        let hash = CompiledPromptVariant::hash_text(text);
        CompiledPromptVariant {
            role: role.to_owned(),
            backend: backend.to_owned(),
            compiled_text: text.to_owned(),
            variant_hash: hash,
            ledger_pass_count: 1,
            fixture_pass_rate: Some(0.9),
            source_pass_rate: Some(0.8),
            promoted_at_ms: 1_700_000_000_000,
            last_used_at_ms: 0,
        }
    }

    #[test]
    fn compiled_prompt_store_round_trip_msgpack() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.msgpack");
        let mut store = CompiledPromptStore::new(path.clone());

        let v = make_variant("executor", "anthropic_api", "# Role: Executor\nConstraints:\n- focus on the task");
        store.insert(v.clone()).unwrap();
        store.persist_to_disk().unwrap();

        let mut store2 = CompiledPromptStore::new(path);
        store2.load_from_disk();
        let found = store2.lookup("executor", "anthropic_api").unwrap();
        assert_eq!(found.role, "executor");
        assert_eq!(found.backend, "anthropic_api");
        assert_eq!(found.variant_hash, v.variant_hash);
    }

    #[test]
    fn cot_scaffold_detection_blocks_insert() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.msgpack");
        let mut store = CompiledPromptStore::new(path);

        let text = "Think step by step: Step 1: Plan. Step 2: Execute.";
        let hash = CompiledPromptVariant::hash_text(text);
        let v = CompiledPromptVariant {
            role: "executor".to_owned(),
            backend: "anthropic_api".to_owned(),
            compiled_text: text.to_owned(),
            variant_hash: hash,
            ledger_pass_count: 1,
            fixture_pass_rate: None,
            source_pass_rate: None,
            promoted_at_ms: 0,
            last_used_at_ms: 0,
        };
        let err = store.insert(v).unwrap_err();
        assert!(matches!(err, CompiledPromptStoreError::CotScaffoldDetected { .. }));
    }

    #[test]
    fn hash_mismatch_blocks_lookup() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.msgpack");
        let mut store = CompiledPromptStore::new(path.clone());

        let text = "# Role: Researcher\nfocus only";
        let mut v = make_variant("researcher", "sidecar", text);
        // Corrupt the hash to simulate disk corruption.
        v.variant_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_owned();
        // Force-insert bypassing the integrity check (simulate corrupt on-disk load).
        store.entries.insert(v.store_key(), v);

        let result = store.lookup("researcher", "sidecar");
        assert!(result.is_none(), "corrupt variant must return None from lookup");
    }

    #[test]
    fn rollback_removes_regressed_variant() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.msgpack");
        let mut store = CompiledPromptStore::new(path);

        let v = make_variant("executor", "anthropic_api", "# Role: Executor\nConstraints:\n- deterministic");
        store.insert(v).unwrap();

        // new_pass_rate (0.6) < source_pass_rate (0.8) => regression.
        let outcome = rollback_on_regression(&mut store, "executor", "anthropic_api", 0.6, None, 0);
        assert!(matches!(outcome, RollbackOutcome::Rolled { source_rate, new_pass_rate }
            if (source_rate - 0.8).abs() < 1e-4 && (new_pass_rate - 0.6).abs() < 1e-4));
        assert!(store.lookup("executor", "anthropic_api").is_none(), "rolled back variant must be absent");
    }

    #[test]
    fn kill_switch_suppresses_lookup() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.msgpack");
        let mut store = CompiledPromptStore::new(path);
        let v = make_variant("executor", "anthropic_api", "# Role: Executor\n");
        store.insert(v).unwrap();

        std::env::set_var("ELAI_PROMPT_COMPILE_DISABLE", "1");
        let result = store.lookup("executor", "anthropic_api");
        std::env::remove_var("ELAI_PROMPT_COMPILE_DISABLE");
        assert!(result.is_none(), "kill-switch must suppress lookup");
    }

    #[test]
    fn has_cot_scaffold_detects_patterns() {
        assert!(CompiledPromptVariant::has_cot_scaffold("Let's think step by step"));
        assert!(CompiledPromptVariant::has_cot_scaffold("Step 1: do the thing"));
        assert!(CompiledPromptVariant::has_cot_scaffold("<thinking>some reasoning</thinking>"));
        assert!(!CompiledPromptVariant::has_cot_scaffold("# Role: Executor\nConstraints:\n- focus"));
    }
}
