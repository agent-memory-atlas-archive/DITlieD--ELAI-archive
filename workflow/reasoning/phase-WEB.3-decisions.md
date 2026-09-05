Phase WEB.3 — Web validator + SSB + nav wiring (decision doc)

What I will implement:
1. New crate `crates/web_validator` (no orchestrator dep) exposing start/stop/check dev server, run_web_build, promote_slot, plus typed error and tier enum.
2. SSB extension: SsbActionType::PreviewAvailable + PreviewAvailablePayload at all 7 sites in orchestrator/src/ssb.rs (enum, as_sql_str, from_str, CHECK SQL, payload struct, SsbPayload variant, action_type match, test_payload, all_test_payloads, count assertions, proptest strategy, test_all_action_types_insert array). Count goes 28 -> 29.
3. Navigator policy: `web_validation_tiers_for_level(DecompositionLevel) -> Vec<WebValidationTier>` in pipeline_tree_navigator.rs that maps BabyStep=[WebBuild], Step=[WebBuild,WebLint], Phase=[WebBuild,WebDevServer], Concept=[WebBuild,WebDevServer,WebLint].

Key technical decision: web_validator is a peer crate — orchestrator depends on it (downstream), never the reverse, preventing circular deps. SSB action_type emission is the orchestrator caller's responsibility, not the validator's.

Expected test outcome: cargo test -p elai-web-validator passes (~12 unit tests), cargo test -p elai_cc_contracts --test web_validator_contract passes (6 tests), cargo test -p elai-orchestrator passes with new SSB count=29 and PreviewAvailable round-trip. cargo check --workspace clean.

Unknowns to verify empirically:
- tokio::process::Child::start_kill behavior on Windows when child is `npm` (parent pid only — node child may persist; documented limitation).
- mklink /J fallback for Windows non-elevated users (verified via bench grounding by context-compiler).
- pre-existing 2 orchestrator warnings count after adding new variant arms.
