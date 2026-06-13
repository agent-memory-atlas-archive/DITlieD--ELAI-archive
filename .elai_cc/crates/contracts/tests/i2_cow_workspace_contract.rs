//! I-2 CoW workspace isolation contract.
//!
//! Parallel branches used to share one mutable working directory (ID-2 hazard).
//! I-2 gives every branch a PRIVATE workspace root (Tier-1: warm git worktree),
//! merges only at an explicit deterministic barrier ordered by SSB seq (GD-2),
//! and treats an absolute-path write outside a branch root as a hard BLOCK.
//!
//! Arms (all fail-closed, R1 missing-evidence = FAIL):
//!   (a) DISJOINT-ROOTS: two spawned branches get non-overlapping workspace
//!       roots (neither is a path-prefix of the other), and each spawn lands a
//!       `BranchWorkspaceEvent::Spawned` SSB row (off-agent evidence, R4).
//!   (b) DETERMINISTIC-MERGE: the SSB-seq-ordered barrier merge of the same two
//!       branch roots into two separate trunk dirs produces BYTE-IDENTICAL
//!       output (GD-2 same-seed re-run determinism), and a `Merged` row lands.
//!   (c) ESCAPE-BLOCKED: a branch writing an ABSOLUTE path OUTSIDE its root is
//!       blocked by `escape_check` (returns `EscapeBlocked`) BEFORE any FS
//!       mutation, and an `EscapeBlocked` SSB row lands.
//!   (d) CONFLICT-ESCALATED: two branches writing the SAME relative path with
//!       DIFFERING bytes escalate to `ConflictEscalated` (the merge HALTS, it is
//!       NOT a clean merge) and a `ConflictEscalated` SSB row lands; the trunk is
//!       NOT partially written.
//!   (e) PAYLOAD + SCHEMA-LOCK: a `BranchWorkspaceEvent` payload round-trips
//!       through the live rmp_serde path to the right Kind arm; the action_type
//!       string bijection holds; `SCHEMA_VERSION_PUBLIC` is at the I-2 floor and
//!       `action_type_lock.rs` declares a `TOTAL_VARIANTS` at/above the I-2 floor.
//!
//! "CoW" here is CONCEPTUAL (branch-isolation semantics), NOT a filesystem-level
//! reflink — NTFS has no reflink; Tier-1 is git-worktree directory isolation. No
//! reflink crate is involved. See workflow/reasoning/I-2-decisions.md.
//!
//! Spec references: docs/thefinalglance/I-plan/I-2_cow_workspace_isolation.md
//! Reads (mandatory): wiring-enforcement.md R1/R4/R7; .claude/rules/maintainability-review.md

use std::path::Path;
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::sandbox::{BranchWorkspace, WorktreePoolImpl};
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, BranchWorkspaceEventKind,
    BranchWorkspaceEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

/// I-2 SCHEMA floor: the BranchWorkspaceEvent variant shipped at SCHEMA_VERSION
/// 246 / TOTAL_VARIANTS 555. Floors (not exact) per the pin doctrine — only
/// e45_cross_language_drift_contract holds the sole exact-equality lock.
const I2_SCHEMA_FLOOR: i64 = 246;
const I2_TOTAL_VARIANTS_FLOOR: usize = 555;

/// Filter `query_by_type(BranchWorkspaceEvent(_))` rows down to a specific
/// [`BranchWorkspaceEventKind`] arm by decoding each row's payload.
///
/// WHY THIS EXISTS: `query_by_type` queries by the SQL `action_type` string,
/// which is `"branch_workspace_event"` for ALL arms of the Pattern-2
/// `BranchWorkspaceEvent` variant (the Kind discriminator lives in the
/// MessagePack payload, not in the DB column). Without this filter the query
/// returns every BranchWorkspaceEvent row regardless of arm (Spawned, Merged,
/// EscapeBlocked, ConflictEscalated), causing false positives when asserting
/// arm-specific row counts. Decoding and filtering by Kind is the correct
/// approach for Pattern-2 multi-arm variants.
fn filter_bwse_kind(
    rows: Vec<elai_orchestrator::ssb::SsbEntry>,
    want: BranchWorkspaceEventKind,
) -> Vec<elai_orchestrator::ssb::SsbEntry> {
    rows.into_iter()
        .filter(|entry| {
            decode_ssb_payload(&entry.payload)
                .ok()
                .and_then(|p| match p {
                    SsbPayload::BranchWorkspaceEvent(ref inner) => Some(inner.kind == want),
                    _ => None,
                })
                .unwrap_or(false)
        })
        .collect()
}

