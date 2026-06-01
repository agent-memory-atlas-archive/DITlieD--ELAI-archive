//! C-38: production-side DKG lifecycle management.
//!
//! Replaces the never-fires gate previously at `live_cli.rs:1722-1732`
//! ("if dkg.db exists, load engine; otherwise silently skip"). The
//! plain existence-check left Phase 1D PageRank dead at runtime on any
//! freshly-cloned workspace, because no production path created
//! `.elai/dkg.db` before the QueryPipeline first ran Stage 2. The
//! C-38 contract test
//! `crates/contracts/tests/c38_pagerank_production_contract.rs` locks
//! the rebuild-on-missing behaviour.
//!
//! Public entry point: [`ensure_dkg_or_rebuild`]. The helper:
//!   1. Resolves the canonical `.elai/dkg.db` path from `workspace_root`.
//!   2. If the file exists AND was modified within the staleness window
//!      (default 24h), opens the existing engine and returns it.
//!   3. Otherwise: emits one `DkgRebuildStartedEvent`, runs the existing
//!      `DkgEngine::build_graph(workspace_root)` rebuild, emits one
//!      `DkgRebuildCompleteEvent` (success OR fail), and returns the
//!      engine.
//!
//! Failure semantics: rebuild errors propagate via `anyhow::Result`
//! rather than the previous silent-skip pattern. Callers that want to
//! tolerate a missing DKG should use the existing
//! `pipeline.dkg_engine = Some(...)` builder path and choose whether to
//! attempt the rebuild based on their own policy. Production wiring at
//! `live_cli.rs:1722-1732` calls this helper and propagates errors as a
//! warning (degradation cascade picks up the absence-of-DKG signal).

use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{anyhow, Result};
use elai_core::clock::{ClockProvider, OsClock};
use elai_core::paths::dkg_db_path as core_dkg_db_path;
use elai_knowledge::dkg::DkgEngine;
use elai_knowledge::knowledge_ssb_sink::{
    DkgRebuildCompleteEvent, DkgRebuildStartedEvent, KnowledgeSsbSink,
};

/// Default DKG staleness window: 24 hours (in seconds). Below this age,
/// `ensure_dkg_or_rebuild` returns the existing on-disk engine without
/// rebuilding. Bumping the value reduces rebuild churn for static
/// workspaces; lowering it forces more frequent re-indexes. The single
/// constant exists so the contract test + future CLI flag can agree on
/// the same threshold.
pub const DEFAULT_STALENESS_SECONDS: u64 = 24 * 60 * 60;

/// C-38 hash prefix helper: derive a stable 64-bit blake3-prefix of the
/// workspace root path bytes. Used as `workspace_root_hash` in the
/// `dkg_rebuild_*` SSB rows so dashboards can group rebuild events by
/// workspace without leaking absolute paths into SSB.
fn workspace_root_hash(workspace_root: &Path) -> u64 {
    let bytes = workspace_root.to_string_lossy();
    let hash = blake3::hash(bytes.as_bytes());
    let h = hash.as_bytes();
    u64::from_le_bytes([h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]])
}

/// C-38: trigger reason carried on the `dkg_rebuild_started` SSB row.
/// Snake-case label so dashboards can group rebuild causes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebuildTrigger {
    /// `.elai/dkg.db` does not exist; first build for this workspace.
    Missing,
    /// File exists but modified-time exceeded the staleness window.
    Stale,
}

impl RebuildTrigger {
    fn as_snake(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::Stale => "stale",
        }
    }
}

