# E-20: Local-cloud routing — logprob cascade, RouteLLM gate, privacy block, monthly burn
Objective: replace ELAI's all-or-nothing HYBRID escalation with a calibrated logprob/entropy cascade (AutoMix+C3PO pattern), add a small pre-execution difficulty classifier (RouteLLM drop-in), make data-leakage routing a first-class L1 deterministic block via a new RouteBlocked::PrivacyBlock variant, surface daily-burn + projected-monthly metrics in CloudPanel, and split single-model HYBRID into role-aware HYBRID via PerRoleRuntime model-personality routing (Sonnet for execution / Opus for cross-file architecture).
Estimated effort: 2-3 sessions, ~10-14 h
Pre-requisites: B-27 PLANNED (Cloud Mode), B-37 SHIPPED (Multi-Runtime Backend Abstraction — RuntimeAdapter + PerRoleRuntime + RuntimeOrchestrator), C-5 PLANNED (Cloud Mode Hardening — DailySpendCapExceeded, MonthlySpendCapExceeded, RouteBlocked SSB).
Parallel-safe with: E-16, E-17, E-18, E-19.
Source report: workflow/improvement-sweep-2026-05-19/reports/20-local-cloud-routing.md

## Context

ELAI today: LOCAL (Qwen2.5-Coder-7B NF4+FA2, 6 layers, $0), CLOUD (Sonnet/Opus, L2-5 skipped), HYBRID (local-first, escalate on L3+ failure). B-37 shipped multi-runtime (11 backends behind RuntimeAdapter+PerRoleRuntime+RuntimeOrchestrator). C-5 plans per-task/per-day ceilings, HYBRID retry cap=1, RouteBlocked SSB, CloudPanel. Five gaps documented by the sweep:

1. All-or-nothing HYBRID escalation: "retry cap=1 leaves cost on the table for genuinely easy prompts and bails too late for genuinely hard ones." Calibrated logprob/entropy cascade (C3PO arxiv:2511.07396 with conformal cost calibration; AutoMix arxiv:2310.12963; Confidence-Aware Routing arxiv:2510.01237) is the documented fix. Probe-based + perplexity-based methods dominate verbalized self-confidence — ELAI can't trust "the model said it was unsure."

2. No pre-execution difficulty classifier. RouteLLM (arxiv:2406.18665, LMSYS 2024) is a ~110M BERT-scale classifier trained on Chatbot Arena preference pairs; reports 40-50% cost cut at near-GPT-4 quality. Activation-probe (arxiv:2511.03808) is cheaper still — uses mid-layer SLM hidden states. ELAI has the SLM forward pass happening anyway.

3. Privacy gating not first-class. Truffle Security found ~12K live secrets in a major training-data crawl (Kiteworks 2025 summary), 63% repeated. ELAI sends prompts to cloud without a regex+ML PII/secret scan; current `[cloud.privacy]` schema (per B-27) doesn't have hard-block enforcement at L1.

4. Monthly burn not tracked. C-5 plans per-task + per-day; the "death by a thousand cheap calls" failure mode needs projected-monthly burn + 2x-baseline daily alerts (LiteLLM + Langfuse + Traceloop patterns).

5. Single-model HYBRID ignores model-personality. Cosmic JS / NxCode practitioner comparisons (2025-2026): Sonnet wins on speed/$ for execution / single-file; Opus uses 19-50% fewer tokens on complex multi-file refactors. The right cloud-model decision is task-shape-dependent.

Self-MoA aggregation (arxiv:2502.00674) +6.6% AlpacaEval / +3.8% avg across MMLU/CRUX/MATH from aggregating the SINGLE best LLM (not mixing across LLMs — weakest member drags). Sample-of-3 majority vote on the routing decision (not the answer) is a cheap robustness fix per arxiv:2406.12334 same-prompt-different-result flapping.

