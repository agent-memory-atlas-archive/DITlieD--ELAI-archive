J-PLAN AUTHORING REASONING — fill the sdl-mcp/Caliber gaps, research-grounded
Authored: 2026-06-26
Purpose: the input a LATER session feeds to paper-to-plan PHASE 5 (plan authoring) to
  create the NEW J-plan files this analysis concludes are warranted. This file does the
  high-judgment work (verdict + composition + doctrine binding + done-gate shape); the
  authoring session writes the J-N files + index updates + feature-map upsert from it.
Lens: spec-eval AS-SPECIFIED (ELAI plans treated as built; wiring is sequencing, not a mark).
Scope: ELAI-the-PRODUCT at F:\.ELAI_workflow\.elai_cc only. Technical->J split (the K-native
  PROVE+PACKAGE+DISCIPLINE halves are out of scope here; this is the J technical track).
Target family: J-plan. Live numbering is J-1..J-102 (ELAI-FEATURE-MAP header, 2026-06-26g).
  New files start at J-103. Confirm the next free integer against J-0_INDEX.md at authoring.

INPUTS GROUNDED THIS SESSION
- workflow/reasoning/elai-gaps-sdl-ai-setup-2026-06-26.md (the gap list G-1..G-7 + 2 carry-fwd).
- Downloads/deep-research-report(11).md — grounds G-3 (code-graph memory ranking), G-7 (memory
  ROI/attribution), G-4 (deterministic grounding/freshness). Read in full.
- Downloads/deep-research-report(12).md — grounds G-1 (graduated disclosure), G-2 (tool-call
  mediation), G-5 (context economy), G-6 (live indexing). Read in full.
- docs/thefinalglance/ELAI-FEATURE-MAP.md — gap-relevant sections read in full (1.1 retrieval,
  1.2 reasoning/GRC, 1.5 context/memory, Section 3 doctrine, Section 5 extension points) + every
  G-1..G-7 composition anchor (J-78, J-28/J-29, J-25, J-34, J-21, J-38, I-46, rules engine
  phase-3F, B-29, B-31, E-5, H-31/32/33, C-45, K-18, H-43, H-39, I-32/I-47). The feature map is
  the consolidated source that REPLACES the legacy 12-file summary read (header line 1-3); the
  B..K SUMMARY_INDEX files were not re-read (superseded + flagged outdated by the user).

================================================================================
0. THE HEADLINE (what the research changed vs the gaps file)
================================================================================
The gaps file's ranking survives almost intact, with ONE research-driven downgrade and one
sharpening:
- G-1 + G-2 (graduated proof-of-need read-access ladder + Read-tool interceptor) are the
  research-STRONGEST cells. Report(12) §1/§2 give hard measured wins (MutaGReP 95.7% fewer
  tokens for a 4.8pt drop; YoloFS 55% fewer prompts at no success cost; HARD interception beats
  soft). They merge into ONE plan unit. CONFIRMED #1.
- G-3 (code-graph centrality ranking of memories) is DOWNGRADED. Report(11) §1 measures it
  MARGINAL: Mem0 graph-memory +2% relative; Codebase-Memory 83% vs 92% quality at 10x token
  saving and ONLY wins on structural/architectural queries; NO public result shows centrality
  ranking beats semantic+recency on general tasks. So G-3 must NOT be a headline default-ON
  ranker. It folds into the memory plan as a default-OFF, observe-only signal applied ONLY to
  graph-shaped queries. This is grounded-by-default in action: marginal evidence => do not act
  on it as if strong.
- G-7 (memory ROI / per-learning attribution) is SHARPENED. Report(11) §2: curated memory lifts
  +28.5pp on hard tasks / +30% Memory-R1, but NAIVE accumulation DEGRADES (Xiong 2025) and
  poisoning succeeds 95% (Schneider 2026); Counterfactual Composition Testing identifies the
  single causal memory entry at 87.5% (Ahad 2026). J-21 already covers the poisoning/quarantine
  half; the residual NET-NEW is per-entry CAUSAL ROI attribution driving trim/demotion.
- G-5 (context-economy meter) collapses into G-1's done-gate (the would-have-read-vs-actually-
  read delta IS how the ladder proves its value). Report(12) §3's "no end-to-end $/correct-answer
  metric exists" is an opportunity recorded as a rider, not authored (K-18/J-82 bench territory).
