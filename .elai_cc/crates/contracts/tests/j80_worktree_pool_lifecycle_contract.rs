//! J-80 warm worktree-pool lifecycle contract.
use std::path::Path;
use elai_orchestrator::sandbox::{BranchWorkspace, WorktreePoolImpl};
use tempfile::TempDir;

fn init_repo(dir: &Path) {
    let repo = git2::Repository::init(dir).expect("git init");
    let sig = git2::Signature::now("j80-contract", "j80@example.invalid").expect("signature");
    std::fs::write(dir.join("seed.txt"), "seed\n").expect("seed");
    let mut index = repo.index().expect("index");
    index.add_path(Path::new("seed.txt")).expect("add seed");
    index.write().expect("write index");
    let tree_id = index.write_tree().expect("tree");
    let tree = repo.find_tree(tree_id).expect("find tree");
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .expect("commit");
}

#[tokio::test]
async fn warm_reuse_heals_vanished_entries_and_keeps_roots_distinct() {
    let repo = TempDir::new().expect("repo tempdir");
    init_repo(repo.path());
    let pool = WorktreePoolImpl::new(repo.path(), "j80");

    let first = pool.spawn("same-node", "j80").await.expect("first spawn");
    let second = pool.spawn("same-node", "j80").await.expect("warm spawn");
    assert_eq!(first.root, second.root, "warm reuse must return the same root");

    let vanished = pool.spawn("vanished-node", "j80").await.expect("vanished spawn");
    std::fs::remove_dir_all(&vanished.root).expect("remove vanished root");
    let healed = pool.spawn("new-node", "j80").await.expect("heal and spawn");
    assert_ne!(healed.root, vanished.root, "vanished entry must not be handed back");

    let other = pool.spawn("other-node", "j80").await.expect("other spawn");
    assert_ne!(first.root, other.root, "distinct branch ids require distinct roots");
    pool.prune().await;
}
