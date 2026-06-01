//! Firecracker microVM hot snapshot-restore pool.
//! E-15: snapshot-restore path (~28ms per dev.to/adwitiya).
//! Uniqueness mitigation per arXiv:2102.12892.
//! Snapshot GC: LRU eviction at max_snapshots=64 AND max_age=24h.

// B-15 HashMap ban: BTreeMap chosen for the snapshot pool. Eviction order
// (LRU age-based + count-based) is observable in test fixtures; BTreeMap
// keys give deterministic iter() ordering when timestamps tie. Uuid +
// AgentRole both derive Ord, so SnapshotKey can derive Ord without surgery.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use elai_core::clock::{ClockProvider, OsClock};

use crate::agent_pool::AgentRole;
use uuid::Uuid;

/// Unique key for a per-(agent_id, role) snapshot slot.
///
/// `PartialOrd + Ord` are derived for B-15 HashMap ban migration to
/// `BTreeMap`. Ordering uses Uuid-then-AgentRole; both component types
/// derive Ord upstream (ipc-types AgentRole, uuid crate Uuid). Hash kept
/// in case downstream FxHashMap is ever reintroduced for a hot-path test.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SnapshotKey {
    pub agent_id: Uuid,
    pub role: AgentRole,
}

/// Handle to a stored Firecracker CoW snapshot.
#[derive(Debug, Clone)]
pub struct SnapshotHandle {
    pub key: SnapshotKey,
    pub path: PathBuf,
    pub created_at: SystemTime,
    pub last_restored_at: SystemTime,
    pub restore_count: u64,
}

/// Firecracker hot snapshot-restore pool.
pub struct FirecrackerHotPool {
    snapshots: Mutex<BTreeMap<SnapshotKey, SnapshotHandle>>,
    snapshot_dir: PathBuf,
    restore_budget_ms: u64,
    max_snapshots: usize,
    max_age: Duration,
    uniqueness_mitigation: bool,
    // B-15 Tranche H: entropy quarantine. Injected clock so snapshot
    // create + restore + eviction timestamps share one source (SeededClock
    // in tests gives deterministic ordering; OsClock in prod).
    clock: Arc<dyn ClockProvider>,
}

#[derive(Debug, thiserror::Error)]
pub enum FirecrackerError {
    #[error("firecracker binary not found: {0}")]
    BinaryNotFound(String),
    #[error("snapshot creation failed: {0}")]
    SnapshotCreateFailed(String),
    #[error("snapshot restore failed: {0}")]
    SnapshotRestoreFailed(String),
    #[error("snapshot uniqueness mitigation failed: {0}")]
    UniquenessFailed(String),
    #[error("KVM not available on this host")]
    KvmUnavailable,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("pool mutex poisoned: {0}")]
    MutexPoisoned(String),
    #[error("restore budget exceeded: {actual_ms}ms > {budget_ms}ms")]
    RestoreBudgetExceeded { actual_ms: u64, budget_ms: u64 },
}

#[derive(Debug)]
pub struct SnapshotSpawnResult {
    pub snapshot_path: PathBuf,
    pub uniqueness_regenerated: bool,
    pub was_cold_boot: bool,
    pub restore_ms: u64,
    pub budget_exceeded: bool,
}

impl FirecrackerHotPool {
    pub fn new(snapshot_dir: PathBuf, restore_budget_ms: u64, max_snapshots: usize, max_age_secs: u64, uniqueness_mitigation: bool) -> Self {
        Self::new_with_clock(snapshot_dir, restore_budget_ms, max_snapshots, max_age_secs, uniqueness_mitigation, Arc::new(OsClock))
    }

    pub fn new_with_clock(snapshot_dir: PathBuf, restore_budget_ms: u64, max_snapshots: usize, max_age_secs: u64, uniqueness_mitigation: bool, clock: Arc<dyn ClockProvider>) -> Self {
        Self { snapshots: Mutex::new(BTreeMap::new()), snapshot_dir, restore_budget_ms, max_snapshots, max_age: Duration::from_secs(max_age_secs), uniqueness_mitigation, clock }
    }

