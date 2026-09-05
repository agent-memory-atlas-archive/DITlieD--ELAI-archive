# Verification Report: testing-trust-design
Source: F:\.ELAI_workflow\docs\specs\guide\testing-trust-design.txt
Date: 2026-05-17
Overall status: NOT_DONE
Confidence: HIGH

## Summary
The testing-trust design is a large proposal (Layers -1 through 6, 25 P-class problems, ~10K-14K LOC estimate). The codebase implements foundational infrastructure that Layer 0, 3, and 5 reference (SSB, DKG, MistakeBook, DeterministicAuditor, self_mod safety, MSI), but none of the eight layers' defining mechanisms are implemented. No new SSB action types from the design (WiringEvidence, WiringViolation, IntentConfirmed, MutationSurvived, DeterminismDrift, SiblingDivergence, ProvenanceAnchored, AttestationSigned, RhsiSpike, CitationRejected) exist. The document's own Status field marks it PROPOSAL (post-MVP, needs user review), and the codebase state confirms no T-0 through T-13 tasks have been started.

## Mechanisms verified DONE
- DeterministicAuditor (5-check gate: Build, VersionGate, FileSizeCensus, LazyCode, Completeness) — evidence: .elai_cc/crates/orchestrator/src/deterministic_auditor.rs:1
- MistakeBook (structured failure memory store, feeds next-gen prompts) — evidence: .elai_cc/crates/core/src/mistake_book.rs:1
- SSB append-only state board with schema-versioned action types (SCHEMA_VERSION 46, ~100 action types) — evidence: .elai_cc/crates/orchestrator/src/ssb.rs:195-961
- ValidationTier enum (Determinism/QualityAst/Unit/Integration/Stress, 5 tiers) — evidence: .elai_cc/crates/core/src/tool_dispatcher.rs:181-220
- EvidenceGate / AutonomousGate infrastructure — evidence: .elai_cc/crates/orchestrator/src/gate.rs, .elai_cc/crates/orchestrator/src/autonomous_gate.rs, .elai_cc/crates/orchestrator/src/evidence.rs
- BSP superstep engine (multi-agent coordination) — evidence: .elai_cc/crates/orchestrator/src/bsp/engine.rs, SsbActionType::BspSuperstep
- Self-mod safety LOCKED_KEYS dispatch gate (schema + audit keys permanently locked) — evidence: .elai_cc/crates/orchestrator/src/self_mod_safety.rs:70-89
- MSI (Modification Solidification Index) outcome-weighted moving average — evidence: .elai_cc/crates/orchestrator/src/msi.rs:1-41
- Staged evaluation engine (L3 structural change requires human approval) — evidence: .elai_cc/crates/orchestrator/src/staged_eval.rs
- stub_ban as commentary/contract convention (used in code comments and contract test assertions) — evidence: .elai_cc/crates/orchestrator/src/agent_role_dispatcher.rs:213,284; .elai_cc/crates/orchestrator/src/deterministic_audit_gate.rs
- DKG (symbol graph, callgraph, tree-sitter parsing) — evidence: .elai_cc/crates/knowledge/src/symbol_graph/types.rs, .elai_cc/crates/knowledge/tests/dkg_tests.rs
- mvp-wiring-audit-2026-04-18.md present (referenced as evidence base for Layer 0 design) — evidence: .elai_cc/docs/mvp-wiring-audit-2026-04-18.md
- proptest/Hypothesis used in codebase (existing PBT infra) — evidence: .elai_cc/crates/orchestrator/src/research_tree.rs, .elai_cc/crates/knowledge/proptest-regressions/

## Mechanisms PARTIAL
- stub_ban enforcement
  - exists: stub_ban concept as code comments and contract-test assertions in ~15 files
  - gap: no DKG-backed StubBanAuditor, no StubBanEntry type, no systematic production-path regex scan enforced at baby-step acceptance; used only as a naming/comment convention

- Layer 4 / ValidationTier mutation gate (tier 3.5)
  - exists: ValidationTier enum covers Determinism/QualityAst/Unit/Integration/Stress (.elai_cc/crates/core/src/tool_dispatcher.rs:181); TieredValidator dispatch in .elai_cc/crates/tools/src/tiered_validation.rs
  - gap: no Mutation tier (3.5), no MutationResult type, no MutationRunner wrapping cargo-mutants/mutmut, no mutation score gate, no EquivalentMutantCache; current 5-tier enum stops at Stress

