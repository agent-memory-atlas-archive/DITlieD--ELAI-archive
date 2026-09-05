# F-plan — Summary Index of All 10 Sessions
Generated: 2026-06-01. Mirrors E-SUMMARY_INDEX format. Each entry preserves the cluster mapping,
classification, the adversarial verdicts (novelty/grounding/feasibility) that survived run
wf_b959963a-db6, the REVISED scope, the production integration point, and the observable_effect that
is the wiring done-gate (W-1). Canonical why + per-item detail: workflow/reasoning/
VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md. Enforcement doctrine: F-0_INDEX.md (W-1..W-10).

## Premise
Every F-N is one implementation session that turns one VibeSec/SHARD proposal into wired,
dashboard-visible reality. The differentiator vs B/C/D/E is the ENFORCED done-gate: a session is not
DONE until wiring_contract_check.py --run-trigger PASSes (produces exist + consumes real + reachable
from a production entrypoint + stub_ban clean + observable_effect actually fired). cargo-green-alone
is rejected. Default-off env gates as a terminal state are banned (W-2).

## Per-session index

### F-1 — VibeGate: unified fail-closed multi-axis in-process write-gate
- Cluster C1 | HYBRID | novelty GENUINELY_NEW (vs E-30 argv-only, E-37 prompt-injection, E-43 skill
  soft-gate) | feasibility HARD | effort L
- Steal/build: one PreToolUse handler fans a file-write across four axes (code-vuln, hallucinated-dep,
  secret, malicious-skill) and BLOCKs fail-closed if any fires, backed by a "vibesec" rule domain
  seeded mutable=0 so per-project edits cannot disable security. No competitor unifies the four;
  CodeGate (the one that tried) is dead.
- Existing ground truth (code-read): rules engine ALWAYS_INCLUDED_DOMAINS already includes "security"
  (rule_engine.rs:187); the mutable column exists (schema.rs:46, default 1) with a tested mutable=0
  path (lifecycle.rs:871); PreToolUse runner is run_pre_tool_use (crates/plugins/src/hooks.rs /
  orchestrator hook_dispatcher.rs). The four axes are NOT wired into one compound check.
