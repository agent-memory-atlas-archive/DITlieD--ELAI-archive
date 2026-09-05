# P-17: provider failover semantics + honest routing economics

STATUS: PLANNED (WAVE OC-1; EXTENSION of J-28 + I-50 + Q-4 + B-37; PROOF_SPINE: feeds PS-4; execute after the proof spine per A-7)

- Track OPENCLAW-STEAL | lever failover-semantics-package | grounding STRONG (openclaw failover is its most battle-tested subsystem, source-verified; J-28/I-50 PLAN-asserted, executor re-greps W-7) | nature EXTENSION | effort M | WAVE OC-1 | PROOF_SPINE feeds PS-4
- Premise: ELAI's rotation-pool spec is thinner than openclaw's shipped failover. Port the whole failure-semantics package into J's free-API rotation pool + I-50 auth broker, plus the never-merge billed-truth-with-estimate discipline. Honest failover is a precondition for the PS-4 $/verified-task number.

## Context — why this matters
Grounded in workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md 2.5 + 2.16 + steals S2/S4. openclaw's failover is its most battle-tested subsystem: ~140 provider plugins, auth profiles in per-agent SQLite, a failure-class cooldown taxonomy (rate-limit 30s->1m->5m; billing -> disabled lane 5h->24h; auth-permanent 10-60m), MODEL-scoped cooldowns so sibling models still try, session-stickiness (a profile pinned per session for cache warmth, rotating only on reset/compaction/cooldown), persist-override-BEFORE-retry to close the live-switch race, and STRICT explicit selection (a user's explicit model choice fails loudly, never silently substituted). Separately (2.16): billed truth and local estimates are NEVER merged. ELAI routes by measured value (ledger-trained, $/verified-task) where openclaw routes by availability — but ELAI's failover MECHANICS are thinner, and the PS-4 number is only honest if failover is.

## Existing ground truth (B..Q coverage; PLAN-asserted; executor re-greps W-7)
- J-28 token ledger (J-SUMMARY_INDEX.md:692): uniform usage_turn_event both lanes + session rollup + honest per-model pricing. EXTEND the UsageTurnEvent with typed billed/estimate fields (do NOT merge them).
- J free-API rotation pool (J-SUMMARY, token+provider economics): the failover consumer this hardens.
- I-50 subscription auth broker (I-SUMMARY_INDEX.md:963): the auth plane the failover lanes plug into.
- B-37 runtime adapters (per-role fallback chains): the fallback-chain consumers.
- Q-4 swap-restart attribution (Q-SUMMARY_INDEX.md): substitution is a recorded event, not a silent fallback — this matches strict-explicit-selection.
- K-2 routing-economics bench ($/verified-task; PS-4; live leg gated on J-28) (K-SUMMARY_INDEX.md:135): the number honest failover feeds.