- G-6 (live-index overlay + idle checkpoint) survives as a small standalone unit (different
  subsystem: indexing, not read-access). Report(12) §4 grounds it (Cursor Merkle 10-min sync,
  sub-10ms hot queries, small overhead; fixes in-session edit staleness).
- G-4 (grounding-ratio/freshness SCORE) is DROPPED from the product set: gaps file already
  judged it harness-scope + already-covered-in-spirit (elai_map_freshness.py + spec_link_check +
  I-39), and report(11) §3 finds NO published correlation between such static scores and agent
  accuracy. Recorded as a rider only.
- Carry-forward META-COMMENTARY OUTPUT ENFORCEMENT survives as a small real gap (grep-confirmed
  absent product-side). Carry-forward HUMAN-SIM E2E TESTER is real and net-new but bigger and
  orthogonal to this context/memory cluster — deferred to its own intake, not authored here.

NET: FOUR new J files (J-103..J-106), one dropped-with-rider (G-4), one deferred candidate
(human-sim e2e). Fewest-files grouping per paper-to-plan PHASE 5.

UNIVERSAL DOCTRINE BINDING (applies to EVERY file below; do not restate per-file at authoring)
- ID-5 / GD-1 / W1: every paper number in this file (MutaGReP 95.7%, YoloFS 55%, Mem0 +2%,
  Cursor 10-min, +28.5pp, 87.5%) is DESIGN RATIONALE ONLY. The done-gate is ELAI-MEASURED on an
  ELAI fixture/gold set, NEVER a paper constant. State this in each plan's done-gate block.
- TD-3: any policy/gate/ranking decision is DETERMINISTIC; the LLM is advisory, never the
  load-bearing decision. The read-access ladder DOWNGRADES deterministically; memory curation
  trims on execution-grounded evidence; no LLM-as-judge on a gate.
- Vertical-slice PRE_GATE (wiring-enforcement): each unit ships producer + in-plan consumer +
  smoke through a REAL entrypoint in the SAME unit. Bare "add fn X" is rejected at decomposition.
- Any new default-OFF flag needs a wire-dark-allowlist.toml row (type/owner/expiry/
  promotion_plan_id/smoke) or it FAILS R9.
- New SSB action_type = Pattern-2 single-variant-with-Kind + full 9-tuple golden-lock lockstep +
  TOTAL_VARIANTS bump from the LIVE integers (ssb.rs / action_type_lock.rs), never a doc number.
  Prefer-reuse an existing variant where the entry below says so (net 0).
- No sdl-mcp CODE copied (its license bans embedding in a for-sale product; ELAI has K-plan GTM).
  Design ideas only. ai-setup is MIT but its product-relevant ideas are subsumed.
- Per-plan _grounding/<id>-GROUNDING-MAP.md with live grep evidence is the W-7 norm.

================================================================================
1. J-103 — GRADUATED PROOF-OF-NEED READ-ACCESS LADDER + READ-TOOL INTERCEPTOR
================================================================================
GAPS COVERED: G-1 (best steal) + G-2 (Read-tool interceptor) + G-5 (savings meter, as the
  done-gate instrument). Also closes the user's earlier idea-3 ("read FILENAME, ELAI decides how
  much").
VERDICT: NEW (the ladder is a new policy/decision layer) composing many SHIPPED pieces.

THE GAP (from gaps file G-1/G-2): ELAI has every PIECE but never assembled a gated read-
  ESCALATION ladder with downgrade over CODE READS. sdl-mcp's Iris Gate Ladder has four rungs
  Symbol Card -> Skeleton IR -> Hot-Path Excerpt -> Code Window; a raw-source request is gated,
  the agent must NAME the identifiers it expects, and the enforcer verifies those identifiers
  exist in real file text BEFORE returning (post-hoc presence check; a hallucinated symbol
  cannot pull a window). ELAI's gates today are WRITE/TOOL/SECURITY-facing (VibeGate F-1,
  capability_gate, F-3 tool filter), not READ-ARTIFACT-facing. I-46 intercepts bare bash
  (grep/cat/find -> refuse-with-redirect) but there is NO interceptor on the model's first-class
  Read tool that steers a naive full-file read to symbol-card/skeleton retrieval first.

