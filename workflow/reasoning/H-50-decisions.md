H-50 regime-aware context management — decisions

WHAT
H-50 fixes ELAI's split-brain context policy: local (Qwen sidecar, 25-32K window, no cross-request KV cache) and cloud (Sonnet, 200K, caching live) need OPPOSITE strategies, but the code ran one cloud-shaped policy for both. Local sessions were effectively unprotected and died at a hard context_window_blocked error. H-50 = one effective_context_window truth + ContextPolicy-by-BackendMode, routing the already-shipped ladder/masking into the interactive LOCAL turn while leaving the cloud caching path intact.

KEY DECISIONS

D-1: ContextPolicy new module (orchestrator/src/context_policy.rs)
Keep as a standalone module with explicit pub-use re-exports from lib.rs. Embedding the policy in conversation.rs or runtime_builder.rs would make it harder to test independently and harder to extend (H-51 will read it for cross-session handoff policy).

D-2: LOCAL trim fraction = 0.75 (conservative)
Plan allowed 0.70-0.75. Chose 0.75 (more conservative, fewer false drops) because GD-4 false-drop rate calibration on pythondev is deferred. Kill-switch ELAI_CONTEXT_POLICY_DISABLE provides full rollback.

D-3: ContextReduction SSB variant REUSED (GD-1 no-bump)
Did not add WindowPolicyApplied. ContextReductionPayload.level=0 (FilteredByRelevance) distinguishes interactive local compaction from the autonomous PreCompact path. Schema stays 226/537.

D-4: LOCAL stub recovery = selective, not whole-set
recover_stubs (whole-set re-inflate) is skipped on LOCAL after a trim so the net token reduction is not zeroed out. Stubs stay as placeholders; recover_single_stub is the on-demand reload path. CLOUD keeps full recover_stubs (cache-stable prefix semantics unchanged).

D-5: WARN-1 kill-switch full revert
When ELAI_CONTEXT_POLICY_DISABLE=1, BOTH sites revert: conversation.rs uses LEGACY_FLAT_THRESHOLD (via trim_threshold() kill-switch path) and live_cli.rs uses max_tokens_for_model (legacy 64K non-opus). The disable path fully undoes H-50 at both sites.

D-6: env-mutating test serialization via static Mutex (no serial_test dep)
serial_test is not in dev-deps; adding deps requires user confirmation per CLAUDE.md. Used std::sync::Mutex::new(()) as a static gate instead. All 3 env-mutating tests acquire the lock before touching env vars; the lock is released when the guard drops at end of scope.

D-7: run_compaction_ladder in maybe_auto_compact uses NullEmitter
The async SSB telemetry path used by task_runner.rs (CollectEmitter) requires async context. maybe_auto_compact is sync. Used NullEmitter (ladder still runs and accounts budget tiers; SSB ladder-tier rows are autonomous-path telemetry, not interactive-path). The ContextReduction row (BLOCK-2) covers the interactive emit obligation.

EXPECTED TEST OUTCOMES
- cargo check --workspace --all-targets: exit 0 (warnings only, no errors)
- cargo test -p elai-orchestrator --test action_type_lock: 2 passed (226/537 locked)
- cargo test -p elai_cc_contracts --test h50_context_policy_contract: 13 passed
- cargo test -p elai-orchestrator --lib context_policy: 7 passed (including env-race serialization)

UNKNOWNS / DEFERRED (Tier-2, explicit)
- GD-4 false-drop rate calibration (>=3 runs on pythondev before tightening fraction from 0.75 toward 0.70)
- Observable_effect done-gate leg (c): control run hits context_window_blocked, enabled run completes. Requires live sidecar session on pythondev. Deferred per user policy (no local sidecar, hardware-heavy).
- PrefixKVCache unbypass (sidecar streaming path): Tier-2, behind its own kill-switch, not blocking Tier-1 ship.
