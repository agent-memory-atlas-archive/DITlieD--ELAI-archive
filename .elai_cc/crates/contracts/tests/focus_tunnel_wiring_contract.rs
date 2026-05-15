// B-28 WIRING-2 contract — live drive_task integration for focus_tunnel.
//
// Locks the runtime wiring surface added in WIRING-2:
//   E. cloud_mode::resolve_exec_mode(&BackendConfig) -> ExecMode
//   F. focus_tunnel::orchestrator::run_fim_step / run_diff_step real bodies
//   G. StreamingValidator on_token invocation (live stream consumer)
//   H. PipelinePreAssembler post-execute hook (prepare + on_step_outcome)
//   I. DependencyResolver post-execute hook (changed_files import scan)
//   J. SpeculativeFileRouter post-execute hook (route decision)
//
// Library-tier only — every test uses stub implementations of LlmProvider,
// BufferEditor, FileReadWrite, DepInstallExecutor, HttpProbe. No real I/O
// outside tempfile-scoped temp dirs.

use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use elai_core::tool_dispatcher::{EditMatchTier, EditRequest, SimilarityHint};
use elai_orchestrator::cloud_mode::{resolve_exec_mode, ExecMode};
use elai_orchestrator::dispatcher::{BackendConfig, BackendMode};
use elai_orchestrator::focus_tunnel::{
    dependency_resolution_stats_for_payloads, file_router_stats_for_payloads,
    pre_assembly_stats_for_payloads, run_diff_step_with_provider, run_fim_step_with_provider,
    Allowlist, BufferEditor,
    DepInstallExecutor, DependencyResolver, DepProposal, DiffApplyError, EditRegion,
    FileReadWrite, FocusTunnelExecError, GenerationMode, HttpProbe, HttpProbeError,
    InstallAttempt, MissingImport, ModeSelection, PipelinePreAssembler, RouteDecision,
    SelectionRule, SpeculativeFileRouter, StreamingValidator, StreamingValidatorConfig,
    SystemTimeClock, ValidationDecision,
};
use elai_orchestrator::research_tree::{BabyStepGrounding, BabyStepId, BabyStepSpec};
use elai_orchestrator::ssb::SsbPayload;
use elai_orchestrator::task_runner::BabyStepExecutionCtx;
use elai_plugins::import_extractor::ImportLanguage;
use elai_providers::{LlmFuture, LlmProvider, OllamaConfig};

// ---------------------------------------------------------------------------
// Stubs — implement provider/editor/fs/installer/probe traits for tests.
// ---------------------------------------------------------------------------

/// Stub LlmProvider that records the prompt it was called with and returns a
/// canned completion string. Captures prompt for assertion in FIM/DIFF tests.
struct StubLlmProvider {
    captured_prompt: Mutex<Option<String>>,
    canned_output: String,
}

impl StubLlmProvider {
    fn new(canned: impl Into<String>) -> Self {
        Self {
            captured_prompt: Mutex::new(None),
            canned_output: canned.into(),
        }
    }

    fn captured(&self) -> Option<String> {
        self.captured_prompt.lock().ok().and_then(|g| g.clone())
    }
}

impl LlmProvider for StubLlmProvider {
    fn complete<'a>(
        &'a self,
        prompt: &'a str,
        _config: &'a OllamaConfig,
    ) -> LlmFuture<'a, Vec<String>> {
        let prompt_owned = prompt.to_string();
        let canned = self.canned_output.clone();
        Box::pin(async move {
            if let Ok(mut g) = self.captured_prompt.lock() {
                *g = Some(prompt_owned);
            }
            Ok(vec![canned])
        })
    }
    fn embed<'a>(&'a self, _text: &'a str) -> LlmFuture<'a, Vec<f32>> {
        Box::pin(async move { Ok(vec![]) })
    }
    fn token_count(&self, _text: &str) -> Result<usize, elai_providers::ApiError> {
        Ok(0)
    }
    fn token_count_estimate(&self, text: &str) -> usize {
        text.len() / 4
    }
    fn model_name(&self) -> &str {
        "stub"
    }
}

