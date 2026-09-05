# J-54: free-API rotation pool — a user-assignable-per-role FreePool backend that spreads work across many providers' free tiers, hand-rolled in Rust with a quota/cooldown/heal economics layer the cloud spine lacks

STATUS: PLANNED (WAVE 1; in-process at dispatch time + network on completion; BUILD-on-shipped — ports Liberty's rotation subsystem natively onto the B-37 trait, no litellm dependency)

Objective: ELAI has the provider PLUMBING to talk to free endpoints (J-30 BYOK custom_providers.toml openai/anthropic-compat on the B-37 factory) but NONE of the free-tier ECONOMICS that make free tiers usable — per-(key,model) daily quota persistence, 429 header-priority cooldown, whole-pool exhaustion backoff, keyless overflow, multi-key rotation. J-54 ships a FreePool RuntimeBackend kind: a hand-rolled Rust port (LIBERTY-CAPABILITIES-AND-ROTATION.md §2, interface map workflow/papers/liberty-rotation-interface.md §1-5) of Liberty's catalog/router/quota/self_heal/providers_extra, USER-ASSIGNABLE PER ROLE exactly like local/cloud/hybrid (B-37 PerRoleRuntime). Hard boundary it must NOT cross: the dynamic catalog fetch (catalog_sync.py) is ALREADY-REJECTED (I-51 static models.toml; OpenRouter dynamic registry rejected three times) — the free catalog ships as STATIC ROWS in models.toml, never auto-fetched. GD-1 do-NOT-re-implement: B-37 RuntimeAdapter trait + PerRoleRuntime + resolve_backend (compose, the FreePool is one more adapter); I-7 EscalationCascade (the pool-exhaustion spill target); C-5 CloudModeRouter cost caps (the per-key spend ceiling reuse); G-2 rra.rs seal + G-10 drift canary (the trust-marker hooks); I-51 models.toml (the catalog store).

Estimated effort: ~2 sessions (~10 h). In-process for quota/routing/heal (entirely local state); network only on the actual free-provider completion call (reqwest). No GPU.
Pre-requisites (re-grep per W-7): B-37 RuntimeAdapter trait + PerRoleRuntime (crates/providers/src/runtime/ — resolve_backend, the per-role assignment surface); I-7 EscalationCascade (local>cloud waterfall); C-5 CloudModeRouter hard cost ceiling (cloud_mode.rs); G-2 rra.rs seal (the RRA bundle); G-10 model-drift canary (the surface-never-swap doctrine); I-51 models.toml store (PLANNED — if not yet shipped, J-54 adds the free rows to the same file it canonicalizes).
Parallel-safe with: J-55 (disjoint files — modality routing vs inference backend), J-48/J-49 (local serving, disjoint from the cloud-free lane).
Source: steal-free-api-rotation-2026-06-20.md S1-S6; interface map workflow/papers/liberty-rotation-interface.md; LIBERTY-CAPABILITIES-AND-ROTATION.md §2. Doctrine: J-0_INDEX.md.

## GROUNDING
- GROUNDED: B-37 RuntimeAdapter + PerRoleRuntime + resolve_backend + RuntimeBackend enum (crates/providers/src/runtime/{mod,types}.rs — re-grep 2026-08-04, PRESENT). RuntimeBackend already has CerebrasFree + GroqFree arms (E-39) that create_runtime returns as NotImplemented (RUNPOD-PENDING). FreePool is a SEPARATE rotation-pool backend kind, not a rename of those arms.
- GROUNDED: E-39 free_tier_guard (crates/orchestrator/src/cost/free_tier_guard.rs — PRESENT, do not restate/regress).
- GROUNDED: K-24 margin lane exists; respect it, do not restate.
- GROUNDED: models.toml lives at .elai_cc/config/models.toml (not worktree root).
- GROUNDED: interface map workflow/papers/liberty-rotation-interface.md exists.
- GROUNDED: free_pool/ directory does NOT exist yet (produce-side symbols ABSENT — this plan creates them).
- RECON_NEEDED: verify I-51 models.toml store is shipped at execution time; if not, J-54 adds the free rows to the same file I-51 canonicalizes. Check: `rg "models.toml" crates/providers/src/ --type rust | head -5`.
- RECON_NEEDED: verify E-39 free_tier_guard does not already cover rotation-pool logic (it guards against free-tier cost leakage; it is NOT a rotation pool). Check: `rg "free_tier_guard" crates/ --type rust | head -10`.
- RECON_NEEDED: verify the RuntimeBackend enum add-site in types.rs has not changed since 2026-08-04 re-grep. Check: `rg "enum RuntimeBackend" crates/providers/src/runtime/types.rs`.

## Context
Why this matters: free-tier models are low-context and rate-limited individually, but their UNION over many providers' free tiers is a usable throughput budget (Liberty's "throughput over depth" thesis, ARCHITECTURE.md:5-9). The value is NOT the plumbing (J-30 already connects an endpoint) — it is the economics layer that keeps a task flowing while each turn may land on a different free key/model/provider: per-(KeyId,model) daily counters so an exhausted key is skipped, 429 cooldown parsed from the provider's own reset headers, a whole-pool sleep-until-soonest-reset, and a keyless last-resort so the pool is never fully empty. Correctness boundaries: (a) a raw key is NEVER persisted — only a blake3 fingerprint (CB-1, CLAUDE.md anti-stale mandates blake3 over Liberty's sha256 for this non-RFC fingerprint use); (b) day counters survive restart, per-minute counters + cooldowns do NOT (interface map I-3); (c) the uncensored group has NO cross-group fallback, not even to overflow (interface map I-6) — silently falling back to a safety-tuned model defeats the lane; (d) soonest_reset is the MINIMUM across reset hints, not the safe-pessimistic max (interface map I-9). Mechanism: a FreePool adapter holds a Vec<Deployment> per group; dispatch = quota-filter the deployments, simple-shuffle within the live group, on group exhaustion walk the fallback chain, on whole-pool exhaustion either spill to the configured cloud lane (I-7, if the role allows) or sleep to soonest_reset and retry.

