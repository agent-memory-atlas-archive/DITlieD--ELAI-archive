F-plan-next master recommendations

Scope: four shard-prior-art topics evaluated against live .elai_cc source. Each verdict is grounded in code anchors (file:line) and primary sources (arXiv IDs WebSearch-verified to resolve, registry checks for any library). UNGROUNDED items are labeled inline. No external claim stands on training memory.

HONESTY NOTE (correcting an earlier main-session overclaim)
The main session earlier stated ELAI "has no DGM". That is wrong. ELAI ships the full DGM substrate, inert. What ELAI actually has, verified against source:
- DGM parent-selection formulas implemented faithfully (sigmoid lambda=10, alpha_0=0.5, w_i=s_i*h_i) at config_archive.rs:34-83, with insert_variant/select_parent/sample_variant (config_archive.rs:198,274,395) and deceptive-dip ancestor retention.
- Successive-Rejects bandit math (darwinian_bai.rs SuccessiveRejects::log_bar).
- The full guard trio, tests green: Two-Gate promotion (promotion_gate.rs:109, DEFAULT_TAU=0.05 line 42, K_m=50 line 34, cites arXiv:2510.04399), cheat-pattern traps (trap_tasks.rs:56-63 MarkerRemoval/FakeTestLog/VerifierCapture), blake3 tripwires (tripwire.rs:34,254), and the Layer 3 human-approval gate (self_mod_safety.rs:248).
- A CLI surface (darwin_loop_cmd.rs).
What is MISSING is exactly one piece: the driver that closes the loop. darwin_loop_cmd drives only a synthetic alternating reward (line 104) and terminates RUNPOD-PENDING (line 127) without ever touching config_archive. config_archive's three core functions have exactly 3 repo-wide occurrences (module + contract test + lib.rs re-export) = zero production callers. So the correct statement is "ELAI has the entire DGM machinery built and guarded but never wired into a closed loop", NOT "ELAI has no DGM".

----------------------------------------------------------------------
TOPIC 1 - DGM / open-ended self-improvement archive-search loop
----------------------------------------------------------------------
RECOMMENDATION: INTEGRATE_NEW_F_PLAN   CONFIDENCE: high   DRAFT AUTHORED: yes (F-NEXT-dgm.md)

ALREADY IN ELAI (with anchors)
- DGM formulas + archive ops: config_archive.rs:34-83 (sigmoid lambda=10, alpha_0=0.5, w_i=s_i*h_i), :198 insert_variant, :274 select_parent, :395 sample_variant; deceptive-dip ancestor retention present.
- Bandit math: darwinian_bai.rs SuccessiveRejects::log_bar.
- Guard trio (tests green): promotion_gate.rs:42 DEFAULT_TAU=0.05, :34 K_m=50, :109 two_gate_promote (cites arXiv:2510.04399); trap_tasks.rs:56-63 (MarkerRemoval/FakeTestLog/VerifierCapture); tripwire.rs:34,254 (blake3); self_mod_safety.rs:248 (human-approval Layer 3).
- CLI shell: darwin_loop_cmd.rs (synthetic reward line 104, RUNPOD-PENDING terminus line 127, no config_archive ref).

THE GAP (verified, not overstated)
The closed loop propose->evaluate->insert_variant->select_parent->mutate is missing exactly ONE piece - the driver - and everything it would call already exists and is tested. Callerless proof: grep select_parent|insert_variant|sample_variant returns exactly 3 files (config_archive.rs + config_archive_contract.rs + lib.rs re-export). The 14-file "config_archive" grep is inflated by an UNRELATED SQLite table of the same name in self_mod_layer1a.rs (module docstring lines 10-12 disambiguates), so the callerless claim survives that confound.

ONE-LINE RATIONALE
Numbers and gap both verify (primary-source and against live code); a tightly-scoped F-plan part to wire ONE config-only generation through the existing guards is the right move - but the input report under-specifies one real safety wiring gap (the Layer 1A security-keyword filter is on a separate path from config_archive and will NOT auto-fire on the admit loop).

