# patch-1.35 decisions — dead-code sweep + module-docs

Session ID: s-589272
Date: 2026-04-18
Scope: patch-1.35 — cosmetic P2 sweep per docs/mvp-wiring-gaps.md L552-L596 + Appendix B resolutions.

## Grounding (6 parallel Explore agents)

| Candidate | Audit claim | Ground truth | Verdict |
| --- | --- | --- | --- |
| DolaDecoder + dola.rs + dola_contract.rs | Dead; sidecar does DoLa | Only test caller is dola_contract.rs; sidecar/server.py:642 DoLaSteeringProcessor is production path; zero spec references Rust-side DoLa | DELETE |
| ValidatedSsb (validated_ssb.rs) | Zero callers anywhere | Confirmed zero non-definition callers; pub mod in orchestrator/lib.rs but no downstream import | DELETE |
| IpcHookDispatcher (plugins/ipc_hooks.rs) | Only tests reference it | **Reversed:** orchestrator/src/ipc_gateway.rs:504,513,555,563 uses IpcHookTrait/IpcHookError/HookResult from ipc_hooks. Since IpcGateway itself is kept (phase-4A.txt mandate), ipc_hooks.rs must stay too. | KEEP WITH DECISION-NOTE |
| run_research_pipeline + SearchBackend trait + /research CLI | Deprecated by WebResearchOrchestrator | **CLI `/research` at cli/src/main.rs:4631 STILL ROUTES to run_research_pipeline; SearxSearchBackend (production, 4D.11) implements SearchBackend trait** | KEEP |
| compute_vector_via_sidecar + SteeringConfig + SteeringVector | Zero Rust callers | **cli/src/main.rs:2669 calls compute_vector_via_sidecar inside try_precompute_role_steering, satisfying patch-1.33 acceptance criterion** | KEEP |
| IpcGateway | Test-only ctors | **phase-4A.txt L259-L280 mandates orchestrator IPC gateway for 4A.5 wiring (wire-4A session)** | KEEP WITH DECISION-NOTE |

The mvp-wiring-gaps.md doc was stale on three items. Three verdicts reversed from doc's recommendation.

## Decision per item

### Deletions (confirmed)

1. Delete `.elai_cc/crates/providers/src/layer_config/dola.rs` (974 lines).
2. Delete `.elai_cc/crates/contracts/tests/dola_contract.rs`.
3. Remove `pub mod dola;` and its re-exports from `.elai_cc/crates/providers/src/layer_config/mod.rs`.
4. Delete `.elai_cc/crates/orchestrator/src/validated_ssb.rs`.
5. Remove `pub mod validated_ssb` from `.elai_cc/crates/orchestrator/src/lib.rs`.
6. (REVERSED — keep ipc_hooks.rs; orchestrator/src/ipc_gateway.rs depends on IpcHookTrait/IpcHookError/HookResult from this module. Decision-note added instead.)

### Module-docs (Appendix B + reversals)

1. `.elai_cc/crates/knowledge/src/lib.rs:242` — KnowledgeEngine trait: add doc saying "Scheduled facade for Phase 1I; sub-engines wired directly by CLI in MVP. Referenced as architectural template by EvidenceGate/GrcEngine — do not remove."
2. `.elai_cc/crates/orchestrator/src/ssb.rs:591` — SsbPayload: add doc saying "All 31 variants are schema-committed. Variants without current emitters (Claim, ModelSwap, KnowledgeDegradation, IsolationMetric, BranchMerge, SpiralDetected) are forward-compat for phases 3F/4G/4H; do not remove — on-disk SSB files and dashboard payload_json pipeline depend on stable discriminants."
3. `.elai_cc/crates/plugins/src/mcp_bridge.rs:1` — DefaultMcpBridge: add doc saying "Behind `mcp` feature (default=off). MVP workflow routes mcp_* tool calls through Claude-Code-hosted Python MCP server; this Rust bridge is scaffolding for a future in-process MCP host (dashboard Tauri → Rust tool invocation). Do not remove without roadmap confirmation."
4. `.elai_cc/crates/orchestrator/src/ipc_gateway.rs:1` — IpcGateway: add doc saying "Production wiring scheduled in Session wire-4A per phase-4A.txt L259-L280 (SO_PEERCRED, 0700 socket dir, frame routing). Test callers cover spec contract (TDD red-phase). Do not remove."
5. `.elai_cc/crates/orchestrator/src/anti_spiral.rs:1` — AntiSpiralDetector: add doc saying "Production wiring scheduled in Session wire-antispiral — task_runner GRC loop calls check_step per step, emits SsbPayload::SpiralDetected on HardStop. Depends on wire-4A (real agent execution)."
6. `.elai_cc/crates/orchestrator/src/deterministic_auditor.rs:1-14` — DeterministicAuditor: amend existing doc to note "Production wiring scheduled in Session wire-a12 — invoked from FSM AUDITING state transition handler before LLM auditor agent."
7. `.elai_cc/crates/orchestrator/src/supervision.rs:1` — SupervisionManager: add doc saying "Production wiring follow-up; test callers validate spec contract. Not in current MVP wire-* roadmap."
8. `.elai_cc/crates/orchestrator/src/ssb.rs` (near SsbActionType::InterventionRequest) — add comment saying "Emitter path follow-up; currently written only by ssb_viewer test helpers. Production emitter lives in the autonomous-gate session (newphase-4G)."
9. `.elai_cc/crates/plugins/src/ipc_hooks.rs:1` — IpcHookDispatcher + IpcHookTrait: add doc saying "Consumed by orchestrator::ipc_gateway::IpcGateway::run_pre_tool / run_post_tool (see ipc_gateway.rs:504-563). Production wiring scheduled in Session wire-4A per phase-4A.txt L259-L280. Test callers (contracts/tests/plugins_hooks.rs + plugins_adversarial.rs) validate spec contract as TDD red-phase."

