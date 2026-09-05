# Verification Report: compiled-pipeline-design
Source: F:\.ELAI_workflow\docs\specs\guide\compiled-pipeline-design.txt
Date: 2026-05-17
Overall status: PARTIAL
Confidence: HIGH

## Summary

The four designs in this document (Compiled Pipeline, Reactive Background Takeover, Cumulative Token Tracking, Safety Sandwich) are all PROPOSAL-status features tagged as C-group work (post-Phase-4). None of the three new types required for the compiled pipeline (CompiledPipeline, TemplateCrystallizer, CompiledExecutor) exist in the codebase. The BackgroundTaskManager with soft-timeout auto-detach does not exist; only explicit user-initiated run_in_background is implemented. A partial foundation exists for Designs 3 and 4: token accumulation from API responses is live in cloud_mode.rs but never fed back to context_budget.rs, and the constrained decoding infrastructure (lm-format-enforcer, Track C-B) is present in the sidecar and grounding_client.rs but not connected to phase_d_extract_facts in research_pipeline.rs.

## Mechanisms verified DONE

- ExtractedFact struct with exact_quote field (optional) — evidence: crates/core/src/research_types.rs:70,80
- phase_d_extract_facts sets snippet as both claim and exact_quote — evidence: crates/orchestrator/src/research_pipeline.rs:249-273
- phase_g_post_verify exists (7-phase oracle pipeline) — evidence: crates/orchestrator/src/research_pipeline.rs:552-609
- SpendAccumulator accumulates input/output/cache tokens from API responses (cloud_mode.rs) — evidence: crates/orchestrator/src/cloud_mode.rs:302-394
- Constrained decoding infrastructure (lm-format-enforcer + Track C-B) live in sidecar — evidence: python/elai_inference/server.py:49,1505,1566,1938; _ERR_CONSTRAINED_DECODING_UNAVAILABLE constant at server.py:117
- GroundingClient supports response_format opt-in for JSON-schema constrained decoding — evidence: crates/orchestrator/src/grounding_client.rs:161,194,226
- Contract test for constrained decoding (C-B track) exists — evidence: crates/contracts/tests/research_decompose_constrained.rs:1-41
- Bash tool supports user-requested run_in_background (fires-and-forgets, returns background_task_id) — evidence: crates/orchestrator/src/bash.rs:84-114; crates/tools/src/lib.rs:5280-5306
- BashCommandOutput carries background_task_id and assistant_auto_backgrounded fields — evidence: crates/orchestrator/src/bash.rs:57-62
- Context budget allocator (context_budget.rs) is fully implemented with profile-based allocation — evidence: crates/orchestrator/src/context_budget.rs (multiple lines 570-2824 via elai.map)

## Mechanisms PARTIAL

- Token accumulation from API response fed to context budget (Design 3: CumulativeTokenTracker)
  - exists: SpendAccumulator in crates/orchestrator/src/cloud_mode.rs:302 accumulates input/cache tokens per round-trip from API usage fields; parsed from openai_compat provider (crates/providers/src/providers/openai_compat.rs:363)
  - gap: SpendAccumulator output flows to SSB CloudSpendPayload for billing display only; no CumulativeTokenTracker struct; no feedback path from actual API token count back to context_budget.rs for recalibration; context_budget.rs still uses estimate-based allocation exclusively

- Safety Sandwich: deterministic input validation before extraction (Design 4 Phase 1)
  - exists: SourceTier classification exists (T1-T4) and T4 sources are classified/rejected at phase_b/phase_e; phase_g_post_verify performs deterministic post-checks — evidence: crates/orchestrator/src/research_pipeline.rs:552
  - gap: no pre-LLM deterministic gating block before phase_d_extract_facts; phase_d is currently a deterministic stub (snippet-as-claim), not an LLM call with constrained decoding; constrained decoding (C-B) is not wired into phase_d_extract_facts

- Safety Sandwich: quote verification in Phase G (Design 4 Change 2)
  - exists: phase_g_post_verify at research_pipeline.rs:552 checks citation completeness and tier consistency
  - gap: no substring check of exact_quote against source page text content; no fabrication detection; no fuzzy match; phase_g only checks structural properties of the answer, not whether quotes appear verbatim in source content

## Mechanisms MISSING

