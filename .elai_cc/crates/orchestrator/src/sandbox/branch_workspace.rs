// ---------------------------------------------------------------------------
// I-2: CoW workspace isolation — per-branch scratch with deterministic barrier merge
// ---------------------------------------------------------------------------
//! Parallel branches executing tool calls used to share one mutable working
//! directory; one branch could silently overwrite another's intermediate state,
//! making wide parallel search unsafe and merge outcomes non-deterministic
//! (binds ID-2). This module gives every branch a PRIVATE workspace root, merges
//! only at an explicit deterministic barrier ordered by SSB seq, and treats an
//! absolute-path write outside a branch root as a hard BLOCK.
//!
//! Cross-platform reality (IC-9): "CoW" here is CONCEPTUAL (branch-isolation
//! semantics), NOT a filesystem-level reflink. NTFS has no reflink; ReFS/Dev
//! Drive block-cloning is not universally available. The Tier-1 cross-platform
//! default ([`WorktreePoolImpl`]) is a pool of warm `git worktree` directories
//! that share the workspace target-dir (cargo build artifacts shared, source
//! isolated per worktree). Linux overlayfs/btrfs reflink is a Tier-2 Linux-gated
//! additive impl behind the same trait; the cloud-sandbox (E2B-class) adapter is
//! an optional impl, never a hard dependency. No reflink crate is introduced.
//!
//! GIT TOPLEVEL TRAP (the single riskiest mechanical fact): `.elai_cc` is NOT a
//! standalone git repo — `git rev-parse --show-toplevel` from inside it returns
//! the PARENT workflow root. A `git worktree add` therefore registers worktrees
//! against the parent repo. [`WorktreePoolImpl`] runs git with `cwd` = the
//! resolved toplevel and places worktree roots under a scratch dir that is
//! explicitly OUTSIDE the project tree, so the project tree is never
//! contaminated. See `workflow/reasoning/I-2-decisions.md`.
//!
//! Composition (do NOT re-implement any of these — compose the seams):
//! - [`BranchWorkspace::escape_check`] delegates to
//!   `elai_safety::blast_radius::check_fs_target` (the same canonical primitive
//!   the I-1 `InProcessSupervisor` G-3 gate uses at `directive.rs:421`).
//! - [`BranchWorkspace::barrier_merge`] is invoked from the BSP barrier site
//!   (`bsp/engine.rs`) after the deferred-task phase; it does NOT re-write the
//!   barrier, it extends it.
//! - On `ConflictEscalated` the Tier-1 contract HALTS the merge (no partial
//!   write). H-49 trunk-rollback via `architecture::timetravel::materialize` is
//!   a documented Tier-2 DEFER (it needs a `FileManifest` + CAS root not
//!   available at the barrier); see the decisions doc + the stub row.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use elai_safety::blast_radius::{check_fs_target, worktree_root_blake3, BlastRadiusVerdict};
use tokio::sync::Mutex;

use crate::ssb::{BranchWorkspaceEventKind, BranchWorkspaceEventPayload, Ssb, SsbPayload};

/// Default scratch directory NAME placed under the resolved git toplevel for
/// worktree roots. Kept OUTSIDE `.elai_cc` so worktree creation never
/// contaminates the project tree (the git-toplevel trap, see module docs).
pub const WORKTREE_SCRATCH_DIR: &str = ".wt-scratch";

/// agent_id stamped on the `BranchWorkspaceEvent` SSB rows this module emits.
const WORKSPACE_EMITTER_AGENT_ID: &str = "branch_workspace";

/// Errors from the branch-workspace isolation layer. Fail-closed: every error
/// path either blocks the offending operation or surfaces a real failure; no
/// silent swallow (R1/R2).
#[derive(Debug, thiserror::Error)]
pub enum BranchWorkspaceError {
    /// A branch attempted a write whose target resolves OUTSIDE its own root.
    /// The write is blocked before it reaches the filesystem (the escape gate).
    #[error("branch '{branch_id}' escape blocked: target '{target}' escapes root '{root}'")]
    EscapeBlocked {
        branch_id: String,
        target: String,
        root: String,
    },

    /// The deterministic barrier merge found two branches writing the SAME
    /// relative path with differing bytes. Tier-1 HALTS (no partial merge);
    /// trunk rollback is the H-49 Tier-2 deferred path.
    #[error("merge conflict escalated on path '{path}' between branches '{lo}' and '{hi}'")]
    ConflictEscalated { path: String, lo: String, hi: String },

