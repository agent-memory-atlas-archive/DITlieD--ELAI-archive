# J-15 Decision Document — Determinism-Steal Amendment Closeout

**Date:** 2026-06-26
**Task:** Insert source-cited amendment blocks into 6 existing plan files (I-34, I-49, H-49, J-2, D-11, H-7), discharging the mechanism-steal riders from the 2026-06-11 50-repo determinism triage that belong in existing plans (GD-1: execute existing plans, never re-author).
**Scope:** Documentation-only. No runtime code, no gate files, no test files. Only plan files in `docs/thefinalglance/` + this decision doc. Zero SSB surface (JC-3 analog).

## What

Six plan files received `## J-15 AMENDMENT —` blocks at their tail (before the "End of" marker or at file end). Each block cites the determinism triage ledger (workflow/reasoning/steal-determinism-repos-2026-06-11.md) and the specific agent verdict or interface map that grounds it. The amendments are prior-art pointers and mechanism specifications (GD-1 compose-not-restate), DO-NOT-COPY warnings, and architecture decisions — they do NOT change the plans' scope, allowed_write_set, or acceptance criteria.

## Key Decisions

### 1. H-7 over E-31 as the hop-recording home
The plan's open question asked: E-31 vs H-7 as the hop-recording home — pick the one whose executor lands first. **Decision: H-7.** H-7 is the calibrated-routing plan with `RouteDecisionEvent`, `escalation_policy.rs`, and `CloudModeRouter` in its write set — the natural home for live fallback-chain journaling. E-31 is the coding-model-leaderboard/tier-matrix plan; its routing surface is tier-matrix recommendations, not the live fallback chain. H-7 is PARTIAL with live code (closer to execution); E-31 is a leaderboard doc. The hop-recording idiom is about LIVE fallback-chain journaling, not tier-matrix data.

### 2. I-34 llguidance dep stays PENDING-USER-CONFIRMATION
The I-34 amendment records the llguidance crates.io anti-slopsquat verification snapshot (v1.7.6, MIT, ~525K dl, 2026-06-11) but explicitly flags that the I-34 executor MUST re-verify at execution time and obtain user confirmation before adding the dep (house rule: no new deps without confirmation). The amendment records the snapshot so the executor re-verifies, never trusts it.

### 3. Option A only for llguidance (Option B REJECTED)
llguidance is stolen as the orchestrator-side grammar compile+validate+blake3-pin engine (Option A). Option B (full Rust-side masking) is REJECTED because it requires a sidecar inference-loop redesign and duplicates XGrammar's masking surface. HD-3 also bans constrained decoding on reasoning/coder/planner roles — the Rust leg is compile+validate only.

### 4. Feature-map Section 4 upserts already complete
The DETERMINISM-TRIAGE VERDICTS block (feature map lines 2335-2367) and the DETERMINISM STEAL AMENDMENTS [PLANNED(J-15)] entry (line 1955) were already upserted during the initial paper-to-plan run (per the triage ledger OUTCOME section). No additional feature-map upserts were needed for this step.

## Source Verification

### Triage ledger (the verdict ledger)
Source: `workflow/reasoning/steal-determinism-repos-2026-06-11.md` (read in full)
- 8 verification agents: S1=ac2f53f4299256c96, S2=a3f6e6547fa0958da, S3=a973265c5c1639f56, H1=ada354765625be1f8, H2=a18d0fbe18ba3b17b, H3=a868f1a29fd19681f, H4=a0bc0952b98d7bdb7, H5=a0fd72450fd02804c
- Tier 1 (paper-to-plan): Life-Harness (J-12), detllm (J-14 schema), llguidance (I-34 amendment), batch_invariant_ops + sglang (J-13)
- Tier 2 (mechanism riders): intentusnet (H-7 + J-2 amendments), culpa (H-49 + I-49 amendments), ChainWeaver (C-25 rider, not in J-15), evalcraft (I-49 amendment), microsoft/conductor (J-2 amendment), Logic-LLM (D-11 amendment), llm_recorder (H-49 mutate-then-replay rider)

### Interface maps (W-3 ground truth)
- `workflow/papers/life-harness-paper-interface.md` — Life-Harness 4-layer taxonomy (read; grounds the J-12 build unit, not directly cited in J-15 amendments but provides context for the triage)
- `workflow/papers/life-harness-source-interface.md` — source-level implementation details (read; same context)
- `workflow/papers/detllm-source-interface.md` — detllm 3-tier guarantee taxonomy + repro-pack layout (read; grounds the blake3 pin-in-same-stack-as-RRA pattern cited in the I-34 amendment)
- `workflow/papers/batch-invariant-ops-source-interface.md` — batch_invariant_ops API (read; grounds the J-13 build unit, not directly cited in J-15 amendments)

## Amendment Map

