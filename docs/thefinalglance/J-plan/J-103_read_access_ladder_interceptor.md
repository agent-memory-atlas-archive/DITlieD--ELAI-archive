# J-103: graduated proof-of-need read-access ladder + Read-tool interceptor + savings meter — gate a raw code read to the cheapest rung that satisfies the NAMED identifiers, climb-on-miss, never hard-deny on economy

STATUS: PLANNED (the read-access lever the sdl-gap run found ELAI had every PIECE for but never assembled; a NEW policy/decision layer composing SHIPPED rungs + the SHIPPED rules engine + the I-46 interceptor mechanism + the J-38 sufficiency gate; default-OFF observe-only first; the escalate-on-miss is the brittleness guard and the over-block guard is the keystone — economy must DOWNGRADE not DENY)

- Track J | lever READ-ECONOMY (the model reads the cheapest rung that proves it has what it asked for, not the whole file) | grounding STRONG (DKG/B-29/E-17/rules-3F/I-46 bash_validation.rs:103/context_budget.rs code-grounded in the sdl-gap reasoning + this session's grep; re-grep at edit W-7) / EXTERNAL (MutaGReP/YoloFS/CodeRay/CrossCodeEval, design-rationale only, NEVER a done-gate number) | nature BUILD (new policy layer over shipped rungs) | effort L | WAVE 1
- Source: workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md §1 (J-103) + the gap file elai-gaps-sdl-ai-setup-2026-06-26.md G-1/G-2/G-5 + Downloads/deep-research-report(12).md §1/§2/§3 (the graduated-disclosure + hard-interception + economy grounding). Doctrine: J-0_INDEX.md + the universal binding block in the reasoning file §0 (ID-5/GD-1/TD-3/W/R9).
- This closes the user's earlier idea-3 ("read FILENAME and ELAI decides how much to actually pull"). sdl-mcp's Iris Gate Ladder (Symbol Card -> Skeleton IR -> Hot-Path Excerpt -> Code Window) is the design rationale; NO sdl-mcp code is copied (its license bans embedding in a for-sale product — ELAI has the K-plan GTM). Design ideas only.

## Context
Why this matters (the gap, from the gaps file G-1/G-2): ELAI's gates today are WRITE / TOOL / SECURITY-facing (VibeGate F-1, capability_gate, F-3 tool filter) — none is READ-ARTIFACT-facing. I-46 intercepts bare bash (grep/cat/find -> refuse-with-redirect) but there is NO interceptor on the model's first-class Read tool that steers a naive full-file read to symbol-card / skeleton retrieval first. ELAI already HAS every rung (DKG symbol cards, B-29 symbol-level impact API, adaptive AST chunking with FAST/STANDARD/DEEP tiers, full file) and the decision substrate (the phase-3F rules engine, rules-as-DATA), but the gated read-ESCALATION ladder with downgrade-over-code-reads was never assembled.

THE MECHANISM (sdl-mcp's contract, ELAI-grounded): a raw-source read request is GATED — the agent must NAME the identifiers it expects to find, and the enforcer verifies those identifiers exist in the served rung's real text BEFORE returning. A request is served the CHEAPEST rung that satisfies its declared identifiers; if the served rung does NOT contain the named identifiers (or symbol-coverage < threshold) the ladder CLIMBS one rung (downgrade-in-reverse) rather than silently serving a too-thin rung. A hallucinated/absent identifier therefore cannot pull a window — it escalates.

THE TWO LOAD-BEARING GUARDS (the research's two hard constraints):
- BRITTLENESS (report(12) §1, CrossCodeEval): over-aggressive pruning FLIPS correct answers to wrong when the skeleton misses a crucial detail. => the ladder MUST verify identifier presence and ESCALATE-ON-MISS, never serve a thin rung on faith. This is the J-38 deterministic context-sufficiency gate (symbol-coverage over the C-29 IntentPair) applied to the served rung.
- OVER-BLOCK (report(12) §2, YoloFS): an interceptor that DENIES a legitimately-needed read is the catastrophic failure mode (Gemini "deny outside project" -> 75% task failure via no-action). => the interceptor BIASES to DOWNGRADE + ESCALATE-ON-MISS and NEVER hard-denies on economy grounds; a deny is reserved for a security / capability_gate overlap, never for "this read is expensive".

HARD interception, not soft (report(12) §2): soft steering (a system-prompt nudge, an ignore-list) gets BYPASSED — Gemini-CLI's search ignores the ignore-list and the agent falls back to plain shell cat. ELAI's I-46 is already a HARD tool-boundary gate (refuse-with-redirect); J-103 widens the SAME mechanism to the Read tool, never a prompt nudge.

## Existing state (what the rungs / policy / interceptor build on — do NOT re-author, GD-1)
- DKG SYMBOL CARDS [WIRED] crates/knowledge/ phase-1A (tree-sitter -> SQLite symbol graph) — the CARD rung.
- SYMBOL-LEVEL IMPACT API [WIRED] B-29, 4 traversal tools impact_radius/call_flow/symbol_context/list, sharded cache, 4 confidence tiers — the CARD/SKELETON rungs (a symbol_context lookup IS the skeleton rung).
- ADAPTIVE CHUNKING + TIER ROUTING [WIRED] E-17/phase-3K FAST/STANDARD/DEEP adaptive AST chunking; J-25 (PLANNED) section-aware budgeted read — the HOT-PATH-EXCERPT rung.
- RULES ENGINE [WIRED] crates/rules/ phase-3F: rules are DATA in SQLite, predicates AST/GREP/SCHEMA/METRIC/COMPOSITE, verdicts ALLOW/WARN/BLOCK/ESCALATE, lifecycle PROPOSED->ACTIVE — the DECISION substrate. predicate.rs / policy_engine.rs / rule_engine.rs (grep-confirmed this session). J-103 ADDS a READ-ARTIFACT predicate family returning approve / DOWNGRADE-to-rung-N / deny.
- BASH INTERCEPTOR + FIXUP [PLANNED(I-46)] crates/tools/src/bash_validation.rs validate_read_only:103 refuse-with-redirect + frozen deterministic fixup table — the HARD interception mechanism J-103 WIDENS to the Read tool (the same PreToolUse / IpcToolDispatcher seam).
- CONTEXT-ACCESS SPEC [PLANNED(J-78)] the per-dispatch DECLARED context-access spec (the Conductor access_list, EXTENDS I-5) — the natural seam: J-78 DECLARES which symbols a dispatch may read, the ladder ENFORCES the declaration. J-103 composes J-78 (may ship alongside or soft-depend on it).
- RETRIEVAL AGENTIC LOOP / CONTEXT-SUFFICIENCY GATE [PLANNED(J-38)] deterministic context-sufficiency gate (symbol-coverage over C-29 IntentPair, TD-3, max-rounds cap) — the PRESENCE-CHECK + ESCALATE-ON-MISS leg J-103 consumes.
- INTENT-PAIR [WIRED] C-29 IntentPair + symbol-coverage — the unit the sufficiency gate scores.
- USAGE WINDOW / LIVE METER [PLANNED(J-29)] in-process LiveUsageGauge + ContextStatePanel absolute counts, read-side ZERO new SSB — the METER J-103 EXTENDS (the would-have-read-vs-served delta is a read-side gauge row).
- CATRAG + A-RAG PIPELINE [WIRED] phase-1E, 55-87% token-savings accounting — the existing token-savings substrate the meter consumes.
- CONTEXT BUDGET ALLOCATOR [WIRED] context_budget.rs phase-3E (CodeContext slot) — the slot the served rung fills.
- live SSB baseline ~268/577 (re-grep at edit, W-9); J-103 prefers-reuse the J-29 read-side gauge (net 0); +ReadAccessDecision ONLY if a journaled decision row is wanted for replay/audit (recommended, GD-2 — it makes the ladder's behavior recomputable).

## Reads (mandatory before coding)
1. workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md §1 IN FULL (the rungs, the policy layer, the presence check, the meter, the flag).
2. Downloads/deep-research-report(12).md §1 (graduated disclosure: MutaGReP/CodeRay), §2 (hard>soft interception: YoloFS), §3 (economy) — DESIGN RATIONALE ONLY, no number becomes a gate.
3. crates/rules/src/predicate.rs + policy_engine.rs + rule_engine.rs (the verdict enum + predicate families — the READ-ARTIFACT family extends these) — re-grep live at edit (W-7).
4. crates/tools/src/bash_validation.rs (validate_read_only / the refuse-with-redirect shape, I-46) + the PreToolUse / IpcToolDispatcher seam where a Read tool-call is routed (the interceptor call-site).
5. B-29 traversal tools (impact_radius/call_flow/symbol_context/list) + E-17 adaptive chunking tier routing + C-29 IntentPair symbol-coverage — the rungs + the coverage signal.
6. docs/thefinalglance/J-plan/J-38_*.md + J-78_context_access_spec.md + J-29 (the sufficiency gate, the access spec, the meter — the composed PLANNED units).

## Tasks (WAVE 1 — observe-only first; the over-block guard is the keystone)
1. [RUNGS] define the read-access RUNG ladder as a typed enum mapping to SHIPPED retrieval: SymbolCard (DKG/B-29 symbol_context) -> Skeleton (B-29 traversal / signatures) -> HotPathExcerpt (E-17 FAST/STANDARD adaptive chunk + J-25 section read) -> CodeWindow (full file). NO new retrieval — each rung DISPATCHES an existing path. PRODUCER: crates/orchestrator/src/read_ladder.rs (new).
2. [POLICY] add a READ-ARTIFACT predicate family to the phase-3F rules engine that, given a Read request + its NAMED identifiers + the declared J-78 access scope, returns approve(rung) / DOWNGRADE-to-rung-N / deny (deny ONLY on a security/capability_gate overlap, NEVER on economy — TD-3 deterministic). PRODUCER: crates/rules predicate family + read_ladder router.
3. [INTERCEPTOR] widen I-46's bash_validation refuse-with-redirect to the Read tool at the PreToolUse / IpcToolDispatcher seam: a naive full-file Read is intercepted -> the agent is REQUIRED to NAME the identifiers it expects -> the ladder serves the cheapest rung satisfying them. HARD gate, never a prompt nudge. CONSUMER: the real Read-tool dispatch entrypoint.
4. [PRESENCE-CHECK + ESCALATE-ON-MISS] after a rung is served, run the J-38 deterministic context-sufficiency gate (symbol-coverage of the NAMED identifiers over the served rung's real text, against the C-29 IntentPair). If coverage < threshold OR a named identifier is absent, CLIMB one rung and re-serve (max-rounds cap, TD-3). A hallucinated identifier escalates to CodeWindow, never returns a thin rung silently.
5. [METER] record would-have-read-tokens (the CodeWindow size) vs actually-served-tokens at every rung decision; surface a live "saved X vs naive read" delta via the J-29 LiveUsageGauge (read-side, prefer-reuse, NO new action_type) and consume the CatRAG phase-1E savings accounting. This is the G-5 instrument AND the done-gate measurement source.
6. [optional +ReadAccessDecision] if a journaled, replayable decision row is wanted (recommended for GD-2 recomputability + audit), add ONE Pattern-2 SSB action_type ReadAccessDecision with Kind {Approve, Downgrade, Deny, EscalateOnMiss} — full 9-tuple golden-lock lockstep, TOTAL_VARIANTS bumped from the LIVE integer at edit (W-9), never a doc number. Otherwise net 0 (ride the J-29 read-side gauge).
7. workflow/reasoning/J-103-decisions.md (the SSB decision: journal-a-row vs net-0-gauge; the rung-threshold DECISION; the over-block observe-only finding).

## WIRING_CONTRACT
produces_symbols:
  - the read-access RUNG ladder enum + read_ladder router (rung -> a SHIPPED retrieval dispatch) [read_ladder.rs]
  - the READ-ARTIFACT rules-engine predicate family (approve(rung)/DOWNGRADE/deny, deny only on security overlap) [crates/rules]
  - the Read-tool interceptor at the PreToolUse/IpcToolDispatcher seam (name-your-identifiers -> serve cheapest satisfying rung) [the real Read dispatch entrypoint]
  - the escalate-on-miss climb driven by the J-38 sufficiency gate over the named identifiers
  - the would-have-read-vs-served meter delta surfaced through the J-29 LiveUsageGauge (read-side)
  - OPTIONAL +ReadAccessDecision (Pattern-2, Kind {Approve,Downgrade,Deny,EscalateOnMiss}) if a journaled decision row is chosen
consumes_symbols:
  - DKG symbol cards (phase-1A) + B-29 traversal tools (impact_radius/call_flow/symbol_context/list) — the card/skeleton rungs, cited not modified
  - E-17 FAST/STANDARD adaptive AST chunking + J-25 section-aware read — the hot-path-excerpt rung
  - the phase-3F rules engine (predicate.rs/policy_engine.rs/rule_engine.rs verdict ALLOW/WARN/BLOCK/ESCALATE) — extended with the read-artifact family
  - crates/tools/src/bash_validation.rs (the I-46 refuse-with-redirect mechanism, widened to Read)
  - the J-38 context-sufficiency gate + C-29 IntentPair symbol-coverage (the presence check)
  - the J-78 declared context-access spec (which symbols the dispatch MAY read — the enforced declaration)
  - the J-29 LiveUsageGauge + CatRAG phase-1E token-savings accounting (the meter)
  - capability_gate / F-1 (the ONLY deny path — a security/scope overlap, never economy)
reachable_from:
  - a live `elai run` turn where the model issues a Read naming identifiers -> the interceptor serves the cheapest satisfying rung and the meter records the saved delta [PRODUCTION]
  - a live Read naming a hallucinated/absent identifier -> the sufficiency gate misses -> the ladder CLIMBS to CodeWindow, never returns a thin rung [PRODUCTION, the brittleness guard]
  - a live Read of a legitimately-needed whole file -> the ladder DOWNGRADES at most, NEVER hard-denies on economy (the over-block guard) [PRODUCTION]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies any leg (R7)
stub_ban:
  - NO hard-deny of a read on ECONOMY grounds (over-block = the YoloFS no-action failure; deny is reserved for a security/capability_gate overlap)
  - NO serving a rung that lacks a NAMED identifier without escalating (the CrossCodeEval brittleness flip — escalate-on-miss is mandatory)
  - NO soft prompt-nudge interception (HARD tool-boundary gate only, the I-46 mechanism; a nudge gets bypassed)
  - NO LLM adjudicating the rung or the climb (TD-3 — the LLM only NAMES the identifiers; the rung decision + escalate-on-miss are deterministic)
  - NO new retrieval path (each rung DISPATCHES a shipped path; bare "add a new chunker" is rejected)
  - NO default-ON before the over-block / FP rate is bounded on >=3 real runs (GD-4); the flag needs a wire-dark-allowlist row + promotion_plan_id J-103-promotion (W-2/R9)
frontend_pair:
  the "saved X vs naive read" delta surfaces on the J-29 usage meter / ContextStatePanel (read-side gauge). A dedicated tile is NOT required; decide at edit (W-4) or W-6-justify.
observable_effect:
  trigger: "a live read request naming real identifiers (serve cheapest satisfying rung); a request naming a hallucinated identifier (must climb to CodeWindow); a legitimately-needed whole-file read (must not hard-deny)"
  expect_ssb: "the J-29 read-side gauge records served-vs-would-have-read tokens (net 0 new action_type); OR if chosen, ReadAccessDecision rows {Approve,Downgrade,Deny,EscalateOnMiss} recomputable from the journal (GD-2). bump TOTAL_VARIANTS from LIVE at edit (W-9)."
  expect_artifact: "the served rung satisfies the named identifiers at minimal tokens; the meter delta is correct vs the CatRAG accounting; the hallucinated-identifier request climbed (zero 'served a rung missing a named identifier' cases); the whole-file request was DOWNGRADED at most, never hard-denied. NEGATIVE: a fixture that names an absent identifier and is served a thin rung FAILS the brittleness assert."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-103_read_access_ladder_interceptor.md --run-trigger
python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/read_ladder.rs --json   # the ladder router has a non-test production caller (the Read interceptor seam)
rg -n "ReadArtifact|read_artifact|RungLadder|read_ladder" .elai_cc/crates/rules/src .elai_cc/crates/orchestrator/src | rg -v "test|contract"   # the predicate family + router are production-wired
rg -n "validate_read_only|refuse|redirect|Read" .elai_cc/crates/tools/src/bash_validation.rs   # the I-46 mechanism widened to Read
rg -n "sufficiency|symbol_coverage|escalate|climb" .elai_cc/crates/orchestrator/src/read_ladder.rs   # the escalate-on-miss leg present
test -f workflow/reasoning/J-103-decisions.md
```

## ACCEPTANCE
- [ ] a live Read naming real identifiers is served the cheapest rung that contains them; the meter records served-vs-would-have-read tokens (the G-5 delta, ELAI-measured).
- [ ] a Read naming a hallucinated/absent identifier CLIMBS (escalate-on-miss) to a rung that contains it, up to CodeWindow — ZERO "served a rung missing a named identifier" cases (the brittleness guard, ELAI-measured).
- [ ] a legitimately-needed whole-file Read is DOWNGRADED at most, NEVER hard-denied on economy (the over-block guard; deny only on a security/capability_gate overlap).
- [ ] task-outcome parity at the reduced rung is measured by GATE VERDICTS (F-4/wiring/intent-pair), never an LLM-judge (TD-3); NO MutaGReP/YoloFS/CodeRay number is a done-gate.
- [ ] the rung decision + the climb are deterministic; the LLM only NAMES identifiers (TD-3).
- [ ] ELAI_READ_LADDER_ENABLE default-OFF with a wire-dark-allowlist row (promotion_plan_id=J-103-promotion); observe-only until over-block/FP bounded on >=3 real runs (GD-4).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator + -p elai-rules (+contracts); action_type_lock golden test GREEN if +ReadAccessDecision was added. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = a live read request is served the cheapest satisfying rung AND a hallucinated-identifier request climbs, observed end-to-end — not "the router exists". W-2: ELAI_READ_LADDER_ENABLE gets a wire-dark-allowlist row + promotion_plan_id J-103-promotion. W-3: N/A (new layer, no port — sdl-mcp code never copied, license).
- W-5: trigger = a live `elai run` read request (real-identifier serve + hallucinated-identifier climb + whole-file no-deny). W-7: rules predicate engine + bash_validation + B-29 traversal + the Read dispatch seam re-grepped at edit. W-9: ~268/577 re-grep; bump only if +ReadAccessDecision.
- TD-3: deterministic rung + climb, LLM names only. GD-4: observe-only first. GD-2: the optional decision row is recomputable. ID-5: no paper number as a gate. R7: test callers excluded. R9: the flag is allowlisted.

## Tests to add
- crates/contracts/tests/j103_read_ladder_contract.rs (~150 LOC): (a) a real-identifier fixture serves the cheapest rung containing them + the meter delta matches the CatRAG accounting; (b) the BRITTLENESS-NEGATIVE arm: a hallucinated/absent-identifier request CLIMBS to CodeWindow and NEVER returns a thin rung missing the named identifier (the load-bearing assert); (c) the OVER-BLOCK-NEGATIVE arm: a legitimately-needed whole-file read is DOWNGRADED at most, NEVER hard-denied on economy; (d) a security/capability_gate overlap is the ONLY deny path; (e) the rung + climb are deterministic across re-runs under ELAI_SEED (TD-3); (f) if +ReadAccessDecision, the rows replay byte-identically (GD-2).

## Files touched
- CREATE: crates/orchestrator/src/read_ladder.rs (the RUNG ladder enum + router + escalate-on-miss climb; exact seams re-grepped W-7)
- MODIFY: crates/rules/src/predicate.rs (+ policy_engine.rs/rule_engine.rs as needed) — the READ-ARTIFACT predicate family (approve(rung)/DOWNGRADE/deny)
- MODIFY: crates/tools/src/bash_validation.rs (widen the I-46 refuse-with-redirect to the Read tool) + the PreToolUse/IpcToolDispatcher Read dispatch seam (the interceptor call-site)
- MODIFY: the J-29 LiveUsageGauge read-side surface (the saved-delta meter) — net 0 new SSB unless +ReadAccessDecision
- MODIFY (only if +ReadAccessDecision): crates/orchestrator/src/ssb.rs + ssb/payloads.rs + action_type.rs + tests/action_type_lock.rs + .golden (full 9-tuple lockstep, TOTAL_VARIANTS bump from LIVE)
- CREATE: workflow/reasoning/J-103-decisions.md
- CONFIG: .elai_cc/wire-dark-allowlist.toml row for ELAI_READ_LADDER_ENABLE (type=experiment, owner, expiry, promotion_plan_id=J-103-promotion, smoke)

## Citations / Grounding
- workflow/reasoning/jplan-sdl-context-memory-readaccess-2026-06-26.md §1 (the full J-103 spec) + elai-gaps-sdl-ai-setup-2026-06-26.md G-1/G-2/G-5.
- Downloads/deep-research-report(12).md §1 (MutaGReP 5,473 tok = 4.3% of a 121k repo reaching 53.9% vs 58.7% = 95.7% fewer tokens for 4.8pt; CodeRay 2-3.4x), §1 brittleness (CrossCodeEval StarCoder 327 correct->wrong), §2 (YoloFS 99%@0.4 vs 98%@0.9 prompts = 55% fewer, HARD>SOFT, Gemini ignore-list bypass + deny->75% failure), §3 (~100 input tokens per output token economy) — ALL [EXTERNAL], design-rationale only, NEVER an ELAI done-gate number (ID-5/GD-1).
- ELAI ground truth (code-grounded this session; re-verify at edit W-7): DKG phase-1A [WIRED]; B-29 traversal tools [WIRED]; E-17 adaptive chunking [WIRED]; rules engine predicate.rs/policy_engine.rs/rule_engine.rs [WIRED]; bash_validation.rs validate_read_only:103 (I-46) [PLANNED]; J-38 sufficiency gate / J-78 access spec / J-29 meter [PLANNED]; CatRAG phase-1E [WIRED]. live ~268/577 (W-9).
- Hard rules invoked: TD-3 (deterministic rung + climb, LLM names only); GD-4 (observe-only); GD-2 (recomputable decision row); GD-3/ID-5 (no paper number as a gate); W-1/W-2/W-7/W-9; R7/R9. sdl-mcp design ideas only, NO code copied (license — the K-plan GTM).

## Open questions / risks
- OVER-BLOCK (highest, the YoloFS failure): the interceptor must NEVER hard-deny a legitimately-needed full read. Mitigation = DOWNGRADE-not-deny default + escalate-on-miss + observe-only first; the over-block rate is the gating metric before any default-ON.
- BRITTLENESS: a too-thin rung flips a correct answer wrong. Mitigation = the J-38 presence-check + escalate-on-miss is mandatory before any rung is returned.
- the SSB DECISION (journal a ReadAccessDecision row vs net-0 gauge): recommended to journal for GD-2 recomputability + audit; resolved in J-103-decisions.md. // DECISION.
- J-78 sequencing: J-103 enforces J-78's declared scope; if J-78 has not shipped, J-103 degrades to "name the identifiers" without the declared-scope cross-check (still correct, weaker provenance). Soft-dep, recorded.

## Next
Ships the read-access ladder the user asked for ("read FILENAME, ELAI decides how much") as a deterministic policy layer over shipped rungs, hard-intercepted like I-46, brittleness-guarded by J-38, over-block-guarded by downgrade-not-deny, and measured by the J-29 meter. Pairs with J-104 (live-index overlay — what the rungs read is never stale) and J-78 (the declared scope the ladder enforces). The economy is real but the guard is the point: never blind the model to save tokens.

End of J-103.
