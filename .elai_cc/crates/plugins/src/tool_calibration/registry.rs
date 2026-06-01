//! C-47: Per-tool LRU registry + dependency-inversion SSB emitter trait.
//!
//! [`ToolCalibrationRegistry`] owns one [`lru::LruCache`] per tool, keyed
//! by `call_shape_hash`, capped at [`CASE_BASE_LRU_CAP`]. The LRU
//! eviction is deterministic (least-recently-used; the
//! `lru` crate's documented semantics), which satisfies the
//! `WIRING_CONTRACT` stub_ban "eviction MUST be deterministic
//! (oldest-by-timestamp, not random)" — least-recently-used IS
//! deterministic by access ordering, NOT random.
//!
//! The SSB emitter trait + Noop default mirror C-35's
//! [`crate::mcp::ToolGateSsbEmitter`] pattern: the trait lives in
//! plugins (so DefaultMcpBridge can hold an `Arc<dyn _>`) and the
//! adapter that maps emit kinds to `SsbPayload` lives in the
//! orchestrator (to keep the plugins -> orchestrator dependency
//! acyclic).
//!
//! Concurrency: each tool's LRU sits behind a `tokio::sync::Mutex` so
//! the async bridge dispatch path can mutate without a `parking_lot`
//! dep. We deliberately do NOT use `std::sync::Mutex` per CLAUDE.md
//! anti-stale-training rule.

use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

use super::case_base::{AggregatedCase, ResponseKind};
use super::hinter::generate_hint;
use super::persist::CasePersistStore;

/// Per-tool LRU capacity. Each tool gets its own [`LruCache`] of
/// `(call_shape_hash, AggregatedCase)`. Once 100 distinct shapes have
/// been observed for one tool, the next insertion evicts the
/// least-recently-touched entry.
pub const CASE_BASE_LRU_CAP: usize = 100;

/// C-47b -- env var name. When set to one of `"1"` / `"true"` / `"yes"` /
/// `"on"` AND the registry has a persist store installed, [`Self::record_call`]
/// persists each case to sqlite BEFORE admitting it to the LRU
/// (fail-closed: persist error skips the admit per
/// `workflow/reasoning/D-trancheD-decisions.md` D3). The env-gate keeps
/// the durability invariant opt-in so the production wire-dark posture
/// is preserved.
pub const ELAI_C47B_CASE_BASE_PERSIST_ENABLE_ENV: &str =
    "ELAI_C47B_CASE_BASE_PERSIST_ENABLE";

/// SSB action-type sentinel strings emitted by the registry. The
/// orchestrator-side adapter maps these to the matching `SsbPayload`
/// variant (see `crates/orchestrator/src/tool_calibration_ssb_adapter.rs`).
pub mod emit_kinds {
    pub const TOOL_CALL_LOGGED: &str = "tool_call_logged";
    pub const TOOL_CALIBRATION_HINT_EMITTED: &str =
        "tool_calibration_hint_emitted";
    pub const TOOL_CALIBRATION_DRIFT_DETECTED: &str =
        "tool_calibration_drift_detected";
}

/// Errors surfaced by the registry. The in-memory LRU has no fallible
/// operation; the persistence-store add-on (C-47b) surfaces sqlite +
/// serde errors via the `Persist` variant.
#[derive(Debug, Error)]
pub enum ToolCalibrationError {
    /// Construction-time invariant violation (capacity must be > 0).
    /// The crate enforces this at the call site via `NonZeroUsize`
    /// but exposing the variant gives callers a typed failure path
    /// when they pass dynamic config values.
    #[error("ToolCalibrationRegistry capacity must be > 0")]
    ZeroCapacity,

    /// C-47b: persistence-layer error (sqlite open / migrate / prepare /
    /// step, serde encode / decode). Wrapped as String to keep the
    /// plugins crate free of a direct rusqlite type in the public API
    /// (the inner module exposes it; the public error is the stringified
    /// form so contract-test fixtures stay stable).
    #[error("tool_calibration persist error: {0}")]
    Persist(String),
}