/// BufferEditor that just records calls and replaces the buffer wholesale
/// with the concatenation of inserted_text values. Sufficient for the
/// happy-path replay test; tier output is fixed to Exact.
struct StubBufferEditor;

impl BufferEditor for StubBufferEditor {
    fn apply_edits_to_buffer(
        &self,
        buf: &str,
        edits: &[EditRequest],
    ) -> Result<
        (String, Vec<(EditMatchTier, Option<SimilarityHint>)>),
        DiffApplyError,
    > {
        let mut out = buf.to_string();
        let mut tiers = Vec::with_capacity(edits.len());
        for e in edits {
            // Naive: append new_text; matches "I executed each edit".
            out.push_str(&e.new_text);
            tiers.push((EditMatchTier::Exact, None));
        }
        Ok((out, tiers))
    }
}

/// In-memory FileReadWrite — records writes so tests can inspect.
#[derive(Default)]
struct StubFs {
    files: Mutex<std::collections::BTreeMap<PathBuf, String>>,
}

impl StubFs {
    fn with(seed: &[(&str, &str)]) -> Self {
        let me = Self::default();
        if let Ok(mut g) = me.files.lock() {
            for (k, v) in seed {
                g.insert(PathBuf::from(k), v.to_string());
            }
        }
        me
    }
    fn read_back(&self, path: &str) -> Option<String> {
        self.files
            .lock()
            .ok()
            .and_then(|g| g.get(&PathBuf::from(path)).cloned())
    }
}

impl FileReadWrite for StubFs {
    fn read(&self, path: &Path) -> std::io::Result<String> {
        self.files
            .lock()
            .ok()
            .and_then(|g| g.get(path).cloned())
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotFound, format!("{}", path.display()))
            })
    }
    fn write(&self, path: &Path, content: &str) -> std::io::Result<()> {
        if let Ok(mut g) = self.files.lock() {
            g.insert(path.to_path_buf(), content.to_string());
        }
        Ok(())
    }
}

/// DepInstallExecutor stub — records install calls; never spawns real
/// processes.
#[derive(Default)]
struct RecordingInstaller {
    calls: Mutex<Vec<DepProposal>>,
}

impl RecordingInstaller {
    fn call_count(&self) -> usize {
        self.calls.lock().map(|g| g.len()).unwrap_or(0)
    }
}

impl DepInstallExecutor for RecordingInstaller {
    fn install(&self, proposal: &DepProposal) -> InstallAttempt {
        if let Ok(mut g) = self.calls.lock() {
            g.push(proposal.clone());
        }
        InstallAttempt {
            proposal: proposal.clone(),
            success: true,
            stderr: None,
        }
    }
}

/// HttpProbe stub that always returns 200 — never makes real requests.
#[derive(Debug, Default)]
struct StubProbe200;

impl HttpProbe for StubProbe200 {
    fn probe<'a>(
        &'a self,
        _url: String,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<u16, HttpProbeError>> + Send + 'a>>
    {
        Box::pin(async move { Ok(200) })
    }
}

fn backend_config_local() -> BackendConfig {
    BackendConfig {
        mode: BackendMode::Local,
        sidecar_url: "http://localhost:8100".into(),
        model: "stub".into(),
        max_tools_local: 8,
        fallback_url: None,
    }
}

fn backend_config_api() -> BackendConfig {
    BackendConfig {
        mode: BackendMode::Api,
        sidecar_url: "http://localhost:8100".into(),
        model: "stub".into(),
        max_tools_local: 8,
        fallback_url: None,
    }
}

// ---------------------------------------------------------------------------
// Item E — resolve_exec_mode
// ---------------------------------------------------------------------------

#[test]
fn resolve_exec_mode_local_for_backend_local() {
    let cfg = backend_config_local();
    assert_eq!(resolve_exec_mode(&cfg), ExecMode::Local);
}

#[test]
fn resolve_exec_mode_cloud_for_backend_api() {
    let cfg = backend_config_api();
    assert_eq!(resolve_exec_mode(&cfg), ExecMode::Cloud);
}

