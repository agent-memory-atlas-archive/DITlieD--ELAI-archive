ELAI future-proofing: pre-research reasoning (training knowledge, before Perplexity results)
date: 2026-06-12
author: main LLM (Fable 5), after a full read of phase-index-MASTER, B/C/D/E/F/G/H/I/J
summary indexes, and ELAI-FEATURE-MAP.md (~350+ planned/built units, live SCHEMA 234/543).
epistemic status: everything below labeled. "High confidence" = grounded in the files read
this session. "From training (unsearched)" = my model of the market up to ~Jan 2026, to be
checked against the three research prompts. This doc is round 1; round 2 happens when the
research returns.

scope note per user instruction: ignore current wiring percentages and completion state.
Reason about what ELAI WILL BE when finished, vs where the market will be.

================================================================================
1. WHAT ELAI-AS-DESIGNED ACTUALLY IS (the one-paragraph identity)
================================================================================

High confidence (from the corpus): ELAI is an autonomous software-engineering system whose
load-bearing idea is that the GENERATOR is never trusted: every claim, edit, test result,
promotion, and self-modification must leave an off-agent, deterministic, journaled,
replayable piece of evidence (SSB journal, RRA bundles, stdout-hash-bound test outcomes,
attestations, taint tags, conformal refusal, frozen evaluators, trial ledgers). Around that
spine it builds: a knowledge engine (DKG/BM25/vector/PageRank/CatRAG), a 9-state FSM with
evidence gates, BSP-deterministic multi-agent orchestration, process isolation, a
self-improvement loop gated by execution-grounded un-gameable filters, a hybrid local+cloud
inference plane, a generative canvas UX, and a measurement doctrine (SAFEDELTA/QDELTA,
ablation-only framing, every number journal-recomputable under a seed).

That identity is coherent and, importantly, it is DIFFERENT from every major competitor I
know of, which are all variants of "frontier model + tools + permissions + nicer review UX".

================================================================================
2. IS ELAI THE ULTIMATE SOTA OF WHAT IT IS TRYING TO BE?
================================================================================

Split the question by axis, because the answer differs per axis.

2.1 VERIFICATION / CANT-LIE AXIS — yes, ahead of anything I know.
From training (unsearched): no shipping product binds test outcomes to orchestrator-captured
artifact hashes (F-4), ships reproducible-run artifacts with replay+divergence reporting
(G-2), does closed-set citation pre-binding (H-53), obligation-closure scanning on formal
lanes (H-54), conformal refusal calibrated per-project (D-16), or per-iteration security
monotonicity (F-10). Academic work exists piecewise (the corpus cites it correctly); nobody
has composed it into one system. The Track-7 insight — a self-improvement loop without an
un-gameable stronger-than-generator filter provably collapses — is correct per the 2025
literature and almost nobody in industry acts on it. Verdict: design-SOTA. The gap is not
ideas; it is that the headline number (G-6 SAFEDELTA) has never been produced, so the
superiority is currently unfalsifiable — which in market terms means it does not exist yet.

2.2 EFFICIENCY AXIS (H Track 1, Track 14) — correct but perishable.
High confidence from the files: the caching/output-token/batch/routing work is well-grounded
in real Anthropic API mechanics. From training (unsearched): this is exactly the layer
providers absorb fastest. Prompt caching was external know-how in 2024 and a checkbox by
2025; context compaction, memory, tool-search, and sub-agent orchestration were all moving
into the platform by late 2025. Expected half-life of a harness-side efficiency trick:
6-18 months. The MEASUREMENT half (H-9 cost cockpit, J-28/J-29 token ledger, I-27
convergence curves) is durable — knowing your own $/verified-task survives any provider
change; the TRICKS half decays. Verdict: keep the instruments, treat the tricks as
depreciating assets, never as moat.

2.3 KNOWLEDGE/RETRIEVAL AXIS — competitive, not uniquely ahead.
From training (unsearched): tree-sitter+SCIP symbol graphs, hybrid BM25+vector+rerank, repo
maps — Sourcegraph, Cursor, Augment, Cody, and the codebase-RAG papers all converge here.
ELAI's distinctive piece is retrieval wired into VERIFICATION (impact cones feeding an
enforced blast-radius gate, J-16; provenance-validated memory, H-32/J-21) rather than just
into generation context. That coupling is the defensible part; raw retrieval quality is not.

2.4 SELF-IMPROVEMENT AXIS — ahead in safety design, behind in applied scale.
From training (unsearched): labs run massive RL/distillation pipelines ELAI cannot match in
compute. ELAI's edge is the opposite end: a PRODUCTION self-improvement loop on a user's own
machine with promotion gated by trial ledgers, deflation, held-out firewalls, reward-tampering
gates. Nobody ships that. But its practical value is unproven (F-11 has zero live admit rows),
and the I-30/I-31 SFT/GRPO flywheel needs RunPod-scale compute to matter. Verdict: a genuine
differentiator IF the verification spine works first; worthless before that.

