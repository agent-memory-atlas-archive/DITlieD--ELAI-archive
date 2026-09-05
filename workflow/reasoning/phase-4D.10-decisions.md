Session 4D.10 — GroundingClient IPC wiring — decision record
Session: s-1635212
FSM: CONFIGURING -> PLANNING -> TESTING_FIRST -> EXECUTING -> (pending) AUDITING

Goal
Replace synthetic FlashResearch reports in research_tree.rs with a real sidecar-backed LLM grounding path. End-to-end: research_decompose becomes async, calls a typed GroundingClient trait, wraps errors as ResearchTreeError::GroundingFailed, and task_runner.rs:644 .awaits the result.

Scope
M bucket (not XL per original prompt). Recursive decomposition types and the 16 recursive_research integration tests were already complete at 4D.9; this session closes the grounding IPC gap only.

Key decisions (grounded)
1. ResearchLevel enum (new) in grounding_client.rs with variants Idea|Concept|Phase|Step.
   Rationale: `ResearchLevel` did not exist; `DecompositionLevel` (pipeline_tree_navigator.rs:74) exists but has no `Idea` variant so is the wrong type for the 4 template files (l0_idea..l3_step). `research_tier` module exists but holds `ResearchTierConfig` (VRAM config) — unrelated.
   Grounding: Grep of crates/orchestrator/src (no ResearchLevel), pipeline_tree_navigator.rs:74, research_tier.rs full read.

2. Native async fn in traits, no async_trait crate. Generic static dispatch (`research_decompose<C: GroundingClient>`) rather than &dyn GroundingClient.
   Rationale: CLAUDE.md anti-stale-training rule forbids async_trait. reqwest 0.12 is already in Cargo.toml:21 with json+rustls-tls features, confirming a modern async stack is available.
   Grounding: Cargo.toml:21, CLAUDE.md anti-stale-training.

3. Cache refactor: added pub peek + put to ResearchCache; rewrote get_or_research on top of them for backward compat. research_decompose uses the inline-await pattern (peek -> client.ground().await -> put).
   Rationale: get_or_research takes FnOnce (sync closure). Making the closure async would require an async-closure refactor or block_on inside async (forbidden by codeRule #7). Inline-await with peek/put leaves get_or_research intact for existing callers (recursive_research.rs:153,161,179,186,337,342).
   Grounding: research_tree.rs:1374-1421 original impl.

4. Production build_synthetic_report deleted. Equivalent logic now lives in SyntheticGroundingClient::ground() in grounding_client.rs. SyntheticGroundingClient is always pub (not cfg(test)) so integration tests in tests/ can use it without feature-flag gymnastics.
   Rationale: Plan required removing synthetic from production. SyntheticGroundingClient is documented as "for tests and deterministic execution; never a silent production fallback". Production path constructs SidecarGroundingClient::default() — no synthetic available.
   Grounding: original research_tree.rs:1510-1544.

5. Sidecar request shape: OpenAI-compatible POST /v1/completions with {"prompt", "max_tokens", "temperature"}, response.choices[0].text. Per-request timeout via RequestBuilder.timeout() (infallible Client::new() construction — no builder.build() fallibility to handle).
   Grounding: server.py:1805-1882 (handler), server.py:1815-1817 (request fields).

6. GroundingError variants: SidecarUnavailable, Timeout, HttpStatus, InvalidJson, EmptyResponse, RequestFailed. thiserror-derived (Send+Sync+'static). Wrapped as ResearchTreeError::GroundingFailed { level, source } so upstream callers see the level-tagged failure. RequestFailed { reason } catches the non-timeout / non-connect reqwest errors (TLS, DNS, redirects, body read) so they do not misattribute as SidecarUnavailable.

7. Timeout constant: new GROUNDING_TIMEOUT_SECS=30 in grounding_client.rs rather than reusing INTEGRATION_CRITERION_TIMEOUT_SECS.
   Rationale: semantic separation — the 30s wall-clock budget for integration criteria (pipeline_tree_navigator.rs:34) is a different concern from sidecar request timeout, even though both happen to be 30s today.

Callers migrated
- task_runner.rs:644 (prod): .await + &SidecarGroundingClient::default().
- task_runner.rs:1197 test: #[tokio::test] async + &SyntheticGroundingClient::new().
- recursive_research.rs: tests 1, 10, 11, 13 migrated to #[tokio::test] + &SyntheticGroundingClient. Tests 2-9, 12, 14, 15, 16 are unaffected (they use extract_* helpers or cache.get_or_research directly).
- New: tests/grounding_client_contract.rs (7 contract tests covering ResearchLevel template names, SidecarGroundingClient ctor + default port, research_decompose error propagation, research_decompose no-silent-fallback via CountingClient, GroundingError Send+Sync + Display per variant, JSON parser empty/malformed rejection, JSON parser code-fence stripping).

Out of scope (deferred)
- TaskSubmitted SSB consumer wiring (DASHBOARD_TASK_SUBMITTED_PICKUP_DEFERRED).
- RetryPolicy, ABMCTS scoring, GG-signal handling.
- L3 step grounding — decompose_to_baby_steps still synthesizes baby steps deterministically from StepNode data (no sidecar call at L3). The l3_step.txt template is created but not yet invoked; that's a later session per the plan.
- Dashboard integration_dashboard_state test DashboardState field mismatch — pre-existing (patch-1.30 MVP wiring), out of this session's scope.

Expected test outcome
- cargo check --workspace: clean for elai-orchestrator, pre-existing error in elai-dashboard (task_runner_cancel field).
- cargo test -p elai-orchestrator: 16/16 recursive_research tests GREEN, 7/7 grounding_client_contract tests GREEN.
- Contract test invariants: ResearchLevel template names, SidecarGroundingClient default port=8100, GroundingError Send+Sync, parser fence stripping, research_decompose error propagation.

Unknowns
- Real end-to-end test against a live sidecar requires server.py running + a model loaded. That's covered by phase 4D.11+ integration, not 4D.10 unit/contract scope.
- SSB Finding row emission path: research_decompose errors propagate through TaskRunnerError::Decompose(#[from] ResearchTreeError) via task_runner.rs:117-121. Whether that path currently emits a Finding row is task_runner's internal mechanism, unchanged by this session — to be verified by the auditor.