- CompiledPipeline struct (Design 1) — searched: CompiledPipeline, compiled_pipeline, CompiledStep, TemplateVariable, TemplateId, PipelineOutcomeId; no evidence
- TemplateCrystallizer (Design 1) — searched: TemplateCrystallizer, template_crystallizer, crystallize, compilation_threshold; no evidence
- CompiledExecutor (Design 1) — searched: CompiledExecutor, compiled_executor, decompil; no evidence
- Pipeline template system with first_try_ratio tracking (Design 1 prerequisite) — searched: pipeline_template, PipelineTemplate, pipeline_factory, first_try_ratio, template_rank; no evidence
- Compilation eligibility check (eligible = L1 only + variable_region_ratio < 0.3 + no replan) — searched: variable_region_ratio, compilation_eligible, all_source_executions_used_only_L1; no evidence
- BackgroundTaskManager with soft-timeout auto-detach (Design 2) — searched: BackgroundTaskManager, background_task_manager, soft_timeout, BACKGROUND_TASK_STARTED, BACKGROUND_TASK_COMPLETE, adaptive polling; no evidence; existing bash tool only supports explicit user-flagged run_in_background with no monitoring
- SSB action types for BACKGROUND_TASK_STARTED / BACKGROUND_TASK_COMPLETE (Design 2) — searched: BACKGROUND_TASK_STARTED, BACKGROUND_TASK_COMPLETE in elai.map; no evidence
- CumulativeTokenTracker struct (Design 3) — searched: CumulativeTokenTracker, cumulative_token, last_response_prompt, prompt_tokens_total, recalibrate; no evidence
- Feedback loop from actual API token count to context_budget recalibration (Design 3 Change 2) — searched: recalibrate, actual > estimated, scaling_factor; no evidence
- Source content cache (LRU, 50KB/page, 100 pages) for quote verification (Design 4 Change 3) — searched: source_content_cache, page_content.*cache, lru.*page; no evidence
- fabrication_rate metric and cloud escalation trigger (Design 4 Change 4) — searched: fabrication_rate, FABRICATED, mark_fabricated, quote_verif; no evidence
- Whitespace-normalized + Levenshtein fuzzy match for quote verification (Design 4 Risk 4 mitigation) — searched: levenshtein, fuzzy.*match.*quote, normalize.*whitespace.*quote; no evidence
- Process priority lowering for background tasks (nice +10 / BELOW_NORMAL_PRIORITY) (Design 2 Risk 2 mitigation) — searched: nice.*background, BELOW_NORMAL_PRIORITY, process.*priority; no evidence

## Mechanisms UNVERIFIABLE

- "Break-even after ~17 executions" claim from arXiv 2604.05150 (Compiled AI) — this is a paper citation, not an implementation; cannot be verified against codebase
- "4.5ms P50 latency (vs 2,004ms), 57x token reduction" claims — same, paper-sourced performance numbers
- Variable region ratio < 0.3 threshold derivation — design document states this as a threshold with no empirical basis cited in codebase

## What must be done (action list)

- [ ] Design 1: Create CompiledPipeline, CompiledStep, TemplateVariable types in crates/orchestrator/src/ (C-1)
- [ ] Design 1: Create TemplateCrystallizer in crates/orchestrator/src/ with N=3 consecutive success threshold monitoring (C-1)
- [ ] Design 1: Implement pipeline template system with outcome tracking and first_try_ratio (prerequisite for C-1; referenced as 4D/4E in spec but absent)
- [ ] Design 1: Create CompiledExecutor with decompilation logic (2 consecutive failures OR confidence < 0.7) (C-2)
- [ ] Design 2: Create BackgroundTaskManager in crates/orchestrator/src/ with soft-timeout auto-detach from BashSandbox (C-3)
- [ ] Design 2: Add SSB action types BACKGROUND_TASK_STARTED and BACKGROUND_TASK_COMPLETE (C-3)
- [ ] Design 2: Implement adaptive polling (5s->10s->30s) and 5x hard-timeout kill in BackgroundTaskManager (C-3)
- [ ] Design 2: Wire autonomous-mode gate — backgrounding only when in autonomous mode (C-3)
- [ ] Design 3: Create CumulativeTokenTracker struct in crates/orchestrator/ tracking prompt_tokens_total, completion_tokens_total, cache_read/write_tokens per session (C-4)
- [ ] Design 3: Wire CumulativeTokenTracker into cloud provider response parsing and sidecar provider; forward last_response_prompt to context_budget.rs as actual context size (C-4)
- [ ] Design 4: Wire constrained decoding (Track C-B response_format) into phase_d_extract_facts; upgrade from stub to real LLM call with ExtractedFact JSON schema (C-5)
- [ ] Design 4: Add quote verification in phase_g_post_verify: substring check of exact_quote against source page text, with whitespace normalization and Levenshtein fuzzy fallback (C-5)
- [ ] Design 4: Add source content LRU cache (50KB/page, 100 pages) populated in phase_b_search, consumed in phase_g quote verification (C-5)
- [ ] Design 4: Add fabrication_rate metric per model and cloud escalation trigger when rate > 20% (C-5)

## Notes

The document is explicitly a PROPOSAL dated 2026-04-16, tagged as C-group post-Phase-4 work. No C-group implementation has started. The closest live code is: SpendAccumulator (partial Design 3 foundation), Track C-B constrained decoding in sidecar/grounding_client (partial Design 4 prerequisite), and user-initiated run_in_background in bash.rs (partial Design 2 structural placeholder — the field exists but the auto-detach-on-timeout semantics do not). Phase_d_extract_facts is currently a deterministic stub (snippet-as-claim), explicitly labeled "Stub: treat snippet as both claim and exact quote" at research_pipeline.rs:255 — it performs no LLM call, making Design 4's constrained-decoding wiring a prerequisite for LLM extraction to exist at all.