/// Build a real git repo in `dir` with one initial commit so `git worktree add`
/// (which requires a HEAD to branch from) succeeds. Uses git2 only as a test
/// fixture (the contracts-crate dev-dep at Cargo.toml:94), matching the
/// `tool_dispatcher_contract.rs:292` pattern; production paths use
/// std::process::Command, never git2.
fn init_repo_with_commit(dir: &Path) {
    let repo = git2::Repository::init(dir).expect("git init");
    // git2 needs an explicit signature; CI envs may lack a global git identity.
    let sig = git2::Signature::now("i2-contract", "i2@example.invalid").expect("signature");
    let file = dir.join("seed.txt");
    std::fs::write(&file, "seed\n").expect("write seed file");
    let mut index = repo.index().expect("repo index");
    index.add_path(Path::new("seed.txt")).expect("add seed.txt");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("write tree");
    let tree = repo.find_tree(tree_id).expect("find tree");
    repo.commit(Some("HEAD"), &sig, &sig, "i2 seed commit", &tree, &[])
        .expect("initial commit");
}

/// Open a throwaway SSB in `dir` for off-agent row assertions (R1/R4).
fn open_ssb(dir: &Path) -> Arc<Ssb> {
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let cfg = SsbConfig {
        db_path: dir.join("ssb.db"),
        pool_size: Some(2),
        project_id: "i2-contract".into(),
        session_id: "i2-session".into(),
    };
    Arc::new(Ssb::new(cfg, clock).expect("ssb open"))
}

// ---------------------------------------------------------------------------
// (a) DISJOINT-ROOTS: two branches get non-overlapping roots + Spawned rows.
// ---------------------------------------------------------------------------
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn disjoint_roots_verified() {
    let repo_dir = TempDir::new().expect("repo tempdir");
    init_repo_with_commit(repo_dir.path());
    let ssb = open_ssb(repo_dir.path());

    let pool = WorktreePoolImpl::new(repo_dir.path(), "i2-disjoint").with_ssb(ssb.clone());
    let a = pool.spawn("branch-a", "i2-session").await.expect("spawn a");
    let b = pool.spawn("branch-b", "i2-session").await.expect("spawn b");

    // Roots must differ and NEITHER may be a path-prefix of the other (true
    // isolation: a write under one root can never land inside the other).
    assert_ne!(a.root, b.root, "two branches must get distinct roots");
    assert!(
        !a.root.starts_with(&b.root) && !b.root.starts_with(&a.root),
        "branch roots must not nest: a={:?} b={:?}",
        a.root,
        b.root
    );
    // Both worktree roots must actually exist on disk (git worktree add ran).
    assert!(a.root.is_dir(), "branch-a worktree root must exist");
    assert!(b.root.is_dir(), "branch-b worktree root must exist");

    // R4 off-agent evidence: two Spawned rows landed.
    let rows = ssb
        .query_by_type(SsbActionType::BranchWorkspaceEvent(
            BranchWorkspaceEventKind::Spawned,
        ))
        .expect("query Spawned rows");
    assert_eq!(rows.len(), 2, "exactly two Spawned rows must land (one per spawn)");

    pool.prune().await;
}

// ---------------------------------------------------------------------------
// (b) DETERMINISTIC-MERGE: same branch roots merged into two trunks => identical.
// ---------------------------------------------------------------------------
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn deterministic_merge_byte_identical() {
    let repo_dir = TempDir::new().expect("repo tempdir");
    init_repo_with_commit(repo_dir.path());
    let ssb = open_ssb(repo_dir.path());

    let pool = WorktreePoolImpl::new(repo_dir.path(), "i2-merge").with_ssb(ssb.clone());
    let a = pool.spawn("merge-a", "i2-session").await.expect("spawn a");
    let b = pool.spawn("merge-b", "i2-session").await.expect("spawn b");

    // Each branch writes a DISTINCT relative file (no overlap => clean merge).
    std::fs::write(a.root.join("from_a.txt"), b"alpha\n").expect("write a");
    std::fs::write(b.root.join("from_b.txt"), b"beta\n").expect("write b");

    let trunk1 = TempDir::new().expect("trunk1");
    let trunk2 = TempDir::new().expect("trunk2");

    let branches = [a.clone(), b.clone()];
    let m1 = pool
        .barrier_merge(&branches, trunk1.path(), &ssb)
        .await
        .expect("merge into trunk1");
    let m2 = pool
        .barrier_merge(&branches, trunk2.path(), &ssb)
        .await
        .expect("merge into trunk2");

    assert!(!m1.conflict && !m2.conflict, "clean disjoint merge must not conflict");
    assert_eq!(m1.files_merged, 2, "both files merged into trunk1");
    assert_eq!(m2.files_merged, 2, "both files merged into trunk2");

    // GD-2: the two independent merges of the same seq-ordered inputs are
    // byte-identical for every merged file.
    for rel in ["from_a.txt", "from_b.txt"] {
        let b1 = std::fs::read(trunk1.path().join(rel)).expect("read trunk1 file");
        let b2 = std::fs::read(trunk2.path().join(rel)).expect("read trunk2 file");
        assert_eq!(b1, b2, "merged file {rel} must be byte-identical across re-runs (GD-2)");
    }

    // R4 off-agent evidence: at least one Merged row landed.
    let rows = ssb
        .query_by_type(SsbActionType::BranchWorkspaceEvent(
            BranchWorkspaceEventKind::Merged,
        ))
        .expect("query Merged rows");
    assert!(!rows.is_empty(), "a Merged row must land on a clean barrier merge");

    pool.prune().await;
}

