ELAI GAP ANALYSIS vs sdl-mcp + ai-setup (Caliber)
Authored: 2026-06-26
Lens: spec-eval AS-SPECIFIED (ELAI plans treated as built; wiring ignored per user).
Scope: ELAI-the-PRODUCT at F:\.ELAI_workflow\.elai_cc, judged via ELAI-FEATURE-MAP.md
  (full read this session) + live .elai_cc greps. The .claude/ WORKFLOW HARNESS is NOT ELAI;
  gaps are tagged PRODUCT (in .elai_cc scope) vs HARNESS (in .claude scope, out of product scope).
Source summary: workflow/reasoning/steal-sdl-mcp-ai-setup-2026-06-26.md (sonnet agent, HIGH reliability,
  clones at docs/.source/{sdl-mcp,ai-setup}).

================================================================================
0. THE HEADLINE
================================================================================
Both repos independently re-derive ELAI's spine. Their shared philosophy -- "ask cheap first,
escalate with justification; off-agent deterministic verification; session telemetry as a
first-class metric; never trust the agent to self-regulate; fail closed" -- is verbatim ELAI's
TD-3 (LLM advisory, never the load-bearing gate) + wiring-enforcement (missing evidence = FAIL)
+ GD-2 (journal is the store, views re-derive). So this is MOSTLY CONFIRMATION. ELAI already
occupies the contrarian cells these tools occupy.

The genuinely net-new for the PRODUCT is small and concentrated in ONE area: sdl-mcp's
proof-of-need GRADUATED CODE-READ ACCESS LADDER. Everything else is covered, a refinement, or
harness-scope. Caliber (ai-setup) is ~90% HARNESS-scope (it maintains CLAUDE.md/AGENTS.md/cursor
rules) and maps to the workflow layer the user excluded; its few product-relevant ideas are
already subsumed.

LICENSE GATE (blocking on any code reuse): sdl-mcp ships a custom "FREE PERSONAL USE" community
license that BANS embedding in a for-sale product (docs/.source/sdl-mcp/LICENSE). ELAI has a
commercial/GTM track (K-plan), so ONLY the design ideas are portable -- never a line of its code.
ai-setup is MIT (clean).

