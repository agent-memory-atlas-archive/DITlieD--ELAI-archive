Phase 3J.14 WIRE.E2E decisions

What: Wire end-to-end pipeline — sidecar fallback to ollama, BM25 retrieval injection, release build, banner update.

Key decisions:
- fallback_url: Option<String> on BackendConfig (not a new BackendMode variant) — keeps enum locked.
- Ollama probe via reqwest GET to localhost:11434/api/tags with 2s timeout — lightweight health check.
- QueryPipeline::query() is sync, run_turn() is sync — no spawn_blocking needed.
- BM25 update_file called with chunks=None for auto-chunking after file tool use.
- Knowledge pipeline results injected into system_prompt vec (Slot 4 CodeContext) before runtime.run_turn().
- Named constants: OLLAMA_DEFAULT_URL, DEGRADED_PROMPT_PREFIX, KNOWLEDGE_TOP_K.

Expected test outcomes:
- cargo check -p elai-cli -p elai-orchestrator: 0 errors after each sub-task.
- Existing dispatcher tests pass (..Default::default() covers new field).
- cargo test -p elai-cli: existing tests pass, no regressions.

Unknowns:
- CatRag and AragOrchestrator default configs — using Default::default() if available, else explicit minimal config.
- Whether file tool use emits a path accessible from run_turn — may need to hook at a different layer.