2.5 UX AXIS — the canvas thesis is right, the surface area is risky.
From training (unsearched): the market converged on two retained paradigms by early 2026:
(a) IDE-embedded autocomplete+chat, (b) cloud/background agents reviewed PR-style. The
"watch the agent work" 3D orbit is a known dead end (the corpus itself retires it, H-16).
The canvas (H-10..H-15) with a journaled, taint-tagged, reproducible op-stream is a genuinely
novel third paradigm — but it competes on polish against companies with 100x the frontend
staffing. The reviewer cockpit (G-5/H-52: per-hunk evidence, DSR) is the UX piece that
directly monetizes the trust spine and is much cheaper to win with. Verdict: cockpit before
canvas; canvas as the long bet.

2.6 DISTRIBUTION / ECOSYSTEM AXIS — behind, structurally.
High confidence: single-user, local-first, Tauri dashboard, no team layer, no SSO/RBAC/org
audit, no cloud execution, GitHub adapter partial. From training: every serious competitor
lives where developers already are (IDE, GitHub, Slack, CI). ELAI's two designed answers are
right: I-40 (`elai wrap` — the trust layer around ANY agent + the open .rra spec) and J-2
(ELAI as an MCP control plane). These convert ELAI from "yet another agent you must switch
to" into "the evidence layer under the agent you already use". That is the distribution
unlock and it is currently just a plan file.

================================================================================
3. WHERE SHOULD ELAI BE BETTER THAN EVERYONE? (the user's biggest question)
================================================================================

The answer the whole corpus converges on, and which I endorse after reading it end to end:

ELAI should be the system whose word you do not have to take.

Concretely, best-in-world at exactly four things:

(1) EVIDENCE-BOUND AUTONOMY. Every autonomous action carries off-agent proof (test bound to
artifact hash, gate verdict, taint label, journal row). Competitors scale autonomy by asking
for more trust; ELAI scales autonomy by needing less of it. This is the only axis where
demand GROWS as models improve — more capable models get delegated bigger tasks, and bigger
delegation raises the cost of unverified claims. The bitter lesson commoditizes scaffolding
that substitutes for model capability; verification does not substitute for capability, it
prices it, so it is bitter-lesson-resistant.

(2) THE MEASURED NUMBER. "N fewer dynamically-exploitable diffs per 100 tasks reach a
reviewer, gate-ON vs gate-OFF, CI included, replayable from the journal." Nobody in the
market publishes ablation-framed safety deltas of their own harness. First mover on an
honest, reproducible number owns the framing. Everything in G-6/I-38/I-43 is designed for
this; none of it has run.

(3) REVIEW-TIME COMPRESSION. The industry's stated bottleneck is reviewing agent output. The
cockpit (G-5 DSR + H-52 anchoring + F-4 vouched hunks + I-37 per-requirement status) turns
the trust spine into minutes saved per PR — the only translation of "trust" into a number a
team lead feels weekly. This should be THE flagship surface, ahead of the canvas.

(4) REPRODUCIBILITY AS A FILE FORMAT. The .rra (sealed, redacted, attestable, replayable run
artifact) as an open spec + conformance checker (I-40), with ELAI as the reference
implementation. If a "show me the run artifact" norm emerges (regulatory pressure, EU AI Act
audit trails, enterprise procurement), the spec owner wins by default. This is the cheapest
venture-scale option in the whole corpus.

And explicitly NOT trying to be best at: raw model quality (ride the frontier), IDE
integration depth (cede to incumbents), app-generator onboarding magic (different market),
consumer polish.

================================================================================
4. THE HONEST RISK LIST (what could make finished-ELAI still lose)
================================================================================

R1. THE UNMEASURED MOAT. High confidence: SAFEDELTA never invoked, the cockpit study never
run, F-2b never run on the real model. Until then the entire wedge is a hypothesis. This is
risk #1 by a wide margin and it is cheap to retire (I-43 is designed as exactly this cheap
falsifier).

R2. PLAN-CORPUS GRAVITY. ~350 sessions at 6-12h each is multiple person-years. The corpus's
own discipline (GD-1, riders, declines) is excellent, but the letters keep growing faster
than wiring audits close. Risk: a competitor ships a worse trust layer 18 months earlier.
The mitigation is ruthless sequencing, not more authoring — the next letter should be a
CUT/SEQUENCE plan, not a feature plan.

R3. FRICTION TAX. From training: every security gate that shipped before bounding its
false-positive rate died of user revolt (the corpus knows this — GD-4 exists because of
CodeGate). With ~30 gates planned on the write path, the COMPOSED FP rate and latency across
all gates is itself a product risk no single plan owns. A "total gate tax budget" (one
end-to-end friction number, measured per release) deserves first-class status.