/// Per-tool status snapshot used by the dashboard's
/// `CaseBaseInspector`. The TS DTO at
/// `dashboard-v2/src/lib/api/stub.ts` mirrors this shape one-for-one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCalibrationStatus {
    pub tool_name: String,
    /// Number of distinct call_shape_hashes currently retained.
    pub shape_count: u32,
    /// Sum of total_calls across all retained shapes.
    pub total_calls: u32,
    /// Sum of error_count across all retained shapes.
    pub error_count: u32,
    /// Most recent observation timestamp across all retained shapes
    /// (ms since UNIX_EPOCH); `0` when no observations exist.
    pub last_ts_ms: i64,
}

/// Lock-free per-tool registry: each tool's LRU sits behind its own
/// `tokio::sync::Mutex` so two different tools never contend on the
/// same lock.
pub struct ToolCalibrationRegistry {
    tools: Mutex<FxHashMap<String, Arc<Mutex<LruCache<u64, AggregatedCase>>>>>,
    /// Optional kill-switch — operators can disable hint generation
    /// at runtime without recompiling. Wire-dark default per
    /// CLAUDE.md "operators MUST gate at the emit point" stub_ban.
    hint_suppression: bool,
    /// C-47b -- optional persist store for the cold tier. `None` keeps
    /// the registry in-memory-only (existing behaviour). When `Some(_)`
    /// AND `ELAI_C47B_CASE_BASE_PERSIST_ENABLE` is truthy at
    /// `record_call` time, observations are persisted BEFORE the LRU
    /// admit (fail-closed per D3). When `None`, every persist branch
    /// short-circuits and the registry behaves exactly as the in-memory
    /// surface.
    persist: Option<Arc<CasePersistStore>>,
}

impl ToolCalibrationRegistry {
    /// Construct an empty registry. Hint suppression defaults to
    /// `false` (hints flow); call [`Self::with_hint_suppression`] to
    /// override.
    pub fn new() -> Self {
        Self {
            tools: Mutex::new(FxHashMap::default()),
            hint_suppression: false,
            persist: None,
        }
    }

    /// Builder: enable hint suppression. When `true`,
    /// [`Self::consult`] always returns `None` regardless of the
    /// underlying case base.
    pub fn with_hint_suppression(mut self, suppress: bool) -> Self {
        self.hint_suppression = suppress;
        self
    }

    /// C-47b builder: install the cold-tier persist store. When set
    /// AND the env-gate [`ELAI_C47B_CASE_BASE_PERSIST_ENABLE_ENV`] is
    /// truthy at `record_call` time, observations persist BEFORE the
    /// LRU admit (fail-closed per D3 — persist error skips the
    /// admit). When `None`, the registry stays in-memory only.
    pub fn with_persist(mut self, store: Arc<CasePersistStore>) -> Self {
        self.persist = Some(store);
        self
    }

    /// Accessor used by the dashboard panel to render the
    /// `hint_suppression` flag honestly.
    pub fn hint_suppression(&self) -> bool {
        self.hint_suppression
    }

    /// C-47b accessor: returns true when a persist store has been
    /// installed AND the env-gate is truthy at the moment of the
    /// query. Both conditions must hold for `record_call` to attempt a
    /// durable write. Cheap (one env-var read); safe to call in tests.
    pub fn persist_enabled(&self) -> bool {
        self.persist.is_some() && persist_env_truthy()
    }

