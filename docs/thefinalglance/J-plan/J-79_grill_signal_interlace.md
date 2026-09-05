# J-79: signal-triggered grill interlace — wire the shipped ambiguity signals (J-32/B-38/B-40/C-29) into the grill OFFER, keep the consent invariant

- Track J (vision-orchestration wave) | lever CANT-MISREAD | grounding STRONG (the grill SKILL.md HARD CONSTRAINT + the four shipped/planned signals are ELAI's own code; the interlace is the missing wire) | nature EXTENSION (J-32 + B-38 + B-40 + C-29 + grill + AutonomousGate) | effort M | WAVE VO-2 | depends none

- Premise: the user's 2026-06-23 observation (vision-and-orchestration-improvements-2026-06-23.md PART C) — grill is user-invoked-only and DISCONNECTED from Channel-A's automatic ambiguity machinery (J-32 front-door confidence / B-38 ambiguity score / B-40 idea-critic / C-29 intent-pair count). The signals BLOCK or chip; grill RESOLVES; nothing wires detector -> resolver. So a high-ambiguity task either gets a single thin J-32 CLARIFY chip (too thin for a multi-branch ambiguity) or silently proceeds on the LLM noticing. The grill SKILL.md HARD CONSTRAINT bans auto-RUNNING grill (anti-sycophancy) but already SANCTIONS the OFFER ("if a task looks fuzzy ... OFFER it in one line and wait"). So the line between banned and allowed is the CONSENT step, not the trigger — today the offer fires on the LLM's vibe, not on the measured signals ELAI already computes. J-79 moves the OFFER trigger from vibe to a measured signal, keeping consent.

## Steal/build
1. G1 [INTERACTIVE mode] the CLARIFY LADDER, rung chosen by a deterministic signal, consent preserved:
   - rung 0 (single missing slot): J-32 low-confidence on ONE arg_span -> the existing single CLARIFY chip. UNCHANGED.
   - rung 1 (structural/multi-branch ambiguity): B-38 ambiguity score > threshold ACROSS multiple decision branches, OR B-40 idea-critic returns BLOCK, OR C-29 cannot reach >=3 intent-pairs for a step -> OFFER grill in one line ("this looks ambiguous across N branches — grill it before i plan ?") and WAIT. consent preserved (user-invoked-only invariant holds; only the TRIGGER moved from vibe to a measured signal).
2. G1 RETURN PATH (the deepest half of the interlace): grill's structured output is exactly what the detecting gate needs to PASS — grill RESOLVED DECISIONS + GLOSSARY lowers the B-38 score (pinned terms); grill's per-branch resolutions supply the >=3 C-29 intent-pairs; grill CONTRADICTIONS RESOLVED clears the B-40 critic BLOCK. So grill becomes the structured UNBLOCK path for the gate that detected the problem: detector (B-38/C-29/J-32/B-40) -> offer grill -> grill resolves -> gate passes. That closed loop IS the interlace.
3. G2 [AUTONOMOUS mode] compose AutonomousGate.decide() + grill: in autonomous mode there is no human to grill mid-run, but the autonomous gate ALREADY escalates to a human on HIGH risk / failed evidence / anti-spiral. When the escalation CAUSE is ambiguity-grade (a B-38/B-40/C-29 evidence-check failure, NOT a risk/cost tripwire), the escalation surface OFFERS grill as the resolution path rather than dumping a raw question — interlacing grill with Channel B, not just Channel A.
4. RULE-TEXT SHARPENING (cheap, do first): sharpen grill SKILL.md's HARD CONSTRAINT to distinguish the two cases it currently blurs — "never auto-RUN grill" (the real ban) vs "signal-trigger the OFFER" (G1, explicitly allowed, consent-gated). As written a literal reader could think any signal-driven behavior violates the constraint; the offer is already sanctioned later in the same file, so the wording is the only thing in the way.
EXCLUDED: auto-RUNNING grill off a signal (the HARD ban — G1/G2 are OFFER-only, consent-gated, never auto-run; an interrogation the user did not ask for is the friction the anti-sycophancy posture guards against); an over-eager offer on every task (itself a mild version of the same friction — a chip that cries wolf trains the user to dismiss it; the threshold gets the GD-4 discipline below); a new ambiguity-detection mechanism (the four signals are shipped/planned — this is the missing WIRE).

## Existing ground truth (grep-confirmed 2026-06-23; re-grep at edit, W-7/W-9)
- ELAI FRONT-DOOR ROUTER [PLANNED(J-32)] T2 local-3B {lane, confidence, arg_spans}; CHAT = safe default, CLARIFY chip on low confidence (fail-closed to asking); chip decisions journal as calibration tuples feeding D-16 conformal tau; +IntentRouted (feature map 1.10). rung-0 IS this chip; the offer reuses IntentRouted.
- AMBIGUITY SCORE GATE [WIRED] B-38 AmbiguityScorer + GroundingSweepGate extension + SSB + dashboard. the rung-1 score trigger + the grill-lowers-it return path.
- IDEA-FRAMER + CRITIC LOOP [DARK(ELAI_IDEA_FRAMING_ENABLE)] B-40 10 critic categories; the idea-critic BLOCK is the strongest, least-noisy rung-1 signal (sequence it first).
- INTENT-PAIR + METAMORPHIC [WIRED] C-29 IntentPair; intent_pairs<3 blocks PLANNING->TESTING_FIRST. the <3-pairs rung-1 trigger + the grill-supplies-pairs return path.
- AUTONOMOUS MODE [DARK(ELAI_DISABLE_AUTONOMOUS_GATE, ELAI_GATE_MODE)] B-22 AutonomousGate / AutoAdvanceEvaluator EvidenceCheck per transition + escalation arms (HIGH risk / failed evidence / anti-spiral). G2 extends the escalation surface.
- grill SKILL.md (.claude/skills/grill): user-invoked-only; HARD CONSTRAINT bans auto-RUN but sanctions the OFFER. the rule-text sharpened in leg 4.

## REVISED scope (W-10 tiered)
- SHIP NOW Tier-1 (cheap, do first): leg 4 the rule-text sharpening (harness, .claude/skills/grill SKILL.md — distinguish auto-RUN ban from signal-triggered OFFER) + G1 rung-1 offer on the B-40 critic BLOCK ONLY (the strongest, least-noisy signal). DEFAULT-CONSERVATIVE / OBSERVE the offer rate first.
- Tier-1 also: the G1 return path (grill resolved-decisions -> lower B-38 / supply C-29 pairs / clear B-40 BLOCK).
- DEFER widening: the B-38-score and C-29-pairs rung-1 triggers widen ONLY after the FALSE-OFFER rate (offer fired on a task the user found perfectly clear) is bounded on >=3 real runs (GD-4 — same posture as I-19's FP-bound). G2 autonomous-escalation offer after the interactive offer is calibrated.

## Integration
- crates/orchestrator (the J-32 FrontDoorRouter seam + the B-38/B-40/C-29 gate sites) — the rung selector: read the signal, pick rung 0 (chip) or rung 1 (offer grill), journal the decision. The offer surface itself is a one-line prompt + wait (consent).
- crates/orchestrator/src/autonomous_gate.rs (G2) — when decide() escalates on an ambiguity-grade cause, the escalation payload names grill as the resolution path.
- the B-38/C-29/B-40 gate consumers — the RETURN path: a grill resolved-decisions artifact lowers the B-38 score / supplies >=3 C-29 pairs / clears the B-40 BLOCK (deterministic, the gates re-evaluate on the resolved input).
- .claude/skills/grill/SKILL.md — leg 4 rule-text sharpening (harness; the only non-runtime deliverable).
- SSB: PREFER-REUSE J-32 IntentRouted (the offer is a chip-class routing decision; the false-offer rate is a calibration tuple feeding D-16, exactly as J-32's chip decisions already do). net 0 — no new action_type; the offer/decline is an IntentRouted Kind. re-read at edit (W-9).
- cross_lang pair: none (Rust signals + a harness skill-md); frontend_pair: the offer chip on the chat/front-door surface (J-32) or W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator clarify-ladder rung selector (signal -> chip|offer-grill, consent-gated, journaled)
- the grill RETURN path: resolved-decisions consumer that lowers B-38 / supplies C-29 pairs / clears B-40 BLOCK
- autonomous_gate.rs escalation-names-grill on ambiguity-grade causes (G2)
- .claude/skills/grill/SKILL.md sharpened HARD CONSTRAINT (auto-RUN ban vs OFFER-allowed)
consumes_symbols:
- J-32 FrontDoorRouter + IntentRouted (the rung-0 chip + the offer's journal; NOT re-authored)
- B-38 AmbiguityScorer (the rung-1 score trigger + the return-path consumer; NOT re-authored)
- B-40 idea-critic BLOCK (the strongest rung-1 trigger + the return-path clear; NOT re-authored)
- C-29 IntentPair (<3-pairs trigger + the return-path supply; NOT re-authored)
- B-22 AutonomousGate.decide() escalation arms (G2; NOT re-authored)
reachable_from:
- a real `elai` chat/plan turn where B-40 returns a critic BLOCK (or B-38 score high / C-29 <3 pairs) -> the offer surfaces  [PRODUCTION]
- a real autonomous run where AutonomousGate.decide() escalates on an ambiguity-grade cause -> the escalation names grill  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO auto-RUN of grill off any signal (the HARD ban — every rung-1 path is an OFFER + WAIT; consent is the invariant)
- NO offer that fires on a clear task above the bounded false-offer rate (GD-4 — default-conservative, widen only after >=3 real runs bound the FP)
- NO new ambiguity-detection mechanism (the four signals are shipped/planned; this is the wire)
- NO new SSB action_type (reuse J-32 IntentRouted; the offer/decline is a chip-class calibration tuple)
- NO grill resolved-decisions treated as an LLM verdict on the gate (TD-3 — the resolved input is re-scored deterministically by B-38/C-29/B-40, grill supplies structured input not a pass)
frontend_pair: the offer chip on the J-32 chat/front-door surface (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "a real `elai` plan turn on an idea where B-40 idea-critic returns BLOCK (a multi-branch ambiguity); the user accepts the grill offer; grill returns resolved decisions; the gate re-evaluates"
- expect_ssb: "J-32 IntentRouted with the offer-surfaced Kind (the chip-class calibration tuple); after grill, B-40 re-evaluates and the BLOCK clears on the resolved input (or B-38 score drops / C-29 reaches >=3 pairs)"
- expect_artifact: "the offer surfaced in ONE line and WAITED (never auto-ran grill); on accept, grill's resolved-decisions lowered the B-38 score / supplied the >=3 C-29 pairs / cleared the B-40 BLOCK so the gate passes. NEGATIVE (the consent invariant): on a task the user finds clear, the offer is declined and the run proceeds (the false-offer is journaled to bound the rate); grill is NEVER auto-run; in autonomous mode with no human, the offer cannot fire mid-run — only the escalation surface names grill (G2)."

## Grounding
- the user's observation + the grill SKILL.md HARD CONSTRAINT (the auto-RUN ban + the sanctioned OFFER) + the four signals (J-32/B-38/B-40/C-29) + AutonomousGate (B-22) — all ELAI's own code/skills, codegraph/feature-map confirmed 2026-06-23. full synthesis: workflow/reasoning/vision-and-orchestration-improvements-2026-06-23.md PART C (G1/G2 + the rule-text sharpening).
- GD-4 FP-bound discipline (same posture as I-19's FP-bound, G-0_INDEX): a signal->offer trigger ships default-conservative and the FALSE-OFFER rate is measured on >=3 real runs before the threshold is widened — an offer that cries wolf is a regression.
- anti-sycophancy (CLAUDE.md / main-llm-role.md): the reason grill is user-invoked-only — auto-RUNNING an interrogation is the friction the rule guards against; G1/G2 must NOT cross into auto-run.
- LIVE SSB ~269/578 (post-J-66; grep 2026-06-22). re-grep at edit (W-9).

## COMPOSES-WITH
J-32 front-door router (rung-0 chip + IntentRouted journal + the offer surface), B-38 ambiguity score (rung-1 trigger + return-path lower), B-40 idea-critic (the strongest rung-1 trigger + return-path clear), C-29 IntentPair (<3-pairs trigger + return-path supply), B-22 AutonomousGate (G2 escalation), grill skill (the resolver + the sharpened rule-text). Restates none — the four signals and grill are all shipped/planned; this is the missing wire + a measured threshold.

## Open questions / risks
- false-offer rate is the whole risk: an over-eager offer is mild friction that trains the user to dismiss the chip — ship the B-40-BLOCK trigger first (strongest, least-noisy), OBSERVE the offer rate, widen to the B-38 score only after the FP is bounded (GD-4).
- B-40 is DARK (ELAI_IDEA_FRAMING_ENABLE): the strongest rung-1 signal is behind a flag — the trigger un-darks with B-40; until then the wire lands behind the same flag (allowlisted) and the C-29 <3-pairs trigger (WIRED) carries the interactive case.
- rule-text sharpening is harness-only (JD-1): leg 4 edits .claude/skills/grill SKILL.md — no SSB, evidence is the skill-md diff; do it first (cheapest, unblocks the offer wording).

End of J-79.
