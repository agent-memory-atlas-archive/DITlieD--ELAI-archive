D4 Obscura Headless-Browser Fetch Adapter — Decision Doc
2026-05-05

WHAT
Add a headless-browser fetch path to elai-web-search using the obscura CLI binary
(subprocess, not a library), selectable via ELAI_FETCHER=obscura env var.

KEY DECISIONS

1. Feature-gate vs trait-extraction
   DECISION: cfg(feature = "obscura") feature-gate.
   REASON: MVP spec explicitly says feature-gate. Trait extraction (dyn Fetcher) is
   listed as "Prod (out of scope here)". Feature-gate avoids adding any runtime overhead
   or heap allocations to the default reqwest path. The two impls are structurally similar
   enough that parallel concrete structs + compile-time selection via the feature flag is
   the lightest path and matches the spec's "cfg-feature swap of type aliases" guidance.

2. Subprocess async wrapping
   DECISION: tokio::task::spawn_blocking wraps std::process::Command.
   REASON: std::process::Command::output() is blocking. Calling it directly on a
   tokio task thread would starve the runtime. spawn_blocking moves it to the blocking
   thread pool. No additional dependency needed (std + tokio::task).

3. Binary path configuration
   DECISION: OBSCURA_PATH env var at runtime; compile-time default to
   <binary-dir-of-obscura-fetcher>/../tools/obscura/obscura.exe on Windows or
   "obscura" (PATH lookup) on non-Windows.
   Actual default used: env var OBSCURA_PATH checked first; fallback to
   F:\.ELAI_workflow\.elai_cc\tools\obscura\obscura.exe as compile-time const on Windows.

4. Output size cap
   DECISION: 5 MB cap on subprocess stdout (matches spec).
   Anything above is truncated with a log warning; not treated as an error (partial
   HTML is still useful for extraction).

5. WebSearchError variant reuse
   DECISION: Reuse FetchFailed{url, reason} for all obscura subprocess errors.
   No new variants. InvalidUrl{url, reason} for URL validation rejection.

EXPECTED TEST OUTCOME
- cargo check -p elai-web-search: 0 errors (default, no feature)
- cargo check -p elai-web-search --features obscura: 0 errors
- cargo check --workspace: 0 errors
- cargo test -p elai-web-search: all existing tests pass (no regression)
- cargo test -p elai-orchestrator -- web_research: all existing tests pass

UNKNOWNS
- obscura binary may not support --stealth on all platforms; flag included per spec.
- Live validation depends on sidecar being up (confirmed in task brief).
- If ELAI_FETCHER=obscura and feature is off: warn + fall back to reqwest (no panic).