- MSI / self-modification feedback
  - exists: MSI outcome-based moving average (.elai_cc/crates/orchestrator/src/msi.rs); self_mod_layer1a with apply-success tracking; MsiUpdate SSB action type
  - gap: no PRPO hybrid (PROCESS_SCORE + OUTCOME_SCORE + RHSI); no RHSI scorer; no PRLCoder process reward; mutation_quality_score not in MSI weighted components

- Self-mod layered immutability (L0/L1/L2/L3 classification)
  - exists: LOCKED_KEYS set (software-enforced dispatch reject); KNOWN_PARAMETER_KEYS closed enumeration; Layer 3 structural change requiring human_approved=true
  - gap: no cryptographic hash-sealing of L0 keys in a signed TOML manifest; no redacted-config view for self-mod agent; no L2 MSI-divergence pause mechanism distinct from L1; classification terminology (L0/L1/L2/L3) absent from code

- DKG / entrypoint reachability
  - exists: DKG symbol graph, tree-sitter parsing, symbol queries, callgraph edges; EntryPointsDetected SSB action type (B-34)
  - gap: no EntrypointSet loader from elai_cc.toml [wiring.entrypoints]; no closed production-entrypoints declaration in config; no DkgReachabilityAuditor that checks produces_symbols against the set at baby-step acceptance

## Mechanisms MISSING
- Layer -1 (Inference Determinism) — all sub-mechanisms:
  - D-1 Hardware-class lock (nvml-wrapper probe, SKU enforcement) — searched: DeterminismConfig, hardware_class, nvml, nvidia-smi, batch_invariant; no evidence
  - D-2 Batch-invariant kernels (TBIK, vLLM batch-invariant) — searched: batch_invariant, TBIK, tree_based_invariant; no evidence
  - D-3 LayerCast FP32 compute path — searched: LayerCast, compute_dtype, fp32_compute, layercast; no evidence
  - D-4 Prefix-cache hygiene (blake3 keys, PinnedCachingManager, prompt normalization) — searched: PinnedCachingManager, blake3 cache key; server.py uses sha256 (hashlib) not blake3; no key-swap logic
  - D-5 system_fingerprint pin + DETERMINISM_DRIFT SSB — searched: system_fingerprint, fingerprint_pin, DETERMINISM_DRIFT; no evidence
  - D-6 LLM-42 speculative verified decoding module — searched: LLM42, speculative_verified; no evidence
  - D-7 FACET R-DAG tool-call contract compiler — searched: FACET, r_dag, RDag, ToolContract; only one mention in plan_skeleton.rs comment
  - DeterminismConfig type — searched: DeterminismConfig; no evidence
  - DigitalSiblingResult / digital sibling parallel sidecar — searched: DigitalSibling, sibling_divergence; no evidence