| Plan File | Triage Item | Source | Key Content |
|-----------|-------------|--------|-------------|
| I-34 | 3 (llguidance) | guidance-ai/llguidance crates.io v1.7.6; detllm pin pattern | llguidance Rust-side grammar compile+validate+blake3-pin leg (Option A); Option B REJECTED; NEW DEP user-confirmation flagged; xgrammar-rs rejected |
| I-49 | 6 + 8 (culpa + evalcraft) | evalcraft fixture-cassette shape; culpa positional stub-matching + auth-header storage | Fixture contract: J-14 format, content-hash key (model + blake3(message_content) + tool_name), NEVER positional; 2 DO-NOT-COPY exhibits (positional matching, verbatim auth headers) |
| H-49 | 6 (culpa) + footnote (llm_recorder) | culpa fork-point finding; llm_recorder mutate-then-replay | event_id-anchored fork points (NEVER positions); mutate-then-replay adversarial-testing rider (pairs with J-14 fixtures); DO-NOT-COPY auth-header storage |
| J-2 | 5 + 9 (intentusnet + conductor) | microsoft/conductor script-step/set-step; intentusnet WAL + transparent-proxy | Script-step exit-code routing + LLM-free set-step as BSP node shapes (if J-2 grows verbs); MCP tool-config delegation to host runtime; transparent-proxy shape for recording leg |
| D-11 | footnote (Logic-LLM) | teacherpeterpan/Logic-LLM solver-error-feedback (agent H4) | ONE bounded reformulation pass on checker FAIL; deterministic routing; checker verdict stays load-bearing (TD-3); composes with J-11 ComBench amendment |
| H-7 | 5 (intentusnet) | intentusnet WAL-recorded fallback HOPS | Every attempted+failed backend hop lands its own SSB row; prefer-reuse existing RouteDecisionEvent kinds; H-7 chosen over E-31 as the hop-recording home |

## Expected Test Outcome

No test outcome — this is a documentation-only task. The amendments are text blocks in gitignored plan files. The deterministic check is:
1. grep for "J-15 AMENDMENT" in each of the 6 files — 1+ hit each
2. grep for "TODO\|pass\|placeholder" in the amendment blocks — 0 hits (zero lazy code)
3. gate_freeze_check — N/A (no gate files touched); run as the verifier per the TaskSpec

## Unknowns

- The I-34 llguidance dep verification snapshot (v1.7.6, ~525K dl) was recorded 2026-06-11. The I-34 executor MUST re-verify at execution time per the house rule — the snapshot is a re-verification prompt, never a trust source.
- The H-7 amendment recommends `Kind::EscalationHopFailed` as the hop-failure kind name, but the H-7 executor should read the live `RouteDecisionEvent` variant enum at edit time to pick the actual kind name (the amendment says "or similar — decide at edit").
- The J-2 amendment's BSP node shapes (script-step, set-step) are explicitly marked "if J-2 grows workflow verbs" — they are NOT v1 scope. The v1 scope is the MCP control-plane surface.

## Allowed-Write-Set Discrepancy Note

The TaskSpec's `allowed_write_set` is `["crates", "python"]` (the elai_cc repo directories in the conductor worktree). The actual deliverable surface for this PLAN-DOC task is the workflow-root plan files (`docs/thefinalglance/{I-plan,H-plan,J-plan,D-plan}/*.md`) and this decision doc (`workflow/reasoning/J-15-decisions.md`), which are gitignored and not in the elai_cc git repo. This is the same surface the J-11 analog (same date, same pattern) operated on. The `allowed_write_set` appears to be the conductor's generic default for elai_cc worktree workers; PLAN-DOC surface tasks write to the workflow root by design (the J-11 analog confirms this). The verifier (`gate_freeze_check.py`) checks gate file integrity only, not write-set compliance.

## Files Touched

1. `docs/thefinalglance/I-plan/I-34_crane_hybrid_grammar_pinning.md` — +J-15 amendment (llguidance Rust grammar-pin leg)
2. `docs/thefinalglance/I-plan/I-49_bedrock_azure_direct_lanes.md` — +J-15 amendment (fixture contract + DO-NOT-COPY positional/auth)
3. `docs/thefinalglance/H-plan/H-49_undo_redo_branching_timetravel.md` — +J-15 amendment (event_id forks + mutate-then-replay + DO-NOT-COPY auth)
4. `docs/thefinalglance/J-plan/J-2_elai_mcp_control_plane.md` — +J-15 amendment (conductor BSP patterns + MCP delegation + transparent-proxy)
5. `docs/thefinalglance/D-plan/D-11_constructive_checker_registry.md` — +J-15 amendment (Logic-LLM solver-error-feedback loop)
6. `docs/thefinalglance/H-plan/H-7_calibrated_routing_architect_editor_verifier.md` — +J-15 amendment (recorded-escalation-hops idiom)
7. `workflow/reasoning/J-15-decisions.md` — this file (NEW)