#[test]
fn resolve_exec_mode_default_backend_is_cloud_routed() {
    // BackendMode::default() == Api today; resolve_exec_mode must reflect.
    let cfg = BackendConfig::default();
    let mode = resolve_exec_mode(&cfg);
    // Stable contract: default BackendConfig has Api mode => Cloud.
    assert_eq!(mode, ExecMode::Cloud);
}

// ---------------------------------------------------------------------------
// Item F — run_fim_step / run_diff_step real bodies
// ---------------------------------------------------------------------------

fn fim_sel() -> ModeSelection {
    ModeSelection {
        mode: GenerationMode::Fim,
        rule_fired: SelectionRule::FimAllConditionsMet,
        reason: "rule_3",
        preferred_was: None,
    }
}

fn diff_sel() -> ModeSelection {
    ModeSelection {
        mode: GenerationMode::Diff,
        rule_fired: SelectionRule::MultiRegionDiff,
        reason: "rule_4",
        preferred_was: None,
    }
}

fn ctx_for(id: u64) -> BabyStepExecutionCtx {
    BabyStepExecutionCtx {
        baby_step_id: id,
        task_summary: "wiring test".into(),
        escalation_message: None,
        mode_selection: None,
        session_credentials: std::collections::BTreeMap::new(),
    }
}

#[tokio::test]
async fn run_fim_step_returns_err_when_provider_missing() {
    // Preserves E-3 contract: without a provider, run_fim_step refuses.
    let ctx = ctx_for(1);
    let sel = fim_sel();
    let err = run_fim_step_with_provider(&ctx, &sel, None, None, None, None)
        .await
        .expect_err("must Err without provider");
    assert!(matches!(err, FocusTunnelExecError::MissingProvider));
}

#[tokio::test]
async fn run_fim_step_happy_path_writes_spliced_content() {
    use std::env;
    let tmp = tempfile::tempdir().expect("tempdir");
    let file = tmp.path().join("foo.rs");
    let original = "fn before() {}\n// HERE\nfn after() {}\n";
    std::fs::write(&file, original).expect("seed file");

    let provider = Arc::new(StubLlmProvider::new("let answer = 42;"));
    let ctx = ctx_for(2);
    let sel = fim_sel();
    let region = EditRegion {
        start_line: 2,
        end_line: 3,
    };

    let outcome = run_fim_step_with_provider(
        &ctx,
        &sel,
        Some(provider.clone()),
        Some(&file),
        Some(region),
        None,
    )
    .await
    .expect("FIM happy path");

    assert_eq!(outcome.changed_files, vec![file.clone()]);
    let captured = provider.captured().expect("provider called");
    assert!(
        captured.contains("<|fim_prefix|>"),
        "FIM prompt missing prefix sentinel: {captured}"
    );
    assert!(captured.contains("<|fim_suffix|>"));
    assert!(captured.contains("<|fim_middle|>"));

    let written = std::fs::read_to_string(&file).expect("read back");
    assert!(
        written.contains("let answer = 42;"),
        "spliced output not written: {written}"
    );
    let _ = env::var("PATH"); // sanity
}

#[tokio::test]
async fn run_fim_step_missing_edit_region_errors() {
    let provider = Arc::new(StubLlmProvider::new("ignored"));
    let tmp = tempfile::tempdir().expect("tempdir");
    let file = tmp.path().join("x.rs");
    std::fs::write(&file, "fn x() {}\n").expect("seed");
    let ctx = ctx_for(3);
    let sel = fim_sel();
    let err = run_fim_step_with_provider(&ctx, &sel, Some(provider), Some(&file), None, None)
        .await
        .expect_err("must Err without region");
    assert!(matches!(
        err,
        FocusTunnelExecError::UnsupportedMode { .. } | FocusTunnelExecError::MissingProvider
    ));
}

#[tokio::test]
async fn run_diff_step_returns_err_when_provider_missing() {
    let ctx = ctx_for(4);
    let sel = diff_sel();
    let err = run_diff_step_with_provider(&ctx, &sel, None, None).await.expect_err("must Err");
    assert!(matches!(err, FocusTunnelExecError::MissingProvider));
}

