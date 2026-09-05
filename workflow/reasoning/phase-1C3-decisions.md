Phase 1C Step 3 decisions

what: Write 21+ failing unit tests + 2 contract tests for vector module (TDD step)
key decision: mock injection via trait + constructor, not global state
expected test outcome: all tests compile (cargo test --no-run = 0), all fail at runtime (expected for TDD)
unknowns: none — all interfaces verified from source

decisions:
- MockEmbedSession: define EmbedSession trait in #[cfg(test)] mod tests of mod.rs. MockEmbedSession holds closures for embed_code/embed_text. Injected via constructor.
- HalfOpen observable: add sync_state(&mut self) to CircuitBreaker — checks elapsed time and transitions Open->HalfOpen without executing a closure. Test scaffolding.
- Cache invalidation: use invalidate_keys() with content_hash()-computed keys. invalidate_file() is NotSupported.
- Concurrency: std::thread::spawn (no tokio in dev-deps). Arc<Mutex<OrtSession>> shared across threads.
- Truncation warning: flag-based mock (no tracing-test in dev-deps). MockEmbedSession records truncation flag.
- HNSW tests: gated with #[cfg(feature="usearch-hnsw")] for Windows MSVC compat.
- ReadinessStatus: uses Degraded variant (no BM25_ONLY in enum).
- RRF formula: k=60, 3 rankers, rrf_score(doc) = sum(1/(60+rank)). f32 precision assert via abs diff < 1e-6.
- Contract tests: compile-time type assertions + 2 runtime tests. Physical isolation via separate open_in_memory() per project.