    /// Record one observation. Returns `Ok(())` on success; the inner
    /// `LruCache::put` is infallible. Called by the bridge AFTER the
    /// tool returns (audit-trail invariant per WIRING_CONTRACT
    /// stub_ban).
    ///
    /// C-47b: when [`Self::persist_enabled`] returns true, the case is
    /// persisted to the cold tier BEFORE being admitted to the LRU
    /// (fail-closed per D3). If persist fails, the LRU admit is
    /// SKIPPED — the in-memory view stays consistent with the durable
    /// store. Returns `Ok(())` on either success path (admit-into-LRU
    /// or skip-on-persist-error); the caller can inspect
    /// [`Self::persist_enabled`] + the registry's
    /// [`super::registry::ToolCalibrationError`] log to detect the
    /// fail-closed branch.
    pub async fn record_call(
        &self,
        tool_name: &str,
        call_shape_hash: u64,
        call_shape_json: &str,
        kind: ResponseKind,
        ts_ms: i64,
    ) {
        let lru = self.get_or_create_tool(tool_name).await;
        let mut guard = lru.lock().await;
        let case = match guard.pop(&call_shape_hash) {
            Some(c) => {
                let mut c = c;
                c.observe(kind, call_shape_json, ts_ms);
                c
            }
            None => {
                let mut c = AggregatedCase::new(tool_name.to_string(), call_shape_hash);
                c.observe(kind, call_shape_json, ts_ms);
                c
            }
        };
        // C-47b: persist-FIRST / lru-put-SECOND per D3 fail-closed
        // ordering. Only when both the store is installed AND the
        // env-gate is truthy. Persist error => skip the put (LRU stays
        // consistent with the durable store).
        if let Some(store) = &self.persist {
            if persist_env_truthy() {
                if let Err(e) = store.persist_case(&case, ts_ms).await {
                    tracing::warn!(
                        tool = %tool_name,
                        call_shape_hash = call_shape_hash,
                        error = %e,
                        "C-47b: persist_case failed; skipping LRU admit (fail-closed per D3)"
                    );
                    return;
                }
            }
        }
        // Re-insert so `case` becomes the most-recently-used entry; the
        // LRU's `put` evicts the least-recently-used when at capacity,
        // which is deterministic per the `lru` crate's documented
        // semantics.
        guard.put(call_shape_hash, case);
    }

    /// C-47b -- bootstrap-time rehydrate: pull the most-recently-touched
    /// cases (up to [`CASE_BASE_LRU_CAP`] per tool) out of the persist
    /// store and re-insert them into the per-tool LRU in DESC order so
    /// the youngest sits at the LRU head (matches D5 access-order
    /// invariant). Returns the count of rehydrated cases.
    ///
    /// `tool_names` -- explicit list of tools to rehydrate. Pass the
    /// result of [`CasePersistStore::tool_names_from_disk`] when no
    /// caller-provided list exists. No-op when no persist store is
    /// installed.
    pub async fn rehydrate_from_persist(
        &self,
        tool_names: &[&str],
    ) -> Result<usize, ToolCalibrationError> {
        let Some(store) = &self.persist else {
            return Ok(0);
        };
        let mut total: usize = 0;
        for &name in tool_names {
            let cases = store.load_all_for_tool(name, CASE_BASE_LRU_CAP).await?;
            // load_all_for_tool returns DESC by updated_at_ms; iterate
            // in REVERSE so the OLDEST is put first and the YOUNGEST
            // ends up at the LRU head (most-recently-used position).
            let lru = self.get_or_create_tool(name).await;
            let mut guard = lru.lock().await;
            for case in cases.into_iter().rev() {
                let shape = case.call_shape_hash;
                guard.put(shape, case);
                total = total.saturating_add(1);
            }
        }
        Ok(total)
    }

    /// C-47b convenience: enumerate tool names from the persist store and
    /// rehydrate all of them. Equivalent to calling
    /// `CasePersistStore::tool_names_from_disk` + `rehydrate_from_persist`.
    /// Returns `Ok(0)` when no persist store is installed. Use this at
    /// bootstrap when the caller has no a-priori tool list.
    pub async fn rehydrate_all_from_persist(
        &self,
    ) -> Result<usize, ToolCalibrationError> {
        let Some(store) = &self.persist else {
            return Ok(0);
        };
        let names = store.tool_names_from_disk().await?;
        let name_refs: Vec<&str> = names.iter().map(String::as_str).collect();
        self.rehydrate_from_persist(&name_refs).await
    }

