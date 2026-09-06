Production Code Rules

Loaded conditionally for ALL coding and implementation tasks. Every rule mandatory. These rules prevent the most common and dangerous anti-patterns that LLM code assistants introduce. Grounded in empirical research: 40%+ of AI-generated code contains security flaws, 19.7% of suggested packages are hallucinations.


Unsafe Extraction (panic-class severity)

1. NEVER use .unwrap() or .expect() in library/production code. Use ? operator, .ok_or(), .ok_or_else(), .map(), .and_then(), .unwrap_or_default(), or explicit match. Acceptable ONLY in: test code (#[cfg(test)]/tests/), main()/entrypoint/bootloader init (panic = won't start), const/static init with provably valid literals (e.g. NonZeroUsize::new(4).unwrap()). Same rule applies to .expect() — panic is not error handling.

2. NEVER use panic!(), unreachable!(), unimplemented!() in library code. Library code returns Result. Only binaries may panic at top-level. todo!() is banned everywhere outside WIP branches.

3. NEVER silently discard errors. No let _ = fallible_call(); without a comment explaining WHY the error is intentionally ignored. No catch-all error handlers that swallow without logging.


Memory and Ownership (Rust-specific)

4. NEVER clone to satisfy the borrow checker without justification. Every .clone() on a non-Copy type must be justified: either borrowing is impossible (lifetime conflict) or the clone is intentional (shared ownership across threads). Run cargo clippy — redundant_clone is a hard fail.

5. NEVER use String where &str suffices. Function parameters that only read should take &str, &[T], or impl AsRef<str>. Returning owned types is fine. Same for Vec<T> vs &[T] in read-only positions.

6. NEVER allocate in hot loops without justification. No format!(), to_string(), vec![], or Box::new() inside loops that run per-request or per-item. Pre-allocate or use iterators.


Async and Concurrency

7. NEVER block the async runtime. No std::fs, std::thread::sleep, Mutex::lock (std), or synchronous HTTP inside async fn. Use tokio::fs, tokio::time::sleep, tokio::sync::Mutex, and async HTTP clients. CPU-heavy work goes to spawn_blocking.

8. NEVER assume maps/collections are thread-safe. Rust's type system prevents most of this, but Arc<Mutex<HashMap>> requires careful lock scope. Hold locks for minimum duration. Never hold a lock across an .await point.

9. NEVER retry without backoff. Network retries must use exponential backoff with jitter. Never retry 4xx client errors. Always cap max retries. Idempotency keys for state-mutating operations.


Input Validation and Security

10. NEVER trust input at system boundaries. All public API endpoints, deserialization points, CLI argument parsing, file path inputs, and environment variable reads must validate. Internal module boundaries may trust typed arguments.

11. NEVER interpolate user input into SQL, shell commands, or file paths. Use parameterized queries, Command::arg() (not format!() into shell strings), and canonicalize file paths to prevent traversal.

12. NEVER hardcode secrets, API keys, connection strings, or credentials. Use environment variables or secret managers. If a literal looks like a secret in a code review, it is one.

13. NEVER use eval(), dynamic code execution from strings, or deserialize untrusted data into arbitrary types without validation. Serde with explicit structs is fine; serde_json::Value from untrusted sources requires schema validation before use.


Error Types and Propagation

14. Public library APIs use typed errors (thiserror). Application/binary code may use anyhow. Never use Box<dyn Error> in public APIs — it erases the error contract.

15. Error messages must never leak internal state to end users. Log the full error server-side; return a sanitized message to the client. Stack traces, file paths, and SQL queries must never appear in user-facing responses.


Code Quality

16. NEVER leave dead code, commented-out code blocks, or unused imports in production. Use version control for history. #[deny(dead_code, unused_imports)] in lib.rs.

17. NEVER use magic numbers or magic strings. Extract into named constants with documentation. If a number appears in logic, it needs a name.

18. NEVER add a dependency without verifying it exists and is actively maintained. Check: registry presence, download count, last publish date, known CVEs. This is the anti-slopsquatting rule — LLMs hallucinate package names at a 19.7% rate.


Network and Distributed Systems

19. NEVER call an external service without a timeout. All HTTP clients, database connections, and RPC calls must have explicit connect and request timeouts. No infinite waits.

20. NEVER design a service call without considering the failure case. External calls need: timeout, retry with backoff, circuit breaker or fallback, and graceful degradation. If the downstream is down, the upstream must not crash.
