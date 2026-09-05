# L-23: `/reason --profile explore` — a conjecture-criticism mode that maps rival explanations, on the local lane [KEYSTONE — the one NEW surface]

STATUS: PLANNED (WAVE 2; M-L; NEW surface — ~90% composition of planned units, the flagship DeepReason steal; depends L-22)

PROOF_SPINE (PSR-2): feeds PS-3 (hallucination / grounded-claims delta) — the explore deliverable is a survivor map of rival explanations each carrying its refutation STATUS, i.e. a grounded-claim class L-10 can score. The embedded S2 orbit-detector leg is itself PROOF_SPINE: none (discipline / waste-reduction; user-acked 2026-07-08 per steal-deepreason §5), riding inside this PS-3 unit.

Objective: ELAI has no lane for the question class "hard open explanatory question -> a MAP of rival explanations with their refutation status." B-39 /reason REFLECT refines ONE answer; L-9 grounds ONE token-perfect research answer; the idea-framer critic loop attacks ONE framing. DeepReason (steal-deepreason-2026-07-08.md S8) is a Popperian conjecture-criticism harness whose deliverable is exactly that map — which explanations survived, which died and WHY, and what evidence would decide between the survivors. L-23 composes ELAI's already-planned pieces into a `/reason --profile explore` (equivalently `elai explore`) mode: VS-shaped conjecture spread (L-22 propose_distribution on I-4) + stance rotation (anti-spiral personas) + claimant-authored falsifiers compiled to checks (L-22) + a refuted-relapse gate + the S2 orbit detector (this file) + problem turnover (I-26/I-8) + cross-family judging of survivors (D-13, order-swap + verbosity penalty = I-24/N-15) + the survivor map as the rendered artifact (K-11/N-7) — all journaled (SSB), replayable (RRA). THE DIFFERENTIATOR (and the integration smoke for a whole substrate): run the conjecturer on the LOCAL 7B with grammar-masked skeleton output (L-22 via E-35/N-13) and a frozen-prefix / KV-reused pack (H-2/I-10/J-36), i.e. ship the thing DeepReason's author explicitly COULD NOT — the harness working on small models, for cents, offline. This file ALSO lands the two general anti-spiral upgrades the explore mode needs first (S2 the gate-block-rate orbit detector as a 5th StagnationPattern, S3 the negative-case-law-at-the-gate-never-in-prompt doctrine), because the explore surface is their first consumer. HARD BOUNDARY: L-23 is a PROFILE that composes shipped/planned engines — it does NOT re-implement /reason, the personas, D-13, I-8, or the local-serving stack (GD-1). It does NOT adopt DeepReason's Dung argument-graph adjudicator (REJECTED, steal §4.3 — ELAI verdicts are execution-grounded).

Estimated effort: ~1 session (~M-L). In-process orchestration; the LOCAL conjecturer leg is RunPod/sidecar-gated (W-10) — the deterministic FIXTURE arm (recorded conjecture rounds) ships now, the live-7B arm lands behind the sidecar health gate.
Pre-requisites (re-grep live per W-7): B-39 /reason profile seam; L-22 falsifier + propose_distribution (crates/orchestrator/src/claim/falsifier.rs + verbalized_sampling.rs — land L-22 first); anti_spiral_extended.rs (StagnationPattern + personas); i8_voluntary_abort_contract.rs anchor (I-8 dead-end hash) + I-26 plateau collapse; D-13 adversary + J-45 cross-family; I-24/N-15 judge audits (order-swap + verbosity penalty); E-35 grammar mask (sidecar_xgrammar_hook.rs) + N-13 resolver; H-2 cache breakpoints + I-10 prefix-invariance + J-36 frozen-head prefix-KV reuse; K-11/N-7 human render; SSB (ssb.rs:1519) + RRA; LIVE SCHEMA/TV.
Parallel-safe with: N-16 (disjoint files). Depends: L-22 (the falsifier-carrying claim class + VS spread); the S2/S3 legs in THIS file are consumed by any later anti-spiral caller.
Source: workflow/reasoning/steal-deepreason-2026-07-08.md S8 (the flagship steal) + S2 (orbit detector) + S3 (negative case law) + §3.1/§3.2 (the local-lane serving trick). Doctrine: L-0_INDEX.md (EX-1..EX-4 new, HG-4, TD-3, ID-5, HD-2).

