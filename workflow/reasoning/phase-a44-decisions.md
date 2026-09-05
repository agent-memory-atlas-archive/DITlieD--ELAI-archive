Phase A-44: PROJECT INDEXING WIRING decisions

What (4 deliverables):
1. orchestrator/Cargo.toml: add elai-knowledge path dependency
2. crates/orchestrator/src/indexing_service.rs: IndexingService struct with run_full_index, reindex_file, indexes_exist, needs_rebuild
3. crates/cli/src/main.rs: wire IndexingService into new_local (first-run check), startup_banner (progress), notify_file_changed (incremental reindex)
4. crates/contracts/tests/indexing_service_a44_contract.rs: contract test for public interface

Key decisions:
- Degraded-mode contract: run_full_index failure logs tracing::warn and CLI continues with no index. Call site uses match, not ? propagation.
- SSB wiring point: reindex_file called from notify_file_changed after existing bm25.update_file call. Reads from existing SSB tokio::mpsc channel, no new channel.
- check_and_upgrade usage: reuse knowledge::index_version::check_and_upgrade for schema version gate in needs_rebuild. Do not reimplement.
- spawn_blocking: run_full_index wraps DKG build_graph and BM25 indexing in tokio::task::spawn_blocking since they are CPU/IO heavy.
- Binary detection: read first 8192 bytes, skip file if null byte found. Named constant BINARY_DETECT_BYTES.
- Exclusion list: hardcoded defaults (node_modules, target, .git, __pycache__, dist, build, .env, *.key, *.pem, credentials.*, secrets.*). Configurable via elai_cc.toml [knowledge.exclude].
- Error type: IndexingServiceError enum with thiserror, variants for Knowledge, IndexVersion, Io.

Expected test outcome:
- Contract test defines IndexingService interface, compiles, exercises new/indexes_exist/run_full_index
- cargo check --workspace passes
- cargo test -p elai-orchestrator passes (new tests)

Unknowns:
- Exact DkgEngine::new and Bm25Index::new signatures (must verify V1-V6 before coding)
- Whether SSB notify_file_changed exists in current cli/main.rs or needs to be added
- Whether orchestrator already has any knowledge-related code
