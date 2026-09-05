# Verification Report: focus-tunnel-design.txt
Source: F:\.ELAI_workflow\docs\specs\guide\focus-tunnel-design.txt
Date: 2026-05-17
Overall status: MOSTLY_DONE
Confidence: HIGH

## Summary
The focus tunnel is substantially implemented. All seven library-tier submodules exist under
crates/orchestrator/src/focus_tunnel/ and are wired into task_runner. The generation-mode
selection (FIM/DIFF/STRUCTURED), FIM prompt assembly, diff parsing, streaming validator,
pipeline pre-assembler, dependency resolver, and speculative file router are all present in
source and exercised by contract tests. The primary gap is that the production TaskRunnerConfig
construction site still defaults all three focus-tunnel Arcs to None, so the resolver, pre-
assembler, and file-router hooks execute only when callers opt-in; FIM/DIFF execution itself
is gated behind ELAI_FOCUS_TUNNEL_EXECUTE=1. Streaming validation is defined but the comment
in orchestrator.rs:5-7 notes it is not yet wired into run_fim_step/run_diff_step at the call
site (E-4 spec only defines the library surface).

## Mechanisms verified DONE

- THREE GENERATION MODES enum (Fim / Diff / Structured) — evidence:
  crates/orchestrator/src/focus_tunnel/mode.rs:154 (ModeSelector struct),
  mode.rs:401 (parse_generation_mode_str), mode.rs:412 (generation_mode_str)

- Deterministic mode selection logic (ModeSelector::select, SelectionRule) — evidence:
  crates/orchestrator/src/focus_tunnel/mode.rs:301 (build_step_context_from),
  mode.rs:563 (select test), contracts/tests/focus_tunnel_e3_contract.rs:143
  (edit_existing_with_region_and_fim_capable_returns_fim),
  focus_tunnel_e3_contract.rs:99 (cloud_execution_returns_structured / CloudOverride)

- Cloud-override rule: cloud ExecMode always returns STRUCTURED — evidence:
  crates/orchestrator/src/focus_tunnel/mode.rs:466-467 (CloudOverride rule + assertion)

- FIM prompt assembler with Qwen2.5-Coder sentinel tokens (fim_prefix/fim_suffix/fim_middle),
  token-budgeted prefix/suffix, hint injection — evidence:
  crates/orchestrator/src/focus_tunnel/fim.rs:161-165 (QWEN_FIM_PREFIX/SUFFIX/MIDDLE push),
  fim.rs:146-150 (hint filter), fim.rs:251 (suffix truncation path)

- FIM model capability detection (supports_fim flag) — evidence:
  crates/orchestrator/src/context_budget.rs:554 (detect_model_capability_profile),
  task_runner.rs:4935 (supports_fim field read)

- Diff output parser (DiffParser, unified diff, 3-tier edit matching) — evidence:
  crates/orchestrator/src/focus_tunnel/diff.rs:758 (DiffParser::new().parse),
  contracts/tests/focus_tunnel_e2_contract.rs:165-546 (DiffParser contract tests),
  focus_tunnel_wiring_contract.rs:364 (FocusTunnelExecError::DiffParseFailed fallback)

- Diff fallback to STRUCTURED on DiffParseFailed — evidence:
  crates/contracts/tests/focus_tunnel_wiring_contract.rs:358-364

- Streaming validator: token-level lazy-code gate (todo!, unimplemented!) with code-body
  position filter — evidence:
  crates/orchestrator/src/focus_tunnel/streaming.rs:451-453 (LAZY regex),
  streaming.rs:967-983 (lazy_gate_skips_comment test),
  streaming.rs:940 (lazy_gate_fires_on_rust_todo test)

- Streaming validator: chunk-level brace-balance gate (UnbalancedBraces) — evidence:
  crates/orchestrator/src/focus_tunnel/streaming.rs:50-55 (DEFAULT_BRACE_DIVERGENCE_THRESHOLD,
  DEFAULT_BRACE_MIN_TOKENS), streaming.rs:654 (terminate on UnbalancedBraces),
  contracts/tests/focus_tunnel_e4_contract.rs:519 (over_close_braces_triggers_unbalanced_braces)

- Streaming validator: token-budget / OutputTooLong gate (2x expected factor) — evidence:
  crates/orchestrator/src/focus_tunnel/streaming.rs:57-60 (DEFAULT_EXPECTED_FACTOR=2),
  streaming.rs:127 (EarlyTermReason::OutputTooLong), streaming.rs:143 (output_too_long wire),
  contracts/tests/focus_tunnel_e4_contract.rs:149 (output_too_long_triggers_at_expected_factor)