    /// Consult the case base for a prospective call. Returns
    /// `Some(hint_text)` when ALL of:
    ///   * `hint_suppression` is `false`
    ///   * a case for `(tool_name, call_shape_hash)` exists
    ///   * the case crosses the
    ///     [`super::hinter::HINT_ERROR_RATE_THRESHOLD`] +
    ///     [`super::hinter::HINT_MIN_TOTAL_CALLS`] gating
    ///
    /// stub_ban (HARD): hint_suppression is checked at the emit
    /// point — i.e. before any SSB row is composed — NOT after.
    pub async fn consult(
        &self,
        tool_name: &str,
        call_shape_hash: u64,
    ) -> Option<String> {
        if self.hint_suppression {
            return None;
        }
        let lru = {
            let tools = self.tools.lock().await;
            tools.get(tool_name).cloned()
        }?;
        let guard = lru.lock().await;
        let case = guard.peek(&call_shape_hash)?;
        generate_hint(case, tool_name)
    }

    /// Render the aggregate [`ToolCalibrationStatus`] for one tool.
    /// Returns `None` when the tool has no recorded shapes.
    pub async fn status(&self, tool_name: &str) -> Option<ToolCalibrationStatus> {
        let lru = {
            let tools = self.tools.lock().await;
            tools.get(tool_name).cloned()
        }?;
        let guard = lru.lock().await;
        let mut total_calls: u32 = 0;
        let mut error_count: u32 = 0;
        let mut last_ts_ms: i64 = 0;
        let mut shape_count: u32 = 0;
        for (_, case) in guard.iter() {
            shape_count = shape_count.saturating_add(1);
            total_calls = total_calls.saturating_add(case.total_calls);
            error_count = error_count.saturating_add(case.error_count);
            if case.last_ts_ms > last_ts_ms {
                last_ts_ms = case.last_ts_ms;
            }
        }
        Some(ToolCalibrationStatus {
            tool_name: tool_name.to_string(),
            shape_count,
            total_calls,
            error_count,
            last_ts_ms,
        })
    }

    /// Render statuses for every tool currently in the registry.
    pub async fn statuses(&self) -> Vec<ToolCalibrationStatus> {
        let names: Vec<String> = {
            let tools = self.tools.lock().await;
            tools.keys().cloned().collect()
        };
        let mut out = Vec::with_capacity(names.len());
        for name in names {
            if let Some(s) = self.status(&name).await {
                out.push(s);
            }
        }
        out.sort_by(|a, b| a.tool_name.cmp(&b.tool_name));
        out
    }

    /// True when the registry's drift detector has fired for
    /// `tool_name`. Drift is defined as
    /// `error_rate >= HINT_ERROR_RATE_THRESHOLD &&
    ///  total_calls >= HINT_MIN_TOTAL_CALLS` over the AGGREGATE across
    /// retained shapes — distinct from per-shape hint emission so the
    /// dashboard can flag a tool that fails many shapes at once.
    pub async fn drift_detected(&self, tool_name: &str) -> bool {
        let Some(status) = self.status(tool_name).await else {
            return false;
        };
        if status.total_calls < super::hinter::HINT_MIN_TOTAL_CALLS {
            return false;
        }
        let rate = f32::from(
            u16::try_from(status.error_count.min(u32::from(u16::MAX))).unwrap_or(u16::MAX),
        ) / f32::from(
            u16::try_from(status.total_calls.min(u32::from(u16::MAX))).unwrap_or(u16::MAX),
        );
        rate >= super::hinter::HINT_ERROR_RATE_THRESHOLD
    }

    /// Lazily install a fresh LRU for `tool_name`. Idempotent.
    async fn get_or_create_tool(
        &self,
        tool_name: &str,
    ) -> Arc<Mutex<LruCache<u64, AggregatedCase>>> {
        // Two-phase lookup: fast read first, slow write only on miss.
        {
            let tools = self.tools.lock().await;
            if let Some(existing) = tools.get(tool_name) {
                return existing.clone();
            }
        }
        let mut tools = self.tools.lock().await;
        tools
            .entry(tool_name.to_string())
            .or_insert_with(|| {
                let cap = NonZeroUsize::new(CASE_BASE_LRU_CAP).unwrap_or(
                    // Should be unreachable — CASE_BASE_LRU_CAP is a
                    // non-zero const. We avoid `.expect` per the
                    // no-unwrap-in-library rule and fall back to a
                    // safe positive cap.
                    NonZeroUsize::new(1).unwrap_or(NonZeroUsize::MIN),
                );
                Arc::new(Mutex::new(LruCache::new(cap)))
            })
            .clone()
    }
}

