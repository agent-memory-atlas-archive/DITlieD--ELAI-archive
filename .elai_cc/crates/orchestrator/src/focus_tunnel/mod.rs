//! Focus Tunnel — zero-overhead LLM execution path (B-28). E-1 ships only the
//! FIM prompt assembler + capability bit; live routing (DIFF, mode select,
//! streaming validation, pipeline pre-assembly, dep resolution, micro-step
//! routing) lands in subsequent E-steps.
//!
//! E-2 adds the DIFF mode plumbing: a hand-rolled unified diff parser plus
//! the multi-region replay coordinator. The coordinator is exposed standalone
//! so it can be unit-tested before E-3 wires it into `task_runner::drive_task`.
//!
//! E-3 adds the [`mode`] submodule (ModeSelector + GenerationMode + the
//! deterministic 5-rule precedence table) plus an [`orchestrator`] submodule
//! whose `run_fim_step` / `run_diff_step` signatures are locked now so E-4
//! can implement the sidecar invocations without re-shaping the API. The
//! task_runner integration emits one `GenerationModeSelected` SSB row per
//! baby-step decision; the executor side falls back to STRUCTURED gracefully
//! when the provider client is missing (see `orchestrator.rs` doc-comment).
//!
//! E-4 adds the [`streaming`] submodule (StreamingValidator + LazyPatternGate
//! + ChunkParseGate + ImportValidationGate + TokenPositionTracker +
//! EarlyTermReason + ValidationDecision + reader-side aggregation). The
//! validator is library-tier — synchronous, no I/O — and is NOT wired into
//! `run_fim_step` / `run_diff_step` in E-4 (those stay STUBS per locked
//! decision #10). Runtime wiring lives in a dedicated post-E-7 WIRING phase
//! that plugs the validator into the streaming response handler.
//!
//! E-5a adds the [`pre_assembly`] submodule (PipelinePreAssembler +
//! PreAssembled + PreAssembledContext + DisableReason + Clock trait +
//! SystemTimeClock + pre_assembly_stats_for_payloads). Library-tier only —
//! synchronous, no async, no tokio, no I/O. SSB integration
//! (`pipeline_pre_assembled` action_type, SCHEMA_VERSION 32 → 33) is
//! DEFERRED to sub-phase E-5b once ssb.rs releases from session s-28564.
//! The reader-side aggregator [`pre_assembly_stats_for_payloads`] ships in
//! E-5a but returns zero stats today because the
//! `SsbPayload::PipelinePreAssembled` variant does not exist yet — the API
//! contract is locked so E-5b only needs to plug in the match arm.
//!
//! E-7 adds the [`file_router`] submodule (SpeculativeFileRouter +
//! RouteDecision + MicroStep + MicroStepType + file_router_stats_for_payloads).
//! Library-tier sync only — AST-based undefined-symbol detector that emits
//! MicroStep::CreateFile when model output references a symbol not defined
//! in the reachable AST of `target_file`. Cap of 2 micro-steps per baby step
//! (spec L118). SSB `micro_step_created` action_type DEFERRED to the BATCHED
//! schema bump that bundles E-5b + E-6 + E-7 wire forms (ssb.rs LOCKED by
//! session s-28564). ReplanRequest::Minor emission DEFERRED to the WIRING
//! phase (task_runner.rs:4638-4640 TODO).

// cross-lang-skip: focus_tunnel/file_router — Rust-only AST router; no
// Python sidecar twin (the sidecar consumes the SSB row, not the router).

pub mod dep_resolver;
pub mod diff;
pub mod file_router;
pub mod fim;
pub mod mode;
pub mod orchestrator;
pub mod pre_assembly;
pub mod streaming;

pub use diff::{
    apply_edits_to_buffer_via, replay_hunks, BufferEditor, DiffApplyError, DiffHunk,
    DiffParseError, DiffParser, FileReadWrite, HunkApplyOutcome, RealFs, ReplayPlan,
    ReplayedFile,
};
pub use fim::{FimContext, FimPromptAssembler, FimPromptError, FimPromptOutput};
pub use mode::{
    build_step_context_from, exec_mode_str, generation_mode_str,
    mode_distribution_for_payloads, parse_generation_mode_str, selection_rule_str,
    EditRegion, GenerationMode, ModeSelection, ModeSelector, SelectionRule, StepAction,
    StepContext,
};
pub use orchestrator::{
    run_diff_step, run_diff_step_with_provider, run_fim_step, run_fim_step_with_provider,
    DiffExecutionOutcome, DiffStepIo, FimExecutionOutcome, FocusTunnelExecError,
};
pub use pre_assembly::{
    pre_assembly_stats_for_payloads, Clock, DisableReason, PipelinePreAssembler,
    PreAssembled, PreAssembledContext, PreAssemblyStats, SystemTimeClock,
    DEFAULT_ADAPTIVE_THRESHOLD, DEFAULT_COLD_START_PASS_RATE, DEFAULT_HISTORY_WINDOW, MAX_DEPTH,
};
pub use streaming::{
    streaming_early_termination_stats_for_payloads, EarlyTermReason, EarlyTermWireForm,
    LanguageHint, StreamingEarlyTermStats, StreamingValidator, StreamingValidatorConfig,
    TokenPositionTracker, ValidationDecision, ValidationOutcome,
};
// B-28 E-6: library-tier DependencyResolver. SSB action_type
// `dependency_resolved` DEFERRED until ssb.rs releases from session
// s-28564 — reader-side aggregator
// `dependency_resolution_stats_for_payloads` ships with locked API and
// returns zero stats today (mirrors E-5a precedent). See
// `focus_tunnel/dep_resolver.rs` doc-comment for the pipeline order
// (deprecated -> allowlist -> registry -> autonomous-transitive ->
// human gate) and the locked anti-stale waiver for `ort`.
pub use dep_resolver::{
    dependency_resolution_stats_for_payloads, load_allowlist, Allowlist, ApplyOutcome,
    AutoApproveReason, BlockedReason, DepInstallExecutor, DepProposal, DependencyResolutionStats,
    DependencyResolver, HttpProbe, HttpProbeError, InstallAttempt, MissingImport,
    ProposalSource, ReqwestHttpProbe, ResolutionDecision, ResolutionPlan, Span,
    ORT_WAIVER_CRATE, ORT_WAIVER_DOC,
};
// B-28 E-7: library-tier SpeculativeFileRouter. SSB action_type
// `micro_step_created` DEFERRED until the batched ssb.rs schema bump
// (bundles E-5b + E-6 + E-7) — reader-side aggregator
// `file_router_stats_for_payloads` ships with locked API and returns zero
// stats today (mirrors E-5a / E-6 precedent). ReplanRequest::Minor emission
// also DEFERRED to the post-E-7 WIRING phase (task_runner.rs:4638-4640 TODO).
pub use file_router::{
    file_router_stats_for_payloads, FileRouterStats, MicroStep, MicroStepType, RouteDecision,
    SpeculativeFileRouter,
};
