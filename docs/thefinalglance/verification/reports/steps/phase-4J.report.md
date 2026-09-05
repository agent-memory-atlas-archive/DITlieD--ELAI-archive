# Verification Report: phase-4J.txt
Source: F:\.ELAI_workflow\docs\specs\guide\steps\phase-4J.txt
Date: 2026-05-17 (initial audit), 2026-05-23 (C-34 closure update)
Overall status: PARTIAL (CLOSURE_PENDING_ACTIVATION)
Confidence: HIGH

## C-34 closure update (2026-05-23)

Per workflow/reasoning/C-34-decisions.md, the 4J.1 host-side scaffold has
shipped (orchestrator::empire submodule per phase-4J-decisions.md DECISION 2;
5 SsbActionType empire arms per B-25-decisions.md DECISION B-25-A; --empire
CLI flag per B-25-C; EmpireBridgePanel + Tauri commands per B-25-D; AppState
.empire_bridge field + with_empire_bridge installer per B-25-B; ElaiNeuron
4-capability whitelist per B-25-F6; FactoryLineTransport + Unix/Tcp
transports + build_transport factory per B-25-G; EmpireRepairKind 4-variant
enum + heuristic detector per B-25-E). The C-34 contract test at
.elai_cc/crates/contracts/tests/c34_empire_integration_contract.rs pins
the SHIPPED interface shapes (8 assertions, all green).

4J.2-4J.9 live IO + measurement remain EMPIRE-PENDING (47 markers across
16 files) until the Empire codebase deploys. Original 2026-05-17 audit body
preserved below for archaeology.

---

## Summary
Phase 4J (Empire Integration) is entirely unimplemented. No `elai_empire` crate exists in the workspace and is absent from `Cargo.toml` members. All 4J-specific types (FactoryLineTransport, FactoryLineClient, BrainManifest, EmpireEventBridge, ElaiNeuron, EmpireIssue, FactoryCmd) return zero matches across all `.rs` files. The phase carries an explicit condition: "only implement when Empire codebase is accessible and deployed," which accounts for the absence.

## Deliverables verified DONE
None.

## Deliverables PARTIAL
None.

## Deliverables MISSING

### 4J.1 SKELETON — Types and stubs

- `trait FactoryLineTransport` — searched: FactoryLineTransport; no evidence
- `struct FactoryLineClient` — searched: FactoryLineClient; no evidence
- `enum TransportConfig` (Unix | Tcp) — searched: TransportConfig empire; no evidence
- `struct UnixTransport` / `struct TcpTransport` — searched: UnixTransport, TcpTransport; no evidence
- `struct FactoryLineConfig` — searched: FactoryLineConfig; no evidence
- `struct FactoryLineCommand` — searched: FactoryLineCommand; no evidence
- `enum FactoryCmd` (Register | Heartbeat | List | RequestUpdate) — searched: FactoryCmd; no evidence
- `struct FactoryLineResponse` — searched: FactoryLineResponse; no evidence
- `struct IpcMessage` (Empire wire type, distinct from elai internal IpcMessageType) — searched: Empire IpcMessage; no evidence (existing IpcMessageType is elai-internal, not Empire protocol)
- `enum IpcMessageType` (Empire variant: RegisterHook | Emit | QueryHooks | Subscribe | Unsubscribe | BusCreate | BusJoin | BusLeave | BusList) — searched: Empire IpcMessageType variants; existing IpcMessageType in ipc-types crate is elai-internal only
- `struct IpcResponse` (Empire) — searched: Empire IpcResponse; no evidence
- `struct BrainManifest` with `validate()` — searched: BrainManifest; no evidence
- `enum EngineTier` (MKI | MKII | MKIII) — searched: EngineTier; no evidence
- `struct NeuronRegistration` — searched: NeuronRegistration; no evidence
- `struct HookBinding` / `enum HookKind` — searched: HookBinding, HookKind; no evidence
- `struct EmpireEventBridge` with `rate_limiter` field — searched: EmpireEventBridge; no evidence
- `struct EventFilter` — searched: EventFilter; no evidence
- `struct ElaiNeuron` — searched: ElaiNeuron; no evidence
- `enum EmpireIssue` (CircularDep | HardImport | StdlibShadow) — searched: EmpireIssue; no evidence
- `struct EmpireRepairTarget` — searched: EmpireRepairTarget; no evidence
- `elai_empire` crate — searched: Cargo.toml workspace members; no empire entry

### 4J.2 VERIFY — Cross-platform compile check
- Compile verification of empire crate (cargo check -p elai_empire) — not applicable; crate absent