## Existing state (B/C/D/E/F/G/H/I/J coverage)
- B-37 MULTI-RUNTIME BACKEND [WIRED]: RuntimeAdapter trait + 11 backends + PerRoleRuntime + resolve_backend + RuntimePanel (crates/providers/src/runtime/; feature-map §1.9). FreePool is backend kind #12-class; PLAN-asserted exact file:line, executor re-greps.
- J-30 PROVIDER BYOK [PLANNED]: custom_providers.toml openai/anthropic-compat dynamic backends on the B-37 factory; per-backend base-url. J-54's free providers are openai-compat endpoints — reuse J-30's base-url generalization for the wire, add ONLY the rotation/quota layer on top.
- I-7 CASCADING ESCALATION [WIRED]: local>cloud EscalationCascade (feature-map §1.9). The spill target when free-pool exhausts and the role permits cloud.
- C-5 CLOUD MODE cost caps [WIRED]: CloudModeRouter hard ceiling RouteBlocked/WarnLimitReached (cloud_mode.rs:284). Reused for the optional per-key/per-pool spend ceiling.
- G-2 RRA [WIRED] (rra.rs) + G-10 MODEL-DRIFT CANARY [WIRED-static]: the trust-marker hooks — RRA records the lane + non-reproducible flag; G-10 surface-never-swap is the suppress-with-notice pattern J-54 follows.
- I-51 MODEL CAPABILITY CATALOG [PLANNED]: one models.toml. The free-model rows live here (NOT a parallel registry; NOT a dynamic fetch — S7 already-rejected).
- E-31 tier matrix [PARTIAL] RouteDecisionEvent (3-arm): prefer-reuse considered; J-54 adds a dedicated FreePoolEvent because the observable surface (rotation, backoff, trust-downgrade) is distinct from cloud route decisions.

## Reads (mandatory before coding)
1. workflow/papers/liberty-rotation-interface.md (the whole file — the port spec; §3 quota + §4 self_heal + the CRITICAL INVARIANTS I-1..I-10 are load-bearing).
2. steal-free-api-rotation-2026-06-20.md (verdicts + CB-1/CB-2/CB-3).
3. crates/providers/src/runtime/ — the RuntimeAdapter trait + PerRoleRuntime + resolve_backend (the seam FreePool plugs into); cloud_mode.rs:284 (C-5 caps); the I-7 EscalationCascade site.
4. crates/orchestrator/src/rra.rs (the seal — where the non-reproducible flag lands) + the G-10 drift-canary surface.
5. CLAUDE.md hard rules: blake3-only (CB-1), no .unwrap in prod, no new deps without confirmation, anti-slopsquatting, W-2 kill-switch-not-enable-flag.