## Steal/build (REVISED scope, W-10 tiered)
- SHIP NOW (Tier-1): (a) a failure-class cooldown taxonomy with separate billing/auth/rate-limit lanes, (b) MODEL-scoped vs profile-scoped cooldowns, (c) session-stickiness-for-cache-warmth as an explicit invariant (rotation is a cache-invalidation event, journaled as one — composes H cache-stable layout), (d) persist-override-BEFORE-retry, (e) strict-explicit-selection (a user's explicit model choice never silently substituted); PLUS never-merge billed-truth-with-estimate as typed end-to-end fields on the J-28 event.
- DEFER (Tier-2): the full ~140-plugin auth-profile store; the auto-override reprobe-origin-every-5min loop.
- EXCLUDED as vaporware: any silent substitution of a user's explicit model choice; a merged billed/estimate number on any surface.

## Reads (mandatory before coding)
- workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md (2.5, 2.16; steals S2, S4)
- docs/thefinalglance/J-plan/J-28_*.md (the ledger event), I-50 (auth broker), K-2 (the PS-4 bench), Q-4 (swap-restart)
- crates/providers/ (the provider dispatch + rotation path)

## WIRING_CONTRACT
produces_symbols:
  - crates/providers/src/failover.rs (FailoverPolicy: failure-class cooldown taxonomy + model/profile scope + stickiness + persist-before-retry)
  - crates/providers/src/strict_selection.rs (explicit-selection guard: a user-pinned model never silently substituted)
consumes_symbols:
  - crates/knowledge/src/usage/ledger.rs::UsageTurnEvent (J-28; EXTEND with typed billed + estimate fields, do NOT merge)
  - crates/providers/src/auth_broker.rs::AuthBroker (I-50; the auth plane)
reachable_from:
  - crates/providers/src/dispatch.rs (the production provider dispatch path invoking FailoverPolicy) [PRODUCTION provider dispatch]
  - a #[cfg(test)] or contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO billing failure that cools the WHOLE provider (only the billed lane cools; sibling models still try)
  - NO silent substitution of a user's explicit model choice (strict-explicit-selection; substitution is a Q-4 recorded event)
  - NO merged billed/estimate value (typed separate fields end-to-end; any surface shows which it shows)
  - NO .unwrap()/.expect() on the dispatch/failover path; NO blake2/sha2
frontend_pair:
  failover state + a billed-vs-estimate label on the usage surface (RuntimePanel / CloudPanel; W-4).
observable_effect:
  trigger: "elai provider failover-test --fixture tests/fixtures/providers/billing_then_ratelimit.json"
  expect_ssb: "provider_failover_event"
  expect_artifact: "a provider_failover_event row per event with the correct lane + cooldown; a sibling model on the same provider still tries after a billing failure; the usage surface shows billed vs estimate as distinct typed fields; NEGATIVE: a billing failure that cools the whole provider, a silent substitution of a user's explicit model, or a merged billed/estimate number, FAILS"

## Wargame (plan-standard unit shape)
EXPECTED_OBSERVATION: forcing a billing failure then a rate-limit on one provider lands one provider_failover_event per event with the correct lane + cooldown; a sibling model on the same provider still tries after the billing failure; billed and estimate appear as distinct typed fields on the usage surface.
ON_FAIL: a billing failure cools the whole provider, or billed/estimate render as one number (CAUSE: cooldown is provider-scoped not lane/model-scoped, or the ledger merges the two fields). COUNTER-MOVE: scope cooldowns by {provider, lane, model}; keep billed + estimate as separate typed fields; the negative test (whole-provider cool / merged number) must FAIL.
FORK: if a user has pinned an explicit model and it enters cooldown, DO NOT silently substitute — surface a strict failure (or a Q-4-recorded swap-restart if the user opted in); route only within the pinned model's own retry ladder.
RECON_NEEDED: confirm openclaw's exact cooldown ladders + the J-28 UsageTurnEvent struct fields (avoid a second usage event). Settling check: source-read openclaw provider cooldown code + `grep -n "struct UsageTurnEvent" crates/knowledge/src/usage/`.
ABORT: if failover would ever cross a user's explicit-model boundary silently, STOP and flag (strict-explicit-selection is the invariant; a silent cross is a trust break that also corrupts the PS-4 per-model attribution).
VERIFY: the load-bearing proof is the off-agent provider_failover_event rows with correct lanes + the separate typed billed/estimate fields, driven by the real dispatch path. Run:
```bash
python .claude/tools/plan_gate_check.py --file docs/thefinalglance/P-plan/P-17_provider_failover_economics.md   # authoring gate (exit 0)
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-17_provider_failover_economics.md   # execution done-gate
```
REDTEAM: surviving-attack seed — a provider that returns a billing error on every model is contained by the lane-scoped cooldown (only the billed lane cools; auth/rate-limit lanes independent) so a transient billing blip does not brick the whole provider. Attack that would succeed if unpatched: inflating $/verified-task by silently substituting a cheaper model for the user's pinned expensive one — PATCH: strict-explicit-selection forbids the substitution; per-model credit attribution stays clean (composes Q-4). Deep red-team DEFERRED to execution pre-flight.

## INTEGRATION_SMOKE
An independent agent drives the real provider dispatch on a HARD multi-turn session across a STRICT provider that 422s on a leaked field, a billing failure, and a rate-limit in one run, asserting: correct per-lane cooldowns, sibling-model survival, no silent substitution of a pinned model, and separate billed/estimate fields flowing into K-2's $/verified-task. Toy single-failure test does NOT satisfy R10.

## Grounding / citations
- Source: workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md 2.5/2.16, steals S2/S4 (openclaw failover source-verified there).
- ELAI ground truth (PLAN-asserted, executor re-greps W-7): J-28, I-50, B-37, Q-4, K-2 per the indexes cited.
- CLAUDE.md hard rules: no silent fallback (strict selection); tokio::sync::Mutex in async; no .unwrap on prod; blake3.

## PRODUCTION_HOOK
crates/providers/src/dispatch.rs — the production provider dispatch path invokes FailoverPolicy + the strict-selection guard on every provider call; the J-28 UsageTurnEvent gains typed billed/estimate fields consumed by K-2. Wired in this changeset.

## Next
Closes O1 (failover semantics — ELAI's thinner subsystem) and makes the PS-4 $/verified-task number honest (a failover that silently substitutes models corrupts the economics). Pairs with Q-4 (swap-restart) and K-2 (the bench).

End of P-17.