================================================================================
1. THE FILES-vs-DATABASE QUESTION (the user's direct ask)
================================================================================
Q: does ELAI depend on files to keep agents on track? would a comment-system where ELAI writes
current state into files help, or is it redundant given a database-style done/not-done store?

A: ELAI is JOURNAL-FIRST (database), not file-first, for state. The load-bearing record is the
SSB (single-writer append-only SQLite WAL, ssb.rs); doctrine GD-2 = "the journal is the store,
views re-derive byte-identically, NO PARALLEL STATE". I-39 already derives `elai plan status`
FROM the journal explicitly "to kill hand-maintained drift". Agents run in their own context,
garbage-collected on return; a context-compiler builds a per-sub-phase context-package.md as a
DERIVED, disposable input. So files are projections, not truth.

THEREFORE a self-authored per-file state-comment system is REDUNDANT and HARMFUL: it creates the
exact parallel mutable state GD-2 forbids (a state-comment drifts from the journal; the append-only
journal cannot). The two repos AGREE with ELAI here: sdl-mcp keeps the symbol graph as the store
and writes memory files only for HUMAN inspection (memory/file-sync.ts:writeMemoryFile); Caliber
derives drift DETERMINISTICALLY and treats CALIBER_LEARNINGS.md as a projection. Both = journal/db
is truth, files are views. The user's instinct (state-in-files) is the one move all three designs
reject.

The only slice of the Reddit "docs->comments" idea worth anything is RATIONALE comments next to
code for model ATTENTION (not state). ELAI READS in-code comments that exist (context-budget
CodeContext/RepoMap slots + DKG + contextual_retrieval) and STRIPS ceremony so attention lands on
the contract (I-32). It does not AUTHOR rationale comments, and by GD-2 it must never author STATE
comments. Authoring rationale comments is an open, low-priority question, not a gap.

Reddit-comment point-by-point (for completeness):
- mermaid/sequence + C4 boundaries: B-33 Graph Viewer (Mermaid/cytoscape export) + J-31 canvas
  system-mode (nodes bind to plan-units/wiring-contracts = a boundary view). PARTIAL gap:
  C4-/sequence-diagram-as-maintained-artifact is not explicit.
- docs->comments for attention: see above (reads yes, authors no; by-design).
- defined unit/integration/e2e/smoke/regression + spec-vs-impl review: WELL COVERED -- contract
  tests, DST (3I), E-14 e2e, wiring-R10 integration smoke, C-28/I-20 mutation, J-16 regression
  selection; spec-vs-impl = I-37 EARS + C-29 intent-pairs/metamorphic + J-90 anti-vacuity.
- fresh-agent >= in-session, don't rely on agent memory: this IS ELAI's thesis -- fresh-agent-per-
  node, context-package, J-83/I-11 BeliefState handoff, H-51 handoff digest, journal.

================================================================================
2. GAPS FOUND (ranked; each = ELAI plan-ids it composes + verdict)
================================================================================

G-1  GRADUATED PROOF-OF-NEED CODE-READ ACCESS LADDER  [PRODUCT, REAL, MEDIUM -- best steal]
  sdl-mcp Iris Gate Ladder (docs/.source/sdl-mcp/src/policy/code-access.ts:decideCodeAccess,
  src/code/enforce.ts:enforceCodeWindow, src/CONTEXT.md): four rungs Symbol Card -> Skeleton IR
  -> Hot-Path Excerpt -> Code Window. A raw-source request is gated: the policy layer returns
  approve/DOWNGRADE/deny, the agent must name the identifiers it expects, and enforceCodeWindow
  verifies those identifiers actually exist in real file text BEFORE returning (post-hoc presence
  check; a hallucinated symbol cannot pull a code window).
  ELAI STATE: has every PIECE but not assembled as a gated read-escalation ladder with downgrade.
  DKG/B-29 symbol graph + impact API (the cards), adaptive AST chunking + FAST/STANDARD/DEEP tiers
  (E-17/phase-3K), J-25 section-aware budgeted read, J-78 context-access spec (declared per-dispatch
  read scope), I-46 bash interceptor (refuse-with-redirect -- but BASH only). What's MISSING: a
  policy decision-layer over CODE READS that DOWNGRADES a full-file/code-window request to a cheaper
  rung unless justified, with post-hoc presence verification. ELAI's gates are write/tool/security-
  facing (VibeGate F-1, capability_gate, F-3 tool filter), not read-artifact-facing.
  VERDICT: NEW/EXTENSION. Composes DKG + B-29 + J-78 + I-46 + context-budget + the rules engine
  (predicates already do AST/GREP/SCHEMA). Pure-decision-layer + downgrade-not-deny is the move.
  This is also the cleanest answer to the user's earlier idea-3 ("read FILENAME, ELAI decides how
  much") -- it is that idea with a policy gate and a presence check.

G-2  READ-TOOL INTERCEPTOR (the I-46 analog for the first-class Read tool)  [PRODUCT, REAL, SMALL-MED]
  sdl-mcp injects a server-level instruction (src/mcp/server-instructions.ts) steering the agent
  AWAY from native file reads toward the ladder, and exposes symbol-first tools so a naive read is
  the wrong default.
  ELAI STATE: I-46 intercepts bare bash (grep/cat/find -> redirect to a registered tool,
  bash_interceptor.rs). There is NO equivalent interceptor on the model's first-class Read tool that
  steers a naive full-file read to symbol-card/skeleton retrieval first. J-25 budgeted read exists
  but is not a gate that redirects.
  VERDICT: EXTENSION of I-46 (widen from bash to the Read tool) + J-25. Small, high-leverage,
  directly feeds the user's idea-3 and G-1.

G-3  MEMORY RANKED BY CODE-GRAPH CENTRALITY  [PRODUCT, REAL, SMALL]
  sdl-mcp ranks surfaced agent memories by centrality signals of the code they touch
  (src/mcp/tools/memory.ts:memory.surface, loadCentralitySignals) -- memory relevance tied to the
  ARCHITECTURAL IMPORTANCE of the symbol.
  ELAI STATE: E-5 memory + H-31/H-32/H-33 tiers rank by trust-tier + recency + supersession, and
  C-45 case-base by credit-attribution. None ranks by code-graph centrality (PageRank/fan-in over
  the DKG, which ELAI already computes in phase-1D + B-29).
  VERDICT: EXTENSION of E-5/H-31 retrieval ranking, consuming the SHIPPED PageRank/impact API.
  Genuinely net-new signal, small surface.

G-4  GROUNDING-RATIO + FRESHNESS SCORE for context/instruction docs  [MOSTLY HARNESS, SMALL]
  Caliber's deterministic scoring engine (docs/.source/ai-setup/src/scoring/checks/grounding.ts:
  checkGrounding) measures the fraction of REAL project dirs/files cited in a config doc and fails
  under a threshold; freshness.ts cross-references git log. No LLM.
  ELAI STATE: spec_link_check.py + spec-grounder verify "Spec references" map to real content;
  elai_map_freshness.py checks map staleness; I-39 derives plan status from the journal; K-22
  doctrine-coverage tags clauses by enforcer. The specific "grounding RATIO as a 0-100 score on a
  context doc" is not a product metric, and the docs it would score (CLAUDE.md/memory-bank) are
  HARNESS, which is out of product scope.
  VERDICT: ALREADY-COVERED in spirit for the product; the score-as-a-number is a HARNESS nicety.
  Not a product gap. Recorded for completeness.

