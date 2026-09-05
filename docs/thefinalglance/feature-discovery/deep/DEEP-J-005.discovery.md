# DEEP-J-005 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L943-L958`, heading `### DEEP-J-005`; the next chunk starts at L960.
- Declared chunk: family J, **10 files**, **1,228 logical lines**, **149,610 bytes**, two auxiliary/grounding + eight numbered originals; target status **under 1,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all ten listed sources were read completely, first through last logical line. No original assigned to another deep chunk was opened. `10-J-SUMMARY_INDEX.discovery.md` was consulted only as an orientation and delta-comparison artifact for the relevant J-21/J-24/J-25/J-61/J-62/J-73/J-86/J-101 blocks plus grounding-map path inventory and cross-cutting composition notes; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `NASA:Lx-Ly`, `RLM:Lx-Ly`, `J-21:Lx-Ly`, `J-24:Lx-Ly`, `J-25:Lx-Ly`, `J-61:Lx-Ly`, `J-62:Lx-Ly`, `J-73:Lx-Ly`, `J-86:Lx-Ly`, and `J-101:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| auxiliary/grounding/fixture | 82 | 7,001 | b95f0969ee320db99c0d890ed9a15db9184d207d52b9262918778ce2b3f9c4a1 | docs/thefinalglance/J-plan/_grounding/J-NASA-GATES-GROUNDING-MAP.md | exact |
| auxiliary/grounding/fixture | 123 | 11,356 | c582305db03a7a219f3326b7f7e5e516d44a65f3cbd9f1bace681a795b229335 | docs/thefinalglance/J-plan/_grounding/J-RLM-GROUNDING-MAP.md | exact |
| numbered/index original | 76 | 11,507 | 9fce1c93f78b935be1b54fc865ae75f61fde8b78ed87c36a0b9fcb7d2c7aa873 | docs/thefinalglance/J-plan/J-21_memory_semantic_validation_poisoning.md | exact |
| numbered/index original | 72 | 11,224 | 32e1b64cf55987d2709626648561af3eb0b0b3e3ac155482f776ac509dafe446 | docs/thefinalglance/J-plan/J-24_nonprompt_steering_context_frugal_tools.md | exact |
| numbered/index original | 242 | 17,760 | 588642b09e1d311aa9938a08dc2898443c7bfbce7557377791efdad86d051806 | docs/thefinalglance/J-plan/J-25_mimocode_context_memory_mechanics.md | exact |
| numbered/index original | 184 | 15,967 | 08641ab134f7fdbd706f6a3d7fe9516c227d198d10f026a64f21485158ec9f8a | docs/thefinalglance/J-plan/J-61_canvas_redirect_loop.md | exact |
| numbered/index original | 129 | 24,172 | 499e523b2427ccc1df9aff1c006d134d1780e28e499e472c78501d48e7d715e8 | docs/thefinalglance/J-plan/J-62_env_constraint_skill_cards_prewrite_gate.md | exact |
| numbered/index original | 64 | 8,087 | 4d74609faccba0c2b46d119af96c1857fec3fb9da372c33ef971bed0fc1dba67 | docs/thefinalglance/J-plan/J-73_retrieval_abstention_gate.md | exact |
| numbered/index original | 117 | 18,276 | 610d965ec0aa4f3a9f14fe7187b89236c3985269bf1694a7d9dd06fdefcf9c5a | docs/thefinalglance/J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md | exact |
| numbered/index original | 139 | 24,260 | 95e4140231e2fc54c8969ed6d1010192c09b6792a5d0452395f9b07f10b0ea95 | docs/thefinalglance/J-plan/J-101_transformation_surviving_taint_binding.md | exact |
| **Aggregate** | **1,228** | **149,610** | — | **10 exact paths** | **exact** |

## Per-file atomic ledgers

### J-NASA-GATES-GROUNDING-MAP — deterministic-gates wave (J-65..J-71)

Source: `docs/thefinalglance/J-plan/_grounding/J-NASA-GATES-GROUNDING-MAP.md` (82/82 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| NASA-E001 | E/STATUS | Map is the W-7 anchor for the deterministic-gates wave J-65..J-71, authored 2026-06-22 from the 31-work NASA-grade gates research report and phase-4 plan `NASA-GRADE-DETERMINISTIC-GATES-PLAN.md`. | NASA:L1-L7 |
| NASA-E002 | E/EVIDENCE | Thirteen works were faithfully ingested to `workflow/papers/*-interface.md` by 13 sonnet paper-ingestors; every file:line a J-65..J-71 detail uses must be listed here or marked PLAN-asserted for executor re-grep. | NASA:L2-L7 |
| NASA-E003 | E/CFG | Live constants (grep 2026-06-22): `SCHEMA_VERSION = SCHEMA_VERSION_DDL_EPOCH + SSB_ACTION_TYPES.len()`; epoch `-309` at `ssb.rs:1503`, derived at `:1508`, public alias `:1513`; array head `ssb.rs:62`; len `577`; LIVE `SCHEMA_VERSION = 268`. NEVER hard-code; re-read at edit (W-9). | NASA:L9-L15 |
| NASA-E004 | E/STATUS | Feature-map stamp `241/550` (2026-06-12) is STALE; the `5b0a66d` drain stamp `~268/~577` matches this grep. BASELINE FOR THIS WAVE: SCHEMA 268 / SSB_ACTION_TYPES 577. | NASA:L14-L15 |
| NASA-E005 | E/DEP | Interface map W1 power-of-ten → J-65; W2 enforceable-security-policies → ND-1 feasibility + J-66/J-67; W3 edit-automata → J-67; W4 efficient-monitoring-safety → J-66 past-time leg. | NASA:L17-L21 |
| NASA-E006 | E/DEP | W5 rv-survey → ND-2 layer routing + J-66/J-68; W6 cbmc → J-68 Kani analog; W7 do333-case-studies → ND-3 + J-70; W8 tla-specifying-systems → J-68 D3 TLA+. | NASA:L22-L26 |
| NASA-E007 | E/DEP | W9 cedar → J-71 spike; W10 spark-gnatprove → J-68 D2 contracts; W11 nasa-std-8739-8b → ND-4 + J-70; W12 raft → J-69 hash-chain/watermark; W13 monitoring-rewriting → J-66 future-time leg. | NASA:L27-L30 |
| NASA-E008 | E/INT | SG-1: SSB append-only stream + action_type discipline at `ssb.rs:62/1503/1508` (grep-confirmed); broadcast-after-commit and MEMORIZING→COMPLETE are J-66 consumer seams (PLAN-asserted). | NASA:L32-L34 |
| NASA-E009 | E/SEC | SG-2: F-4 cant-lie gate is `AntiFabricationGate.check()` at task_runner.rs on the post-EXECUTING boundary (feature-map 1.6 WIRED); J-65/J-69 are siblings. The `test_passed` preceded by `stdout_hash` temporal gate IS F-4 — do NOT rebuild as a J-66 monitor. | NASA:L35-L37 |
| NASA-E010 | E/TEST | SG-3: frozen-gate harness is `.claude/gate-manifest.txt` + `gate_freeze_check.py` (grep-confirmed) plus checkers maintainability/production_caller/traced_test_assert/python_wire/wiring_contract/spec_link (all grep-confirmed). J-65 is a NEW sibling; J-70 enriches the manifest. | NASA:L38-L41 |
| NASA-E011 | E/INT | SG-4: J-12 RuntimeInterventionHarness (PLANNED) is the H2 pre-tool gate on rules `lifecycle.rs` in PreToolUse chain at `runtime_setup.rs:120` + HarnessInterventionEvent. J-67 EXTENDS its Block-only action to typed `{Accept,Suppress,Insert,Block}` taxonomy. | NASA:L42-L44 |
| NASA-E012 | E/DEP | SG-5: I-18 per-row `chain_prev_hash` + 5-layer-stack hash (PLANNED) + J-22 writer-isolation/sole-commit-point (PLANNED). J-69 is watermark-monotonicity + O(n) chain-SCAN-gate consumer, NOT a competing chain producer (GROUP F: do NOT author a competing plan). | NASA:L45-L47 |
| NASA-E013 | E/INT | SG-6: `file_ops.rs:272/300` captures-then-discards pre-image (cited by J-14/H-49); J-69 snapshot leg and J-68 serializer round-trip both touch SSB row bytes (PLAN-asserted; re-grep). | NASA:L48-L49 |
| NASA-E014 | E/DET | SG-7: phase-3I DST (madsim proptest 10000) is the runtime FSM simulator; J-68 D3 (TLA+/TLC) is its design-time exhaustive-BFS complement, NOT a replacement. | NASA:L50-L52 |
| NASA-E015 | E/SEC | SG-8: G-1 capability/taint sets + rules-engine COMPOSITE; J-71 Cedar spike re-expresses ONE existing capability gate as policy over `(principal,action,resource,context)`. | NASA:L53-L55 |
| NASA-E016 | E/LIMIT | ND-1 safety-property feasibility (W2): a runtime monitor gate MUST be a safety property + predicate on STEP/EVENT counts, never wall-clock; a liveness/wall-clock predicate is rejected at authoring. | NASA:L57-L60 |
| NASA-E017 | E/LIMIT | ND-2 RV layer routing (W5): assign each property to its cheapest SOUND layer (proof/model-check/RV/test); a property a layer cannot soundly decide is a finding; respect the 3-valued monitorability caveat. | NASA:L61-L62 |
| NASA-E018 | E/FAIL | ND-3 indeterminate-never-pass + DO-333 traceability (W7): every frozen gate declares `{objective, technique, evidence, satisfaction}`; UNKNOWN verdict = FAIL (mirrors maintainability). | NASA:L61-L63 |
| NASA-E019 | E/SEC | ND-4 IV&V independence (W11): auditor keeps FINANCIAL independence (own token/cycle budget orchestrator cannot raid) + MANAGERIAL independence (not short-circuitable by orchestrate H-10 skips on safety-critical changesets). | NASA:L64-L66 |
| NASA-E020 | E/DEP | Anti-slopsquat LIVE deps: kani-verifier 0.67.0 PRIMARY for J-68 D1; creusot 0.12.0 optional gold contracts J-68 D2; Prusti REJECTED (unmaintained, last 2024-03); cedar-policy 4.11.1 LIVE Apache-2.0 for J-71 spike only. | NASA:L68-L70 |
| NASA-E021 | E/LIMIT | blake3 not sha256 for the J-69 chain (CLAUDE.md anti-stale); TLA+/TLC is a toolchain dep (J-68 D3), CI-cost-scoped. Re-verify all deps at edit before any dep add. | NASA:L68-L71 |
| NASA-E022 | E/LIMIT | NG-1: the `test_passed -> stdout_hash` gate is ALREADY F-4 (SG-2); J-66 builds ONLY G1/G2 + the obligations. | NASA:L73-L74 |
| NASA-E023 | E/LIMIT | NG-2: GROUP F is an EXTENSION of I-18/J-22, never a competing chain (SG-5); J-69 adds the scan-gate + watermark. | NASA:L75-L75 |
| NASA-E024 | E/LIMIT | NG-3 N3 RETRACTED: ELAI already has source-grounded copy-not-fabricate substrate (SOURCE_CHECK / source_ingest / A-RAG extractive / H-53 citation pre-binding / source-tier list) — NOT a steal candidate. | NASA:L76-L78 |
| NASA-E025 | E/EVIDENCE | NG-4: every external figure (88.5% / -61.7% / 71% SIF / DO-333 objective counts) is DESIGN-rationale only; done-gate produces ELAI-measured evidence (HD-1 / ID-5). Never claim a borrowed number. | NASA:L79-L80 |
| NASA-A001 | A/STATUS | Many SG-* consumer seams and J-12/I-18/J-22 substrates are PLAN-asserted; exact live line numbers and wiring status must be re-grepped at edit before load-bearing use. | NASA:L32-L55 |
| NASA-C001 | C/STATUS | Canonical live baseline is 268/577 (2026-06-22 grep); any consumer still citing 241/550 is stale and must re-derive. | NASA:L9-L15 |

Category coverage: capabilities/doctrine ✓; workflows/layer routing ✓; schemas/SSB constants ✓; flags/config (derived only) ✓; integrations/seams ✓; security/permissions/IV&V ✓; failures/UNKNOWN=FAIL ✓; determinism/simulator complement ✓; observability/evidence ✓; tests/frozen-gate ✓; constraints/non-goals/corrections ✓; dependencies/status/deferred ✓. No standalone UI surface; no retry/idempotency API.

### J-RLM-GROUNDING-MAP — RLM recursive-reasoning wave (J-107..J-108)

Source: `docs/thefinalglance/J-plan/_grounding/J-RLM-GROUNDING-MAP.md` (123/123 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| RLM-E001 | E/STATUS | Map is the RLM-RECURSIVE-REASONING wave grounding for J-107..J-108, authored 2026-06-26 over phase-1..4 reasoning `rlm-recursive-agents-grc-2026-06-26.md` + two measured experiments; run-id `s-rlm-jplan-2026-06-26`. | RLM:L1-L8 |
| RLM-E002 | E/EVIDENCE | Paper-side reliability is HIGH for the two units' load-bearing facts (own measured data + code just read); not on single-source-LOW RLM marketing numbers. Status legend mirrors J-0_INDEX. | RLM:L5-L8 |
| RLM-E003 | E/CAP | RG-1: RLM core (arXiv:2512.24601) is a DSPy.RLM SCAFFOLD harness (no fine-tuning) where the model WRITES PYTHON to slice/recurse over a context VARIABLE (REPL-as-context). | RLM:L13-L16 |
| RLM-E004 | E/LIMIT | RG-1 G6 gate: WHO chooses decomposition — full-RLM = LLM at run time; map-reduce/not-RLM = orchestrator fixes at design time. ELAI is plan-first/FSM-gated/TD-3-deterministic => DELIBERATELY Level-2/3 by G6. | RLM:L17-L19 |
| RLM-E005 | E/LIMIT | COROLLARY binding both units: "be more RLM at the orchestration grain" is an ANTI-GOAL (violates TD-3, Workflow-tool ban, HD-2). Only defensible steal is the narrow WITHIN-REASONING-STEP primitive + its measurement. | RLM:L19-L22 |
| RLM-E006 | E/CAP | RG-2: LongCoT bench (arXiv:2604.14140) harness GitHub LongHorizonReasoning/longcot is MIT + HF dataset live => PORTABLE, license-clean; concrete buildable unit J-107 and RG-6 measurement instrument. | RLM:L23-L26 |
| RLM-E007 | E/OBS | RG-3 exp-1 lexical long-context (OWN DATA; groq llama-3.1-8b, W=6000, bands 0.5/2/8W, n=12/band): A 25% | B RLM 62% erratic | C ELAI BM25 100% | D merge 100% cheapest. Verdict: deterministic retrieval alone captures the RLM benefit; weak 8B NEVER engaged rlm_query recursion. | RLM:L27-L33 |
| RLM-E008 | E/OBS | RG-4 exp-2 multi-hop (OWN DATA; cerebras gpt-oss-120b; 2-hop): A 12% | B 0% (hallucinated "42" on 7/8 at 8xW) | C 0% abstains safe | D 25% only arm >0% at 8xW, correct-or-abstains. Free-code RLM fails by FABRICATING. | RLM:L34-L40 |
| RLM-E009 | E/OBS | RG-5 ceiling: ORACLE pure deterministic traversal ZERO model = 8/8 = 100%. D@18 model-driven navigator plateaus (8xW 25% flat). GRAPH-HOP HYBRID (deterministic hops, model only atomic entity+number read) on weak 8B = 7/8 = 87%. LEVER = DETERMINISM. | RLM:L41-L47 |
| RLM-E010 | E/SEC | RG-6 resolves J-55 tension: constrained named-op path (D) held 100% on weak lane; free-code path (B) erratic + fabricated. Constrained single-string-slice + capped-child-spawn sits BELOW general code-mode collapse threshold; free-Python RLM is REJECTED on every lane. | RLM:L48-L53 |
| RLM-E011 | E/INT | RS-1: `query_with_tier` at `crates/knowledge/src/pipeline.rs:581` [WIRED] — Stage-1 deterministic candidate set (BM25+vector+rerank); J-108's candidate set; production superset of exp-1 Arm C. | RLM:L58-L60 |
| RLM-E012 | E/INT | RS-2: ToolDispatcher + `dispatch_file_read_outline` at `adaptive_tool.rs:295` [WIRED] — journaled named-op substrate (SSB markers ToolCallStart/Read/Branch/Finding/Converge/Escalate/ToolCallEnd). One journaled named op per turn == J-108 Stage-2 shape. | RLM:L61-L63 |
| RLM-E013 | E/INT | RS-3: `SearchLadder::run_best_of_n` at `search/ladder.rs:173` [DARK(ELAI_SEARCH_LADDER_ENABLE), `search_ladder_enabled()` :382] — WIDTH template; J-86 activates it; J-108 is DEPTH sibling (copy struct/handle/journal idiom, do NOT re-implement). | RLM:L64-L67 |
| RLM-E014 | E/INT | RS-4: `hippo_rag.rs::retrieve_notes` at `:57` [WIRED] — HippoRAG Personalized-PageRank; PRODUCTION form of RG-5 graph-hop hybrid = J-108 PRIMARY multi-hop path. | RLM:L68-L71 |
| RLM-E015 | E/INT | RS-5: K-15 `elai bench reasoning-budget` + ConvergenceCurveMeasured + bench_convergence + ConvergencePanel (I-27) [PLANNED/DARK] — bench RUNNER SHAPE J-107 reuses; GD-1 do-NOT-fork-the-runner. | RLM:L72-L75 |
| RLM-E016 | E/DEP | RS-6: J-42 within-step distill-reset, J-38 mid-task retrieval, J-86 receipt-gated WIDTH ladder, I-38 QDELTA, J-34 empirical context calibration — DEPTH×WIDTH axes + measurement siblings J-108/J-107 extend. | RLM:L76-L78 |
| RLM-E017 | E/SEC | RS-7: J-55 model-strength modality router [PLANNED, KEYSTONE] — Qwen-7B local lane NEVER routed to CodeAct. J-108 NavOp set fixed + executor deterministic; local lane named-op + HippoRAG only; capable-lane default for recursive SubQuery. | RLM:L79-L83 |
| RLM-E018 | E/CFG | LIVE SCHEMA baseline at authoring ~268/~577 DERIVED (`ssb.rs:1508` = DDL_EPOCH(-309)+len(SSB_ACTION_TYPES)); NEVER hard-code, re-derive at edit W-9. | RLM:L82-L83, L117 |
| RLM-E019 | E/LIMIT | RC-1 SUPERSEDES-BUT-SCOPES feature-map "RLM REPL-as-context: DEFER": revisit ONLY as CONSTRAINED within-reasoning-step primitive (single string-slice + one capped rlm_query-style child-spawn over shipped orchestrate/BSP), capable-lane DEFAULT, local-lane GATED on J-107 measurement. Free-Python/orchestration-RLM defer STANDS. | RLM:L88-L93 |
| RLM-E020 | E/FLOW | RC-2: J-108 PRIMARY = HippoRAG (RS-4) + atomic model extraction; model-driven RetrievalNavigator (NavOp loop) is FALLBACK for no clean entity linkage (caps ~25-37%). Model NEVER navigates by writing code. | RLM:L94-L98 |
| RLM-E021 | E/EVIDENCE | RC-3 arXiv IDs grounded via on-disk interface maps: RLM 2512.24601 HIGH; LongCoT 2604.14140 HIGH; HippoRAG 2405.14831 NeurIPS 2024 in-code. Supporting blogs MED-LOW = confirmation only, never a unit. | RLM:L99-L103 |
| RLM-E022 | E/EVIDENCE | Carried web landmines: RLM headline numbers are SINGLE-SOURCE LOW reliability; MOTIVATE local-lane wedge only; J-107/J-108 done-gates produce ELAI-MEASURED deltas per-(model,lane), never these figures. | RLM:L107-L110 |
| RLM-E023 | E/EVIDENCE | RLM "Math/Chemistry ~0" prediction was REFUTED by raw.works follow-up — reinforces scaffold-not-model lever, not a target. | RLM:L111-L112 |
| RLM-E024 | E/EVENT | J-107 NO new action_type — reuse ConvergenceCurveMeasured (I-27) +Kind LongCotArm if discriminator needed; baseline+0..+1. | RLM:L118-L119 |
| RLM-E025 | E/EVENT | J-108 NO new action_type — reuse SearchNodeEvent (I-4/J-86; Promoted/BudgetCollapsed) + ToolDispatcher markers for each NavOp; HippoRAG rides query_with_tier rows; baseline+0..+1. | RLM:L120-L121 |
| RLM-A001 | A/CFG | Whether J-107 needs a LongCotArm discriminator on ConvergenceCurveMeasured is decide-at-edit. | RLM:L118-L119 |
| RLM-C001 | C/LIMIT | Feature-map blanket "RLM DEFER" is scoped: constrained within-step primitive is revisited; free-Python/orchestration-RLM remains deferred anti-goal. | RLM:L88-L93 |

Category coverage: capabilities/anti-goals ✓; measured experiments ✓; substrate seams ✓; design corrections ✓; events/SSB prefer-reuse ✓; security/lane guards ✓; evidence/landmines ✓; deferred free-code ✓. No dashboard surface defined in this map.

### J-21 — memory semantic-validation + poisoning hardening

Source: `docs/thefinalglance/J-plan/J-21_memory_semantic_validation_poisoning.md` (76/76 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J21-E001 | E/STATUS | J-21 is Track J, lever CANT-MISREMEMBER, grounding STRONG (A-MEM/GEM/MPBench WebFetch-CONFIRMED), nature BUILD extending H-31/H-32/E-5 (never amends them), effort M, WAVE 3, depends none (rides shipped crates/memory). | J-21:L1-L3 |
| J21-E002 | E/FAIL | Gap 1: citation-validity ≠ semantic-validity — Tier-A revalidates file:line/symbol EXISTS but refactor/rename/behavior-change can preserve path while invalidating the stored CLAIM. | J-21:L5-L6 |
| J21-E003 | E/SEC | Gap 2: write-time-only poisoning defense misses semantic poison; MPBench shows benign-at-write / harmful-after-retrieval records survive; prompt-injection defenses do NOT transfer to memory poisoning. | J-21:L6-L7 |
| J21-E004 | E/STATE | Gap 3: fixed TTL forgets rare-but-important invariants (28-day expiry can drop low-frequency high-value project conventions). | J-21:L7-L8 |
| J21-E005 | E/LIMIT | CRITICAL: seven memory platform names (TierMem/MemIR/Eywa/RIG/SPADE/Prometheus/MemMachine/SimpleMem) are unverifiable — cite design PATTERN + confirmed arXiv only, NEVER platform names. | J-21:L7-L8, L15-L16, L47-L47 |
| J21-E006 | E/CAP | Commit-aware semantic validation (Tier-A upgrade): store fact against commit hash + AST/function-signature identity; re-derive claim against current checkout scoped by repo+branch+commit-range before inject. Symbol exists but signature/behavior changed = STALE, not valid. | J-21:L10-L10 |
| J21-E007 | E/SEC | Retrieval-time authorization + typed claim/evidence: separate EVIDENCE from truth-bearing CLAIMS; retrieval only authorizes still-supported claims; new writes QUARANTINED until validated; sanitize BEFORE abstraction/compaction. | J-21:L11-L11 |
| J21-E008 | E/DATA | Provenance-linked raw-evidence tier: immutable raw layer beneath extracted-fact layer; escalate only when fact layer insufficient; deterministic, minimal-LLM on read path. | J-21:L12-L12 |
| J21-E009 | E/CFG | TTL → activation-decay: demote fixed TTL to maximum-age GUARDRAIL; activation/heat score (usage + confidence + revalidation) so revalidated/used facts live longer. | J-21:L13-L13 |
| J21-E010 | E/SEC | Zero-memory auditor refinement CONFIRMS H-31 firewall: auditor/critic/adversary get ZERO memory injection; J-21 asserts typed-evidence-packet form, does not rebuild the firewall. | J-21:L14-L14, L50-L50 |
| J21-E011 | E/LIMIT | EXCLUDED: Mem0/Zep/Letta swap; citing 7 platforms as platforms; [OUTDATED]-narration of superseded rows; DELETE of superseded rows (banned for replay, H-31). | J-21:L15-L15, L48-L49 |
| J21-E012 | E/DEP | Existing substrate: crates/memory PARTIAL (Mem0/Zep/MINJA); H-32 Tier-A existence check + 28d TTL + FactValidationFailed(210); H-31 Tier-B bi-temporal + hard-filter + ContextSlot::Memory + auditor firewall; H-30 stale_fact_reached_prompt; F-12 NLI + F-5 quarantine; DKG + B-29 + J-18 LSP. | J-21:L17-L23 |
| J21-E013 | E/INT | Integration LOC estimates: commit-aware ~120 over H-32; retrieval-time auth + quarantine ~150; raw-evidence tier ~100; activation-decay ~60. | J-21:L25-L26 |
| J21-E014 | E/EVENT | +MemoryValidationEvent SSB Pattern-2 single-variant; Kind: SemanticRevalidated / RetrievalAuthorized / RawEvidenceEscalated / PoisonQuarantined. baseline+1 from LIVE 231/542 (grep 2026-06-12); re-read at edit. PREFER-REUSE: extend H-32 FactValidationFailed(210) with semantic-vs-existence discriminator if cleaner (net 0). | J-21:L27-L27 |
| J21-E015 | E/SURFACE | frontend_pair: memory provenance/validation surface on MemoryGovernanceEvent / memory-hub panel (H-33) or W-6-justified absence. | J-21:L28-L28, L51-L51 |
| J21-E016 | E/INT | Production reachability: real `elai` run that injects a code-grounded fact into a prompt (capture+retrieval path once un-darked). contracts/test caller NEVER satisfies R7. | J-21:L41-L43 |
| J21-E017 | E/FAIL | Stub bans: no existence-only as semantic; no write-time-only poison defense; no platform-name citations; no [OUTDATED] narration/DELETE; no fixed-TTL sole policy; no memory injection into auditor/critic/adversary. | J-21:L44-L50 |
| J21-E018 | E/OBS | Trigger: stored Tier-A fact whose symbol is REFACTORED, then later prompt would inject it. Expect SemanticRevalidated → STALE; optional RawEvidenceEscalated; PoisonQuarantined on poisoned write. Stale fact NOT injected; existence-only would have injected. | J-21:L52-L55 |
| J21-E019 | E/TEST | NEGATIVE: symbol exists AND semantics unchanged injects normally; unresolved symbol FAILS-LOUD as UNVERIFIED (never silent inject); fixed-TTL-expired but high-activation invariant is RETAINED. | J-21:L55-L55 |
| J21-E020 | E/EVIDENCE | Grounding: A-MEM arXiv:2502.12110 CONFIRMED; GEM arXiv:2605.26252 CONFIRMED; MPBench arXiv:2606.04329 CONFIRMED; MINJA arXiv:2503.03704; H-31 arXiv:2510.09905. live SSB 231/542 (2026-06-12). | J-21:L57-L63 |
| J21-E021 | E/DEP | COMPOSES-WITH H-32 (existence→semantic), H-31 firewall (typed packet), H-30 instrument (activation trigger), E-5 substrate, F-12/F-5, J-18 LSP, G-1 taint on raw-evidence. Restates none. | J-21:L65-L66 |
| J21-E022 | E/DEFER | Open: re-derivation cost — make SELECTIVE (revalidate on H-30 signal or commit-range change, cache verdict). | J-21:L69-L69 |
| J21-E023 | E/CFG | Open: branch scope — scope memories by repo+branch+commit-range; revalidate against relevant checkout not only HEAD. | J-21:L70-L70 |
| J21-E024 | E/LIMIT | Open: raw-evidence growth needs explicit compaction/namespaces + scoped forensic query; bound it, log the bound. | J-21:L71-L71 |
| J21-I001 | I/DEP | Commit-aware identity is the same compiler-grade signature J-18 LSP resolves and J-16 blast-radius traverses — shared "what a symbol IS" across memory/retrieval/impact. | J-21:L73-L74 |
| J21-A001 | A/EVENT | Whether to add MemoryValidationEvent (+1) or extend FactValidationFailed discriminator (net 0) is decide-at-edit. | J-21:L27-L27 |
| J21-A002 | A/SURFACE | Memory-hub panel presence vs W-6-justified absence is decide-at-edit. | J-21:L28-L28, L51-L51 |
| J21-C001 | C/STATUS | Authoring SSB stamp 231/542 (2026-06-12) is older than NASA/RLM maps' ~268/577; executor must re-grep live baseline at edit (W-9), not hard-code either. | J-21:L27-L27, L63-L63 |

Category coverage: capabilities ✓; workflows/states/lifecycle ✓; schemas/claim-evidence ✓; surfaces (memory-hub) ✓; events ✓; security/poisoning ✓; failures/STALE/UNVERIFIED ✓; observability ✓; tests/negatives ✓; constraints/exclusions ✓; dependencies/deferred ✓. Retry/idempotency not specified.

### J-24 — non-prompt steering + context-frugal tools

Source: `docs/thefinalglance/J-plan/J-24_nonprompt_steering_context_frugal_tools.md` (72/72 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J24-E001 | E/STATUS | J-24 is Track J, lever EFFICIENT/STEERING, grounding STRONG (sidecar /v1/steering live; H-4/H-34 substrate), nature BUILD (activate dark substrate), effort M, WAVE 3, depends none. | J-24:L1-L3 |
| J24-E002 | E/CAP | Answers Q1: activation steering (ActAdd residual-stream contrast vector) shapes behavior/personality at ZERO context tokens; sidecar exposes /v1/steering/* currently DARK (`--no-steering` on boot). | J-24:L5-L6, L10-L10 |
| J24-E003 | E/CAP | Answers Q2: tools cannot be zero-token; model needs SOME schema to CALL a tool. Win is lazy/deferred via H-4 Tool Search (77K→8.7K), H-34 MCP defer-loading, E-22 lazy MCP schema. | J-24:L6-L7, L11-L11, L48-L48 |
| J24-E004 | E/LIMIT | Honest framing: rules are DATA evaluated in Rust (zero tokens, 100% fire) vs skills (tokens every load + probabilistic). MEASURED not yet — SAFEDELTA/efficiency benchmarks unrun (G-6 BENCH-PENDING). J-24 activates, does not claim measured win. | J-24:L7-L7 |
| J24-E005 | E/CAP | Build vector-extraction + apply-on-generate path (~150 LOC) behind a flag; MEASURE effect vs equivalent role-prompt at 0 token cost. | J-24:L10-L10, L24-L24 |
| J24-E006 | E/INT | Context-frugal tools thread: assert Tool Search + MCP defer-loading on production OAuth lane + STDIO defer (McpToolDeferred 212); wire activation + measure tool-def token delta; do not rebuild H-4/H-34. | J-24:L11-L11, L26-L26 |
| J24-E007 | E/CFG | Deterministic observation-size pre-assessment (BACM clean part): before committing observation, query token size; if observed > slot_remaining × alpha, trigger compaction BEFORE insert (3-mode NONE/SELECTIVE/ALL). Size check not neural rewrite; <50ms compatible; IC-3 clean. | J-24:L12-L12, L25-L25 |
| J24-E008 | E/DEFER | EXCLUDED: measured win claim before benchmark; SWE-Pruner neural skimmer (IC-3; deferred opt-in-measured); finetune-based steering (E-47/E-55 deferred); force-JSON/constrained-decode as "steering" (HD-3 banned). | J-24:L13-L13, L45-L46 |
| J24-E009 | E/DEP | Existing: FastAPI /v1/messages + /v1/steering/* + LogitsProcessor chain + ActAdd named phase-3J; H-4 PARTIAL; H-34 PARTIAL (212/213); C-35 tool gating + E-22 lazy MCP; context_budget 8 slots phase-3E WIRED; H-6 adaptive_reduce PARTIAL. | J-24:L15-L21 |
| J24-E010 | E/INT | Integration: `python/elai_inference/server.py` + crates/providers steering path; `context_budget.rs` size pre-assessment ~60 LOC; H-4/H-34 activation is wiring+measurement. | J-24:L23-L26 |
| J24-E011 | E/EVENT | +BehaviorSteeringEvent Pattern-2; Kind: SteeringVectorApplied / SteeringEffectMeasured / ObservationSizePreassessed / ToolSchemaDeferred. baseline+1 from LIVE 231/542. PREFER-REUSE: ToolSchemaDeferred may reuse H-34 McpToolDeferred(212); net 0..+1. | J-24:L27-L27 |
| J24-E012 | E/SURFACE | frontend_pair: steering-effect + token-budget surface on existing runtime/context panel (decide at edit; W-4) or W-6-justified absence. | J-24:L28-L28, L49-L49 |
| J24-E013 | E/INT | Production reachability: (a) `elai run --local` with steering on → sidecar /v1/steering apply at generate [PRODUCTION local, remote-GPU-gated W-10]; (b) OAuth lane with Tool Search on. contracts NEVER R7. | J-24:L39-L42 |
| J24-E014 | E/LIMIT | Stub bans: no measured win claim pre-benchmark; no SWE-Pruner in context path; no force-JSON as steering; no default-ON steering/tool-defer until effect+FP bounded on ≥3 real runs (GD-4) + wire-dark-allowlist row (W-2/R9); no zero-token-tools CLAIM. | J-24:L43-L48 |
| J24-E015 | E/OBS | Trigger: two `elai run --local` same task/seed — steered vector (zero role-prompt tokens) vs equivalent role-prompt. Expect SteeringVectorApplied; SteeringEffectMeasured; ObservationSizePreassessed on oversized grep obs; ToolSchemaDeferred on lazy MCP. | J-24:L50-L53 |
| J24-E016 | E/TEST | NEGATIVE: steering off reverts behavior; fitting observation does NOT trigger compaction; deferred tool still loads on demand (defer ≠ drop); no zero-token-tools claim emitted. | J-24:L53-L53 |
| J24-E017 | E/EVIDENCE | Grounding: sidecar ActAdd feature-map 1.9; H-4/H-34/E-22/C-35; BACM arXiv:2604.01664 VERIFIED; SWE-Pruner arXiv:2601.16746 EXCLUDED; G-6 BENCH-PENDING. live SSB 231/542. | J-24:L55-L59 |
| J24-E018 | E/DEP | COMPOSES-WITH phase-3J sidecar, H-4/H-34/E-22/C-35, H-6+context_budget 3E, I-32 structural role scoping (steering is non-prompt complement), G-6 SAFEDELTA. Restates none. | J-24:L61-L62 |
| J24-E019 | E/SEC | Open: steering-vector provenance — promote through ledger like config (ID-6); extract deterministically (contrast pairs) and version it; not un-audited backdoor. | J-24:L65-L65 |
| J24-E020 | E/CFG | Open: steering stability — high magnitude can degrade coherence (DoLa-class risk); measure coherence alongside behavior shift; default conservative GD-4-bounded. | J-24:L66-L66 |
| J24-A001 | A/EVENT | Whether ToolSchemaDeferred reuses McpToolDeferred(212) or adds a new Kind is decide-at-edit. | J-24:L27-L27 |
| J24-A002 | A/SURFACE | Runtime/context panel tile vs W-6 absence is decide-at-edit. | J-24:L28-L28 |
| J24-A003 | A/CFG | Exact `alpha` for slot_remaining × alpha and default steering vector magnitude are unspecified. | J-24:L12-L12, L66-L66 |
| J24-C001 | C/STATUS | Authoring SSB 231/542 vs later wave baselines; re-grep at edit. | J-24:L27-L27, L59-L59 |

Category coverage: capabilities/CLI/endpoints ✓; surfaces (runtime panel, /v1/steering) ✓; workflows ✓; flags/defaults ✓; events ✓; security/vector provenance ✓; failures/negatives ✓; observability/measurement ✓; constraints ✓; deferred neural/finetune ✓. Formal idempotency not specified.

### J-25 — MiMoCode context+memory mechanics

Source: `docs/thefinalglance/J-plan/J-25_mimocode_context_memory_mechanics.md` (242/242 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J25-E001 | E/STATUS | J-25 PLANNED WAVE RSW-1/RSW-2; EXTENSION of seven MiMoCode mechanics into H-50/H-51/H-6/E-25/SessionDag/B-22/E-5/H-33; nothing is a new subsystem. Effort M-L; RSW-1 ships T3/T4/T6 then RSW-2 T1/T2/T5/T7. | J-25:L1-L5 |
| J25-E002 | E/EVIDENCE | Steal is seven precise mechanics from MiMoCode source (docs/.source/MiMo-Code), not architecture; citations in MiMo-Code-INDEX.md §4 + mimo-code-part-2/7.md; steal file `steal-mimocode-sia-agentskills-2026-06-12.md` A1..A7. | J-25:L7-L13 |
| J25-E003 | E/DATA | T1 typed checkpoint template: 11 sections with per-section token budgets (intent 500, next-action 1000, directives 800, task-tree 1000, current-work 2000, files 1500, discoveries 2000, errors 1500, live-resources 1000, decisions 3000, notes 800; ~11K total) + discoveries-dedup validator (prior-titles). | J-25:L16-L21 |
| J25-E004 | E/CAP | T1 lands as OUTPUT CONTRACT of H-51 handoff digest (`handoff_digest.rs`): typed DigestSection enum + per-section byte budget + dedup; NOT a second digest format. | J-25:L19-L22 |
| J25-E005 | E/FLOW | T2 section-aware budgeted read: ALL section headers + index lines preserved unconditionally; body filled to budget in section order; file:line resumption hint on overflow; 0.95 safety factor. Pure function for H-51 tiered load + H-6 selective-reload. | J-25:L23-L26 |
| J25-E006 | E/DET | T3 tool-pair boundary guard: pure backward walk before ANY trim/mask/compaction; un-orphan tool_use/tool_result pairs then same-message thinking-block siblings; zero I/O. Lands at SessionDag 3-pass trim (`session_dag.rs:1551` region) + H-6 + E-25. Violated pair = malformed Anthropic request. | J-25:L27-L32 |
| J25-E007 | E/CFG | T4 graded pressure ladder: pressureLevel 0/<50% 1/50-70% 2/70-85% 3/≥85% of usable (= input_limit − min(20K, max_output)) as TRIGGER INPUT to H-50 ContextPolicy; replaces single-threshold gates (conversation.rs:22 flat auto-compact, cli_consts.rs:20 85% gate). | J-25:L33-L36 |
| J25-E008 | E/CFG | T4 protected classes: skill/anchor slots never erased; recent-turn floor 25% of usable; map to ContextSlot floors (context_budget phase-3E SHIPPED — compose, don't re-implement). | J-25:L37-L38 |
| J25-E009 | E/CONC | T5 writer coalescing + fork snapshot: background digest-writer with 1-slot newest-wins pending queue (newer evicts older); frozen (system_parts, tool_defs, watermark_msg_id) snapshot at spawn so writer never recomputes prompt (prefix-cache aligned, HD-2). Lands on H-51; writer-isolation matches I-1. | J-25:L39-L43 |
| J25-E010 | E/FAIL | T6 deterministic completion gate: at agent-return, query task registry for non-terminal tasks owned by returning agent; if any, ONE re-entry OR downgrade status to partial with open IDs listed. Off-agent DB rows; TD-3-clean. Lands as EvidenceCheck on B-22 AutoAdvanceEvaluator / task_runner. LLM-judge sibling (goal.ts) REJECTED. | J-25:L44-L49 |
| J25-E011 | E/DATA | T7 retrieval mechanics: (a) relative score floor = 0.15 × topScore, 3× over-fetch cap 50; (b) file fingerprint `"{size}-{mtimeMs}"` upsert-on-change reconcile with prune-on-delete. Lands in crates/memory (E-5/H-33); mechanics input to J-21. NOT platform swap. | J-25:L50-L54 |
| J25-E012 | E/DEFER | Riders (NOT tasks): A5 microcompaction = H-49 RV-2; A10 dream/distill cadence = H-51/I-12; A11 midpoint cache-marker = H-2; A12 subscriber seq-continuity = J-22. | J-25:L55-L58 |
| J25-E013 | E/DEP | Existing: H-50 PARTIAL uncommitted; H-51 PARTIAL + HandoffLifecycleEvent(227); SessionDag 3-pass WIRED; H-6 PARTIAL; E-25 DARK×9; context_budget WIRED; B-22 DARK; E-5/H-33 PARTIAL; J-21 PLANNED. | J-25:L60-L70 |
| J25-E014 | E/DEFER | REVISED Tier-1 SHIP NOW: T3, T4, T6, T2. Tier-2 DEFER-WITH H-50/H-51: T1 template + T5 writer coalescing. T7 lands with E-5 un-dark or J-21, whichever first. | J-25:L72-L77 |
| J25-E015 | E/INT | Files: handoff_digest.rs (T1/T2/T5 ~250 LOC); session_dag + context_policy (T3/T4 ~120) + new `context_boundary.rs` ~80; task_runner (T6 ~60); crates/memory (T7 ~80). | J-25:L79-L84 |
| J25-E016 | E/SURFACE | frontend_pair N/A — no new dashboard surface; pressure level + digest events surface through EXISTING H-51/H-9 panels. Justified. | J-25:L85-L86 |
| J25-E017 | E/CAP | Produces: `adjust_boundary_for_invariants` pure fn; DigestSection template; `read_budgeted_section_aware`; `pressure_level` grading; completion gate; relative-floor scoring + fingerprint reconcile. | J-25:L89-L95 |
| J25-E018 | E/INT | Reachable from live session loop: drive_task → context policy tick → trim/digest paths (PRODUCTION). contracts never R7. | J-25:L100-L102 |
| J25-E019 | E/LIMIT | Stub bans: no second digest format; no LLM in T2/T3/T4/T6/T7; no absolute-score floor reintroduction; no unwrap/expect; no blake2/sha2; no re-summarization touching cache prefix (HD-2). | J-25:L103-L106 |
| J25-E020 | E/OBS | Trigger: long fixture crossing pressure level 2 with orphaned tool_use at candidate boundary, then `elai run --emit-rra` replay. Expect digest write with section budgets; adjusted boundary on trim rows. | J-25:L107-L111 |
| J25-E021 | E/TEST | NEGATIVE: orphaned pair with guard DISABLED must produce malformed-request failure (R2 harness check). Agent return with open tasks + gate ON must be `partial`, never DONE. | J-25:L112-L115 |
| J25-E022 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check; rg adjust_boundary non-test caller; unwrap count 0 in context_boundary.rs; J-25-decisions.md exists. | J-25:L117-L123 |
| J25-E023 | E/EVENT | Prefer-reuse H-51 HandoffLifecycleEvent(227) + existing trim/compaction rows; if new needed, +ContextMechanicsEvent Kind BoundaryAdjusted/PressureGraded/CompletionDowngraded/RetrievalRefloored; baseline+1 from LIVE 231/542; net 0..+1. | J-25:L132-L135 |
| J25-E024 | E/DEP | COMPOSES-WITH H-50/H-51/H-6/E-25/SessionDag/context-budget/B-22/E-5/H-33/H-49/H-2/I-12/J-21/J-22. Restates none. Doctrine: HD-2, TD-3, RV-2, CL-1..CL-4, GD-1. | J-25:L129-L139 |
| J25-E025 | E/FLOW | FORK-1 thinking siblings: session::ContentBlock has NO Thinking variant. DEFAULT ROUTE A = tool_use/tool_result un-orphan only for Tier-1 T3. ROUTE B if Thinking variant appears at edit time. | J-25:L141-L147 |
| J25-E026 | E/DEFER | FORK-2: T1/T5 remain DEFER-WITH H-50/H-51; if they land before wave closes, re-scope into follow-up Tier-1. | J-25:L148-L150 |
| J25-E027 | E/FAIL | ABORT-1 inventing second digest format; ABORT-2 LLM call in T2/T3/T4/T6/T7; ABORT-3 modifying context_policy inside T3-only slice; ABORT-4 greenfield undesigned trim seam → SET_ASIDE RECON_NEEDED. | J-25:L152-L162 |
| J25-E028 | E/TEST | MOVE T3/T4/T6/T2 expected observations and ON_FAIL recoveries; Tier-2 T1/T5/T7 EXPECTED_OBSERVATION "not in this execute wave". | J-25:L164-L188 |
| J25-E029 | E/TEST | VERIFY off-agent only: plan_gate_check PASS; wiring_contract_check PASS; production_caller 0 DARK; cargo check GREEN; nextest boundary contract (orphan FAIL disabled / PASS enabled); rg non-test caller. Executor stdout NEVER pass artifact (R4). | J-25:L190-L205 |
| J25-E030 | E/TEST | REDTEAM ATTACK-1..5: disabled guard must FAIL Anthropic invariant; no unauthorized Thinking variant; no absolute floor; no second digest; no LLM adjudication in T6. | J-25:L207-L219 |
| J25-E031 | E/TEST | INTEGRATION_SMOKE per Tier-1 unit + cross-seam smoke after every 2-3 units AND before DONE on HARD input (orphans, pressure crossings, open tasks) via real entrypoint. Full `elai run --emit-rra` deferred until T4 pressure fixture lands. | J-25:L221-L240 |
| J25-A001 | A/EVENT | Whether new ContextMechanicsEvent is needed vs pure reuse of HandoffLifecycleEvent is decide-at-edit. | J-25:L132-L135 |
| J25-A002 | A/FLOW | FORK-1 Thinking variant presence is runtime-observed at edit; default is tool-pair only. | J-25:L141-L147 |
| J25-C001 | C/STATUS | Authoring SSB 231/542 may be stale vs later greps; re-grep W-9. Session_dag line anchors are PLAN-asserted. | J-25:L29-L30, L132-L135 |

Category coverage: capabilities/mechanics ✓; surfaces (existing H-51/H-9 only) ✓; workflows/pressure/completion ✓; schemas/section budgets ✓; config/thresholds ✓; events prefer-reuse ✓; failures/abort ✓; determinism ✓; tests/redteam/smoke ✓; constraints/forks/deferred ✓. No new UI panel.

### J-61 — canvas redirect loop

Source: `docs/thefinalglance/J-plan/J-61_canvas_redirect_loop.md` (184/184 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J61-E001 | E/STATUS | J-61 PLANNED WAVE CFW-4, tier L, EXTENSION (steer half of layered canvas); depends J-60, H-15, J-31, J-12 (PLANNED agent-control), H-49 (PARTIAL preempt). Lever STEERING+CANT-WRECK. | J-61:L1-L5 |
| J61-E002 | E/LIMIT | Does NOT add hand-authoring: new nodes still born in chat (J-32). Category plateaus because manual node-wiring > typing (web-verified 2026-06-20). | J-61:L9-L13 |
| J61-E003 | E/CAP | Closed verb set ONLY: CONNECT(a,b), CUT-EDGE(e), REORDER(node, before/after), REBIND(node, new binding), AGENT-CONTROL(agent, pause\|redirect\|kill). New-node creation REJECTED as canvas verb. No node-delete verb (removal is chat/lifecycle; journal append-only). | J-61:L22-L28 |
| J61-E004 | E/DET | Edit = journaled INTENT command, not mutation. Canvas is read-only on SSB (SsbViewer WAL poll). Redirect submitted as CanvasRedirectIntent to single-writer engine. SSB total order by sequence is conflict model — corrects mis-citation of I-6 as concurrency model (I-6 is context-pane slot). | J-61:L29-L35, L66-L67 |
| J61-E005 | E/FLOW | Cross-altitude translation: PROCESS REORDER re-derives SYSTEM dependencies; affected edges flip broken via J-31 verdict ladder + Kahn cycle detection (phase-3G) when reorder creates cycle or violates wiring. LLM proposes; checker/journal adjudicate (TD-3); LLM never colors an edge. | J-61:L36-L41 |
| J61-E006 | E/RECOVERY | User preempts: pending user intent vs not-yet-committed LLM action on same node → user supersedes. Rides H-49 RV-1 snapshot-before-restore + TaskStop / J-12 intervention. LLM re-derives on user-intent row. No locks, no CRDT. | J-61:L42-L47 |
| J61-E007 | E/LIMIT | Build stays FSM: node build dispatches bound unit through normal planning-workflow FSM; redirect changes plan/order/binding the FSM executes; never executes on canvas. | J-61:L48-L50 |
| J61-E008 | E/DEP | Existing: phase-3B SSB single-writer WAL SHIPPED; SsbViewer READ-ONLY; CanvasUiAction at canvas_ui_action.rs:61 (H-15); InProcessCanvasDispatcher at canvas_dispatcher.rs:101; J-31 system_canvas verdict ladder; Kahn phase-3G; H-49 RV-1 + EditHistoryEvent(226) PARTIAL; J-12 PLANNED. | J-61:L52-L67 |
| J61-E009 | E/DEFER | Tier-1: CONNECT/CUT/REORDER/REBIND + cross-altitude + user-preempt graph edits via H-49 + AGENT-CONTROL=kill via shipped stop. Tier-2: pause/redirect via J-12; mid-token preempt (H-49 turn-grain); multi-user concurrent redirect (J-49 seats). | J-61:L69-L77 |
| J61-E010 | E/SURFACE | frontend: redirect gestures (drag-connect / cut / reorder / rebind / tab agent-control) on J-60 surface emit CanvasRedirectIntent via dispatcher; broken edges render J-31 broken state. cross_lang_diff + svelte-check in-unit. | J-61:L81-L84, L119-L120 |
| J61-E011 | E/INT | Dispatcher: NEW 'redirect' op-namespace on InProcessCanvasDispatcher: connect / cut_edge / reorder / rebind / agent_control via 3-arm tool pattern (catalog+convert+dispatch, production_agent_rpc.rs). | J-61:L85-L87 |
| J61-E012 | E/CAP | NEW `crates/orchestrator/src/canvas_redirect.rs` (~240 LOC): CanvasRedirectIntent + single-writer append + cross-altitude translation + preempt arbiter + J-31 re-derive trigger. FSM dispatch REUSED for build. | J-61:L88-L91 |
| J61-E013 | E/EVENT | +CanvasRedirectIntent Pattern-2; Kind: Connect / CutEdge / Reorder / Rebind / AgentControl. baseline+1 from LIVE 241/550 (re-read W-9; golden-lock 9-tuple). Preempt outcome reuses H-49 EditHistoryEvent (no second action_type). | J-61:L92-L94 |
| J61-E014 | E/INT | Production reachability: dashboard-v2 canvas redirect gestures → InProcessCanvasDispatcher 'redirect' → canvas_redirect.rs via production_agent_rpc. contracts never R7. | J-61:L110-L112 |
| J61-E015 | E/LIMIT | Stub bans: no direct canvas graph mutation; no LLM-set edge verdict; no canvas-side runtime; no ADD-NODE/DELETE-NODE canvas verb; no lock/CRDT; no second SSB writer; no unwrap/expect; blake3 only. | J-61:L113-L118 |
| J61-E016 | E/OBS | Trigger: open canvas on bound plan; REORDER two not-yet-run nodes violating wiring; redirect on node agent is emitting. Expect CanvasRedirectIntent rows in sequence; preempt lands H-49 EditHistoryEvent rollback. | J-61:L121-L123 |
| J61-E017 | E/TEST | Artifact: edge flips broken DETERMINISTICALLY after re-derive (checker/Kahn not LLM); user supersedes in-flight LLM; re-derive under ELAI_SEED. NEGATIVE: mutate without journal FAIL; edge verdict from model FAIL; redirect loses to LLM (queue semantics) FAIL preempt test. | J-61:L124-L124 |
| J61-E018 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check; rg CanvasRedirectIntent; rg graph insert/remove/mutate in dashboard canvas MUST be 0; rg model/LLM verdict MUST be 0; J-61-decisions.md. | J-61:L126-L133 |
| J61-E019 | E/CFG | W-2: ships ON behind kill-switch `ELAI_CANVAS_REDIRECT_DISABLE` (NOT default-OFF enable-flag); registered. | J-61:L141-L141, L147-L148 |
| J61-E020 | E/TEST | ACCEPTANCE checklist: journaled intents; TD-3 verdicts; user preempt + H-49; closed verb set; FSM build; kill-switch; cargo check + svelte-check; no new dependency. | J-61:L135-L143 |
| J61-E021 | E/TEST | Contract `j61_canvas_redirect_contract.rs` (~150 LOC): (a) re-derive from intent; (b) REORDER cycle → broken via Kahn; (c) preempt + H-49; (d) queue-semantics rejected; (e) ADD-NODE refused; (f) production namespace reachability R7. | J-61:L155-L156 |
| J61-E022 | E/DATA | Files: CREATE canvas_redirect.rs; MODIFY canvas_dispatcher (~25), production_agent_rpc (~30), ssb+lock (~40), dashboard canvas gestures (~180); CREATE contract + J-61-decisions.md. | J-61:L158-L165 |
| J61-E023 | E/EVIDENCE | Grounding: SSB/dispatcher/H-15/J-31/Kahn/H-49; I-6 corrected; category web search 2026-06-20; arXiv:2603.22386 ACG framing. | J-61:L167-L170 |
| J61-E024 | E/DEFER | Open: mid-token preempt needs H-49 turn-grain (Tier-2; v1 node/edit grain only). AGENT-CONTROL pause/redirect depends J-12; if slips, v1 ships connect/cut/reorder/rebind+kill only. | J-61:L172-L174 |
| J61-A001 | A/STATE | REORDER with no SYSTEM consequence should be no-op verdict not false breakage — re-derive must distinguish "decoupled" from "broken"; decide verdict mapping at authoring. | J-61:L175-L175 |
| J61-A002 | A/RECOVERY | Exact preempt grain (node/edit vs mid-token) decided at executing. | J-61:L172-L173 |
| J61-C001 | C/STATUS | LIVE baseline cited 241/550 here vs NASA/RLM ~268/577 and J-21/J-24 231/542 — all require re-grep at edit; do not hard-code any stamp. | J-61:L92-L93 |

Category coverage: user-facing gestures/commands/kill-switch ✓; workflows/preempt ✓; schemas/intent kinds ✓; security/single-writer ✓; failures/negatives ✓; determinism/TD-3 ✓; tests/contracts ✓; deferred agent-control ✓. Privacy/compliance not standalone.

### J-62 — env-constraint skill cards + pre-write mandatory-read gate

Source: `docs/thefinalglance/J-plan/J-62_env_constraint_skill_cards_prewrite_gate.md` (129/129 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J62-E001 | E/STATUS | J-62 PLANNED WAVE FABLE-STEAL-1; EXTENSION over context_budget Rules slot + hook_dispatcher PreToolUse + vibegate F-1 + E-43 skills; COMPOSES J-52 LEG 3; parallel-safe with J-63. Effort M. | J-62:L1-L5 |
| J62-E002 | E/FAIL | dashboardRules.md DOES NOT EXIST anywhere in .claude yet design-taste.md and visual-ui-loop.md both reference it and flag React framing while stack is Svelte 5 + Vite + Tauri v2. | J-62:L6-L6, L28-L28 |
| J62-E003 | E/CAP | LEG 1 env-constraint skill cards: author must-read-before-touch env-card per build surface. CANONICAL first card: dashboard-v2 (Svelte 5 runes $state/$props/$derived/$effect, Vite, Tauri v2 IPC #[tauri::command]+ts-rs+invoke, component inventory WindowHost/GraphWindow/panel family/tokens.css, design-token pointers oklch/prefers-reduced-motion/@font-face, capture harness pointers, dashboard-handoff cross_lang_diff). | J-62:L10-L10 |
| J62-E004 | E/DATA | REGISTRY: a card per surface class — dashboard-v2 first; python sidecar lane; Rust-crate lane — authored ONLY as measured pain arises (GD-4, not all at once). Path pattern `.claude/skills/<surface>-env/`. | J-62:L10-L10, L32-L32 |
| J62-E005 | E/FLOW | LEG 2 pre-write mandatory-read gate: on FIRST Write/Edit of session to path matching env-card fingerprint, ensure matching env-card is resident in context_budget Rules/Skill slot BEFORE write accepted. Default inject-then-proceed; mandatory card gets ONE bounded re-dispatch, never hard-kill. | J-62:L12-L12, L17-L17 |
| J62-E006 | E/INT | CRITICAL: does NOT re-implement PathRuleInjector / fingerprint map / Rules-slot dedup (J-52 LEG 3). Reuses PathRuleInjector; adds (a) env-card SOURCE class to priority list and (b) PreToolUse lifecycle wrapper (different event than J-52 PostToolUse). Sequence J-62 AFTER or WITH J-52. | J-62:L12-L12, L15-L15, L22-L23 |
| J62-E007 | E/DEFER | Fallback: if action-type-lock review says J-52 PostToolUse injection is sufficient, LEG 2 degrades to "register env-cards as J-52 source" and J-62 stands on LEG 1 alone. | J-62:L12-L12, L118-L119 |
| J62-E008 | E/LIMIT | Env-card NEVER restates design-taste.md or visual-ui-loop.md — POINTERS only. React-framed card is BLOCK. | J-62:L16-L16, L20-L20, L60-L61 |
| J62-E009 | E/CFG | NO new ELAI_*_ENABLE: default-ON behind kill-switch `ELAI_ENV_CARD_GATE_DISABLE`; disabled gate LOGS disable, never silent no-op. | J-62:L18-L18, L63-L63 |
| J62-E010 | E/CFG | Ships OBSERVE-ONLY first (JD-3/GD-4): injects card + emits SSB, but mandatory-re-dispatch arm activates only after inject/no-op FP rate observed across ≥3 real dashboard-v2 edit sessions. | J-62:L19-L19, L32-L32 |
| J62-E011 | E/DEP | Seams: J-52 PathRuleInjector PLANNED; context_budget WIRED 8 slots; vibegate F-1 PreToolUse WIRED; hook_dispatcher PreToolUse + dispatch_post_tool_use:1971; E-43 skill system WIRED-dark + agentskills_validator; design-taste/visual-ui-loop WIRED; dashboard-handoff + cross_lang_diff WIRED. | J-62:L22-L29 |
| J62-E012 | E/DEFER | Tier-1: dashboard-v2 env-card package + registry convention + PreToolUse wrapper OBSERVE-ONLY. Tier-2: python/Rust cards; mandatory-re-dispatch after FP bound; learned card-relevance score. | J-62:L31-L33 |
| J62-E013 | E/DATA | NEW harness: `.claude/skills/dashboard-v2-env/SKILL.md` + eval/positive.jsonl + negative.jsonl + frontmatter via agentskills_validator. NEW one-line conditional-routing.md Dashboard/frontend pointer. | J-62:L35-L36 |
| J62-E014 | E/INT | MODIFY path_rule_inject.rs (~15 LOC source class); vibegate.rs OR hook_dispatcher.rs PreToolUse (~30 LOC wrapper). NO new module unless wrapper cannot fit F-1 compound path (material fork R-4). | J-62:L37-L37 |
| J62-E015 | E/EVENT | +EnvCardGated Pattern-2; Kind: CardInjected / AlreadyResident / NoMatchingCard / MandatoryReDispatch. baseline+1; SCHEMA computed. Prefer collapse into J-52 PathRuleInjected wider Kind (net 0) if lock review allows — env-card decision must stay journal-distinct either way. | J-62:L38-L38 |
| J62-E016 | E/SURFACE | frontend_pair: fold env-card-injection count into J-52 HookEnforcementPanel tile (NO second panel — GD-1); #[tauri::command] + ts-rs + cross_lang_diff if field added. | J-62:L39-L39, L66-L67 |
| J62-E017 | E/INT | Production reachability: PreToolUse on FIRST real Write/Edit to `.elai_cc/crates/dashboard-v2/**` → EnvCardGated::CardInjected + card in Rules slot. contracts never R7. | J-62:L55-L57 |
| J62-E018 | E/LIMIT | Stub bans: no PathRuleInjector re-impl; no taste/loop restatement; no React card; no hard-kill; no new ENABLE flag; no blocking before ≥3 sessions; no unwrap/blake2/sha2 (blake3 fingerprint reusing J-52). | J-62:L58-L65 |
| J62-E019 | E/OBS | Trigger: first session Write/Edit under dashboard-v2 path with no card resident. Expect CardInjected (path + card id); AlreadyResident on later edit; NoMatchingCard unregistered; artifact = card in Rules slot + HookEnforcementPanel count. | J-62:L68-L70 |
| J62-E020 | E/TEST | NEGATIVE FAIL: dashboard edit with no card and no SSB; second edit re-injects despite resident; silent no-op when DISABLE set; module re-implements J-52 resolver. | J-62:L71-L71 |
| J62-E021 | E/TEST | DETERMINISTIC_CHECK: wiring_contract; skill file exists; agentskills_validator; rg Svelte5/$state/tauri::command; rg design-taste/visual-ui-loop pointers ≥2; PathRuleInjector reuse; EnvCardGated/PathRuleInjected; no unwrap/blake2/sha2; no new ENABLE; J-62-decisions.md. | J-62:L73-L85 |
| J62-E022 | E/TEST | Contract j62_env_card_gate_contract.rs (~70 LOC): (a) CardInjected; (b) AlreadyResident dedup; (c) NoMatchingCard; (d) DISABLE logs; (e) calls PathRuleInjector. Plus harness real session check. | J-62:L101-L103 |
| J62-E023 | E/EVIDENCE | Fable source is ILLUSTRATIVE UNVERIFIED PROVENANCE (leaked prompt); PATTERN independently corroborated by Anthropic public Skills + Agent SDK. Design grounded in ELAI seams not the leak. | J-62:L110-L111 |
| J62-E024 | E/DEFER | Riders: python/Rust env-cards; mandatory-re-dispatch arm; learned relevance; auto-regenerate card from component graph. | J-62:L123-L127 |
| J62-A001 | A/DATA | Whether env-card belongs under .claude/skills (lean) vs .claude/rulebook is decide-at-executing. | J-62:L121-L121 |
| J62-A002 | A/EVENT | EnvCardGated new variant vs PathRuleInjected wider Kind is lock-time decision. | J-62:L38-L38 |
| J62-A003 | A/FLOW | Pre-write vs post-edit sufficiency is open; material relocation is handoff fork R-4. | J-62:L118-L119 |
| J62-C001 | C/STATUS | SCHEMA baseline note ~268/~577 from J-0_INDEX; must re-grep both at edit. | J-62:L115-L115 |

Category coverage: skill package surface ✓; HookEnforcementPanel ✓; kill-switch flag ✓; PreToolUse workflow ✓; events ✓; security/graceful disable ✓; tests ✓; deferred cards ✓. Privacy not standalone.

### J-73 — retrieval-abstention gate

Source: `docs/thefinalglance/J-plan/J-73_retrieval_abstention_gate.md` (64/64 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J73-E001 | E/STATUS | J-73 Track J context-retrieval wave, lever CHEAPER, grounding STRONG (Repoformer arXiv:2403.10059), nature EXTENSION repurposing answer-abstention infra, effort M, WAVE CR-2, depends none. | J-73:L1-L3 |
| J73-E002 | E/FAIL | Premise: ELAI ALWAYS retrieves in retrieval tier — codegraph-confirmed 2026-06-23 NO should_retrieve gate exists. | J-73:L5-L5 |
| J73-E003 | E/EVIDENCE | Repoformer: self-assessment P(retrieve) threshold T=0.15 function / 0.20 line/API yields >70% speedup dense, >80% abstention precision — DESIGN-rationale; ELAI threshold calibrated on ELAI data (ID-5). | J-73:L5-L5, L51-L52 |
| J73-E004 | E/CAP | should_retrieve GATE in pipeline BEFORE BM25/ONNX cascade. Signal from EITHER sidecar (Qwen2.5-Coder `<cc>`/abstain special token — Tier-2) OR default cheap calibrated heuristic reusing abstention_score + conformal_calibration (Abstain enum, decide_low_score_abstains, agent_pool low_score_routes_to_abstain). | J-73:L8-L8 |
| J73-E005 | E/CFG | Per-task-type threshold T (function vs line/API) lives near existing FAST/STANDARD/DEEP tier router (query_class). Named constants, not magic numbers. | J-73:L8-L8, L14-L14, L24-L24 |
| J73-E006 | E/DET | Gate is DETERMINISTIC at decision time (TD-3): calibrated number + config threshold; LLM does not adjudicate should_retrieve. Gated-OFF retrieval is journaled for auditable savings and FP (skipped-but-needed) rate. | J-73:L9-L9 |
| J73-E007 | E/LIMIT | EXCLUDED: training NEW retrieval-decision head from scratch (cheap path reuses calibration; special-token is Tier-2 RunPod-gated); removing cascade stages (that is J-75, user-gated). | J-73:L10-L10, L43-L43 |
| J73-E008 | E/DEP | Existing: abstention_score.rs + conformal_calibration.rs (answers); query_class tier routing phase-3K; CATRAG+A-RAG cascade in `pipeline.rs`; D-16 UniCR conformal tau DARK (compose, do not re-author). | J-73:L12-L16 |
| J73-E009 | E/DEFER | Tier-1 SHIP NOW: should_retrieve in pipeline.rs reusing abstention calibration + T config; OBSERVE-ONLY first (GD-4 journal would-skip without skipping until FP bounded ≥3 real runs), then enforce. Tier-2: Repoformer-style special-token head remote-GPU-gated. | J-73:L18-L20 |
| J73-E010 | E/INT | Integration: pipeline.rs ~60 LOC before cascade; T config near query_class. | J-73:L22-L24 |
| J73-E011 | E/EVENT | +RetrievalAbstained Pattern-2; Kind: GatedSkip / GatedProceed / AbstainOverride. baseline+1 from LIVE ~269/578 (post-J-66). PREFER-REUSE: if QueryTrace decision-stage field exists, add abstain verdict there (net 0). | J-73:L25-L25 |
| J73-E012 | E/SURFACE | frontend_pair: retrieval-skipped rate on KnowledgeDrawer (B-29) or W-6-justified absence. No cross_lang pair (Rust-internal). | J-73:L26-L26, L45-L45 |
| J73-E013 | E/INT | Production reachability: real `elai` retrieval query/task path entering pipeline.rs. contracts never R7. | J-73:L36-L38 |
| J73-E014 | E/LIMIT | Stub bans: no LLM adjudication; no enforced skip before FP bounded ≥3 runs; no new abstention calibration impl (IC-13 one-calibration-impl); no cascade-stage removal; no default-OFF ENABLE without allowlist+promotion (R9/W-2). | J-73:L39-L44 |
| J73-E015 | E/OBS | Trigger: retrieval query where local context already suffices (within-file completion). Expect RetrievalAbstained Kind=GatedSkip with score+T (or QueryTrace decision-stage). Cascade did NOT fire; answer from local context. | J-73:L46-L49 |
| J73-E016 | E/FAIL | NEGATIVE fail-closed: score BELOW T proceeds full cascade (missing/uncertain signal RETRIEVES, never silently skips). OBSERVE-ONLY journals would-skip without skipping. | J-73:L49-L49 |
| J73-E017 | E/DEP | COMPOSES-WITH abstention_score/conformal_calibration, D-16, query_class, QueryTrace, J-72 (gate decides whether to invoke adapted retriever). Restates none. | J-73:L57-L58 |
| J73-E018 | E/OBS | Open risk: answer-abstention calibration distribution ≠ "answerable from local context" — first checkbox is measuring FP on held-out query set before enforce. | J-73:L60-L61 |
| J73-A001 | A/EVENT | RetrievalAbstained new vs QueryTrace decision-stage field is decide-at-edit. | J-73:L25-L25 |
| J73-A002 | A/SURFACE | KnowledgeDrawer skip-rate tile vs W-6 absence is decide-at-edit. | J-73:L26-L26 |
| J73-A003 | A/CFG | Exact ELAI-calibrated T values (vs paper 0.15/0.20) unspecified until local measurement. | J-73:L8-L8, L51-L52 |
| J73-C001 | C/STATUS | LIVE ~269/578 post-J-66 differs from other units' stamps; re-grep W-9. | J-73:L25-L25, L55-L55 |

Category coverage: should_retrieve capability ✓; KnowledgeDrawer surface ✓; cascade entry workflow ✓; events ✓; observe-only config ✓; fail-closed ✓; deferred sidecar head ✓. No privacy surface.

### J-86 — GRC-v3 receipt-gated candidate ladder

Source: `docs/thefinalglance/J-plan/J-86_grcv3_receipt_gated_candidate_ladder.md` (117/117 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J86-E001 | E/STATUS | J-86 PLANNED WAVE GRC-3; ACTIVATION of I-4 + D-2 + I-24 + I-26 under receipt-first rule; depends J-83; sequence after J-84/J-85. Effort ~M. | J-86:L1-L8 |
| J86-E002 | E/CAP | Candidate system = I-4 evidence-ranked search ladder (DARK GS-6) over J-83 BeliefState carrier: branch candidates on SessionDag, close each with D-10-routed domain receipt; receipt-passing ADVANCES (commit+freeze); rest STAY/backtrack. | J-86:L5-L5, L14-L15 |
| J86-E003 | E/FLOW | SL-1 selector hierarchy: deterministic RECEIPT (D-10: F-4/D-11/J-1/H-trading) > VOTE over independent samples (receipt-ABSENT fallback ONLY) > LLM self-eval (BANNED). When receipt exists it is selector full stop (DC-3, IC-6). | J-86:L15-L15 |
| J86-E004 | E/FLOW | SL-2 receipt-absent vote rung (IMPLICATION 8 ADDITION): only when NO deterministic receipt AND independent-error task; NEVER LLM self-eval; NEVER for factual-recall (routes to retrieval — shared training bias / "Consensus is Not Verification"). | J-86:L16-L16 |
| J86-E005 | E/CFG | SL-3 early-stop (IMPLICATION 9): branch until receipt confirms OR confidence converges OR J-84 depth budget caps — never fixed best-of-N (CGES ~69% sample cut design-rationale). | J-86:L17-L17 |
| J86-E006 | E/CAP | SL-4 PRM Tier-2-under-receipt (IMPLICATION 27): activate dark D-2 PPM sidecar + I-22 progress rewards for intermediate no-receipt steps; ALWAYS under Tier-1 receipt; guarded by I-24 isomorphic canary (over-optimization past ~400 samples ADD-6). | J-86:L18-L18 |
| J86-E007 | E/RECOVERY | SL-5 backtrack (IMPLICATION 14): candidate step receipt FAIL → backtrack to last receipt-passing J-83 entry over SessionDag/H-49; no bespoke tree-search engine. | J-86:L19-L19 |
| J86-E008 | E/LIMIT | HARD BOUNDARY: selector is RECEIPTS never vote-when-receipt-exists and never LLM self-eval (IC-6, DC-3, F-4, D-13); NO MCTS-first (ladder best-of-N→tournament→PUCT, mass at bottom rungs, full tree-search rare classifier-gated J-84); does NOT re-author I-4/D-2/I-24/I-26/H-49. | J-86:L5-L5 |
| J86-E009 | E/DEP | Prerequisites: J-83 carrier; I-4 search/ladder.rs run_best_of_n (:237/277) + SearchNodeEvent + i4 contract (GS-6); D-2 PPM port 8101 + I-22 + I-24 (GS-9); I-26 early-stop budget; SessionDag + H-49 (GS-10); D-10; I-7 escalate. | J-86:L7-L7, L21-L25 |
| J86-E010 | E/INT | Tasks: activate I-4 receipt-first (~60 LOC) un-dark kill-switch+allowlist; vote rung (~40); early-stop (~25); PRM Tier-2 (~40) canary-guarded; backtrack (~30); J-86-decisions.md. | J-86:L34-L40 |
| J86-E011 | E/INT | Produces edits to search/ladder.rs + D-2/I-22/I-24 activation seam. Consumes I-4/D-2/I-22/I-24/SessionDag/H-49/J-83/D-10/I-26/I-7/I-8 without reimplementation. | J-86:L42-L49 |
| J86-E012 | E/INT | Production reachability: J-84 WIDTH tier → branch over J-83 select by receipt; receipt FAIL → backtrack SessionDag/H-49. contracts never R7. | J-86:L50-L53 |
| J86-E013 | E/LIMIT | Stub bans: no vote-as-selector when receipt exists; no LLM self-eval selector; no vote for recall; no PRM as Tier-1; no uncapped width; no MCTS-first/bespoke tree-search; no re-impl of substrates; no unwrap. | J-86:L54-L58 |
| J86-E014 | E/SURFACE | frontend_pair: existing search-ladder / ConvergencePanel surfaces candidate branches + selected receipt + early-stop point — decide tile at edit (W-4); justified N/A if rides I-4/I-27 surfaces. | J-86:L59-L60 |
| J86-E015 | E/OBS | Trigger integration smoke R10: hard branching WITH deterministic receipt; open-ended NO receipt; recall query. Assert receipt-selection / independent-sample-vote / retrieval-not-vote respectively. | J-86:L61-L62 |
| J86-E016 | E/EVENT | expect_ssb: reuse SearchNodeEvent (I-4) / ProgressRewardScored (I-22) / PrmCanaryEvent (I-24) — net 0 from LIVE 268/577 (Kind add decide-at-edit W-9). | J-86:L63-L63 |
| J86-E017 | E/TEST | Artifact: receipt-passing advances rest pruned-never-promoted; receipt-absent open-ended uses vote NOT self-eval; recall→retrieval; fail backtracks; NEGATIVE: LLM grading own paths MUST NOT happen; PRM never Tier-1. | J-86:L64-L64 |
| J86-E018 | E/TEST | DETERMINISTIC_CHECK: wiring_contract; rg receipt/vote/early-stop/backtrack/tier-2; rg self-eval/vote-recall MUST 0; rg mcts MUST 0; production_caller; J-86-decisions.md. | J-86:L66-L74 |
| J86-E019 | E/CFG | W-2: I-4/D-2 un-darked behind kill-switches with wire-dark-allowlist rows + promotion plan (R9); not terminal default-OFF. D-2 PRM remote-sidecar W-10-gated separately; ladder must not DEPEND on PRM being up. | J-86:L86-L88, L111-L111 |
| J86-E020 | E/TEST | Contract j86_candidate_ladder_contract.rs (~90 LOC): (a) prune-never-promoted; (b) vote on open-ended; (c) recall→retrieval; (d) reject self-eval; (e) early-stop; (f) PRM under receipt canary; (g) backtrack; (h) over-width stop; (i) corrupt receipt → abstain/escalate not silent self-eval. | J-86:L93-L94 |
| J86-E021 | E/DATA | Files: MODIFY ladder.rs ~180 LOC; D-2/I-22/I-24 seam ~40; wire-dark-allowlist; CREATE contract + decisions. | J-86:L96-L101 |
| J86-E022 | E/EVIDENCE | Grounding: reasoning-engine-grc-v3 IMPLICATIONS 7/8/9/13/14/15/27 + ADD-6; GS-6/9/10; arXiv self-consistency 2203.11171, Consensus-not-Verification 2603.06612, ToT 2305.10601, TTC 2408.03314, CGES 2511.02603. | J-86:L103-L107 |
| J86-A001 | A/FLOW | independent-error vs factual-recall classification for SL-2 must be deterministic (reuse J-84 TYPE axis / D-10); verify routing forecloses vote-for-recall structurally. | J-86:L110-L110 |
| J86-A002 | A/SURFACE | ConvergencePanel tile content is decide-at-edit. | J-86:L59-L60 |
| J86-A003 | A/EVENT | Whether a new Kind is needed on reused events is decide-at-edit. | J-86:L63-L63 |
| J86-I001 | I/DEP | RLM map RS-3 explicitly names J-86 as activator of SearchLadder WIDTH; J-108 is DEPTH sibling copying its idiom — J-86 is the WIDTH half of that design. | RLM:L64-L67; J-86:L5-L8 |

Category coverage: ladder capability ✓; ConvergencePanel surface ✓; selector hierarchy workflow ✓; kill-switches ✓; events prefer-reuse ✓; security/anti-self-eval ✓; backtrack recovery ✓; tests ✓; PRM deferred live ✓.

### J-101 — transformation-surviving taint binding

Source: `docs/thefinalglance/J-plan/J-101_transformation_surviving_taint_binding.md` (139/139 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J101-E001 | E/STATUS | J-101 PLANNED WAVE 1; technical injection-hardening half that K-13 OUT-OF-K recorded NOT authored; EXTENSION on E-37/G-1 (not in-place amend); effort L (3 legs); SSB prefer-reuse TaintFlow, net 0..+1. Lever CANT-LAUNDER-TRUST. | J-101:L1-L7 |
| J101-E002 | E/SEC | Companion to K-13 (OWASP control-mapping PROVE/PACKAGE). Threats: LLM01/LLM05/ASI01/ASI06/ASI07. Three legs WI-1/WI-2/WI-3 ship as ONE extension file. Source owasp-llm-agentic-gap-analysis-2026-06-25.md §D/E/F/G. | J-101:L6-L7 |
| J101-E003 | E/FAIL | Residual gap: per-argv binding is VERBATIM SUBSTRING MATCH (`lookup_for_argv`, ≥ MIN_TAINT_MATCH_LEN, same-process). Misses (1) TRANSFORMED/SUMMARIZED/PARAPHRASED untrusted (dominant indirect-injection path); (2) CROSS-PROCESS sinks; (3) non-McpToolRegistry MCP transports. | J-101:L10-L13 |
| J101-E004 | E/SEC | Provenanced<T> TYPE exists but is NOT threaded end-to-end through LLM-output→argv; wired mechanism is substring registry MVP. G-7 lethal-trifecta SESSION gate (`enforce_trifecta_quarantine_gate` production_agent_rpc.rs:334) is existing backstop for paraphrase case — lean on it. | J-101:L14-L14 |
| J101-E005 | E/SEC | WI-1a PRIMARY: strengthen trifecta gate — if untrusted content entered context AND destructive sink reached AND egress/exfil capability present => HITL/quarantine REGARDLESS of per-value match. Default-ON, kill-switchable, emits existing SSB block row. | J-101:L16-L17, L47-L47 |
| J101-E006 | E/SEC | WI-1b SECONDARY: taint-WINDOW heuristic — mark argv model produced AFTER consuming untrusted context in SAME turn (span/turn window) as DERIVED-UNTRUSTED provenance so paraphrase carries trust. TD-3 deterministic; LLM does not adjudicate. OBSERVE-ONLY first (GD-4) measure FP on ten-benign flows before defaulting denial ON. | J-101:L18-L18, L48-L48 |
| J101-E007 | E/SEC | WI-1c FAST PATH: keep substring gate (`enforce_taint_sink_gate` :224) as cheap exact-match first check. | J-101:L19-L19, L49-L49 |
| J101-E008 | E/SEC | ASI06 DECISION default: treat Recallable-written-after-an-untrusted-ingest-in-the-same-session as derived-untrusted under the window (fail-closed, conservative). capability_gate.rs:96 currently does NOT block Recallable. | J-101:L20-L20, L49-L49 |
| J101-E009 | E/SEC | WI-2 cross-process: VERIFY `taint_flow.rs:472 process_global_slot_is_shared` FIRST; if gap confirmed, propagate Provenance across Tier-C postcard IPC. Scope CONFIRMED gap only; if already covered, degrade to assertion contract + note. | J-101:L22-L22, L50-L50 |
| J101-E010 | E/CAP | WI-3 E-37 perimeter ACTIVATION: build recorded-ABSENT `output_sanitizer.rs` + `output_classifier.rs` + egress allowlist (compose G-3 IP-pin) + AgentDojo CI job with recorded baseline. | J-101:L24-L24, L51-L51 |
| J101-E011 | E/LIMIT | WI-4 NUMBER owned by K-13 (SAFEDELTA AgentDojo arm [BENCH-PENDING] F-2b RunPod). J-101 ships measurable fixture + contract (paraphrase BLOCKED, ten-benign FP green); do NOT fabricate injection number (GD-3). | J-101:L26-L26 |
| J101-E012 | E/DEP | Existing WIRED: Provenance/Provenanced/from_file_screened (G-1, fail-closed empty root/`..`); capability_gate allow_tool_with_origin; both production gates default-ON; g1_taint_propagation_contract (incl ten_benign FP); E-37 PARTIAL Spotlighting+PromptArmor with absent sanitizer/classifier/egress/CI; G-3; F-3; I-1; G-6 BENCH-PENDING. live ~268/577. | J-101:L28-L37 |
| J101-E013 | E/EVIDENCE | Mandatory reads before coding: OWASP analysis §D full + E/F/G; provenance.rs; capability_gate.rs:96; production_agent_rpc.rs:224/334/~234/:2321; taint_flow.rs:472; ipc-types postcard; g1 contract; E-37 plan; G-6 AgentDojo arm shape. | J-101:L39-L44 |
| J101-E014 | E/DATA | Decisions artifact: workflow/reasoning/J-101-decisions.md records ASI06 Recallable DECISION + WI-2 verify finding + WI-1 window FP observe-only delta. | J-101:L52-L52 |
| J101-E015 | E/INT | Production reachability (keystone): live `elai run` where model reads poisoned doc, PARAPHRASES malicious instruction, paraphrase flows to destructive sink → window/trifecta BLOCKS. Cross-process sink blocks. Seeded exfil host blocked by WI-3. contracts never R7. | J-101:L67-L71 |
| J101-E016 | E/LIMIT | Stub bans: no prose system-prompt guard primary; no full CaMeL interpreter (Section F DEFERRED); no rebuild working process_global_slot; no fabricated injection number; no default-ON WI-1b before FP measured; new kill-switch gets wire-dark row; no LLM adjudicating window. | J-101:L72-L78 |
| J101-E017 | E/SURFACE | frontend_pair: taint-block + quarantine events surface on existing safety/SSB timeline; AgentDojo CI baseline is CI artifact not dashboard tile. W-6-justified (security gate, no new UI) or decide injection-defense tile at edit (W-4). | J-101:L79-L80 |
| J101-E018 | E/OBS | Trigger: AgentDojo/SAFEDELTA paraphrase fixture; cross-process untrusted→sink; model output seeded exfil host. expect_ssb existing taint-flow/TaintSinkBlocked/quarantine rows; net 0..+1. | J-101:L81-L83 |
| J101-E019 | E/TEST | MEASURED done-gate: paraphrase BLOCKED AND ten-benign FP GREEN (extend g1 contract). NEGATIVE: WI-1b window-OFF LETS paraphrase through (proves load-bearing); benign same-turn-after-read NOT blocked. | J-101:L84-L84 |
| J101-E020 | E/TEST | DETERMINISTIC_CHECK: wiring_contract; production_caller on production_agent_rpc + taint_flow; rg derived-untrusted/taint-window; rg enforce_trifecta; output_sanitizer+classifier exist; paraphrase case in g1 contract; AgentDojo CI path; J-101-decisions.md. | J-101:L86-L96 |
| J101-E021 | E/TEST | ACCEPTANCE WI-1a/b/c + measured paraphrase; WI-2 verify-first; WI-3 perimeter files + CI + seeded exfil block; no fabricated number; cargo check; auditor PASS elai-safety + orchestrator (+contracts). | J-101:L98-L104 |
| J101-E022 | E/TEST | Tests: EXTEND g1_taint_propagation_contract (+~60 LOC paraphrase/window-OFF/FP/Recallable/fail-closed); CREATE j101_cross_process_taint_contract (~50); CREATE j101_e37_perimeter_contract (~50). | J-101:L111-L114 |
| J101-E023 | E/DATA | Files: MODIFY production_agent_rpc / taint_flow / capability_gate / provenance; CREATE output_sanitizer + output_classifier + egress allowlist + AgentDojo CI; EXTEND g1 + two j101 contracts; CREATE decisions. | J-101:L116-L121 |
| J101-E024 | E/EVIDENCE | Grounding: OWASP analysis codegraph-grounded 2026-06-25; CaMeL arXiv:2503.18813 §3.1 EXTERNAL design-rationale for hybrid; OWASP LLM01/05 + ASI01/06/07. | J-101:L123-L126 |
| J101-E025 | E/DEP | K-13 owns WI-4 SAFEDELTA RUN + WI-5 mapping doc; J-101 is technical hardening. Pairs with J-102 (config-ingress; J-101 is untrusted DATA into sinks). | J-101:L128-L128, L136-L137 |
| J101-E026 | E/FAIL | Open risk: over-broad taint window blocks benign same-turn-after-read (RTBAS 33% utility lesson). Mitigation = span-bounded + OBSERVE-ONLY + ten-benign hard gate. | J-101:L130-L131 |
| J101-A001 | A/SEC | ASI06 Recallable posture confirm at authoring vs user (conservative default may over-block legitimate memory recall). | J-101:L133-L133 |
| J101-A002 | A/SURFACE | Whether to add injection-defense tile beyond safety/SSB timeline is W-4 decide-at-edit. | J-101:L79-L80 |
| J101-A003 | A/INT | WI-2 may already be covered by process_global_slot — leg may be assertion-only. | J-101:L132-L132 |
| J101-C001 | C/STATUS | live ~268/577 must be re-grepped; do not treat K-13 SAFEDELTA number as already measured. | J-101:L26-L26, L37-L37 |

Category coverage: security hardening ✓; safety/SSB timeline surface ✓; trifecta/window workflows ✓; observe-only config ✓; events prefer-reuse ✓; failures/FP guard ✓; tests/contracts ✓; deferred CaMeL ✓; K-13 metric ownership ✓. Privacy via taint not separate privacy policy.

## Cross-file interaction records

| IX ID | Class | Interaction | Source atoms |
|---|---|---|---|
| IX-001 | E | NASA map SG-2/NG-1 forbids rebuilding F-4 temporal gate as J-66 monitor; J-65/J-69 are siblings of F-4, not replacements. | NASA-E009, NASA-E022 |
| IX-002 | E | NASA SG-4 makes J-12 intervention the substrate J-67 extends; J-61 AGENT-CONTROL pause/redirect also depends on J-12 (Tier-2). | NASA-E011; J61-E009, J61-E024 |
| IX-003 | E | NASA SG-5 + NG-2: J-69 consumes I-18/J-22 chain; never a competing chain producer. | NASA-E012, NASA-E023 |
| IX-004 | E | RLM RS-3 names J-86 as WIDTH activator of SearchLadder; J-108 is DEPTH sibling of the same idiom. | RLM-E013; J86-E001, J86-I001 |
| IX-005 | E | RLM RC-1/RC-2 + RG-6 reject free-Python RLM and model-chosen orchestration; constrained within-step + HippoRAG primary only. | RLM-E005, RLM-E010, RLM-E019, RLM-E020 |
| IX-006 | E | J-25 T7 relative-floor + fingerprint mechanics are explicit input to J-21 retrieval-time leg; both compose E-5/H-33. | J25-E011; J21-E007, J21-E012 |
| IX-007 | E | J-21 raw-evidence tier carries G-1 taint provenance; J-101 hardens transformation-surviving taint on the data plane that memory may later re-inject. | J21-E008, J21-E021; J101-E001, J101-E006 |
| IX-008 | E | J-24 size pre-assessment triggers H-6 compaction on context_budget slots; J-25 T4 pressure ladder is the H-50 trigger that also composes those ContextSlot floors. | J24-E007; J25-E007, J25-E008 |
| IX-009 | E | J-24 activates H-4/H-34 tool-schema frugality; J-25 T5 fork snapshot freezes tool_defs for prefix-cache alignment (HD-2). | J24-E006; J25-E009 |
| IX-010 | E | J-61 steers the J-60 canvas with closed verbs; J-31 verdict ladder + Kahn are shared with CONNECT/CUT/REORDER breakage. | J61-E003, J61-E005, J61-E008 |
| IX-011 | E | J-61 preempt reuses H-49 EditHistoryEvent; J-25 rider A5 also confirms H-49 RV-2 microcompaction without owning it. | J61-E006, J61-E013; J25-E012 |
| IX-012 | E | J-62 PreToolUse env-card gate reuses J-52 PathRuleInjector; J-52 PostToolUse is the sibling lifecycle. | J62-E005, J62-E006 |
| IX-013 | E | J-62 dashboard-v2 env-card supplies the missing doctrine design-taste/visual-ui-loop already assumed; conditional-routing pointer mechanized by LEG 2. | J62-E002, J62-E003, J62-E013 |
| IX-014 | E | J-73 should_retrieve gates entry to the same knowledge pipeline `query_with_tier` that RLM RS-1 and J-86/J-108 retrieval paths consume. | J73-E004, J73-E010; RLM-E011 |
| IX-015 | E | J-86 selector hierarchy (receipt > vote > banned self-eval) implements the trust-spine IC-6/DC-3 that NASA ND-3 UNKNOWN=FAIL mirrors at gate altitude. | J86-E003; NASA-E018 |
| IX-016 | E | J-86 backtrack over SessionDag/H-49 and J-61 preempt over H-49 share reversibility substrate without sharing semantics (candidate fail vs user intent). | J86-E007; J61-E006 |
| IX-017 | E | J-101 WI-1 strengthens G-7 trifecta that sits beside G-1 per-argv gate; J-21 write quarantine is memory-plane cousin of retrieval-time poison defense, not the sink gate. | J101-E005, J101-E007; J21-E007 |
| IX-018 | E | J-101 WI-3 activates E-37 perimeter files; SAFEDELTA AgentDojo number remains K-13's row, not J-101's done claim. | J101-E010, J101-E011, J101-E025 |
| IX-019 | E | Multiple units prefer-reuse SSB events (J-21 MemoryValidationEvent; J-24 BehaviorSteeringEvent; J-25 HandoffLifecycle; J-61 CanvasRedirectIntent; J-62 EnvCardGated; J-73 RetrievalAbstained; J-86 SearchNode/PRM; J-101 TaintFlow) and all require live baseline re-grep (W-9). | J21-E014; J24-E011; J25-E023; J61-E013; J62-E015; J73-E011; J86-E016; J101-E001 |
| IX-020 | E | Observe-only-first pattern (GD-4/JD-3) recurs: J-24 steering default-ON after ≥3 runs; J-62 mandatory re-dispatch after ≥3 sessions; J-73 enforce after FP bound; J-86 PRM W-10-gated; J-101 WI-1b window denial after ten-benign FP. | J24-E014; J62-E010; J73-E009; J86-E019; J101-E006 |
| IX-021 | E | User-facing surface cluster for dashboard DESIGN.md: memory-hub provenance (J-21); runtime/context steering+budget (J-24); existing H-51/H-9 pressure (J-25); canvas redirect gestures+broken edges+Tauri command (J-61); HookEnforcementPanel env-card counts (J-62); KnowledgeDrawer skip rate (J-73); ConvergencePanel candidate ladder (J-86); safety/SSB taint timeline (J-101). | J21-E015; J24-E012; J25-E016; J61-E010; J62-E016; J73-E012; J86-E014; J101-E017 |
| IX-022 | E | Kill-switches / flags as operator surfaces: ELAI_CANVAS_REDIRECT_DISABLE (J-61 default-ON); ELAI_ENV_CARD_GATE_DISABLE (J-62 default-ON); ELAI_SEARCH_LADDER_ENABLE / ELAI_PPM_ENABLE (J-86 activation, RLM RS-3); --no-steering boot (J-24). | J61-E019; J62-E009; J86-E019; RLM-E013; J24-E002 |
| IX-023 | I | J-25 T6 completion gate (partial on open tasks) and J-86 candidate prune-never-promoted both enforce "incomplete work is not DONE" at different altitudes (task return vs beliefstate selection). | J25-E010; J86-E002, J86-E017 |
| IX-024 | I | NASA ND-1 wall-clock ban and J-73/J-86 deterministic thresholds share the rule that runtime gates use counts/calibrated scores, never LLM wall-clock adjudication. | NASA-E016; J73-E006; J86-E003 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | conflict/status | Live SSB baseline stamps disagree across sources (231/542, 241/550, ~268/577, ~269/578). NASA/RLM compute formula is authoritative method; every unit must re-grep at edit. | NASA-C001; J21-C001; J24-C001; J61-C001; J73-C001; J101-C001 |
| AC-002 | ambiguity/event | Multiple prefer-reuse vs +1 decisions open (MemoryValidationEvent, BehaviorSteeringEvent/ToolSchemaDeferred, ContextMechanicsEvent, EnvCardGated vs PathRuleInjected, RetrievalAbstained vs QueryTrace, J-86 Kind). | J21-A001; J24-A001; J25-A001; J62-A002; J73-A001; J86-A003 |
| AC-003 | ambiguity/surface | Several frontend pairs are "decide at edit or W-6 absence" (memory-hub, runtime steering tile, KnowledgeDrawer skip, ConvergencePanel, injection-defense tile). | J21-A002; J24-A002; J73-A002; J86-A002; J101-A002 |
| AC-004 | ambiguity/config | J-24 alpha and steering magnitude; J-73 local T values; J-61 decoupled-vs-broken mapping; J-62 skills-vs-rulebook home. | J24-A003; J73-A003; J61-A001; J62-A001 |
| AC-005 | dependency | J-61 Tier-2 agent-control and mid-token preempt depend on PLANNED J-12 and PARTIAL H-49 turn-grain. | J61-E009, J61-E024, J61-A002 |
| AC-006 | dependency | J-62 LEG 2 depends on J-52 PathRuleInjector existing; fallback drops to LEG 1 only. | J62-E006, J62-E007, J62-A003 |
| AC-007 | conflict/scope | RLM RC-1 revises feature-map blanket RLM DEFER into constrained within-step only; free-Python remains anti-goal. | RLM-C001 |
| AC-008 | ambiguity/security | J-101 ASI06 Recallable default is conservative; user confirmation may change over-block posture. WI-2 may already be covered. | J101-A001, J101-A003 |
| AC-009 | ambiguity/flow | J-25 FORK-1 Thinking sibling route depends on ContentBlock variant at edit time. | J25-A002 |
| AC-010 | evidence | J-62 Fable source is illustrative/unverified provenance; pattern is independently corroborated. | J62-E023 |
| AC-011 | PLAN-asserted | NASA SG consumer seams and many J-25 SessionDag line anchors are PLAN-asserted; re-grep required. | NASA-A001; J25-C001 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/10-J-SUMMARY_INDEX.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms. The two grounding maps are essentially absent from per-unit summary entries (path inventory only); their atoms are treated as full ADD relative to that inventory.

### NASA grounding map deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-NASA-001 | **ADD** full W-7 map contents: live SCHEMA 268/577 formula, 13 paper→unit interface bindings, SG-1..SG-8 seams, ND-1..ND-4 doctrine, anti-slopsquat dep pins, NG-1..NG-4 corrections. | 10-J:L1481 path inventory only; J-65..J-71 summary blocks lack map internals | NASA-E001–NASA-E025 |
| SD-NASA-002 | **CORRECT** any residual 241/550 feature-map stamp: wave baseline is 268/577; 241/550 is STALE. | 10-J path inventory; cross-cutting stamps | NASA-E004, NASA-C001 |
| SD-NASA-003 | **ADD** F-4 is the test_passed/stdout_hash gate; J-66 must not rebuild it; GROUP F extends I-18/J-22 only. | partial J-66/J-69 entries | NASA-E009, NASA-E022, NASA-E023 |

### RLM grounding map deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-RLM-001 | **ADD** full RG-1..RG-6 measured calculus (exp-1/2/ceiling), RS-1..RS-7 substrate anchors, RC-1..RC-3 corrections, SSB prefer-reuse for J-107/J-108. | 10-J:L1485 path inventory; L1149 free-Python reject; L1559 ambiguity | RLM-E001–RLM-E025 |
| SD-RLM-002 | **ADD** G6 Level-2/3 anti-goal and constrained within-step-only steal; HippoRAG primary vs NavOp fallback caps. | 10-J:L1149 compressed | RLM-E004, RLM-E005, RLM-E019, RLM-E020 |
| SD-RLM-003 | **ADD** J-86 WIDTH / J-108 DEPTH sibling relationship via SearchLadder idiom. | 10-J:L1274 GRC spine; L1559 | RLM-E013; J86-I001 |
| SD-RLM-004 | **QUALIFY** summary ambiguity that 12 interface maps are unresolved: RLM map names RLM 2512.24601, LongCoT 2604.14140, HippoRAG 2405.14831 as load-bearing HIGH; blogs MED-LOW confirmation only. | 10-J:L1559 | RLM-E021 |

### J-21 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J21-001 | **ADD** ban on seven unverifiable platform names; sanitize-before-abstraction; selective revalidation cost control; branch-scoped revalidation; raw-evidence bound/compaction risk. | 10-J:L245-L253 | J21-E005, J21-E007, J21-E022–J21-E024 |
| SD-J21-002 | **ADD** exact MemoryValidationEvent kinds, prefer-reuse FactValidationFailed option, production reachability, STALE/UNVERIFIED/RETAINED negatives. | 10-J:L252-L253 | J21-E014, J21-E016, J21-E018–J21-E019 |
| SD-J21-003 | **ADD** arXiv set (A-MEM/GEM/MPBench/MINJA/H-31) and composition list including G-1 taint on raw tier. | partial | J21-E020–J21-E021 |
| SD-J21-004 | **FLAG** SSB stamp 231/542 vs later baselines; event/+1 ambiguity. | none | J21-C001, J21-A001–J21-A002 |

### J-24 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J24-001 | **ADD** exact BehaviorSteeringEvent kinds, prefer-reuse McpToolDeferred option, `--no-steering` boot state, remote-GPU W-10 gate on local steering proof. | 10-J:L275-L283 | J24-E002, J24-E011, J24-E013 |
| SD-J24-002 | **ADD** GD-4 ≥3-run bound before default-ON steering/tool-defer; vector provenance ID-6; coherence risk; honest tools floor statement. | 10-J:L279, L1283 | J24-E003, J24-E014, J24-E019–J24-E020 |
| SD-J24-003 | **ADD** production dual reachability (local steering + OAuth Tool Search), negative arms, BACM arXiv IDs, compose with I-32. | partial | J24-E013, J24-E015–J24-E018 |
| SD-J24-004 | **FLAG** alpha magnitude and surface tile decisions absent from summary. | none | J24-A001–J24-A003 |

### J-25 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J25-001 | **ADD** exact per-section token budgets, 0.95 safety factor, usable-window formula, 25% recent-turn floor, 0.15×topScore + cap 50, fingerprint format. | 10-J:L285-L294 | J25-E003, J25-E005, J25-E007–J25-E008, J25-E011 |
| SD-J25-002 | **ADD** Tier-1 vs Tier-2 revised scope (T3/T4/T6/T2 now; T1/T5/T7 deferred), FORK-1/2, ABORT-1..4, MOVE/VERIFY/REDTEAM/INTEGRATION_SMOKE. | compressed | J25-E014, J25-E025–J25-E031 |
| SD-J25-003 | **ADD** module paths (context_boundary.rs, adjust_boundary_for_invariants), prefer-reuse HandoffLifecycleEvent(227), optional ContextMechanicsEvent kinds. | partial | J25-E015, J25-E017, J25-E023 |
| SD-J25-004 | **QUALIFY** summary "thinking siblings": Tier-1 default is tool-pair only; thinking-sibling is FORK-1 ROUTE B if ContentBlock gains Thinking. | 10-J:L289 | J25-E006, J25-E025, J25-A002 |
| SD-J25-005 | **ADD** riders A5/A10/A11/A12 ownership and no-new-dashboard justification (H-51/H-9). | none | J25-E012, J25-E016 |

### J-61 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J61-001 | **ADD** I-6 mis-citation correction (context-pane not concurrency); SSB total order as conflict model; kill-switch ELAI_CANVAS_REDIRECT_DISABLE default-ON. | 10-J:L664-L671 | J61-E004, J61-E019 |
| SD-J61-002 | **ADD** Tier-1 vs Tier-2 agent-control split, exact dispatcher namespace/ops, canvas_redirect.rs LOC, contract arms a–f, file touch list. | compressed | J61-E009–J61-E013, J61-E021–J61-E022 |
| SD-J61-003 | **ADD** DETERMINISTIC_CHECK greps forbidding direct graph mutation and LLM verdicts; W-done-gate GD-2 ELAI_SEED re-derive. | partial | J61-E018, J61-E017 |
| SD-J61-004 | **ADD** open REORDER decoupled-vs-broken mapping and mid-token preempt grain decision. | none | J61-A001–J61-A002 |
| SD-J61-005 | **FLAG** baseline stamp 241/550 needs re-grep. | none | J61-C001 |

### J-62 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J62-001 | **ADD** dashboardRules.md absence + React mismatch evidence; env-card contents (runes, IPC, inventory, pointers); REGISTRY GD-4 pain-gated expansion. | 10-J:L673-L680, L1561 | J62-E002–J62-E004 |
| SD-J62-002 | **ADD** OBSERVE-ONLY first, ELAI_ENV_CARD_GATE_DISABLE log-not-silent, EnvCardGated kinds, HookEnforcementPanel fold, LEG 2 fallback to LEG 1. | compressed | J62-E009–J62-E010, J62-E015–J62-E016, J62-E007 |
| SD-J62-003 | **ADD** Fable illustrative provenance + Anthropic Skills corroboration; skills-vs-rulebook open choice; contract arms a–e. | partial | J62-E023, J62-A001, J62-E022 |
| SD-J62-004 | **ADD** riders (python/Rust cards, mandatory re-dispatch, learned score, auto-regen) and J-52 sequencing. | none | J62-E024, J62-E006 |

### J-73 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J73-001 | **ADD** exact cascade stage list (bm25→vector→pagerank→catrag→arag), QueryTrace prefer-reuse option, LIVE ~269/578 stamp, IC-13 one-calibration-impl ban. | 10-J:L779-L787 | J73-E008, J73-E011, J73-E014 |
| SD-J73-002 | **ADD** fail-closed below-T always retrieves; OBSERVE-ONLY would-skip journal; calibration-distribution risk vs answer-abstention. | 10-J:L785-L786 | J73-E016, J73-E009, J73-E018 |
| SD-J73-003 | **ADD** compose with J-72 and D-16; special-token Tier-2 pod_stop after. | partial | J73-E017, J73-E009 |

### J-86 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J86-001 | **ADD** SL-1..SL-5 formal ladder labels, IMPLICATION numbers, ADD-6 ~400-sample cap, CGES design-rationale, D-10 receipt router domain list. | 10-J:L906-L914 | J86-E003–J86-E008 |
| SD-J86-002 | **ADD** exact LOC task split, contract arms a–i, kill-switch/allowlist W-2/R9, PRM W-10 independence rule. | compressed | J86-E010, J86-E019–J86-E020 |
| SD-J86-003 | **ADD** sequence after J-84/J-85, depends J-83, parallel-safe none in-wave; structural vote-for-recall foreclosure open question. | 10-J:L1274 | J86-E001, J86-A001 |
| SD-J86-004 | **ADD** net-0 event reuse policy and optional Kind decision. | 10-J:L914 | J86-E016, J86-A003 |

### J-101 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J101-001 | **ADD** three residual gap bullets (paraphrase/cross-process/non-registry MCP), GD-5 honest-scope comment ~234, Provenanced<T> not end-to-end. | 10-J:L990-L999 | J101-E003–J101-E004 |
| SD-J101-002 | **ADD** WI-1a/b/c split, OBSERVE-ONLY window, ten-benign FP hard gate, exact production line anchors :224/:334/:472. | compressed | J101-E005–J101-E009, J101-E012 |
| SD-J101-003 | **ADD** mandatory read list, contract suite (extend g1 + two new), file touch list, K-13 WI-4/WI-5 ownership, J-102 pairing. | partial | J101-E013–J101-E014, J101-E022–J101-E025 |
| SD-J101-004 | **ADD** RTBAS false-block risk, process_global_slot may already cover, ASI06 open confirm. | none | J101-E026, J101-A001, J101-A003 |
| SD-J101-005 | **FLAG** no new UI default; safety/SSB timeline only unless W-4 decides tile. | 10-J:L999 compressed | J101-E017, J101-A002 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **10/10 files**, **1,228/1,228 logical lines**, **149,610/149,610 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **273 unique atoms** = **239 explicit**, **002 inferred**, **023 ambiguous**, **009 conflicting/corrective**.
- Per-file atom counts: NASA **027**; RLM **027**; J-21 **028**; J-24 **024**; J-25 **034**; J-61 **027**; J-62 **028**; J-73 **022**; J-86 **026**; J-101 **030**. Sum **273**.
- Relational interaction records: **24** (`IX-001`–`IX-024`). Summary-delta records: **041** (`SD-*`). Ambiguity/conflict register: **11** (`AC-001`–`AC-011`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `NASA/RLM/J21/J24/J25/J61/J62/J73/J86/J101-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them. User-facing surfaces captured as own atoms: memory-hub / MemoryGovernanceEvent panel (J-21); `/v1/steering/*` endpoints + runtime/context steering-effect/token-budget panel (J-24); `elai run --local` steering proof command; existing H-51/H-9 pressure/digest panels (J-25); canvas drag-connect/cut/reorder/rebind/tab agent-control gestures + CanvasRedirectIntent Tauri command + J-31 broken-edge render (J-61); `ELAI_CANVAS_REDIRECT_DISABLE`; dashboard-v2-env skill package + HookEnforcementPanel injection counts + `ELAI_ENV_CARD_GATE_DISABLE` (J-62); KnowledgeDrawer retrieval-skipped rate (J-73); search-ladder / ConvergencePanel candidate branches/receipt/early-stop (J-86); safety/SSB taint-block/quarantine timeline (J-101); AgentDojo CI artifact (not a dashboard tile).
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the ten exact DEEP-J-005 manifest rows. The only orientation artifact consulted was `10-J-SUMMARY_INDEX.discovery.md`. Format reference consulted: `DEEP-I-004.discovery.md` and neighboring `DEEP-J-004.discovery.md` for shape only.
- Output metrics after finalization: `output_lines=0547`; `output_bytes=093498`.
- Canonical normalized output SHA-256: 789bfbc4c99f919366486b67fec420f2da19ee8f830c0f7af3075fc2e05939a4
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