## Tasks (WAVE 1 — port the engine + wire the backend + the per-role trust-marker)

### UNIT 1: FreeModel catalog as data
1. FreeModel catalog as data (~60 LOC + models.toml rows): add the free-model rows (id/provider/group/context_window/max_output/rpm/tpm/rpd/tpd, interface map §1) to models.toml under a [free_pool] table + a DENYLISTED_MODEL_IDS set; a typed loader. Do NOT port catalog_sync (S7 rejected). NOT a hardcoded Rust list — data in models.toml (I-51).

EXPECTED_OBSERVATION: `rg "\\[free_pool\\]" .elai_cc/config/models.toml` returns >=1 match; `rg "DENYLISTED" crates/providers/src/runtime/free_pool/` returns the typed loader struct; cargo check passes with the new loader module.
ON_FAIL: models.toml parse error -> cause: typo in TOML key -> counter-move: fix the TOML syntax, re-run `cargo check -p providers`.
ON_FAIL: loader fails to compile -> cause: struct field type mismatch with TOML schema -> counter-move: align the struct with the TOML layout, re-check.

### UNIT 2: QuotaTracker
2. QuotaTracker (~180 LOC): Mutex<struct> with per-(key_str,model) day_reqs/day_tokens + per-minute counters + _cooldowns; KeyId{provider, fp=blake3(raw_key).to_hex()[:8]} (CB-1, raw key never stored — enforce at the type: from_raw_key consumes the key, returns only the fp); UTC-day roll (clears day+minute+cooldowns) / UTC-minute roll (clears minute only); persist day counters only to ~/.elai/free_pool/quota_state.json (atomic tmp+rename), discard on new UTC day; note_429 with header priority x-ratelimit-reset-requests > Retry-After > x-ratelimit-reset-tokens > 60s default, MINIMUM-wins (interface map §3); has_headroom (rpd/tpd + cooldown); soonest_reset (min active). NOT: persisting minute counters or cooldowns; storing a raw key.

EXPECTED_OBSERVATION: `cargo check -p providers` passes; `rg "blake3" crates/providers/src/runtime/free_pool/quota.rs` returns >=1; `rg -c "sha256|sha2|blake2" crates/providers/src/runtime/free_pool/quota.rs` returns 0; unit test j54_quota test_day_persists_minute_does_not passes.
ON_FAIL: blake3 not found -> cause: blake3 crate not in Cargo.toml -> counter-move: `rg "blake3" Cargo.toml` to confirm it is in-tree (it is), fix the use statement.
ON_FAIL: quota_state.json write fails -> cause: ~/.elai/free_pool/ does not exist -> counter-move: create the dir in the persist fn with `std::fs::create_dir_all`.

### UNIT 3: FreePool dispatcher
3. FreePool dispatcher (~200 LOC): build Vec<Deployment> per group from the catalog x _collect_keys(env GROQ_API_KEY/_1/_2..); simple-shuffle within group; the EXACT _FALLBACK_CHAIN (interface map §2, big>fast-small>gemini-flash>mistral>openrouter-free, uncensored EMPTY); overflow-pollinations suffix on all NON-uncensored groups; quota-filter before dispatch (drop no-headroom deployments); empty-after-filter => PoolExhausted. NOT: a cross-group fallback for uncensored; an overflow suffix for uncensored.

EXPECTED_OBSERVATION: `cargo check -p providers` passes; `rg "PoolExhausted" crates/providers/src/runtime/free_pool/dispatch.rs` returns >=1; contract test j54_uncensored_empty_fallback passes (uncensored group fallback list is empty, no overflow suffix).
ON_FAIL: no deployments built -> cause: no GROQ_API_KEY env vars set -> counter-move: the dispatcher must still return Pollinations overflow (invariant I-7 >=1 always reachable).
ON_FAIL: cross-group fallback leaks into uncensored -> cause: fallback chain not conditioned on group -> counter-move: condition the fallback chain walk on `group != Uncensored`, re-run contract test.

