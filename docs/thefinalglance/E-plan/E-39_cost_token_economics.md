# E-39: Cost + token economics — HARD ceiling enforcement, AutoMix verify-before-escalate, velocity caps, pre-execution difficulty estimator
Objective: Convert ELAI's existing `max_spend_per_task` / `max_spend_per_day` / `warn_at_percent` from soft alerts into HARD ceiling enforcement that ABORTS mid-task on breach (the LangChain $47K wallet-loop class of failure). Add AutoMix-style self-verify before cloud escalation (>50% cost reduction at iso-quality per ICLR 2025). Add velocity caps (req/min, $/min, $/hour) for retry-storm defense. Add a pre-execution difficulty estimator that gates the cloud route before the call is even attempted. Wire 5-min prompt-cache discipline + DeepSeek V4-Flash backend + Cerebras/Groq free-tier paths.
Estimated effort: 2-3 sessions, ~8-12 h
Pre-requisites: B-27 PLANNED (Cloud Mode — D-1..D-4: `--mode cloud|hybrid|local`, CLOUD_SPEND SSB, ceiling enforcement, estimate_task_cost_usd; per B-PLAN-SUMMARY line 523), C-5 PLANNED (Cloud Mode Hardening — adds hard ceiling enforcement + warn_at_percent + HYBRID escalation retry cap=1 + LOC sizing constants; per C-SUMMARY_INDEX line 69), B-37 SHIPPED (RuntimeAdapter — new backends slot in here per memory:project_b37_shipped commit fd5c2e5)
Parallel-safe with: E-38 (different concern but cross-references DeepSeek V4-Flash legal-posture), E-36 (different concern; vision parses count against ceilings here), E-37 (different concern; PromptArmor classifier cost counts here), E-40 (different concern)
Source report: workflow/improvement-sweep-2026-05-19/reports/39-cost-token-economics.md

## Context

The report's headline incident is unmistakable: a $47K LangChain 4-agent loop in 11 days (ravoid.com/blog/ai-agent-budget-enforcement) and a 2.3M-call retry storm in a weekend (leanopstech.com/blog/agentic-ai-cost-runaway-token-budget-2026). Observability does not stop spend — only a HARD cutoff that ABORTS mid-task does. ELAI today has the soft scaffolding via B-27 PLANNED (CLOUD_SPEND SSB action type, `max_spend_per_task`, `max_spend_per_day`, `warn_at_percent`) and C-5 PLANNED hardens it with `DailySpendCapExceeded` / `MonthlySpendCapExceeded` returns plus HYBRID escalation retry cap=1. E-39's primary delta: the C-5 ceiling enforcement is the FLOOR, not the ceiling — E-39 adds (a) per-agent-role caps, (b) velocity caps (req/min, $/min, $/hour, $/day), (c) AutoMix-style self-verify before cloud escalation, (d) a pre-execution difficulty estimator that runs locally before any cloud call is attempted.