/// Determine whether the on-disk DKG file should be rebuilt. Returns
/// `Some(trigger)` when a rebuild is required, `None` when the existing
/// file is fresh enough to load directly.
///
/// `now_secs` is the current wall-clock time in seconds-since-epoch;
/// callers pass `SystemTime::now()` derived from their injected clock
/// (or a deterministic value in tests). This lets the contract test
/// drive the staleness branch without touching the real system clock.
pub fn rebuild_decision(
    db_path: &Path,
    staleness_secs: u64,
    now_secs: u64,
) -> Option<RebuildTrigger> {
    if !db_path.exists() {
        return Some(RebuildTrigger::Missing);
    }
    // File exists; check modified-time against the staleness window.
    let metadata = match std::fs::metadata(db_path) {
        Ok(m) => m,
        Err(_) => return Some(RebuildTrigger::Missing),
    };
    let modified_secs = match metadata.modified() {
        Ok(m) => match m.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(_) => return Some(RebuildTrigger::Stale),
        },
        Err(_) => return Some(RebuildTrigger::Stale),
    };
    if now_secs.saturating_sub(modified_secs) > staleness_secs {
        Some(RebuildTrigger::Stale)
    } else {
        None
    }
}

/// C-38 production entry point. Replaces the never-fires
/// "if dkg.db exists" gate at `live_cli.rs:1722-1732`.
///
/// `workspace_root` is the project root (the same value passed to the
/// existing `DkgEngine::build_graph`). `sink` is the typed knowledge
/// SSB sink already constructed by `build_local_knowledge_stack`;
/// production passes the `OrchestratorKnowledgeSsbSink` adapter, tests
/// pass a recording sink or the no-op default.
///
/// Returns the loaded-or-rebuilt `DkgEngine`. Errors propagate via
/// `anyhow::Result` so callers can choose to surface a warning OR
/// abort entirely — the previous gate silently dropped failures on the
/// floor, which is the bug this helper closes.
pub fn ensure_dkg_or_rebuild(
    workspace_root: &Path,
    sink: Arc<dyn KnowledgeSsbSink>,
) -> Result<DkgEngine> {
    // B-15 Tranche E E.1: ClockProvider via zero-sized OsClock Copy impl.
    // The trait's default `now_secs` absorbs the
    // `.duration_since(UNIX_EPOCH).unwrap_or(0)` saturation
    // (crates/core/src/clock.rs:67-72). _with_clock variant below remains
    // the test-friendly entry point.
    let now_secs = OsClock.now_secs();
    ensure_dkg_or_rebuild_with_clock(
        workspace_root,
        sink,
        DEFAULT_STALENESS_SECONDS,
        now_secs,
    )
}

/// Test-friendly variant of [`ensure_dkg_or_rebuild`] that takes the
/// staleness window and wall-clock time as explicit arguments. The
/// C-38 contract test uses this entry point to drive the rebuild path
/// deterministically.
pub fn ensure_dkg_or_rebuild_with_clock(
    workspace_root: &Path,
    sink: Arc<dyn KnowledgeSsbSink>,
    staleness_secs: u64,
    now_secs: u64,
) -> Result<DkgEngine> {
    let db_path = core_dkg_db_path(workspace_root);
    // Ensure the parent `.elai/` directory exists so DkgEngine::new can
    // open / create the sqlite file. Missing-parent is a real failure
    // mode on a freshly-cloned workspace.
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            anyhow!(
                "ensure_dkg_or_rebuild: cannot create parent {}: {}",
                parent.display(),
                e
            )
        })?;
    }

    let ws_hash = workspace_root_hash(workspace_root);
    let trigger = rebuild_decision(&db_path, staleness_secs, now_secs);

    if trigger.is_none() {
        // Fresh on-disk file: load and return without rebuild.
        let engine = DkgEngine::new(&db_path, None, 0).map_err(|e| {
            anyhow!(
                "ensure_dkg_or_rebuild: DkgEngine::new on existing fresh {}: {}",
                db_path.display(),
                e
            )
        })?;
        return Ok(engine);
    }

    let trigger = trigger.expect("trigger is Some when rebuild needed");
    sink.emit_dkg_rebuild_started(DkgRebuildStartedEvent {
        workspace_root_hash: ws_hash,
        trigger_reason: trigger.as_snake().to_owned(),
    });

    // B-15 Tranche E E.1: ClockProvider via zero-sized OsClock Copy impl.
    // Switched from SystemTime::now()+elapsed() (which can fault on clock
    // jumps) to the trait's monotonic `instant_now()`
    // (crates/core/src/clock.rs:54). Duration measurement is identical
    // shape, but jump-safe.
    let started_at = OsClock.instant_now();
    let rebuild_result = run_rebuild(&db_path, workspace_root);
    let duration_ms = started_at.elapsed().as_millis() as u64;

    match rebuild_result {
        Ok((engine, node_count)) => {
            let node_count_u32 = u32::try_from(node_count).unwrap_or(u32::MAX);
            sink.emit_dkg_rebuild_complete(DkgRebuildCompleteEvent {
                workspace_root_hash: ws_hash,
                success: true,
                duration_ms,
                node_count: node_count_u32,
            });
            Ok(engine)
        }
        Err(e) => {
            sink.emit_dkg_rebuild_complete(DkgRebuildCompleteEvent {
                workspace_root_hash: ws_hash,
                success: false,
                duration_ms,
                node_count: 0,
            });
            Err(e)
        }
    }
}