    /// A `git worktree` subcommand failed (creation/removal). Carries the git
    /// stderr so a forensic reader can see the underlying cause.
    #[error("git worktree command failed: {0}")]
    GitCommand(String),

    /// Resolving the git toplevel (`git rev-parse --show-toplevel`) failed.
    #[error("could not resolve git toplevel: {0}")]
    ToplevelResolve(String),

    /// A filesystem operation (read/copy/mkdir) failed during spawn or merge.
    #[error("workspace filesystem error: {0}")]
    Io(String),

    /// Emitting a `BranchWorkspaceEvent` SSB row failed.
    #[error("ssb write error: {0}")]
    Ssb(String),
}

/// One branch's private workspace root plus the SSB seq that orders it in the
/// deterministic barrier merge (GD-2: merge order = SSB seq, no other order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchRoot {
    /// Logical branch identifier (also the git worktree branch name).
    pub branch_id: String,
    /// Absolute path to this branch's isolated working directory.
    pub root: PathBuf,
    /// SSB seq assigned when the `Spawned` row was written. The merge orders
    /// branches by ascending seq so a same-seed re-run is byte-identical.
    pub seq: i64,
}

/// Outcome of a deterministic barrier merge.
///
/// RENAMED from `MergeResult` deliberately: `session_dag.rs:121` already exports
/// a `pub enum MergeResult` in the same crate, so this carries a distinct name
/// to avoid import ambiguity in the `orchestrator::sandbox` namespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceMergeResult {
    /// The merge target (trunk) directory the branch scratch was merged into.
    pub merged_path: PathBuf,
    /// True when the merge escalated a conflict (Tier-1: merge halted, no
    /// partial write). False on a clean last-seq-wins merge.
    pub conflict: bool,
    /// Count of files written into the merge target on a clean merge. Zero when
    /// `conflict` is true (nothing was written).
    pub files_merged: usize,
}

/// The workspace-isolation face of the agent-isolation contract.
///
/// This is NOT a replacement for `AgentSandbox` (`sandbox/mod.rs:347`, the
/// per-PROCESS isolation face) — the two COMPOSE: the I-1 Supervisor dispatches
/// a Directive that calls both an `AgentSandbox` spawn (process) and a
/// `BranchWorkspace::spawn` (files). `Debug` is a supertrait so a
/// `dyn BranchWorkspace` can live inside a `#[derive(Debug)]` config struct
/// (e.g. `BspConfig`) without a manual Debug impl.
pub trait BranchWorkspace: std::fmt::Debug + Send + Sync {
    /// Create a private workspace root for one branch agent and emit a
    /// `BranchWorkspaceEvent::Spawned` SSB row. The returned [`BranchRoot`]
    /// carries the SSB seq the row was assigned (used as the merge order).
    fn spawn(
        &self,
        branch_id: &str,
        session_id: &str,
    ) -> impl std::future::Future<Output = Result<BranchRoot, BranchWorkspaceError>> + Send;

    /// Confirm `path` stays inside `branch_root`. A target resolving outside the
    /// root is blocked fail-closed and emits a `BranchWorkspaceEvent::EscapeBlocked`
    /// row BEFORE any filesystem mutation. Delegates the containment decision to
    /// the canonical `elai_safety::blast_radius::check_fs_target` primitive (the
    /// same one the I-1 Supervisor G-3 gate uses) — it does NOT re-implement
    /// path containment.
    fn escape_check(
        &self,
        branch_root: &Path,
        branch_id: &str,
        path: &Path,
    ) -> impl std::future::Future<Output = Result<(), BranchWorkspaceError>> + Send;

    /// Merge all branch scratch roots into `merge_target`, ordered by ascending
    /// SSB seq (GD-2). Last-seq-wins on non-overlapping relative paths; two
    /// branches writing the SAME relative path with differing bytes is a
    /// `ConflictEscalated` (Tier-1 halts: no partial write). Emits `Merged` on a
    /// clean merge or `ConflictEscalated` on overlap.
    fn barrier_merge(
        &self,
        branches: &[BranchRoot],
        merge_target: &Path,
        ssb: &Ssb,
    ) -> impl std::future::Future<Output = Result<WorkspaceMergeResult, BranchWorkspaceError>> + Send;
}

