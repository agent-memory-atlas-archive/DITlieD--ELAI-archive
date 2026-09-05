B-25 decisions — Phase 4J Empire integration (post-scaffolding residual closure)

Date: 2026-05-23
Session: s-25120 (completion-drive — executor pass for B-25 residuals)
Pre-resolved by: phase-4J scaffolding pass (workflow/reasoning/phase-4J-decisions.md)
Cross-check report: workflow/sessions/s-25120/cross-check/B-25_phase_4J_empire_integration-report.md
Context package: workflow/sessions/s-25120/completion-drive/contexts/B-25_phase_4J_empire_integration-context.md
Exploration: workflow/sessions/s-25120/completion-drive/exploration/B-25_phase_4J_empire_integration-exploration.md
Grounding: workflow/sessions/s-25120/completion-drive/grounding/B-25_phase_4J_empire_integration-grounding.md
Sibling spec: docs/specs/guide/steps/phase-4J.txt

Relationship to phase-4J-decisions.md

B-25 is the implementation plan that mirrors spec phase-4J. The phase-4J scaffolding pass in this same session (s-25120) shipped 21 of the B-25 plan items as SHIPPED-ON-V2 or PARTIAL-V2 by building host-side scaffolding under feature gate `cfg(feature = "empire")` with EMPIRE-PENDING markers on every external-IO seam. Closing the phase-4J scaffolding pass closed the majority of B-25 too. This file documents the 7 ADDITIONAL B-25-specific decisions made in the residual-closure pass (this session) on top of the 6 pre-resolved decisions in phase-4J-decisions.md.

For the canonical phase-4J decision narrative (EMPIRE-PENDING strategy, crate vs submodule choice, 5-variant SSB granularity, IPC namespace, spec drift fixes, perf-number labelling), READ phase-4J-decisions.md first. The 7 decisions below build on that file and do not duplicate it.

DECISION B-25-A: SSB QUINTUPLE-UPDATE lockstep + SCHEMA_VERSION 101 -> 102

Resolves phase-4J-decisions.md F2 followup ("SSB action_type variants in ssb.rs via the 9-tuple lockstep").
Choice: ship the full atomic 7-site QUINTUPLE-UPDATE for the 5 empire_* variants now (NOT deferred to EMPIRE-PENDING flip).
Rationale: the bridge already typed-dispatches to the 5 string labels in phase-4J-decisions.md DECISION 3; adding the SSB lockstep now means the only EMPIRE-PENDING flip left for SSB rows is the bridge's `emit_action` body switching from `Err(NotConnected)` to `ssb.write_payload(...)`. Single-point flip, atomic, easy to audit.
Sites touched in ssb.rs (7 sites + 1 lock site):
  1. SCHEMA_VERSION const: 101 -> 102 (with the customary block-comment narrating the bump)
  2. CREATE_SSB_TABLE CHECK constraint: 5 'empire_*' strings appended
  3. SsbActionType enum: 5 new variants (EmpireConnected / EmpireDisconnected / EmpireEventReceived / EmpireCapabilityInvoked / EmpireRepairProposed) with rustdoc
  4. as_sql_str: 5 new match arms
  5. ssb_action_type_from_str: 5 new match arms
  6. 5 new payload structs (EmpireConnectedPayload etc.) ahead of BranchMergePayload
  7. SsbPayload enum: 5 new variants + action_type() dispatcher arms
  8. test_payload + all_test_payloads: 5 new fixtures
8th lockstep site (action_type_lock.rs/.golden):
  - TOTAL_VARIANTS const: 298 -> 303
  - ALL_VARIANTS array: 5 new entries appended (B-25 group)
  - exhaustive_match_guard: 5 new match arms
  - golden file regenerated via `cargo test ... regen_golden_file -- --ignored` (303 alphabetically-sorted entries)
Verification: `cargo test -p elai-orchestrator --features empire --lib ssb::tests::test_typed_payloads` PASSED with 242-payload bijection; `cargo test -p elai-orchestrator --test action_type_lock` PASSED (count_is_locked + golden_matches_runtime).

DECISION B-25-B: AppState.empire_bridge field + with_empire_bridge installer

Resolves phase-4J-decisions.md F3 followup ("Wire Arc<EmpireBridge> into AppState").
Choice: add `pub empire_bridge: Option<Arc<elai_orchestrator::empire::EmpireBridge>>` gated by `#[cfg(feature = "empire")]` plus an `AppState::with_empire_bridge` chainable installer + `empire_bridge_snapshot` cheap-clone helper.
Rationale: the dashboard launcher needs ONE place to hand the pre-constructed bridge to AppState; the snapshot helper is the read-side primitive commands_empire.rs consumes (drops the per-call `EmpireBridge::new()` construction in favour of the shared handle). The fallback in commands_empire.rs is "empire_bridge_snapshot() returns None -> use a transient EmpireBridge to render the same empty state we used to render". When activation lands, every empire-feature dashboard build has `Some(...)` populated and the fallback becomes unreachable.
Site: crates/dashboard-v2/src-tauri/src/state.rs (field + 2 methods, all feature-gated).