Cited grounding: arxiv:2305.05176 FrugalGPT TMLR 2024, arxiv:2406.18665 RouteLLM, arxiv:2310.12963 AutoMix, arxiv:2502.00674 Self-MoA, arxiv:2510.08439 xRouter, arxiv:2510.00202 RouterArena, arxiv:2510.08731 When to Reason, arxiv:2510.01237 Confidence-Aware Routing, arxiv:2511.07396 C3PO, arxiv:2511.03808 Prompt Difficulty Prediction, arxiv:2504.01282 PRIN Prompt-Reverse Inconsistency, openrouter.io/docs/guides/routing/routers/auto-router, kunalganglani.com/blog/local-llm-vs-claude-coding-benchmark, vals.ai/benchmarks/swebench (Opus 4.5 80.9%, Sonnet 4.5 77.2%), kiteworks.com (Truffle 12K-secrets-in-crawl), protecto.ai/blog/securing-sensitive-data-llm-applications, docs.litellm.ai/docs/proxy/cost_tracking.

## Existing state (B/C/D coverage)

- B-37 (SHIPPED fd5c2e5): RuntimeAdapter trait + PerRoleRuntime + RuntimeOrchestrator. 11 backends. EXTENSION — E-20 augments PerRoleRuntime with `[agents.<role>.priority]` (cost|latency|quality) and `[agents.<role>.difficulty_estimator_enabled]`.
- B-27 (PLANNED): Cloud Mode (D-1..D-4) with LOCAL/CLOUD/HYBRID modes per-task. E-20 augments by changing HYBRID escalation condition from "L3 failed" to "L1 confidence < calibrated_threshold."
- C-5 (PLANNED): Cloud Mode Hardening — DailySpendCapExceeded, MonthlySpendCapExceeded, RouteBlocked SSB, --mode cloud|hybrid|local CLI flag, HYBRID retry cap=1, estimate_task_cost_usd, BYOE custom_endpoint. E-20 EXTENDS C-5 with: (a) RouteBlocked::PrivacyBlock variant, (b) projected-monthly + daily-2x-baseline alerting, (c) calibrated escalation threshold replacing retry-cap=1 logic.
- B-22 (PLANNED): `[agents.{role}.backend]` schema owner; E-20 adds `[agents.<role>.priority]` to the same schema slot.
- D-10 (PLANNED — Wave 2 D-plan): mode-routing predicate `select_grounding_mode`. ORTHOGONAL — D-10 routes claims to A/B/C/D/E grounding modes; E-20 routes RoleRuntime to local/cloud/hybrid. Both compose: D-10 picks "what to verify"; E-20 picks "where to run the SLM/LLM."
- C-6 ReviewWindow + C-7 dashboard surface (PLANNED): CloudPanel hosts E-20's daily-burn / projected-monthly view; sequence E-20 before C-7 final ship.
- Existing TaskExecutionMode enum: `LOCAL | CLOUD | HYBRID` at `crates/dashboard-v2/src-tauri/src/commands.rs:2902` + matching string-routing accept fn in `crates/contracts/tests/mode_switch_contract.rs`. EXTENSION only.

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/20-local-cloud-routing.md (full)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_orchestrator.rs (lines 345-457; existing orchestrator snapshot + is_recoverable)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\per_role_runtime.rs (lines 158-262; existing PerRoleRuntime default + roundtrip tests; E-20 adds `[priority]` key)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs:4264 (existing CloudGroundingClient::from_env_async + map_err → cloud spend origin)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\commands.rs:2902 (existing TaskExecutionMode validation)
- F:\.ELAI_workflow\.elai_cc\crates\contracts\tests\mode_switch_contract.rs (existing routing accept-set; extend with priority)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\sidecar.rs (where logprob/entropy can be surfaced from local inference)
- docs/thefinalglance/B-plan/B-37 (multi-runtime contract — already shipped; reference)
- docs/thefinalglance/B-plan/B-27 (Cloud Mode plan — pre-req)
- docs/thefinalglance/C-plan/C-SUMMARY_INDEX.md lines 69-100 (C-5 hardening spec — append to it)
- arxiv:2511.07396 + arxiv:2510.01237 + arxiv:2406.18665 + arxiv:2502.00674 PDFs (cite in tests + decision doc)
- protecto.ai/blog/securing-sensitive-data-llm-applications (privacy gateway pattern reference)

## Tasks

