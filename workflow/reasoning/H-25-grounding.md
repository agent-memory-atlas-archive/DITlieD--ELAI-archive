GROUNDING SWEEP -- phase H-25
Source: F:\.ELAI_workflow\docs\thefinalglance\H-plan\H-25_result_manifest_rederivation.md
Date: 2026-06-05
FSM: PLANNING | Session: s-8404

SPEC REFERENCES

_grounding/H-TRUST-GROUNDING-MAP.md:TS-6 | GROUNDED | evidence: map lines 79-85 confirm F-4 stdout_hash leg at task_runner.rs, evidence.rs:845, gate.rs:441 -- exact keywords present
_grounding/H-TRUST-GROUNDING-MAP.md:TS-5 | GROUNDED | evidence: map lines 70-77 confirm DriftComparator:317, rra.rs:200 RunArtifact, ELAI_SEED -- keywords match H-25 claims
_grounding/H-TRUST-GROUNDING-MAP.md:NN-9 | GROUNDED | evidence: map line 120-121 explicitly states "result manifest + re-derivation: ZERO hits" -- absent status confirmed
_grounding/H-TRUST-GROUNDING-MAP.md:TD-1,TD-2 | GROUNDED | evidence: map lines 133-141 define TD-1 (persisted N keystone) and TD-2 (frozen evaluator) -- keywords present
design T-9 + PART 0 | VERIFY_NEEDED | file path not resolvable from plan; no absolute path given; the grounding map references "D:\.REGRET_workflow\memory-bank\plans\elai-trust-hardening-design.md" which is on D: drive (deleted 2026-05-29 per memory); design content summarised in the grounding map itself (TS-5/TS-6/NN-9/TD-1/TD-2) but original is inaccessible -- treat as VERIFY_NEEDED (grounding map is the surviving proxy; claims it summarises are GROUNDED via code grep)
H-17 (n_trials) prereq | GROUNDED | evidence: trial_ledger.rs lines 11/113/332 show TrialLedger::trial_count SHIPPED; trial_count PRESENT (not PLANNED)
H-18 (dsr_pvalue/pbo) prereq | GROUNDED | evidence: deflation_gate.rs present in orchestrator/src; deflation_gate named in NN-2 as T-2 (H-18) BUILD surface; dsr_pvalue hit in deflation_gate.rs confirms H-18 fields exist
evidence.rs:840-850 (anti-fab filter) | GROUNDED | evidence: Read lines 843-847 show .filter(|r| r.passed && r.stdout_hash.is_none()) at line 845 -- exact match
gate.rs:435-445 (None-hash fabrication comment) | GROUNDED | evidence: Read lines 440-443 show "a passing row with None hash is the fabrication signature" comment -- exact match
rra.rs:180-260 (RunArtifact) | GROUNDED | evidence: Read lines 200-224 show pub struct RunArtifact with root_blake3, manifest fields, events -- on-topic; extends claim confirmed
predicate.rs:100-120 (SchemaRequired + AstPattern + MetricGate) | GROUNDED | evidence: Read lines 100-133 show AstPattern at 102, SchemaRequired at 121, MetricGate at 126 -- all three present exactly as claimed
rule_engine.rs:98 (CompiledPredicate) | GROUNDED | evidence: Read lines 95-104 show CompiledPredicate enum with MetricGate at 99; SCIP_DEF cited in grounding map at :98 maps to the enum definition
Citations/Grounding block: DVC dvc.lock + Sigstore/Rekor + in-toto | VERIFY_NEEDED | named as "provenance-binding precedent" but no source file is cited and RAG returns no ingested topic; methodology precedent claim is load-bearing (the plan derives the manifest-binding design from these systems)

CLAIM CLASSIFICATION