DECISION B-25-C: CLI --empire flag on `elai run` subcommand

Resolves B-25 WIRING_CONTRACT.reachable_from clause ("cli with --empire flag (or feature enabled)").
Choice: extend `CliAction::Run` with a new `empire: bool` field, add the `--empire` no-argument flag to the existing hand-rolled `parse_run_subcommand_args` parser, thread the bool through `RunArgs` to `run_async`. The orchestrator construction body emits `tracing::warn!` when `--empire` is requested but the orchestrator was NOT compiled with `--features empire` (graceful degradation; no error to user, just a session-diagnostics breadcrumb).
Rationale: matches the existing `--via-mcp` precedent (boolean opt-in flag, plumbed via RunArgs, no compile-time CLI feature mirror needed). The actual `Arc<EmpireBridge>` construction is gated on the orchestrator crate having the `empire` feature ON — when the CLI is built with `cargo build -p elai_cc --features 'elai-orchestrator/empire'` (or via a future top-level CLI feature) the warn becomes a real bridge construction. The flag is parsed + logged today so user intent is captured in the session record even before the construction lands.
Sites touched: crates/cli/src/args.rs (CliAction::Run field + parser arm), crates/cli/src/main.rs (destructure update), crates/cli/src/run_subcommand.rs (RunArgs field + run_async warn-log site).

DECISION B-25-D: EmpireBridgePanel mounted in SettingsWindow

Resolves B-25 task 8 ("Dashboard surface: EmpireBridgePanel ...") + phase-4J-decisions.md GAP-B mount-site choice.
Choice: mount inside SettingsWindow.svelte as a sibling section between `runtime backends` (RuntimePanel) and `cloud` (CloudPanel), matching the panel-per-section idiom Settings already uses.
Rationale: SettingsWindow is the established collection point for runtime-adjacent observability panels (RuntimePanel + CloudPanel both live there). A dedicated EmpireWindow would have its own tab + chrome — premature until the panel renders enough data to justify a full window. The activation patch can promote later.
Site: crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte (1 import + 1 section).

DECISION B-25-E: Empire repair pipeline types + heuristic detect_repair_kind

