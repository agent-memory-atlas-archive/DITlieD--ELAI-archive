E-31 tier matrix — canonical VRAM + routing decision table

Status: 2026-05-31 — sub-phase 6A shipped (Pattern-2 RouteDecisionEvent + RuntimeRoutingPolicy wire-dark). 24 GB Pro column (Qwen2.5-Coder-32B SWE-Pro measurement) deferred to 2x3090 hardware. The 7B local config is canonical per metrics/baseline_findings_and_actions.md Part 1.

Objective

Pin the per-VRAM-tier model choice + routing matrix so the runtime knows, ahead of an inference call, which backend to dispatch under which condition. The matrix below is the authoritative answer for "which model on which tier" — code is allowed to read it but not duplicate it.

VRAM tier table

Tier | Local model | dtype / kernel | Context window | KV cache @ ctx | Target bench range | Source
-----+-------------+----------------+----------------+----------------+--------------------+-------
8 GB | Qwen2.5-Coder-3B-Instruct-abliterated | NF4 4-bit + SDPA | 8K-12K | ~0.4 GB @ 12K | Pass@1 0.45-0.55 on internal-PRO (proxy) | memory:project_vram_tier_plan + project_qwen3b_baseline
12 GB | Qwen2.5-Coder-7B-Instruct-abliterated | NF4 4-bit + SDPA | 16K-20K | ~0.8 GB @ 20K | Pass@1 0.55-0.65 on internal-PRO (proxy) | memory:project_qwen3b_baseline (measured 1.92 GB model + ~0.8 GB KV @ 20K = ~2.7 GB working set)
16 GB | Qwen2.5-Coder-7B-Instruct-abliterated | NF4 4-bit + FA2 (env=elai-fa2) | 20K-25K | ~1.0 GB @ 25K | Pass@1 0.60-0.70 on internal-PRO (proxy) | memory:project_dola_tq_findings (FA2 + TQ-4 best combo for 16 GB)
24 GB | Qwen2.5-Coder-7B-Instruct-abliterated | NF4 4-bit + FA2 | 25K-32K | ~1.2 GB @ 32K | Pass@1 0.65-0.75 on internal-PRO (proxy) | memory:project_qwen3b_baseline (HF model-card 32K native window)
24 GB Pro | Qwen2.5-Coder-32B-Instruct-abliterated | TQ-4 + FA2 (RUNPOD-PENDING) | 16K-24K | ~3.5 GB @ 24K | NOT MEASURED — open question | memory:project_vram_tier_plan (2x3090 coming for 32B + TQ testing)
48 GB | Qwen2.5-Coder-32B-Instruct-abliterated | FA2 full precision | 32K | ~4.5 GB @ 32K | NOT MEASURED — open question | memory:project_vram_tier_plan
Cloud | Claude Sonnet 4.6 (workhorse), Opus 4.7 (confirmation-gated), DeepSeek V4-Flash (cheap-volume floor RUNPOD-PENDING) | API | 200K | n/a | n/a — cloud handles latency-budget itself | crates/orchestrator/src/cost/cost_router.rs (E-39 RouteDecision matrix)

Routing matrix (RuntimeRoutingPolicy::evaluate)

Difficulty x Criticality | Budget Ample (>5 USD) | Budget Tight (<=5 USD)
-------------------------+-----------------------+---------------------------
Easy x Low or High | LocalSidecar | LocalSidecar
Medium x Low | CloudSonnet | CloudDeepSeekFlash
Medium x High | CloudSonnet | CloudDeepSeekFlash
Hard x Low | CloudSonnet | CloudDeepSeekFlash
Hard x High (first attempt) | CloudSonnet | CloudDeepSeekFlash (degraded; logs warning)
Hard x High (audit re-try) | CloudOpus (confirmation-gated per RouteDecision::Opus) | CloudOpus

Additional gates

