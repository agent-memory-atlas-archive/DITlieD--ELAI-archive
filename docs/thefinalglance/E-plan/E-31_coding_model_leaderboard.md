# E-31: Coding model leaderboard refresh + per-VRAM tier matrix lock
Objective: lock ELAI's per-VRAM model lane against the May 2026 leaderboard (SWE-bench Pro contamination-aware), formalise the "dense + head_dim 128 + FA2-compat + NF4-quantizable" invariant as an enforced runtime check, and ship a multi-runtime cloud-escalation policy table (Sonnet 4.6 default, Opus 4.7 hard-task, DeepSeek V4-Flash cheap-volume) under B-37's existing RuntimeOrchestrator. STRATEGIC E-N: deliverables are a runtime evaluation matrix + tier-table refresh + decision doc + one config file, not a feature rebuild.
Estimated effort: 1-2 sessions, ~6-10 h
Pre-requisites: B-37 SHIPPED commit fd5c2e5 (RuntimeAdapter + PerRoleRuntime + RuntimeOrchestrator); CLAUDE.md per-VRAM tier plan (memory:project_vram_tier_plan); CLAUDE.md "Local LLM" + "Remote sidecar (canonical)" sections.
Parallel-safe with: E-32 (context window — same model lane, different axis), E-33 (UX), E-34 (Rust ecosystem), E-35 (structured output).
Source report: workflow/improvement-sweep-2026-05-19/reports/31-coding-model-leaderboard.md

## Context

The May 2026 leaderboard refresh confirms ELAI's lane is correct but exposes that the Qwen2.5-Coder family has NO head_dim-128 dense successor: every Qwen3 generation (3-Coder-Next 80B-A3B, 3.6-27B, 3.6-A3B) moved to hybrid Gated DeltaNet + Gated Attention with head_dim 256, breaking the FA2 + DoLa/steering invariants ELAI relies on (Qwen3.6-27B blog 22 Apr 2026; CLAUDE.md anti-MoE rule). SWE-bench Verified scores >80 are now known to be contamination + scaffold tuning (Opus 4.5: 80.9 Verified vs 45.9 Pro on standardized scaffold; morphllm.com/swe-bench-pro). The contamination-aware Pro leaderboard puts Opus 4.7 at 64.3 / Sonnet 4.6 not yet measured but Verified 79.6 / GPT-5.5 58.6 Pro / Kimi K2.6 58.6 Pro / DeepSeek V4-Pro presumed mid-50s. METR's 2026 follow-up to the 2025 productivity study swung from -19% to -18% (faster) but with CI -38 to +9: self-reported speedup data is empirically noisy — ELAI must measure wall-clock task-completion on a private bench, not trust the Verified number. Pricing-wise: Haiku 4.5 $1/$5, Sonnet 4.6 $3/$15, Opus 4.7 $5/$25 per MTok (benchlm.ai April 2026); DeepSeek V4-Flash ~$0.14/MTok input — ~21x cheaper than Sonnet input for cheap-volume escape hatch.

Cited grounding: marc0.dev May 2026 leaderboard, morphllm.com/swe-bench-pro + claude-benchmarks, arxiv:2409.12186 (Qwen2.5-Coder tech report: 7B head_dim 128 28Q/4KV; 32B head_dim 128 40Q/8KV 64 layers), qwen.ai/blog?id=qwen3.6-27b (head_dim 256 hybrid), qwen.ai/blog?id=qwen3-coder-next (80B-A3B MoE — REJECTED), mistral.ai/news/devstral (Devstral Small 2 24B 68.0 Verified — head_dim unverified, VERIFY), insiderllm.com/guides/best-local-coding-models-2026/, nathan.sapwell.net/posts/hauhaucs-abliteration-analysis/ (abliteration: TruthfulQA -3pp, coding nearly unchanged), metr.org/blog/2026-02-24-uplift-update/.

## Existing state (B/C/D coverage)