Resolves B-25 task 6 ("Empire repair pipeline: detect 3 verified issues...") + phase-4J-decisions.md F7 ("Implement EmpireRepairTarget + EmpireIssue ... via tree-sitter-python").
Choice: ship `EmpireRepairKind` (CircularDep / HardImport / StdlibShadow / Other(String)) + `EmpireRepairProposal` + `EmpireRepairError` + a regex/substring `detect_repair_kind(symbol_path) -> Option<EmpireRepairKind>` heuristic now. The tree-sitter-python AST walker is EMPIRE-PENDING — documented in the module rustdoc.
Rationale: types are real (serde-stable, round-trip tested) so the SSB `empire_repair_proposed` row payload can carry them once the activation patch lands. Heuristics are correct enough for the dashboard smoke surface (CircularDep fires on known module pairs in empire.txt's correction log; HardImport fires on `sys.path` markers; StdlibShadow fires on POSIX + Windows-style paths whose basename matches a 24-entry stdlib name list). The empty-path case returns `EmpireRepairError::EmptyPath` so callers never get silent swallow.
Why heuristics not AST: tree-sitter-python is a workspace dep but the Empire codebase is not in tree — building an AST walker against fixtures we do not have would be guess-work. The heuristic surface ships, compiles, tests, and is one swap away from the AST detector when the activation patch lands.
Site: crates/orchestrator/src/empire/repair.rs (NEW, ~220 lines + 8 unit tests).

DECISION B-25-F: reasoning::VerificationMethod::EmpireEventCheck + requires_empire_bridge

Resolves B-25 task 4 ("Empire events become GRC grounding source") + phase-4J-decisions.md DRIFT-5.
Choice: add `EmpireEventCheck { source: String, event_type: String }` as a new variant on `VerificationMethod` (NOT extend SourceCheck with subtypes). Add a `requires_empire_bridge` predicate mirroring `requires_cosmos_store`. NO change to the config.rs `all_verification_methods()` validator list (consistent with CosmosCheck / ArtifactCheck which also carry data and are not on the static known-variants list — the list validates ZERO-PARAMETER variants only; data-carrying variants are constructed at the call site).
Rationale: a new variant beats extending SourceCheck because (a) the verifier dispatch needs a distinct "read SSB rows of type empire_event_received filtered by source+event_type" path that has no overlap with SourceCheck's plain text/file source check; (b) the dashboard surface can filter by method type cleanly; (c) per phase-4J-exploration.md DRIFT-5 recommendation. EMPIRE-PENDING: the verifier's row-query path activates once the bridge starts writing rows (post-DECISION-B-25-A SSB lockstep is shipped, the writer flip is the final EMPIRE-PENDING gate).
Site: crates/reasoning/src/types.rs (variant + predicate).

DECISION B-25-G: FactoryLineTransport trait + UnixTransport (cfg(unix)) + TcpTransport + build_transport factory

Resolves B-25 WIRING_CONTRACT.produces_symbols ("Transport trait + UnixTransport + TcpTransport") + phase-4J-decisions.md F4.
Choice: add an object-safe `FactoryLineTransport` trait with native `async fn` returning explicit `Pin<Box<dyn Future + Send>>` (so the trait stays object-safe without async-trait). Add `UnixTransport` (cfg(unix)) and `TcpTransport` (all platforms) impls that return `EmpireBridgeError::NotConnected` until Empire deploys. Add a `build_transport(&TransportConfig) -> Result<Box<dyn FactoryLineTransport>, ...>` factory that picks the right impl from the existing `TransportConfig` carrier; the Unix arm on non-unix platforms returns `EmpireBridgeError::Protocol("...wrong transport for platform...")` so the caller surfaces a clear error instead of silent downgrade.
Rationale: the trait shape itself is host-side and useful TODAY (the bridge can hold `Box<dyn FactoryLineTransport>` once the activation patch wires it in). The impl bodies are EMPIRE-PENDING because the real tokio UnixStream / TcpStream boot needs captured Empire fixtures or the live socket — neither is available pre-deployment. Native `async fn` in trait per CLAUDE.md anti-stale-training (no async-trait crate); explicit `Pin<Box<dyn Future + Send>>` is the object-safe shape.
Site: crates/orchestrator/src/empire/factory_line.rs (trait + 2 impls + factory + 3 new unit tests).

DETERMINISTIC CHECK (cargo tree zero-pyo3)

`cargo tree -p elai-orchestrator --features empire | grep -c "^pyo3"` returns 0. Zero pyo3/cpython transitive deps leaked into the empire submodule (the R-NEW-3 phase-4J guard from phase-4J-decisions.md is satisfied).

EMPIRE-PENDING markers remaining

The phase-4J-decisions.md F1..F11 followup list shrinks as follows after this pass:
  F1 (flip every EMPIRE-PENDING marker)   — UNCHANGED; activation patch
  F2 (SSB lockstep + SCHEMA_VERSION bump) — CLOSED via DECISION B-25-A
  F3 (AppState.empire_bridge field)       — CLOSED via DECISION B-25-B
  F4 (Transport trait + impls)            — TRAIT-CLOSED via DECISION B-25-G; impl bodies remain EMPIRE-PENDING
  F5 (EventFilter + RateLimiter + Priority + dropped_count) — UNCHANGED; activation patch
  F6 (ElaiNeuron + ToolFilter)            — CLOSED via empire::types ElaiNeuron::tool_filter + ::allows() (shipped this pass with the AppState pass's types.rs edit)
  F7 (Empire repair types + AST detection) — TYPES-CLOSED via DECISION B-25-E; AST walker remains EMPIRE-PENDING (heuristic ships now)
  F8 (FSB binary transport 12-byte header) — UNCHANGED; activation patch
  F9 (replace UNGROUNDED_TARGET perf numbers) — UNCHANGED; measure once live
  F10 (captured Empire wire fixtures)     — UNCHANGED; needs Empire access
  F11 (RuntimeClient EXIT-CRITERIA empire-scoped) — UNCHANGED; deferred per F11 decision (empire-scoped only when activation lands)

Activation-patch sizing (post-B-25 closure): the remaining EMPIRE-PENDING flips are bounded to (a) the bridge's emit_action body (one function), (b) FactoryLineClient.connect/send/recv bodies (three functions), (c) UnixTransport/TcpTransport connect/send/recv bodies (six functions), (d) the heuristic-to-AST swap in detect_repair_kind (one function), (e) EventFilter+RateLimiter+Priority types (F5, not yet started), (f) FSB header parser (F8, not yet started). Everything else is host-side complete.

INGEST item (1 from grounding)

The grounding sweep flagged Firecracker boot <125ms numbers cited indirectly via orchestrator.txt:454 as INGEST. Per the grounding doc this is peripheral to B-25 closure (the sandbox cost claim is not on the Empire critical path). Deferred — paper-ingestor pass against Firecracker docs can run independently when sandbox perf becomes a B-25 dependency.

CARGO TREE VERIFICATION

Ran post-edit: `cargo tree -p elai-orchestrator --features empire 2>&1 | grep -c "^[│├└─ ]*pyo3"` -> 0. Confirmed no pyo3/cpython transitive dep leaked in via the empire module.