#[tokio::test]
async fn run_diff_step_falls_back_on_not_unified_diff() {
    // Model returns garbage (no `---`/`+++` headers).
    let provider = Arc::new(StubLlmProvider::new("just some text, no diff here"));
    let ctx = ctx_for(5);
    let sel = diff_sel();
    let err = run_diff_step_with_provider(&ctx, &sel, Some(provider), None)
        .await
        .expect_err("must Err on garbage output");
    assert!(matches!(err, FocusTunnelExecError::DiffParseFailed));
}

#[tokio::test]
async fn run_diff_step_happy_path_applies_hunk_through_replay() {
    // Construct a minimal unified diff with one hunk targeting hello.rs.
    let file_str = "hello.rs";
    let diff_text = format!(
        "--- a/{}\n+++ b/{}\n@@ -1,1 +1,2 @@\n fn hello() {{}}\n+fn world() {{}}\n",
        file_str, file_str
    );
    let provider = Arc::new(StubLlmProvider::new(diff_text));

    let editor = StubBufferEditor;
    let fs = StubFs::with(&[("hello.rs", "fn hello() {}\n")]);
    let io = elai_orchestrator::focus_tunnel::orchestrator::DiffStepIo::new(&editor, &fs);

    let ctx = ctx_for(6);
    let sel = diff_sel();
    let outcome = run_diff_step_with_provider(&ctx, &sel, Some(provider), Some(io))
        .await
        .expect("DIFF happy path");
    assert!(
        outcome.hunks_applied >= 1,
        "expected hunks_applied >= 1; got {}",
        outcome.hunks_applied
    );
    assert!(
        !outcome.changed_files.is_empty(),
        "changed_files must be populated"
    );
}

// ---------------------------------------------------------------------------
// Item G — StreamingValidator integrated with stub stream
// ---------------------------------------------------------------------------

#[test]
fn streaming_validator_terminates_on_lazy_pattern() {
    // Drive a fake stream into on_token; expect EarlyTerm on `todo!()`.
    use elai_orchestrator::focus_tunnel::LanguageHint;
    let mut v = StreamingValidator::new(StreamingValidatorConfig {
        language: LanguageHint::Rust,
        max_gen_tokens: 1024,
        step_id: "baby_step_77".into(),
        mode: "fim".into(),
        ..StreamingValidatorConfig::default()
    });
    // Tokens that trigger the lazy pattern gate.
    let mut decisions = Vec::new();
    for tok in ["fn foo() { ", "todo!()", " }"] {
        decisions.push(v.on_token(tok));
    }
    assert!(
        decisions
            .iter()
            .any(|d| matches!(d, ValidationDecision::EarlyTerm { .. })),
        "expected at least one EarlyTerm; got {decisions:?}"
    );
}