/// Run the actual rebuild: open the engine, walk the workspace, return
/// the populated engine alongside the node count. Split out so
/// [`ensure_dkg_or_rebuild_with_clock`] can wrap a single Result<_>
/// branch in the success/failure SSB-emission pair.
fn run_rebuild(db_path: &Path, workspace_root: &Path) -> Result<(DkgEngine, usize)> {
    let mut engine = DkgEngine::new(db_path, None, 0).map_err(|e| {
        anyhow!(
            "ensure_dkg_or_rebuild: DkgEngine::new for rebuild on {}: {}",
            db_path.display(),
            e
        )
    })?;
    engine.build_graph(workspace_root).map_err(|e| {
        anyhow!(
            "ensure_dkg_or_rebuild: build_graph failed on {}: {}",
            workspace_root.display(),
            e
        )
    })?;
    let node_count = engine
        .nodes()
        .map(|ns| ns.len())
        .map_err(|e| anyhow!("ensure_dkg_or_rebuild: post-rebuild node count: {}", e))?;
    Ok((engine, node_count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[test]
    fn missing_file_triggers_missing() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("not_there.db");
        assert_eq!(
            rebuild_decision(&path, DEFAULT_STALENESS_SECONDS, 0),
            Some(RebuildTrigger::Missing)
        );
    }

    #[test]
    fn fresh_file_returns_none() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("fresh.db");
        std::fs::write(&path, b"").expect("write");
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("epoch")
            .as_secs();
        // Just-written file is fresh — should not trigger.
        assert_eq!(rebuild_decision(&path, DEFAULT_STALENESS_SECONDS, now), None);
    }

    #[test]
    fn stale_file_triggers_stale() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("stale.db");
        std::fs::write(&path, b"").expect("write");
        // Pretend "now" is far in the future relative to the file's
        // modified-time, larger than the configured window.
        let modified_secs = std::fs::metadata(&path)
            .expect("metadata")
            .modified()
            .expect("modified")
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("epoch")
            .as_secs();
        let future_now = modified_secs + DEFAULT_STALENESS_SECONDS + 1;
        assert_eq!(
            rebuild_decision(&path, DEFAULT_STALENESS_SECONDS, future_now),
            Some(RebuildTrigger::Stale)
        );
    }

    #[test]
    fn rebuild_trigger_snake_labels() {
        assert_eq!(RebuildTrigger::Missing.as_snake(), "missing");
        assert_eq!(RebuildTrigger::Stale.as_snake(), "stale");
    }

    #[test]
    fn workspace_hash_is_stable() {
        let p1 = Path::new("/workspace/projA");
        let p2 = Path::new("/workspace/projA");
        let p3 = Path::new("/workspace/projB");
        assert_eq!(workspace_root_hash(p1), workspace_root_hash(p2));
        assert_ne!(workspace_root_hash(p1), workspace_root_hash(p3));
    }

    #[test]
    fn workspace_hash_does_not_depend_on_duration() {
        // Compile-time presence-check for the std::time::Duration import
        // in the test block (avoid the dead-import lint without inflating
        // the production module).
        let _ = Duration::from_secs(1);
    }
}