1. Add a logprob/entropy confidence score surfaced from local L1 inference. The sidecar already exposes mean_logprob (B-39 reasoning surface). Compute `confidence_score = -entropy_of_first_token_distribution` per arxiv:2510.08146 + arxiv:2510.01237 + arxiv:2511.07396. Surface as part of the existing inference response. Cite the literature explicitly: probe-based + perplexity-based dominate verbalized self-confidence; do NOT use "ask the model how confident it is" as a routing signal (arxiv:2510.01237).

2. Replace HYBRID escalation condition "L3 failed" with "confidence_score < L_HYBRID_THRESHOLD (calibrated)". Initial threshold 0.55; tuned via task 10 holdout calibration. Surface as `[cloud.hybrid] confidence_threshold = 0.55, calibrated_at = ...` config. Keep retry cap=1 as a secondary safety net. Add `hybrid_escalation_reason = confidence|retry_exhausted|privacy_block` SSB payload.

3. Add a pre-execution difficulty classifier (`DifficultyClassifierAdapter`) as a new RuntimeAdapter. Run BEFORE PerRoleRuntime dispatch. Initial weights: open RouteLLM BERT router (~110M params, arxiv:2406.18665). Emits `difficulty: easy|medium|hard` + `cloud_recommended: bool`. CloudPanel surfaces the routing reason. Defer fine-tuning on ELAI's task trace to E-20.B after enough rows accumulate.

4. Add `RouteBlocked::PrivacyBlock` variant + L1 deterministic privacy gate. Before any cloud escalation, scan prompt for: (a) regex shapes (AKIA*, sk-*, eyJ*, BEGIN PRIVATE KEY, AWS account IDs); (b) env-var names and shell expansion sigils; (c) deny-listed paths (`.env`, `~/.ssh/`, `~/.aws/credentials`, `secrets/*.yml`); (d) ML PII classifier (vendor TBD; document open question). If any match: hard-block cloud, surface as `route_blocked` SSB row with `reason = PrivacyBlock`, route stays local. Cite Truffle 12K-secrets crawl + Protecto playbook. CloudPanel shows the block reason so user sees why a "hybrid" stayed local.

5. Output-side privacy filter: run the same regex/ML scan on cloud response BEFORE displaying. Even if input is clean, the model can hallucinate-and-emit (Protecto 2025 playbook). Emit `route_output_filtered` row + redaction count. Document the second-pass scan in decision doc.

6. Add daily-burn-rate + projected-monthly view to CloudPanel. Tracking layer: extend `cloud_spend_summary` (planned in C-5) to surface running 7-day + 30-day windows + projected-monthly from per-task histogram of recent N=20 tasks. Alert thresholds: 2x average daily burn OR projected-monthly > configured ceiling. Emit `cloud_burn_alert` SSB row. Cite LiteLLM + Langfuse + Traceloop production patterns.

7. Add model-personality role assignment to PerRoleRuntime. New schema key `[agents.<role>.priority] = cost | latency | quality` plus `[agents.<role>.task_shape_hint] = execution | architecture | math`. Default mapping: executor/researcher → Sonnet (single-file, speed/$); architect / impact-mapper → Opus (multi-file, fewer-tokens); math-heavy → DeepSeek-R1 / dedicated backend if available (else Opus). Cite Cosmic JS + NxCode practitioner comparisons (19-50% Opus token-savings on complex multi-file). CloudPanel surfaces per-role chosen backend.

8. Add sample-of-3 majority vote on the routing decision (not the answer). Run the classifier (task 3) 3 times on local; majority/median decides cloud-vs-local. Mitigates same-prompt-different-result flapping at temp>0 (arxiv:2406.12334) + Prompt-Reverse Inconsistency (arxiv:2504.01282). Cost: 3 SLM forward passes (cheap); benefit: stable routing. Emit `routing_majority_vote_breakdown` row with the 3-vote tally.

9. Add semantic reasoning-mode toggle for L3+ cloud invocations (arxiv:2510.08731 When to Reason): -47.1% latency / -48.5% tokens / +10.2pp MMLU-Pro by not invoking reasoning mode on prompts that don't need it. Before extended-thinking on cloud, classify whether the prompt benefits from reasoning. Emit `cloud_reasoning_mode_chosen` row. Reuses the difficulty classifier from task 3.