The pricing facts that shape the design (verified May 2026):
- Anthropic Sonnet 4.6: $3/$15 per MTok input/output; cache write 1.25x (5-min TTL), 2x (1-hour TTL); cache read 0.10x (90% off). Batch API 50% off across all Anthropic models. Stack cache + batch → ~95% combined savings on non-interactive workloads (Anthropic pricing + finout.io/blog).
- Anthropic 5-min TTL change (CRITICAL May 2026): TTL cut from 60min to 5min in early 2026 raised many production bills 30-60% (dev.to/whoffagents). ELAI HYBRID retry loop MUST re-warm within 5 min or pay the cache-write multiplier twice.
- Agentic coding burns 1-3.5M tokens per task with input outnumbering output 20-25x and 30x variance same-task (arxiv:2604.22750 — VERIFY arXiv ID; that's the report-cited ID but year-tag is suspicious; WebSearch before citing in the decision doc).
- DeepSeek V4-Flash: $0.30/$0.50 per MTok, 90% cache discount = the cheapest cloud floor for non-critical L3 escalations.
- Cerebras free tier: 1M tokens/day free since April 2026 (getaiperks.com); Groq free tier: 14.4K req/day default + Llama 4 Scout / Qwen3 32B / Kimi K2 (tokenmix.ai).
- Local Qwen 7B NF4 on user's GPU: $0 marginal cost (per memory:project_qwen3b_baseline 1.92 GB VRAM); per-task power = fractions of a cent at $0.30/kWh.
- FrugalGPT (arxiv:2305.05176) cascade: 98% cost reduction or +4% accuracy at iso-cost. AutoMix (ICLR 2025, arxiv:2310.12963) adds self-verify gate: >50% cost reduction at iso-quality.

The structural moves:
1. HARD ceiling that ABORTS — the C-5 enforcement returns an error but does it ABORT the in-flight task or just refuse the next call? E-39 makes it BOTH: refuse the next call AND abort the task with `TaskAbortedBySpendCap` SSB row + partial-result return.
2. Velocity caps — $/min + req/min + $/hour + $/day. RPS-based limits don't protect budgets per handsonarchitects.com (a single 200K-token prompt outspends 10K 1K-token prompts).
3. AutoMix self-verify — before HYBRID escalation triggers cloud, run a cheap local verifier ("is this output structurally valid"). Gate the cloud call on verifier verdict. ICLR 2025 measures >50% cost reduction at iso-quality.
4. Pre-execution difficulty estimator — before any cloud route is taken, estimate task difficulty locally (trigram features + centrality + reference-task cosine). C-46 (Selective Delegation, PLANNED per C-SUMMARY_INDEX line 681) has a `TaskDifficultyClassifier` — E-39 EXTENDS it with the cost dimension: difficulty + budget-remaining → route decision. Easy task + ample budget = local; hard task + tight budget = abort or downgrade-to-Haiku; easy task + tight budget = local.
5. Prompt-cache discipline — every cloud call carries `cache_control` blocks; variable content suffix-only; 5-min TTL (1.25x write, 0.10x read). vLLM prefix cache for self-hosted is already aligned (docs.vllm.ai/en/stable/design/prefix_caching).

## Existing state (B/C/D coverage)

- B-27 PLANNED (Cloud Mode — D-1..D-4): per B-PLAN-SUMMARY line 523. EXTENSION: E-39 layers HARD enforcement + velocity caps + AutoMix verify + difficulty estimator on TOP of B-27's `--mode cloud|hybrid|local` + CLOUD_SPEND SSB. E-39 does NOT replace B-27 — it tightens it. Schema fields `max_spend_per_task` / `max_spend_per_day` / `warn_at_percent` already defined in B-27.
- C-5 PLANNED (Cloud Mode Hardening): per C-SUMMARY_INDEX line 69. EXTENSION: C-5 ships `DailySpendCapExceeded` + `MonthlySpendCapExceeded` returns + warn_at_percent default 80% + HYBRID retry cap=1 + LOC sizing constants (25/40/300/500) + `estimate_task_cost_usd` + BYOE custom_endpoint. E-39 adds (a) `TaskAbortedBySpendCap` mid-task abort, (b) `per_role_caps` (executor vs auditor different ceilings), (c) velocity caps (req/min, $/min, $/hour), (d) AutoMix verifier between L3-failure and cloud escalation, (e) pre-execution difficulty estimator.
- C-46 PLANNED (Selective Delegation — TaskDifficultyClassifier per C-SUMMARY_INDEX line 681): EXTENSION: E-39 Task 4 reuses C-46's TaskDifficultyClassifier (trigram + centrality + ref-task cosine) and adds a budget-aware decision layer. If C-46 ships first, E-39 plugs into its classifier. If E-39 ships first, the classifier moves into a shared module under `crates/orchestrator/src/difficulty.rs`.
- B-37 SHIPPED (PerRoleRuntime + 11 backends; commit fd5c2e5): EXTENSION: register 3 new cloud backends — `Backend::DeepSeekV4Flash` (cheapest cloud floor), `Backend::CerebrasFree` (1M tokens/day free), `Backend::GroqFree` (14.4K req/day default). Reuses the RuntimeAdapter trait. Anti-slopsquatting: VERIFY API base URLs + model identifiers + last-update dates before wiring.
- Existing CLOUD_SPEND SSB action type (per B-PLAN-SUMMARY line 980): EXTENSION: 6 new SSB variants (see WIRING_CONTRACT).
- C-4 PLANNED (Cloud Spend Summary — per C-SUMMARY_INDEX line 63): cloud_spend_summary viewer method. RELATED: E-39 surfaces the new velocity-cap stats + AutoMix-saved-spend in the same panel.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\39-cost-token-economics.md` (full — 195 lines)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\grounding_client.rs` (existing CloudGroundingClient per elai.map line 77007; `from_env_async` at task_runner.rs:4264; auto OAuth refresh per patch-R2)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\task_runner.rs` lines 4264-4280 (cloud client construction site)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs` (existing SsbActionType enum; add 6 new variants)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_orchestrator.rs` (B-37 SHIPPED PerRoleRuntime; new backends register here)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 523-560 (B-27 Cloud Mode design)
- `F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-SUMMARY_INDEX.md` lines 69-100 (C-5 Cloud Hardening), line 681 (C-46 Selective Delegation)
- platform.claude.com/docs/en/about-claude/pricing — WebFetch (Anthropic pricing May 2026)
- dev.to/whoffagents/claude-prompt-caching-in-2026-the-5-minute-ttl-change-thats-costing-you-money-4363 — WebFetch (5-min TTL change)
- arxiv:2305.05176 FrugalGPT — WebFetch via paper-ingestor mode=paper
- arxiv:2310.12963 AutoMix — WebFetch via paper-ingestor (NOTE: verify; report cites a different ID for AutoMix's pseudonymous link)
- arxiv:2511.07396 C3PO — WebFetch
- ravoid.com/blog/ai-agent-budget-enforcement — WebFetch ($47K LangChain incident)
- handsonarchitects.com/blog/2025/denial-of-wallet-cost-aware-rate-limiting-part-1 — WebFetch (cost-aware rate limiting)
- api-docs.deepseek.com/quick_start/pricing — WebFetch (DeepSeek V4-Flash pricing)
- getaiperks.com/en/ai/cerebras-free-tier-guide — WebFetch (Cerebras 1M tok/day free)
- tokenmix.ai/blog/groq-free-tier-limits-2026 — WebFetch (Groq free tier)

## Tasks

1. HARD ceiling mid-task abort. Modify the existing `crates/orchestrator/src/cloud_spend.rs` (or wherever C-5's ceiling lives at time of execution) so when `task_spend_usd_cents > max_spend_per_task_cents`, the in-flight task is ABORTED — not just the next call refused. Cleanup path: drop pending SSB writes, return partial result with `aborted_by_spend_cap: true`, emit `TaskAbortedBySpendCap` SSB row with `spent_usd_cents`, `cap_usd_cents`, `last_action_type`. The abort is COOPERATIVE — wait for the current in-flight LLM call to complete and bank that token cost before aborting (avoids losing already-paid-for output). Reference: ravoid.com $47K incident — observability does NOT stop spend; only a HARD cutoff does.

2. Per-agent-role ceilings. Extend `[cloud] max_spend_per_task = "$X"` to `[cloud.per_role_caps] executor = "$X", auditor = "$Y", scope_planner = "$Z", memory_writer = "$W"`. Auditor gets larger budget (reads everything per memory:project_b40_shipped + memory:project_b41_shipped); executor gets smaller (focused edits per file_plan). Per-role spend tracked in `task_spend_by_role` HashMap in the task ctx. Reference: arxiv:2604.22750 (or verified equivalent) — agentic input/output 20-25x ratio, 30x variance same-task.

3. Velocity caps. New module `crates/orchestrator/src/velocity_cap.rs` (~180 LOC). Rolling-window counters: `req/min`, `$/min`, `$/hour`, `$/day` (1-min, 10-min, 1-hour, 24-hour windows). Configurable in `[cloud.velocity_caps]`. Breach mid-task → abort + `VelocityCapBreached` SSB row with `window`, `count`, `limit`. Hourly velocity cap is the retry-storm defense (2.3M-call weekend incident). Reference: leanopstech.com/blog/agentic-ai-cost-runaway-token-budget-2026 + handsonarchitects.com — RPS-based limits insufficient for LLM endpoints because a single 200K-token prompt outspends 10K 1K-token prompts.

4. AutoMix-style self-verify before HYBRID escalation. New module `crates/orchestrator/src/automix_gate.rs` (~220 LOC). Before the existing C-5 HYBRID retry triggers a cloud call, invoke a local Qwen 3B sidecar verifier with the failed-output + the original task: "Is this output structurally valid? Output JSON `{ verdict: Continue | Retry | Escalate, reason: ... }`." On `Continue`, accept the failed output without escalation. On `Retry`, do a same-tier retry (one more local attempt). On `Escalate`, proceed to cloud. The verifier is the gate — measured >50% cost reduction at iso-quality per AutoMix ICLR 2025. SSB row `AutomixVerifierVerdict` per decision. Reference: arxiv:2310.12963 (verify ID).

5. Pre-execution difficulty estimator with budget-aware routing. New module `crates/orchestrator/src/cost_router.rs` (~250 LOC). Before any cloud route is attempted, run a local difficulty estimator (extending C-46's TaskDifficultyClassifier if shipped; otherwise standalone). Features: trigram of task description + centrality of touched symbols in the symbol graph + cosine to reference-task corpus. Output: `difficulty: f32 ∈ [0, 1]`. Combine with `budget_remaining_usd`: decision matrix `(easy + ample) → local`, `(easy + tight) → local`, `(hard + ample) → cloud Sonnet`, `(hard + tight) → cloud Haiku 4.5 OR DeepSeek V4-Flash`, `(very-hard + critical) → cloud Opus 4.7 with explicit user confirmation`. Emit `CostRouterDecision` SSB row with `difficulty`, `budget_remaining`, `chosen_backend`, `rationale`.

6. Prompt-cache discipline pass. Audit every existing `CloudGroundingClient` call site (per elai.map line 4264) and ensure `cache_control` blocks are present on (a) system prompt, (b) project spec, (c) repo manifest, (d) tool schema. Variable content goes SUFFIX-ONLY. Default to 5-min TTL (1.25x write, 0.10x read — breaks even after 1 read). Add a `[cloud.cache] ttl = "5min"|"1hour"` config flag; default 5min. Reference: platform.claude.com/docs/en/about-claude/pricing + dev.to/whoffagents (the 5-min TTL change cost many production bills 30-60%). Add a per-session cache-hit-rate gauge surfaced in MonitorWindow; target >70% per report Open Question 1.

7. Register 3 new cloud backends in B-37 RuntimeAdapter: (a) `Backend::DeepSeekV4Flash` ($0.30/$0.50 + 90% cache discount; the cheap cloud floor) — VERIFY DeepSeek API base URL + current model identifier on api-docs.deepseek.com before wiring. (b) `Backend::CerebrasFree` (1M tokens/day free since April 2026) — VERIFY current free-tier model list on Cerebras docs. (c) `Backend::GroqFree` (14.4K req/day default; Llama 4 Scout / Qwen3 32B / Kimi K2) — VERIFY current Groq free-tier model list. Anti-slopsquatting (CLAUDE.md HARD RULE): VERIFY ALL THREE backends' API URLs + model identifiers + last-published dates BEFORE adding to runtime_orchestrator.rs. The CostRouter (Task 5) routes to DeepSeek V4-Flash for `(hard + tight)` decisions; to Cerebras/Groq free-tier for `(easy + tight)` scaffold work where free quota suffices.

8. Cerebras / Groq free-tier guard. Each free-tier backend has a hard daily quota — exceeding it costs $ if billing is configured, or fails outright if not. New module `crates/orchestrator/src/free_tier_guard.rs` (~100 LOC). Track per-backend daily usage; refuse calls when ≥95% of daily quota consumed; emit `FreeTierQuotaApproaching` SSB row at 80%, `FreeTierQuotaExceeded` SSB row + reject on actual exceed. Reset counter at provider's local-time day boundary (Cerebras + Groq both use UTC).

9. Pre-execution cost estimate banner. Before any cloud call dispatches, log a structured row showing the estimated cost (existing C-5 `estimate_task_cost_usd`) ALONGSIDE the post-call actual cost. Surface in the dashboard's existing CloudPanel (per memory:project_b37_shipped RuntimePanel). Big delta between estimate and actual = a `CostEstimateInaccurate` SSB row (threshold: actual / estimate > 2.0 or < 0.5). Over time the estimate model can be re-calibrated. Reference: arxiv:2604.22750 — 30x variance same-task means budget must be range-based (P50+P95), not point. The estimate field becomes `{ p50_usd, p95_usd }` not a point.

10. Layered cap recommendation from rocketedge.com 2026 (per report wallet-attack table): $50/day soft alert > $100/day hard cutoff > $1000/month manual-override-required. Implement as the DEFAULT `[cloud.layered_caps]` schema (user overrides per-project). The $1000/month manual override requires `elai cloud override-cap` CLI subcommand with a typed confirmation string (preventing accidental override). SSB row `LayeredCapManualOverride` with `user_session_id` + `override_amount` + `confirmation_string` for audit-trail (composes with E-38's EU AI Act audit-trail liveness gate).

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::cloud_spend::abort_task_on_cap_breach(ctx: &TaskCtx) -> AbortDecision (new fn)
  - orchestrator::cloud_spend::TaskAbortReason::SpendCapBreached / ::VelocityCapBreached (new enum)
  - orchestrator::velocity_cap::VelocityCapEnforcer (new struct)
  - orchestrator::velocity_cap::VelocityWindow { OneMin, TenMin, OneHour, TwentyFourHour } (new enum)
  - orchestrator::automix_gate::verify_before_escalate(failed_output: &str, task: &Task) -> AutomixVerdict (new fn)
  - orchestrator::automix_gate::AutomixVerdict { Continue, Retry, Escalate } (new enum)
  - orchestrator::cost_router::route_for_difficulty_and_budget(diff: f32, budget_usd: f64) -> RouteDecision (new fn)
  - orchestrator::cost_router::RouteDecision { Local, Sonnet, Haiku, DeepSeekFlash, Opus { requires_confirmation: bool } } (new enum)
  - orchestrator::free_tier_guard::check_quota(backend: Backend) -> QuotaStatus (new fn)
  - orchestrator::free_tier_guard::QuotaStatus { Ok(remaining), Approaching(remaining), Exceeded } (new enum)
  - orchestrator::runtime_orchestrator::Backend::DeepSeekV4Flash (new variant)
  - orchestrator::runtime_orchestrator::Backend::CerebrasFree (new variant)
  - orchestrator::runtime_orchestrator::Backend::GroqFree (new variant)
  - cli::cloud::override_cap_subcommand (`elai cloud override-cap`)
  - SsbActionType::TaskAbortedBySpendCap (new)
  - SsbActionType::VelocityCapBreached (new)
  - SsbActionType::AutomixVerifierVerdict (new)
  - SsbActionType::CostRouterDecision (new)
  - SsbActionType::CostEstimateInaccurate (new)
  - SsbActionType::FreeTierQuotaApproaching (new)
  - SsbActionType::FreeTierQuotaExceeded (new)
  - SsbActionType::LayeredCapManualOverride (new)
consumes_symbols:
  - orchestrator::grounding_client::CloudGroundingClient (existing per elai.map line 77007)
  - orchestrator::runtime_orchestrator::PerRoleRuntime (B-37 SHIPPED)
  - orchestrator::task_runner::TaskRunner (existing; cost-router invoked before cloud dispatch)
  - core::clock::ClockProvider (C-15 PLANNED; velocity-cap rolling windows need clock injection)
  - core::paths (B-1 SHIPPED; cache TOML location)
  - C-46 TaskDifficultyClassifier (PLANNED; if shipped, share the classifier; else standalone)
  - SsbActionType::CloudSpend (existing per B-27 PLANNED; E-39 extends with 8 new siblings)
reachable_from:
  - crates/orchestrator/src/task_runner.rs::cloud_dispatch (pre-call: cost_router + free_tier_guard; per-call: velocity_cap; post-call: ceiling check + abort decision)
  - crates/cli/src/main.rs (`elai cloud override-cap` subcommand; `elai cloud status` to show velocity-cap counters)
  - crates/dashboard-v2/src-tauri/src/commands.rs (`get_velocity_cap_status`, `get_cost_router_recent`, `get_automix_verdicts_recent`)
stub_ban:
  - no soft warn-only on max_spend_per_task breach (CLAUDE.md HARD RULE inferred from report: must ABORT)
  - no point-estimate cost budget (must be P50+P95 band per arxiv:2604.22750 — 30x variance)
  - no .unwrap() in cost-router / velocity-cap / automix-gate paths
  - no cloud route without cost_router decision logged (CostRouterDecision SSB MANDATORY)
  - no LLM gateway URL accepted (E-24 Task 6 covers; reinforced here — gateway adds 10-50ms + breaks prompt cache)
  - no all-Opus default (report anti-pattern: $15-30/session vs $3-7 mixed; default routing to Sonnet, Opus only on `(very-hard + critical)`)
  - no Batch API for interactive workloads (anti-pattern per report — 24h SLA breaks user feedback loop)
  - no cache-write to 1-hour TTL without 2+ reads (anti-pattern: cache-write multiplier debt)
frontend_pair: crates/dashboard-v2/src/lib/components/CostEnforcementPanel.svelte (new — surfaces velocity-cap counters, free-tier remaining quota, AutoMix saved-spend, cost-estimate accuracy, layered-cap status)
observable_effect:
  trigger: "elai /task --mode hybrid 'simulate retry storm: 100 fast cloud calls' (synthetic fixture; injects 100 cloud_dispatch invocations within 60s)"
  expect_ssb: "VelocityCapBreached on minute-window cap; TaskAbortedBySpendCap if $/min cap also breached; AutomixVerifierVerdict::Continue on intermediate retries that the verifier judged unworthy of escalation"
  expect_artifact: "task aborts mid-storm with partial result + aborted_by_spend_cap=true; CostRouterDecision rows logged for every dispatch; total dollar burn capped at max_spend_per_task"

## DETERMINISTIC_CHECK

```bash
# 1. HARD ceiling enforces mid-task abort (not just refuse-next-call)
rg "TaskAbortedBySpendCap|abort_task_on_cap_breach" .elai_cc/crates --type rust
# Expected: ≥2 hits (variant + dispatch)

# 2. Velocity cap module exists with 4 windows
rg "VelocityWindow::(OneMin|TenMin|OneHour|TwentyFourHour)" .elai_cc/crates/orchestrator/src/velocity_cap.rs
# Expected: 4 hits

# 3. AutoMix gate runs before HYBRID escalation
rg "verify_before_escalate|AutomixVerdict" .elai_cc/crates/orchestrator --type rust
# Expected: ≥3 hits (impl + call site + variant decls)

# 4. Cost router with difficulty + budget decision matrix
rg "route_for_difficulty_and_budget|RouteDecision::(Local|Sonnet|Haiku|DeepSeekFlash|Opus)" .elai_cc/crates/orchestrator --type rust
# Expected: ≥5 hits

# 5. New backends registered
rg "Backend::(DeepSeekV4Flash|CerebrasFree|GroqFree)" .elai_cc/crates/orchestrator/src/runtime_orchestrator.rs
# Expected: 3 hits

# 6. Free-tier guard
rg "FreeTierQuotaApproaching|FreeTierQuotaExceeded" .elai_cc/crates/orchestrator/src/free_tier_guard.rs
# Expected: ≥2 hits

# 7. Prompt-cache discipline — every cloud call has cache_control
rg "cache_control" .elai_cc/crates/orchestrator/src/grounding_client.rs
# Expected: ≥4 hits (system / spec / manifest / tool schema)

# 8. 8 new SSB action variants
rg "TaskAbortedBySpendCap|VelocityCapBreached|AutomixVerifierVerdict|CostRouterDecision|CostEstimateInaccurate|FreeTierQuotaApproaching|FreeTierQuotaExceeded|LayeredCapManualOverride" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: 8 hits

# 9. Layered cap default
rg "layered_caps.*100|cloud.layered_caps" .elai_cc/elai_cc.toml
# Expected: ≥1 hit (default $100/day hard cutoff)

# 10. CLI override-cap subcommand
rg "override-cap|override_cap_subcommand" .elai_cc/crates/cli/src/main.rs
# Expected: ≥1 hit
```

## ACCEPTANCE

- [ ] Spend-cap breach mid-task ABORTS cooperatively (drains in-flight call, banks cost, emits TaskAbortedBySpendCap SSB row)
- [ ] Per-agent-role caps (executor / auditor / scope_planner / memory_writer) configurable + enforced
- [ ] Velocity caps (1-min, 10-min, 1-hour, 24-hour) enforced; breach aborts task + emits SSB row
- [ ] AutoMix verifier runs before HYBRID escalation; >50% of failed-output cases route to Continue/Retry without cloud (measured on a 30-baby-step ablation per report Open Question 3)
- [ ] Cost router decision logged for every cloud dispatch; difficulty + budget → route mapping deterministic
- [ ] Prompt-cache discipline: every cloud call has `cache_control` on system+spec+manifest+tool schema; per-session cache-hit-rate gauge surfaced; target >70% on typical session
- [ ] 3 new backends (DeepSeekV4Flash / CerebrasFree / GroqFree) registered, with anti-slopsquatting verification dates in the decision doc
- [ ] Free-tier quota guard refuses calls at ≥95% daily quota; warns at 80%
- [ ] Cost-estimate accuracy tracked; CostEstimateInaccurate SSB fires on actual/estimate outside [0.5, 2.0]; cost field is `{ p50, p95 }` band
- [ ] Layered cap default ships at $50/day soft alert > $100/day hard cutoff > $1000/month manual-override-required
- [ ] `elai cloud override-cap` subcommand requires typed confirmation string; emits LayeredCapManualOverride SSB row
- [ ] No all-Opus default routing; no Batch API for interactive workloads; no 1-hour cache write without 2+ reads
- [ ] No new LLM gateway dependencies in Cargo.lock (E-24 invariant reinforced here)

## Tests to add

- `crates/contracts/tests/spend_cap_mid_task_abort_contract.rs`: synthetic task with `max_spend_per_task = $0.05`; cloud call exceeds → assert abort + partial result + TaskAbortedBySpendCap SSB row
- `crates/contracts/tests/velocity_cap_1min_breach_contract.rs`: 50 cloud_dispatch calls within 60s with limit=10/min → assert breach + abort + VelocityCapBreached SSB
- `crates/contracts/tests/automix_verifier_gate_contract.rs`: 10 failed-output fixtures (5 structurally valid → Continue, 3 borderline → Retry, 2 truly broken → Escalate); assert verdict mix
- `crates/contracts/tests/cost_router_difficulty_x_budget_contract.rs`: 16-cell decision matrix (4 difficulty bins × 4 budget bins) → assert correct RouteDecision for each
- `crates/contracts/tests/prompt_cache_control_present_on_every_cloud_call_contract.rs`: 5 synthetic cloud calls → assert `cache_control` on system + spec + manifest + tool schema in every payload
- `crates/contracts/tests/free_tier_quota_guard_blocks_at_95_contract.rs`: GroqFree at 13680 / 14400 daily req → next call rejected + FreeTierQuotaExceeded SSB; at 11520 (80%) → FreeTierQuotaApproaching warning
- `crates/contracts/tests/cost_estimate_inaccuracy_threshold_contract.rs`: estimate=$0.10, actual=$0.30 → CostEstimateInaccurate (3.0× ratio); estimate=$0.10, actual=$0.12 → no row (within band)
- `crates/contracts/tests/per_role_cap_enforcement_contract.rs`: executor at $0.04 of $0.05 cap blocks; auditor at $0.04 of $0.10 cap continues — same task ctx
- `crates/contracts/tests/layered_cap_manual_override_requires_confirmation_contract.rs`: `elai cloud override-cap` without typed string → error; with correct string → LayeredCapManualOverride SSB row
- `crates/contracts/tests/no_all_opus_default_contract.rs`: 100 synthetic routing decisions with default config → fewer than 10% route to Opus

## Files touched

- CREATE: `crates/orchestrator/src/velocity_cap.rs` (~180 LOC)
- CREATE: `crates/orchestrator/src/automix_gate.rs` (~220 LOC)
- CREATE: `crates/orchestrator/src/cost_router.rs` (~250 LOC)
- CREATE: `crates/orchestrator/src/free_tier_guard.rs` (~100 LOC)
- MODIFY: `crates/orchestrator/src/cloud_spend.rs` (+~120 LOC for mid-task abort + per-role caps + layered caps)
- MODIFY: `crates/orchestrator/src/grounding_client.rs` (+~80 LOC for cache_control discipline pass + cache-hit-rate gauge)
- MODIFY: `crates/orchestrator/src/runtime_orchestrator.rs` (+~60 LOC for 3 new backend variants + adapter impls)
- MODIFY: `crates/orchestrator/src/task_runner.rs` (+~50 LOC for cost-router invocation before cloud dispatch + velocity-cap check + automix gate)
- MODIFY: `crates/orchestrator/src/ssb.rs` (+8 SsbActionType variants)
- MODIFY: `crates/cli/src/main.rs` (+~30 LOC for `elai cloud override-cap` subcommand)
- MODIFY: `elai_cc.toml` schema docs (`[cloud.per_role_caps]`, `[cloud.velocity_caps]`, `[cloud.cache]`, `[cloud.layered_caps]`, `[cloud.cost_router]`, `[cloud.free_tier]`)
- MODIFY: `crates/dashboard-v2/src-tauri/src/commands.rs` (+~80 LOC for `get_velocity_cap_status`, `get_cost_router_recent`, `get_automix_verdicts_recent`, `get_free_tier_quotas`)
- CREATE: `crates/dashboard-v2/src/lib/components/CostEnforcementPanel.svelte` (~160 LOC)
- CREATE: 10 contract tests under `crates/contracts/tests/` per "Tests to add"
- CREATE: `workflow/reasoning/E-39-decisions.md`

## Citations / Grounding

- platform.claude.com/docs/en/about-claude/pricing (cited in source) — Opus 4.7 $5/$25; Sonnet 4.6 $3/$15; Haiku 4.5 $1/$5; cache write 1.25x/2x; cache read 0.1x; Batch 50% off; Fast mode 6x
- finout.io/blog/anthropic-api-pricing (cited in source) — Batch + caching stack to ~95% combined savings
- dev.to/whoffagents/claude-prompt-caching-in-2026-the-5-minute-ttl-change (cited in source) — Anthropic TTL cut 60→5min early 2026; 30-60% bill increase
- developers.openai.com/api/docs/pricing (cited in source) — GPT-5.5 $5/$30; GPT-5.4 $2.50/$15; auto-caching free at >1024 tokens
- ai.google.dev/gemini-api/docs/pricing (cited in source) — Gemini 3 Flash $0.50/$3
- api-docs.deepseek.com/quick_start/pricing (cited in source) — DeepSeek V4-Flash $0.30/$0.50; V4-Pro $0.435/$0.870 promo; 90% cache discount
- arxiv:2305.05176 FrugalGPT (cited in source) — cascade matches GPT-4 at 98% cost reduction
- arxiv:2310.12963 AutoMix ICLR 2025 (cited in source — VERIFY ID before final commit) — self-verify cascade; >50% cost reduction at iso-quality
- arxiv:2511.07396 C3PO (cited in source) — probabilistic cost-constraint optimization for cascades
- arxiv:2604.22750 "How Do AI Agents Spend Your Money" (cited in source — VERIFY arXiv ID; year-tag suspicious) — agentic 1000x tokens vs chat; 20-25x input/output; 30x variance same-task
- vantage.sh/blog/agentic-coding-costs (cited in source) — agentic coding 1-3.5M tokens/task; input drives bill
- ravoid.com/blog/ai-agent-budget-enforcement (cited in source) — $47K LangChain 4-agent loop incident
- leanopstech.com/blog/agentic-ai-cost-runaway-token-budget-2026 (cited in source) — 2.3M-call weekend retry storm
- handsonarchitects.com/blog/2025/denial-of-wallet-cost-aware-rate-limiting-part-1 (cited in source) — RPS limits insufficient for LLM endpoints
- tokenmix.ai/blog/groq-free-tier-limits-2026 (cited in source) — Groq 30 RPM / 6K TPM / 14.4K req/day default
- getaiperks.com/en/ai/cerebras-free-tier-guide (cited in source) — Cerebras 1M tokens/day free since April 2026
- docs.vllm.ai/en/stable/design/prefix_caching (cited in source) — vLLM hash-based LRU; <1% throughput penalty at 0% hit rate
- venturebeat.com/programming-development/devin-2-0-is-here-cognition-slashes-price (cited in source) — Devin $2.25/ACU; moderate task 5-20 ACU = $11-45
- morphllm.com/comparisons/morph-vs-aider-diff (cited in source) — Aider 4.2x fewer tokens than Claude Code

## Open questions / risks

- arxiv:2604.22750 citation verify (Tasks 2 + 9, used for the 20-25x input/output and 30x variance numbers): the year-tag `2604` is anomalous for May 2026 (would be April 2026 in YYMM, plausible but pre-pub). WebSearch arxiv.org before final commit. If the paper doesn't exist or numbers aren't grounded, cite vantage.sh/blog/agentic-coding-costs as the lower-strength alternative and mark the 20-25x / 30x figures as "Vantage 2026 measurement, paper-pending."
- arxiv:2310.12963 AutoMix citation verify (Task 4): the report's "automix-llm.github.io/automix/" page and "ICLR 2025" tag are referenced but the arXiv ID isn't given in the report TL;DR. WebSearch the canonical AutoMix arXiv ID; the >50% cost reduction claim must be paper-grounded before E-39 ships.
- DeepSeek V4-Flash legal posture (Task 7): per E-38 strategic posture, DeepSeek's Huawei-chip-native posture is a decoupling signal. Confirm US export-controls (Bisi 2026) + EU GDPR posture before wiring `Backend::DeepSeekV4Flash` for ELAI tenants in restricted jurisdictions. The decision doc must surface this question to the user.
- Cerebras 1M-token-free sustainability (report Open Question 5): is the free tier a launch promo or sustainable subsidy? Re-check Q3 2026 before architecting dependency. The free_tier_guard module (Task 8) makes the dependency explicit + graceful — if Cerebras revokes the free tier, the guard fails closed and routes to DeepSeek/Sonnet automatically.
- AutoMix verifier overhead: every L3 failure now adds a local Qwen 3B sidecar call. On the typical 20% L3-failure rate, that's a +20% sidecar call rate. MEASURE the verifier latency + per-call sidecar cost on a 30-step ablation before declaring ship-worthy. If the verifier itself drops >50% of would-be-cloud calls, the math works; if not, gate Task 4 to opt-in.
- Per-role cap default values (Task 2): need user input on whether `auditor > executor` (auditor reads more context but writes less) or the opposite. Decision-doc proposal: auditor 2× executor cap; revise on first session feedback.
- Anti-slopsquatting on Cerebras + Groq + DeepSeek API URLs + model IDs (Task 7): MANDATORY before adding any of the 3 backends. Check api-docs.deepseek.com / cloud.cerebras.ai / console.groq.com for current model identifiers + last-published dates. Document in decision doc.
- E-37 PromptArmor classifier cost (cross-cutting): the L3 PromptArmor classifier (E-37 Task 3) adds per-ingest classifier cost. If it routes through Claude Haiku for high-stakes, it counts against the E-39 ceilings. Coordinate so the classifier cost is bucketed under a separate `safety_classifier` role with its own per-role cap.
- Pre-execution difficulty estimator (Task 5) bootstrap: needs a reference-task corpus for cosine similarity. C-46's TaskDifficultyClassifier (PLANNED) defines this. If C-46 is not yet shipped, E-39 Task 5 ships with a minimal corpus (10 hand-curated reference tasks) and notes the upgrade path. Otherwise inherits from C-46.

## Next

E-39 + B-27 + C-5 together complete the cost-economics picture. E-39 unblocks: confident cloud-mode autonomous sessions (memory:feedback_autonomous_mode) — HARD ceilings + velocity caps + AutoMix gate make the wallet attack class survivable. Future E-Ns: (a) full FrugalGPT cascade chain (cheap-first → next-tier → frontier) — current E-39 implements 2-tier; the 3-4-tier chain is a follow-up; (b) C3PO probabilistic cost-constraint integration once arxiv:2511.07396 is verified; (c) Process Reward Model (PRM) extra-spend monitoring per arxiv:2509.02360 — additive only if PRM lands in ELAI's reasoning stack. Coordination: E-37 (PromptArmor classifier cost) buckets under `safety_classifier` per-role cap here; E-36 (vision parses) buckets under `doc_parser` per-role cap. E-38 posture doc (Section: per-lab signals) must reflect the 3 new backend wirings and the DeepSeek legal-posture caveat.
