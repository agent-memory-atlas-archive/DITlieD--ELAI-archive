E-31 — Coding model leaderboard + escalation policy — decisions
session: s-orch-e31 (orchestrate continuation in s-26556 campaign)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-31_coding_model_leaderboard.md

(a) SCOPE-OWNERSHIP DETERMINATION + TIER SPLIT

E-31 spans crates this run does NOT own. Per the s-26556 orchestrate brief: OWNED = crates/dashboard-v2 + crates/knowledge + crates/cli + crates/contracts + workflow/{archetypes,vision_rag,compliance}/. AVOIDED = crates/providers/runtime + crates/orchestrator/sandbox + python/elai_inference + crates/reasoning. Grey-zone = crates/orchestrator/{root}.

The integration-explorer + context-compiler agents (s-orch-e31 batch) classified per-task:

  Task 1 — tier-matrix.md doc (workflow/) — OWNED
  Task 2 — sidecar head_dim assert (python/elai_inference/server.py) — AVOIDED
  Task 3 — escalation_policy.rs in providers/runtime — AVOIDED + plan-drift (file already exists at crates/orchestrator/src/escalation_policy.rs shipped by E-32)
  Task 4 — wire EscalationPolicy into RuntimeOrchestrator::pick_runtime — PLAN DRIFT (method does not exist; real surface is try_complete/health_snapshot; routing lives in cloud_mode.rs::CloudModeRouter::route, not in providers/runtime)
  Task 5 — benches/coding_lane/run_internal_pro.py scaffold — OWNED (benches/ is not AVOIDED)
  Task 6 — CLAUDE.md pointer (workflow tooling root) — OWNED
  Task 7 — head_dim_invariant_contract.rs in contracts/tests — OWNED (statically parses config.json, no VRAM, no sidecar)
  Task 8 — workflow/reasoning/E-31-decisions.md — OWNED (this file)
  Task 9 — head_dim_invariant_verified + cloud_route_decision SSB rows — grey-zone (ssb.rs lives in orchestrator/{root})
  Task 10 — RuntimePanel.svelte EscalationPolicy row — OWNED dashboard side, but requires hand-written bindings AND a producer to wire — deferred

E-31 SHIPPED THIS RUN (E-31.A): Task 7 + Task 8 only.
  - .elai_cc/crates/contracts/tests/head_dim_invariant_contract.rs (NEW): 3 tests — (1) walks canonical_model_paths() and asserts head_dim == 128 against any config.json that happens to be staged; (2) fixture-shape parser test always exercises the parser even when no weights are staged; (3) malformed-input parser-resilience test. CI-resilient: absence of staged weights is NOT a contract violation. Producer-locked at .elai_cc/models/Qwen2.5-Coder-{3B,7B}-Instruct{,-abliterated}/config.json.
  - this file.

DEFERRED to E-31.B (a session that owns python/elai_inference + providers/runtime + orchestrator/{root}):
  - Task 2 — sidecar refuse-yarn-at-boot precedent extends to refuse-bad-head-dim-at-boot.
  - Task 3 — escalation_policy.rs already exists at .elai_cc/crates/orchestrator/src/escalation_policy.rs (E-32). E-31.B reconciles the plan's "create new" claim against the shipped-and-different shape.
  - Task 4 — CloudModeRouter::route extension OR (per integration-explorer suggestion) rename to cloud_route_policy.rs and colocate with E-32. Decision needed at the start of E-31.B.
  - Task 9 — 2 SSB variants (CloudRouteDecision + HeadDimInvariantVerified), 14 atomic touchpoints in ssb.rs (under the 3-variant deferral threshold but still meaningful work). Rides with Task 4.
  - Task 10 — RuntimePanel EscalationPolicy row, hand-written bindings + Tauri command. Rides with Task 9 producers.

