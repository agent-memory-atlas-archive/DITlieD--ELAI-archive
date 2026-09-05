vision + orchestration improvements for ELAI — reasoning log
date: 2026-06-23
sources: (A) C:\Users\user\Downloads\deep-research-report(3).md (tiled image->text for text-only LLMs);
         (B) sonnet agent fugu-research (Sakana "Fugu" + arXiv:2512.04695 + arXiv:2512.04388) — PENDING, appended below when it returns.
posture: AS-SPECIFIED (all plans considered built). every improvement is grounded in the cited source + the existing ELAI feature-map entry it composes with. external figures are DESIGN-rationale only (HD-1/ID-5); any ELAI done-gate number is ELAI-measured.

================================================================================
PART A — VISION PIPELINE (deep-research report -> J-1/J-8 improvements)
================================================================================

META-FINDING (the important one)
The report's bottom line IS ELAI's existing perception doctrine, independently arrived at: deterministic-first (OCR + layout + detectors own text/coords/counts), small VLM only on the semantic residue, fuse into strict JSON with normalized 0-1000 coords, hand the downstream model a STRUCTURE not a prose blob. ELAI already encodes the spine of this (J-1 gateway, J-8 VR-2 0-1000 coords, VR-3 counts-from-AXTree-never-VLM, B-24 Tier-1 pixel-diff veto, probe-small-escalate). So the report is CONFIRMATION of direction, not a redesign.
The leverage point: the report says faithful vision REQUIRES evidence-grounding, per-element provenance, a canonical coordinate frame, and a layout graph for the blind consumer. ELAI ALREADY OWNS all four primitives in other subsystems (H-53 citation pre-binding, G-1 taint Provenanced<T>, RRA transform/receipt ledger G-2, the DKG graph engine). The improvement is not "build a vision system" — it is "route the perception scaffold THROUGH the existing trust spine." That is the on-brand synthesis and the highest-value takeaway.

CONCRETE IMPROVEMENTS

I1  TILE-SIZE table distinct from the resize-sweet-spot table.
    J-8 sweet_spots.toml currently keys a single longest-side resize per target model (1568 Claude / 2048 GPT / 1280 Qwen-VL). The report distinguishes WHOLE-IMAGE resize from PER-TILE size and they are not the same number. Add a tile_px column keyed to the DESCRIBER family: 448 for InternVL-family, direct 768/1024 (no tiling first) for FastVLM-family, native dynamic-resolution for Qwen2.5/Qwen3-VL. Grounding: FastVLM (citeturn30/31) "raising resolution essential, AnyRes tiling not automatically optimal, fewer tiles better"; InternVL 448-tile + use_thumbnail reference path. Touch: data/perception/sweet_spots.toml + sweet_spot.rs (J-8 task 1).

I2  Tile overlap value + owner-tile policy.
    J-1 leg-1 has a tile_overlap field with no value. Set 10-15% (12.5% single-number default) as a BOUNDARY-REPAIR heuristic, never the main resolution strategy. Pair it with an owner-tile assignment: every OCR/detector box is owned by the tile containing its CENTER; overlap tiles only rescue truncated boxes in the overlap band. Grounding: report "What the evidence says about tiling" + "merge and de-dup" (owner-tile policy). Honest caveat the report itself flags: exact best overlap % is NOT literature-settled — ship 12.5% as an engineering default, make it config, measure on the J-8 fixture.

I3  NEW — typed merge graph (fusion algorithm). GENUINE GAP.
    Neither J-1 nor J-8 specifies HOW per-tile detections fuse. The report's answer: a typed merge graph words->lines->blocks->semantic-regions where only ONE level contributes to counts, merge by IoU + text similarity + class agreement, counts/exact-text always from deterministic detections never captions. Deterministic check: no two final regions of the same type exceed an IoU threshold unless their texts meaningfully differ. This kills double-counting (the report's "fifth failure"). Touch: a new perception/fusion.rs leg under J-8. This is TD-3-clean (pure deterministic) and fits ELAI's "counts are deterministic" law.