### UNIT 4: Keyless overflow
4. Keyless overflow (~40 LOC): Pollinations always-present deployment (openai-compat, api_key literal "none", https://text.pollinations.ai/openai, interface map §5). Drop g4f (CB-2, Python-only). Guarantee >=1 overflow deployment always reachable (invariant I-7).

EXPECTED_OBSERVATION: contract test j54_pollinations_overflow passes (>=1 deployment always present even with zero env keys); `rg "pollinations" crates/providers/src/runtime/free_pool/dispatch.rs` returns >=1.
ON_FAIL: Pollinations endpoint unreachable -> cause: network outage -> counter-move: the deployment is still BUILT (reachable_from is structural, not a live ping); the heal layer handles the network error at completion time.

### UNIT 5: Heal wrapper
5. Heal wrapper (~80 LOC): on PoolExhausted, if the role permits cloud spill -> I-7 EscalationCascade; else sleep to soonest_reset (bounded max_total_wait/max_retries), retry; ALL other errors propagate immediately (interface map §4). Transparent delegation for everything except the completion call.

EXPECTED_OBSERVATION: `rg "heal_or_spill" crates/providers/src/runtime/free_pool/heal.rs` returns >=1; `rg "EscalationCascade" crates/providers/src/runtime/free_pool/heal.rs` returns >=1; cargo check passes.
ON_FAIL: heal sleeps forever -> cause: max_total_wait not enforced -> counter-move: add a bounded retry counter + total-wait cap, re-check.

### UNIT 6: FreePool as a RuntimeBackend kind
6. FreePool as a RuntimeBackend kind (~120 LOC): implement RuntimeAdapter for FreePoolAdapter; register in the B-37 factory; resolve_backend returns it when a role is assigned kind=FreePool. The per-role assignment uses the EXISTING PerRoleRuntime config surface (the same place local/cloud/hybrid is chosen) — add the enum arm, do NOT build a parallel config.

EXPECTED_OBSERVATION: `rg "FreePool" crates/providers/src/runtime/types.rs` returns the RuntimeBackend enum arm; `rg "FreePoolAdapter" crates/providers/src/runtime/mod.rs` returns the factory registration; `python .claude/tools/production_caller_check.py --files crates/providers/src/runtime/free_pool/mod.rs` exits 0 (WIRED, not DARK).
ON_FAIL: production_caller_check reports DARK -> cause: resolve_backend does not reference FreePoolAdapter -> counter-move: add the match arm in resolve_backend for RuntimeBackend::FreePool, re-run the checker.
ON_FAIL: enum exhaustiveness error -> cause: RuntimeBackend is #[non_exhaustive] -> counter-move: add the arm in the same match, the enum is internal (not #[non_exhaustive] per B-37 internal dispatch).

### UNIT 7: Trust-marker
7. Trust-marker (~70 LOC): when resolve_backend returns FreePool for AgentRole::Executor or ::Auditor, stamp the RRA seal non_reproducible=true + emit FreePoolEvent::FreePoolTrustDowngrade + suppress-with-notice the G-10 drift canary for that run (surface a one-line notice, never silently swap). For all other roles, no downgrade.

EXPECTED_OBSERVATION: `rg "non_reproducible" crates/providers/src/runtime/free_pool/mod.rs` returns >=1; `rg "FreePoolTrustDowngrade" crates/providers/src/runtime/free_pool/mod.rs` returns >=1; contract test j54_executor_downgrade passes (executor on free-pool => non_reproducible=true + FreePoolTrustDowngrade emitted).
ON_FAIL: trust downgrade not emitted for executor -> cause: role check missing or wrong -> counter-move: verify `matches!(role, AgentRole::Executor | AgentRole::Auditor)` in the trust-marker path, re-run contract test.

### UNIT 8: SSB FreePoolEvent
8. SSB FreePoolEvent (~50 LOC): Pattern-2 single-variant, 3-arm FreePoolEventKind { ModelRotated{group,model_fp,key_fp}, PoolExhaustedBackoff{soonest_reset_ms}, FreePoolTrustDowngrade{role} }; the 9-tuple golden-lock lockstep (action_type_lock.rs); payload carries NO key material (only fingerprints).

EXPECTED_OBSERVATION: `rg "FreePoolEvent" crates/orchestrator/src/ssb.rs` returns >=1; `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-54_free_api_rotation_pool.md` exits 0; SSB SCHEMA_VERSION incremented by 1.
ON_FAIL: golden-lock mismatch -> cause: 9-tuple not in lockstep with action_type_lock.rs -> counter-move: copy the exact tuple shape from an existing 3-arm event, re-run wiring_contract_check.

### UNIT 9: Decision doc
9. Decision doc: workflow/reasoning/J-54-decisions.md.

EXPECTED_OBSERVATION: `test -f workflow/reasoning/J-54-decisions.md` returns 0.
ON_FAIL: directory missing -> cause: workflow/reasoning/ not created -> counter-move: `mkdir -p workflow/reasoning/` then write the file.

## WIRING_CONTRACT
produces_symbols:
  - crates/providers/src/runtime/free_pool/mod.rs::FreePoolAdapter (the RuntimeAdapter impl, backend kind)
  - crates/providers/src/runtime/free_pool/quota.rs::QuotaTracker (KeyId blake3 fp, day-persisted counters, note_429, has_headroom, soonest_reset)
  - crates/providers/src/runtime/free_pool/dispatch.rs::FreePoolDispatcher (per-group deployments, simple-shuffle, fallback chain, quota filter, keyless overflow)
  - crates/providers/src/runtime/free_pool/heal.rs::heal_or_spill (pool-exhaustion backoff/spill)
  - models.toml [free_pool] rows + DENYLISTED set (catalog as data)
  - crates/orchestrator/src/ssb.rs::SsbActionType::FreePoolEvent (3-arm)
consumes_symbols:
  - crates/providers/src/runtime/*::RuntimeAdapter + PerRoleRuntime + resolve_backend (B-37 — the trait FreePool implements + the per-role assignment surface; do NOT re-implement)
  - crates/orchestrator/src/cloud_mode.rs::CloudModeRouter cost ceiling (C-5 — per-key spend cap)
  - the I-7 EscalationCascade site (pool-exhaustion cloud spill)
  - crates/orchestrator/src/rra.rs seal (G-2 — non_reproducible flag) + the G-10 drift-canary surface (suppress-with-notice)
  - models.toml loader (I-51 — the free catalog store)
reachable_from:
  - crates/providers/src/runtime/...::resolve_backend (PRODUCTION per-role backend resolution — returns FreePoolAdapter when a role is assigned kind=FreePool; a real completion on that role goes through it)
  - the CLI/config surface that assigns a role's backend (the same entrypoint that sets local/cloud/hybrid today)
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO storing a raw API key anywhere (KeyId keeps only the blake3 fp; from_raw_key consumes the key)
  - NO sha256/sha2/blake2 for the fingerprint (blake3 only, CB-1)
  - NO dynamic catalog fetch / OpenRouter live registry / cheahjs sync (S7 already-rejected; catalog is models.toml data)
  - NO cross-group fallback for the uncensored group; NO overflow suffix for uncensored (invariant I-6)
  - NO silent model swap on the executor/auditor lane without the FreePoolTrustDowngrade stamp + G-10 notice
  - NO .unwrap()/.expect() on the dispatch/quota/heal prod paths
  - NO persisting per-minute counters or cooldowns (interface map I-3)
  - NO new crate without user confirmation (reqwest/blake3/serde_json/tokio already in-tree)
frontend_pair:
  RuntimePanel gains a free-pool assignment control + a per-run "non-reproducible (free-pool)" badge when a verification role is on the pool; Tauri read of the role->backend assignment + the FreePoolEvent stream. (If RuntimePanel role-assignment UI is not yet present, the W-6 item is the badge + the assignment row; justified minimal surface.)
observable_effect:
  trigger: "assign a non-verification role to the free pool and drive one task: `elai run --role-backend draft=free-pool \"<task>\"` (or the config-file equivalent), with GROQ_API_KEY set"
  expect_ssb: "a FreePoolEvent::ModelRotated row with a group + model_fp + key_fp (fingerprints only, no key material); on a forced-exhaustion fixture, a PoolExhaustedBackoff row with soonest_reset_ms"
  expect_artifact: "the draft turn returns a completion served by a free-pool model; ~/.elai/free_pool/quota_state.json gains a day-counter row for the (key_fp,model). NEGATIVE: assigning the pool to executor MUST produce a FreePoolTrustDowngrade row + a non_reproducible=true RRA seal + the G-10 notice; an executor run on the pool that seals reproducible=true is a FAIL."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-54_free_api_rotation_pool.md --run-trigger
python .claude/tools/production_caller_check.py --files crates/providers/src/runtime/free_pool/mod.rs   # FreePoolAdapter WIRED (resolve_backend), not DARK
rg -n "blake3" crates/providers/src/runtime/free_pool/quota.rs    # KeyId fp uses blake3
rg -c "sha256|sha2|blake2" crates/providers/src/runtime/free_pool/quota.rs    # MUST be 0 (CB-1)
rg -c "catalog_sync|openrouter.ai/api/v1/models|cheahjs" crates/providers/src/runtime/free_pool/    # MUST be 0 (S7 rejected)
rg -n "non_reproducible|FreePoolTrustDowngrade" crates/providers/src/runtime/free_pool/    # trust-marker present
test -f workflow/reasoning/J-54-decisions.md
```

## ACCEPTANCE
- [ ] W-1: free-pool drives a REAL completion via resolve_backend; off-agent proof = the FreePoolEvent SSB row (not stdout).
- [ ] W-2: ships ON when a role is assigned to it; the kill-switch is the role assignment itself (no default-OFF ELAI_*_ENABLE — assignment IS the gate). No wire-dark flag.
- [ ] R7: the production caller is resolve_backend, not a test.
- [ ] CB-1: KeyId fingerprint is blake3; rg for sha256/sha2/blake2 in quota.rs returns 0.
- [ ] CB-3 / S7: no dynamic catalog fetch; the free catalog is models.toml data.
- [ ] invariant I-6: uncensored group has empty fallback + no overflow suffix (contract-tested).
- [ ] invariant I-2: a raw key never appears in quota_state.json or any log (contract-tested with a known key string).
- [ ] trust guardrail: free-pool on executor/auditor => non_reproducible RRA + FreePoolTrustDowngrade + G-10 notice (contract-tested negative arm).
- [ ] cargo check --workspace GREEN; auditor PASS on -p providers -p orchestrator only.
- [ ] No new dependency (reqwest/blake3/serde_json/tokio in-tree); or the ONE new dep, anti-slopsquat-verified + user-confirmed.

## WIRING DONE-GATE (clause by clause)
- W-1: real entrypoint = resolve_backend -> FreePoolAdapter.complete; artifact = FreePoolEvent row. Met.
- W-2: assignment-gated, ships ON when assigned; no default-OFF flag. Met (assignment is the switch).
- W-3: no opt-in flag on the gate. Met.
- W-4: measures the objective (a free model served the turn + the day-counter moved), not stdout. Met.
- W-5: production_caller_check.py over the new pub symbols; resolve_backend is the non-test caller. Met.
- W-6: RuntimePanel free-pool assignment control + non-reproducible badge. Met.
- W-7: every B-37/C-5/I-7/G-2/G-10/I-51 anchor re-grepped at authoring + validation.
- W-8: gate files frozen; this plan does not edit the checker.
- W-9: SCHEMA baseline re-read at edit (FreePoolEvent = baseline+1).
- W-10: in-process quota/routing; network only on completion; tier noted.
- GD-1: composes B-37/C-5/I-7/G-2/G-10/I-51, re-implements none.
- GD-5 honest residue: free-tier provider rate limits + uptime are external; the pool degrades to keyless overflow but a total provider outage is surfaced, not masked. blake3-fp collision risk at 8 hex chars is acknowledged (32 bits; per-provider namespace makes practical collision negligible but it is a fingerprint not a key id — documented).

## Tests to add
- crates/contracts/tests/j54_free_pool_contract.rs (~140 LOC): (a) KeyId never stores raw (a known key string absent from fp + persisted json); (b) DENYLISTED ids absent from the built deployment list; (c) uncensored group fallback list empty + no overflow suffix; (d) note_429 takes the soonest header (x-ratelimit-reset-requests=30 + Retry-After=90 => ~30s cooldown); (e) Pollinations overflow always >=1 deployment; (f) day counters persist across a simulated restart, minute counters + cooldowns do NOT; (g) NEGATIVE/confinement: free-pool on executor stamps non_reproducible + emits FreePoolTrustDowngrade.

## Files touched
- CREATE: crates/providers/src/runtime/free_pool/{mod,quota,dispatch,heal}.rs (~620 LOC total)
- MODIFY: crates/providers/src/runtime/ (factory + PerRoleRuntime enum arm + resolve_backend) (~60 LOC)
- MODIFY: models.toml (free_pool rows + denylist) (~40 LOC)
- MODIFY: crates/orchestrator/src/ssb.rs (FreePoolEvent 3-arm + golden-lock lockstep) (~60 LOC)
- MODIFY: crates/orchestrator/src/rra.rs (non_reproducible flag on free-pool verification lane) (~25 LOC)
- MODIFY: dashboard-v2 RuntimePanel (assignment control + non-reproducible badge) + Tauri read (~80 LOC)
- CREATE: crates/contracts/tests/j54_free_pool_contract.rs (~140 LOC)
- CREATE: workflow/reasoning/J-54-decisions.md (~40 LOC)

## FORK
FORKS: none. The plan has a single linear path: catalog data -> quota tracker -> dispatcher -> overflow -> heal -> backend registration -> trust marker -> SSB event -> decision doc. Every decision is already resolved in the Open questions section below. No judgment calls are left to the executor.

## ABORT
ABORT_CONDITIONS:
- B-37 RuntimeAdapter trait or resolve_backend not found at execution time (re-grep fails) -> STOP, the seam is gone, re-plan.
- E-39 free_tier_guard already implements rotation-pool logic (re-grep shows a FreePool-equivalent) -> STOP, J-54 is redundant, escalate.
- blake3 crate not in Cargo.toml and user does not confirm adding it -> STOP, CB-1 cannot be satisfied without the dep.
- I-51 models.toml store rejected or removed -> STOP, the catalog store is gone, re-plan the catalog as data.
- RuntimeBackend enum is #[non_exhaustive] and cannot be extended -> STOP, the B-37 seam is closed, escalate.

## VERIFY
VERIFICATION runs (auditor-owned, off-agent artifacts):
1. `python .claude/tools/production_caller_check.py --files crates/providers/src/runtime/free_pool/mod.rs` -> exit 0 (FreePoolAdapter WIRED via resolve_backend, not DARK). PASS = exit 0.
2. `python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-54_free_api_rotation_pool.md --run-trigger` -> exit 0. PASS = exit 0.
3. `rg -c "sha256|sha2|blake2" crates/providers/src/runtime/free_pool/quota.rs` -> 0 (CB-1). PASS = count 0.
4. `rg -c "catalog_sync|openrouter.ai/api/v1/models|cheahjs" crates/providers/src/runtime/free_pool/` -> 0 (S7 rejected). PASS = count 0.
5. `cargo check --workspace` -> GREEN. PASS = exit 0.
6. Auditor drives `cargo test -p providers -p orchestrator j54` -> contract tests green. PASS = all j54 tests pass (off-agent NUnit-equivalent artifact, not executor stdout).
7. `rg "non_reproducible|FreePoolTrustDowngrade" crates/providers/src/runtime/free_pool/` -> >=1 match each. PASS = both present.
8. `test -f workflow/reasoning/J-54-decisions.md` -> exit 0. PASS = file exists.

## REDTEAM
REDTEAM attack block:
- Attack 1 (wire-dark producer): executor implements FreePoolAdapter + QuotaTracker + dispatcher but does NOT wire resolve_backend to return FreePoolAdapter. Result: production_caller_check.py reports DARK, unit tests pass (they call the adapter directly). FAILED against this plan: VERIFY step 1 catches it (DARK = BLOCK). Patch: the plan's UNIT 6 EXPECTED_OBSERVATION explicitly names resolve_backend as the production caller, and WIRING_CONTRACT reachable_from names it too.
- Attack 2 (raw key leak): executor stores the raw API key in quota_state.json "for debugging". Result: a known key string appears in the persisted file. FAILED against this plan: contract test (a) checks a known key string is absent from fp + persisted json; invariant I-2 is contract-tested. Patch: KeyId::from_raw_key consumes the key and returns only the blake3 fp; the type enforces it.
- Attack 3 (silent model swap on executor): executor wires FreePool to executor role without the trust-marker. Result: an executor run on the pool seals reproducible=true. FAILED against this plan: contract test (g) checks non_reproducible + FreePoolTrustDowngrade; ACCEPTANCE trust guardrail is contract-tested negative arm. Patch: UNIT 7 names the exact role check (Executor | Auditor) and the RRA + G-10 stamps.
- Attack 4 (cross-group fallback for uncensored): executor applies the fallback chain to the uncensored group. Result: uncensored silently falls back to a safety-tuned model. FAILED against this plan: contract test (c) checks uncensored fallback list is empty + no overflow suffix; invariant I-6. Patch: UNIT 3 ON_FAIL for this case conditions the fallback chain on group != Uncensored.
- Attack 5 (sha256 fingerprint): executor uses sha256 instead of blake3 for the KeyId fingerprint. Result: CB-1 violation. FAILED against this plan: VERIFY step 3 greps for sha256/sha2/blake2 and expects 0. Patch: the plan mandates blake3 in UNIT 2 + WIRING_CONTRACT stub_ban + VERIFY step 3.

## INTEGRATION_SMOKE
INTEGRATION_SMOKE units:
- After UNITS 1-6 (catalog + quota + dispatch + overflow + heal + backend registration): cross-seam smoke driving the REAL entrypoint `elai run --role-backend draft=free-pool "<task>"` with GROQ_API_KEY set. HARD input: a multi-turn task long enough to trigger at least one model rotation (set a low rpd quota in a test fixture to force rotation within the run). Assert: FreePoolEvent::ModelRotated SSB row lands (off-agent artifact), quota_state.json gains a day-counter row, the completion returns. This is NOT a toy single-turn happy-path; the rotation is the secondary path.
- After UNITS 7-8 (trust-marker + SSB): cross-seam smoke assigning executor role to free-pool. HARD input: `elai run --role-backend executor=free-pool "<task>"`. Assert: FreePoolEvent::FreePoolTrustDowngrade SSB row lands, RRA seal non_reproducible=true, G-10 notice surfaces. NEGATIVE arm: an executor run on the pool that seals reproducible=true is a FAIL.
- Before DONE: full integration smoke driving the REAL entrypoint on a forced-exhaustion fixture (all keys quota-exhausted). Assert: FreePoolEvent::PoolExhaustedBackoff SSB row lands with soonest_reset_ms, the pool either spills to cloud (I-7) if the role permits, or sleeps to soonest_reset and retries. This exercises the heal layer end-to-end, not just the happy path.

## Citations / Grounding
- interface map workflow/papers/liberty-rotation-interface.md (reviewed-source, MEDIUM): catalog §1, router/fallback §2, quota+429-priority §3, self_heal §4, overflow §5, invariants I-1..I-10. LIBERTY-CAPABILITIES-AND-ROTATION.md §2 (cross-model continuity) + §6 (the keep-light steal principle).
- ELAI ground truth (PLAN-asserted, executor re-greps per W-7): B-37 RuntimeAdapter+PerRoleRuntime+resolve_backend (crates/providers/src/runtime/); C-5 CloudModeRouter cost ceiling (cloud_mode.rs:284); I-7 EscalationCascade; G-2 rra.rs; G-10 drift canary; I-51 models.toml.
- CLAUDE.md hard rules: blake3-only (CB-1 — this OVERRIDES Liberty's sha256), no .unwrap in prod, no new deps without confirmation, W-2 assignment-gated.
- LIVE SCHEMA baseline 268/577 as of 2026-06-15 (J-37 re-grep; ssb.rs:1508 SCHEMA_VERSION = -309 + SSB_ACTION_TYPES.len()); FreePoolEvent = baseline+1; re-read at edit (R-c/W-9).

## Open questions / risks
- per-key spend ceiling: free models are $0 so C-5's USD cap is a no-op; the real ceiling is rpd/tpd quota (in-engine). Decide at authoring whether to also surface a per-pool request-budget in the config (lean: yes, reuse the rpd accounting; cheap).
- uncensored lane: ship it at all? It exists in Liberty's catalog. Lean: include the group structure + the empty-fallback invariant (the safety property is the point), but leave the uncensored rows commented in models.toml so the user opts in by uncommenting (no default uncensored routing). Decide at authoring.
- env-var multi-key convention (GROQ_API_KEY_1/_2): port verbatim (interface map §2) or generalize into custom_providers.toml (J-30)? Lean: port the env convention for the known free providers, allow J-30 custom endpoints for the rest. Mitigation: the two are additive.
- blake3 8-hex-char fingerprint collision: 32 bits, per-provider namespace. Acceptable for a quota key (a collision over-counts one key, never leaks a key); documented in GD-5 residue.

## Next
J-54 closes the free-tier ECONOMICS whitespace: ELAI becomes assignable onto free models per role without losing the trust spine (verification lanes carry the non-reproducible stamp). It pairs with J-55 (the modality router decides HOW a free/local model acts; J-54 decides WHICH free model serves). After both, "best harness for local/free models" is a measured claim, not an aspiration: J-54 supplies the cheap throughput, J-55 supplies the token-efficient agentic surface, and the trust-marker keeps the wedge honest.

End of J-54.