- B-37 (SHIPPED commit fd5c2e5): `RuntimeAdapter` trait + 11 backends (Sidecar, AnthropicApi, 7 CLI ports, vLLM, Ollama) + `PerRoleRuntime` + `RuntimeOrchestrator` + 4 SSB action types + `/runtime` CLI + `RuntimePanel`. E-31's cloud-escalation policy table EXTENDS the RuntimeOrchestrator routing rules; no trait changes.
- CLAUDE.md "Local LLM" block: model paths point to Qwen2.5-Coder-7B + 3B-abliterated. E-31 keeps these paths but adds a startup-time head_dim+FA2-compat assertion (loud failure > silent fallback per the report's "Qwen3 broke FA2 compat" finding).
- CLAUDE.md "Sidecar --model default" line at `python/elai_inference/server.py:2338` + `crates/cli/src/main.rs:2558`: 7B-abliterated path. E-31 adds a per-VRAM tier guard but does NOT change the default.
- Memory:project_vram_tier_plan ("ship per-VRAM configs (8/12/16/24/48GB)"): E-31 formalises 4 tiers (8/12/24/48) with grounded model picks; no new tier introduced.
- Memory:feedback_no_gemma4: aligns with the report's "head_dim 256 / hybrid attention" rejection — Gemma family ruled out for the same reason as Qwen3.
- Memory:project_qwen3b_baseline: 25K tokens 100% pass, NF4+FA2, 1.92GB model — used as the baseline anchor for the new tier-matrix doc.
- This E-N is NET-NEW STRATEGIC + DELTA on RuntimeOrchestrator routing + DELTA on CLAUDE.md model-decision block. No Rust crate is added.

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/31-coding-model-leaderboard.md (full)
- F:\.ELAI_workflow\CLAUDE.md "Local LLM" + "Remote sidecar (canonical)" + "Anti-Stale-Training" sections
- F:\.ELAI_workflow\.elai_cc\python\elai_inference\server.py lines around model.from_pretrained + the head_dim / config check site (grep for `head_dim` or `config.hidden_size`)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\sidecar.rs (RuntimeAdapter::Sidecar implementation; 240-380)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\mod.rs (RuntimeAdapter trait + variants list)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\decoding.rs (CloudGroundingClient / routing entry points)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\grounding_client.rs (cloud router)
- F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md (B-37 + memory tier plan grounding)
- workflow/papers/ (check for existing notes on Qwen2.5-Coder tech report; ingest arxiv:2409.12186 if not present)

## Tasks

1. Author `workflow/improvement-sweep-2026-05-19/E-plan/E-31-tier-matrix.md` (~120 LOC) carrying the canonical per-VRAM tier table: 8GB → Qwen2.5-Coder-7B-abliterated NF4 (current default); 12GB → same 7B at 25-32K ctx (current ELAI default); 24GB → Qwen2.5-Coder-32B-Instruct NF4 (head_dim 128, 64 layers, 40Q/8KV); 48GB → 1× 32B + cloud overflow OR 2× 32B multi-tenant. Each row carries: model path, params, head_dim, NF4+FA2 verified flag, SWE-bench Pro number (or "not measured — open question"), citation, abliteration cost. Mirror the table format from the source report Section "Per-VRAM tier recommendation".

2. Add a sidecar-boot architectural-invariant check at `python/elai_inference/server.py` (~70 LOC). On boot, inspect `model.config` and assert: (a) `head_dim == 128` (computed from `hidden_size / num_attention_heads`); (b) no hybrid attention markers (check for `linear_attention_layers` / `delta_net_layers` / `gated_delta_net` keys); (c) `num_key_value_heads` present (GQA). On failure: raise `SidecarBootError("model arch violates ELAI head_dim-128 dense + FA2 invariant: {details}")` — loud, not silent fallback. Cite arxiv:2409.12186 + qwen.ai/blog?id=qwen3.6-27b (the hybrid-attention rejection grounding).

3. Add `crates/providers/src/runtime/escalation_policy.rs` (~180 LOC NEW). A pure data + decision-table module: `EscalationPolicy::route(task_kind, ctx_len, prior_audit_failures) -> RuntimeChoice`. Default = local Sidecar; escalate to cloud only on SessionDag handoff signal OR explicit `--cloud` flag (per existing B-37 behavior). When cloud is chosen: Sonnet 4.6 is the DEFAULT route (best $/quality); Opus 4.7 is conditional on SWE-Pro-style markers (real-repo PR detection, multi-file refactor > 5 files, prior audit BLOCK with HIGH severity); DeepSeek V4-Flash (Anthropic-compat shim) is conditional on `--volume` flag or `cheap_volume` policy mode. Output one new SsbActionType `cloud_route_decision` with `{chosen_runtime, reason_code, est_cost_usd}`.

4. Wire the EscalationPolicy into `RuntimeOrchestrator::pick_runtime` (~40 LOC delta in `crates/providers/src/runtime/mod.rs` or wherever PerRoleRuntime lives — grep for `pick_runtime` / `RuntimeOrchestrator::route`). Behavior change: when role=fixer or auditor AND prior cycle returned BLOCK with severity HIGH AND task touches >5 files, route to Opus 4.7 not Sonnet 4.6 by default. Cite morphllm Pro 64.3 (Opus 4.7) vs Sonnet 4.6 not-yet-measured-Pro number — this is the empirically defensible "real-repo PR" escalation.

5. Author a private SWE-bench-Pro-style internal eval harness scaffold at `.elai_cc/benches/coding_lane/run_internal_pro.py` (~150 LOC). Pulls 10-15 ELAI plan-execution traces from `workflow/sessions/` (SHIPPED B-Ns are the gold set), runs each task end-to-end against (a) 7B local, (b) Sonnet 4.6, (c) Opus 4.7, records wall-clock-to-PASS + cost. Output: `benches/coding_lane/results-{date}.json`. THIS IS THE EVAL SCAFFOLD only — running it is a follow-up, not this E-N's deliverable.

6. Update CLAUDE.md "Local LLM" block with one line: a pointer to `workflow/improvement-sweep-2026-05-19/E-plan/E-31-tier-matrix.md` as the canonical source of truth for per-VRAM model picks. Do NOT inline the table — it grows; CLAUDE.md must stay token-bounded per token-discipline rule. Add one anti-stale-rejection line: "Qwen3 family (3-Coder-Next, 3.6-27B, 3.6-A3B) REJECTED for head_dim 256 + hybrid attention breaking FA2 + DoLa/steering invariants." Cite qwen.ai/blog?id=qwen3.6-27b + arxiv:2409.12186.

7. Add a `crates/contracts/tests/head_dim_invariant_contract.rs` (~90 LOC). Loads the configured sidecar model path's `config.json`, asserts head_dim 128 + GQA + no hybrid-attention markers. The test is a static guard — it does NOT load model weights (no VRAM cost in CI). Skipped if `ELAI_SKIP_SIDECAR_TESTS=1`.

8. Author `workflow/reasoning/E-31-decisions.md` (~120 LOC) capturing: (a) why dense head_dim-128 invariant is enforced (DoLa/steering correctness, FA2 path); (b) why MoE families are rejected for local (memory:feedback_no_gemma4 alignment); (c) why Sonnet 4.6 is the cloud default (~98% of Opus 4.7 Verified for 1/5 the price per benchlm + morphllm); (d) when Opus 4.7 triggers (SWE-Pro markers); (e) when DeepSeek V4-Flash triggers (cheap-volume opt-in); (f) explicit DEFERRED list: Devstral Small 2 (head_dim VERIFY pending), OpenHands LM 32B (fine-tune; same lineage so transferable but no SWE-Pro number yet), Qwen2.5-Coder-32B at 24GB tier (waiting on private SWE-Pro number per open question 3).

9. Add a `RuntimeOrchestrator` SSB row `head_dim_invariant_verified` emitted once at sidecar boot after task 2 check passes — surfaces the verification to the dashboard's RuntimePanel + AuditTimelineWindow.

10. Defer (E-31.B): Qwen2.5-Coder-32B 24GB tier private SWE-Pro measurement. Requires friend's-box VRAM (current measured tier is 12GB local + remote 3090 24GB). When that measurement lands, the tier-matrix file gets a numeric Pro column for the 24GB row.

## WIRING_CONTRACT

produces_symbols:
  - workflow/improvement-sweep-2026-05-19/E-plan/E-31-tier-matrix.md (canonical tier table file)
  - python/elai_inference/server.py::assert_head_dim_invariant (new fn)
  - python/elai_inference/server.py::SidecarBootError (new exception class if not present)
  - crates/providers/src/runtime/escalation_policy.rs::EscalationPolicy (new struct)
  - crates/providers/src/runtime/escalation_policy.rs::RuntimeChoice (new enum)
  - SsbActionType::RouteDecisionEvent (Pattern-2 43rd application: single-variant SsbActionType + 3-arm RouteDecisionEventKind discriminator covering CloudRouteChosen / TierModelSwapped / LocalRouteKept; canonical per SCHEMA_VERSION budget; see Theme-1 of workflow/sessions/s-night-2026-05-30/audit/MASTER-INDEX-NIGHT-2026-05-30.md)
  - SsbActionType::HeadDimInvariantVerified (boot-time sidecar invariant; Pattern-2 not applicable — single boot beacon, not a recurring event family)
  - benches/coding_lane/run_internal_pro.py (eval scaffold; not auto-run)
consumes_symbols:
  - crates/providers/src/runtime/mod.rs::RuntimeOrchestrator (existing B-37 surface; pick_runtime gets EscalationPolicy hook)
  - crates/orchestrator/src/grounding_client.rs::CloudGroundingClient (existing; receives RuntimeChoice from EscalationPolicy)
  - python/elai_inference/server.py boot path (head_dim check called pre-model-load)
reachable_from:
  - sidecar boot -> assert_head_dim_invariant -> head_dim_invariant_verified SSB
  - any role dispatch through RuntimeOrchestrator -> EscalationPolicy::route -> cloud_route_decision SSB
  - elai /runtime status -> RuntimePanel renders current EscalationPolicy table
stub_ban:
  - no silent fallback if head_dim != 128 (raise, do not warn)
  - no Qwen3 / Qwen3.6 / Qwen3-Coder-Next model path in any default config
  - no MoE model path in any default config (memory:feedback_no_gemma4 + CLAUDE.md anti-MoE for local)
  - no .unwrap() in EscalationPolicy production paths
  - no hard-coded "Sonnet 4.6 always" — must go through policy table
frontend_pair: crates/dashboard-v2/src/lib/components/RuntimePanel.svelte (extends with EscalationPolicy current-state row)
observable_effect:
  trigger: "elai /task --mode cloud 'refactor src/ across 8 files: rename TaskRunner to TaskRunnerV2'"
  expect_ssb: "cloud_route_decision with chosen_runtime=opus-4-7, reason_code=multi_file_refactor_5plus, est_cost_usd populated"
  expect_artifact: "audit timeline shows escalation event; RuntimePanel current-policy row reflects Opus 4.7 route"

## DETERMINISTIC_CHECK

```bash
# 1. Tier matrix file authored
test -f workflow/improvement-sweep-2026-05-19/E-plan/E-31-tier-matrix.md
grep -c "head_dim" workflow/improvement-sweep-2026-05-19/E-plan/E-31-tier-matrix.md
# Expected: >= 4 occurrences (one per tier row)

# 2. Sidecar head_dim invariant check present
rg -n "assert_head_dim_invariant|SidecarBootError|head_dim.*128" .elai_cc/python/elai_inference/server.py
# Expected: >= 3 hits

# 3. EscalationPolicy module exists
test -f .elai_cc/crates/providers/src/runtime/escalation_policy.rs
rg -n "EscalationPolicy|RuntimeChoice::Sonnet|RuntimeChoice::Opus|RuntimeChoice::DeepSeekFlash" .elai_cc/crates/providers/src/runtime/escalation_policy.rs
# Expected: >= 4 hits

# 4. Pattern-2 SsbActionType variant + 3-arm Kind discriminator + boot beacon registered
rg "RouteDecisionEvent|RouteDecisionEventKind|CloudRouteChosen|TierModelSwapped|LocalRouteKept|HeadDimInvariantVerified" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: >= 6 hits (1 SsbActionType + 3-arm Kind + boot beacon + as_sql_str entry)

# 5. CLAUDE.md updated with pointer + anti-stale-rejection line
grep -c "E-31-tier-matrix\|Qwen3 family.*REJECTED" F:/.ELAI_workflow/CLAUDE.md
# Expected: >= 2

# 6. head_dim invariant contract test exists
test -f .elai_cc/crates/contracts/tests/head_dim_invariant_contract.rs

# 7. End-to-end smoke: 8-file refactor escalates to Opus 4.7
elai /task --mode cloud "refactor src/ across 8 files: rename TaskRunner to TaskRunnerV2"
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT json_extract(payload,'$.chosen_runtime'), json_extract(payload,'$.reason_code') FROM ssb_events WHERE action_type='cloud_route_decision' ORDER BY id DESC LIMIT 1;"
# Expected: opus-4-7 | multi_file_refactor_5plus
```

## ACCEPTANCE

- [ ] `E-31-tier-matrix.md` lives at the canonical path with 4 VRAM rows + citations
- [ ] Sidecar refuses to boot when head_dim != 128 OR hybrid-attention markers present (loud failure)
- [ ] `EscalationPolicy::route` returns RuntimeChoice per the documented decision table (Sonnet default, Opus on SWE-Pro markers, DeepSeek V4-Flash on cheap-volume opt-in)
- [ ] `cloud_route_decision` SSB row emitted on every cloud route with `{chosen_runtime, reason_code, est_cost_usd}` populated
- [ ] `head_dim_invariant_verified` SSB row emitted once per sidecar boot
- [ ] CLAUDE.md "Local LLM" block points to E-31-tier-matrix.md + carries the Qwen3-family rejection line
- [ ] `head_dim_invariant_contract.rs` test passes against the configured 7B and (skipped on CI) 32B configs
- [ ] No new dependency added (this is a strategy + config + 1 Rust module + 1 Python invariant change)
- [ ] benches/coding_lane/ eval scaffold present; running it is a follow-up
- [ ] Decision doc at `workflow/reasoning/E-31-decisions.md` captures the rejection rationale + open questions Q1-Q5 from the source report

## Tests to add

- `.elai_cc/crates/contracts/tests/head_dim_invariant_contract.rs`: static config-json parse + assertion; skipped under `ELAI_SKIP_SIDECAR_TESTS`.
- `.elai_cc/crates/providers/tests/escalation_policy_route.rs`: 8 fixture (task_kind, ctx_len, prior_audit_failures) cases → assert RuntimeChoice matches the decision table; assert `cloud_route_decision` SSB shape.
- `.elai_cc/crates/contracts/tests/cloud_route_decision_contract.rs`: SSB row schema validity.
- `.elai_cc/python/tests/test_head_dim_invariant.py`: synthetic config dicts (Qwen2.5-Coder-7B → pass; Qwen3.6-27B → SidecarBootError; Gemma-3 → SidecarBootError); 3 cases.

## Files touched

- CREATE: `workflow/improvement-sweep-2026-05-19/E-plan/E-31-tier-matrix.md` (~120 LOC)
- CREATE: `.elai_cc/crates/providers/src/runtime/escalation_policy.rs` (~180 LOC)
- CREATE: `.elai_cc/crates/contracts/tests/head_dim_invariant_contract.rs` (~90 LOC)
- CREATE: `.elai_cc/crates/providers/tests/escalation_policy_route.rs` (~120 LOC)
- CREATE: `.elai_cc/benches/coding_lane/run_internal_pro.py` (~150 LOC, scaffold)
- CREATE: `workflow/reasoning/E-31-decisions.md` (~120 LOC)
- MODIFY: `.elai_cc/python/elai_inference/server.py` (+~70 LOC head_dim invariant check + SidecarBootError class)
- MODIFY: `.elai_cc/crates/providers/src/runtime/mod.rs` (+~40 LOC RuntimeOrchestrator wires EscalationPolicy)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (+~20 LOC: Pattern-2 43rd application — single-variant SsbActionType::RouteDecisionEvent + 3-arm RouteDecisionEventKind discriminator (CloudRouteChosen / TierModelSwapped / LocalRouteKept) + single HeadDimInvariantVerified boot beacon; canonical per SCHEMA_VERSION budget per Theme-1 of MASTER-INDEX-NIGHT-2026-05-30.md)
- MODIFY: `.elai_cc/crates/dashboard-v2/src/lib/components/RuntimePanel.svelte` (~30 LOC EscalationPolicy current-state row)
- MODIFY: `F:\.ELAI_workflow\CLAUDE.md` (+~3 LOC: pointer to tier-matrix + Qwen3-rejection line)

## Citations / Grounding

- marc0.dev May 2026 SWE-bench Verified leaderboard (cited in source) — GPT-5.5 88.7, Opus 4.7 87.6, Sonnet 4.6 79.6
- morphllm.com/swe-bench-pro (cited in source) — Pro is contamination-resistant; Claude Mythos 45.9 Pro vs 93.9 Verified gap
- morphllm.com/claude-benchmarks (cited in source) — Opus 4.7 64.3 Pro / 70 CursorBench
- benchlm.ai/blog/posts/claude-api-pricing (cited in source) — Haiku 4.5 $1/$5, Sonnet 4.6 $3/$15, Opus 4.7 $5/$25 per MTok
- arXiv:2409.12186 (cited in source) — Qwen2.5-Coder tech report: 7B head_dim 128 28Q/4KV; 32B head_dim 128 40Q/8KV 64 layers
- qwen.ai/blog?id=qwen3.6-27b (cited in source) — head_dim 256 hybrid Gated DeltaNet + Gated Attention; FA2 incompatible
- qwen.ai/blog?id=qwen3-coder-next (cited in source) — 80B-A3B MoE — REJECTED for ELAI per CLAUDE.md
- mistral.ai/news/devstral (cited in source) — Devstral Small 2 24B dense, 68.0 Verified; head_dim unverified (DEFERRED)
- insiderllm.com/guides/best-local-coding-models-2026/ (cited in source) — Qwen3.6-27B 77.2 Verified / 83.9 LiveCodeBench
- nathan.sapwell.net/posts/hauhaucs-abliteration-analysis/ (cited in source) — huihui abliteration: 100% refusal removal, TruthfulQA -3pp, coding nearly unchanged
- metr.org/blog/2026-02-24-uplift-update/ (cited in source) — -18% (faster) for returning devs; CI -38 to +9 (noise — measure wall-clock, not self-report)
- thamizhelango.medium.com/the-scaffold-problem-why-ai-coding-benchmarks-are-broken (cited in source) — scaffold-vs-model decomposition warning
- openhands.dev/blog/introducing-openhands-lm-32b (cited in source) — fine-tune of Qwen2.5-Coder-32B; 37.2 Verified on older scaffold (DEFERRED)
- memory:project_vram_tier_plan, memory:project_qwen3b_baseline, memory:feedback_no_gemma4 (local grounding)

## Open questions / risks

- Devstral Small 2 24B head_dim is UNVERIFIED in public material — must read Mistral's reference impl before adoption (report Open Question / source row VERIFY). If head_dim != 128 it's out; if 128 it becomes a real 24GB-tier alternative to Qwen2.5-Coder-32B.
- Qwen2.5-Coder-32B's SWE-bench Pro number does NOT exist publicly (predates Pro benchmark) — the 24GB-tier recommendation is grounded on dense + head_dim 128 + best-open-source-per-tech-report, not on a measured Pro score. Source report Open Question 3 calls this out; E-31.B captures the follow-up measurement.
- DeepSeek V4-Flash via "Anthropic-compat shim" assumes the shim exists or is easy to write — verify against current adk-anthropic / async-openai capabilities BEFORE adding the cheap-volume route. If shim cost > 1 day, defer DeepSeek route entirely; Sonnet/Opus is the proven path.
- Abliteration regression risk: huihui's abliterated 7B is the current default. Source report flags TruthfulQA -3pp and coding-bench-near-unchanged, but the in-tree code bench is the only authoritative ELAI signal — re-run on the 8 BabyStepSpec authoring tasks before promoting an abliterated 32B alternative.
- METR signal is noisy (CI -38 to +9). Internal eval harness (task 5) is the only way to defensibly claim ELAI uplift; do not over-claim on the bench's first run.
- Multi-runtime SSB cardinality: `cloud_route_decision` fires per-task — verify it doesn't drown the SSB on long sessions (rate-limit / dedupe by `(task_id, chosen_runtime)`).

## Next

- E-31.B (deferred): private SWE-bench-Pro-style measurement of Qwen2.5-Coder-32B + 7B on the 10-15 SHIPPED B-N gold set. Fills the empty Pro column in the 24GB tier row.
- E-31.C (deferred): Devstral Small 2 head_dim verification + smoke + adoption decision. If head_dim 128 + FA2 OK, ship as the 24GB-tier alternative.
- B-16 (PLANNED, anti-stale rule enforcement): E-31's tier-matrix file becomes one of the canonical inputs to B-16's CI lint pass — any PR pointing the sidecar at a non-tier-matrix model fails the lint.
- C-13 (PLANNED, IPC dispatcher production swap): the EscalationPolicy module is reachable through the IPC dispatcher; sequence C-13 after E-31 so EscalationPolicy is in place when C-13 lands.
- D-1 multi-role auditor (if extended later) will use the same EscalationPolicy to route audit role to Opus 4.7 on HIGH-severity tasks.