G-5  FIRST-CLASS SESSION TOKEN-SAVINGS METER (raw-equivalent vs actual)  [PRODUCT, PACKAGING, SMALL]
  sdl-mcp tracks tokens-that-would-have-been-read vs tokens-actually-consumed and reports it at
  session end (src/mcp/savings-meter.ts, token-accumulator.ts) -- empirical context-economy, not a
  vibe claim.
  ELAI STATE: J-28/J-29 token ledger + usage window + live meter measure ACTUAL usage; CatRAG has a
  "55-87% token-savings target" (phase-1E); J-82 has `elai bench tool-output`; K-18 case-reuse
  economics. ELAI measures usage but does not surface a live "saved X vs naive-read this session"
  delta as a first-class meter.
  VERDICT: PACKAGING/EXTENSION of J-29 + CatRAG. Minor; the substrate exists.

G-6  OVERLAY-BEFORE-BASE LIVE INDEX + IDLE CHECKPOINT  [PRODUCT, SMALL refinement]
  sdl-mcp queries an in-memory overlay of recent edits BEFORE the base graph so edits are visible
  immediately without a full re-index, and an IdleMonitor drains the overlay to durable state during
  idle (src/live-index/coordinator.ts, idle-monitor.ts, overlay-reader.ts).
  ELAI STATE: B-31 resumable indexing + lock lifecycle + blake3 incremental update with CASCADE +
  IndexingService. Incremental, but the "overlay-first read for instant edit visibility + idle
  checkpoint" pattern is a refinement ELAI does not explicitly do.
  VERDICT: EXTENSION of B-31. Minor.

G-7  PER-LEARNING ROI ATTRIBUTION  [PRODUCT, ALREADY-COVERED-in-spirit, SMALL]
  Caliber measures which past learnings PREVENTED a failure this session to compute an ROI score
  (src/learner/attribution.ts:matchLearningsToFailures).
  ELAI STATE: K-18 recurrence-hit-rate (how often a new failing task matches an existing proven
  case), H-43 execution-grounded fitness, abstention scoring (H-39). ELAI's version is stronger
  (off-agent, execution-grounded). Caliber's simple per-learning attribution framing is a cue, not
  a gap.
  VERDICT: ALREADY-COVERED (K-18/H-43). Recorded.

================================================================================
3. ALREADY-COVERED (do NOT re-author -- the bulk of both repos)
================================================================================
- sdl-mcp Symbol Cards / symbol graph -> DKG (phase-1A) + PageRank repo map (phase-1D) + B-29 impact API.
- sdl-mcp Delta Pack + Blast Radius (fanIn + shortest-path + test proximity) -> J-16 BLAST-RADIUS gate
  (KEYSTONE: reverse-dep cone + regression-test selection pass->fail=BLOCK + diff-scope) + J-90 L4
  usage-weighted. STRONG match.
- sdl-mcp beam-search token-budgeted slice -> ELAI uses RRF + reranker + PageRank binary-search token
  fit (phase-1C/1D/1G); different algorithm, broader. Not a gap.
