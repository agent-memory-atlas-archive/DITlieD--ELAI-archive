# J-57: prompts-as-code — eval-gated prompt-template promotion (test -> staging -> prod) + rollback-on-regression (make the shipped hash-chained versioning load-bearing)

STATUS: PLANNED (WAVE PV-1; effort M; EXTENSION — composes D-1 hash-chained templates + I-36 CompiledPromptStore + G-10 drift + J-56 eval gate, re-implements none; depends J-56)

Objective: the Five-Pillars Pillar-5 "treat prompts as CODE — every change goes test -> staging -> prod through a promotion process, eval-gated, with rollback when something breaks, and a governed commit message recording what failed and why" maps onto ELAI substrate that already STORES versioned prompts but never GATES their promotion: REASONING TEMPLATES (D-1 — ReasoningTemplate typed schema + blake3 hash-chain archive + template-migrate CLI, but select_template still dark in prod, feature-map ~236) and CROSS-MODEL PROMPT COMPILATION (I-36 — CompiledPromptStore msgpack+atomic, ~1081). J-57 ships the missing WORKFLOW: a PromptStage state machine (Test/Staging/Prod), a promotion gate that runs the J-56 eval corpus subset on a candidate template version and promotes ONLY on no-regression vs the current Prod version, and an automatic rollback to the last-good Prod hash when G-10 detects a behavioural regression. HARD BOUNDARY: J-57 does NOT author the hash-chain (D-1), the compiled store (I-36), the drift canary (G-10), or the eval corpus (J-56) — it is the promotion state machine + the eval-gate wiring + the rollback trigger only. the "abstract version control" is D-1; the EVAL-GATED PROMOTION is the net-new. GD-1 do-NOT-re-implement: D-1 ReasoningTemplate + hash-chain; I-36 CompiledPromptStore; G-10 drift canary; J-56 `elai eval run --scope`.

Estimated effort: ~1-2 sessions (~8 h). In-process at gate time (the stage machine + the promote/rollback decision are pure logic over recorded template hashes + a J-56 subset-run verdict); the eval subset the gate runs is itself in-process for the deterministic legs (the full ablation stays G-6 bench-tier, W-10).
Pre-requisites (re-grep at edit, W-7): J-56 `elai eval run --scope subset` SHIPPED (the gate it runs); D-1 ReasoningTemplate + blake3 hash-chain + template-migrate CLI SHIPPED; I-36 CompiledPromptStore SHIPPED (the prod-stage load path); G-10 MODEL-DRIFT CANARY SHIPPED (the regression signal).
Parallel-safe with: J-59 (disjoint files). Depends J-56; pairs with J-58 (both harden the generate/verify loop).
Source: workflow/reasoning/steal-production-ai-talks-2026-06-20.md (Five-Pillars Pillar-5 prompt-as-code + lesson-2 governed commit messages). Doctrine: J-0_INDEX.md.