DEFERRED to E-31.C (a session focused on the bench harness):
  - Task 1 — tier-matrix.md doc requires a baseline measurement pass first; rather than write a stale-on-arrival matrix, defer until benches/coding_lane/ has actually run.
  - Task 5 — benches/coding_lane/ scaffold. Rides with Task 1.
  - Task 6 — CLAUDE.md pointer. Rides with Task 1 (no point pointing at a doc that doesn't exist yet).

(b) PLAN-DRIFT FINDINGS (carry forward to E-31.B)

The integration-explorer agent surfaced four HIGH-severity drifts that any future E-31 session must reconcile:

  DRIFT-1: Plan Task 4 references `RuntimeOrchestrator::pick_runtime`. The real surface is `RuntimeOrchestrator::try_complete` + `health_snapshot` per runtime_orchestrator.rs:9-13. Plan was written against a different code shape.

  DRIFT-2: Plan Task 3 directs creating `.elai_cc/crates/providers/src/runtime/escalation_policy.rs`. The escalation policy module ALREADY EXISTS at `.elai_cc/crates/orchestrator/src/escalation_policy.rs` shipped by E-32 with `HandoffContext` / `HandoffDecision` / `EscalationPolicyError` / `evaluate_handoff`. The plan's "new file" claim conflicts with shipped reality.

  DRIFT-3: Plan claim "head_dim mismatch triggers performance fallback" — the python/elai_inference/server.py model loader at server.py:1706-1711 reads head_dim with `getattr(..., "head_dim", 256)` DEFAULT 256, which means a Qwen2-family model loads with the WRONG head_dim attribute on the inference path. This is a real bug the plan's "assert at boot" pattern would catch, but the actual config.json (verified this session) has NO head_dim field — Qwen2 computes it from hidden_size / num_attention_heads. Any sidecar assert must compute the same way.

  DRIFT-4: 12+ paper/URL citations in the plan are UNVERIFIED. Per memory feedback_paper_citation_verify, 2 prior bad cites were caught in B-40/B-41/B-42 plans (DPO arxiv:2305.18290 and Kamoi arxiv:2406.01297 substitutions). E-31.B should run a WebSearch verification pass on every cited URL / arxiv ID before relying on any quoted figure.

(c) WHY THE CONTRACT TEST IS ENOUGH FOR THIS RUN

Head-dim is a load-bearing invariant for the sidecar attention kernel path (SDPA selects 128 by default; FA2 has a 128/64/256 head-dim switch; xformers has its own table). A model family swap that quietly lands a 96- or 64-head-dim variant would degrade attention throughput by an order of magnitude AND change the cache layout in ways that don't surface as a hard error — they surface as latency creep and intermittent NaN diagnostics under load. Locking this in a CI contract is a real safety win, independent of when the sidecar-side runtime assert lands.

The contract is CI-resilient: when models aren't staged (the common case in CI sandboxes where huge weight files are gitignored), the test skips silently. When models ARE staged (developer machines, the GPU host), the test fails loudly on any mismatch. The fixture-shape parser test always runs and verifies parser correctness even when no weights are present.

(d) SCHEMA + SSB DELIBERATELY NOT BUMPED

This run does not add SsbActionType::CloudRouteDecision or SsbActionType::HeadDimInvariantVerified. Rationale: both producers live in AVOIDED crates (cloud_mode.rs is grey-zone; python sidecar is AVOIDED). Adding the enum variants without producers creates dead schema rows that no writer ever produces — declined per B-23 4I.1 precedent already documented in ssb.rs prior bumps. When E-31.B lands the producers, the SCHEMA bump 97 → 98 + 2 variants + 14 touchpoints ride atomically with that work.

(e) WHY THIS E-N WAS SCOPED DOWN MID-DISPATCH

s-orch-e31 dispatched context-compiler + integration-explorer-elai in parallel per orchestrate-protocol H-6 BEFORE any Edit/Write. Both agents independently returned PLAN_DRIFTED_REQUIRES_IMPROVEMENT verdicts with overlapping findings. Per H-1, the agent dispatch is not optional, so I took their findings at face value rather than overriding with optimism. Per H-7, when a plan requires extensive forbidden-crate edits, the responsible path is to ship the OWNED slice and defer the rest with explicit per-task split docs (this file). The alternative — running the plan-improvement protocol on E-31 in this run — would have consumed the remaining session budget for a downstream-pass-only deliverable; better to leave that work to a session that has cleared time for the full improve + execute cycle.

Pattern across this orchestrate campaign so far: E-44, E-16, E-31 all returned PLAN_DRIFTED_REQUIRES_IMPROVEMENT. Hypothesis: most of the s-26556-queue E-plans were authored against a snapshot of the codebase from before B-37/E-17/E-32/E-33 shipped, when SCHEMA_VERSION was still in the 40s and several modules they reference had different shapes. The plan-improvement protocol in .claude/skills/planning-workflow/SKILL.md should be run on each before EXECUTING. Carry this to the SESSION_HANDOFF.

END E-31-decisions.md