rra.rs:200 RunArtifact is the evidence manifest to extend | GROUNDED | Read rra.rs:200-224 confirms pub struct RunArtifact with blake3 root, manifest, events
blake3 for captured_output + data + env hashes | GROUNDED | CLAUDE.md HARD RULE + rra.rs content_blake3 at line 189-196 uses blake3::Hasher -- verified in source
DriftComparator exists at determinism.rs:317 | GROUNDED | Grep hit: determinism.rs line 317 "pub struct DriftComparator" confirmed
task_runner.rs:16268 emits stdout_hash via emit_test_outcome | VERIFY | MISMATCH: line 16268 is a .await in VerdictResult::Escalate path; actual emit_test_outcome stdout_hash call is at line 16514; grounding map repeats :16268 (map line 80); the functional claim is correct (stdout_hash is emitted) but the line number is wrong by 246 lines -- low risk but line citations in DETERMINISTIC_CHECK greps will point at wrong location
SchemaRequired + AstPattern + MetricGate in predicate.rs:102 | GROUNDED | Read predicate.rs:100-133 confirms AstPattern at 102, SchemaRequired at 121, MetricGate at 126
result-manifest verifier is ABSENT (NN-9) | GROUNDED | grounding map line 120-121 + Grep shows no result_manifest.rs in orchestrator/src -- confirmed absent, pure BUILD surface
H-17 trial_count SHIPPED (not PLANNED) | GROUNDED | trial_ledger.rs exists with TrialLedger::trial_count at line 332, tests at 502-649
AST_PATTERN forbids monkeypatch of metric harness | GROUNDED | predicate.rs AstPattern variant exists; rules-engine mutable=0 path confirmed in lifecycle.rs:871 (per grounding map TS-2 line 46)
Parser_expr injection risk is flagged as open question | GROUNDED | plan lines 157-159 explicitly name this as a risk; no overclaim made
Completeness guarantee (manifest = binding "every reported number") | UNGROUNDED | the plan claims "every reported empirical number MUST carry one" (Task 1) enforced at the result-reporting path; completeness (that ALL reporting paths are gated) is asserted but no enumeration of reporting call sites is provided; a missed reporting path = silent fail-open; not grounded via code audit
in-toto / SLSA / Sigstore as methodology precedent for provenance binding | INGEST | plan cites "DVC dvc.lock + Sigstore/Rekor append-only + in-toto" as precedent; RAG has no ingested topic; if the manifest binding design inherits guarantees from these systems (e.g. append-only, supply-chain attestation semantics), that claim needs source-grounding; if cited as inspiration only (design choice already in plan), risk is low but should be labeled
Replit failure class (faked tests + fabricated 4000 users) as motivation | VERIFY | stated in objective line 6 as a generalised failure class; this is a real incident (public); no arXiv needed; treat as GROUNDED for motivational framing
SCHEMA_VERSION 207 / TOTAL_VARIANTS 484 baseline | GROUNDED | grounding map lines 18-19 state these are LIVE 2026-06-04 from ssb.rs:2909 and action_type_lock.rs:549; plan correctly says "re-read + bump e45:294" (baseline+1 at edit time)

ACTIONS REQUIRED
- VERIFY line number mismatch: task_runner.rs:16268 cited in plan (line 13, line 37, grounding map line 80) does NOT contain emit_test_outcome; actual call is at task_runner.rs:16514. The functional claim is correct but the line cite is stale. Update the line citation in H-25 plan task description and grounding map before TESTING_FIRST to prevent grep-based DETERMINISTIC_CHECK from failing silently (the check on line 110 greps for "stdout_hash" broadly so will find it, but a targeted line-anchored audit will miss).
- VERIFY completeness: before EXECUTING, grep all reporting call sites that produce Sharpe/P&L/empirical numbers in the result-reporting path and confirm all route through verify_result; an enumeration is required to close the UNGROUNDED completeness claim. Pattern: rg "sharpe|p_and_l|backtest_result|empirical" .elai_cc/crates/orchestrator/src -- identify every reporting emit site.
- INGEST source: source-ground in-toto/SLSA/Sigstore/DVC if any guarantee claims from those systems are load-bearing in H-25. If cited only as design inspiration (not guarantees), label as "design-inspired-by" in H-25-decisions.md and close. If provenance-binding semantics are inherited (append-only, non-repudiation), spawn paper-ingestor mode=source with paths to in-toto spec or sigstore docs before TESTING_FIRST.
- UNGROUNDED claim "every reported empirical number MUST carry one [manifest]" -- verify by enumerating all reporting call sites before declaring ACCEPTANCE item W-3 PASS; a missed path is a silent fail-open (violates R2).

VALIDATION WARNINGS
- design T-9 + PART 0 source file is inaccessible (D: drive deleted 2026-05-29); grounding map is the surviving proxy; content is adequately summarised in TS-5/TS-6/NN-9/TD-1/TD-2 with grep-confirmed line numbers -- acceptable proxy but note the original is gone.
- task_runner.rs:16268 line citation is stale (off by 246 lines); low functional risk but will confuse future auditors.
- all sources validated (spec files exist, grounding map is present and on-topic)
