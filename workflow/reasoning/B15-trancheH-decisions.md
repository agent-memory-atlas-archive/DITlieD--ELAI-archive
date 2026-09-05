B-15 Tranche H decisions (s-22600 trancheH executor)
Date: 2026-05-31
Scope: entropy burndown across orchestrator + knowledge + plugins + providers crates
Source ctx: workflow/sessions/s-22600/context-package-trancheH.md

OVERALL POSTURE
Tranche H finishes the long tail of `SystemTime::now()` / `Instant::now()` migrations to elai_core::clock::{ClockProvider, OsClock}. Pattern is mechanical inline substitution per the CLI Tranche E precedent; struct-field injection reserved for sandbox/firecracker.rs SnapshotPool only. ~40 of 65 orchestrator src files already import the abstraction; this tranche closes the gap.

DH1 SESSION_DAG NOT 171 SITES
Stubs.md row claimed session_dag.rs had 171 entropy sites. Actual Grep confirms exactly 1 site at L2126, already `// TEST_KEEP: wall-clock perf gate for batch insert`. ZERO migration work this tranche; the 171 figure is a transcription error to be corrected by memory-writer.

DH2 ORCHESTRATOR FULL SCOPE SAFE
Migrate ALL ~35-40 prod files in ONE tranche. Per-file work is ~1-2 sites mechanical; cascade risk LOW everywhere; auditor runs cargo check --workspace --tests; compile errors surface immediately. Reject 2-tranche artificial split.

DH3 INLINE VS FIELD INJECTION
INLINE `elai_core::OsClock.now_ms()` / `OsClock.instant_now()` for single-shot reads at fn body. Struct-field `clock: Arc<dyn ClockProvider>` ONLY for sandbox/firecracker.rs SnapshotPool (multi-site state, deterministic snapshot timestamps matter for SeededClock-driven tests). Reject full Arc threading.

DH4 RENDER ADAPTERS PRESERVE EXEMPT
media_validator/tier1.rs (4 sites) + render_adapters/*.rs (14 marker sites total) STAY AS-IS with their `entropy-allow: C-15 deferred — render-adapter Clock plumbing scheduled` markers. Removing the marker breaks the C-15 quarantine contract per gotchas.md C15_ENTROPY_ALLOW_RENDER_ADAPTERS_REPLACE_WHEN_THREADED.

DH5 GITHUB_PR/CLOCK PRESERVE-OR-MARK
orchestrator/src/github_pr/clock.rs IS the E-41 submodule clock abstraction layer (per L1-5 doc-comment). NOT a quarantine violation. ADD `// entropy-allow: e41-github-pr-submodule-clock-abstraction` marker @L20 so check_entropy_quarantine.sh sees the intentional design. Reject deletion + forcing every E-41 caller to import elai_core::OsClock.

DH6 KNOWLEDGE NULL RESULT
H.2 verified NO-OP. All 4 grep hits in knowledge/src are entropy-allow-marked or doc-comments. cosmos_worker.rs:884 entropy-allow + startup.rs:706 doc-comment + startup.rs:709 entropy-allow (cfg-test) + symbol_graph/build_pipeline.rs:215 doc-comment. Write null-result decision; no source edits. Update s-22300 stubs.md to mark knowledge segment VERIFIED-COMPLIANT-NULL-RESULT.

DH7 PLUGINS MIGRATE-3 MIGRATE-1
Cargo.toml confirms plugins lists `elai-core = { path = "../core" }`. Per W3 CASE A: mcp_bridge.rs:1128 MIGRATES inline (the C-47 doc-comment claim "no orchestrator clock dep" is stale; the abstraction is in core, already a dep). Migrations: mcp_schema_pin.rs:140 + :186 + mcp/lazy.rs:54 + mcp_bridge.rs:1128 = 4 inline. ZERO new markers.

DH8 PROVIDERS MIGRATE-2 MARK-1
Cargo.toml confirms providers lists `elai-core = { path = "../core" }`. Migrations: anthropic_api.rs:240 + sidecar.rs:274 (Instant::now -> OsClock.instant_now). escalation_policy.rs:328 gets `// entropy-allow: e31-escalation-policy-saturating-on-failure-mirrors-ssb-ts-now-ms` marker per user directive (L324 doc grounds the saturating-on-failure rationale mirroring ssb.rs::ts_now_ms; matches structured_output_cmd.rs:467 precedent).

DH9 TEST_KEEP PRESERVE ALL
ALL TEST_KEEP-marked sites across H.1/H.2/H.3/H.4 stay verbatim (~30 sites total). CLAUDE.md test-exempt. Removing TEST_KEEP markers from test helpers may change tmp-dir-name collision behavior + invalidate timing assertions.

DH10 NO SCHEMA BUMP
All 4 items add ZERO new SsbActionType / Kind / payload / variant. SCHEMA_VERSION stays at 195. TOTAL_VARIANTS stays. ZERO golden-file regen.

EXECUTION ORDER
H.1 orchestrator (heaviest) -> H.3 plugins -> H.4 providers -> H.2 knowledge null-result. cargo check -p <crate> --tests after each item; cargo check --workspace --tests at end.