#[test]
fn streaming_validator_passes_clean_token_stream() {
    use elai_orchestrator::focus_tunnel::LanguageHint;
    let mut v = StreamingValidator::new(StreamingValidatorConfig {
        language: LanguageHint::Rust,
        max_gen_tokens: 1024,
        step_id: "baby_step_78".into(),
        mode: "fim".into(),
        ..StreamingValidatorConfig::default()
    });
    for tok in ["fn foo() { ", "let x = 1;", " x ", "}"] {
        let d = v.on_token(tok);
        assert!(
            matches!(d, ValidationDecision::Continue),
            "clean stream should not early-term; got {d:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Item H — PipelinePreAssembler hook (prepare + on_step_outcome + discard)
// ---------------------------------------------------------------------------

fn dummy_spec(id: u64) -> BabyStepSpec {
    BabyStepSpec {
        id: BabyStepId(id),
        task: "next step".into(),
        estimated_tokens: 100,
        grounding: BabyStepGrounding::default(),
        tdd: None,
        acceptance: None,
        preferred_mode: None,
        target_file: None,
        edit_region: None,
    }
}

#[test]
fn pre_assembler_prepares_after_step_outcome_pass() {
    let mut pa = PipelinePreAssembler::new();
    let spec = dummy_spec(2);
    let target = vec![PathBuf::from("crates/foo/src/lib.rs")];

    pa.on_step_outcome(1, true, &target);
    let result = pa.prepare(2, &spec, &target, true, ExecMode::Local);
    // After a pass + no in-flight, prepare must succeed (not Disabled or
    // AlreadyHaveOneInFlight on first call).
    use elai_orchestrator::focus_tunnel::PreAssembled;
    assert!(matches!(result, PreAssembled::Ready { .. }));
}

#[test]
fn pre_assembler_discards_on_target_file_change() {
    let mut pa = PipelinePreAssembler::new();
    let spec = dummy_spec(2);
    let targets_a = vec![PathBuf::from("a.rs")];
    let targets_b = vec![PathBuf::from("b.rs")];
    // Prepare for step 2 targeting a.rs.
    let _ = pa.prepare(2, &spec, &targets_a, true, ExecMode::Local);
    assert!(pa.has_in_flight());
    // Step 1 outcome: passed, but next target is b.rs (disjoint).
    pa.on_step_outcome(1, true, &targets_b);
    assert!(
        !pa.has_in_flight(),
        "must discard in-flight payload when next target_files diverge"
    );
}

// ---------------------------------------------------------------------------
// Item I — DependencyResolver post-execute integration
// ---------------------------------------------------------------------------

#[tokio::test]
async fn dep_resolver_allowlist_auto_approves_with_recording_installer() {
    let mut allowlist = Allowlist::default();
    allowlist.insert_many("cargo", ["tokio".to_string()].into_iter());
    let resolver = DependencyResolver::new(
        allowlist,
        false,
        Arc::new(SystemTimeClock),
        Arc::new(StubProbe200),
    );
    let plan = resolver
        .resolve(
            vec![MissingImport {
                language: ImportLanguage::Rust,
                name: "tokio".into(),
                source_span: None,
                transitive_of: None,
            }],
            &[],
        )
        .await
        .expect("resolver");
    assert_eq!(plan.decisions.len(), 1);
    let installer = RecordingInstaller::default();
    let outcome = resolver.apply(&plan, &installer);
    assert_eq!(installer.call_count(), 1);
    assert_eq!(outcome.install_results.len(), 1);
    assert!(outcome.install_results[0].success);
}

#[tokio::test]
async fn dep_resolver_not_allowlisted_queue_for_human_skips_installer() {
    let allowlist = Allowlist::default(); // empty
    let resolver = DependencyResolver::new(
        allowlist,
        false,
        Arc::new(SystemTimeClock),
        Arc::new(StubProbe200),
    );
    let plan = resolver
        .resolve(
            vec![MissingImport {
                language: ImportLanguage::Rust,
                name: "serde".into(),
                source_span: None,
                transitive_of: None,
            }],
            &[],
        )
        .await
        .expect("resolver");
    let installer = RecordingInstaller::default();
    let _ = resolver.apply(&plan, &installer);
    assert_eq!(
        installer.call_count(),
        0,
        "QueueForHuman MUST NOT invoke installer"
    );
}

// ---------------------------------------------------------------------------
// Item J — SpeculativeFileRouter post-execute hook
// ---------------------------------------------------------------------------

#[test]
fn file_router_emits_needs_micro_step_on_undefined_symbol_rust() {
    let mut router = SpeculativeFileRouter::new(2);
    let target = PathBuf::from("src/lib.rs");
    let target_content = "fn known() {}\n";
    // Model output that references an undefined symbol (Module casing).
    let model_output = "use crate::widgets::Foo;\nfn caller() { widgets::Foo::new(); }\n";
    let dec = router.route(
        &target,
        target_content,
        None,
        model_output,
        ImportLanguage::Rust,
    );
    // Either NeedsMicroStep or ParseSkipped is acceptable depending on the
    // AST heuristic; the contract is that we DO NOT silently Proceed when
    // there is an undefined symbol path.
    assert!(
        matches!(dec, RouteDecision::NeedsMicroStep(_) | RouteDecision::ParseSkipped { .. }),
        "router should not silently Proceed on undefined-symbol input: got {dec:?}"
    );
}

#[test]
fn file_router_cap_exhausted_after_max_micro_steps() {
    let mut router = SpeculativeFileRouter::new(1);
    // Pre-consume the single allotted micro-step.
    router.record_micro_step_completion();
    let target = PathBuf::from("src/lib.rs");
    let target_content = "fn known() {}\n";
    let model_output = "fn caller() { undefined_thing(); }\n";
    let dec = router.route(
        &target,
        target_content,
        None,
        model_output,
        ImportLanguage::Rust,
    );
    // With the budget consumed and undefined references present, the router
    // MUST emit MicroStepCapExhausted instead of producing more replans.
    // ParseSkipped is also valid for cases where AST can't determine.
    // B-28 WIRING-2 fix BLOCK 1: Proceed is NOT a valid outcome here —
    // accepting it allowed a regression where the router silently proceeded
    // despite an exhausted micro-step budget. The security boundary is that
    // exhausted-budget + undefined-symbol output MUST surface a cap signal
    // or a parse-skip, never a silent green light.
    assert!(
        matches!(
            dec,
            RouteDecision::MicroStepCapExhausted { .. }
                | RouteDecision::ParseSkipped { .. }
        ),
        "unexpected decision under cap-exhausted budget: {dec:?}"
    );
}

// ---------------------------------------------------------------------------
// SSB aggregator round-trip via real rows (sanity — wiring emits these).
// ---------------------------------------------------------------------------

#[test]
fn pre_assembly_stats_aggregator_handles_typed_payload() {
    use elai_orchestrator::ssb::PipelinePreAssembledPayload;
    let rows = vec![SsbPayload::PipelinePreAssembled(
        PipelinePreAssembledPayload {
            step_idx: 1,
            pass_rate_bp: 9000,
            was_discarded: false,
            depth_at_time: 1,
            bytes_assembled: 4096,
        },
    )];
    let stats = pre_assembly_stats_for_payloads(&rows);
    // The aggregator returns a populated stats struct now that the variant
    // exists; the exact field shape is owned by pre_assembly.rs. We only
    // assert it does not panic and returns SOMETHING by calling the public
    // surface (Debug fmt is sufficient).
    let _ = format!("{stats:?}");
}

#[test]
fn dep_resolution_stats_aggregator_handles_typed_payload() {
    use elai_orchestrator::ssb::DependencyResolvedPayload;
    let rows = vec![SsbPayload::DependencyResolved(DependencyResolvedPayload {
        language: "rust".into(),
        name: "tokio".into(),
        decision: "auto_approve".into(),
        reason: "allowlist_match".into(),
        install_command: Some("cargo add tokio".into()),
    })];
    let stats = dependency_resolution_stats_for_payloads(&rows);
    let _ = format!("{stats:?}");
}

#[test]
fn file_router_stats_aggregator_handles_typed_payload() {
    use elai_orchestrator::ssb::MicroStepCreatedPayload;
    let rows = vec![SsbPayload::MicroStepCreated(MicroStepCreatedPayload {
        parent_step_id: 11,
        created_path: "src/foo.rs".into(),
        type_hint: "Module".into(),
        reason: "undefined_symbol".into(),
        count: 1,
    })];
    let stats = file_router_stats_for_payloads(&rows);
    let _ = format!("{stats:?}");
}

// ---------------------------------------------------------------------------
// B-28 WIRING-2 fix BLOCK 2: BabyStepSpec.target_file + edit_region plumbing.
// These tests pin the contract that:
//   (a) a spec carrying both fields makes Rule 3 (FIM) reachable through
//       build_step_context_from -> ModeSelector, and run_fim_step_with_provider
//       executes the real splice when the orchestrator forwards those fields.
//   (b) the DIFF dispatcher pass-through works end-to-end when an editor
//       is wired (matching the production-equivalent test path).
// ---------------------------------------------------------------------------

#[tokio::test]
async fn fim_step_executes_when_target_file_and_edit_region_set() {
    // BLOCK 2 contract: build a synthetic BabyStepSpec with both fields,
    // confirm the StepContext carries the region, and run_fim_step_with_provider
    // splices model output into the target file. This is the regression test
    // for the legacy "BabyStepSpec lacks target_file/edit_region (deferred)"
    // UnsupportedMode path: it MUST NOT fire when both fields are set.
    use elai_orchestrator::focus_tunnel::{build_step_context_from, ModeSelector};

    let tmp = tempfile::tempdir().expect("tempdir");
    let file = tmp.path().join("synth.rs");
    let original = "fn before() {}\n// FIM_TARGET\nfn after() {}\n";
    std::fs::write(&file, original).expect("seed file");

    let region = EditRegion { start_line: 2, end_line: 3 };
    // Build a synthetic BabyStepSpec with both fields populated.
    let spec = BabyStepSpec {
        id: BabyStepId(42),
        task: "edit fn before to add helper".into(),
        estimated_tokens: 100,
        grounding: BabyStepGrounding::default(),
        tdd: None,
        acceptance: None,
        preferred_mode: None,
        target_file: Some(file.clone()),
        edit_region: Some(region),
    };

    // BLOCK 2b: build_step_context_from MUST forward the region verbatim.
    let ctx = build_step_context_from(&spec);
    assert_eq!(
        ctx.edit_region,
        Some(region),
        "build_step_context_from must propagate spec.edit_region"
    );

    // The ModeSelector with a FIM-capable model + EditExisting + region
    // MUST land on GenerationMode::Fim (Rule 3).
    let mut features = elai_orchestrator::context_budget::ModelFeatures::none();
    features.supports_fim = true;
    let selection = ModeSelector::new().select(&ctx, &features, ExecMode::Local);
    assert_eq!(
        selection.mode,
        GenerationMode::Fim,
        "Rule 3 should fire when edit_region is set on the spec"
    );

    // Now run the FIM dispatcher with a stub provider. The synthetic
    // run wires through the same signature the task_runner's BLOCK 2c
    // branch uses (Some(target), Some(region)).
    let provider = Arc::new(StubLlmProvider::new("let answer = 42;"));
    let ctx_exec = ctx_for(spec.id.0);
    let outcome = run_fim_step_with_provider(
        &ctx_exec,
        &selection,
        Some(provider.clone()),
        Some(file.as_path()),
        Some(region),
        None,
    )
    .await
    .expect("FIM happy path with spec-populated target/region");
    assert_eq!(
        outcome.changed_files,
        vec![file.clone()],
        "changed_files must mirror the spec target_file"
    );
    let written = std::fs::read_to_string(&file).expect("read back");
    assert!(
        written.contains("let answer = 42;"),
        "spliced output not written: {written}"
    );
}

#[tokio::test]
async fn diff_step_executes_with_real_dispatcher() {
    // BLOCK 2 contract: when the orchestrator wires a real BufferEditor
    // (stub here standing in for InProcessToolDispatcher in production),
    // run_diff_step_with_provider replays hunks through the editor surface
    // without returning UnsupportedMode. The wiring contract test from
    // WIRING-2 already exercised this with StubBufferEditor; this test
    // pins the same surface as a BLOCK 2 acceptance signal so a regression
    // surface (editor=None always errors) is caught at the contract layer.
    let file_str = "wired.rs";
    let diff_text = format!(
        "--- a/{}\n+++ b/{}\n@@ -1,1 +1,2 @@\n fn hello() {{}}\n+fn world() {{}}\n",
        file_str, file_str
    );
    let provider = Arc::new(StubLlmProvider::new(diff_text));

    let editor = StubBufferEditor;
    let fs = StubFs::with(&[("wired.rs", "fn hello() {}\n")]);
    let io = elai_orchestrator::focus_tunnel::orchestrator::DiffStepIo::new(&editor, &fs);

    let ctx = ctx_for(99);
    let sel = diff_sel();
    let outcome = run_diff_step_with_provider(&ctx, &sel, Some(provider), Some(io))
        .await
        .expect("DIFF should run when editor + fs are wired");
    assert!(
        outcome.hunks_applied >= 1,
        "expected hunks_applied >= 1 with real dispatcher path; got {}",
        outcome.hunks_applied
    );
    assert!(
        !outcome.changed_files.is_empty(),
        "real dispatcher path must populate changed_files"
    );
}
