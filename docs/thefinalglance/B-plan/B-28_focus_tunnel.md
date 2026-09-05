# B-28: Focus Tunnel (E-1 through E-7)
Objective: implement Focus Tunnel per focus-tunnel-design.txt and
post-phase4-implementation-order.txt GROUP E — FIM, diff, mode
selection, streaming validation, pipeline parallelism, dependency
resolution, file routing.
Estimated effort: 3-4 sessions.
Pre-requisites: B-15..B-20 (production hardening); B-22 (autonomous
mode) for full benefit.
Parallel-safe with: B-25, B-26, B-27.

## Context

Per focus-tunnel-design.txt and post-phase4-implementation-order.txt
STEPS E-1 through E-7:

Three generation modes per step:
- FIM (fill-in-middle): model outputs raw code between prefix/suffix.
  No tool schemas, no JSON wrapper. ~350 tokens saved per step.
  Single-region edits in existing files. Qwen2.5-Coder native FIM.
- DIFF: model outputs unified diff. System applies via 3-tier edit
  matching. ~30-50% smaller than full file. Multi-region.
- STRUCTURED: current default, fallback for new files and cloud.

Mode selection deterministic (no LLM):
- step.type == "edit_existing" AND single region AND model.supports_fim
  → FIM
- step.type == "edit_existing" AND multi-region → DIFF
- step.type == "create_file" → STRUCTURED
- else → STRUCTURED

Streaming validation:
- Token-level: import validation (deprecated_patterns), lazy code
  detection (todo!/pass/etc) — stop generation early
- Chunk-level: tree-sitter partial parse every ~50 tokens — early term
  on structural impossibility
- Saves ~90% of generation time when bad gen caught at token 50 vs 500

Pipeline parallelism:
- While step N validates, pre-assemble context for step N+1
- Saves 100-500ms per step when validation passes (~88% of time)
- Depth 1 only

Speculative file routing:
- Plan specifies target_file per step; model never specifies path
- AST mismatch detection → micro-step (create file) → retry

Automated dependency management:
- Detect missing deps from import statements
- Check anti-staleness deprecated_patterns
- Resolve version, install
- Model never runs cargo add / pip install
- Pre-approved allowlist for autonomous mode

Token budget impact: ~6,800 tokens saved on 30-step task = 27% of 25K.
Combined with ShortCoder: 45-65% effective budget increase.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\focus-tunnel-design.txt (FULL)
- F:\.ELAI_workflow\docs\specs\guide\post-phase4-implementation-order.txt
  STEPS E-1 through E-7

## Tasks

### E-1: FIM prompt assembly + model capability detection

1. ModelCapabilities.supports_fim: bool field. Set true for Qwen2.5-Coder,
   StarCoder, DeepSeek-Coder. Cloud=false.
2. FimPromptAssembler: prefix (header + imports + referenced symbols +
   pre-edit code) + suffix (post-edit + 50 lines or EOF). FIM tokens.
3. FIM output handler: insert at edit region. PostToolUse hooks validate.
4. Fallback: model doesn't support FIM → STRUCTURED.

### E-2: Diff parser + edit matching integration

5. Unified diff parser: hunks with file/line/added/removed.
6. Wire hunks through 3-tier edit matching (B-20).
7. Multi-region support.
8. Fallback on parse fail → STRUCTURED.

### E-3: Generation mode selection logic

9. ModeSelector: deterministic rules per spec.
10. PipelineSpec step gains preferred_mode: Option<GenerationMode>.
11. Metrics: track mode_distribution per task in SSB.

### E-4: Streaming validation framework

12. Token-level: import validation + lazy code detection.
13. Chunk-level: tree-sitter partial parse every ~50 tokens.
14. Early termination saves ~90% time on bad gens.
15. Metrics: early_termination_rate, tokens_saved.

### E-5: Pipeline parallelism for context pre-assembly

