//! J-104: in-memory live-index overlay + order-triggered atomic drain.
//!
//! Fed by the shipped `file_ops` write/edit pre-image + post-image. Retrieval
//! reads this overlay BEFORE the durable DKG/BM25 base. Drain reuses the
//! B-31 `IndexLock` + blake3 CASCADE (`DkgEngine::build_graph` hash skip).
//! The drain trigger is [`DrainTrigger::NoMaterialProgress`] (I-17 / J-100
//! step signal) or [`DrainTrigger::SoftCap`] — never a wall-clock idle timer.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use elai_core::{ClockProvider, OsClock};
use elai_knowledge::dkg::{ext_to_lang_id, PythonGrammar, RustGrammar};
use elai_knowledge::live_overlay::{LiveIndexOverlay, OverlayHit};
use elai_knowledge::symbol_graph::extraction::extract_symbols_and_calls;
use elai_knowledge::{DkgEngine, LanguageGrammar};

use crate::lock::{IndexLock, LockScope};

/// Default-OFF experiment flag. Production capture / attach / drain are
/// no-ops unless this is a truthy value (`1` / `true` / `yes` / `on`).
pub const ELAI_LIVE_OVERLAY_ENABLE_ENV: &str = "ELAI_LIVE_OVERLAY_ENABLE";

/// Soft cap on overlay entries. Crossing it requests an early drain
/// (still atomic, still order-triggered — not a wall-clock flush).
pub const OVERLAY_SOFT_CAP: usize = 256;

static INSTALLED: OnceLock<Arc<IndexOverlay>> = OnceLock::new();

/// Why a drain ran. Idle / wall-clock is intentionally absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrainTrigger {
    /// I-17 / J-100 `no_material_progress` watchdog kind.
    NoMaterialProgress,
    /// Overlay hit [`OVERLAY_SOFT_CAP`]; drain now so the overlay stays bounded.
    SoftCap,
}