## Context
Why this matters (external evidence, scope-tagged per HD-1): the Five-Pillars governance pillar treats a prompt change exactly like a code change — "test -> staging -> production, and when something breaks you need to know which version caused it"; lesson-2 adds that the COMMIT MESSAGE for a prompt change must record what failure caused it and what the next version corrects (generic git messages lose the trace). [DESIGN, vendor-framework — directional]. ELAI already has the version-control SUBSTRATE (D-1's blake3 hash-chain is a stronger record than a git message) but every template is effectively auto-promoted: there is no gate between authoring a new template version and it serving in prod, and no automatic demotion when it regresses. correctness boundary: the promotion decision is DETERMINISTIC (TD-3) — it is a J-56 subset-run verdict (pass/fail on deterministic gates), the LLM never adjudicates whether a prompt is "better"; rollback is triggered by the G-10 drift signal, which already pins identity and SURFACES-never-auto-swaps at the model layer — J-57 applies the same posture at the PROMPT layer but DOES auto-demote-to-last-good (a prompt rollback is reversible and local, unlike a model swap).

## Existing state (B/C/D/E/F/G/H/I/J coverage — W-7 anchors, nothing from memory)
- REASONING TEMPLATES [PARTIAL, D-1]: ReasoningTemplate typed schema + blake3 hash-chain archive + template-migrate CLI; select_template still DARK in prod (feature-map ~236-238). J-57's Prod stage is what makes select_template live (it serves the Prod-stage hash).
- CROSS-MODEL PROMPT COMPILATION [DARK(ELAI_PROMPT_COMPILE_DISABLE, default-ON), I-36]: CompiledPromptStore msgpack+atomic, keyed AgentRole+RuntimeBackend; substitution at build_system_prompt_for_role (feature-map ~1081-1086). The compiled prod variant is what a promoted template compiles INTO.
- MODEL-DRIFT CANARY [WIRED-static, G-10]: pin model+system_fingerprint+lane, re-run canary on identity change, SURFACE never auto-swap (feature-map ~625). J-57 reuses the canary signal as the prompt-regression trigger.
- J-56 LIVING EVAL CORPUS [PLANNED, this wave]: `elai eval run --scope subset` — the deterministic gate the promotion runs. PLAN-asserted; J-57 sequenced after J-56.

## Reads (mandatory before coding)
1. workflow/reasoning/steal-production-ai-talks-2026-06-20.md (Pillar-5 + lesson-2).
2. the D-1 detail file + ReasoningTemplate struct + the hash-chain archive + template-migrate CLI (re-grep the path).
3. I-36 CompiledPromptStore + build_system_prompt_for_role substitution; G-10 drift-canary emit site.
4. J-56_living_eval_corpus_governance.md (the `elai eval run --scope` contract this gate calls).
5. CLAUDE.md: TD-3 (deterministic promotion verdict), W-2 kill-switch, blake3 (template hashes), no-.unwrap-in-prod, named-constants.

## Tasks (WAVE PV-1 — the stage machine + the eval gate + the rollback)
1. PromptStage state machine (~80 LOC): enum PromptStage { Test, Staging, Prod }; each ReasoningTemplate version (a blake3 hash in the D-1 chain) carries a stage. The load path (select_template / build_system_prompt_for_role) serves the Prod-stage version; un-promoted versions are invisible to prod. NOT: a new versioning scheme (reuse the D-1 hash-chain); NOT: a free-text status.
2. Eval-gated promotion (~90 LOC): `elai prompt promote <template-id> --to staging|prod`. Promotion runs `elai eval run --scope subset` (J-56) over the categories the template affects, comparing the candidate version's pass/fail against the current Prod version's; promote ONLY on no-regression. A regressing candidate -> Blocked, Prod pointer UNCHANGED. NOT: an LLM deciding "this prompt is better" (TD-3); NOT: promoting Test straight to Prod (must pass through Staging).
3. Rollback-on-regression (~70 LOC): when G-10's drift canary fires on a turn whose prompt is a recently-promoted Prod version, auto-demote the template to the last-good Prod hash (reversible, local — the prompt-layer analog of G-10's model-layer surface; here the demotion is automatic because it is cheap and reversible) and emit the regression provenance. NOT: auto-swapping the MODEL (G-10's model posture stays surface-only); NOT: a silent demotion (the row records which version, which canary, which last-good hash).
4. Governed promotion provenance (~40 LOC): every promote/rollback records {from_hash, to_hash, eval_verdict | drift_signal, reason} (lesson-2 — what failed and why), in the D-1 chain. NOT: a generic message.
5. SSB PromptPromotionEvent (~50 LOC): Pattern-2, 4-arm PromptPromotionEventKind { Staged{template_id,to_hash}, Promoted{from,to,eval_pass}, BlockedByEval{candidate,regressed_categories}, RolledBack{from,to_lastgood,drift_signal} }; golden-lock lockstep.
6. Decision doc: workflow/reasoning/J-57-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - the PromptStage enum + per-version stage on the D-1 ReasoningTemplate chain
  - the `elai prompt promote --to` subcommand (the eval-gated promotion entrypoint)
  - the rollback-on-drift trigger wired into the G-10 canary-fire path
  - crates/.../ssb.rs::SsbActionType::PromptPromotionEvent (4-arm)
consumes_symbols:
  - D-1 ReasoningTemplate + blake3 hash-chain archive + select_template (the versioned store this stages; GD-1)
  - I-36 CompiledPromptStore + build_system_prompt_for_role (the prod-stage load path; GD-1)
  - G-10 drift canary (the regression signal for rollback; GD-1 do NOT re-implement the canary)
  - J-56 `elai eval run --scope subset` (the promotion gate; GD-1)
reachable_from:
  - the `elai prompt promote` CLI subcommand (PRODUCTION — an operator promotes a template version)
  - the prod template load path / build_system_prompt_for_role serving the Prod-stage hash (PRODUCTION read)
  - the G-10 canary-fire handler calling the rollback (PRODUCTION — a real drift event demotes)
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO LLM judging whether a prompt is "better" (TD-3 — the promotion verdict is a J-56 deterministic subset run)
  - NO new versioning scheme (reuse the D-1 blake3 hash-chain — GD-1)
  - NO Test->Prod skip (must pass Staging)
  - NO silent rollback (every demotion records from/to_lastgood/drift_signal — lesson-2)
  - NO auto-MODEL-swap (G-10's model posture stays surface-only; J-57 only demotes the PROMPT)
  - NO sha2 on template hashes (blake3)
  - NO .unwrap()/.expect() on the promote / load / rollback prod path
frontend_pair:
  N/A with justification — promotion + rollback surface through the EXISTING D-26 ReasoningTemplate dashboard (D-1, WIRED) by adding the stage badge + the promotion/rollback provenance rows; no new Tauri command. Justified. (If D-26 lacks a stage column, the ~20 LOC column add rides cross_lang_diff, W-4.)
observable_effect:
  trigger: "elai prompt promote <id> --to staging  then  --to prod  on a clean version; then promote a candidate version that regresses a J-56 category"
  expect_ssb: "PromptPromotionEvent::Staged then ::Promoted{eval_pass=true} for the clean version; ::BlockedByEval{regressed_categories=[...]} for the regressing candidate with the Prod pointer unchanged"
  expect_artifact: "the prod load path serves the newly-promoted hash; the regressing candidate never reaches Prod. NEGATIVE that must FAIL: a regressing version reaching Prod; a Test->Prod promotion skipping Staging; a drift-triggered demotion that does NOT record the last-good hash."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-57_eval_gated_prompt_promotion.md --run-trigger
rg -n "PromptStage|prompt promote|PromptPromotionEvent|RolledBack" crates   # the stage machine + entrypoints exist
rg -c "llm.*better|judge_model|gpt_chat|claude_chat" crates/.../prompt_promotion*  # MUST be 0 (TD-3)
rg -c "sha256|blake2" crates/.../prompt_promotion*                          # MUST be 0 (blake3 only)
rg -c "unwrap\(\)|expect\(" crates/.../prompt_promotion*                    # MUST be 0 (prod path)
test -f workflow/reasoning/J-57-decisions.md
```

## ACCEPTANCE
- [ ] TD-3: promotion verdict = J-56 deterministic subset run; NO LLM "better-prompt" judgement (rg count 0).
- [ ] STAGE DISCIPLINE: Test->Staging->Prod; a Test->Prod skip is rejected (contract-tested).
- [ ] ROLLBACK: a G-10 drift fire on a freshly-promoted Prod version auto-demotes to last-good with full provenance (contract-tested).
- [ ] R7/D-030: promote/load/rollback reachable from production (CLI + load path + canary handler), not a test.
- [ ] GD-1: D-1 chain / I-36 store / G-10 canary / J-56 gate consumed, not restated.
- [ ] W-2: promotion is a CLI + a gate, no new ELAI_*_ENABLE; I-36's ELAI_PROMPT_COMPILE_DISABLE unchanged.
- [ ] cargo check --workspace GREEN; auditor PASS on -p the touched crate only.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: entrypoint = `elai prompt promote` + the prod load path; artifact = PromptPromotionEvent rows. Met.
- W-2: CLI + gate, no enable-flag graveyard. Met.
- W-3: the talk verdict in-repo (steal doc). Met.
- W-4: measures the objective (the prod pointer actually moved / actually did NOT move on a regression), not stdout. Met.
- W-5: production_caller_check; promote/rollback called from CLI + canary handler, not a test. Met.
- W-6: rides the D-26 ReasoningTemplate tile (stage badge + provenance). Justified.
- W-7: D-1/I-36/G-10/J-56 anchors re-grepped at edit.
- W-9: SCHEMA baseline re-read at edit (PromptPromotionEvent = baseline+1).
- W-10: stage machine + gate in-process; the full ablation behind the subset is G-6 bench-tier. Tiered.
- GD-4: the eval-gate defaults to BLOCK-on-regression (fail-closed); the rollback's drift threshold reuses G-10's already-tuned signal, so no new FP surface. GD-5: a promotion is only as trustworthy as the J-56 corpus coverage of the affected category — honest residue: a thin category gates weakly until it grows.

## Tests to add
- crates/contracts/tests/j57_prompt_promotion_contract.rs (~110 LOC): (a) clean version promotes Test->Staging->Prod and the load path serves the new hash; (b) a regressing candidate -> BlockedByEval, Prod pointer unchanged; (c) Test->Prod skip rejected; (d) a simulated G-10 drift on a fresh Prod version -> RolledBack to last-good with from/to/signal recorded; (e) every promote/rollback writes governed provenance (from_hash/to_hash/reason); (f) template hashes are blake3; (g) NO LLM-judge on the promotion path.

## Files touched
- CREATE: crates/.../prompt_promotion.rs (~240 LOC: stage machine, eval gate, rollback) — re-grep the owning crate at edit
- MODIFY: crates/cli/src/main.rs (~40 LOC: `elai prompt promote --to`)
- MODIFY: the D-1 select_template / I-36 build_system_prompt_for_role load path (~40 LOC: serve Prod-stage hash) — re-grep
- MODIFY: the G-10 canary-fire handler (~30 LOC: rollback trigger) — re-grep
- MODIFY: crates/.../ssb.rs (PromptPromotionEvent 4-arm + golden lock) (~50 LOC)
- MODIFY: dashboard-v2 D-26 ReasoningTemplate tile (~20 LOC: stage badge + provenance) + Tauri read — re-grep
- CREATE: crates/contracts/tests/j57_prompt_promotion_contract.rs (~110 LOC)
- CREATE: workflow/reasoning/J-57-decisions.md (~30 LOC)

## Citations / Grounding
- Source: workflow/reasoning/steal-production-ai-talks-2026-06-20.md (Five-Pillars Pillar-5 prompts-as-code + lesson-2 governed messages). [DESIGN, vendor-framework directional.]
- ELAI ground truth (PLAN-asserted, executor re-greps per W-7): D-1 ReasoningTemplate + blake3 hash-chain + select_template; I-36 CompiledPromptStore + build_system_prompt_for_role; G-10 drift canary; J-56 `elai eval run --scope`.
- CLAUDE.md: TD-3 (deterministic promotion), W-2, blake3-only, no-.unwrap-in-prod, named-constants.
- LIVE SCHEMA baseline ~268/577 (Tier-B derived, 2026-06-15); PromptPromotionEvent = baseline+1; re-read at edit (R-c/W-9).

## Open questions / risks
- Where select_template / build_system_prompt_for_role live and how to thread the stage (D-1 vs I-36 crate) — PLAN-asserted; executor re-greps at EXECUTING (W-7).
- Auto-demote vs surface-only on rollback: J-57 auto-demotes the PROMPT (cheap, reversible, local) but mirrors G-10's surface posture on the MODEL — decide-at-edit only if a prompt rollback turns out non-reversible in a given store (it should not be; the last-good hash is in the chain).
- Honest residue (GD-5): a promotion gate is only as strong as the J-56 corpus subset for that category; J-57 depends on J-56 having real coverage — thin categories gate weakly until the corpus grows.

## Next
J-57 turns ELAI's already-versioned prompts into governed, eval-gated, rollback-able artifacts — prompts become code with a test->staging->prod pipeline. It is the prompt-layer twin of G-10's model-change-management. Pairs with J-56 (the gate it runs) and J-58 (which hardens the contract the prompts execute under).

End of J-57.