16. While step N validates: pre-assemble context for step N+1.
17. Discard if N fails (depends on N's output).
18. Adaptive depth: start at 1, disable if validation pass rate <80%.

### E-6: Automated dependency detection + resolution

19. Detect missing deps from import statements (per language).
20. Resolution pipeline: identify → check anti-staleness → query registry
    → conflict check → add manifest → install.
21. Approval gate: new deps require user confirmation (CLAUDE.md rule).
    Autonomous: pre-approved allowlist + transitive deps auto-approved.
22. Model NEVER runs cargo add / pip install.

### E-7: Speculative file routing + micro-step

23. Plan specifies target_file + edit_region. Orchestrator reads file,
    sets up FIM/DIFF context, writes output to that file.
24. Mismatch detection: AST analysis shows undefined symbols → new
    file/module needed.
25. Micro-step creation: orchestrator emits MINOR ReplanRequest with
    "create {path} with {type}" step. STRUCTURED mode. Retries original
    step with updated context.
26. Max 2 micro-steps per baby step (prevent infinite loops).

### Dashboard

27. Mode distribution panel: shows FIM/DIFF/STRUCTURED % per task.
28. Streaming validation metrics: early_termination_rate gauge.
29. Pipeline parallelism status: pre-assembly depth + waste rate.
30. Dependency resolution log: auto-installed deps with version + source.

## WIRING_CONTRACT

produces_symbols:
  - FimPromptAssembler
  - DiffParser
  - ModeSelector
  - StreamingValidator
  - PipelinePreAssembler
  - DependencyResolver
  - SpeculativeFileRouter
  - GenerationModeSelected
  - StreamingEarlyTerminated
  - PipelinePreAssembled
  - DependencyResolved
consumes_symbols:
  - sidecar with FIM token support (3J)
  - tools 3-tier edit matching (B-20)
  - rules deprecated_patterns (3F)
reachable_from:
  - task_runner.rs::drive_task (every baby step generation)
stub_ban:
  - model running cargo/pip directly disallowed
  - parallel pre-assembly depth >1 disallowed
  - missing deps without confirmation in INTERACTIVE mode disallowed
frontend_pair: dashboard mode distribution + streaming + pre-assembly +
                dep resolution panels
observable_effect:
  trigger: cd /d F:\.ELAI_workflow\.elai_cc && cargo test -p elai_cc_contracts --test b28_focus_tunnel_smoke_ssb -- --nocapture
  expect_ssb: generation_mode_selected
  expect_artifact: dashboard shows mode distribution and pre-assembly stats

## DETERMINISTIC_CHECK

```bash
# 1. FIM support detected
rg "supports_fim" .elai_cc/crates/providers/src
# 2. Mode selector
rg "ModeSelector::select" .elai_cc/crates/orchestrator/src
# 3. Dep resolution
rg "DependencyResolver::resolve" .elai_cc/crates/orchestrator/src
```

## ACCEPTANCE

- [ ] All 7 E-steps deliverables present
- [ ] FIM works for Qwen2.5-Coder edits
- [ ] DIFF works for multi-region
- [ ] Mode selection deterministic
- [ ] Streaming validation early-terminates bad gens
- [ ] Pipeline parallelism saves ~100-500ms per step
- [ ] Auto-dependency resolution + allowlist
- [ ] Speculative file routing + micro-step
- [ ] dashboard panels per spec

## Files touched

- ADD: crates/orchestrator/src/focus_tunnel/{fim, diff, mode, streaming,
  pre_assembly, dep_resolver, file_router}.rs
- MODIFY: crates/providers/src/* (FIM token support, ModelCapabilities)
- MODIFY: crates/tools/src/* (diff parser integration)
- MODIFY: crates/orchestrator/src/task_runner.rs (mode-aware generation)
- MODIFY: crates/orchestrator/src/ssb.rs (4 new action_types)
- MODIFY: crates/dashboard-v2/src-tauri/src/lib.rs (V2 — was V1
  crates/dashboard/src/commands.rs); ADD crates/dashboard-v2/src/lib/
  components/FocusTunnelPanel.svelte (V2 has no equivalent yet)
- CREATE: workflow/reasoning/B-28-decisions.md

## GATE FOR PART IV

After B-21 through B-28:
- Phase 4F-4J + 4K post-MVP fully implemented
- Cloud mode + focus tunnel operational
- Self-modification running with MSI tracking
- Empire integration (when accessible) via FactoryLine

Full system per architecture-overview.txt complete. Production-ready
for v1.0 ship.

## Next

End of B-plan. After B-28:
- Phase 5 deferred (Empire deep work — B-25 covers initial integration)
- Phase 3L (Project Intelligence) recommended after B-1..20 land
- Compiled pipeline (compiled-pipeline-design.txt) optional Phase 5+
- Spec amendments backfill — covered piecemeal by B-15 + B-19/20