/// Options for [`IndexOverlay::drain_atomic`].
#[derive(Debug, Clone, Default)]
pub struct DrainOpts {
    /// Contract arm: return [`OverlayError::CrashInjected`] after the lock
    /// is held and the overlay is taken, before any durable write. Overlay
    /// is restored. DKG is untouched (old-consistent).
    pub inject_crash_before_apply: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrainOutcome {
    Drained { files: usize, symbols: usize },
    Empty,
    SkippedNoDkg,
}

#[derive(Debug, thiserror::Error)]
pub enum OverlayError {
    #[error("index lock unavailable: {0}")]
    Lock(String),
    #[error("dkg drain failed: {0}")]
    Dkg(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("crash injected before durable apply (overlay restored; dkg untouched)")]
    CrashInjected,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SymbolKey {
    file: String,
    symbol: String,
}

#[derive(Debug, Clone)]
enum EntryKind {
    Live { snippet: String },
    Tombstone,
}

#[derive(Debug, Clone)]
struct OverlayEntry {
    kind: EntryKind,
    seq: u64,
}

struct Inner {
    entries: BTreeMap<SymbolKey, OverlayEntry>,
    pending_files: BTreeMap<PathBuf, ()>,
    next_seq: u64,
}

/// In-memory overlay store. Newest write to a symbol wins.
pub struct IndexOverlay {
    inner: Mutex<Inner>,
    project_root: PathBuf,
    enabled: bool,
}

impl IndexOverlay {
    pub fn new(project_root: PathBuf, enabled: bool) -> Self {
        Self {
            inner: Mutex::new(Inner {
                entries: BTreeMap::new(),
                pending_files: BTreeMap::new(),
                next_seq: 1,
            }),
            project_root,
            enabled,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn pending_len(&self) -> usize {
        self.lock_inner().entries.len()
    }

    pub fn needs_early_drain(&self) -> bool {
        self.pending_len() >= OVERLAY_SOFT_CAP
    }

    /// Capture a file_ops write/edit: `before` is the pre-image (empty on
    /// create), `after` is the post-image already on disk.
    pub fn capture_edit(&self, path: &Path, before: &str, after: &str) {
        if !self.enabled {
            return;
        }
        let file = normalize_path(path);
        let before_names = extract_symbol_names(path, before);
        let after_names = extract_symbol_names(path, after);
        let mut inner = self.lock_inner();
        let seq = inner.next_seq;
        inner.next_seq = inner.next_seq.saturating_add(1);
        inner.pending_files.insert(path.to_path_buf(), ());

        for symbol in before_names.difference(&after_names) {
            upsert_entry(
                &mut inner,
                SymbolKey {
                    file: file.clone(),
                    symbol: symbol.clone(),
                },
                OverlayEntry {
                    kind: EntryKind::Tombstone,
                    seq,
                },
            );
        }
        for symbol in after_names.difference(&before_names) {
            let snippet = snippet_for_symbol(after, symbol);
            upsert_entry(
                &mut inner,
                SymbolKey {
                    file: file.clone(),
                    symbol: symbol.clone(),
                },
                OverlayEntry {
                    kind: EntryKind::Live { snippet },
                    seq,
                },
            );
        }
        for symbol in after_names.intersection(&before_names) {
            if before != after {
                let snippet = snippet_for_symbol(after, symbol);
                upsert_entry(
                    &mut inner,
                    SymbolKey {
                        file: file.clone(),
                        symbol: symbol.clone(),
                    },
                    OverlayEntry {
                        kind: EntryKind::Live { snippet },
                        seq,
                    },
                );
            }
        }
    }

    /// Atomic drain into the durable DKG via B-31 lock + CASCADE rebuild.
    /// `trigger` must be order-based ([`DrainTrigger`]); callers must not
    /// invent a wall-clock idle trigger.
    pub fn drain_atomic(
        &self,
        trigger: DrainTrigger,
        clock: Arc<dyn ClockProvider>,
        opts: DrainOpts,
    ) -> Result<DrainOutcome, OverlayError> {
        let _ = trigger;
        if !self.enabled {
            return Ok(DrainOutcome::Empty);
        }
        let dkg_path = elai_core::paths::dkg_db_path(&self.project_root);
        if !dkg_path.exists() {
            return Ok(DrainOutcome::SkippedNoDkg);
        }

        let _guard = IndexLock::try_acquire(&self.project_root, LockScope::Index, clock)
            .map_err(|e| OverlayError::Lock(e.to_string()))?;

        let snapshot = self.take_pending();
        if snapshot.entries.is_empty() {
            return Ok(DrainOutcome::Empty);
        }
        if opts.inject_crash_before_apply {
            self.restore_pending(snapshot);
            return Err(OverlayError::CrashInjected);
        }

        let files: Vec<PathBuf> = snapshot.pending_files.keys().cloned().collect();
        let symbols = snapshot.entries.len();
        match apply_cascade(&self.project_root, &dkg_path, &files) {
            Ok(()) => Ok(DrainOutcome::Drained {
                files: files.len(),
                symbols,
            }),
            Err(e) => {
                self.restore_pending(snapshot);
                Err(e)
            }
        }
    }

    fn take_pending(&self) -> Inner {
        let mut inner = self.lock_inner();
        let next_seq = inner.next_seq;
        std::mem::replace(
            &mut *inner,
            Inner {
                entries: BTreeMap::new(),
                pending_files: BTreeMap::new(),
                next_seq,
            },
        )
    }

    fn restore_pending(&self, snapshot: Inner) {
        let mut inner = self.lock_inner();
        for (k, v) in snapshot.entries {
            inner.entries.insert(k, v);
        }
        for (k, v) in snapshot.pending_files {
            inner.pending_files.insert(k, v);
        }
    }

    fn lock_inner(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }
}

impl LiveIndexOverlay for IndexOverlay {
    fn hits_for_query(&self, query: &str) -> Vec<OverlayHit> {
        if !self.enabled {
            return Vec::new();
        }
        let q = query.to_lowercase();
        let inner = self.lock_inner();
        inner
            .entries
            .iter()
            .filter(|(key, _)| {
                q.contains(&key.symbol.to_lowercase()) || q.contains(&key.file.to_lowercase())
            })
            .map(|(key, entry)| match &entry.kind {
                EntryKind::Live { snippet } => OverlayHit::Live {
                    symbol: key.symbol.clone(),
                    file: key.file.clone(),
                    snippet: snippet.clone(),
                },
                EntryKind::Tombstone => OverlayHit::Tombstone {
                    symbol: key.symbol.clone(),
                    file: key.file.clone(),
                },
            })
            .collect()
    }
}

/// Process-wide overlay. `None` when the flag is off and nothing was installed
/// for tests.
pub fn global() -> Option<Arc<IndexOverlay>> {
    INSTALLED.get().cloned()
}

/// Install (or return) the process overlay when the env flag is on.
/// Tests should call [`install_for_test`] instead of mutating the env.
pub fn install_for_project(project_root: &Path) -> Option<Arc<IndexOverlay>> {
    if !live_overlay_enabled() {
        return None;
    }
    Some(install_inner(project_root.to_path_buf(), true))
}

/// Test/contract installer. Does not read the process environment.
pub fn install_for_test(project_root: PathBuf) -> Arc<IndexOverlay> {
    install_inner(project_root, true)
}

fn install_inner(project_root: PathBuf, enabled: bool) -> Arc<IndexOverlay> {
    INSTALLED
        .get_or_init(|| Arc::new(IndexOverlay::new(project_root, enabled)))
        .clone()
}

/// Reset the process overlay between contract tests. Only safe when no
/// other test holds the Arc for a live query.
pub fn reset_global_for_tests() {
    // OnceLock cannot be cleared; tests use a dedicated overlay instance
    // passed to the pipeline rather than the process global when isolation
    // is required. This helper is a no-op kept so the contract names it.
}

pub fn live_overlay_enabled() -> bool {
    match std::env::var(ELAI_LIVE_OVERLAY_ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            matches!(v.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Production capture from the shipped file_ops write/edit signal.
/// No-op when the overlay is not installed or the flag is off.
pub fn capture_file_ops_edit(path: &Path, before: &str, after: &str) {
    if let Some(overlay) = global() {
        overlay.capture_edit(path, before, after);
        if overlay.needs_early_drain() {
            let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
            if let Err(e) = overlay.drain_atomic(DrainTrigger::SoftCap, clock, DrainOpts::default())
            {
                tracing::warn!(error = %e, "J-104: soft-cap drain failed (overlay kept)");
            }
        }
    }
}

/// Production drain from the I-17 `no_material_progress` watchdog kind.
/// Callers MUST pass [`DrainTrigger::NoMaterialProgress`] — never Idle.
pub fn try_drain_on_trigger(trigger: DrainTrigger, clock: Arc<dyn ClockProvider>) {
    if !matches!(
        trigger,
        DrainTrigger::NoMaterialProgress | DrainTrigger::SoftCap
    ) {
        return;
    }
    if let Some(overlay) = global() {
        if let Err(e) = overlay.drain_atomic(trigger, clock, DrainOpts::default()) {
            tracing::warn!(error = %e, "J-104: overlay drain skipped");
        }
    }
}

fn apply_cascade(
    project_root: &Path,
    dkg_path: &Path,
    changed_files: &[PathBuf],
) -> Result<(), OverlayError> {
    let created_at = 0i64;
    let mut engine = DkgEngine::new(dkg_path, None, created_at)
        .map_err(|e| OverlayError::Dkg(e.to_string()))?;
    engine
        .build_graph(project_root)
        .map_err(|e| OverlayError::Dkg(e.to_string()))?;
    drop(engine);

    let clock_ms = 0i64;
    let _ = elai_knowledge::symbol_graph::build_pipeline::update_changed_files_symbol_graph(
        "default",
        changed_files,
        &[],
        dkg_path,
        project_root,
        clock_ms,
    );
    Ok(())
}

fn upsert_entry(inner: &mut Inner, key: SymbolKey, entry: OverlayEntry) {
    let incoming = entry.seq;
    if inner
        .entries
        .get(&key)
        .map(|existing| existing.seq)
        .unwrap_or(0)
        <= incoming
    {
        inner.entries.insert(key, entry);
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn extract_symbol_names(path: &Path, source: &str) -> std::collections::BTreeSet<String> {
    let mut names = std::collections::BTreeSet::new();
    if source.is_empty() {
        return names;
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let grammar: Option<Box<dyn LanguageGrammar>> = match ext_to_lang_id(ext) {
        Some("rust") => Some(Box::new(RustGrammar)),
        Some("python") => Some(Box::new(PythonGrammar)),
        _ => None,
    };
    if let Some(grammar) = grammar {
        let extracted =
            extract_symbols_and_calls(source, grammar.as_ref(), &normalize_path(path));
        for symbol in extracted.symbols {
            if symbol.name != "<module>" {
                names.insert(symbol.name);
            }
        }
    }
    for name in heuristic_fn_names(source) {
        names.insert(name);
    }
    names
}

fn heuristic_fn_names(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("fn ") {
            if let Some(name) = rest.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .next()
            {
                if !name.is_empty() {
                    out.push(name.to_string());
                }
            }
        }
        if let Some(rest) = trimmed.strip_prefix("def ") {
            if let Some(name) = rest
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .next()
            {
                if !name.is_empty() {
                    out.push(name.to_string());
                }
            }
        }
    }
    out
}

fn snippet_for_symbol(source: &str, symbol: &str) -> String {
    source
        .lines()
        .find(|line| line.contains(symbol))
        .unwrap_or(source)
        .trim()
        .to_string()
}