### 4J.3 TEST — FactoryLine protocol + registration tests
- Protocol round-trip tests (4 FactoryCmd variants) — searched: test files in empire crate; crate absent
- IPC round-trip tests (9 Empire IpcMessageType variants) — searched: same; crate absent
- Mock FactoryLine server tests — no evidence
- Registration + heartbeat tests — no evidence

### 4J.4 TEST — Event subscription + SSB bridge tests
- `brainctl_replacement` test (zero brainctl references) — no evidence
- `event_bridge` tests — no evidence
- `rate_limiter` tests — no evidence
- Global rate limiter (not per-pattern) tests — no evidence

### 4J.5 TEST — Capability exposure tests
- `capability` tests (exactly 4 capabilities exposed) — no evidence
- `trust_boundary` tests (rejected commands return generic error) — no evidence
- Async task lifecycle (submit -> poll -> result) tests — no evidence

### 4J.6 TEST — Empire repair pipeline tests
- `issue_detection` tests (3 EmpireIssue variants) — no evidence
- `repair_pipeline` tests — no evidence
- `TestOutcomeReport` zero PASS_TO_FAIL assertion — no evidence

### 4J.7 TEST — Cross-platform + resilience tests
- `cross_platform` transport parity tests — no evidence
- `resilience` reconnection + backoff tests — no evidence
- `degradation` graceful degradation tests — no evidence
- `fsb_parse` FSB header parsing tests — no evidence

### 4J.8 IMPLEMENT — Full implementation
- FactoryLine protocol client (tokio async, zero Python deps) — no evidence
- Brain registration with manifest validation — no evidence
- Event subscription + SSB bridge (EmpireEventBridge) — no evidence
- Capability exposure via ElaiNeuron + ToolFilter — no evidence
- Empire repair pipeline (Python AST analysis + PipelineSpec generation) — no evidence
- Graceful degradation (empire feature gate) — no evidence
- `cfg(feature = "empire")` feature gate — searched: feature empire Cargo.toml; no evidence

### 4J.9 MEASURE — Metrics file
- `F:\.ELAI_workflow\.elai_cc\metrics\phase_4J.toml` — searched: metrics/; file absent; existing metrics stop at phase_4I.toml

## Deliverables UNVERIFIABLE

- Empire codebase accessibility (workspace/factory-brain/) — the phase condition "only implement when Empire codebase is accessible and deployed" cannot be evaluated from this audit; Empire codebase path not present under .elai_cc/
- `docs/specs/integration/empire.txt` grounding claims — referenced spec was not read per audit scope (READ is limited to .elai_cc/ and the step file); grounding claims in the spec cannot be confirmed or denied

## What must be done (action list)

- [ ] Confirm Empire codebase is accessible at the configured path (elai_cc.toml [empire.codebase_path] or workspace/factory-brain/) before beginning implementation
- [ ] Create `crates/empire/` directory and add `elai_empire` as a workspace member in `F:\.ELAI_workflow\.elai_cc\Cargo.toml`
- [ ] Implement `trait FactoryLineTransport` with `UnixTransport` (cfg(unix)) and `TcpTransport` (all platforms)
- [ ] Implement `struct FactoryLineClient<T: FactoryLineTransport>` with heartbeat loop and reconnect backoff
- [ ] Implement all Empire wire types: FactoryLineCommand, FactoryCmd, FactoryLineResponse, IpcMessage (Empire), IpcMessageType (Empire variants), IpcResponse
- [ ] Implement `struct BrainManifest` with `validate()` method rejecting empty brain_id, empty neurons, duplicate neuron_ids
- [ ] Implement `struct EmpireEventBridge` with mandatory (non-Option) `rate_limiter` field
- [ ] Implement `struct ElaiNeuron` with hardcoded whitelist of exactly 4 capabilities
- [ ] Implement `enum EmpireIssue` with CircularDep, HardImport, StdlibShadow variants
- [ ] Write all 4J.3–4J.7 tests (TDD: write tests first, then implement 4J.8)
- [ ] Gate entire crate behind `cfg(feature = "empire")`; verify zero compile cost without the feature
- [ ] Create `F:\.ELAI_workflow\.elai_cc\metrics\phase_4J.toml` after 4J.9 measurements complete (>= 40 tests, all metric fields populated)

## Notes
The phase is CONDITIONAL (requires Empire codebase accessibility). This is the only phase in the 4x series with an explicit deployment prerequisite. The absence of implementation is consistent with the condition not yet being met. The `IpcMessageType` enum found in `crates/ipc-types/src/lib.rs` is elai's internal agent-orchestrator IPC type and is unrelated to the Empire wire protocol IpcMessageType defined in 4J.1.