R4. PROVIDER ABSORPTION + PROVIDER DEPENDENCE. The efficiency tracks decay (2.2). Worse:
deterministic replay and caching guarantees lean on Anthropic API behaviors that can change
under ELAI (the corpus is honest about cloud-lane "action-stream not tokens"). The strategic
hedge is exactly J-30/I-49 (BYOK, multiple lanes) plus keeping every guarantee
capability-gated and honestly downgraded (J-14 tiers) — designed, not yet live.

R5. LOCAL-LANE AGING. Qwen2.5-Coder-7B was the right 2025 pick; by 2027 the floor moves
(Qwen3-class, MoE-on-consumer-VRAM). J-9's catalog-as-data is the right shape; what is
missing is a standing cadence (quarterly model-refresh evaluation as a recurring checkbox,
not a one-off rejection memo).

R6. NO BUYER-SIDE LAYER. If the wedge is trust, the natural buyer is a team/org (the entity
that bears liability), and ELAI has no multi-user, policy-distribution, or fleet-audit
story. Post-measurement, this is the biggest unplanned surface. (Deliberately out of current
scope — but the research should test whether procurement pressure makes it urgent.)

R7. VERIFICATION DEMAND TIMING. Bear case: models get good enough on routine edits that
teams stop caring about per-step proof, and verification demand stays niche/compliance-only.
I think this is wrong for high-stakes code (the delegation argument in 3.1), but it is the
single assumption most worth stress-testing externally — hence prompt 2 section 6 and
prompt 3 section 6.

================================================================================
5. WHAT I WOULD CHANGE / ADD (pre-research; to be revised after results)
================================================================================

P0 — RUN THE NUMBER. Execute I-43 (claim scoper + Phase-0 kill-gate pilot at N=50-100), then
F-2b live, then G-6 SAFEDELTA. Nothing else in any letter matters more. If the number is
null, the strategy needs re-aiming and it is better to know in weeks than after 200 more
sessions.

P1 — SHIP THE WEDGE ARTIFACT. I-40 (`elai wrap` + open .rra spec) promoted to the front of
the queue. It is the distribution answer, it needs only shipped substrate (G-1/G-2/G-9 +
gates), and it converts every competitor's user into a potential ELAI user.

P2 — COCKPIT OVER CANVAS. G-5 + H-52 + F-4 vouching as the flagship UX, with the METR-style
within-subjects review-time study as its measurement. Canvas Track 2 continues as the
differentiated long bet but stops being implicitly co-equal.

P3 — ACTIVATION OVER AUTHORING. The feature map shows a large DARK/PARTIAL inventory whose
designs are done. A wiring-audit-driven activation sweep (H-style FINISH pass for D/E/H)
beats authoring K-features. Adopt a standing ratio: no new letter until the previous
letter's keystones are WIRED-or-killed.

P4 — A GATE-TAX BUDGET. One new small unit: measure composed PreToolUse+audit latency and
composed FP rate across all default-ON gates per release, with a locked ceiling. This is the
R3 mitigation and currently no plan owns it end-to-end (GD-4 bounds gates individually).

P5 — MODEL-REFRESH CADENCE. Make J-9's catalog a quarterly evaluated checkbox (local lane
candidate vs incumbent on ELAI's own fixtures) so the local lane never silently ages.

P6 — DEPRECIATION TAGS ON EFFICIENCY WORK. Tag H-Track-1/Track-14 units as
provider-absorbable; before building each remaining one, check whether the provider now
ships it natively (the research prompts will refresh this).

These are directional, not yet a plan. The K-plan (or whatever letter) gets authored only
after the user agrees post-research, per the agreed workflow.

================================================================================
6. WHAT THE RESEARCH MUST SETTLE (mapping prompts -> open questions)
================================================================================

Prompt 1 (market direction) -> tests 2.5/2.6: which interaction paradigm retains; what got
absorbed; whether review-bottleneck is the consensus pain; whether anyone else is building
evidence-bound review (if yes, ELAI's window is shorter than assumed).

Prompt 2 (verification as a market) -> tests 3.(1,2,4) and R6/R7: is there willingness to
pay; are procurement/regulatory drivers real on a 12-24mo horizon; is anyone shipping
provenance/attestation for agent code; does the self-grading conflict actually leave the
independent layer open.

Prompt 3 (moat/economics) -> tests 2.2, R4, R5, R7: the absorption record; the bitter-lesson
evidence both ways; open-weight gap and the hybrid-lane viability; whether heavy gating
loses to lighter-touch competitors on perceived productivity.

Round-2 reasoning will be appended/companioned to this file once the three reports are back.