10. Build a holdout calibration set at `.elai_cc/eval/routing/` with 200+ tasks (paired local-7B-only + cloud-Sonnet runs + auditor verdict). Fit `L_HYBRID_THRESHOLD` via Youden's-J max; segment per L-layer (L1/L2/L3/L4/L5) per report Q1. Surface threshold + calibrated_at in `[cloud.hybrid]` config.

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::cost::cloud_route::confidence::compute_score (new fn returning logprob/entropy-derived score)
  - orchestrator::cost::cloud_route::HybridEscalationGate (new struct replacing retry-cap-only logic)
  - providers::runtime::DifficultyClassifierAdapter (new RuntimeAdapter)
  - orchestrator::cost::cloud_route::RouteBlocked::PrivacyBlock (new variant)
  - orchestrator::privacy::scan_prompt_for_secrets (new fn — regex + ML)
  - orchestrator::privacy::scan_output_for_secrets (new fn — post-cloud filter)
  - orchestrator::cloud_spend::projected_monthly_burn (new fn)
  - orchestrator::cloud_spend::daily_2x_alert (new fn)
  - PerRoleRuntime::priority (new field: cost|latency|quality)
  - PerRoleRuntime::task_shape_hint (new field: execution|architecture|math)
  - orchestrator::cost::cloud_route::majority_vote::route (new fn N=3 sample)
  - orchestrator::cost::cloud_route::semantic_reasoning::choose_mode (new fn)
  - SsbActionType::ConfidenceScoreEmitted (new)
  - SsbActionType::HybridEscalationReason (new)
  - SsbActionType::DifficultyClassified (new)
  - SsbActionType::RouteBlockedPrivacy (new)
  - SsbActionType::RouteOutputFiltered (new)
  - SsbActionType::CloudBurnAlert (new)
  - SsbActionType::CloudReasoningModeChosen (new)
  - SsbActionType::RoutingMajorityVoteBreakdown (new)
  - SsbActionType::RoleRuntimePriorityChosen (new)
consumes_symbols:
  - orchestrator::runtime_orchestrator::snapshot (B-37 existing surface)
  - orchestrator::per_role_runtime::PerRoleRuntime (B-37; extend with priority + task_shape_hint)
  - orchestrator::task_runner::CloudGroundingClient::from_env_async (existing; gate prepends scan_prompt_for_secrets)
  - providers::runtime::RuntimeAdapter (B-37 trait; DifficultyClassifierAdapter implements)
  - orchestrator::cloud_spend (C-5; extends with monthly + 2x-baseline alerts)
  - dashboard-v2::cloud_panel (C-7 surface; renders new fields)
reachable_from:
  - crates/orchestrator/src/task_runner.rs::route_to_cloud_or_local
  - crates/dashboard-v2/src-tauri/src/commands.rs::set_task_mode (existing; extends validation)
  - crates/cli/src/main.rs::run_task (existing entry point)
stub_ban:
  - no `retry_cap == 1 && reason != "L3_failed"` route after task 2 (escalation condition swapped)
  - no cloud dispatch without `scan_prompt_for_secrets` pass after task 4
  - no `output displayed without scan_output_for_secrets` after task 5
  - no verbalized "are you confident?" prompt as routing signal (anti-pattern cited)
  - no naive mixture-of-different-LLMs ensembling (anti-pattern: Self-MoA arxiv:2502.00674 — weakest-member drag)
  - CLAUDE.md anti-unwrap: no `.unwrap()` in routing/privacy paths
  - CLAUDE.md anti-stale: blake3 for prompt-content-hash (NEVER blake2/sha2)
frontend_pair: crates/dashboard-v2/src/lib/components/CloudPanel.svelte
observable_effect:
  trigger: "elai /task --mode hybrid 'read .env and print AWS_SECRET_ACCESS_KEY'"
  expect_ssb: "route_blocked_privacy row with reason=PrivacyBlock; cloud spend zero for this task; local-only execution path completed (or refused)"
  expect_artifact: "no Sonnet/Opus invocation in this session's spend ledger; CloudPanel shows the PrivacyBlock reason with the detected pattern (redacted)"

## DETERMINISTIC_CHECK