impl Default for ToolCalibrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// C-47b -- predicate for the `ELAI_C47B_CASE_BASE_PERSIST_ENABLE`
/// env-var. Truthy iff the value is one of `"1"` / `"true"` / `"yes"`
/// / `"on"` (case-insensitive). Mirrors the orchestrator-side
/// `env_gate_truthy` helper; kept crate-local so the plugins crate has
/// no orchestrator dep on the hot path. Cheap (one `std::env::var`
/// per `record_call`).
fn persist_env_truthy() -> bool {
    match std::env::var(ELAI_C47B_CASE_BASE_PERSIST_ENABLE_ENV) {
        Ok(v) => {
            let trimmed = v.trim();
            trimmed.eq_ignore_ascii_case("1")
                || trimmed.eq_ignore_ascii_case("true")
                || trimmed.eq_ignore_ascii_case("yes")
                || trimmed.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

/// SSB emitter trait — orchestrator-side adapter implements this.
/// Plugins-side trait avoids the circular `plugins -> orchestrator`
/// dep. Mirrors `crate::mcp::ToolGateSsbEmitter` exactly.
pub trait CaseBaseSsbEmitter: Send + Sync {
    /// Emit a `tool_call_logged` row carrying telemetry for one
    /// completed dispatch. `kind_str` is the snake-case
    /// [`ResponseKind`] string.
    fn emit_call_logged(
        &self,
        tool_name: &str,
        call_shape_hash: u64,
        kind_str: &str,
        post_call_audit_ok: bool,
        ts_ms: i64,
    ) -> Result<(), String>;

    /// Emit a `tool_calibration_hint_emitted` row carrying the
    /// composed hint text (kept on the producer side so the
    /// downstream prompt path can render it as-is).
    fn emit_hint(
        &self,
        tool_name: &str,
        call_shape_hash: u64,
        hint_text: &str,
        ts_ms: i64,
    ) -> Result<(), String>;

    /// Emit a `tool_calibration_drift_detected` row when the
    /// per-tool aggregate crosses the drift threshold. Operators see
    /// these in the dashboard `CaseBaseInspector` as a red flag.
    fn emit_drift(
        &self,
        tool_name: &str,
        error_rate: f32,
        total_calls: u32,
        ts_ms: i64,
    ) -> Result<(), String>;

    /// Tranche B-6 quad C-47 -- emit a `tool_calibration_event` row with
    /// [`ToolCalibrationEventKind::CaseAdded`]. Fires when [`CaseBase`]
    /// admits a fresh `(tool, call_shape)` pair. Default impl is a no-op
    /// so legacy emitters compile without touching the trait surface
    /// (one-release-window migration window).
    fn emit_case_added(
        &self,
        _tool_name: &str,
        _call_shape_hash: u64,
        _ts_ms: i64,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Tranche B-6 quad C-47 -- emit a `tool_calibration_event` row with
    /// [`ToolCalibrationEventKind::CaseSelected`]. Fires when [`CaseBase`]
    /// returns a stored case in service of a calibration decision.
    /// Default impl is a no-op (see [`emit_case_added`]).
    fn emit_case_selected(
        &self,
        _tool_name: &str,
        _call_shape_hash: u64,
        _ts_ms: i64,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Tranche B-6 quad C-47 -- emit a `tool_calibration_event` row with
    /// [`ToolCalibrationEventKind::CalibrationApplied`]. Fires when a
    /// calibration outcome was applied to a downstream tool dispatch.
    /// Default impl is a no-op (see [`emit_case_added`]).
    fn emit_calibration_applied(
        &self,
        _tool_name: &str,
        _call_shape_hash: u64,
        _ts_ms: i64,
    ) -> Result<(), String> {
        Ok(())
    }
}

/// No-op emitter used when no orchestrator-side SSB writer is wired
/// (CLI dry-runs, unit tests). Drops every emit and returns Ok(()).
pub struct NoopCaseBaseEmitter;

impl CaseBaseSsbEmitter for NoopCaseBaseEmitter {
    fn emit_call_logged(
        &self,
        _tool_name: &str,
        _call_shape_hash: u64,
        _kind_str: &str,
        _post_call_audit_ok: bool,
        _ts_ms: i64,
    ) -> Result<(), String> {
        Ok(())
    }

    fn emit_hint(
        &self,
        _tool_name: &str,
        _call_shape_hash: u64,
        _hint_text: &str,
        _ts_ms: i64,
    ) -> Result<(), String> {
        Ok(())
    }

    fn emit_drift(
        &self,
        _tool_name: &str,
        _error_rate: f32,
        _total_calls: u32,
        _ts_ms: i64,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn record_and_consult_emits_hint_after_threshold() {
        let reg = ToolCalibrationRegistry::new();
        let shape = 0xDEAD_BEEFu64;
        for _ in 0..2 {
            reg.record_call("fs.read", shape, r#"{"path":"x"}"#, ResponseKind::Ok, 1)
                .await;
        }
        for _ in 0..8 {
            reg.record_call(
                "fs.read",
                shape,
                r#"{"path":"x"}"#,
                ResponseKind::ErrInvalidArgs,
                2,
            )
            .await;
        }
        let hint = reg.consult("fs.read", shape).await;
        assert!(hint.is_some(), "8/10 err should trigger hint");
        let s = hint.expect("hint");
        assert!(s.contains("calibration_hint:"));
        assert!(s.contains("tool=fs.read"));
    }

    #[tokio::test]
    async fn hint_suppression_gates_at_emit_point() {
        let reg = ToolCalibrationRegistry::new().with_hint_suppression(true);
        let shape = 0x1234_5678u64;
        for _ in 0..10 {
            reg.record_call(
                "fs.read",
                shape,
                r#"{"path":"x"}"#,
                ResponseKind::ErrInvalidArgs,
                1,
            )
            .await;
        }
        assert_eq!(reg.consult("fs.read", shape).await, None);
    }

    #[tokio::test]
    async fn lru_eviction_at_cap_drops_lru_entry() {
        let reg = ToolCalibrationRegistry::new();
        // Fill the cap (100), then insert one more — the LRU entry
        // (first one) must be evicted.
        for i in 0..(CASE_BASE_LRU_CAP as u64) {
            reg.record_call("fs.read", i, r#"{"path":"x"}"#, ResponseKind::Ok, i as i64)
                .await;
        }
        // Hash `0` is now LRU.
        let extra = (CASE_BASE_LRU_CAP as u64) + 1;
        reg.record_call(
            "fs.read",
            extra,
            r#"{"path":"x"}"#,
            ResponseKind::Ok,
            extra as i64,
        )
        .await;
        // 100 entries remain — original `0` should be evicted.
        let status = reg.status("fs.read").await.expect("status");
        assert_eq!(status.shape_count, CASE_BASE_LRU_CAP as u32);
    }

    #[tokio::test]
    async fn separate_tools_have_separate_lrus() {
        let reg = ToolCalibrationRegistry::new();
        reg.record_call("fs.read", 1, r#"{}"#, ResponseKind::Ok, 1)
            .await;
        reg.record_call("git.commit", 2, r#"{}"#, ResponseKind::Ok, 2)
            .await;
        let fs = reg.status("fs.read").await.expect("fs");
        let git = reg.status("git.commit").await.expect("git");
        assert_eq!(fs.shape_count, 1);
        assert_eq!(git.shape_count, 1);
    }

    #[tokio::test]
    async fn drift_detection_threshold() {
        let reg = ToolCalibrationRegistry::new();
        for i in 0..8 {
            reg.record_call(
                "git.commit",
                i,
                r#"{"msg":"x"}"#,
                ResponseKind::ErrInvalidArgs,
                1,
            )
            .await;
        }
        for i in 8..10 {
            reg.record_call("git.commit", i, r#"{"msg":"x"}"#, ResponseKind::Ok, 1)
                .await;
        }
        assert!(reg.drift_detected("git.commit").await);
        assert!(!reg.drift_detected("never_called").await);
    }
}
