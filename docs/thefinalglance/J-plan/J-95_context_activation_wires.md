# J-95: context-activation wires + sequencing track — the CMG technical half (IMP-1..IMP-8), mostly wiring shipped code, ONE thesis: ACTIVATION not invention

STATUS: PLANNED (WAVE 1 for the three concrete wires; the sequencing pointers ride their own plans' waves; BUILD = wiring shipped halves + verify-and-wire-if-dark, ZERO new capability, net 0 new SSB)

- Track J | lever IMPORTANT-CONTEXT-NO-BLOAT | grounding STRONG (context_budget.rs/context_policy.rs/adaptive_reduce/pristine_path_filter.rs/belief_state.rs code-grounded 2026-06-25) | nature BUILD (activation + sequencing) | effort M (3 wires) + S (pointers) | depends per-leg below
- This is the J-family context-activation track the K-17 OUT-OF-K block recorded but did not author (the run's HONESTY NOTE: "ELAI context-mgmt DESIGN is at/beyond SOTA; ACTIVATION is the gap... highest-leverage = finish/commit + build J-34, not a new plan"). So J-95 is NOT a new mechanism: it is THREE concrete call-site wires the deep-dive's PART-2 code investigation found dark (G2/G4/G5) plus a SEQUENCING note pointing the rest of IMP-1..IMP-8 at their EXISTING plans. Every leg is BOUND by the CL-1..CL-4 context-regime doctrine (CITED, not re-authored — H-50/H-51 wave) + HD-2 (deterministic masking never busts the cache prefix) + RR-3/ID-5 (any knee is ELAI-measured per-(model,lane), never a paper constant).
- Source: CONTEXT-MANAGEMENT-DEEP-DIVE-2026-06-25.md PART 2 (W1-W6 wired / G1-G6 gaps, grounded against the live tree) + PART 3 (IMP-1..IMP-8) + the SEQUENCING block (Waves A-D). K-17 PACKAGES the differentiator; J-95 wires the runtime. Doctrine: J-0_INDEX.md.

## What the deep-dive's code investigation actually found (the honest delta — most of IMP-1..IMP-8 is ALREADY built)
The 06-25 grep against the live tree found the design more wired than the 06-23 feature map claimed:
- W1 ContextBudget allocator [WIRED, now 10 slots] compute()+update_breakpoints() on the live cloud build (runtime_client.rs:544-577), 2 cache breakpoints + the HD-2 cache-bust debug_assert.
- W2 H-50 regime policy [WIRED — the "uncommitted" map note was STALE] context_policy.rs called from live_cli.rs:1227 / runtime_builder.rs:371 / runtime_client.rs:592,850; Local=floor(window*0.75) reads ELAI_LOCAL_MAX_CONTEXT, Cloud=floor(window*0.85); the big "local ran a cloud-shaped policy" bug is FIXED.
- W3 I-10 cache-economics TTL tiering [WIRED] cloud path; LOCAL->NoCache (correct).
- W4 dual tool-result serialization [WIRED, I-13-shape] to_ui_dict vs to_llm_content + trace_query expand-on-demand — big tool outputs never hit the model whole.
- W5 FSM phase-aware tool/rule loading [WIRED, 3A] PLANNING-context != EXECUTING-context.
- W6 BeliefState substrate [BUILT, schema ~248] belief_state.rs 437 LOC, handoff_digest.rs 526 LOC (H-51 built).
So IMP-1/IMP-3/IMP-7/IMP-8 are largely DONE or have a dedicated plan; the genuine remaining wires are G2, G5, and a G4 verify. This file builds those three and points the rest.

## THE THREE CONCRETE WIRES (each a vertical slice: shipped producer + the live consumer call-site + a smoke)

### LEG A — IMP-2 (G2): wire the SHIPPED adaptive_reduce relevance ladder into the live overflow path
- Lacuna (G2): adaptive_reduce (H-6's 3-level overflow ladder FilteredByRelevance -> ReducedGranularity -> ReducedScope, file_overlap_score sort) has ZERO production callers — only context_budget_integration.rs / context_budget_contract.rs (tests). compute()+update_breakpoints() ARE wired, but on a live turn OVERFLOW the path falls through to conversation.rs trim/compaction (coarse, NOT relevance-aware).
- Wire: at the conversation/turn-prep overflow branch, run adaptive_reduce (keep high-overlap SSB rows + current-file+direct-deps code, drop the irrelevant) BEFORE the lossy conversation-level compaction fallback. This is "keep the IMPORTANT context, drop the bloat" — the user's exact ask — by RELEVANCE not blind truncation. Pairs with the H-50 LOCAL regime (window-minimization reduces by relevance, never by tail-chop). TD-3: the reduction order is deterministic (file_overlap_score); no LLM picks what to drop.
- Reuse the existing context_reduction SSB row (the ladder already emits it); net 0 new action_type.

### LEG B — IMP-4 (G5): bring the I-5 pristine-injector discipline INTO the product context_budget assembly + compose J-78
- Lacuna (G5): pristine_path_filter.rs is BUILT (i5_pristine_context_contract.rs proves pristine_rebuild_zero_traceback_bytes), but the PRODUCT's own context_budget assembly does not yet hard-exclude failed-path noise — a failed exploration / wrong-path tool burst can still accumulate into the CodeContext / SsbEntries slots. This is the user's "no bloat from searching/finding the WRONG things."
- Wire: at the 3E slot-assembly seam (context_budget.rs compute path), route the SsbEntries/CodeContext candidate set through pristine_path_filter so a failed-sibling traceback / wrong-path burst becomes a contrastive CONSTRAINT ("don't do X, it failed because Y") not 5K of raw error text (the "vividness tax": failed paths over-anchor attention). COMPOSE with J-78's declared, journaled per-dispatch context-access spec (what a dispatch MAY read, computed deterministically) so what enters each model call is auditable, not whatever accumulated. OBSERVE-ONLY first (GD-4): emit the would-exclude set, measure the byte/relevance delta, before defaulting the exclusion ON.
- Reuse the I-5 pristine-path SSB row + the J-78 ContextAccessSpecEvent (or agent-dispatch-row reuse); net 0 new action_type here (J-78 owns its event).

### LEG C — IMP-3 (G4): VERIFY-and-wire the I-11 BeliefState condense at the I-7 escalation + cross-tier handoff call-site
- Lacuna (G4): the deep-dive's 06-25 grep could not see belief_state.rs callers on escalation_policy/cloud_mode, so a local->cloud escalation may still carry raw-ish history rather than the typed ~2K belief. The feature map (later code-grounding) says it IS wired at escalation_waterfall.rs:239 (BeliefStateEventKind::Condensed). RECONCILE: this leg is a VERIFY — confirm the condense fires on the LIVE I-7 EscalationCascade + the cross-tier handoff seam, and if a path is still dark, wire it. J-83 (GRC-v3 carrier) COMPLETES the carrier; J-95-C only guarantees the condense CALL-SITE is reachable on escalation, not a second carrier.
- Wire (only where dark): condense {tried / failed-because / current-AST-state / constraints, DKG-keyed} into the typed BeliefState on escalation/handoff instead of shipping >=20K of raw transcript (I-7 condense-on-escalate). The faithfulness of the condense is K-12's gate (Critical-Atom-Recall), not re-authored here.
- Reuse BeliefStateEvent (Condensed); net 0 new action_type. If the verify finds NO dark path, this leg degrades to a contract test asserting the wire + a one-line note (no code), recorded honestly.

## THE SEQUENCING POINTERS (existing plans — NOT re-authored here; GD-1 compose-not-restate)
Each IMP below already has a home plan; J-95 only records the build ORDER + the CL-binding, it does not duplicate the plan:
- IMP-1 [MEASURE, #1 leverage] = BUILD J-34 `elai bench context-fill` (the measured per-(model,lane) knee + positional curve feeding recalibrate_budget:849, the hook already there). Wave B. EVERYTHING downstream optimizes against J-34's number; until it lands, 0.75/0.85 + slot order stay folklore (G1). RR-3: never a paper knee.
- IMP-5 = BUILD J-16 (blast-radius ENFORCED auditor gate, user's #1) + J-38 (mid-task callable hybrid_search + the deterministic context-sufficiency gate). The "know everything it touches before a fix" leg — anti-half-blind. Wave A (J-16 gate) + Wave C (J-38 loop).
- IMP-6 = BUILD J-54 (free-API rotation pool) + H-7 (calibrated architect/editor/verifier role-routing). Concurrency WITHIN the parallel_cap=3 rate-limit ceiling: parallelism across ROLES + more lanes, not one giant call. Wave C. Batch-RACING stays REJECTED (IC-1).
- IMP-7 [DESIGN->BUILD] = I-1 directive-supervisor plane (generator emits pure Intent->Directive, cannot reach gate/side-effect code) — the STRUCTURAL form of the plan/execute split. Wave D, adversarial-check the local-lane latency budget first.
- IMP-8 [ACTIVATE] = J-19 vLLM backend (automatic prefix caching = the cross-request KV reuse the HF sidecar lacks on the streaming path, G3 — the lone REPLACE verdict, additive, no HF removal without user confirm). Wave D. Until then IMP-1's measured local knee + LEG A relevance-reduce keep the local window small enough that the missing cache hurts less.

## SEQUENCING (the deep-dive's Wave A-D, cheapest-first, each independently valuable)
- Wave A (pure activation, days): LEG A adaptive_reduce wire; LEG C BeliefState escalation verify/wire; IMP-5a J-16 gate enforce.
- Wave B (measurement, needs remote GPU): IMP-1 J-34 -> feeds everything.
- Wave C (build): LEG B pristine injector + J-78 access spec; IMP-5b J-38 retrieval loop + sufficiency gate; IMP-6b J-54 rotation pool.
- Wave D (structural, design-first): IMP-7 I-1 directive-supervisor; IMP-8 J-19 vLLM local cache.

## Existing ground truth (code-grounded 2026-06-25; re-grep at edit, W-7/W-9)
- context_budget.rs [WIRED, 10 slots] + adaptive_reduce [BUILT, zero prod callers — LEG A target]. context_policy.rs [WIRED] regime split. pristine_path_filter.rs [BUILT, i5 contract proven — LEG B target]. belief_state.rs [BUILT, escalation_waterfall.rs:239 — LEG C verify target]. J-34 context_calibration.rs [ABSENT — IMP-1]. live SSB baseline ~268/577 (re-grep at edit).

## Integration
- crates/orchestrator/src/ (context_budget.rs slot-assembly seam for LEG B; the conversation/turn-prep overflow branch for LEG A; the I-7 escalation_waterfall / cross-tier handoff seam for LEG C). NO new SSB (all three legs PREFER-REUSE shipped rows: context_reduction / I-5 pristine-path / BeliefStateEvent). frontend_pair: the would-exclude/would-reduce deltas surface on ContextStatePanel (decide at edit, W-4) or W-6-justified.
- Each leg AUDITOR-owned where it is a gate; each OBSERVE-ONLY first (GD-4) — LEG A/B emit the would-reduce/would-exclude set and measure the delta before defaulting the behavior ON.

## WIRING_CONTRACT (per-leg; each a vertical slice)
produces_symbols:
- A: the adaptive_reduce call at the live conversation/turn-prep overflow branch (relevance ladder BEFORE lossy compaction)
- B: the pristine_path_filter call at the context_budget.rs slot-assembly seam (failed-path -> contrastive constraint) + the J-78 access-spec compose point
- C: the verified/wired BeliefState condense at the I-7 escalation + cross-tier handoff call-site
consumes_symbols:
- A: adaptive_reduce (H-6, BUILT) + the conversation overflow branch + context_reduction SSB | B: pristine_path_filter (I-5, BUILT) + the 3E slot assembly + J-78 ContextAccessSpec | C: belief_state.rs condense + I-7 EscalationCascade + the handoff seam + BeliefStateEvent
reachable_from:
- A: a live `elai run` whose turn OVERFLOWS the trim threshold -> adaptive_reduce runs (NOT a unit test calling it) [PRODUCTION]
- B: a live model dispatch whose candidate context contains a failed-path burst -> the burst renders as a constraint, not raw text [PRODUCTION]
- C: a live local->cloud escalation -> a typed BeliefState (not raw transcript) crosses the boundary [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies any leg (R7)
stub_ban:
- NO LLM picks what to drop/exclude/condense (A/B/C are deterministic: file_overlap_score / pristine_path_filter / DKG-keyed condense; TD-3)
- NO cache-prefix rewrite (A/B place reductions AFTER the last cache breakpoint; HD-2 byte-stability assert holds)
- NO default-ON behavior before the would-reduce/would-exclude delta is measured on >=3 runs (GD-4) + a wire-dark-allowlist row where a flag is added (W-2/R9)
- NO paper knee anywhere (RR-3/ID-5; the trim fractions stay folklore until J-34 measures them — that is IMP-1, not this file)
frontend_pair: ContextStatePanel reduction/exclusion deltas (decide at edit, W-4) or W-6-justified.
observable_effect:
- trigger: "A: a fixture session driven past the trim threshold. B: a dispatch whose context candidates include a seeded failed-exploration burst. C: a local->cloud escalation on a hard fixture."
- expect_ssb: "A: a context_reduction row showing the RELEVANCE ladder ran (FilteredByRelevance ordering by file_overlap_score), recomputable (GD-2), BEFORE any conversation-level trim. B: an I-5 pristine-path row + a J-78 access-spec row showing the failed burst excluded/constrained. C: a BeliefStateEvent(Condensed) on the escalation seam carrying the typed ~2K belief."
- expect_artifact: "A: the high-overlap SSB rows + current-file code SURVIVE; the irrelevant is dropped (the model still answers the on-task question). B: the failed burst appears as a one-line contrastive constraint, not 5K of raw error. C: the cloud call receives the belief, not >=20K of raw transcript. NEGATIVE (R2): LEG A's --no-adaptive-reduce arm falls back to blind truncation and drops a high-overlap row (proves the ladder is load-bearing); LEG B's --no-pristine arm re-admits the failed burst raw; LEG C, if it finds NO dark escalation path, ships a contract test ASSERTING the wire (never a silent SKIP)."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-95_context_activation_wires.md --run-trigger
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/context_budget.rs --json   # adaptive_reduce + pristine_path_filter now have a non-test production caller (LEG A/B)
rg -n "adaptive_reduce" .elai_cc/crates/orchestrator/src/ | rg -v "test|contract"   # >=1 production call-site (was 0)
rg -n "pristine_path_filter" .elai_cc/crates/orchestrator/src/context_budget.rs       # the slot-assembly compose point exists
rg -n "BeliefState|condense" .elai_cc/crates/orchestrator/src/escalation_waterfall.rs # LEG C call-site verified
test -f workflow/reasoning/J-95-decisions.md
```

## ACCEPTANCE
- [ ] LEG A: adaptive_reduce has a real production caller at the overflow branch; the relevance ladder runs BEFORE lossy compaction; placed after the last cache breakpoint (HD-2 holds).
- [ ] LEG B: the context_budget slot assembly routes candidate context through pristine_path_filter; a failed-path burst renders as a contrastive constraint; composes J-78; OBSERVE-ONLY first.
- [ ] LEG C: the BeliefState condense is verified reachable on the I-7 escalation + cross-tier handoff; if any path was dark it is now wired; a contract test asserts the wire.
- [ ] net 0 new SSB action_types (all three legs prefer-reuse shipped rows; baseline re-read at edit, W-9).
- [ ] the SEQUENCING pointers are recorded, NOT re-authored (IMP-1->J-34, IMP-5->J-16+J-38, IMP-6->J-54+H-7, IMP-7->I-1, IMP-8->J-19); each cited with its wave.
- [ ] CL-1..CL-4 cited as the binding regime doctrine (not re-authored); RR-3/ID-5 honored (no paper knee).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator (+contracts). No new dependency.

## Tests to add
- crates/contracts/tests/j95_context_activation_contract.rs (~120 LOC): (a) an overflow fixture proves adaptive_reduce ran (relevance ordering) before conversation-level trim, and the --no-adaptive-reduce arm drops a high-overlap row (R2); (b) a failed-burst fixture proves the pristine filter rendered it as a constraint at the budget seam, --no-pristine re-admits it raw; (c) an escalation fixture proves a BeliefStateEvent(Condensed) lands on the seam (traced-test: logs_contain inside assert! per tracing-test-assert.md T-1); (d) HD-2 byte-stability assert across A/B (the cache prefix is unchanged).

## Files touched
- MODIFY: crates/orchestrator/src/ — the conversation/turn-prep overflow branch (LEG A), context_budget.rs slot assembly (LEG B), escalation_waterfall.rs / cross-tier handoff seam (LEG C). Exact sites re-grepped at edit (W-7).
- CREATE: workflow/reasoning/J-95-decisions.md (the three wire decisions + the would-reduce/would-exclude observe-only deltas + the LEG-C verify finding).

## Citations / Grounding
- CONTEXT-MANAGEMENT-DEEP-DIVE-2026-06-25.md: PART 2 W1-W6 (wired) + G1-G6 (gaps); PART 3 IMP-1..IMP-8; the Wave A-D sequencing. RULER arXiv:2404.06654 + Chroma context-rot (WHY guessed thresholds are wrong — design-rationale for IMP-1/J-34, never a claimed ELAI number, ID-5).
- ELAI ground truth (code-grounded 2026-06-25, re-verify at edit, W-7): context_budget.rs (10 slots), context_policy.rs (regime split WIRED), adaptive_reduce (BUILT, zero prod callers), pristine_path_filter.rs (BUILT), belief_state.rs (escalation_waterfall.rs:239). live ~268/577 (W-9).
- Hard rules invoked: TD-3 (LLM never picks the reduction), HD-2 (cache byte-stability), GD-4 (observe-first), RR-3/ID-5 (measure-your-own knee), GD-1 (the pointers compose, never re-author), W-2/R9, W-7/W-9. BOUND by CL-1..CL-4 (context-regime doctrine, H-50/H-51 wave — cited).
- K-17 is the K-native PROVE+PACKAGE half (the measured+regime-aware differentiator); J-95 is the technical activation half it recorded OUT-OF-K.

## Open questions / risks
- LEG C may find the escalation condense FULLY wired (feature map says so) -> the leg becomes a contract-test-only assertion + a note; recorded honestly, not padded into code.
- LEG A overflow seam: the exact branch (conversation.rs trim vs the 3E adaptive path) must be grepped at edit; the wire goes at whichever branch fires FIRST on a live overflow, before the lossy fallback.
- LEG B observe-only: defaulting the exclusion ON risks dropping a failed-path that was actually relevant (a near-miss). Measure the would-exclude precision on >=3 runs before flipping (GD-4); a constrained-not-deleted render keeps the information recoverable.

## Next
Wires the three dark context call-sites (G2/G5/G4-verify) and sequences the rest of IMP-1..IMP-8 onto their existing plans. The single highest-leverage downstream is IMP-1/J-34 (measurement) — until it lands, every reduction here optimizes against folklore thresholds. Pairs with the VCG track (J-96..J-98) only loosely: both are "the design is built, wire/measure it," not new architecture.

End of J-95.