### Stale-comment rewrites

1. `.elai_cc/crates/orchestrator/src/ssb.rs:580` — "pickup by the orchestrator is deferred" → current state comment (orchestrator consumes TaskSubmitted since patch-1.30 commit 8ce26a0).
2. `.elai_cc/crates/dashboard/src/commands.rs:238-239` — `INTERVENTION_WRITE_DEFERRED` const: verify & update (4K.5 landed).
3. `.elai_cc/crates/knowledge/src/vector/mod.rs:93` — "current stub" → embed is real (ort_session.rs:128).
4. `.elai_cc/crates/knowledge/tests/dkg_tests.rs:62,109,145,324,344,367,400,420,437,457` — "todo!()" comments → reflect build_graph is implemented.
5. `.elai_cc/crates/knowledge/tests/catrag_tests.rs:44,56,70` — "todo!()" test labels → reflect compute_overlay is implemented.

## Dependencies

- patch-1.32, patch-1.33, patch-1.34 landed (git log: 174835f).
- wire-4A, wire-antispiral, wire-a12 NOT yet landed (git log confirms). Module-docs for these three point to planned sessions.

## Tests

No new feature tests needed; sweep is mechanical. After deletions:
- `cargo check --workspace --lib --bins --tests` — must remain clean.
- Scoped: `-p elai_cc_providers -p elai_cc_orchestrator -p elai_cc_plugins -p elai_cc_contracts` covers blast radius.

## Unknowns

- Whether plugins_adversarial.rs contains non-IpcHookDispatcher tests — verify before delete-whole vs surgical removal. (N/A: IpcHookDispatcher deletion reversed.)
- Whether SupervisionManager is scheduled in a specific session doc not surfaced by grep — assume no (decision-note is conservative).

## Audit outcomes (adversarial-auditor, 2026-04-18)

Score 0.96. 0 BLOCK, 5 WARN, 10 INFO. All deletions re-verified zero non-doc references. KEEP reversals (compute_vector_via_sidecar, run_research_pipeline, IpcGateway, IpcHookDispatcher) correctly grounded against real production callers / spec mandates.

### Deferred WARNs (intentional MVP scope)

1. **patch-1.36 acceptance criterion #1 deferred.** Spec says "conversation layer injects WebSearch baby step on `last_search_trigger`". Shipped: `SignalObserver` trait + `with_observer` builder + tracing event. Caller-side baby-step injection requires task_runner's conversation loop integration (M-effort, outside this S-effort patch). Observer hook provides the plumbing; follow-up session wires the injector.
2. **patch-1.36 acceptance criterion #2 deferred.** Spec says "`last_gg_signals` recorded into SSB Finding/Claim". Shipped: tracing::debug! event with all five GG fields; observer hook ready to receive. SSB write requires injecting an `SsbWriter` handle into the CLI's HyDE construction path (M-effort). Follow-up session adds a `SidecarSignalSsbObserver` that calls `writer.write_payload(SsbActionType::Finding, ...)`.
3. **Integration test coverage of the observer wiring deferred.** `with_observer_attaches_callback` exercises `with_observer` + invokes `on_signals` directly; it does NOT drive `generate_hypothesis → complete → observer.on_signals`. Full integration requires a mock HTTP server or a trait-level `SidecarProvider` abstraction (M-effort). Current test validates the observer slot + callback signature only.
4. **systemPatterns.md stale entry on INTERVENTION_WRITE_DEFERRED.** Memory-bank is gitignored; the context-compiler agent will pick up the const's deletion from the codebase on next session load. No inline memory edit required (memory-writer agent would be the canonical updater).
5. **SidecarProvider mutex-poison swallow (pre-existing).** `last_search_trigger()` and `last_gg_signals()` on SidecarProvider silently return None/default on mutex poison. This is pre-existing behavior (patch-1.33) not introduced by this patch; flagged for a future signal-reliability hardening pass.

### Tests

- cargo check --workspace --lib --bins --tests: clean (warnings only).
- cargo test -p elai-providers --lib: 126/126 passed.
- cargo test -p elai-knowledge --test dkg_tests --test catrag_tests: 34/34 passed.
- cargo test -p elai-cli sidecar_hyde: 3/3 passed (including `with_observer_attaches_callback`).
- Pre-existing drift (NOT this patch's responsibility): orchestrator lib tests reference `PhaseNode.is_parallel`; drift from uncommitted wire-4A working-tree changes to research_tree.rs. `startup_banner_mentions_workflow_completions` flakes on Windows tempdir cleanup.