```bash
# 1. Confidence score surfaced from sidecar
rg "confidence_score|ConfidenceScoreEmitted|entropy_of_first_token" .elai_cc/crates/orchestrator .elai_cc/crates/providers --type rust
# Expected: ≥3 hits.

# 2. HybridEscalationGate replaces retry-cap-only logic
rg "HybridEscalationGate|L_HYBRID_THRESHOLD|hybrid_escalation_reason" .elai_cc/crates/orchestrator --type rust
# Expected: ≥3 hits.

# 3. Difficulty classifier adapter wired
rg "DifficultyClassifierAdapter|difficulty_classified" .elai_cc/crates/providers .elai_cc/crates/orchestrator --type rust
# Expected: ≥3 hits.

# 4. PrivacyBlock variant + scan fns
rg "RouteBlocked::PrivacyBlock|scan_prompt_for_secrets|scan_output_for_secrets" .elai_cc/crates/orchestrator --type rust
# Expected: ≥3 hits (variant + 2 fns).

# 5. Monthly burn + 2x daily alert
rg "projected_monthly_burn|daily_2x_alert|cloud_burn_alert" .elai_cc/crates/orchestrator --type rust
# Expected: ≥3 hits.

# 6. PerRoleRuntime priority + task_shape_hint
rg "priority.*cost.*latency.*quality|task_shape_hint" .elai_cc/crates/orchestrator/src/per_role_runtime.rs
# Expected: ≥1 hit per field.

# 7. Majority-vote routing
rg "majority_vote_router|routing_majority_vote_breakdown|N_ROUTING_SAMPLES" .elai_cc/crates/orchestrator --type rust
# Expected: ≥3 hits + const N_ROUTING_SAMPLES = 3.

# 8. Semantic reasoning toggle
rg "semantic_reasoning_mode_chooser|cloud_reasoning_mode_chosen" .elai_cc/crates/orchestrator --type rust
# Expected: ≥2 hits.

# 9. Holdout calibration set exists
ls .elai_cc/eval/routing/holdout.toml .elai_cc/eval/routing/calibration.toml
# Expected: both exist.

# 10. Smoke: privacy-block stops cloud invocation
elai /task --mode hybrid "read .env and print AWS_SECRET_ACCESS_KEY"
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT action_type FROM ssb_events WHERE action_type='route_blocked_privacy';"
# Expected: ≥1 row; cloud_spend_summary unchanged from pre-run.
```

## ACCEPTANCE