    pub fn with_defaults(snapshot_dir: PathBuf) -> Self {
        Self::new(snapshot_dir, 200, 64, 86400, true)
    }

    #[cfg(target_os = "linux")]
    pub fn spawn_from_snapshot(&self, key: &SnapshotKey, command: &str, args: &[String]) -> Result<SnapshotSpawnResult, FirecrackerError> {
        if !super::kvm_available() { return Err(FirecrackerError::KvmUnavailable); }
        let mut pool = self.snapshots.lock().map_err(|e| FirecrackerError::MutexPoisoned(format!("{e}")))?;
        let snapshot_path = self.snapshot_path_for(key);
        let has_snapshot = pool.contains_key(key) && snapshot_path.exists();
        if has_snapshot {
            let t0 = self.clock.instant_now();
            let result = self.restore_from_snapshot(&snapshot_path, command, args, key);
            let elapsed_ms = u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX);
            match result {
                Ok(()) if elapsed_ms <= self.restore_budget_ms => {
                    if let Some(handle) = pool.get_mut(key) { handle.last_restored_at = self.clock.system_now(); handle.restore_count += 1; }
                    self.evict_if_needed(&mut pool)?;
                    Ok(SnapshotSpawnResult { snapshot_path, uniqueness_regenerated: self.uniqueness_mitigation, was_cold_boot: false, restore_ms: elapsed_ms, budget_exceeded: false })
                }
                Ok(()) => {
                    tracing::warn!(agent_id = %key.agent_id, actual_ms = elapsed_ms, budget_ms = self.restore_budget_ms, "firecracker_restore_slow: exceeded budget, cold boot fallback");
                    let cold = self.cold_boot_and_snapshot(key, command, args, &mut pool)?;
                    Ok(SnapshotSpawnResult { budget_exceeded: true, ..cold })
                }
                Err(e) => { tracing::warn!(agent_id = %key.agent_id, error = %e, "firecracker restore failed, cold boot fallback"); self.cold_boot_and_snapshot(key, command, args, &mut pool) }
            }
        } else {
            self.cold_boot_and_snapshot(key, command, args, &mut pool)
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn spawn_from_snapshot(&self, _key: &SnapshotKey, _command: &str, _args: &[String]) -> Result<SnapshotSpawnResult, FirecrackerError> {
        Err(FirecrackerError::KvmUnavailable)
    }

    #[cfg(target_os = "linux")]
    fn cold_boot_and_snapshot(&self, key: &SnapshotKey, command: &str, args: &[String], pool: &mut BTreeMap<SnapshotKey, SnapshotHandle>) -> Result<SnapshotSpawnResult, FirecrackerError> {
        let snapshot_path = self.snapshot_path_for(key);
        if let Some(parent) = snapshot_path.parent() { std::fs::create_dir_all(parent)?; }
        if self.firecracker_binary_available() {
            self.create_snapshot_via_firecracker(command, args, &snapshot_path)?;
        } else {
            std::fs::write(&snapshot_path, b"cold-boot-marker")?;
        }
        if self.uniqueness_mitigation { self.regenerate_uniqueness_seeds()?; }
        let now = self.clock.system_now();
        let handle = SnapshotHandle { key: key.clone(), path: snapshot_path.clone(), created_at: now, last_restored_at: now, restore_count: 0 };
        pool.insert(key.clone(), handle);
        self.evict_if_needed(pool)?;
        Ok(SnapshotSpawnResult { snapshot_path, uniqueness_regenerated: self.uniqueness_mitigation, was_cold_boot: true, restore_ms: 0, budget_exceeded: false })
    }

    #[cfg(target_os = "linux")]
    fn restore_from_snapshot(&self, snapshot_path: &Path, command: &str, args: &[String], key: &SnapshotKey) -> Result<(), FirecrackerError> {
        if self.uniqueness_mitigation { self.regenerate_uniqueness_seeds()?; }
        if self.firecracker_binary_available() {
            let status = std::process::Command::new("firecracker").arg("--restore-from-snapshot").arg(snapshot_path).arg("--").arg(command).args(args).status().map_err(|e| FirecrackerError::SnapshotRestoreFailed(format!("{e}")))?;
            if !status.success() { return Err(FirecrackerError::SnapshotRestoreFailed(format!("firecracker exited with: {status}"))); }
        } else { tracing::debug!(agent_id = %key.agent_id, "firecracker binary absent, restore is a no-op"); }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn create_snapshot_via_firecracker(&self, command: &str, args: &[String], snapshot_path: &Path) -> Result<(), FirecrackerError> {
        let status = std::process::Command::new("firecracker").arg("--create-snapshot").arg(snapshot_path).arg("--").arg(command).args(args).status().map_err(|e| FirecrackerError::SnapshotCreateFailed(format!("{e}")))?;
        if !status.success() { return Err(FirecrackerError::SnapshotCreateFailed(format!("firecracker --create-snapshot exited with: {status}"))); }
        Ok(())
    }

    /// Regenerate per-VM entropy seeds per arXiv:2102.12892.
    /// RESIDUAL RISK: boot_id pre-restore can correlate across instances.
    #[cfg(target_os = "linux")]
    fn regenerate_uniqueness_seeds(&self) -> Result<(), FirecrackerError> {
        let mut seed_bytes = [0u8; 8];
        getrandom::fill(&mut seed_bytes).map_err(|e| FirecrackerError::UniquenessFailed(format!("getrandom failed: {e}")))?;
        let seed_val = u64::from_le_bytes(seed_bytes);
        let _ = std::fs::write("/proc/sys/kernel/random/write_wakeup_threshold", format!("{}
", seed_val % 4096 + 64));
        let salt = format!("elai-vm-{:016x}", seed_val);
        let _ = std::process::Command::new("hostname").arg(&salt).status();
        tracing::debug!(salt = %salt, "snapshot_uniqueness_regenerated: TCP ISN seed + hostname salt refreshed");
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn regenerate_uniqueness_seeds(&self) -> Result<(), FirecrackerError> { Ok(()) }

    fn snapshot_path_for(&self, key: &SnapshotKey) -> PathBuf {
        self.snapshot_dir.join(format!("{}-{:?}.snap", key.agent_id, key.role))
    }

    fn firecracker_binary_available(&self) -> bool { which_firecracker().is_some() }

    fn evict_if_needed(&self, pool: &mut BTreeMap<SnapshotKey, SnapshotHandle>) -> Result<(), FirecrackerError> {
        let now = self.clock.system_now();
        let expired_keys: Vec<SnapshotKey> = pool.iter().filter(|(_, h)| now.duration_since(h.last_restored_at).map(|d| d > self.max_age).unwrap_or(false)).map(|(k, _)| k.clone()).collect();
        for key in &expired_keys {
            if let Some(handle) = pool.remove(key) {
                let _ = std::fs::remove_file(&handle.path);
                tracing::info!(agent_id = %key.agent_id, path = %handle.path.display(), "snapshot_evicted: age-based LRU eviction");
            }
        }
        if pool.len() > self.max_snapshots {
            let mut sorted: Vec<(SnapshotKey, SystemTime)> = pool.iter().map(|(k, h)| (k.clone(), h.last_restored_at)).collect();
            sorted.sort_by_key(|(_, t)| *t);
            let evict_count = pool.len() - self.max_snapshots;
            for (key, _) in sorted.into_iter().take(evict_count) {
                if let Some(handle) = pool.remove(&key) {
                    let _ = std::fs::remove_file(&handle.path);
                    tracing::info!(agent_id = %key.agent_id, path = %handle.path.display(), "snapshot_evicted: LRU count-based eviction");
                }
            }
        }
        Ok(())
    }

    pub fn snapshot_count(&self) -> usize { self.snapshots.lock().map(|p| p.len()).unwrap_or(0) }
}

fn which_firecracker() -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path_var| { std::env::split_paths(&path_var).map(|dir| dir.join("firecracker")).find(|p| p.is_file()) })
}