I4  NEW — canonical coordinate frame + transform ledger. GENUINE GAP, perfect ELAI fit.
    Define ONE canonical pixel space at ingest; every resize/crop/tile records its transform RELATIVE to that frame; reproject all boxes back and REJECT any element whose inverse transform is inconsistent. The report's "third failure" (coordinate drift) is exactly what this defends. This is structurally identical to ELAI's RRA receipt ledger (G-2) and the H-49 transform-receipt idea — a transform ledger IS a reproducible-artifact ledger. So implement it AS an RRA-style ledger, not a bespoke thing. Deterministic, verifiable, on-brand. Touch: perception/preprocess.rs (ledger) + a contract test asserting reprojection consistency.

I5  EXTEND H-53 citation pre-binding + D-16 refusal to PERCEPTION. STRONGEST SINGLE IDEA.
    The report's "evidence-gated generation": the VLM may ONLY describe regions already proposed by OCR/layout/detector; a described element with no backing region id is a hallucination -> dropped or routed to an `unverified_claims` channel. This is EXACTLY ELAI's citation pre-binding (H-53): freeze the ranked evidence set BEFORE generation, [N] is a closed-set membership check, out-of-set -> D-16 refusal, "ghost citations structurally impossible." A "ghost region" is a "ghost citation." So compose H-53/citation_prebind.rs over the perception region set: the deterministic detections ARE the EvidenceAnchorSet, a VLM description referencing a non-existent region fails the closed-set check -> typed refusal/unverified. ZERO new mechanism — pure GD-1 composition. This is the single highest-value improvement: it makes the describe pass faithfulness-gated using machinery ELAI already shipped for text.

I6  Per-element provenance tag via G-1 taint. COMPOSITION.
    The report's output contract gives every atomic item a `source` in {ocr, layout, detector, vlm, merged} + confidence; VLM-written fields separated into semantic_label / appearance_notes / uncertain_visual_guess. ELAI already threads Provenanced<String> at 4 ingest sites (G-1 taint). Add the image-perception path as a 5th provenance ingest site, and make `source` the taint label. Then the downstream model (and the auditor) can see which fields are deterministic-grounded vs VLM-guessed, and the capability_gate can treat vlm-sourced fields as lower-trust. Touch: scaffold.rs schema v2 (J-8 task 2) gains the source/confidence fields; G-1 Provenance constructor at the perception site.

I7  Output a LAYOUT GRAPH for the text-only model, not prose. KEY for the GLM-5.2 / blind-model case.
    The report's most underappreciated failure: a text-only LLM fed accurate PROSE still fails on 2D structure (Qianfan-OCR OCR+LLM baselines score 0.0 on CharXiv vs end-to-end vision; chart/layout collapse). Fix: hand the blind model a layout GRAPH — coordinates, reading order, adjacency, containment, z-order, region descriptors — not a narrative. ELAI is a graph shop (DKG = tree-sitter -> SQLite typed-edge graph). Emit the perception result as a graph in the same shape, with a deterministic consistency check (parent boxes contain children, reading-order edges don't cross impossibly, explicit counts == region-list length). For the pure text-only describer lane: drop the image entirely, hand the graph. This is the concrete answer to "GLM 5.2 is text-only" — it gets a verifiable layout graph, not a blind caption.