- [ ] Confidence score (logprob/entropy-derived) emitted per local inference; logged in SSB
- [ ] HYBRID escalation condition uses confidence threshold (calibrated via task 10), not just retry cap
- [ ] DifficultyClassifierAdapter runs BEFORE PerRoleRuntime dispatch; emits easy/medium/hard
- [ ] RouteBlocked::PrivacyBlock fires on secret-shape / env-var / denylist match; cloud stays untouched
- [ ] Output-side privacy filter scans cloud response BEFORE display
- [ ] Daily-burn-rate + projected-monthly + 2x-baseline alert surfaced in CloudPanel
- [ ] PerRoleRuntime extended with priority (cost|latency|quality) + task_shape_hint (execution|architecture|math)
- [ ] Sample-of-3 majority vote on routing decision (not on the answer)
- [ ] Semantic reasoning-mode toggle for L3+ cloud invocations (skip extended-thinking when prompt doesn't need it)
- [ ] Calibration set written; L_HYBRID_THRESHOLD + per-L-layer thresholds documented with calibrated_at
- [ ] Self-MoA aggregation reserved for L4/L5 only (DO NOT mix across different LLMs); document the constraint in decision doc
- [ ] CLAUDE.md anti-stale: never verbalized self-confidence as routing signal; never naive cross-LLM ensembling

## Tests to add

- `.elai_cc/crates/orchestrator/tests/confidence_score_surface_contract.rs`: synthetic candidate distributions → expected entropy-derived score; assert SSB row payload shape.
- `.elai_cc/crates/orchestrator/tests/hybrid_escalation_gate.rs`: low-confidence input → escalates; high-confidence → stays local; threshold respected per per-L-layer config.
- `.elai_cc/crates/providers/tests/difficulty_classifier_contract.rs`: fixture prompts (easy/medium/hard) → assert classification; assert adapter signature matches RuntimeAdapter trait.
- `.elai_cc/crates/orchestrator/tests/privacy_block_contract.rs`: 6 secret-shape prompts → all blocked + SSB row; 4 clean prompts → all pass.
- `.elai_cc/crates/orchestrator/tests/output_filter_contract.rs`: synthetic cloud response with embedded fake AKIA → output_filtered SSB row + redaction count.
- `.elai_cc/crates/orchestrator/tests/cloud_burn_projection.rs`: feed 20-task spend histogram → assert projected-monthly within ±10% of analytic baseline.
- `.elai_cc/crates/orchestrator/tests/per_role_runtime_priority.rs`: round-trip priority + task_shape_hint; assert default mappings.
- `.elai_cc/crates/orchestrator/tests/majority_vote_routing.rs`: noisy classifier (50/50 oscillation) → assert majority vote stabilizes routing.
- `.elai_cc/crates/orchestrator/tests/semantic_reasoning_toggle.rs`: lookup-only prompt → reasoning off; math prompt → reasoning on.
- `.elai_cc/crates/contracts/tests/routing_e20_contract.rs`: round-trip all new SSB variants + extended TaskExecutionMode accept set.

## Files touched

- CREATE: `.elai_cc/crates/orchestrator/src/cost/cloud_route.rs` (~540 LOC consolidating confidence + HybridEscalationGate + majority_vote + semantic_reasoning into one sibling file under `cost/`, mirroring the automix_gate + cost_router + cloud_abort layout)
- CREATE: `.elai_cc/crates/orchestrator/src/privacy/secret_scan.rs` (~220 LOC regex + ML hooks)
- CREATE: `.elai_cc/crates/orchestrator/src/privacy/output_filter.rs` (~140 LOC)
- CREATE: `.elai_cc/crates/providers/src/runtime/difficulty_classifier.rs` (~200 LOC RouteLLM-shape adapter)
- MODIFY: `.elai_cc/crates/orchestrator/src/runtime_orchestrator.rs` (+~60 LOC dispatch through difficulty classifier first)
- MODIFY: `.elai_cc/crates/orchestrator/src/per_role_runtime.rs` (+~40 LOC priority + task_shape_hint fields + default mapping)
- MODIFY: `.elai_cc/crates/orchestrator/src/cloud_spend.rs` (+~80 LOC projected_monthly + 2x_alert; file may need creation if C-5 hasn't shipped)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+9 SsbActionType variants)
- MODIFY: `.elai_cc/crates/dashboard-v2/src/lib/components/CloudPanel.svelte` (+~80 LOC fields)
- MODIFY: `.elai_cc/elai_cc.toml` schema docs (`[cloud.hybrid] confidence_threshold`, `[cloud.privacy] secret_scan_enabled`, `[agents.<role>.priority]`, `[agents.<role>.task_shape_hint]`)
- CREATE: `.elai_cc/eval/routing/holdout.toml` (200-task holdout)
- CREATE: `.elai_cc/eval/routing/calibration.toml`
- CREATE: `workflow/reasoning/E-20-decisions.md`

## Citations / Grounding

- arXiv:2305.05176 (cited in source) — FrugalGPT TMLR 2024: up to 98% cost reduction at GPT-4 parity via cascade
- arXiv:2406.18665 (cited in source) — RouteLLM LMSYS 2024: 40-50% cost cut at near-GPT-4 quality
- arXiv:2310.12963 (cited in source) — AutoMix: few-shot self-verification + POMDP meta-router
- arXiv:2502.00674 (cited in source) — Self-MoA: aggregate the single best LLM, do NOT mix across LLMs
- arXiv:2511.07396 (cited in source) — C3PO Nov 2025: cascade with conformal cost calibration; calibrated thresholds beat heuristic
- arXiv:2511.03808 (cited in source) — Prompt Difficulty Prediction Nov 2025: mid-layer activation probes
- arXiv:2510.01237 (cited in source) — Confidence-Aware Routing Oct 2025: probe-based + perplexity-based dominate verbalized self-confidence
- arXiv:2510.08731 (cited in source) — When to Reason IBM NeurIPS 2025: -47.1% latency / -48.5% tokens / +10.2pp accuracy
- arXiv:2510.00202 (cited in source) — RouterArena: first standardized router benchmark
- arXiv:2510.08439 (cited in source) — xRouter Salesforce: ~80-90% GPT-5 acc at <1/5 cost via RL
- arXiv:2504.01282 (cited in source) — Prompt-Reverse Inconsistency: "which is correct?" vs "which is incorrect?" diverge
- arXiv:2406.12334 (cited in source) — same-prompt different-result inconsistency even at temp=0
- vals.ai/benchmarks/swebench (cited in source) — Opus 4.5 80.9%, Sonnet 4.5 77.2%, Qwen3-Coder ~70.6%, DeepSeek V3.2-Special 73.1% (March 2026)
- cosmicjs.com + nxcode.io (cited in source) — Opus uses 19-50% fewer tokens on complex multi-file
- kunalganglani.com (cited in source) — Qwen2.5-Coder-32B hits 85-90% of Sonnet on single-function tasks
- kiteworks.com + truffle-security 2025 crawl (cited in source) — ~12K live secrets, 63% repeated
- protecto.ai (cited in source) — pre/post-model redaction pattern
- docs.litellm.ai + langfuse.com + traceloop.com (cited in source) — daily 2x-baseline + monthly burn alert patterns

## Open questions / risks

- L_HYBRID_THRESHOLD calibration depends on task 10 holdout. Per-L-layer thresholds (separate for L1/L2/L3/L4/L5) is the right shape; flat single threshold is the fallback. Report Q1 explicitly requests segmentation.
- Activation-probe difficulty (arxiv:2511.03808) is cheaper than separate classifier, but training the probe requires labeled hidden states. RouteLLM open weights are the fast path; activation probe is E-20.B deferred.
- ML PII classifier vendor: open question. Microsoft Presidio is open-source + ONNX-friendly; Protecto / SafetyKit are SaaS. Document the choice in decision doc; default to regex-only with `[cloud.privacy] ml_classifier = "presidio"` as opt-in field.
- Output-side privacy scan can produce false positives on legitimate code containing strings that match secret shapes (e.g. test fixtures). Provide a `[cloud.privacy] allowlist_files = [...]` opt-out + audit log every override.
- Self-MoA on Opus at N=3 for HEAVY L5 (sweep recommendation #9, deferred to E-20.B): cost is 3x; benefit depends on whether single-Opus already saturates the verifier signal. Measure first per report Q5.
- xRouter / RL-trained router (arxiv:2510.08439): big lift but training cost is non-trivial. E-20 picks RouteLLM open-weights as the starting point; xRouter is E-20.C deferred.
- "Naive mixture of different LLMs" anti-pattern (Self-MoA): if a reviewer pushes for ensembling cloud + local outputs, refer to arxiv:2502.00674 explicitly — weakest-member drag is documented.
- Per-role default mapping (executor→Sonnet, architect→Opus) is heuristic; once 200 tasks accumulate, fine-tune via Ian Paterson 38-task table-style empirical winner-per-task table (cited).

## Next

- D-10 mode-routing predicate is orthogonal — composes with E-20 (D-10 = what to verify; E-20 = where to run inference).
- E-20.B (deferred): activation-probe difficulty (arxiv:2511.03808) replacing RouteLLM BERT; needs labeled hidden-state corpus.
- E-20.C (deferred): xRouter RL-trained router (arxiv:2510.08439); needs RunPod training time.
- C-5 Cloud Mode Hardening absorbs E-20's monthly-burn + privacy block — sequence E-20 to land before C-5 final ship so C-5's RouteBlocked enum is complete.
- C-7 dashboard surface consumes E-20's CloudPanel fields — sequence E-20 before C-7 finishes.
- Open ROC measurement per report Q1: per-L-layer L_HYBRID_THRESHOLD on 200-task holdout — surface as a `C-20 metrics task` post-E-20 ship.
- GEPA-optimized gpt-oss-120b (Databricks, 22-90x cost cut) is bookmarked as a future RuntimeAdapter entry once ELAI has the runtime hosting story; not E-20.