RESEARCH GROUNDING (report(12) §1/§2/§3 — DESIGN RATIONALE, not done-gate):
- MutaGReP plan-search: 5,473 tokens (4.3% of a 121k repo) reaches 53.9% best-of-5 vs 58.7% on
  full repo => 95.7% fewer tokens for a 4.8pt drop. CodeRay snippet lookup: 2-3.4x token
  reduction avg, up to 6x on large files (vendor, flagged). The graduated-disclosure thesis is
  measured and strong.
- BRITTLENESS (the mandatory design constraint): CrossCodeEval shows over-aggressive pruning
  FLIPS correct answers to wrong when the skeleton misses a crucial detail (StarCoder Python:
  327 correct->wrong as 468 went the other way). => the ladder MUST verify identifier presence
  and ESCALATE-on-miss (climb a rung), never silently serve a too-thin rung. This is exactly
  sdl-mcp's enforceCodeWindow post-hoc check; ELAI gets the same from a deterministic context-
  sufficiency gate (J-38) over C-29 IntentPair symbol-coverage.
- HARD > SOFT interception (report(12) §2): YoloFS effect-level VFS hook = 99% success at 0.4
  prompts/task vs 98%/0.9 baseline (55% fewer prompts, no success hit). SOFT steering (system-
  prompt nudge, ignore-lists) gets BYPASSED — Gemini-CLI's search ignores the ignore-list and
  the agent falls back to plain shell cat. => the interceptor must be a HARD tool-boundary gate,
  not a prompt nudge. ELAI's I-46 is already hard (refuse-with-redirect); widen the same
  mechanism to the Read tool.
- ECONOMY (report(12) §3): production agents ~100 input tokens per output token, so each
  extraneous read token is expensive; saving 90-95% of context typically costs only a few % of
  accuracy. The savings meter quantifies this per session.

ELAI COMPOSITION (all SHIPPED unless noted):
- THE RUNGS: DKG symbol cards (phase-1A) + B-29 symbol-level impact API (4 traversal tools:
  impact_radius/call_flow/symbol_context/list) = the card/skeleton rungs. Adaptive AST chunking +
  FAST/STANDARD/DEEP tiers (E-17/phase-3K) + J-25 section-aware budgeted read = the hot-path-
  excerpt rung. Full file = the top rung (Code Window).
- THE POLICY LAYER: the rules engine (phase-3F: predicates AST/GREP/SCHEMA/METRIC/COMPOSITE,
  verdicts ALLOW/WARN/BLOCK/ESCALATE, rules-as-DATA-in-SQLite) is the decision substrate. ADD a
  READ-ARTIFACT predicate family that returns approve / DOWNGRADE-to-rung-N / deny.
- THE DECLARED SCOPE: J-78 context-access spec (PLANNED) — the dispatch DECLARES which symbols/
  rows it may read; the ladder enforces the declaration. J-78 is the natural seam; this unit may
  ship alongside or depend on J-78.