/// Tier-1 cross-platform default: a warm pool of `git worktree` directories
/// sharing the workspace target-dir. Mirrors the `FirecrackerHotPool` pool shape
/// (`BTreeMap` behind a `tokio::sync::Mutex`, lazy creation, reuse on spawn,
/// prune on teardown) WITHOUT calling any Firecracker / microVM APIs.
#[derive(Clone)]
pub struct WorktreePoolImpl {
    /// branch_id -> created worktree root. Behind a `tokio::sync::Mutex` because
    /// `spawn`/`prune` run in async context (never `std::sync::Mutex` in async).
    pool: Arc<Mutex<BTreeMap<String, PathBuf>>>,
    /// Resolved git toplevel the `git worktree` subcommands run against. This is
    /// the PARENT workflow root, not `.elai_cc` (the git-toplevel trap).
    toplevel: PathBuf,
    /// Scratch base for worktree roots, under `toplevel/.wt-scratch/<session>/`.
    /// Always OUTSIDE the project source tree.
    scratch_base: PathBuf,
    /// SSB handle used to emit `BranchWorkspaceEvent` rows from `spawn` /
    /// `escape_check`. `Some` in production (wired with the engine's `Ssb` so
    /// the `Spawned` row's assigned seq is the deterministic merge-order key,
    /// GD-2); `None` only in offline/no-emit contexts where the seq is taken
    /// from a deterministic local counter. Mirrors
    /// `InProcessSupervisor::ssb` (`directive.rs:258`).
    ssb: Option<Arc<Ssb>>,
    /// Superstep stamped on emitted rows. Fixed at 0 for the workspace plane (it
    /// is barrier-scoped, not superstep-scoped); kept so the emit shape mirrors
    /// the supervisor's.
    superstep: i64,
}

// Manual Debug impl: elides the `ssb` field because `Ssb` holds unbounded-sender
// and broadcast-sender internals that intentionally have no Debug derive.
// Mirrors the `InProcessSupervisor` pattern in `directive.rs`.
impl std::fmt::Debug for WorktreePoolImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorktreePoolImpl")
            .field("toplevel", &self.toplevel)
            .field("scratch_base", &self.scratch_base)
            .field("superstep", &self.superstep)
            .field("ssb", &self.ssb.as_ref().map(|_| "<Ssb>"))
            .finish()
    }
}

impl WorktreePoolImpl {
    /// Construct a pool rooted at `toplevel` (the resolved git root) with
    /// worktree scratch under `toplevel/.wt-scratch/<session_id>/`. No SSB writer
    /// by default; chain [`with_ssb`](Self::with_ssb) to wire emission (the
    /// production dispatch path always does, so `Spawned`/`EscapeBlocked` rows
    /// land and `spawn` returns the real assigned seq).
    #[must_use]
    pub fn new(toplevel: impl Into<PathBuf>, session_id: &str) -> Self {
        let toplevel = toplevel.into();
        let scratch_base = toplevel.join(WORKTREE_SCRATCH_DIR).join(session_id);
        Self {
            pool: Arc::new(Mutex::new(BTreeMap::new())),
            toplevel,
            scratch_base,
            ssb: None,
            superstep: 0,
        }
    }

    /// Install the SSB writer used to emit `BranchWorkspaceEvent` rows from
    /// `spawn`/`escape_check`. The production dispatch path wires the engine's
    /// `Ssb` here so a `Spawned` row's assigned seq becomes the merge-order key.
    #[must_use]
    pub fn with_ssb(mut self, ssb: Arc<Ssb>) -> Self {
        self.ssb = Some(ssb);
        self
    }