- REVISED scope (W-10 tiered): Tier-1 = code-vuln (AST/GREP) + secret (GREP on diff), pure in-process,
  genuinely fail-closed, ships now with the four per-tool contract tests (Edit/Write/NotebookEdit/Bash
  per #37210) + a blake3 startup integrity check of the mutable=0 seed rows. Tier-2 = dep-existence +
  mcp-scan (network/subprocess) deferred behind a design-gate (cache at plan-start; per-session not
  per-write).
- Integration: hooks.rs run_pre_tool_use -> compound VibeGate evaluator -> RuleEngine; seed mutable=0
  security rows in schema.rs. frontend_pair: VibeGate panel in a dashboard drawer.
- observable_effect (done-gate): an Edit/Write carrying a known SQLi pattern OR a hardcoded secret is
  refused before disk; SSB vibesec_code_vuln_blocked / vibesec_secret_blocked row + the write does not
  land + the panel shows the block. Measurable: 0 secrets and 0 known vuln-class AST matches reach disk
  on a defined synthetic corpus, with a measured-and-bounded false-positive rate on >=3 real orchestrate
  runs (CodeGate died of FP friction - bound it before release).

### F-2 — Code-vuln-class ruleset + the FIRST measured pre-write insecure-rate number (the keystone)
- Cluster C2 | HYBRID | novelty GENUINELY_NEW (vs E-12 post-write semgrep, C-26/E-35 decode substrate,
  E-9/E-49 functional-only benchmarks) | feasibility HARD | grounding ACCURATE
- Steal/build: populate the always-on security rule domain with vuln-class patterns (SQLi/XSS/
  weak-crypto/missing-authz) that BLOCK at write; produce the first published before/after insecure-rate.
  SVEN 59.1->92.3% (prefix steering) and CodeGuard+ 43.2->76.0% (constrained decode) are VERIFIED but on
  CodeGen-2.7B - they do NOT transfer to Qwen2.5-Coder-7B-abliterated; the number must be measured.
- REVISED scope (split): F-2a (effort M) - ship the ruleset + write-time BLOCK, scoped to Python/TS
  first to bound false-positives, NO performance claim yet. F-2b (gated on C-26/E-35 + RunPod) - run
  SeCodePLT/CodeGuard+ on Qwen, produce the first measured before/after via the E-9 harness
  (crates/cli/src/benchmark.rs --suite seccodeplt) reporting insecure-rate alongside functional pass@1.
- Integration: new crates/rules/rules/security/*.toml; PreToolUse BLOCK; benchmark.rs extension.
- observable_effect: F-2a - a generated SQLi snippet is BLOCKed pre-write with a security_rule_blocked
  row. F-2b - `elai benchmark --suite seccodeplt` writes insecure_rate_before/after to a phase TOML +
  emits insecure_rate_measured. WHY THIS IS THE KEYSTONE: it is R1 (a real number on ELAI's engine) +
  R2 (the anti-Replit demo) + first-in-category, in one deliverable. Do NOT ship the claim from the
  transferred CodeGen numbers; gate the headline on the measured Qwen result.

### F-3 — CaMeL / Tool-Filter data-plane defense for untrusted-content agents
- Cluster C3 | NEW | novelty GENUINELY_NEW (vs E-37 input-marking, E-24 MCP pin, C-11 OS isolation) |
  feasibility HARD | grounding OVERSTATED on token overhead (secondary-only; measure on ELAI IPC)
- Steal/build: Tier-C isolation defends the CONTROL plane; injected instructions inside repo/web/skill/
  MCP content still steer a tool-capable model (lethal trifecta). Adopt the capability/Dual-LLM pattern.
  CaMeL 77% provable-secure (VERIFIED); AgentDojo Tool Filter cut ASR 57.7->6.84% at 73% utility
  (VERIFIED).
- REVISED scope (split): phase 1 (MEDIUM, ship first, FSM-native, no IPC schema change) - Tool Filter:
  lock the researcher's tool set to a per-task minimal subset BEFORE any untrusted byte is observed;
  measure ELAI's own injection ASR before/after on an AgentDojo-style fixture. Phase 2 (XL, conditional)
  - full CaMeL SymbolicVar, gated on (a) primary-source token overhead from the PDF, (b) a measured
  overhead on ELAI's real IPC proving the 32K ceiling survives, (c) acceptance of the IPC breaking change.
- Integration: phase 1 hooks the existing per-phase FSM tool gate (PreToolUse path) + researcher
  dispatch. observable_effect: an AgentDojo-style adversarial fixture run through the researcher pipeline
  shows tool_filter_committed + a measured ASR drop; injected instruction cannot fire an out-of-set tool.

### F-4 — Test-integrity + output-grounding gate (the literal "cant-lie" half of the north star)
- Cluster C4 | HYBRID | novelty PARTIALLY_COVERED (C-22 stdout_hash on feasibility path; C-27
  DeterminismDrift+ELAI_SEED; C-28 mutation) | feasibility HARD
- Build: bind every claimed test outcome to an orchestrator-captured artifact so a fabricated PASS is
  detectable (Replit faked tests + fabricated 4,000 users + false rollback). Code-read: TestOutcomePayload
  journals {test_name, passed, duration_ms} but has NO stdout_hash; the auditor runs tests in its own LLM
  Bash tool so its PASS narrative is unbound. C-22 already adds stdout_hash on the feasibility path - so
  the pattern exists; the gap is TestOutcomePayload specifically.
- REVISED scope: add stdout_hash: Option<String> (blake3, the feasibility_probe.rs:168 pattern) to
  TestOutcomePayload; gate so the auditor cannot emit passed=true without a matching orchestrator-captured
  hash row in the same superstep. PRE-RESOLVE the blocking design question first: the auditor must run
  tests through an orchestrator-managed process slot (so there IS a stdout handle to hash) - otherwise the
  field is dead. Split the semantic output-grounding check to a later F-N (needs spec-pinned expected
  outputs). Pre-req on C-27 + C-28.
- observable_effect: inject a baby step whose test process exits 1 while the auditor claims PASS -> the
  AUDITING->MEMORIZING gate emits anti_fabrication_block because the SSB row has passed=true with
  stdout_hash=None (the fabrication signature). Dashboard: the audit-timeline shows the block.

### F-5 — Cross-subsystem staleness-quarantine + bi-temporal contradiction cascade
- Cluster C5 | HYBRID | novelty GENUINELY_NEW (E-5 ships bi-temporal for memory_facts ONLY) |
  feasibility HARD | grounding ACCURATE
- Steal/build: SHARD time-decay quarantine + the cross-subsystem handle_stale_memory self-healing
  cascade (invalidate -> demote dependents -> broadcast) + Mem0 A.U.D.N. + Graphiti bi-temporal
  non-destructive invalidation. E-5 already ships bi-temporal supersede for memory_facts; the gap is
  cross-subsystem SCOPE - propagate a quarantined fact's demotion to DKG concept annotations
  (concept_stale=1 ALWAYS, never populated), grounding_cache, and Cosmos artifacts via one broadcast.
- REVISED scope (the sharp hazard): concept_stale=1-always may be hard-coded into retrieval paths as a
  constant filter; first real 0 could silently change ranking with no test failure. Phase 1 (after B-30):
  audit every concept_stale consumer + add a retrieval contract test with concept_stale=0 BEFORE any
  population. Phase 2: the cascade with a BOUNDED fan-out queue (explicit channel capacity) to prevent
  scheduler-tick pile-up.
- Integration: crates/memory/src/decay.rs emits StalenessQuarantineBroadcast; consumers in
  crates/knowledge/src/dkg + cosmos_worker.rs. observable_effect: quarantine a fact -> within one tick,
  concept_stale=1 on dependent DKG concepts + a grounding_cache miss on re-query, via a mock-clock
  contract test, surfaced in a dashboard staleness panel.

### F-6 — Learned-skill library: trust ladder + critic-gated acquisition + staleness demotion
- Cluster C6 | HYBRID | novelty PARTIALLY_COVERED (C-25 TemplateCrystallizer by success-count; E-43
  telemetry; memory TrustTier T0-T3 is provenance, collides namespace) | feasibility HARD
- Steal/build: Voyager critic-gated acquisition (curriculum->gen->execute->critic gate->retry->commit,
  embedding retrieval) + SHARD T1-T4 trust ladder (promotion by invocation-count + failure-rate + min-days;
  top tier human-gated) + staleness demotion. The NEW capability: a trust tier that GATES AUTONOMY (a
  sub-top skill cannot run autonomous without a human gate). Voyager 3.3x is Minecraft, non-transferable.
- REVISED scope (decompose): C6a - SkillTrustTier (S1-S4, DISTINCT Rust enum + distinct SSB tags from
  memory TrustTier, auditor-enforced type barrier). C6b - extend TemplateCrystallizer's critic to REUSE
  the existing BestOfN MiniCritic verdict from the triggering SSB row (avoids triple-critic redundancy).
  C6c - staleness demotion cascade. Defer the 70%-reuse claim until concept_class SSB tagging + >=20
  acquisitions are logged.
- Integration: compiled_pipeline.rs (TemplateCrystallizer) + new skill_trust.rs + autonomous-mode gate.
  observable_effect: an S2 skill is BLOCKed from autonomous dispatch without a human gate (skill_trust_
  promoted / skill_demoted_stale rows); dashboard skill-library panel shows tiers.

### F-7 — Safety-lattice completeness proof + outcome-utility CBR template ranking
- Cluster C7 | HYBRID | novelty GENUINELY_NEW (E-53 proves FSM/IPC invariants, NOT the mutable=0 rule
  lattice; no plan does CBR ranking) | feasibility HARD | grounding OVERSTATED (Springdrift numbers are
  SYNTHETIC/UNREPRODUCED - design hypotheses, never targets)
- Steal/build: (1) prove ELAI's mutable=0 (security/finance) rule subset complete + monotone +
  conflict-free as a SET (not just unit-tested); (2) Laplace-utility CBR ranking for pipeline templates
  by learned OUTCOME utility, replacing first-try-ratio ranking.
- REVISED scope (split; do NOT bundle): (1) lattice completeness = an AMENDMENT to E-53's
  integration-point list (exhaustive mutable=0 pair enumeration + Z3 monotonicity checks), strictly
  sequenced AFTER E-53's z3/kani/verus stubs resolve (RUNPOD-PENDING). (2) CBR ranking = a standalone
  plan over the pipeline template store, Laplace weighting over EXISTING SSB outcome events; DROP the
  +10.7% target, replace with a measured delta after >=100 labelled sessions vs the first-try baseline.
- observable_effect: (1) `elai verify safety-lattice` reports zero conflict pairs over the mutable=0
  enumeration (safety_lattice_verified). (2) template ranking by learned utility emits
  template_utility_ranked + a measured P@k delta after the labelled corpus accrues.

### F-8 — SCF symbolic semantic conflict detector (self-mod pre-execution first)
- Cluster C8 | HYBRID | novelty GENUINELY_NEW (Phase 3G coherence_check is (name,type_sig) set-compare;
  C-50 ContractSpec is codegen-scoped; D-3/D-4/E-7 gate self-mod by score/consensus/drift, not semantic
  conflict) | feasibility HARD | grounding OVERSTATED (27.9% precision is a simulation limitation;
  SCF is sim-only, 0 stars, empty tests/)
- Steal/build: SCF IntentNode pre/postcondition conflict detection (3 types, LLM-free). The Pillar-4
  synthesis target ("DGM archive search under guards PLUS a real semantic pre-execution review - neither
  ships").
- REVISED scope (the strong steer): SELF-MOD half ONLY (one config promotion per session -> 27.9%-precision
  WARN-then-escalate tolerable; self_mod_safety.rs / promotion_gate.rs has no C-50 namespace collision).
  DEFER the inter-agent BSP half (27.9% FP in a cap=3 parallel loop = noise; C-50 collision); flag it
  RUNPOD-PENDING and evaluate Graphiti's shipped LLM-judged contradiction detection (acceptable on the
  cloud lane) as the higher-maturity alternative there.
- Integration: self_mod_safety.rs / promotion_gate.rs, pre-archive-admission. observable_effect: a
  self-mod config variant that contradicts an active locked-key invariant is flagged before archive
  admission on an injected DGM-class reward-hack fixture (symbolic_conflict_detected).

### F-9 — Hallucinated-dependency existence+age+provenance gate at WRITE-time
- Cluster C9 | HYBRID | novelty PARTIALLY_COVERED (D-11 registry_check.py is real HTTP; E-11 idea-framing
  block; E-41 PR-leg) | feasibility HARD | grounding ACCURATE (correction: ELAI's check is NOT shape-only)
- Steal/build: move D-11's real HTTP existence+publish-date checker from the PR boundary to ALSO fire as
  a PreToolUse hook the instant an import line is written; add cargo (crates.io); wrap Socket Firewall at
  install boundary (ops). Spracklen 19.7% hallucinated / 205,474 fake names / 43% recur is VERIFIED;
  Socket explicitly cannot close the fresh-slop gap (no CVE) - that codegen-time existence slot is the gap.
- REVISED scope: extension to D-11 + E-41 (prereq: D-11 ships). Add _check_cargo() to registry_check.py;
  wire a PreToolUse hook firing only when the path matches an import/use/Cargo.toml pattern AND the name
  misses a local LRU existence cache (TTL ~1h) - the cache keeps the network call OFF the hot per-write
  path (W-10). Per-ecosystem-configurable stale threshold (stable Rust crates go years without a publish).
  Move the Socket Firewall wrap to an ops runbook.
- observable_effect: a generated `import nonexistent_pkg` is refused before disk; write_time_slopsquat_blocked
  row; 0 hallucinated imports reach disk on a 50-sample 404-confirmed suite; 0 FP on the 20 most-used real
  ELAI deps.

### F-10 — Per-iteration security-monotonicity gate (the autonomous-loop blind spot)
- Cluster C10 | NEW | novelty GENUINELY_NEW (vs B-39 REFINE, B-22 auto-advance, D-4 tripwires, E-7
  Two-Gate - all correctness/score-domain only) | feasibility HARD | grounding ACCURATE
- Build: at each REFINE / auto-advance / self-mod iteration, fingerprint the generated DELTA by CWE-class
  and BLOCK/escalate if iteration N introduces a vuln class absent at N-1. Security degrades +37.6% over
  5 AI-improvement iterations (VERIFIED); a standalone LLM injects ~11x more vulns than humans, more for
  more-autonomous frameworks (VERIFIED v3).
- REVISED scope (the hidden cost): the gate struct is 1-2 days; the CWE classifier it calls does NOT
  exist and building one calibrated for Rust (sparse semgrep coverage) is L-effort itself. Split: phase 1
  (L) build + calibrate the classifier on ELAI's historical REFINE deltas to FP<5% on 50 clean patches
  per language; phase 2 (S-M) wire it. Run as a POST-write async check feeding the auditor, not a blocking
  per-write hook (W-10), to avoid stalling the live REFLECT pipeline.
- Integration: SecurityMonotonicityGate wired pre-REFINE-commit (chat_reflect.rs / live_cli.rs:~1685) +
  pre-auto-advance (task_runner.rs:~3996). observable_effect: inject a CWE-95 pattern into REFINE at
  iteration N+1 (absent at N) -> security_class_regression_blocked fires + REFINE halts before the patch
  commits; dashboard reasoning drawer shows the block.

## Post-F-10 extensions (added 2026-06-02 — PLANNED, implementation pending)
Four parts added after the original C1..C10 set, grounded in live code + primary sources and
adversarially verified (research run wm3rajyyo; basis at workflow/research/F-plan-next/00-RECOMMENDATIONS.md).

### F-2c — Low-FP CWE rules (798/321/532/95) on registered Py+TS + first measured baseline
- Extension of F-2/C2 | HYBRID | EXTEND_EXISTING (high) | feasibility EASY | effort S-M
- Steal/build: add 4 high-precision predicates (hardcoded-creds CWE-798, hardcoded-key CWE-321,
  secrets-in-logs CWE-532, eval/exec-injection CWE-95) as GrepPattern->AstPattern Composites on the
  ALREADY-registered Python+TS grammars (data/rules/web.toml is the working template), then RUN the
  shipped `elai bench seccodeplt` for the FIRST measured static-gate insecure-rate baseline.
- Existing ground truth (code-read): F-2a's 12 rules + seed_security_rules(mutable=0) ship live;
  F-2b's bench (bench_seccodeplt.rs + SecCodePltRunner + InsecureRateMeasured SSB) ships but was never
  RUN; TreeSitterAstEvaluator registers 10 langs incl python+typescript (NOT 16 — research overclaim
  corrected); detect_language_from_ext returns None for .svelte/.html/.css.
- REVISED scope: NO new dep, NO ast_adapter edit, NO SSB variant; multi-language + ast-grep-core
  DEFERRED (need an ast_adapter::new edit). Only existing-test edit = immune-rule row count 12->16.
- Integration: 4 TOMLs under crates/rules/seed/security/ via the existing loader; surfaces through the
  existing SecurityRulesPanel (no new frontend pair).
- observable_effect (done-gate): a Python `password = "..."` (CWE-798) Edit is BLOCKed pre-write
  (security_rule_blocked); `elai bench seccodeplt` emits one insecure_rate_measured row + a
  seccodeplt-*.json report (static-gate, synthetic-corpus caveat explicit).

### F-11 — Close one config-only DGM generation under existing guards (driver-only)
- NEW topic (post-C10) | HYBRID | INTEGRATE_NEW_F_PLAN (high) | feasibility HARD | effort M
- Steal/build: ELAI ships the full Darwin-Godel substrate INERT — config_archive (sigmoid
  lambda=10/alpha_0=0.5 parent-select + insert_variant/select_parent/sample_variant) + bandit
  (darwinian_bai) + the guard trio (two_gate_promote, trap_tasks cheat-patterns, blake3 tripwire,
  Layer-3 human gate), all built+tested with ZERO production callers (verified 3-file grep). Add the
  ONE missing piece: a driver closing propose->evaluate(real bench)->guard-chain->insert_variant->
  select_parent for ONE config-only generation.
- Existing ground truth (code-read): config_archive.rs:198/274/395 callerless; darwin_loop_cmd.rs
  drives a synthetic reward (:104) + terminates RUNPOD-PENDING (:127) without touching the archive;
  is_security_sensitive (self_mod_layer1a.rs:383) is on a SEPARATE path the archive admit bypasses;
  promotion_gate tau/K_m self-labeled UNCONFIRMED.
- REVISED scope: mutant = config/template variant ONLY (stays within 2510.04399 bounded-capacity +
  avoids the Layer-3 human gate by construction); driver MUST explicitly call is_security_sensitive
  (killer-risk #1) and thread the REAL candidate through trap scoring (killer-risk #3). Acceptance =
  loop-closed + a guard caught a trap mutant, NOT score-went-up. Multi-generation live run DEFERRED.
- Integration: driver in darwin_loop_cmd.rs (live `elai darwin-loop` entrypoint); +darwinian_admit
  SSB; ReasoningDrawer "Darwinian Admits" pair. Wire-dark behind feature darwinian-evolve +
  ELAI_DARWINIAN_LOOP_ENABLE (deliberate W-2 deviation for an autonomous self-mod loop).
- observable_effect (done-gate): `ELAI_DARWINIAN_LOOP_ENABLE=1 elai darwin-loop --generations 1` with
  an injected trap mutant -> >=2 darwinian_admit rows (one RejectedByGuard naming the firing guard +
  one LoopClosed proving select_parent ran after insert_variant).

### F-12 — Semantic contradiction pre-filter: local deterministic NLI judge (trigger-only)
- NEW topic (post-C10) | HYBRID | EXTEND_EXISTING (high) | feasibility HARD (net-new model) | effort M
- Steal/build: mirror Graphiti/Zep LLM-judged contradiction invalidation (invalid_at = invalidating
  fact's valid_at, non-destructive) but with a LOCAL deterministic-argmax NLI cross-encoder on ELAI's
  own ort/ONNX runtime — NOT the cloud lane (a cloud LLM judge breaks ELAI_SEED per arXiv:2506.09501).
  Second pass on audit_conflicts' Surface residue, localized to ~2 rows per call.
- Existing ground truth (code-read): audit_conflicts (contradiction.rs:64) only pairs BYTE-IDENTICAL
  keys and is itself WIRE-DARK (zero prod callers, verified 2-file grep); supersede/soft_close
  (store.rs:208/303) are the non-destructive primitives (docstring cites Zep 2.2.3); MemoryConflictDetected
  SSB has a FREE-FORM conflict_kind (no bump); reranker.rs is the ort deterministic-load pattern; NO NLI
  model staged today.
- REVISED scope: storage path UNCHANGED (trigger-only); confidence-gated auto-close, SurfaceToUser as
  the safe default; reuses MemoryConflictDetected (conflict_kind="semantic_contradiction", NO SCHEMA
  bump); also supplies the FIRST production caller of audit_conflicts. Determinism = "practically
  stable, NOT bit-identical" (honest framing). DKG exact-text join + F-5 age-quarantine stay as-is.
- Integration: new crates/memory/{semantic_contradiction,nli_onnx}.rs; wired into the live_cli.rs
  memory tick (sibling to F-5); MemoryFactsDrawer "Semantic Contradictions" pair. Tier-2 NLI ONNX model
  is ops-gated; a NliJudge trait seam ships the Tier-1 logic + fixture harness dark of the model.
- observable_effect (done-gate): with the model staged, the live tick over a cross-key contradiction
  pair soft-closes the older row (valid_to_ms == newer.valid_from_ms) + emits memory_conflict_detected
  (conflict_kind="semantic_contradiction"); deterministic fixture harness (fake judge) ships now.

### F-13 — Session archive on SSB schema-bump (wire-dark, EXTEND_EXISTING)
- NEW topic (post-C10) | HYBRID | EXTEND_EXISTING (high) | feasibility EASY | effort S
- Steal/build: close the single most-damaging gap vs Springdrift's git-backed whole-state recovery
  WITHOUT adopting the Backup-Actor mechanism — archive the stale ssb.db before the destructive
  schema-bump DROP so audit history is not silently destroyed.
- Existing ground truth (code-read): ssb run_migration DROPs prior-version ssb/ssb_files at the ONE
  destructive site (ssb.rs:14342, "data loss acceptable for old sessions"); session-DAG migration is
  NON-destructive (verdict overstated; corrected); checkpoints store hashes-not-bytes; no Backup-Actor
  / unified store; working tree is an external-git pointer. ELAI recovers by SSB-replay under ELAI_SEED.
- REVISED scope (deliberately minimal): one file-copy + one guard at the DROP site, gated by
  ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE; NO periodic committer, NO git dep, NO unified store. Gaps #1/#3/#4
  DEFERRED. Fallback = document-the-gap-build-nothing. Open W-2 disposition recorded for the implementer.
- Integration: new crates/orchestrator/session_archive.rs; archive call inside run_migration before the
  DROP (DROP still proceeds — additive); +session_archived SSB; a session/recovery drawer pair.
- observable_effect (done-gate): opening a stale-version ssb.db with the flag set writes
  workflow/sessions/{id}/archive/ssb-v{N}.sqlite (blake3-verified) + a session_archived row, and the
  current db is still freshly recreated (DROP preserved).

## Cross-cutting (what makes F-plan different from B/C/D/E)
- Every F-N's done-gate is wiring_contract_check.py --run-trigger PASS (W-1), not cargo-green.
- Default-off env gates are banned as a terminal state (W-2) - the exact night-run failure.
- Every dashboard-surfaceable signal needs a frontend_pair + cross_lang_diff PASS (W-4).
- Every observable_effect is a live end-to-end trigger (W-5); RunPod-gated items ship a deterministic
  fixture harness exercising the real path, with the live run as a separate gated checkbox.
- Tiered split (W-10) keeps network/subprocess/RunPod axes out of the fail-closed hot path.

End of F-SUMMARY_INDEX.md.