- THE INTERCEPTOR: extend I-46's bash_validation.rs refuse-with-redirect mechanism to the Read
  tool (PLANNED I-46 widened). A naive full-file Read is intercepted -> redirected to the
  cheapest rung that satisfies the declared identifiers, with the agent required to NAME the
  identifiers it expects (sdl-mcp's contract).
- THE PRESENCE CHECK + ESCALATE-ON-MISS: J-38 deterministic context-sufficiency gate over C-29
  IntentPair symbol-coverage. If the served rung does not contain the named identifiers (or
  coverage < threshold), CLIMB one rung (downgrade-not-deny in reverse). This is the CrossCodeEval
  brittleness mitigation. TD-3: the climb is deterministic.
- THE METER (G-5, the done-gate instrument): record would-have-read-tokens (full file) vs
  actually-served-tokens at every rung decision; surface a live session "saved X vs naive read"
  delta. EXTENDS J-29 usage window / live meter; consumes CatRAG's existing 55-87% token-savings
  accounting (phase-1E). Read-side, prefer-reuse — net 0 new SSB if it rides J-29's read-side
  rows; +ReadAccessDecision (Pattern-2 with Kind = {Approve,Downgrade,Deny,EscalateOnMiss}) only
  if a journaled decision row is wanted for replay/audit (recommended: it makes the ladder's
  behavior recomputable, GD-2).

VERTICAL SLICE: producer = the read-access predicate + ladder router; consumer = the Read-tool
  interceptor call-site (the real entrypoint where the model's Read is dispatched) + the J-29
  meter surfacing; smoke = drive a real read request that names identifiers, assert the ladder
  serves the cheapest satisfying rung AND that a request naming a hallucinated/absent identifier
  is escalated (climbs) rather than served a thin rung.
DONE-GATE (ELAI-MEASURED, ID-5): on an ELAI gold set of read requests, measure (a) token
  delta served-vs-full, (b) task-outcome parity (gate verdicts, not LLM-judge) at the reduced
  rung, (c) zero "served a rung missing a named identifier" cases (the brittleness guard). NEVER
  cite MutaGReP/YoloFS numbers as the gate.
DETERMINISM: TD-3 — the rung decision and the escalate-on-miss are deterministic; the LLM only
  NAMES the identifiers it wants, it does not adjudicate the rung. Downgrade-not-deny is the
  default posture (deny only on a security/capability_gate overlap, not on economy).
FLAG: default-OFF first behind ELAI_READ_LADDER_ENABLE (+ allowlist row, promotion_plan_id =
  J-103-promotion), observe-only (GD-4 / GD-3: no gate flips default-ON until FP/over-block
  bounded on >=3 real runs). The interceptor must NOT block a legitimately-needed full read —
  over-block is the YoloFS failure mode (Gemini "deny outside project" -> 75% task failure via
  no-action). Bias to DOWNGRADE + ESCALATE-ON-MISS, never hard-deny on economy grounds.
CROSS-LINKS: composes DKG/phase-1A, B-29, E-17/phase-3K, J-25, J-38, C-29, J-78, I-46, rules
  engine/phase-3F, context_budget.rs/phase-3E, J-29, CatRAG/phase-1E. EXTENDS I-46 + J-78.

================================================================================
2. J-104 — LIVE-INDEX OVERLAY-BEFORE-BASE READ + IDLE CHECKPOINT
================================================================================
GAPS COVERED: G-6.
VERDICT: EXTENSION of B-31 (resumable indexing + lock lifecycle + blake3 incremental update +
  IndexingService). Small.

THE GAP: sdl-mcp queries an in-memory OVERLAY of recent edits BEFORE the base graph so edits are
  visible immediately without a full re-index, and an IdleMonitor drains the overlay to durable
  state during idle (overlay-reader.ts, coordinator.ts, idle-monitor.ts). ELAI's B-31 is
  incremental (blake3 CASCADE) but BATCH — it does not do overlay-first-read for instant in-
  session edit visibility, so an agent that just edited a file can retrieve stale symbols until
  the next incremental pass.

RESEARCH GROUNDING (report(12) §4 — DESIGN RATIONALE):
- Cursor uses a Merkle-tree diff: hashes recomputed ~every 10 min, only changed chunks re-sent;
  hot queries ~8-10ms, cold ~500-600ms. Staleness is BOUNDED by the sync interval — exactly the
  in-session-edit-not-yet-indexed failure the overlay fixes.
- YoloFS-class union/snapshot systems commit 100k files in ~3.5s with unchanged single-file I/O
  => maintaining a small overlay + idle drain is cheap. The overhead is small; the win is the
  agent never reads a symbol it just deleted/renamed.
- HONEST CAVEAT (report(12) §4): no public "accuracy vs staleness" benchmark exists; rapid edits
  could confuse retrieval (deleted functions still present) if the overlay is not authoritative-
  on-conflict. => the overlay MUST win over base on conflict, and the drain MUST be atomic.

ELAI COMPOSITION: EXTENDS B-31 IndexingService — add an in-memory overlay layer that captures
  post-edit symbol deltas (the edit already flows through file_ops.rs; reuse that signal), query
  overlay-first then base, and an idle-triggered atomic drain into the durable DKG/blake3 store
  (reuse the existing CASCADE update path; the drain is just an earlier trigger). The idle trigger
  composes with the existing supervisor/watchdog plane (I-17 / J-100 step-no-progress signal) for
  "idle" detection — order-based, replay-stable, NOT wall-clock-gated (TZ-4 / RD: order not
  duration).
VERTICAL SLICE: producer = overlay store + overlay-first read path + idle-drain; consumer = the
  DKG/retrieval read entrypoint reads overlay-first; smoke = edit a file, IMMEDIATELY retrieve,
  assert the new symbol is visible and the deleted symbol is gone BEFORE any batch re-index runs;
  then trigger idle, assert atomic drain leaves the durable store byte-consistent.
DONE-GATE (ELAI-MEASURED): zero "retrieved a symbol the just-completed edit removed" cases on an
  edit->retrieve fixture; drain is atomic (crash mid-drain leaves either old-consistent or new-
  consistent, never partial — reuse B-31 lock lifecycle). NEVER cite Cursor's 10-min number.
DETERMINISM: overlay-wins-on-conflict is deterministic; drain is atomic + replay-safe (GD-2,
  the durable store after drain is byte-identical to a from-scratch index of the same edits).
FLAG: ELAI_LIVE_OVERLAY_ENABLE default-OFF first (allowlist row, promotion_plan_id = J-104-
  promotion); the existing ELAI_DISABLE_INDEXING kill-switch still governs the base.
SSB: prefer-reuse B-31's indexing rows; +OverlayDrained only if a journaled drain checkpoint is
  wanted (net 0..+1).
CROSS-LINKS: EXTENDS B-31; composes IndexingService, file_ops.rs edit signal, blake3 CASCADE,
  I-17/J-100 idle/no-progress signal.

================================================================================
3. J-105 — COUNTERFACTUAL MEMORY-ROI ATTRIBUTION + ROI-DRIVEN CURATION
            (with G-3 centrality as a default-OFF observe-only rung)
================================================================================
GAPS COVERED: G-7 (residual net-new) + G-3 (folded, downgraded to observe-only).
VERDICT: EXTENSION of J-21 + K-18 + H-43 + H-39. Medium. NEW FILE (house rule: never amend the
  already-PLANNED J-21 in place; this EXTENDS it).

THE GAP (G-7 residual): ELAI measures memory value coarsely — K-18 recurrence-hit-rate (how
  often a new failing task matches an existing proven case), H-43 execution-grounded fitness,
  H-39 abstention scoring. J-21 (PLANNED) already does poisoning/quarantine + commit-aware
  semantic validation + retrieval-time authorization + TTL->activation-decay. What is MISSING is
  PER-MEMORY-ENTRY CAUSAL ROI attribution at retrieval grain: which specific memory entry CAUSED
  this session's success or failure, used to TRIM/DEMOTE negative-ROI entries.

THE GAP (G-3, downgraded): sdl-mcp ranks surfaced memories by code-graph CENTRALITY of the
  symbols they touch (PageRank/fan-in). ELAI ranks by trust-tier + recency + supersession (H-31)
  and credit-attribution (C-45), never by code-graph centrality.

RESEARCH GROUNDING (report(11) §1/§2 — DESIGN RATIONALE):
- G-7 is real and high-upside BUT double-edged. Curated/selective memory: Trajectory-Informed
  Memory +14.3pp goal-completion, +28.5pp (149% rel) on hard multi-step tasks; Memory-R1 +30%
  on LoCoMo. BUT Xiong 2025: NAIVE add-all accumulation flattens/DEGRADES performance vs a fixed
  memory; quality-based deletion kept retained memories 72% vs 55% correct => ~20% of unfiltered
  records are noisy enough to mislead. Schneider 2026: memory-poisoning 95% success. Ahad 2026
  Counterfactual Composition Testing identifies the single causal memory entry at 87.5%. => the
  attribution mechanism is the lever that turns memory from net-negative to +28pp; it MUST drive
  trimming, and it MUST be defended (J-21 covers the poisoning half).
- G-3 is MARGINAL. Mem0 GraphMemory +2% relative (LoCoMo 90->92). Codebase-Memory AST graph:
  83% vs 92% answer quality at ~10x token saving, and "matches or exceeds" text search ONLY on
  19/31 languages for hub-detection/caller-fan-in (structural queries); NO study shows structural
  ranking of episodic memories OUTRANKS semantic+recency on a general benchmark. => centrality is
  NOT a default ranker; it is at best an opt-in signal for explicitly graph-shaped queries.

ELAI COMPOSITION:
- ROI ATTRIBUTION (the G-7 net-new): an OFF-AGENT, EXECUTION-GROUNDED counterfactual attribution
  — for a failed/succeeded session, attribute the outcome delta to the specific retrieved memory
  entry/entries (counterfactual-composition style: would the outcome have flipped without entry
  E?). VD-1: fitness is execution-grounded, NEVER self-confidence; TD-3: no LLM-as-judge (the
  research's explicit "LLM-judge masks memory errors" warning). Reuse H-43 execution-grounded
  fitness as the outcome signal, C-45 credit-attribution as the per-entry credit substrate, K-18
  recurrence-hit-rate as the reuse-value signal, H-39 abstention scoring (correct=+1/abstain=0/
  confident-wrong=-t/(1-t)) as the scoring shape. The attribution score DRIVES J-21's TTL->
  activation-decay: negative-ROI entries demote/trim faster; proven-positive entries persist.
- POISONING/VALIDATION (G-7's other half): ALREADY J-21 — do NOT re-author; this unit CONSUMES
  J-21's quarantine + commit-aware validation. EXTENDS J-21, depends on it.
- CENTRALITY SIGNAL (G-3, observe-only): a default-OFF re-ranking rung in the memory retrieval
  path (E-5 retrieval-into-prompt, currently DARK; H-31 retrieval ranking) that ADDS code-graph
  centrality (the SHIPPED PageRank/phase-1D + B-29 fan-in) as a tiebreaker — APPLIED ONLY when
  the query is classified graph-shaped/architectural (reuse D-10 select_grounding_mode-style
  deterministic routing to detect the query shape). Observe-only first; never default-ON (the
  research forbids it). This is the honest way to bank G-3 without overclaiming.
- AUDITOR FIREWALL (carry from H-31, arXiv:2510.09905): auditor/critic/adversary roles get ZERO
  memory injection — the attribution + centrality re-ranking MUST respect this firewall; do not
  surface ROI-ranked memory to objectivity roles.
VERTICAL SLICE: producer = the counterfactual attribution scorer + the ROI->decay driver + the
  default-OFF centrality re-ranker; consumer = J-21's curation/decay path consumes the ROI score
  (trim/demote), and the memory retrieval entrypoint consumes the (gated) centrality tiebreaker;
  smoke = inject a known-bad memory + a known-good memory, run a session whose outcome depends on
  them, assert the bad entry's ROI score is negative and it is demoted/trimmed, and the good
  entry persists; assert the centrality rung is inert unless the query is graph-shaped AND the
  flag is ON.
DONE-GATE (ELAI-MEASURED): on an ELAI memory fixture, (a) attribution correctly identifies the
  causal entry on seeded good/bad memories (ELAI's own accuracy number, NOT Ahad's 87.5%),
  (b) ROI-driven trim improves task outcome vs naive-keep-all (the Xiong degradation reproduced
  and fixed), (c) the centrality rung shows NO regression when OFF and is measured separately
  when ON (do not bank a lift it does not earn). NEVER cite Mem0/Memory-R1/Ahad numbers as gate.
DETERMINISM: TD-3 — attribution is execution-grounded + deterministic; the LLM never scores its
  own memory ROI. The centrality rung is a deterministic re-rank, gated and observe-only.
FLAG: ELAI_MEMORY_ROI_ATTRIBUTION_ENABLE + ELAI_MEMORY_CENTRALITY_RERANK_ENABLE, both default-OFF
  (allowlist rows; promotion_plan_id = J-105-promotion). The centrality flag stays observe-only
  longest given the marginal evidence.
SSB: prefer-reuse H-33 MemoryGovernanceEvent / J-21 MemoryValidationEvent for the trim/demote
  decisions (payload never carries memory content); +MemoryRoiAttributed only if a journaled
  per-entry ROI row is wanted (net 0..+1).
CROSS-LINKS: EXTENDS J-21, K-18, H-43, H-39; composes C-45 credit-attribution, E-5 retrieval,
  H-31/H-32/H-33, PageRank/phase-1D, B-29 fan-in, D-10 query-shape routing, the H-31 auditor
  firewall.

================================================================================
4. J-106 — OUTPUT-SIDE META-COMMENTARY ENFORCEMENT (op-emission roles)
================================================================================
GAPS COVERED: carry-forward META-COMMENTARY OUTPUT ENFORCEMENT.
VERDICT: NEW, small. Composes I-32 / I-47 / E-35.

THE GAP (grep-confirmed absent in .elai_cc): I-32 strips narrative persona / enforces schema-
  first role contracts on the INPUT side (report_fns.rs:1140 + role_contracts.rs +
  i32_role_scoping_contract.rs — BUILT). I-47 is the dropped-row scream path. But ELAI has NO
  runtime OUTPUT validator (banned-phrase reject + retry), NO work-log output grammar
  (READ/FOUND/DECISION/...), and NO action-density eval metric. The model can still emit
  ceremony/narration on the output side.

GROUNDING: this is a house-doctrine + harness-convention gap, not a paper claim. It operationa-
  lizes the global "act, don't narrate" / measured-Fable-signature rule (result-first openings,
  ~4 tool-actions per prose block, low words/message) into a product-side runtime check. The
  evidence is the ELAI codebase grep (absent) + the workflow's own output-formatting doctrine.
HARD BOUNDARY (HD-3 — critical, do not violate): "never constrained-decode reasoning roles;
  grammars only on pure op-emission." The output grammar + banned-phrase reject applies ONLY to
  OP-EMISSION / work-log roles (executor tool-call narration, status emission), NEVER to
  reasoning/synthesis roles (a reasoner must be free to think in prose). Decoding is sacred for
  reasoning; this gate is op-emission-only.

ELAI COMPOSITION: a runtime output validator on op-emission role output — a banned-phrase set
  (ceremony/affirmation/hedging openers) that triggers a reject + bounded retry, a work-log
  output grammar (READ/FOUND/DECISION/ACTION/RESULT-shaped) for op-emission roles only, and an
  ACTION-DENSITY eval metric (tool-actions per prose block / words-per-message) as an eval-corpus
  signal. Composes I-32 role contracts (the role classification already exists — reuse it to
  scope this to op-emission roles), I-47 (the scream-path pattern for a dropped/violating row),
  E-35 (the eval-metric seam).
VERTICAL SLICE: producer = the output validator + banned-phrase table + action-density metric;
  consumer = the op-emission role output path calls the validator (reject+retry) and the eval
  harness records action-density; smoke = feed an op-emission role a ceremony-laden output,
  assert reject+retry yields a compliant work-log-grammar output; feed a REASONING role the same
  and assert it is UNTOUCHED (HD-3 boundary holds).
DONE-GATE (ELAI-MEASURED): banned-phrase outputs are rejected+retried to compliance on an op-
  emission fixture; reasoning-role outputs are never constrained (zero false-positive on the
  reasoning corpus); action-density metric is recorded on the eval set.
DETERMINISM: the banned-phrase reject is deterministic (a frozen table, like I-46's frozen fixup
  table); retry is bounded.
FLAG: ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE default-OFF first (allowlist row, promotion_plan_id
  = J-106-promotion), observe-only until FP rate on the reasoning corpus is bounded at 0.
SSB: prefer-reuse an existing emission/violation row; +OutputCommentaryRejected only if a
  journaled reject row is wanted (net 0..+1).
CROSS-LINKS: composes I-32, I-47, E-35; binds HD-3.

================================================================================
5. DROPPED / DEFERRED LEDGER (record so it is never re-litigated — Section-5 norm)
================================================================================
- G-4 GROUNDING-RATIO + FRESHNESS SCORE: DROPPED from the product J set. Reason: gaps file judged
  it harness-scope + already-covered-in-spirit (elai_map_freshness.py + spec_link_check +
  spec-grounder + I-39 journal-derived status + K-22 doctrine-coverage); report(11) §3 found NO
  published correlation between a static grounding score and agent accuracy (unproven lift). The
  ONLY slightly-stronger idea is Veridge's anti-drift GATE (refuse to USE a stale map vs merely
  warn) — a HARNESS hardening of elai_map_freshness.py into fail-closed, NOT a product feature.
  RIDER: a future harness pass may flip elai_map_freshness.py from advisory to a fail-closed
  pre-gate; out of scope for these J product files.
- G-5 END-TO-END $/CORRECT-ANSWER METRIC: NOT authored as its own file. The live savings meter is
  folded into J-103's done-gate. The bigger "optimal retrieval policy / $-per-verified-outcome
  cost-curve" that report(12) §3 names as the field's biggest open question is K-18 case-reuse-
  economics / J-82 bench territory — RIDER on those, not a new J file (avoid a GD-1 duplicate
  bench).
- CARRY-FORWARD: GENERAL ADAPTIVE HUMAN-SIM E2E TESTER: DEFERRED to its own intake. It is real
  and net-new (product e2e is E-14 browser-only; no general cross-platform Unity/desktop/arbitrary
  -app adaptive human-simulator) but it is BIGGER and ORTHOGONAL to this context/memory/read-
  access cluster. It would compose perception (J-1/J-8/J-77) + AXTree-first + B-24 media validator
  + G-1 taint + the RRA spine and MUST stay deterministic-perception-first (HD-5: a rendered
  picture is not proof; coverage_count is the backstop). RECOMMEND: route it through its own
  paper-to-plan / idea-intake pass rather than bolting it onto this run.
- G-3 standalone file: NOT authored. Folded into J-105 as a default-OFF observe-only centrality
  rung (research downgrade). Do not author a dedicated centrality-ranking plan.

================================================================================
6. AUTHORING CHECKLIST FOR THE NEXT SESSION (paper-to-plan PHASE 5 entry)
================================================================================
1. Confirm next free J integer against docs/thefinalglance/J-plan/J-0_INDEX.md (this file assumes
   J-103..J-106; adjust if J-103+ were taken since 2026-06-26g).
2. Author FOUR files: J-103 (read-access ladder + interceptor + meter), J-104 (live-index
   overlay), J-105 (memory ROI attribution + folded G-3 observe-only), J-106 (output meta-
   commentary). Fewest-files grouping is already decided above; do not split further.
3. Each file is a vertical slice (producer + in-plan consumer + smoke through a REAL entrypoint).
   The smoke + done-gate blocks above are the W-7/R10 starting point — refine to live grep'd
   seams at authoring (re-grep every integration anchor; the line numbers here are 2026-06-26).
4. Re-grep the LIVE SCHEMA/TV integers (ssb.rs / action_type_lock.rs) for any new SSB variant;
   bump from live, never from a doc number. Default to prefer-reuse (net 0) per each entry above.
5. Every default-OFF flag named above needs a wire-dark-allowlist.toml row with its
   promotion_plan_id (J-10x-promotion).
6. ID-5/GD-1: NO paper number (MutaGReP/YoloFS/Mem0/Cursor/Ahad/Xiong) becomes a done-gate
   threshold — they are design rationale; the gate is ELAI-measured on an ELAI fixture/gold set.
7. Write _grounding/J-READ-ACCESS-GROUNDING-MAP.md (or per-file grounding maps) with live grep
   evidence (W-7 norm).
8. Update BOTH J-0_INDEX.md and J-SUMMARY_INDEX.md, then UPSERT docs/thefinalglance/
   ELAI-FEATURE-MAP.md (PLANNED(J-103..J-106) entries in 1.1/1.5/1.2 + cross-links + the dropped/
   deferred ledger riders in Section 4 + bump the header date) — feature-map upsert is mandatory.
9. Verify cargo check --workspace (not -p) + the action_type_lock golden test at authoring-close.
10. No commit (docs/ + workflow/ are gitignored). No sdl-mcp code copied (license).

SOURCES
- workflow/reasoning/elai-gaps-sdl-ai-setup-2026-06-26.md (gap list; spec-eval AS-SPECIFIED).
- Downloads/deep-research-report(11).md (G-3/G-7/G-4 grounding) + (12).md (G-1/G-2/G-5/G-6).
- docs/thefinalglance/ELAI-FEATURE-MAP.md (2026-06-26g; gap-relevant sections full-read + every
  composition anchor confirmed by grep).
- GRC decision ledger: topic 'sdl-gaps-jplan' (this session).