I8  RESOLVE J-8's open model EVAL with the report's candidate set.
    J-8 left the describer choice as an open EVAL (dots.ocr vs Qwen3-VL-Embedding, anti-slopsquat-gated). The report ranks them by role (every figure DESIGN-only, ELAI re-measures):
      - best doc parser under compute: PaddleOCR-VL-0.9B/1.5 (TEDS 0.9195 table, RMS-F1 0.844 chart; but ~43.7GB VRAM in the A100/vLLM serving config — server-style, re-measure local).
      - best small general describer: Qwen3-VL-4B (873 OCRBench, 94.9 DocVQA, 83.3 ChartQA) OR InternVL3-2B/8B (GUI grounding, ScreenSpot-V2 81.4 on 8B; clean 448-tile reference path).
      - best latency captioner: FastVLM (measured TTFT 152ms @768/144tok on 1-2B).
      - retrieval adjunct for long docs: ColQwen2/ColPali (ViDoRe 86.6/81.3 nDCG@5, 0.39 s/page index).
      - dots.ocr: strong end-to-end multilingual BUT heavy (78.5GB VRAM benchmarking, ~20min/100pg on a 4090) — DEMOTE from "ship candidate" to "heavy lane only."
    So J-8's `elai bench vlm-eval` should measure Qwen3-VL-4B + InternVL3-2B + PaddleOCR-VL + FastVLM against the canonical E-36 lane, with dots.ocr flagged heavy. All anti-slopsquat-resolved before any name lands in code (JC-7/JC-8).

I9  Refusal-policy additions (extend VR-5).
    Add: micro-text refusal when DUAL-OCR engines disagree materially (not just low single-engine confidence); chart/table refusal unless a dedicated parser extracted a structured representation; the fabrication check (I5). Grounding: report "Faithfulness controls". A dual-OCR path (one modern engine + a second recognizer or doc-VLM verify ONLY on low-confidence spans) is the cheapest stylized/rotated/low-contrast-text defense (report "second failure").

I10 CORRECTION — drop the unverified "instruction-before-image" assertion.
    J-1 leg-3 asserts instruction-before-image as if settled. The report explicitly found NO strong benchmark evidence that instruction-before-image universally improves faithfulness, and says prioritize task-specific SCHEMA design over prompt-order superstition. ELAI's own grounded-by-default rule requires demoting this from an assertion to "unverified folklore, measure if it matters" and leaning on the J-8 VR-2 schema constraint (which the report DOES endorse). Small but it's a grounded-by-default hygiene fix.

I11 Make the done-gate CONCRETE + MEASURABLE (replaces J-1's abstract "answers the fine-text question").
    Adopt the report's acceptance thresholds as the J-1/J-8 fixture done-gate (ELAI-measured, per-config, not the report's numbers): >=98% exact-match OCR on legible text above a min font size; >=95% region-grounding recall @ IoU0.5 for text blocks + major controls; <1% duplicate final regions after fusion; >=90% exact-count accuracy for <=10 separated elements; ZERO uncited claims in the final schema (every element maps to deterministic evidence or is marked unverified — this is the I5 gate made measurable); downstream blind-LLM QA materially beats an OCR-only baseline on chart/diagram/layout questions. This turns "the gateway arm answers the question the control fails" into a 6-metric measured gate, which is exactly ELAI's SAFEDELTA-style "believe-the-number" posture.

PRIORITY ORDER (cheapest + highest-leverage first)
1. I5 (citation-prebind over regions) + I6 (taint provenance) — pure composition, reuse shipped machinery, biggest faithfulness win.
2. I7 (layout graph for blind models) — directly answers the text-only-model concern; reuse DKG graph shape.
3. I4 (canonical frame + transform ledger as an RRA-style artifact) — deterministic, verifiable, on-brand.
4. I3 (typed merge graph) — fills a genuine fusion gap.
5. I8 (resolve the EVAL candidate set) — unblocks J-8 Tier-2.
6. I1/I2/I9/I11 (tile sizes, overlap, refusal, measured gate) — table/config/threshold refinements.
7. I10 (drop instruction-before-image folklore) — hygiene.
None of these is a new plan letter; all are J-1/J-8 refinements + cross-links to H-53/G-1/G-2/DKG. paper-to-plan PHASE 5 would fold them as J-1/J-8 task additions + an ELAI-FEATURE-MAP upsert, not new files (FEWEST FILES, GD-1).