    /// Resolve the git toplevel by running `git rev-parse --show-toplevel` with
    /// `cwd` = `anchor`. Returns the PARENT workflow root when `anchor` is inside
    /// `.elai_cc` (the git-toplevel trap). Used by the production dispatch path
    /// so worktree creation targets the real repo, never `.elai_cc`.
    pub fn resolve_toplevel(anchor: &Path) -> Result<PathBuf, BranchWorkspaceError> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .current_dir(anchor)
            .output()
            .map_err(|e| BranchWorkspaceError::ToplevelResolve(e.to_string()))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            return Err(BranchWorkspaceError::ToplevelResolve(stderr));
        }
        let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if path.is_empty() {
            return Err(BranchWorkspaceError::ToplevelResolve(
                "git returned an empty toplevel".to_owned(),
            ));
        }
        Ok(PathBuf::from(path))
    }

    /// Remove all worktrees this pool created (`git worktree remove --force`).
    /// On Windows a worktree removal can fail with access-denied if a process
    /// still holds an open handle inside it; this is a KNOWN LIMIT with a
    /// bounded retry and a `tracing::warn` on persistent failure (NOT a panic,
    /// NOT a silent swallow — a leaked scratch worktree is non-fatal and visible
    /// in the log). See decisions doc "Windows worktree-removal race".
    pub async fn prune(&self) {
        let entries: Vec<(String, PathBuf)> = {
            let pool = self.pool.lock().await;
            pool.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        };
        for (branch_id, root) in entries {
            let mut removed = false;
            for attempt in 0..WORKTREE_REMOVE_RETRIES {
                let output = Command::new("git")
                    .arg("worktree")
                    .arg("remove")
                    .arg("--force")
                    .arg(&root)
                    .current_dir(&self.toplevel)
                    .output();
                match output {
                    Ok(o) if o.status.success() => {
                        removed = true;
                        break;
                    }
                    Ok(o) => {
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        tracing::warn!(
                            branch_id = %branch_id,
                            root = %root.display(),
                            attempt,
                            stderr = %stderr,
                            "branch_workspace: git worktree remove failed (retrying)"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            branch_id = %branch_id,
                            root = %root.display(),
                            attempt,
                            error = %e,
                            "branch_workspace: git worktree remove errored (retrying)"
                        );
                    }
                }
            }
            if !removed {
                tracing::warn!(
                    branch_id = %branch_id,
                    root = %root.display(),
                    "branch_workspace: git worktree remove exhausted retries; leaking scratch worktree (non-fatal, Windows handle race)"
                );
            }
        }
        self.pool.lock().await.clear();
    }
}

/// Bounded retry count for `git worktree remove` on Windows handle races.
const WORKTREE_REMOVE_RETRIES: u32 = 3;

impl BranchWorkspace for WorktreePoolImpl {
    async fn spawn(
        &self,
        branch_id: &str,
        session_id: &str,
    ) -> Result<BranchRoot, BranchWorkspaceError> {
        let root = self.scratch_base.join(branch_id);

        // Ensure the scratch base exists. mkdir -p semantics; failure is fatal
        // (we cannot isolate a branch without its root) and surfaced as Io.
        if let Some(parent) = root.parent() {
            std::fs::create_dir_all(parent).map_err(|e| BranchWorkspaceError::Io(e.to_string()))?;
        }

        // `git worktree add <root> -b <branch_id>` against the resolved toplevel
        // (NOT .elai_cc). std::process::Command per the production git idiom
        // (agent_template.rs:332); no git2 in production paths (git2 is a
        // contracts/tests-only dev-dep).
        let output = Command::new("git")
            .arg("worktree")
            .arg("add")
            .arg(&root)
            .arg("-b")
            .arg(branch_id)
            .current_dir(&self.toplevel)
            .output()
            .map_err(|e| BranchWorkspaceError::GitCommand(e.to_string()))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            return Err(BranchWorkspaceError::GitCommand(stderr));
        }

        {
            let mut pool = self.pool.lock().await;
            pool.insert(branch_id.to_owned(), root.clone());
        }

        // Emit the Spawned row; its assigned seq is the deterministic merge
        // order key (GD-2). `write_payload` returns the assigned seq. When no
        // SSB is wired (offline/no-emit context) fall back to the pool size as a
        // deterministic monotonic local counter so ordering is still stable.
        let payload = SsbPayload::BranchWorkspaceEvent(BranchWorkspaceEventPayload {
            kind: BranchWorkspaceEventKind::Spawned,
            branch_id: branch_id.to_owned(),
            session_id: session_id.to_owned(),
            worktree_root_blake3: worktree_root_blake3(&root),
            conflict_path: None,
        });
        let seq = match &self.ssb {
            Some(ssb) => ssb
                .write_payload(WORKSPACE_EMITTER_AGENT_ID, self.superstep, payload, vec![])
                .await
                .map_err(|e| BranchWorkspaceError::Ssb(e.to_string()))?,
            None => self.pool.lock().await.len() as i64,
        };