- VRAM headroom floor: 6 GB. If detected_vram < 6 GB AND > 0 GB (cloud-tier == 0 means no measurement), force cloud route. Low criticality -> CloudDeepSeekFlash. High criticality -> CloudSonnet. Reason: VramHeadroomLow.
- Audit-failure bin shift: prior_audit_failures shifts the difficulty bin upward (Easy -> Medium -> Hard, capped at Hard). 1 failure on a Hard + High step escalates to CloudOpus with reason DifficultyEscalation.
- Ops override: RoutingContext.ops_override short-circuits everything. Reason: OpsOverride. Reserved for `elai_cc.toml` operator pins or CLI --force-route flags.

Anti-stale notes

- Qwen3-32B REJECTED per metrics/baseline_findings_and_actions.md Part 1. Qwen2.5-Coder family is canonical for the local lane.
- Qwen2.5-Coder-32B SWE-Pro measurement deferred — requires 2x3090 (~32 GB combined effective VRAM with TQ-4) per memory:project_vram_tier_plan. Until then the 24 GB Pro column above is an architectural placeholder, not a measured number.
- DoLa marginal on 2B per memory:project_dola_tq_findings. TQ-4 is the workhorse quant for 7B-on-12 GB; DoLa is disabled by default.
- FA2 env: elai-fa2 (16 GB+). On 12 GB tier, SDPA is the safer default (FA2 introduces a ~0.5 GB allocator overhead that competes with KV cache at 20K context).
- ort 2.x rc.12 is the pinned ONNX runtime per CLAUDE.md anti-stale rules (ort 1.13.1 abandoned 2022). Used by sidecar VLM integration; not on the routing-decision hot path.
- Cloud lane: DeepSeek V4-Flash RuntimeAdapter wiring is RUNPOD-PENDING (anti-slopsquatting verification of api-docs.deepseek.com URL + model id + last-published date). Until verified the adapter surfaces NotImplemented; the cost-router still emits the route arm so the SSB row stays honest.

Wire surface

- RuntimeRoutingPolicy: crates/providers/src/runtime/escalation_policy.rs — pure data + decision-table.
- RouteDecisionEvent SSB: crates/orchestrator/src/ssb.rs — Pattern-2 (43rd application); SCHEMA_VERSION 191.
- Producer: crates/orchestrator/src/cloud_mode.rs::CloudModeRouter::route — emits tracing log behind ELAI_ROUTE_DECISION_SSB_ENABLE env gate. Ssb::write_payload injection lands in sub-phase 6B alongside the dashboard RuntimePanel tile.
- Internal-pro bench scaffold: benches/coding_lane/run_internal_pro.py — dry-run scaffold; real runner RUNPOD-PENDING.

Staleness check

This document follows the docs/strategy/ELAI-2026-POSTURE.md 90-day staleness cadence. The roadmap_signals tile on the dashboard (wire-dark unless ELAI_AI_ROADMAP_SIGNALS_ENABLE=1) tracks the mtime; refresh before any major routing-matrix change.

References

- crates/providers/src/runtime/escalation_policy.rs (RuntimeRoutingPolicy)
- crates/orchestrator/src/cost/cost_router.rs (E-39 RouteDecision matrix)
- crates/orchestrator/src/cloud_mode.rs (CloudModeRouter::route)
- crates/orchestrator/src/ssb.rs (RouteDecisionEvent SCHEMA_VERSION 191)
- crates/core/src/vram_tier.rs (VramTier enum 8/12/16/24/48/Cloud)
- metrics/baseline_findings_and_actions.md Part 1 (Qwen3-32B rejection)
- memory:project_vram_tier_plan, project_qwen3b_baseline, project_dola_tq_findings, project_gpu_context
- docs/strategy/ELAI-2026-POSTURE.md (90-day staleness check precedent)
- docs/thefinalglance/E-plan/E-31_coding_model_leaderboard.md (plan source)
- workflow/sessions/s-21636/exploration/tranche6-E31-wiring.md (exploration)
- workflow/reasoning/E-31-decisions.md (DRIFT-2 reconcile)