================================================================================
PART B — ORCHESTRATION (Sakana "Fugu" + arXiv:2512.04695/04388) -> ELAI
================================================================================
source: sonnet agent fugu-research (returned 2026-06-23; all URLs fetched live, arXiv IDs resolve).

WHAT FUGU ACTUALLY IS (the user's "model to orchestrate other models" — CONFIRMED)
Sakana AI Fugu, GA 2026-06-22 (sakana.ai/fugu/). A LEARNED orchestrator in two tiers:
 - Fugu = TRINITY (arXiv:2512.04695, ICLR 2026): a 0.6B coordinator + a 10K-param linear routing head, optimized by separable CMA-ES (evolution, not RL). Per turn (<=5) it picks ONE LLM from a 7-model pool and assigns it a role Thinker/Worker/Verifier, routing on the coordinator's HIDDEN STATE — it emits zero text, all generation is in the selected worker.
 - Fugu Ultra = the Conductor (arXiv:2512.04388, ICLR 2026): a 7B model trained with GRPO that GENERATES a natural-language workflow spec as three lists — model_id / subtasks / access_list (who sees which prior outputs) — building chain/tree/debate/recursive topologies per query; ~3 steps avg; can recurse on itself.
Benchmarks (provider-reported, NOT independently verified — buildfastwithai.com): Fugu Ultra SWE-Bench Pro 73.7 / LiveCodeBench 93.2 / GPQA-D 95.5, claimed top on 10/11.

THE 5x-SLOWER/5x-PRICIER CLAIM — REFUTED as stated (grounded-by-default; do not repeat the user's premise as fact).
 - No primary/secondary source uses "5x slower AND 5x pricier."
 - The Conductor paper's OWN figure: 735.2 tokens/sample vs a 5x-majority-consensus 1412.8 — i.e. Conductor is MORE efficient than naive ensembles, cost-adjusted 103.49 vs 42.94.
 - One real build test (paddo.dev 2026-06-23): Fugu Ultra 22min/~89K tok/$7.32 vs Opus 4.8 79min/~940K tok/$37.85 on the same task — Fugu CHEAPER + FASTER, Opus won on quality.
 - What IS real: qualitative slowness ("extremely slow", "not a day-to-day workhorse", paddo.dev) from multi-turn sequential calls + a per-token orchestration markup ($5/M in, $30/M out). So the thing to beat is NOT cost — it is the learned router's novel-task topology discovery, while ELAI keeps the reproducibility + zero-orchestration-token edge.

CORE COMPARISON — the user's thesis ("we can beat it with pure orchestration") is LARGELY VINDICATED by the Conductor paper's own ablation.
The agent's synthesis (grounded): most of Fugu's gain comes from THREE primitives, all deterministic-expressible, and the token-efficiency win is "a routing-discipline victory, not a neural-only phenomenon":
 1. targeted role assignment (Thinker/Worker/Verifier)        -> ELAI AgentPool 8 roles + generator/evaluator split (I-21) + auditor-as-Verifier (AUDITING mandatory) + H-7 calibrated role routing. ALREADY-SHIPPED.
 2. selective context windows (access_list)                  -> ELAI scoped-context discipline (orchestrate/auto-orchestrate "a worker receives ONLY what its task needs") + CONTEXT BUDGET ALLOCATOR (8 slots) + PRISTINE CONTEXT INJECTOR (I-5, hard-exclude failed-sibling tracebacks) + auditor-objectivity firewall (auditor gets ZERO memory). ALREADY-SHIPPED as a discipline.
 3. pool-heterogeneous dispatch                              -> MULTI-RUNTIME BACKEND (B-37, 11 backends) + PerRoleRuntime + E-31 tier matrix + J-55 modality router. ALREADY-SHIPPED/PLANNED.
And TRINITY's "coordinator emits ZERO tokens, all generation in workers" IS ELAI's main-llm-role rule (strategist-never-executor) — except ELAI's router is a PURE FUNCTION (zero model tokens at all), so ELAI is STRONGER: it doesn't even pay for the 0.6B coordinator.

WHERE A LEARNED ORCHESTRATOR HONESTLY WINS (do not hand-wave this)
Genuinely novel / unclassifiable tasks where the task-type can't be keyed upfront: the Conductor DISCOVERS non-obvious topologies (recursive sub-delegation, debate) a static dispatch table can't represent without enumerating every case. GPQA-Diamond +5.2 over best single model is the exhibit. A pure table loses there.
ELAI's deterministic-but-adaptive answer: the EVIDENCE-RANKED SEARCH LADDER (I-4) — when routing confidence is low, run best-of-N -> tournament -> PUCT over the SessionDag and SELECT BY RECEIPTS, not by a learned prior. A receipt-selected topology is reproducible (RRA) and grounded in measured outcome, which a learned router is not. So ELAI matches the novel-task case by MEASURING candidates, not by training a router.

ORCHESTRATION IMPROVEMENTS / STEAL VERDICTS

O1  EXTENSION(I-5) — formalize access_list as a first-class, journaled per-dispatch CONTEXT-ACCESS SPEC. STRONGEST STEAL.
    ELAI has scoped context as an orchestrator HABIT; the Conductor makes "who sees which prior outputs" an explicit, declared list. Turn it into a declared, auditable per-agent access_spec (which prior SSB rows / sibling outputs this dispatch may read), composing PRISTINE CONTEXT INJECTOR (I-5) + the auditor firewall + the CONTEXT BUDGET ALLOCATOR slots. Deterministic (TD-3-clean), journaled (GD-2 recomputable), and it upgrades a habit into a verifiable contract. This is the cleanest genuine net-new from the whole intake.

O2  CONFIRMATION — Thinker/Worker/Verifier triad + always-end-with-Verifier at a <=5-turn cap.
    ELAI already has generator/evaluator (I-21) + mandatory AUDITING (Verifier) + BSP/cascade caps. The empirical finding (3 steps default, 5 ceiling; quality doesn't collapse at the cap) GROUNDS ELAI's existing turn/agent caps. No build; cite as external confirmation of the cap discipline.

O3  RIDER/grounding for I-4 — the search ladder IS ELAI's answer to novel-task topology discovery.
    Fugu's only honest edge (novel/unclassifiable decomposition) is exactly what I-4 (select-by-receipts best-of-N -> tournament -> PUCT) addresses deterministically. Record the Conductor's GPQA +5.2 as the GROUNDING for WHY I-4 matters (a measured-topology beats a learned-prior topology on reproducibility), not a new unit. Recorded NOT authored (GD-1).

O4  POSITIONING/measure-the-number — Fugu proves the orchestration-as-product market AND that it can be cheaper than a single frontier model. ELAI's deterministic orchestration is structurally cheaper still (zero orchestration-model tokens vs Fugu's $5/$30 markup) IF it matches quality. The gap: ELAI's quality number is BENCH-PENDING (SAFEDELTA G-6 / E-31 coding-lane, RunPod-gated). ACTION: the head-to-head needs the measured ELAI number; until then the cost-edge claim is grounded but the quality-parity claim is not. This is a GTM/bench priority, not a code change.

O5  REJECT — TRINITY latent hidden-state routing.
    Routing on the coordinator's hidden state requires white-box access to the worker models' internals. ELAI's cloud wedge (Anthropic/OpenAI text APIs) has NO hidden-state access — same structural ineligibility that rejected RecursiveMAS latent inter-agent comms. The PORTABLE arm is the text-level role/access/topology spec (O1/O2), never the latent mechanism.

O6  CONFIRMATION + ELAI-ahead — "coordinator emits zero tokens, all generation in workers" = main-llm-role.md (strategist-never-executor). ELAI's router is a pure function (zero model tokens), strictly cheaper than even a 0.6B coordinator. No steal; note ELAI is ahead.

PAPER SUMMARIES (verified)
 - arXiv:2512.04695 TRINITY: An Evolved LLM Coordinator (Sakana + UMich + IST). 0.6B coordinator + 10K linear routing head, separable CMA-ES, role(Thinker/Worker/Verifier) + model selection per turn on hidden state, label-free end-to-end. LiveCodeBench V6 86.2 vs GPT-5 59.5; approaches the per-question-best oracle. Steal: role taxonomy + 5-turn cap + zero-token-coordinator discipline + pool heterogeneity (all deterministic-expressible).
 - arXiv:2512.04388 Learning to Orchestrate Agents in Natural Language with the Conductor (Sakana). 7B GRPO, emits model_id/subtasks/access_list -> dynamic topology; pool-randomized training => model-swap robust; ~3 steps; recursive +2.2 BigCodeBench. GPQA-D 87.5 vs best single 84.8; 735 tok/sample vs consensus 1412. Steal: the access_list primitive (O1), task-type-keyed (not model-name-keyed) dispatch => free model-swap robustness, 3-step default.

NET: the user's instinct is correct — ELAI's pure deterministic orchestration already implements the primitives Fugu learns, is reproducible + free where Fugu is a learned + paid markup, and answers Fugu's one real edge (novel-task topology) with the receipt-ranked search ladder (I-4) rather than a trained router. The only genuine code steal is O1 (formalize the access_list contract); the rest is confirmation, grounding, a rejected latent arm, and a measure-the-number GTM action.

================================================================================
PART C — INTERLACE GRILL WITH THE DETERMINISTIC AMBIGUITY SIGNALS (user observation 2026-06-23)
================================================================================
OBSERVATION (user): grill is user-invoked-only and disconnected from Channel-A's automatic ambiguity machinery (J-32 confidence / B-38 ambiguity score / B-40 critic / C-29 intent-pair count). It should be interlaced.

THE INVARIANT TO PRESERVE (grill SKILL.md HARD CONSTRAINT): "NEVER auto-fire grill off a keyword or off the shape of a task ... an interrogation the user did not ask for is friction, and ELAI's whole anti-sycophancy posture is undercut if the assistant starts cross-examining unprompted." Auto-RUNNING grill is banned. BUT the same file already sanctions the OFFER: "if a task looks fuzzy and you think grilling would help, OFFER it in one line ('want me to grill this before i plan it ?') and wait." So the line between banned and allowed is the CONSENT step, not the trigger.

THE GAP: today the offer fires on the main LLM's vibe ("if you think"), NOT on the measured signals ELAI already computes. The signals BLOCK or chip; grill RESOLVES; nothing wires detector -> resolver. So a high-ambiguity task either gets a single J-32 CLARIFY chip (too thin for a multi-branch ambiguity) or silently proceeds on the LLM noticing.

IMPROVEMENT G1 (EXTENSION; composes J-32/B-38/B-40/C-29/grill; the clean interlace, INTERACTIVE mode)
Make the grill OFFER signal-triggered, keep consent. A CLARIFY LADDER with the rung chosen by a deterministic signal:
 - rung 0 (single missing slot): J-32 low-confidence on ONE arg_span -> the existing single CLARIFY chip. unchanged.
 - rung 1 (structural/multi-branch ambiguity): B-38 ambiguity score > threshold ACROSS multiple decision branches, OR B-40 idea-critic returns BLOCK, OR C-29 cannot reach >=3 intent-pairs for a step -> OFFER grill in one line ("this looks ambiguous across N branches — grill it before i plan ?") and WAIT. consent preserved, so the user-invoked-only invariant holds; only the TRIGGER moved from vibe to a measured signal.
The deepest half of the interlace is the RETURN path: grill's structured output is exactly what those gates need to pass. grill RESOLVED DECISIONS + GLOSSARY -> lowers the B-38 score (pinned terms) ; grill's per-branch resolutions -> the >=3 C-29 intent-pairs ; grill CONTRADICTIONS RESOLVED -> clears the B-40 critic BLOCK. So grill becomes the structured UNBLOCK path for the gate that detected the problem, instead of a parallel manual route the user has to think to use. detector (B-38/C-29/J-32/B-40) -> offer grill -> grill resolves -> gate passes. That is the interlace.

IMPROVEMENT G2 (the AUTONOMOUS-mode half; composes AutonomousGate.decide() + grill)
In autonomous mode there is no human to grill, so the offer cannot fire mid-run. But the autonomous gate ALREADY escalates to a human on HIGH risk / failed evidence / anti-spiral (decide() escalation arms). Today that escalation is a bare "blocked, your call." Improvement: when the escalation CAUSE is ambiguity-grade (a B-38/B-40/C-29 evidence-check failure, not a risk/cost tripwire), the escalation surface OFFERS grill as the resolution path rather than dumping a raw question. So grill becomes the human re-entry tool exactly at the gate's escalation moment — interlacing it with Channel B, not just Channel A.

WHERE THE CURRENT DESIGN IS RIGHT (do not over-correct)
The reason grill is user-invoked-only is sound: auto-RUNNING an interrogation is the friction the anti-sycophancy rule guards against. G1/G2 must NOT cross into auto-run. An over-eager OFFER is itself a mild version of the same friction (a chip that fires on every task trains the user to dismiss it). So the threshold is the risk, and it gets the GD-4 discipline: the signal->offer trigger ships default-OFF / conservative, and the FALSE-OFFER rate (offer fired on a task the user found perfectly clear) is measured on >=3 real runs and bounded before the threshold is tuned up. Same posture as I-19's FP-bound. An offer that cries wolf is a regression, not a feature.

RULE-TEXT SHARPENING (cheap, do first)
grill SKILL.md's HARD CONSTRAINT should be sharpened to distinguish the two cases it currently blurs: "never auto-RUN grill" (the real ban) vs "signal-trigger the OFFER" (G1, explicitly allowed, consent-gated). As written, a literal reader could think any signal-driven behavior violates the constraint; the offer is already sanctioned later in the same file, so the constraint wording is the only thing in the way.

VERDICT: G1 is an EXTENSION wiring four shipped/planned signals (J-32/B-38/B-40/C-29) into the shipped grill offer — no new mechanism, just the missing wire + a measured threshold (GD-4). G2 extends the autonomous-gate escalation surface to name grill as the resolution path. Both preserve the user-invoked/consent invariant. Highest-value, lowest-risk: the rule-text sharpening + G1 rung-1 offer on a B-40 critic BLOCK (the strongest, least-noisy signal) first; widen to the B-38 score only after the FP rate is bounded.

================================================================================
OUTCOME (2026-06-23, paper-to-plan PHASE 5)
================================================================================
Authored J-77 perception faithfulness via the trust spine (vision PART A I1-I11 bundled into ONE file; EXTENDS J-1/J-8; composes H-53/G-1/G-2/DKG — the source doc's "fold into J-1/J-8 task additions" became a NEW file per the house never-amend-old-plans rule) + J-78 context-access spec (PART B orchestration O1, the only genuine Fugu code-steal; EXTENDS I-5; O5 TRINITY latent routing rejected; O2-O6 recorded as confirmations/grounding/reject) + J-79 signal-triggered grill interlace (PART C G1/G2 + the rule-text sharpening; EXTENDS J-32/B-38/B-40/C-29/grill/AutonomousGate). Riders + the Fugu "5x" refutation recorded in the J-0_INDEX DECLINES + the ELAI-FEATURE-MAP Section-4 ledger. J-0_INDEX + J-SUMMARY_INDEX + ELAI-FEATURE-MAP upserted (date 2026-06-23). Plan family J now J-1..J-79. ADDS no delta family.