- Streaming validator: tree-sitter chunk-level partial parse (PARSE_ERROR_DEBOUNCE) — evidence:
  crates/orchestrator/src/focus_tunnel/streaming.rs:23-27 (tree-sitter parser owned per language),
  streaming.rs:63-66 (PARSE_ERROR_DEBOUNCE=2 constant)

- Streaming validator: deprecated-import detection mid-stream (check_deprecated_imports) — evidence:
  crates/orchestrator/src/focus_tunnel/streaming.rs:34-36 (check_deprecated_imports import),
  streaming.rs:706 (check_deprecated_imports call in chunk gate),
  streaming.rs:129 (EarlyTermReason::DeprecatedImport)

- Pipeline pre-assembler (PipelinePreAssembler): adaptive pass-rate window, depth=1 cap,
  on_step_outcome recording, SSB PipelinePreAssembled beacon — evidence:
  crates/orchestrator/src/focus_tunnel/pre_assembly.rs:262 (set_adaptive_threshold),
  pre_assembly.rs:386 (on_step_outcome history), pre_assembly.rs:308 (in_flight depth-1 cap),
  task_runner.rs:2823-2857 (Item H hook, SSB write)

- Dependency resolver (DependencyResolver): Allowlist, auto-approve, queue-for-human,
  per-ecosystem registry URL, install_command generation, DependencyResolutionStats — evidence:
  crates/orchestrator/src/focus_tunnel/dep_resolver.rs:304 (Allowlist::contains),
  dep_resolver.rs:796 (DependencyResolutionStats struct),
  dep_resolver.rs:904 (install_command_per_language test),
  dep_resolver.rs:895 (allowlist_round_trips_per_ecosystem),
  contracts/tests/focus_tunnel_e6_contract.rs:237 (allowlist auto-approve),
  contracts/tests/focus_tunnel_e6_contract.rs:393 (transitive auto-approve)

- Allowlist backed by elai_cc.toml (load_allowlist) — evidence:
  contracts/tests/focus_tunnel_e6_contract.rs:599-615 (load_allowlist from elai_cc.toml path)

- Speculative file router (SpeculativeFileRouter): AST-based undefined-symbol detection,
  MicroStep emit, max-micro-steps cap=2, MicroStepType classification — evidence:
  crates/orchestrator/src/focus_tunnel/file_router.rs:1-45 (module doc + design contract),
  file_router.rs:65 (MicroStepType enum), file_router.rs:145 (new(max_micro_steps)),
  file_router.rs:171 (current_micro_steps increment),
  contracts/tests/focus_tunnel_e7_contract.rs:82-91 (micro_step emitted assertions)

- SsbActionType::MicroStepCreated registered — evidence:
  crates/orchestrator/src/ssb.rs:1264 (micro_step_created => MicroStepCreated)

- FIM/DIFF wired into task_runner drive_task behind ELAI_FOCUS_TUNNEL_EXECUTE=1 — evidence:
  crates/orchestrator/src/task_runner.rs:5143-5206 (B-28 WIRING-2 block, run_fim_step_with_provider call)

- GenerationModeSelected SSB beacon emitted per baby-step — evidence:
  task_runner.rs:4919-4922 (GenerationModeSelected payload construction and SSB write)

- Dashboard GenerationMode distribution display — evidence:
  crates/dashboard-v2/src-tauri/src/commands.rs:5895-5898 (mode_str from GenerationMode::Fim/Diff dist)

## Mechanisms PARTIAL

- Streaming validator wired into run_fim_step / run_diff_step
  - exists: crates/orchestrator/src/focus_tunnel/streaming.rs (full library), StreamingValidator
    called in contracts/tests/focus_tunnel_wiring_contract.rs:405,428
  - gap: orchestrator.rs:5-7 doc comment states "Wired into run_fim_step / run_diff_step in a
    dedicated post-E-7 WIRING phase; E-4 only defines the library surface." No call site
    invoking StreamingValidator inside run_fim_step_with_provider or run_diff_step_with_provider
    was found in orchestrator.rs or task_runner.rs.