        Ok(BranchRoot {
            branch_id: branch_id.to_owned(),
            root,
            seq,
        })
    }

    async fn escape_check(
        &self,
        branch_root: &Path,
        branch_id: &str,
        path: &Path,
    ) -> Result<(), BranchWorkspaceError> {
        // Delegate the containment decision to the canonical primitive (the same
        // one InProcessSupervisor's G-3 gate uses). pre_expansion is the raw path
        // string for the forensic SSB field; the gate canonicalises internally.
        let pre = path.to_string_lossy().into_owned();
        match check_fs_target(path, branch_root, &pre) {
            BlastRadiusVerdict::Allow => Ok(()),
            BlastRadiusVerdict::Block { post_expansion, .. } => {
                // Escape: emit EscapeBlocked BEFORE returning the block (no FS
                // mutation has happened — the caller checks before writing).
                let payload = SsbPayload::BranchWorkspaceEvent(BranchWorkspaceEventPayload {
                    kind: BranchWorkspaceEventKind::EscapeBlocked,
                    branch_id: branch_id.to_owned(),
                    session_id: String::new(),
                    worktree_root_blake3: worktree_root_blake3(branch_root),
                    conflict_path: Some(post_expansion.clone()),
                });
                // Best-effort emit; the BLOCK is the load-bearing outcome, so an
                // SSB write failure must not convert a block into an allow. Log
                // and still return the block. (No SSB wired => no row; the block
                // is still returned, which is the fail-closed guarantee.)
                if let Some(ssb) = &self.ssb {
                    if let Err(e) = ssb
                        .write_payload(WORKSPACE_EMITTER_AGENT_ID, self.superstep, payload, vec![])
                        .await
                    {
                        tracing::warn!(error = %e, "branch_workspace: EscapeBlocked SSB emit failed (still blocking)");
                    }
                }
                Err(BranchWorkspaceError::EscapeBlocked {
                    branch_id: branch_id.to_owned(),
                    target: post_expansion,
                    root: branch_root.to_string_lossy().into_owned(),
                })
            }
        }
    }

    async fn barrier_merge(
        &self,
        branches: &[BranchRoot],
        merge_target: &Path,
        ssb: &Ssb,
    ) -> Result<WorkspaceMergeResult, BranchWorkspaceError> {
        // GD-2: order strictly by ascending SSB seq (then branch_id as a stable
        // tiebreak so identical-seq inputs still produce one deterministic
        // order). Same-seed re-run => byte-identical merged tree.
        let mut ordered: Vec<&BranchRoot> = branches.iter().collect();
        ordered.sort_by(|a, b| a.seq.cmp(&b.seq).then_with(|| a.branch_id.cmp(&b.branch_id)));

        // Track which branch last wrote each relative path and the bytes it
        // wrote, so an OVERLAP (same path, differing bytes) escalates instead of
        // silently last-seq-winning over a real divergence. First conflict wins
        // the detail (deterministic: ordered + BTreeMap give a stable scan).
        //
        // `branch_count`: how many branches contributed each relative path. Used
        // post-loop to distinguish HEAD-baseline files (present in ALL N branches
        // with identical bytes — they came from the initial worktree checkout, not
        // from branch work) from delta files (present in fewer than N branches, or
        // with differing bytes). `files_merged` counts ONLY delta files per the
        // plan semantics (NB-1 fix).
        let mut winner: BTreeMap<PathBuf, (String, Vec<u8>)> = BTreeMap::new();
        let mut branch_count: BTreeMap<PathBuf, usize> = BTreeMap::new();
        let mut conflict_detail: Option<(PathBuf, String, String)> = None;
        let branch_total = ordered.len();

        for branch in &ordered {
            let files = collect_rel_files(&branch.root)?;
            for rel in files {
                let abs = branch.root.join(&rel);
                // Defense-in-depth: confirm the collected file actually resolves
                // INSIDE the branch root before staging it for trunk (a symlink
                // inside the worktree could otherwise smuggle out-of-root bytes
                // into the merge). Same canonical containment primitive as the
                // spawn-time gate; emits EscapeBlocked + returns Err on a breach.
                self.escape_check(&branch.root, &branch.branch_id, &abs).await?;
                let bytes = std::fs::read(&abs).map_err(|e| BranchWorkspaceError::Io(e.to_string()))?;
                *branch_count.entry(rel.clone()).or_insert(0) += 1;
                match winner.get(&rel) {
                    Some((prev_branch, prev_bytes)) if prev_bytes != &bytes => {
                        // Two branches wrote the SAME path with DIFFERING bytes:
                        // a real conflict. Record the first; Tier-1 halts the merge.
                        if conflict_detail.is_none() {
                            conflict_detail = Some((
                                rel.clone(),
                                prev_branch.clone(),
                                branch.branch_id.clone(),
                            ));
                        }
                    }
                    _ => {
                        // First writer, or an identical re-write (idempotent):
                        // last-seq-wins. ordered ascending => later seq overwrites.
                        winner.insert(rel.clone(), (branch.branch_id.clone(), bytes));
                    }
                }
            }
        }

        if let Some((path, lo, hi)) = conflict_detail {
            let path_str = path.to_string_lossy().into_owned();
            // Emit ConflictEscalated; Tier-1 HALTS (no partial write to trunk).
            // H-49 materialize() trunk-rollback is the Tier-2 deferred path.
            let payload = SsbPayload::BranchWorkspaceEvent(BranchWorkspaceEventPayload {
                kind: BranchWorkspaceEventKind::ConflictEscalated,
                branch_id: hi.clone(),
                session_id: String::new(),
                worktree_root_blake3: worktree_root_blake3(merge_target),
                conflict_path: Some(path_str.clone()),
            });
            ssb.write_payload(WORKSPACE_EMITTER_AGENT_ID, self.superstep, payload, vec![])
                .await
                .map_err(|e| BranchWorkspaceError::Ssb(e.to_string()))?;
            return Err(BranchWorkspaceError::ConflictEscalated {
                path: path_str,
                lo,
                hi,
            });
        }

        // Clean merge: write each winner into the merge target atomically-ish
        // (mkdir parents + write). Deterministic because `winner` is a BTreeMap
        // (sorted key iteration) and the winning bytes were chosen by seq order.
        //
        // files_merged counts ONLY delta files — files contributed by branch work,
        // not the HEAD-baseline files that were present in ALL N branch worktrees
        // with identical bytes (those came from the initial worktree checkout, not
        // from any branch's edits). A file is a baseline file iff its branch_count
        // == branch_total (every branch had it, meaning it came from HEAD). A file
        // present in fewer branches is a new file a branch added; it is a delta.
        let mut files_merged = 0usize;
        for (rel, (_branch, bytes)) in &winner {
            let dest = merge_target.join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| BranchWorkspaceError::Io(e.to_string()))?;
            }
            std::fs::write(&dest, bytes).map_err(|e| BranchWorkspaceError::Io(e.to_string()))?;
            // Only count as a merged delta if not every branch had this file
            // (i.e., it is not a pure HEAD-baseline carry-through).
            let count = branch_count.get(rel).copied().unwrap_or(0);
            if count < branch_total {
                files_merged += 1;
            }
        }

        let payload = SsbPayload::BranchWorkspaceEvent(BranchWorkspaceEventPayload {
            kind: BranchWorkspaceEventKind::Merged,
            branch_id: ordered.last().map(|b| b.branch_id.clone()).unwrap_or_default(),
            session_id: String::new(),
            worktree_root_blake3: worktree_root_blake3(merge_target),
            conflict_path: None,
        });
        ssb.write_payload(WORKSPACE_EMITTER_AGENT_ID, self.superstep, payload, vec![])
            .await
            .map_err(|e| BranchWorkspaceError::Ssb(e.to_string()))?;

        Ok(WorkspaceMergeResult {
            merged_path: merge_target.to_path_buf(),
            conflict: false,
            files_merged,
        })
    }
}