// ---------------------------------------------------------------------------
// (c) ESCAPE-BLOCKED: an absolute path outside the root is blocked pre-mutation.
// ---------------------------------------------------------------------------
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn escape_fixture_blocked() {
    let repo_dir = TempDir::new().expect("repo tempdir");
    init_repo_with_commit(repo_dir.path());
    let ssb = open_ssb(repo_dir.path());

    let pool = WorktreePoolImpl::new(repo_dir.path(), "i2-escape").with_ssb(ssb.clone());
    let a = pool.spawn("escape-a", "i2-session").await.expect("spawn a");

    // A sibling dir OUTSIDE the branch root. The escape check must reject a
    // write targeting it BEFORE any filesystem mutation happens.
    let outside = TempDir::new().expect("outside dir");
    let escape_target = outside.path().join("victim.txt");
    assert!(
        !escape_target.exists(),
        "precondition: escape target must not exist before the check"
    );

    let verdict = pool
        .escape_check(&a.root, "escape-a", &escape_target)
        .await;
    assert!(
        verdict.is_err(),
        "a write outside the branch root must be blocked (EscapeBlocked)"
    );
    // The block must not have created the file (pre-FS-mutation guarantee).
    assert!(
        !escape_target.exists(),
        "EscapeBlocked must fire BEFORE any FS mutation; the file must not exist"
    );

    // An IN-root path must pass the same gate (false-positive guard: a legit
    // write is not blocked).
    let in_root = a.root.join("legit.txt");
    pool.escape_check(&a.root, "escape-a", &in_root)
        .await
        .expect("an in-root write must pass escape_check");

    // R4 off-agent evidence: exactly one EscapeBlocked row landed.
    // Note: query_by_type returns ALL branch_workspace_event rows (Pattern-2
    // variant — the arm discriminator is in the payload, not the DB column).
    // filter_bwse_kind decodes the payload and keeps only EscapeBlocked rows.
    let rows = filter_bwse_kind(
        ssb.query_by_type(SsbActionType::BranchWorkspaceEvent(
            BranchWorkspaceEventKind::EscapeBlocked,
        ))
        .expect("query branch_workspace_event rows"),
        BranchWorkspaceEventKind::EscapeBlocked,
    );
    assert_eq!(rows.len(), 1, "exactly one EscapeBlocked row must land");

    pool.prune().await;
}

