B-28 Focus Tunnel — Strategic Scope (E-1..E-7 + dashboard wiring)

Source phase file: docs/thefinalglance/B-plan/B-28_focus_tunnel.md
Spec ground: docs/specs/guide/focus-tunnel-design.txt + post-phase4-implementation-order.txt (STEPS E-1..E-7)
FSM at scope time: PLANNING, attempts 0/3
B-N type: NEW-FEATURE (Producer partitioning)


1. Module layout — crates/orchestrator/src/focus_tunnel/

Confirmed subdir layout from the "Files touched" manifest. Proposed mod.rs:

    pub mod fim;
    pub mod diff;
    pub mod mode;
    pub mod streaming;
    pub mod pre_assembly;
    pub mod dep_resolver;
    pub mod file_router;

    pub use mode::{GenerationMode, ModeSelector, ModeSelection};
    pub use fim::{FimPromptAssembler, FimContext};
    pub use diff::{DiffParser, DiffHunk};
    pub use streaming::{StreamingValidator, ValidationEarlyTerm};
    pub use pre_assembly::{PipelinePreAssembler, PreAssembled};
    pub use dep_resolver::{DependencyResolver, DepResolution};
    pub use file_router::{SpeculativeFileRouter, FileRouterDecision};

Public surface to task_runner.rs is intentionally narrow: ModeSelector::select,
FimPromptAssembler::assemble, DiffParser::parse, StreamingValidator::wrap_stream,
PipelinePreAssembler::spawn_lookahead, DependencyResolver::resolve,
SpeculativeFileRouter::route. Everything else stays pub(crate).


2. Cross-cutting types — placement

In focus_tunnel/mod.rs (re-exported, used by 2+ submodules):
  - GenerationMode { Fim, Diff, Structured } — used by mode, fim, diff, streaming, task_runner
  - ModeSelection { mode, reason } — emitted into SSB by task_runner