SMALLEST INCREMENT
One driver function in darwin_loop_cmd.rs plus a thin orchestrator wiring call:
(1) replace synthetic reward (line 104) with ONE real benchmark eval from external_benches;
(2) feed real alpha into config_archive::insert_variant, then call select_parent to seed next;
(3) route every admit IN SERIES through tripwire (pre-exec) -> trap_tasks scoring on the LIVE candidate -> AND ALSO self_mod_layer1a::is_security_sensitive (the report omitted this) -> promotion_gate::two_gate_promote;
(4) keep mutant = config/template variant ONLY, NOT agent-source self-rewrite (respects the 2510.04399 bounded-capacity requirement; avoids the Layer 3 human-approval gate by construction);
(5) wire-dark behind ELAI_DARWINIAN_LOOP_ENABLE=1 + cfg(feature="darwinian-evolve"), emit SSB.
Acceptance = "loop closed + a guard caught a trap-task mutant", NOT "score went up".

KILLER RISKS
1. SECURITY-FILTER BYPASS (decisive): config_archive::insert_variant validates score-range ONLY (line 199), with NO is_security_sensitive check and NO AutonomousGate routing. That keyword filter lives in the separate self_mod_layer1a.rs module, so wiring the loop creates a NEW autonomous mutation path that bypasses auth/credential/token rejection unless the driver explicitly calls it. The report's "route through existing guards in series" silently omits this path.
2. TAU/K_M NOT CALIBRATED: promotion_gate.rs self-labels tau and K_m as "UNCONFIRMED for ELAI" (lines 34,39) - carried-forward defaults, not held-out-calibrated. The report's "guard theory correctly grounded" overstates calibration; a miscalibrated tau admits regressions or starves the archive.
3. TRAP-SCORING MUST BIND THE LIVE CANDIDATE: containment is structurally PRE-promotion (stronger than DGM's post-hoc lineage) BUT only if trap_tasks is scored on the live candidate. darwin_loop_cmd currently scores traps on synthetic fixtures decoupled from the candidate; the driver must thread the real candidate or cheat-detection is theatre.
4. UNGROUNDED that one closed generation shows a positive delta - DGM needed many iterations and hit deceptive dips. v1 acceptance must be loop-closure + guard-fire, not capability gain.
5. REPORT NUMERIC ERROR: claims "17 async contract tests" for config_archive; actual is 14 (9 contract + 5 inline, not all async). Does not change the gap, signals rounding-up.

PRIMARY SOURCES (verified to resolve)
arXiv:2505.22954 DGM (Zhang/Hu/Lu/Lange/Clune, 20.0->50.0 / 14.2->30.7); sakana.ai/dgm (4 reward-hack quotes verbatim); arXiv:2510.04399 Two-Gate tau+K_m PAC-iff-bounded (Wang/Dorchen/Jin); arXiv:2504.15228 SICA 17->53 (Robeyns/Szummer/Aitchison); arXiv:2603.19461 HyperAgents/DGM-H (claimed DGM successor).

----------------------------------------------------------------------
TOPIC 2 - git-backed whole-state recovery (Springdrift Backup Actor) vs ELAI SSB-replay
----------------------------------------------------------------------
RECOMMENDATION: EXTEND_EXISTING   CONFIDENCE: high   DRAFT AUTHORED: yes (F-NEXT-springdrift-recovery.md, recommendation kept as EXTEND, not a numbered F-N)

ALREADY IN ELAI (with anchors)
Three INDEPENDENT, never-co-committed stores: (1) SSB SQLite event journal (typed catalog, blake3 integrity, ELAI_SEED re-run determinism + DriftComparator at ssb.rs:16070,:403); (2) session-DAG SQLite branching/merge snapshots (session_dag.rs:205-242); (3) agent checkpoints = metadata-in-SSB + 5 msgpack files on disk (checkpoint.rs:198-278), with a lazy migrator (checkpoint.rs:70-77,91-97) so checkpoints survive schema bumps. Recovery is replay/re-derivation, not byte-restore. PreCompact hook auto-writes SESSION_HANDOFF. Working-file durability is delegated to the PROJECT's external git (session_dag.rs:146 commit_hashes:Vec<String>, TEXT at :221 - pointers, not content).

THE GAP (four confirmed vs Springdrift's Backup Actor at 07-springdrift.md:49,:110)
(1) byte-exact state is not in the replayable journal - SSB stores hashes+sizes only (checkpoint.rs:207,248-258,263-269); lose the msgpack dir and the journal points at nothing.
(2) SSB schema bumps are DESTRUCTIVE - every version DROPs prior-version session DBs with no migration (ssb.rs ~23 occurrences), destroying the audit journal + session-DAG (lazy migrator exists ONLY on the checkpoint path).
(3) NO Backup-Actor analog and NO unified store - the only periodic ticks (task_runner.rs:13179-13280) are a refusal-SLA monitor and a skill-staleness sweep; the only other "snapshot" is dst.rs:33 StateSnapshot (in-memory DST struct, no bytes/disk/git).
(4) working file tree is a pointer into external git, not versioned content.

ONE-LINE RATIONALE
Real but narrow capability gap; not worth a full F-plan part for ELAI's BOUNDED multi-agent sessions over an already-git-backed project. The right move is a wire-dark archive-on-schema-bump guard that closes the most damaging gap (silent destruction of audit history) without importing the Backup-Actor mechanism.

SMALLEST INCREMENT
Opt-in, wire-dark "session archive on schema-bump": before SSB DROPs a stale-version session DB on first read (the destructive path at ssb.rs migration sites), copy old DB + the session-DAG DB to workflow/sessions/{id}/archive/ssb-v{N}.sqlite, gated behind an ELAI_*_ENABLE flag. One file-copy + one-line guard at the existing drop site. Converts "destroyed forever" into "cold-archived, manually inspectable". Adds NO periodic commits, NO Backup Actor, NO unified store, NO git dependency on agent state. Fallback: document-the-gap-build-nothing (one paragraph in the SSB schema-version header stating DROP is intentional, journals forensic-within-session-only).

KILLER RISKS
1. OVER-ENGINEERING: the full Backup-Actor pattern (periodic git-commit of SSB+session-DAG+msgpack) is heavy mechanism for a failure mode ELAI rarely hits - Springdrift targets MONTHS-long single autonomous agents; ELAI runs bounded multi-agent sessions over source ALREADY in real git.
2. EVIDENCE TOO WEAK FOR A HEAVY BUILD: every Springdrift number is vendor-reported, single-author (Seamus Brady), n=1, unreproduced, 45 stars; eval scaffolding on HuggingFace not independently reproduced (07-springdrift.md:174).
3. LICENSE: Springdrift is AGPL-3.0 (07-springdrift.md:18,:24) - concepts reusable, CODE not without copyleft exposure on a network service.
4. CARRIED UNKNOWN: NO agent checkpoint *.msgpack files exist on disk in the repo, so the checkpoint path is wired-but-unexercised by any persisted live run - gap #1 (hashes-not-bytes) is verified from code, not from a captured live failure.
5. The archive copy must be ADDITIVE - the DROP must still proceed; do not silently change DROP semantics for callers expecting a clean DB.

PRIMARY SOURCES
07-springdrift.md:49,:110 (Backup Actor = git commits + remote push, default 5-min, git checkout whole-state), :15 (arXiv:2604.04660 verified-resolving), :18,:24 (AGPL-3.0), :174 (eval not independently reproduced). ELAI determinism mitigation backed by ssb.rs:16070,:16079 (ELAI_SEED first_divergence_index), :403 (DriftComparator).

----------------------------------------------------------------------
TOPIC 3 - Graphiti/Zep LLM-judged contradiction invalidation vs ELAI deterministic audit_conflicts
----------------------------------------------------------------------
RECOMMENDATION: EXTEND_EXISTING   CONFIDENCE: high   DRAFT AUTHORED: yes (F-NEXT-graphiti-contradiction.md)

ALREADY IN ELAI (with anchors)
- Deterministic non-LLM contradiction path: audit_conflicts (contradiction.rs:64-143) groups currently-valid rows by (project_id, scope, key) (line 75-80), flags conflict_kind="contradiction" only when older.value != newer.value for the SAME byte-identical key (line 97-102); resolution is trust-tier priority T0=3..T3=0 (line 152-160), tier-supersede only when auto_supersede_by_tier && newer tier strictly > older (line 104-112), else Surface to user (line 114). Module header (18-23) defers LLM-backed semantic detection to E-5-followup.
- mem0_dispatcher.rs:144-256 exact-key match (line 244) + byte-identical JSON compare (line 189).
- Non-lossy substrate ALREADY exists: supersede (store.rs:204-265) closes old row (valid_to_ms+superseded_by) and INSERTs replacement in one transaction, docstring cites "Zep 2.2.3" (line 207). ELAI already mirrors Graphiti's transaction-axis non-lossiness; only the TRIGGER differs, not storage.
- F-5 quarantine (quarantine_broadcast.rs) is pure age-based (staleness_ratio=age_days/half_life_days, threshold 1.5, line 53/101-102); docstring (111-114) says it does NOT run the contradiction adjudicator - orthogonal, composes.
- Cloud routing exists: cloud_mode.rs route (284) / route_with_spend (309), spend caps caller-driven (None=skipped, 305-308).

THE GAP
ELAI cannot detect a contradiction between two facts semantically opposed but NOT sharing a byte-identical key string + identical JSON shape. Its adjudicator is a string-equality grouper with a tier tiebreak. Graphiti's resolve_extracted_edge/resolve_edge_contradictions is a semantic judge that, on contradiction, sets the loser's event-time invalid_at = the invalidating edge's valid_at and returns it non-destructively in invalidated_edges (edge_operations.py @ v0.29.1 L656/629/667/897, WebFetch verbatim). So ELAI misses paraphrase/implication contradictions across non-matching keys.

ONE-LINE RATIONALE
The recall gap is real and worth closing, but a CLOUD LLM judge is the WRONG instrument: it structurally breaks ELAI_SEED reproducibility (arXiv:2506.09501 - temp=0 + seed does NOT yield deterministic cloud output across batch/GPU) for no benefit a LOCAL deterministic-argmax NLI cross-encoder cannot also deliver at zero cloud spend. Extend the existing deterministic path with a local NLI pre-filter; do NOT route the judge through the cloud lane.

SMALLEST INCREMENT
Trigger-only change in crates/memory, storage path unchanged. When audit_conflicts returns Surface{conflict_kind:"contradiction"} (contradiction.rs:114), add a second pass that re-pairs candidates across DIFFERENT keys within the same (project_id, scope), pre-filtered by the existing key/entity grouping so the model only ever sees ~2 rows (Graphiti's localize-first cost control). Feed each (older.value, newer.value) pair to a LOCAL NLI cross-encoder on the sidecar (sentence-transformers CrossEncoder NLI family -> {entailment,neutral,contradiction} via argmax). On contradiction above a confidence threshold (>=0.85): set loser's invalidation to invalidating fact's valid_from_ms (mirroring invalid_at=valid_at) and call existing non-destructive supersede/soft_close (store.rs:204). On neutral/low-confidence: keep current Surface-to-user default. Byte-identical and tier-dominant cases still resolve locally with zero model cost. MUST run anti-slopsquatting registry verification before any import line. Reuses MemoryConflictDetected with conflict_kind="semantic_contradiction" - no SCHEMA bump.

KILLER RISKS
1. DETERMINISM REGRESSION IF ROUTED THROUGH CLOUD (Option A, decisive): arXiv:2506.09501 (NeurIPS 2025 oral) confirms temp=0 + fixed seed does NOT yield deterministic cloud output - batch-variant GPU kernel reduction order breaks FP associativity; an auto-close could flip between identical ELAI_SEED replays, directly contradicting ELAI's deterministic-trust-spine value prop. Decisive reason to avoid the cloud lane.
2. FALSE-POSITIVE / OVER-EAGER AUTO-CLOSE: an NLI judge auto-closing a fact the deterministic path would have Surfaced is the real precision hazard. MUST be confidence-gated and MUST keep Surface-to-user as fallback default.
3. LOCAL NLI IS "PRACTICALLY DETERMINISTIC / ROBUSTLY STABLE", NOT FORMALLY SEED-INVARIANT: argmax over a wide 3-class logit gap is directionally deterministic, but the same GPU FP batch-variance class (2506.09501) applies to its logits. State it as robustly stable for replay, not fully deterministic.
4. OPTION B (local NLI) IS UNBUILT: no NLI model is wired into ELAI's sidecar today; net-new wiring + a model dependency, not a config flip. Pin/verify the model via registry before asserting it exists. Note also: audit_conflicts itself is currently wire-dark with zero production callers (W-7 caveat).
5. SCOPE-CREEP TRAP: the DKG cascade exact-text join (knowledge/src/dkg/mod.rs:751, concept==fact_id) is a SEPARATE exact-equality limitation needing embedding/alias resolution - do NOT bundle. F-5 age-based quarantine stays as-is (Graphiti has no time-decay; that remains ELAI's edge).

PRIMARY SOURCES
arXiv:2501.13956 = "Zep: A Temporal Knowledge Graph Architecture for Agent Memory" (Rasmussen et al., 2025-01-20) WebSearch-verified; Graphiti edge_operations.py @ v0.29.1 WebFetch-verified verbatim (resolve_extracted_edge L656, resolve_edge_contradictions L629, invalid_at=valid_at L667, invalidated_edges returned not deleted L897); arXiv:2506.09501 = "Understanding and Mitigating Numerical Sources of Nondeterminism in LLM Inference" (NeurIPS 2025 oral) WebSearch-verified; local NLI pattern verified current (cross-encoder/nli-deberta-v3-base 90.04% MNLI-mm, nli-distilroberta-base) - package NOT pinned, anti-slopsquatting deferred to executor.

----------------------------------------------------------------------
TOPIC 4 - F-2 multi-language security ruleset: regex->AST tiering + multi-lang vuln-rule corpus
----------------------------------------------------------------------
RECOMMENDATION: EXTEND_EXISTING   CONFIDENCE: high   DRAFT AUTHORED: yes (F-NEXT-f2-security-rules / F-2c)

ALREADY IN ELAI (with anchors)
- Real operational AST gate: TreeSitterAstEvaluator is a genuine tree-sitter impl (ast_adapter.rs:53-102: set_language + Query::new + cursor.matches), NOT a stub. NoOpAstEvaluator is test-only (rule_engine.rs:912, inside #[cfg(test)]).
- Locked eval order MetricGate(0)->GrepPattern(1)->AstPattern(2)->Composite(3)->GraphReachability(4) is real (rule_engine.rs:328-339).
- 12 hand-regex security rules confirmed exactly (missing_authz=2 CWE-862, sqli=3 CWE-89, weak_crypto=4 CWE-327, xss=3 CWE-79; Python+TS only).
- web.toml uses AST/Composite Tier-2 predicates today - proves structural rules work with zero new deps.
- SeCodePLT bench HARNESS already wired (bench_seccodeplt.rs + SecCodePltRunner + vibegate::evaluate_vibegate + InsecureRateMeasured SSB).
- language_agnostic fragment-vs-full-file split is real (rule_engine.rs:80-85,547).

THE GAP (two real, report overstates reach)
(1) TreeSitterAstEvaluator::new() (ast_adapter.rs:18-44) registers only 10 languages (rust/python/typescript/go/java/c_sharp/ruby/html/css/svelte) - NOT the "16 grammars" the report claims. JavaScript/C/C++/Swift/Kotlin/Scala/PHP/Bash are pinned in Cargo.toml but UNREGISTERED in the evaluator, so Tier-2 rules for them require editing ast_adapter.rs::new(), not a "one-line registry insert".
(2) detect_language_from_ext (rule_engine.rs:346-362) returns None for .svelte/.html/.css - gap IS real as claimed.
(3) 4 highest-ROI CWEs (798 hardcoded-creds, 321 hardcoded-key, 532 secrets-in-logs, 95 eval-injection) genuinely uncovered.

ONE-LINE RATIONALE
Directionally sound and license-clean - adopt Approach A (existing TreeSitterAstEvaluator, zero new dep) for new structural rules - but the report materially overstates multi-language readiness: the AST evaluator registers 10 languages, not 16, so C/C++/JS/Swift Tier-2 rules need an ast_adapter.rs::new() edit, not a free registry insert.

SMALLEST INCREMENT
Add the 4 uncovered low-FP CWE rules (798/321/532/95) for the already-registered Python+TS via the existing GrepPattern->AstPattern Composite (web.toml is the working template); zero new deps, zero ast_adapter changes. Then RUN the shipped SeCodePLT bench harness for the first published insecure-rate baseline over the now-16-rule corpus BEFORE any "reduces FP"/"adds languages" claim. Defer all multi-language expansion (C/C++/JS/Swift) and the ast-grep-core dependency until after that baseline.

KILLER RISKS
1. GRAMMAR-COUNT OVERSTATEMENT: report says "16 grammars wired" / "one-line-per-grammar insert" for C/C++/JS/Swift, but new() (ast_adapter.rs:18-44) only registers 10. Acting on this would scope multi-lang as trivial when it needs evaluator edits + grammar-ABI verification per language.
2. SEMVER PATCH-FLOOR: ast-grep-core 0.43 needs tree-sitter ^0.26.3; ELAI pins "0.26". Cargo.lock resolves tree-sitter to 0.26.8, which DOES meet the floor - but the report asserted compatibility WITHOUT the lockfile check; require the Cargo.lock check before any ast-grep-core add.
3. UNMEASURED EFFICACY: no SeCodePLT run has produced FP/recall numbers, so "would reduce FP AND add languages" is unproven (report's own ungrounded caveat is correct). The harness exists; a baseline run is the cheapest next step.
4. COMPLEXITY-VS-VALUE: ast-grep-core (Approach B) adds a new MIT dep whose only win over the existing engine is a richer YAML DSL; buys nothing for the first rule batch. A reader skimming a "TOP RECOMMENDATION: ast-grep" header could add a dep for no gain.
5. FALSE-POSITIVE UX on provenance/absence classes (798/321/532) over-firing on placeholders/non-secret hashes (the CodeGate failure mode); mitigate with Grep+AST Composite + file_context=production + WARN-not-BLOCK for the noisy print( shape; gate any efficacy claim on the baseline run.

PRIMARY SOURCES
arXiv:2410.11096 = SeCodePLT (Nie et al, 5.9k samples, 44 CWE, Py/C/C++/Java) VERIFIED; 2511.04023 = AdaTaint (-43.7% FP) VERIFIED; 2601.18844 = Tencent FP-reduction study VERIFIED; Semgrep Rules License v1.0 (Dec-2024, non-SaaS/non-competing) VERIFIED at semgrep.dev/legal/rules-license; ast-grep-core 0.43.0 (MIT, last published 2026-05-25, crates.io) verified clean for the record but DEFERRED/unused; tree-sitter resolves to 0.26.8 in Cargo.lock (meets ^0.26.3 floor).

======================================================================
PRIORITIZED DO-NEXT LIST (highest leverage first)
======================================================================
1. F-2c (Topic 4): author the 4 low-FP CWE rules (798/321/532/95) on registered Py+TS via the existing Composite engine. Zero new deps, zero ast_adapter edit, working web.toml template. Lowest risk, immediate security coverage. Then RUN the shipped SeCodePLT harness for the first insecure-rate baseline - the cheapest measurement in the whole set and a precondition for every other efficacy claim.
2. F-DGM-1 (Topic 1): wire ONE config-only DGM generation through the four-guard chain in series, mutant = config/template only, source-rewrite deferred. Highest strategic leverage (turns a fully-built-but-inert substrate into a closed loop) but MUST close the security-filter-bypass gap first (driver explicitly calls is_security_sensitive) and thread trap-scoring on the live candidate. Acceptance = loop-closed + guard-caught-a-trap, not score-up. Multi-generation live run is RUNPOD-PENDING Tier-2.
3. Graphiti contradiction pre-filter (Topic 3): trigger-only local-NLI second pass in crates/memory; storage path unchanged, confidence-gated, Surface-to-user default preserved. Real recall gain - but Option B is UNBUILT (net-new model wiring) and audit_conflicts is itself wire-dark, so it ranks below the two above. NEVER route through the cloud lane (breaks ELAI_SEED determinism per 2506.09501). Anti-slopsquat the NLI package at import time.
4. Springdrift archive-on-schema-bump (Topic 2): smallest of all - one file-copy + one-line guard at the destructive SSB DROP site, wire-dark. Closes the most damaging gap (silent audit-history destruction) without the heavy Backup-Actor mechanism. Lowest strategic value (narrow gap, project source already in git, AGPL code not reusable); fallback is document-the-gap-build-nothing. Do last or fold into a future hygiene F-part.

Cross-cutting note: every "reduces FP / improves recall / score went up" claim across all four topics is UNMEASURED today. Do not publish any efficacy number before the relevant bench runs (SeCodePLT for Topic 4; a closed-generation guard-fire demo for Topic 1; a measured-precision follow-up for Topic 3). Loop-closure and guard-fire are acceptable v1 acceptance criteria; capability-gain is not.
