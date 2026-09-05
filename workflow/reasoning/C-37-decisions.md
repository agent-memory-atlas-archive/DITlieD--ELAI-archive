C-37 per-role decoding defaults re-evaluation — session 2/4 (s-25120 cluster)

Date: 2026-05-24

Scope
-----
Close the 14 MISSING / 1 PARTIAL / 1 NOT_WIRED rows from the prior re-check
report (workflow/sessions/s-25120/cross-check/C-37_per_role_decoding_defaults_reeval-report.md)
without running live GPU. Bring the C-37 surface from PLANNED to STAGED — a
real pilot run is RUNPOD-PENDING per plan §3.

Plan-drift resolution
---------------------
The plan cites 7 nonexistent paths (runtime crate, agent/auditor.rs,
agent/critic.rs, testing-trust crate, ssb/schema.rs submodule,
verification/reports/C-plan dir, and an absent [decoding.per_role] toml
section). Per the session prompt PLAN REBASE block, we executed in-line
substitutions:

  - testing-trust crate -> .elai_cc/python/elai_inference/c37_snowballing/
    Python pkg (mirrors mucoco pattern from C-36).
  - runtime/src/decoding.rs -> crates/providers/src/decoding.rs (real path).
  - agent/auditor.rs + critic.rs -> orchestrator dispatch surface
    (agent_pool::spawn role-dispatch site).
  - ssb/schema.rs -> single-file ssb.rs (same as C-36).
  - elai_cc.toml [decoding.per_role] section -> ADDED.
  - verification/reports/C-plan/ -> workflow/reasoning/ (this file).
  - ROLE TAXONOMY MISMATCH -> kept ELAI's 4-role taxonomy for internal
    decoding params (PaperRole enum: Planner/Coder/Critic/Reflection)
    with paper-grounded values for Critic + Reflection; Planner + Coder
    tagged "ungrounded-pending-eval".

GAP-B closure (§0 prerequisite)
-------------------------------
Integration explorer flagged that default_decoding_for(role) had ZERO
production callers — the C-26 policy table shipped without dispatch
wiring. C-37 closes the gap via two complementary moves:

  1. SidecarProvider::apply_role_default_decoding(role: RoleDecoding) is
     a new public API on the provider that wraps set_decoding_config with
     a tracing::info! row. Child processes (which own the SidecarProvider)
     can now call this once per session to apply the per-role policy.

  2. agent_pool::spawn emits a tracing::info! row at the orchestrator-side
     dispatch site naming the role + the resolved DecodingMode discriminant.
     This is observable from the parent process (where the orchestrator
     runs) even though the SidecarProvider itself lives in the child.
     The new RoleDecoding::from_snake_case helper bridges between the
     agent-role label and the providers-crate enum without taking a
     providers->orchestrator dep edge.

The bridge is a 2-edit refactor (decoding.rs from_snake_case + agent_pool.rs
emit), not a full IPC re-plumbing — that would be its own plan.

Paper-mapping decision
----------------------
arXiv:2604.06066 ternary roles (Actor/Evaluator/Reflector, all Qwen3-8B)
map onto ELAI's 4-role taxonomy as:

  Actor      <=> CODER
  Evaluator  <=> CRITIC
  Reflector  <=> REFLECTION

The paper does NOT describe a Planner role; ELAI's Planner is a workflow
extension. Its decoding defaults remain at the pre-C-37 values and carry
the explicit grounding tag "ungrounded-pending-eval" so future re-grounding
work surfaces it by grep.

Coder defers per explicit C-37 scope: the paper uses temperature=0.1
uniformly across Actor (code/reasoning generator) but ELAI's Coder runs
warmer (0.4). Flipping it requires its own measurement (the paper's 8B
model and ELAI's 7B abliterated model are different studies). Both Coder
and Planner are explicitly tagged ungrounded so a future C-plan can ground
them per-role.

SCHEMA_VERSION bump
-------------------
Read live value 105 at edit time (concurrent session s-31708 has not
bumped yet). Bumped to 106 atomically with:

  - new SsbActionType::PerRoleDecodingApplied variant
  - new PerRoleDecodingAppliedPayload struct (role, decoding_mode,
    constrained, max_retries_on_invalid)
  - SQL allowlist gains 'per_role_decoding_applied'
  - 8-site lockstep: enum @2394 + SQL CHECK @1321 + as_sql_str @3356 +
    from_str @3765 + SsbPayload @6617 + action_type() @7012 + sample dispatch
    @12245 + roundtrip vec @14581
  - 9th lockstep site: action_type_lock.golden (ordinal 0202) + ALL_VARIANTS
    + TOTAL_VARIANTS (310 -> 311 — note the count is decoupled from the
    SCHEMA_VERSION; both moved by +1 here but the rules are independent)

Runpod gating
-------------
Live model inference (Qwen3-8B on remote 3090) is deferred per plan §3.
The harness is fully authored but the actual replication run requires GPU
and is documented as RUNPOD-PENDING in run.py:

  - Pool A (n=55, baseline-solved-2-to-5): paper Avg Trials anchor
  - Pool B (n=45, baseline-failed): paper Success Rate anchor

VRAM strategy (per source-interface findings):
  Path (a) — Qwen3-8B at NF4 (~5 GB) alongside existing 7B sidecar
              (~7 GB total on the 24 GB 3090)
  Path (b) — Qwen3-8B at bf16 (~16 GB) — STOP the existing sidecar first

Default in run.py is the schema_hint mode (sidecar-compatible). The full
paper-exact Outlines + pydantic FSM path needs in-process generation with
a logit processor; treat that as a separate replication study.

Open follow-up items
--------------------
1. session 3 (s-31708) is rebuilding main.rs; the new bench_snowballing
   module file ships but the `mod bench_snowballing;` declaration is
   deferred. memory-writer logs this for session-end pickup.
2. RUNPOD-PENDING items (2) listed in run.py docstring.
3. C-37 decision matrix (plan §4) cannot fire until pilot data lands; the
   harness measures, the analyzer flips.
4. SidecarProvider::apply_role_default_decoding has no in-process caller
   yet — child-process agents need to call it once at startup. That's
   the next ELAI-side wire-in step (would belong to a successor plan or
   C-37c if the pilot returns positive).

Test results
------------
- providers crate: cargo check -p elai-providers GREEN (1 pre-existing
  dead_code warning, not C-37).
- orchestrator crate: cargo check -p elai-orchestrator GREEN (8
  pre-existing warnings, not C-37).
- contracts crate (relevant tests):
    cargo check -p elai_cc_contracts --test c37_per_role_decoding_contract
    --test c1_ssb_new_variants_contract
    --test c26_constrained_decoding_contract
  All three GREEN (the new c37 contract joins the existing two without
  breaking either).
- Python harness:
    PYTHONPATH=. python -m pytest elai_inference/tests/test_c37_snowballing_contract.py
    7 PASSED (5-class enum / schema round-trip / uniform hyperparams /
    thresholds / accuracy-direction / max-trials clamp / error-dist shape).
- Workspace-level errors in elai-cli main.rs ARE PRE-EXISTING (per
  context-package "Broken: pre-existing CLI workspace errors") and owned
  by concurrent session 3.