/// Recursively collect every FILE under `root` as a path RELATIVE to `root`,
/// skipping the `.git` worktree-pointer file/dir (a git worktree root contains a
/// `.git` file that points back to the parent repo; it is not branch content and
/// must never be merged into trunk). Sorted deterministically via `BTreeSet`
/// before return so callers get a stable order.
fn collect_rel_files(root: &Path) -> Result<Vec<PathBuf>, BranchWorkspaceError> {
    let mut out: BTreeSet<PathBuf> = BTreeSet::new();
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| BranchWorkspaceError::Io(e.to_string()))?;
        for entry in entries {
            let entry = entry.map_err(|e| BranchWorkspaceError::Io(e.to_string()))?;
            let path = entry.path();
            let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned());
            // Skip the worktree's `.git` pointer (file in a worktree, dir in a
            // primary checkout) — never branch content.
            if file_name.as_deref() == Some(".git") {
                continue;
            }
            let ft = entry.file_type().map_err(|e| BranchWorkspaceError::Io(e.to_string()))?;
            if ft.is_dir() {
                stack.push(path);
            } else if ft.is_file() {
                if let Ok(rel) = path.strip_prefix(root) {
                    out.insert(rel.to_path_buf());
                }
            }
        }
    }
    Ok(out.into_iter().collect())
}