- Layer 0 (Wiring Contract) — all new types:
  - WiringContract struct (produces_symbols / consumes_symbols / reachable_from / stub_ban / frontend_pair / observable_effect) — searched: WiringContract, produces_symbols, reachable_from, frontend_pair; no evidence
  - StubBanEntry type — searched: StubBanEntry; no evidence
  - ObservableEffect / ObservableEffectRunner — searched: ObservableEffect, observable_effect struct/type; no evidence
  - EntrypointSet type + [wiring.entrypoints] in elai_cc.toml — searched: EntrypointSet, wiring.entrypoints; absent from config
  - WiringEvidence / WiringViolation SSB action types (design #19/#20) — searched full SsbActionType enum; not present
  - UserFacingClassification tag for BabyStepSpec — searched: UserFacingClassification, USER_FACING tag; not present
  - ArchCodex DSL (FORBID_CALL, CIRCULAR_DEP_BAN, LAYER_VIOLATION, UNCONSUMED_EMIT predicates) — searched: ArchCodex, FORBID_CALL, CIRCULAR_DEP_BAN; no evidence
  - WiringContractValidator reverse-check — searched: WiringContractValidator; no evidence

- Layer 1 (Intent Formalization):
  - IntentPair / IntentPairSet / IntentConfirmation types — searched: IntentPair, IntentPairSet, IntentConfirmation, TiCoder; no evidence
  - INTENT_CONFIRMED SSB action type — searched full SsbActionType enum; not present
  - PLANNING -> TESTING_FIRST gate requiring >=3 IntentPairs — searched: intent_confirmed, gate evidence; no evidence

- Layer 2 (Property and Metamorphic Tests):
  - PropertyTest type in BabyStepSpec — searched: PropertyTest, BabyStepSpec.properties; no evidence
  - MetamorphicRelation / RelationLibrary types — searched: MetamorphicRelation, RelationLibrary; no evidence
  - TESTING_FIRST gate requiring >=1 property or metamorphic test — no evidence
  - rules.db metamorphic relation seed entries — rules.db exists but no metamorphic content confirmed

- Layer 3 (Generation-Time Constraints) — new additions:
  - XGrammar CFG constrained decoding (replacing Outlines) — searched: XGrammar, xgrammar; no evidence; server.py uses outlines as fallback only (no XGrammar)
  - CRANE reasoning pocket (unconstrained CoT window inside constrained output) — searched: CRANE, crane_pocket; no evidence
  - CodeFast / GenGuard JSON decoding — searched: CodeFast, GenGuard; no evidence
  - PRLCoder process rewards (incremental compile+lint feedback as PRM signal) — searched: PRLCoder, process_reward; no evidence

- Layer 4 mutation gate new types / tools:
  - MutationRunner (cargo-mutants, mutmut, LLMorpheus adapters) — searched: MutationRunner, cargo-mutants, mutmut, LLMorpheus; no evidence
  - MutationResult / MutationThreshold / DifferentialResult / FormalProofResult / FuzzResult types — no evidence
  - DifferentialRunner (2-impl generator, framing-diverse prompts) — searched: DifferentialRunner; no evidence
  - FormalVerifierHook / Kani / crosshair / VeriStruct integration — searched: kani, crosshair, VeriStruct, FormalVerifierHook; no evidence
  - FuzzRunner (cargo-fuzz, AFL++, PILLM) — searched: FuzzRunner, cargo-fuzz, AFL; no evidence
  - EquivalentMutantCache (SQLite-backed, blake3-keyed) — searched: EquivalentMutantCache; no evidence
  - ConTested consistency vote (tier 3.55) — searched: ConTested, consistency_vote; no evidence
  - MUTATION_SURVIVED / FORMAL_PROVED SSB action types — searched full SsbActionType enum; not present
  - AUDITING -> MEMORIZING gate requiring mutation_score >= threshold — no evidence

- Layer 5 (Learning from Failure) new mechanisms:
  - MuTAP feedback loop (surviving mutants injected into next test-gen prompt) — searched: MuTAP, mutant.*feedback; no evidence
  - AdverTest co-evolution (Mutant-Generation + Test-Generation paired agents) — searched: AdverTest, co_evolution; no evidence
  - CiteAudit / FRONT citation verification shim — searched: CiteAudit, FRONT citation; minimal mentions in lib.rs/prompt.rs but no implementation
  - CITATION_REJECTED / INTENT_VIOLATED / REGRESSION_ADDED / RHSI_SPIKE SSB action types — searched full SsbActionType enum; not present
  - RegressionCorpusManager (per-project bug corpus) — searched: RegressionCorpus, regression_corpus; no evidence
  - PRPO hybrid MSI with RHSI — searched: RHSI, prpo, MSI_composite; not in msi.rs or self_mod_types.rs

- Layer 5.5 (Cryptographic Provenance):
  - ProvenanceEntry / ProvenanceLog / ProvenanceBundle / AttestationArtifact types — searched: ProvenanceEntry, ProvenanceLog, merkle, ed25519; found only elai-remote.bat mention; no implementation
  - RFC6962 Merkle log implementation — no evidence
  - Ed25519 keypair per ELAI install — no evidence
  - RFC3161 timestamp client — no evidence
  - PROVENANCE_ANCHORED / ATTESTATION_SIGNED SSB action types — searched full SsbActionType enum; not present
  - `elai trust attest` CLI subcommand — searched: trust attest; no evidence

- Layer 6 new mechanisms:
  - Risk classification engine (TRIVIAL/STANDARD/CRITICAL/PROVEN/UNVERIFIED) — searched: RiskClassification, TRIVIAL.*CRITICAL; no dedicated type or engine found; action_classifier.rs does not implement this taxonomy
  - testing-risk.toml per-project override file — searched: testing-risk.toml; not present
  - Rulebreakers Paired-Accuracy tau runner — searched: rulebreakers, paired_accuracy, tau.*0.9; no evidence
  - Reference-model independence enforcement (auditor must differ from executor on CRITICAL) — searched: reference_model_independence, AUDITOR_MODEL SSB field; no evidence
  - Dashboard trust tier surface (per-concept trust tier + drill-down) — searched: trust_tier, TrustTier; no evidence
  - `elai wiring audit` CLI subcommand — no evidence
  - `elai trust <project>` CLI subcommand — no evidence

## Mechanisms UNVERIFIABLE
- Firecracker / gVisor / Kata microVM isolation for CRITICAL observable-effect tests — the design references Phase 4A isolation tiers; Phase 4A exists but microVM routing for CRITICAL classification cannot be confirmed without running the system
- AutonomousGate WIRING_FAIL / OBSERVABLE_FAIL -> HIGH risk routing — AutonomousGate exists (.elai_cc/crates/orchestrator/src/autonomous_gate.rs) but these specific failure-type routes cannot be confirmed without reading the full dispatch logic against unimplemented types

## What must be done (action list)
- [ ] Implement T-(-1): DeterminismConfig type + hardware-class lock (nvml probe) + blake3 prefix-cache keys + system_fingerprint pin + DETERMINISM_DRIFT SSB action type in sidecar + providers crate
- [ ] Implement T-0: WiringContract type (produces/consumes/reachable_from/stub_ban/frontend_pair/observable_effect), StubBanAuditor, DkgReachabilityAuditor, ObservableEffectRunner, EntrypointSet loader from elai_cc.toml [wiring.entrypoints], WiringEvidence + WiringViolation SSB action types
- [ ] Implement T-1: MutationRunner wrapping cargo-mutants (Rust) + mutmut (Python); add Mutation tier 3.5 to ValidationTier enum; add mutation_score gate to AUDITING -> MEMORIZING transition
- [ ] Implement T-2: IntentPair / IntentPairSet / IntentConfirmation types; INTENT_CONFIRMED SSB action type; PLANNING -> TESTING_FIRST gate requiring >= 3 pairs
- [ ] Implement T-3: Metamorphic relation seed library in rules.db; MetamorphicRelation / RelationLibrary types; BabyStepSpec.properties field; TESTING_FIRST gate requiring >= 1 property or MR
- [ ] Implement T-4: Risk classification engine (TRIVIAL/STANDARD/CRITICAL/PROVEN/UNVERIFIED) from DKG effect tags + module path regex + testing-risk.toml override; wire into Layer 6 dashboard surface
- [ ] Implement T-5: EquivalentMutantCache (SQLite-backed, blake3-keyed per pre-mutation AST); ACH-style LLM equivalence classifier shim
- [ ] Implement T-6: RegressionCorpusManager (per-project persistent bug corpus, runs on every sibling baby step); REGRESSION_ADDED SSB action type
- [ ] Implement T-7: FormalVerifierHook wrapping Kani opportunistically (DKG scan for unsafe + bounded pure functions); FORMAL_PROVED SSB action type
- [ ] Implement T-8: Cryptographic provenance kernel (RFC6962 Merkle log, Ed25519, RFC3161 timestamp client, SSB hash-chain, `elai trust attest` CLI); ProvenanceLog / ProvenanceBundle / ProvenanceEntry types; PROVENANCE_ANCHORED + ATTESTATION_SIGNED SSB action types
- [ ] Implement T-9: CiteAudit shim for phase-X-grounding.md outputs; CITATION_REJECTED SSB action type; FRONT alignment scoring integration
- [ ] Implement T-10: PRPO hybrid MSI refactor (PROCESS_SCORE + OUTCOME_SCORE + RHSI weighted); RHSI scorer; RHSI_SPIKE SSB action type; wire mutation_quality_score into MSI components
- [ ] Implement T-11: Rulebreakers Paired-Accuracy tau runner for CRITICAL classification (paired follow/break IntentPairs, tau >= 0.9 gate)
- [ ] Implement T-12: XGrammar integration in providers crate (replacing Outlines default); CRANE reasoning pocket for STANDARD+ JSON-mode
- [ ] Implement T-13: FACET R-DAG tool-contract compiler for ToolDispatcher (deterministic arg-schema rejection)
- [ ] Add [wiring.entrypoints] section to elai_cc.toml with the closed production entrypoint set
- [ ] Add L0 cryptographic hash-sealing to self_mod_safety.rs LOCKED_KEYS (currently software-only, not tamper-evident)

## Notes
The design document is explicitly PROPOSAL status (post-MVP). The absence of implementation is expected per the document's own timeline (T-0 through T-13 are pre-implementation tasks listed for post-MVP execution). The codebase provides real foundations that T-0 through T-13 can build on: DKG symbol graph is functional, SSB schema is extensible (SCHEMA_VERSION 46), MistakeBook exists, self_mod safety gate exists, DeterministicAuditor exists, BSP superstep engine exists, and proptest infrastructure is already used. The existing ValidationTier and TieredValidator can be extended to add the mutation tier (T-1) without rewriting the existing 5-tier chain.