- Pipeline pre-assembler: context pre-assembly for step N+1 (lookahead)
  - exists: crates/orchestrator/src/focus_tunnel/pre_assembly.rs (PipelinePreAssembler,
    prepare(), in_flight tracking, adaptive threshold)
  - gap: task_runner.rs:2793 sets focus_tunnel_pre_assembler: None in the default
    ResolvedConfig. The Item H hook records outcomes but the comment at task_runner.rs:2824-2825
    notes "Pre-assembly of step N+1 is deferred to the production wiring patch that owns the
    spec lookup." The actual prepare() call for lookahead is not present in task_runner.

- Dependency resolver post-execute hook wired in task_runner (Item I)
  - exists: task_runner.rs:2860-2879 (Item I block reads changed_files, calls dep_resolver)
  - gap: task_runner.rs:2794 defaults focus_tunnel_dep_resolver: None; the production
    TaskRunnerConfig construction site that sets a real Arc is noted as a future patch.
    Hook executes only if caller populates the Arc.

- Speculative file router post-execute hook wired in task_runner (Item J)
  - exists: task_runner.rs:2795 (focus_tunnel_file_router field), file_router.rs full impl
  - gap: task_runner.rs:2795 defaults to None; the ReplanRequest::Minor emission via
    Fsm::submit_replan is explicitly marked DEFERRED in file_router.rs:21-28. AST mismatch
    detection will not trigger until the production Arc is populated and the replan path lands.

## Mechanisms MISSING

- Constrained decoding blocking forbidden token sequences during generation (e.g. stop
  "use blake2::" mid-stream at the token level via the constrained decoder)
  - searched: constrained_decoding, grammar_mode, token_sequence_block, logit_bias,
    forbidden_token; no evidence of a constrained-decoder integration in streaming.rs or
    server.py that blocks individual token sequences. The streaming validator fires AFTER the
    token is emitted and terminates the stream; it does not prevent the token from being sampled.

## Mechanisms UNVERIFIABLE

- Sidecar FIM endpoint (3J): whether server.py accepts fim_prefix/fim_suffix/fim_middle
  in the Anthropic-protocol request and routes to the model's native FIM head
  - reason: elai.map has no FIM-specific route in server.py SCIP refs; the sidecar is a
    remote deployment; behavior of the Qwen2.5-Coder 3B NF4 FIM head under the Anthropic
    protocol wrapper cannot be verified without live inference.

- Token savings numbers (350 tok / step, 27% of 25K budget freed)
  - reason: arithmetic estimates from design doc; no measured benchmark data exists in the
    codebase (benches/model_capability/run_benchmark.py is a harness template, not FIM results).

- Adaptive pipeline depth disable when pass-rate < 80%
  - reason: pre_assembly.rs implements adaptive_threshold field and set_adaptive_threshold(),
    but the threshold comparison that would skip prepare() when pass rate is too low was
    not observable in the task_runner hook (pre-assembly of N+1 is deferred per comment).
    Cannot verify the 80% threshold is enforced end-to-end.

## What must be done (action list)

- [ ] Wire StreamingValidator into run_fim_step_with_provider and run_diff_step_with_provider
  in crates/orchestrator/src/focus_tunnel/orchestrator.rs (post-E-7 WIRING phase per doc comment)
- [ ] Populate focus_tunnel_pre_assembler Arc in production TaskRunnerConfig construction site
  (task_runner.rs:2793) and add the prepare() call for step N+1 lookahead
- [ ] Populate focus_tunnel_dep_resolver Arc in production TaskRunnerConfig (task_runner.rs:2794)
- [ ] Wire ReplanRequest::Minor emission from SpeculativeFileRouter into Fsm::submit_replan
  (file_router.rs:21-28 deferred note); populate focus_tunnel_file_router Arc (task_runner.rs:2795)
- [ ] Add constrained-decoder / logit-bias token-blocking for deprecated imports at generation
  time (currently only post-token termination is implemented)
- [ ] Run FIM vs STRUCTURED benchmark (extend benches/model_capability/run_benchmark.py) to
  ground the 350-token-per-step savings claim with measured data

## Notes
The focus_tunnel module directory contains: fim.rs, diff.rs, mode.rs, streaming.rs,
pre_assembly.rs, dep_resolver.rs, file_router.rs, orchestrator.rs — all seven A-steps
(A-51 through A-57) have a corresponding library implementation. Contract test coverage is
extensive (focus_tunnel_e2 through e7 + wiring contract). The primary gap is production
wiring: three Arcs default to None and the streaming validator is explicitly deferred to a
post-E-7 wiring phase. The feature is functional in isolation (unit + contract tests pass)
but requires the deferred wiring work before it operates in the live baby-step execution path.