## Context
Why this matters: DeepReason's own measured findings say what works in open-ended exploration is STRUCTURAL, not prompt-content — fast stance rotation (late/early novelty 0.973 vs 0.846) and problem turnover (the only novelty-RAISING force, 1.12) beat every prompt trick, while "avoid the likely continuation" complement directives are a PLACEBO (they move output relative to the PROMPT, not out of the model's REPERTOIRE) — grounding S3. His clean orbit detector is a zero-embedding counter: GATE BLOCKS per window separated healthy from refuted-attractor-orbiting PERFECTLY on all 15 roots (0 vs 54 blocks, 4.3x token burn) where the embedding detector could NOT (medians 0.645 vs 0.671) — grounding S2. And his flagship result — a CHEAP model inside a deterministic epistemology harness >= a frontier model raw at ~1% cost on the question class the harness fits — is ELAI's own harness>model thesis, second-confirmed in the open-ended reasoning domain where ELAI's corpus is thinnest (the L-20 codebuff read was the first such confirmation). So explore is not chasing DeepReason; it is the demo that exercises E-35/N-13 + H-2/I-10 + J-36 on a real workload no competitor (and not even DeepReason itself) has run on small models offline. All DeepReason figures are n=1 single-seed self-diversity = DESIGN-RATIONALE only (ID-5); the L-10 explore arm re-measures ELAI's own survivor-map quality / cost.

## Existing state (B..K coverage — this profile INVENTS almost nothing)
- B-39 /reason REFLECT [WIRED-class] — the reasoning surface a new profile attaches to (L-9 is the precedent: a profile, not a new engine). [steal-deepreason §4.2]
- L-22 falsifier + propose_distribution [PLANNED, this run] — the falsifier-carrying claim class + the VS conjecture spread explore consumes. [L-22]
- anti-spiral [WIRED, anti_spiral_extended.rs] — StagnationPattern{Spinning,Oscillation,NoDrift,DiminishingReturns} + 5 personas; explore ADDS a 5th pattern Orbiting(target) (S2). [elai.map]
- I-8 dead-end hashing [WIRED, i8_voluntary_abort_contract.rs] — AST/approach-hash prune before the LLM call; the refuted-relapse gate + one of the three counters the orbit rate sums. [elai.map]
- I-26 plateau collapse + voluntary abort [WIRED] — problem-turnover / never-loop-a-dry-problem. [steal-deepreason §4.1]
- D-13 adversary + J-45 cross-family [PLANNED/PARTIAL] — the foreign-reviewer rule for judging survivors (his §9); I-24/N-15 supply the order-swap + verbosity-penalty judge audits. [steal-deepreason §4.2]
- E-35 grammar mask [WIRED, sidecar_xgrammar_hook.rs] + N-13 resolver [PLANNED] — the skeleton-valid-by-construction decode on the local 7B. [elai.map]
- H-2 breakpoints + I-10 prefix-invariance [PLANNED/ACTIVATION] + J-36 frozen-head prefix-KV reuse [PLANNED fork] — the cache-stable pack that drops a cycle's marginal cost to the tail tokens (§3.2). [steal-deepreason §3.2]
- K-11 finding-render + N-7 human-render-as-required-field [SHIPPED/PLANNED] — the survivor map rendered for a non-author to read. [N-plan]
- SSB (ssb.rs:1519) + RRA — the journal + byte-for-byte replay every explore round lands in. [elai.map]

## Reads (mandatory before coding)
1. workflow/reasoning/steal-deepreason-2026-07-08.md S8 + S2 + S3 + §3.1/§3.2; DeepReason spec §11 (capture control: generator vs adjudicator capture, the anti-relapse gate hash->embedding-neighbor->battery-equivalence, negative case law lives at the gate NEVER in prompts).
2. L-22 falsifier.rs + verbalized_sampling.rs; anti_spiral_extended.rs (StagnationPattern + persona rotation); the I-8 dead-end-hash + I-26 abort seams; D-13/J-45 dispatch; I-24/N-15 judge-audit; B-39 /reason profile registration; J-36 + H-2/I-10 pack/prefix-KV seams; K-11/N-7 render.
3. F-0 W-1/W-5/W-7/W-10; G-0 GD-1/GD-2/GD-4/GD-5; L-0 EX-1..EX-4 + HD-2 (deterministic-reversible only); CLAUDE.md TD-3, ID-5, no-.unwrap-on-prod, blake3-only; sidecar-protocol (the 3-step health check before any live-7B arm).