// ---------------------------------------------------------------------------
// (d) CONFLICT-ESCALATED: same path, differing bytes => merge halts (not merged).
// ---------------------------------------------------------------------------
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn conflict_escalated_fixture() {
    let repo_dir = TempDir::new().expect("repo tempdir");
    init_repo_with_commit(repo_dir.path());
    let ssb = open_ssb(repo_dir.path());

    let pool = WorktreePoolImpl::new(repo_dir.path(), "i2-conflict").with_ssb(ssb.clone());
    let a = pool.spawn("conflict-a", "i2-session").await.expect("spawn a");
    let b = pool.spawn("conflict-b", "i2-session").await.expect("spawn b");

    // BOTH branches write the SAME relative path with DIFFERING bytes: a real
    // overlap that must escalate, not silently last-seq-win over divergence.
    std::fs::write(a.root.join("shared.rs"), b"fn from_a() {}\n").expect("write a");
    std::fs::write(b.root.join("shared.rs"), b"fn from_b() {}\n").expect("write b");

    let trunk = TempDir::new().expect("trunk");
    let branches = [a.clone(), b.clone()];
    let result = pool.barrier_merge(&branches, trunk.path(), &ssb).await;

    assert!(
        result.is_err(),
        "an overlapping-hunk conflict must escalate (ConflictEscalated), not merge"
    );
    // Tier-1 halts: the trunk must NOT have been partially written.
    assert!(
        !trunk.path().join("shared.rs").exists(),
        "ConflictEscalated must halt the merge; no partial trunk write"
    );

    // R4 off-agent evidence: a ConflictEscalated row landed; NO Merged row did.
    // Note: query_by_type returns ALL branch_workspace_event rows (Pattern-2
    // variant — the arm discriminator is in the payload, not the DB column).
    // filter_bwse_kind decodes the payload and keeps only the requested arm.
    let all_bwse = ssb
        .query_by_type(SsbActionType::BranchWorkspaceEvent(
            BranchWorkspaceEventKind::Spawned,
        ))
        .expect("query branch_workspace_event rows");
    let conflicts =
        filter_bwse_kind(all_bwse.clone(), BranchWorkspaceEventKind::ConflictEscalated);
    assert!(!conflicts.is_empty(), "a ConflictEscalated row must land");
    let merged = filter_bwse_kind(all_bwse, BranchWorkspaceEventKind::Merged);
    assert!(merged.is_empty(), "no Merged row may land when the merge conflicts");

    pool.prune().await;
}

// ---------------------------------------------------------------------------
// (e) PAYLOAD round-trip + SCHEMA floor (pin doctrine: floors, not exact).
// ---------------------------------------------------------------------------
#[test]
fn branch_workspace_event_payload_round_trips_and_schema_floor_holds() {
    // Payload round-trip through the live rmp_serde encode/decode path.
    let payload = SsbPayload::BranchWorkspaceEvent(BranchWorkspaceEventPayload {
        kind: BranchWorkspaceEventKind::EscapeBlocked,
        branch_id: "rt-branch".into(),
        session_id: "rt-session".into(),
        worktree_root_blake3: "0123456789abcdef".into(),
        conflict_path: Some("/etc/shadow".into()),
    });
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode payload");
    let decoded = decode_ssb_payload(&bytes).expect("decode payload");
    match decoded {
        SsbPayload::BranchWorkspaceEvent(ref p) => {
            assert_eq!(p.kind, BranchWorkspaceEventKind::EscapeBlocked, "Kind arm round-trips");
            assert_eq!(p.branch_id, "rt-branch", "branch_id round-trips");
            assert_eq!(p.conflict_path.as_deref(), Some("/etc/shadow"), "conflict_path round-trips");
        }
        other => panic!("decoded to the wrong variant: {other:?}"),
    }
    assert_eq!(
        payload.action_type(),
        SsbActionType::BranchWorkspaceEvent(BranchWorkspaceEventKind::EscapeBlocked),
        "payload.action_type() maps to the BranchWorkspaceEvent variant"
    );

    // action_type string bijection (read-side consumers).
    let at = ssb_action_type_from_str_public("branch_workspace_event")
        .expect("from_str branch_workspace_event must succeed");
    assert_eq!(
        at,
        SsbActionType::BranchWorkspaceEvent(BranchWorkspaceEventKind::Spawned),
        "from_str returns the Spawned default arm"
    );
    assert_eq!(at.as_sql_str(), "branch_workspace_event", "as_sql_str round-trip");

    // SCHEMA floor (NOT exact; e45 holds the sole exact lock).
    assert!(
        SCHEMA_VERSION_PUBLIC >= I2_SCHEMA_FLOOR,
        "SCHEMA_VERSION_PUBLIC must be >= {I2_SCHEMA_FLOOR} (I-2 floor); got {SCHEMA_VERSION_PUBLIC}"
    );

    // TOTAL_VARIANTS floor from the runtime registry. Tier-B Move 2 derived
    // TOTAL_VARIANTS and relocated ALL_VARIANTS (now SSB_ACTION_TYPES in
    // ssb.rs), so read the live registry length. Same I-2 floor.
    let tv_value = elai_orchestrator::ssb::SSB_ACTION_TYPES.len();
    assert!(
        tv_value >= I2_TOTAL_VARIANTS_FLOOR,
        "TOTAL_VARIANTS must be >= {I2_TOTAL_VARIANTS_FLOOR} (I-2 floor); got {tv_value}"
    );
}