- sdl-mcp policy engine (pure decision layer) -> rules engine (phase-3F: AST/GREP/SCHEMA/METRIC
  predicates, ALLOW/WARN/BLOCK/ESCALATE) + capability_gate. The READ-artifact downgrade angle is the
  only uncovered slice = G-1.
- sdl-mcp sandboxed runtime + output-as-queryable-artifact -> Tier-C process isolation + G-3 runtime
  gates + B-35 context artifacts pipeline + SSB.
- sdl-mcp mandatory MCP session instructions -> FSM phase-aware tool/rule loading + I-32 role contracts
  + J-102 config-ingress; ELAI-as-MCP-server is J-2.
- sdl-mcp SKILL-survives-compaction -> E-43 skill system + context-budget Rules slot + J-52 path-keyed
  rule injection.
- caliber session-learning loop (PostToolUse capture -> offline LLM extracts patterns/skills) -> I-12
  evidence-gated playbook curator + J-12 offline failure-classifier + C-25 crystallizer + J-10 insight
  backprop + the VCG (J-96..J-98). STRONGER (execution-grounded, off-agent).
- caliber staged-write + backup + undo -> H-49 reversible edits/time-travel (CAS + undo-tree) + J-3
  harness undo. STRONGLY covered.
- caliber provider-agnostic LLM piggybacking on authed CLIs -> B-37 multi-runtime (11 backends incl
  claude_code/codex_cli/copilot_cli/gemini_cli via cli_common) + J-64 ACP + J-30 BYOK. BROADER.
- caliber git-diff incremental refresh -> H-51 handoff digest + J-16 diff-scope.
- caliber multi-platform config writer (Cursor/Copilot/Codex) -> N/A: ELAI is its own harness, does
  not target third-party agent configs.

================================================================================
4. CARRY-FORWARD (prior-session gaps, this thread, kept in one ledger)
================================================================================
- META-COMMENTARY OUTPUT ENFORCEMENT [PRODUCT, SMALL gap]: ELAI prevents ceremony on the INPUT side
  via I-32 (strip narrative persona, schema-first role contracts -- BUILT, report_fns.rs:1140 +
  role_contracts.rs + i32_role_scoping_contract.rs) + I-47. It has NO runtime OUTPUT validator
  (banned-phrase reject + retry), NO work-log output grammar (READ/FOUND/DECISION/...), NO action-
  density eval metric -- grep-confirmed absent in .elai_cc. Composes I-32/I-47/E-35.
- GENERAL ADAPTIVE HUMAN-SIM E2E TESTER [PRODUCT, NET-NEW]: product e2e is E-14 browser-only (+ E-13
  TDD, B-24 media validator, J-1/J-8/J-77 perception). No general cross-platform (Unity/desktop/
  arbitrary app) adaptive human-simulator tester. Would compose perception + AXTree-first + B-24 +
  G-1 taint + the RRA spine; must stay deterministic-perception-first (HD-5: a rendered picture is
  not proof).

================================================================================
5. RECOMMENDED PRIORITY (if any of this becomes a plan)
================================================================================
1. G-1 graduated proof-of-need code-read ladder (medium, best ROI, also closes user idea-3 + G-2).
2. G-2 Read-tool interceptor (small, extends I-46, pairs with G-1).
3. Carry-forward: meta-commentary output enforcement (small) ; human-sim e2e tester (net-new, bigger).
4. G-3 centrality-ranked memory (small, real signal).
5. G-5/G-6/G-7 packaging/refinements (low).
All would route through paper-to-plan (steal file -> NEW/EXTENSION verdicts -> plan files), inherit
the wiring PRE_GATE (producer + consumer + smoke in one unit), and -- for anything touching reads/
context -- the HD-2/IC-3 determinism doctrine + TD-3 (the policy gate is deterministic, never an LLM
deciding whether to downgrade). No sdl-mcp code copied (license).

SOURCES
- workflow/reasoning/steal-sdl-mcp-ai-setup-2026-06-26.md (sonnet agent, HIGH).
- docs/.source/sdl-mcp (custom non-commercial license) + docs/.source/ai-setup (MIT).
- docs/thefinalglance/ELAI-FEATURE-MAP.md (full read 2026-06-26) + live .elai_cc greps.