## Tasks (WAVE 2 — the orbit detector, the doctrine, then the profile)
1. the S2 gate-block-rate orbit detector (~50 LOC): a windowed counter over EXISTING signals — I-8 dead-end prunes + L-7 gate blocks + L-22/anti-relapse refusals — keyed by refuted-target; rate >= a measured floor => `StagnationPattern::Orbiting(target)` added to the anti-spiral enum; the switch names the attractor the persona rotation is escaping. Zero embeddings (his detector beat the embedding one). Journal-recomputable (GD-2). This is the general upgrade; explore is its first caller.
2. the S3 negative-case-law doctrine wire (~15 LOC + a doctrine paragraph): enforce tabu at the REGISTRATION gate (I-8 hash + the S2 orbit counter), NEVER by rendering refuted/banned content into a prompt (it primes the very content it bans and grows unbounded); keep the I-5 contrastive constraint ONLY for typed error-avoidance (failure class + location), never as a diversity instruction; diversity interventions are structural (stance rotation, problem turnover, L-22 tail-weighted VS). Codify as EX-1 in L-0 + a guard that a diversity directive is never assembled from gate/tabu content.
3. the explore profile state machine (~110 LOC): a `/reason --profile explore` round loop — CONJECTURE (L-22 propose_distribution K candidates, each a falsifier-carrying skeleton) -> free-criticism (run the compiled predicate/program falsifiers at zero LLM cost) -> stance rotation on stagnation/orbit (persona switch, EX-1 structural) -> problem turnover on a dry attractor (I-26) -> cross-family judge the SURVIVORS (D-13 + I-24/N-15 order-swap + verbosity penalty) -> emit the survivor map. Retention is Pareto over (hard-to-vary, reach, coverage) — advisory scoring, TD-3.
4. the survivor-map deliverable + render (~40 LOC): the artifact = {surviving explanations, refuted explanations + the falsifier that killed each, the deciding-evidence gaps between survivors}, emitted as an SSB round + rendered via K-11/N-7 (a non-author can read which rivals stand and why). EX-3: the deliverable is a MAP, never a single collapsed answer.
5. the local-lane differentiator arm (~50 LOC, W-10 tiered): wire the conjecturer to the LOCAL 7B via the E-35 grammar mask (skeleton valid by construction) on a frozen-prefix pack (H-2/I-10 breakpoint discipline) with J-36 head-KV reuse (re-decode only the tail per cycle). EX-4: the skeleton is correct BY CONSTRUCTION on the weak lane, not by template-following. FIXTURE arm ships now (recorded rounds); the live-7B arm is sidecar-gated (sidecar-protocol health check first).
6. workflow/reasoning/L-23-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/anti_spiral_extended.rs — StagnationPattern::Orbiting(target) + the gate-block-rate windowed counter (S2)
  - crates/orchestrator/src/reason/explore.rs — the explore profile round loop + the survivor-map builder
  - the `/reason --profile explore` (elai explore) CLI/profile registration
consumes_symbols:
  - claim/falsifier.rs::Falsifier + verbalized_sampling.rs::propose_distribution (L-22 — the conjecture spread + free criticism)
  - anti_spiral personas + I-8 dead-end hash + I-26 abort (the rotation / turnover / relapse gate — COMPOSE, never re-implement)
  - D-13 adversary + J-45 + I-24/N-15 (cross-family survivor judging + order-swap/verbosity-penalty audits)
  - E-35 DecodingMode + sidecar_xgrammar_hook.rs; N-13 resolver; H-2/I-10 pack+prefix; J-36 frozen-head prefix-KV reuse (the local-lane arm)
  - K-11/N-7 render; SSB emit; RRA replay
reachable_from:
  - `elai reason --profile explore "<hard open question>"` (CLI) > explore round loop [PRODUCTION reasoning surface]
  - the B-39 /reason profile dispatch > explore.rs [PRODUCTION profile registration]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO Dung argument-graph adjudicator (REJECTED steal §4.3; verdicts are execution-grounded)
  - NO refuted/banned content rendered into a prompt (S3/EX-1 — tabu lives at the gate); NO complement "be different" directive as a diversity lever (placebo)
  - NO embedding-based orbit detector (S2 — gate-block RATE, the zero-embedding counter that measured perfectly)
  - NO model adjudicating a survivor verdict or a retention score (TD-3, advisory only); NO re-implementation of /reason, personas, D-13, I-8, or the serving stack (GD-1 P0)
  - NO .unwrap()/.expect() on prod; blake3 not sha2; the live-7B arm never runs without the sidecar health check (sidecar-protocol)