Per-module (stay private to that module's public surface):
  - FimContext (fim.rs)               — only fim::FimPromptAssembler consumes
  - DiffHunk (diff.rs)                — internal to diff parser, exposed via Vec<DiffHunk>
  - ValidationEarlyTerm (streaming.rs)— terminator reason, returned from wrap_stream
  - PreAssembled (pre_assembly.rs)    — buffer carried across step N -> step N+1
  - DepResolution (dep_resolver.rs)   — outcome of resolve(), surfaces in SSB
  - FileRouterDecision (file_router.rs) — drives micro-step ReplanRequest

Rationale: GenerationMode/ModeSelection cross 4+ modules; the rest are output
records of a single submodule and don't need top-level re-export.


3. SSB action_types wiring path — ssb.rs

ssb.rs defines `pub enum SsbActionType` at line 169 and `pub enum SsbPayload`
at line 2375. New variants are appended (never inserted) to preserve the
serde snake_case discriminants used by existing rows:

  SsbActionType:                        SsbPayload:
    + GenerationModeSelected              + GenerationModeSelected(GenerationModeSelectedPayload)
    + StreamingEarlyTerminated            + StreamingEarlyTerminated(StreamingEarlyTerminatedPayload)
    + PipelinePreAssembled                + PipelinePreAssembled(PipelinePreAssembledPayload)
    + DependencyResolved                  + DependencyResolved(DependencyResolvedPayload)

ssb_action_type_from_str (line ~618) must map "generation_mode_selected" etc.
4 new payload structs (Serialize/Deserialize via rmp_serde) live alongside
existing ones; existing variants and rows are untouched.


4. ModelCapabilities.supports_fim — placement

No `ModelCapabilities` exists today in providers/src. (Only
ModelCapabilityProfile in orchestrator/context_budget for VRAM-tier sizing —
that is NOT the same concept.) E-1 introduces a new providers concept.

Proposed: extend providers/src/types.rs (existing public types hub, line ~6+)
with:

    pub struct ModelCapabilities {
        pub supports_fim: bool,
        pub fim_tokens: Option<FimTokenSet>,   // <|fim_prefix|> etc
    }

Re-export from providers/src/lib.rs alongside the existing `pub use types::{...}`
block (line 54-60). The ProviderClient trait (in client.rs) gains
`fn capabilities(&self) -> ModelCapabilities`. Qwen2.5-Coder / StarCoder /
DeepSeek-Coder implementations return true; AnthropicClient / OpenAiCompatClient
return false. sidecar_client.rs returns true gated on model name.


5. task_runner.rs integration points — DO NOT FULL-READ

Grep evidence (no file-read needed):
  - drive_task is defined at task_runner.rs:2879 (the spec-known monolith hub).
  - The production execution surface is the `BabyStepExecutor` trait at
    task_runner.rs:746 with the canonical impl `AgentPoolBabyStepExecutor`
    at task_runner.rs:1029.
  - `baby_step_executor.execute(exec_ctx)` is the single chokepoint (see
    docstrings at lines 745, 885, 2003).

Integration plan: focus_tunnel hooks into the existing BabyStepExecutor path,
NOT drive_task directly. AgentPoolBabyStepExecutor::execute consumes:
  step 1 — ModeSelector::select(step, capabilities) -> ModeSelection
  step 2 — emit GenerationModeSelected to SSB
  step 3 — branch by mode:
            Fim  -> FimPromptAssembler::assemble  -> provider.generate
            Diff -> standard prompt + DiffParser::parse + edit_matching dispatch
            Structured -> existing path unchanged
  step 4 — StreamingValidator::wrap_stream around provider stream
  step 5 — on success: PipelinePreAssembler::spawn_lookahead(step_n+1)
  step 6 — DependencyResolver::resolve on detected import deltas
  step 7 — SpeculativeFileRouter::route — on AST mismatch emit MINOR
            ReplanRequest with create_file micro-step

drive_task itself is not modified. The TaskRunnerConfig.baby_step_executor
slot (line 1955) is already the canonical injection point.


6. Dashboard surface — dashboard-v2 confirmed

Target is dashboard-v2 (V1 dashboard explicitly deprecated by the
"Files touched" manifest line 188-190 "V2 — was V1 …").

Reference panel pattern (read ProposalsPanel.svelte head only, 60 lines):
  - Svelte 5 runes: `let pending = $state<...>([])`
  - imports: `import * as stub from '$lib/api/stub'` + types from
    `$lib/api/types'`
  - polling: `setInterval(load, ...)` in `onMount`, cleared in `onDestroy`
  - browser-only fallback: stub.ts returns mock-data when Tauri absent
  - error surface: `let loadError = $state<string | null>(null)`

State stores are dashboard-v2/src/lib/state/*.svelte.ts (e.g. activity,
commands, plan, media_validation, session). No central "SSB store" — each
panel polls its own Tauri commands; that is the established convention.

FocusTunnelPanel.svelte therefore:
  - lives in .elai_cc/crates/dashboard-v2/src/lib/components/
  - polls 4 new Tauri commands (get_focus_tunnel_modes, get_focus_tunnel_streaming,
    get_focus_tunnel_pre_assembly, get_focus_tunnel_deps) — or one composite
    get_focus_tunnel_metrics — added in dashboard-v2/src-tauri/src/commands.rs
  - shapes: add to src/lib/api/types.ts; mocks to src/lib/api/mock-data.ts;
    stub wrappers in src/lib/api/stub.ts
  - registered in lib.rs invoke_handler list alongside existing commands


7. Sequencing — which E-steps unlock which

Linear E-1 -> E-7 is the safe default. Justified deviations:
  - E-1 (FIM) and E-2 (Diff) are independent producers — can be done in
    parallel by two sub-agents IF the orchestrator-focus-tunnel partition
    is split. Recommendation: keep linear; the providers and tools changes
    are small enough that serialising them is cheaper than coordinating.
  - E-3 (ModeSelector) HARD-depends on E-1 + E-2 (it picks between them).
  - E-4 (StreamingValidator) is independent of E-1/E-2 but only useful
    once at least one mode emits a stream — can land after E-3 in a sibling
    PR, no dependency.
  - E-5 (PipelinePreAssembler) HARD-depends on E-4 (lookahead is spawned
    on validator pass).
  - E-6 (DependencyResolver) is independent; can land anywhere after E-3.
  - E-7 (SpeculativeFileRouter) HARD-depends on E-3 + E-4 (uses mode +
    stream signals to decide micro-step).
  - Dashboard panel: blocked until at least one SSB action_type lands —
    earliest after E-3.

Final order: E-1 -> E-2 -> E-3 -> E-4 -> E-5 -> E-6 -> E-7 -> dashboard.
Treat each E-step as its own commit (8 commits total). Stub_ban per the
WIRING_CONTRACT applies at every commit: no model-invoked cargo/pip, no
parallel depth >1, no missing-dep auto-install in INTERACTIVE mode.


8. Files-touched manifest per E-step (file-claim targets)

E-1 (FIM + capabilities):
  ADD: .elai_cc/crates/orchestrator/src/focus_tunnel/mod.rs
  ADD: .elai_cc/crates/orchestrator/src/focus_tunnel/fim.rs
  MOD: .elai_cc/crates/orchestrator/src/lib.rs                      (pub mod focus_tunnel)
  MOD: .elai_cc/crates/providers/src/types.rs                       (ModelCapabilities)
  MOD: .elai_cc/crates/providers/src/lib.rs                         (pub use)
  MOD: .elai_cc/crates/providers/src/sidecar_client.rs              (capabilities() impl + FIM tokens)
  MOD: .elai_cc/crates/providers/src/client.rs                      (trait method)
  MOD: .elai_cc/python/elai_inference/server.py                     (FIM token surface, raw-text path)

E-2 (Diff parser + edit matching):
  ADD: .elai_cc/crates/orchestrator/src/focus_tunnel/diff.rs
  MOD: .elai_cc/crates/orchestrator/src/focus_tunnel/mod.rs         (pub mod diff + re-exports)
  MOD: .elai_cc/crates/tools/src/edit_matching.rs                   (hunk-driven entry point)
  MOD: .elai_cc/crates/tools/src/lib.rs                             (re-export if needed)

E-3 (Mode selector):
  ADD: .elai_cc/crates/orchestrator/src/focus_tunnel/mode.rs
  MOD: .elai_cc/crates/orchestrator/src/focus_tunnel/mod.rs
  MOD: .elai_cc/crates/orchestrator/src/pipeline_builder.rs         (PipelineSpec.preferred_mode)
  MOD: .elai_cc/crates/orchestrator/src/ssb.rs                      (GenerationModeSelected variant + payload)
  MOD: .elai_cc/crates/orchestrator/src/task_runner.rs              (BabyStepExecutor branch)

E-4 (Streaming validator):
  ADD: .elai_cc/crates/orchestrator/src/focus_tunnel/streaming.rs
  MOD: .elai_cc/crates/orchestrator/src/focus_tunnel/mod.rs
  MOD: .elai_cc/crates/orchestrator/src/ssb.rs                      (StreamingEarlyTerminated variant)
  MOD: .elai_cc/crates/orchestrator/src/task_runner.rs              (wrap_stream call site)

E-5 (Pipeline pre-assembly):
  ADD: .elai_cc/crates/orchestrator/src/focus_tunnel/pre_assembly.rs
  MOD: .elai_cc/crates/orchestrator/src/focus_tunnel/mod.rs
  MOD: .elai_cc/crates/orchestrator/src/ssb.rs                      (PipelinePreAssembled variant)
  MOD: .elai_cc/crates/orchestrator/src/task_runner.rs              (spawn_lookahead on validator OK)

E-6 (Dep resolver):
  ADD: .elai_cc/crates/orchestrator/src/focus_tunnel/dep_resolver.rs
  MOD: .elai_cc/crates/orchestrator/src/focus_tunnel/mod.rs
  MOD: .elai_cc/crates/orchestrator/src/ssb.rs                      (DependencyResolved variant)
  MOD: .elai_cc/crates/orchestrator/src/task_runner.rs              (resolve() call site + allowlist)
  MOD: .elai_cc/crates/rules/src/* (deprecated_patterns consume only — read path, no schema change)

E-7 (File router + micro-step):
  ADD: .elai_cc/crates/orchestrator/src/focus_tunnel/file_router.rs
  MOD: .elai_cc/crates/orchestrator/src/focus_tunnel/mod.rs
  MOD: .elai_cc/crates/orchestrator/src/task_runner.rs              (route() pre-execute; ReplanRequest emit)

Dashboard wiring (post-E-7):
  ADD: .elai_cc/crates/dashboard-v2/src/lib/components/FocusTunnelPanel.svelte
  MOD: .elai_cc/crates/dashboard-v2/src/lib/api/types.ts            (4 row types)
  MOD: .elai_cc/crates/dashboard-v2/src/lib/api/stub.ts             (4 stub wrappers)
  MOD: .elai_cc/crates/dashboard-v2/src/lib/api/mock-data.ts        (4 mock rows)
  MOD: .elai_cc/crates/dashboard-v2/src-tauri/src/commands.rs       (4 #[tauri::command] queries)
  MOD: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs            (invoke_handler register)
  ADD: workflow/reasoning/B-28-decisions.md                          (per WIRING_CONTRACT)


VALIDATION WARNINGS
  - Most seed symbols (FimPromptAssembler, DiffParser, ModeSelector,
    StreamingValidator, PipelinePreAssembler, DependencyResolver,
    SpeculativeFileRouter, supports_fim, GenerationMode) do NOT exist in
    elai.map yet. This is expected for a NEW-FEATURE B-N; explorers must
    verify by reading source (not the map).
  - Hub-watch flags 3 spec-known monoliths (task_runner.rs, dashboard-v2
    lib.rs, server.py). Synthesizer should require 2+ explorer agreement
    before any edit to those files.
  - dashboard-v2 has NO central SSB store; each panel polls its own Tauri
    commands. Don't invent a shared store — follow the per-panel pattern.


HUB-WATCH
  .elai_cc/crates/orchestrator/src/task_runner.rs (auto-flagged monolith)
  .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (auto-flagged monolith)
  .elai_cc/python/elai_inference/server.py (auto-flagged monolith)
  .elai_cc/crates/orchestrator/src/ssb.rs (4+ partition hits — append-only protocol)


DECISION
  EXPLORE — 6 ELAI subsystems. Plan: workflow/sessions/s-27436/exploration-plan.md