frontend_pair:
  the survivor map on the reasoning/canvas surface (rival explanations + refutation status + deciding-evidence gaps) via K-11/N-7 (W-4); cross_lang_diff if a Tauri read is added.
observable_effect:
  trigger: "elai reason --profile explore on a hard open explanatory question with >=2 rival mechanisms, one of which carries a predicate falsifier that fires (FIXTURE arm now; live-7B arm behind the sidecar health check)"
  expect_ssb: "+ExploreRoundCompleted (Pattern-2; Kind: SurvivorRetained/RefutedByFalsifier/OrbitDetected/ProblemTurnedOver); reuse anti_spiral_hit for the Orbiting rotation; baseline+1 from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "the survivor map lists the surviving explanation(s), each refuted rival WITH the falsifier that killed it, and the deciding-evidence gap; a run that re-proposes a refuted-equivalent >= the floor emits OrbitDetected + rotates the persona naming the attractor. NEGATIVE: a run that renders a banned/refuted idea back into the conjecture prompt FAILS (S3); a single collapsed answer with no rival map FAILS (EX-3); an embedding-based orbit detector FAILS (S2)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-23_reason_explore_profile_conjecture_criticism.md --run-trigger
rg "Orbiting" .elai_cc/crates/orchestrator/src/anti_spiral_extended.rs   # the 5th pattern present
rg "profile.*explore|ReasonProfile::Explore" .elai_cc/crates/orchestrator/src/reason/explore.rs   # the profile registered
rg -c "embedding|cosine" .elai_cc/crates/orchestrator/src/anti_spiral_extended.rs   # MUST be 0 in the orbit counter (S2 zero-embedding)
rg -ci "Dung|grounded_extension|argument_graph" .elai_cc/crates/orchestrator/src/reason/explore.rs   # MUST be 0 (REJECTED §4.3)
test -f workflow/reasoning/L-23-decisions.md
```

## ACCEPTANCE
- [ ] EX-1: diversity is structural (stance rotation / problem turnover / tail-weighted VS); no complement directive; no gate/tabu content ever assembled into a prompt.
- [ ] EX-2: the orbit detector is the gate-block RATE per window (zero embeddings), added as StagnationPattern::Orbiting(target).
- [ ] EX-3: the deliverable is a survivor MAP (rivals + refutation status + deciding-evidence), never a single collapsed answer.
- [ ] EX-4: the local-7B conjecturer's skeleton is valid BY CONSTRUCTION via the E-35 mask on a frozen-prefix pack (J-36), not by template-following.
- [ ] TD-3: survivor verdicts + retention scores are advisory; the model never adjudicates; the cross-family judge is order-swap + verbosity-penalty audited (I-24/N-15).
- [ ] GD-1: /reason, personas, D-13, I-8, and the serving stack are COMPOSED, not re-implemented; the Dung adjudicator is NOT adopted.
- [ ] GD-4/W-10: FIXTURE arm ships now; the live-7B arm is sidecar-gated + observe-first before any explore verdict acts.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator (+ the CLI crate). No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = `elai reason --profile explore` runs a real round and emits the survivor-map SSB + artifact (not cargo-green).
- W-2: no enable-flag; the profile is opt-in by invocation, the local arm behind a health gate not an ENABLE flag.
- W-3: composes BUILT engines into a new profile, not a summary-only port; the Orbiting pattern extends the BUILT anti-spiral enum.
- W-4: survivor map rendered via K-11/N-7; cross_lang_diff if a Tauri read is added.
- W-5/W-10: live end-to-end trigger; the network/sidecar leg SPLIT — FIXTURE arm is the fail-closed in-process done-gate, live-7B arm RunPod/sidecar-gated.
- W-6: the survivor map verified THROUGH the render surface (dashboard-smoker) or justified N/A.
- W-7: /reason seam, anti_spiral_extended.rs, I-8, D-13, E-35/N-13, J-36, K-11/N-7 re-grepped at edit.
- W-9: +ExploreRoundCompleted or prefer-reuse anti_spiral_hit; SCHEMA baseline re-read live.
- GD-2: every survivor verdict + orbit signal is recomputable from the journal under ELAI_SEED (RRA byte-replay).
- GD-5: honest residue — the explore harness catches STALLED/orbiting dynamics, NOT wrong-but-stable shared-blind-spot consensus; exogenous anchors (the falsifier programs, retrieval, human ruling) are the only lever against that, named in-file (steal §6).
- ID-5: survivor-map quality / cost is ELAI-measured (L-10 explore arm), never a DeepReason figure.

## Tests to add
- crates/contracts/tests/l23_explore_profile_contract.rs (~110 LOC): (a) a rival explanation carrying a predicate falsifier that fires is RefutedByFalsifier and drops from the survivor set; (b) re-proposing a refuted-equivalent >= the floor emits Orbiting(target) + a persona rotation naming the attractor; (c) the orbit counter uses zero embeddings (assert no cosine/embedding call); (d) the deliverable is a multi-survivor map, not a single answer (EX-3); (e) a diversity directive is never assembled from gate/tabu content (S3/EX-1 — assert the prompt never contains a refuted-idea string); (f) the cross-family judge runs order-swap + verbosity-penalty (I-24/N-15) before a survivor verdict; (g) FIXTURE arm replays byte-identically under ELAI_SEED (GD-2/RRA); (h) NEGATIVE: a Dung/argument-graph symbol is absent (REJECTED §4.3); the live-7B arm refuses to run without the sidecar health check.

## Files touched
- MODIFY: .elai_cc/crates/orchestrator/src/anti_spiral_extended.rs (~50 LOC: Orbiting(target) + the gate-block-rate counter)
- CREATE: .elai_cc/crates/orchestrator/src/reason/explore.rs (~180 LOC: the explore round loop + survivor-map builder + local-lane arm)
- MODIFY: the B-39 /reason profile registration + the CLI verb (~25 LOC)
- MODIFY: the I-5 / context doctrine (~10 LOC: the S3 negative-case-law paragraph, tabu-at-gate-never-in-prompt)
- CREATE: crates/contracts/tests/l23_explore_profile_contract.rs (~110 LOC)
- CREATE: workflow/reasoning/L-23-decisions.md

## Citations / Grounding
- workflow/reasoning/steal-deepreason-2026-07-08.md S8 (the explore surface, the flagship steal), S2 (gate-block-rate orbit detector: 0 vs 54 blocks, embedding detector could not separate), S3 (complement directives placebo; negative case law at the gate never in prompts), §3.1/§3.2 (grammar-mask + frozen-prefix local-lane serving); DeepReason spec §11 (capture control) [EXTERNAL/DESIGN-RATIONALE, ID-5].
- The harness>model thesis second-confirmation (open-ended reasoning domain): steal §2; the first was the codebuff read (L-20). ELAI re-measures its own number (ID-5).
- ELAI ground truth (re-grepped at edit, W-7): anti_spiral_extended.rs, i8_voluntary_abort_contract.rs (I-8), D-13/J-45, I-24/N-15, decoding.rs:229 + sidecar_xgrammar_hook.rs (E-35), J-36/H-2/I-10, K-11/N-7, B-39 /reason.
- CLAUDE.md: TD-3, ID-5, HD-2 (deterministic-reversible only), no .unwrap on prod, blake3-only; sidecar-protocol (health check before the live-7B arm).

## Open questions / risks
- The orbit-rate FLOOR (gate blocks per window that flips Orbiting): his was 15-root-clean at a fixed threshold, but that is n=1 single-seed — ELAI seeds it conservatively + OBSERVE-first (GD-4), measured at L-10 before it acts. Decide the seed floor at authoring.
- Cross-family judging on the LOCAL lane: with only one local 7B, the D-13 foreign-reviewer needs a SECOND family (his own §17 shared-model-gravity residue). Decide at edit whether the local arm judges survivors via a cloud second family (D-13) or ships the FIXTURE arm's recorded cross-family verdicts until a 2x-family local setup exists.
- Scope creep into DeepReason's schools/lineage machinery: DEFERRED (steal §4.3) — the cheap 80% is stance rotation + cross-family dispatch; full lineage islands + reseed earn their keep only on long-horizon explore runs. Do NOT build them here.

## Next
L-23 gives ELAI a question class it had no lane for, and its live leg is the integration smoke that exercises E-35/N-13 + H-2/I-10 + J-36 on a real workload — the "cheap model in a deterministic epistemology harness, offline, for cents" demo no competitor has. It pairs with L-22 (the claim substrate it runs) and feeds L-10's explore arm (the measured survivor-map quality/cost delta, PS-3).

End of L-23.
