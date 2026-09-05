# DEEP-PHASE-006 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1394-L1404`, heading `### DEEP-PHASE-006`; the next chunk starts at L1406.
- Declared chunk: family PHASE; **5 files**, **2,791 logical lines**, **174,259 bytes**, all five classified `numbered/index original`; target status **within 1,800–2,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all five listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `01-phase-index-MASTER.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `1H:Lx-Ly`, `3I:Lx-Ly`, `3K:Lx-Ly`, `4A:Lx-Ly`, and `4H:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 290 | 26,430 | 918196c6188165434d0933fc422002ba58571eb4e3c52645fd70e17b14df8a49 | docs/specs/guide/steps/phase-1H.txt | exact |
| numbered/index original | 465 | 29,892 | ee9455d1cdbb4001df33e8d254d3ac7104e50668a764aaf9eab212aa4f5f494f | docs/specs/guide/steps/phase-3I.txt | exact |
| numbered/index original | 317 | 17,348 | 582c9d9b378529533e099d854a17fa1eb28171d8877df4d86af8cf027dc593e6 | docs/specs/guide/steps/phase-3K.txt | exact |
| numbered/index original | 427 | 25,113 | 319b96ed8902212550fb6ece82dfe1c94aa2155254358c31dec65fc4f270ba43 | docs/specs/guide/steps/phase-4A.txt | exact |
| numbered/index original | 1,292 | 75,476 | 4c7472fffc73db1c463bae62e06ed05e7a2b6e7812159919d30af26fd5727b38 | docs/specs/guide/steps/phase-4H.txt | exact |
| **Aggregate** | **2,791** | **174,259** | — | **5 exact paths** | **exact** |

## Per-file atomic ledgers
### 1H — conditional SPLADE keyword expansion

Source: `docs/specs/guide/steps/phase-1H.txt` (290/290 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1H-E001 | E/STATUS | Phase 1H is conditional SPLADE keyword expansion evaluation; prerequisites Phase 1G complete. | 1H:L1-L2 |
| P1H-E002 | E/CAP | Goal: evaluate SPLADE sparse keyword expansion behind cfg(feature="splade"); compare prithivida/Splade_PP_en_v1 vs naver/splade-v3; pick better on code corpus then keep/drop. | 1H:L6-L9 |
| P1H-E003 | E/FAIL | DROP if <5% recall improvement over BM25-only on test corpus. | 1H:L10-L10 |
| P1H-E004 | E/LIMIT | CRITICAL RISK: BERT WordPiece vocab (30522 tokens) mismatches code tokens (fn, impl, async, Vec<T>, unwrap()); phase MUST measure vocabulary coverage before concluding SPLADE helps on code. | 1H:L12-L17 |
| P1H-E005 | E/DEP | Pre-read knowledge.txt L3.25 SPLADE (L57-62), query pipeline (L88); architecture-overview §10 ort/tokenizers/model candidates; knowledge.txt §6 circuit breaker; §7 optional SPLADE load. | 1H:L19-L24 |
| P1H-E006 | E/LIMIT | knowledge.txt L58 typo "prithvida" and missing SPLADE-v3/quantized/vocab-risk; owned by s-702732; update when unlocked. Correct spelling is prithivida. | 1H:L24-L24, L34-L34 |
| P1H-E007 | E/FAIL | R24: SPLADE ONNX weight ~532MB vs marginal code-search gain. R25: NL benchmarks (29% nDCG@10 ESCI) may not transfer to code. R26: ort downloads prebuilt from Microsoft (air-gap risk). | 1H:L27-L29 |
| P1H-E008 | E/DEP | R27: ort does NOT include tokenization; needs separate tokenizers crate (may exist from 1G cross-encoder). R28: ort is unconditional for GraphCodeBERT/Granite; feature-gating SPLADE only gates SPLADE Session init, not ort. | 1H:L30-L31 |
| P1H-E009 | E/LIMIT | R29: programming tokens not in BERT vocab and split to char n-grams, reducing expansion quality. R30: 10-query eval underpowered; TREC standard ≥50 topics; effects <~15% undetectable with 10 queries. | 1H:L32-L33 |
| P1H-E010 | E/EVIDENCE | Grounding: PP_en_v1 ONNX 532MB main / 436MB quantized; ort pyke; RRF with BM25 GROUNDED knowledge.txt; 5% drop criterion GROUNDED; code-search recall UNGROUNDED; splade-v3 VERIFIED arxiv 2403.06789; 0.3 vocab threshold UNGROUNDED heuristic. | 1H:L37-L50 |
| P1H-E011 | E/CAP | Step 1: optional ort dep; create knowledge/splade.rs fully behind cfg(feature="splade"); stub SpladeExpander and expand_query returning empty Vec<(String,f32)>; re-export from mod.rs gated. | 1H:L55-L68 |
| P1H-E012 | E/TEST | Step 1 checks: cargo check no features compiles without SpladeExpander; cargo check --features splade includes module; non-gated code must not contain SpladeExpander. | 1H:L70-L87 |
| P1H-E013 | E/LIMIT | Do NOT test for ort absence; ort always present for other models. | 1H:L78-L80 |
| P1H-E014 | E/TEST | Step 2 TDD: feature-gated tests/knowledge/splade_tests.rs — expand_query synonym terms, empty query empty vec, long >10000 chars no panic, RRF differs from BM25-only, property finite weights >=0, integration recall BM25 vs BM25+SPLADE. | 1H:L90-L103 |
| P1H-E015 | E/TEST | Step 2 acceptance: ≥5 unit + 1 property + 1 integration; tests FAIL (red); at least one asserts non-empty expand; property corpus includes Unicode. | 1H:L105-L120 |
| P1H-E016 | E/CFG | Model path default .elai_cc/models/splade/model.onnx; override SPLADE_MODEL_PATH env. | 1H:L130-L131 |
| P1H-E017 | E/CAP | SpladeExpander::new loads ONNX via ort::Session and BERT tokenizer (vocab.txt + tokenizer.json). expand_query tokenizes to input_ids/attention_mask/token_type_ids, runs inference, returns top-k (token, weight). | 1H:L133-L136 |
| P1H-E018 | E/CAP | model_variant enum PrePooled vs RawMLM: PP_en_v1 may output pre-pooled; splade-v3 optimum export is RawMLM [1,seq,30522] requiring max-pool+ReLU+log1p in Rust. | 1H:L137-L139 |
| P1H-E019 | E/OBS | vocab_coverage(corpus_tokens)->f64 fraction of unique tokens mapping to single WordPiece; if coverage <0.3 SPLADE unlikely to help. | 1H:L140-L140 |
| P1H-E020 | E/CAP | rrf_fuse(bm25, splade, k=60, weight_bm25=0.5, weight_splade=0.5); ranks 1-based; k=max(k,1) prevents div-by-zero. | 1H:L141-L141 |
| P1H-E021 | E/RECOVERY | Circuit breaker on ONNX calls (knowledge.txt §6); SPLADE failure returns empty expansion so BM25-only continues. | 1H:L142-L143 |
| P1H-E022 | E/CFG | Both model candidates share same interface; only model_path differs: A PP_en_v1 532/436MB; B splade-v3 ONNX export. | 1H:L144-L147 |
| P1H-E023 | E/TEST | Step 3 adversarial: code-specific vocab_coverage on tree-sitter identifiers not all tokens; corrupted 1-byte model returns Err not panic; non-English no panic; expand_query p50 <2s/100-char excluding warmup; k=0 clamps; missing tokenizer files Err. | 1H:L159-L181 |
| P1H-E024 | E/FLOW | 4a early-exit: extract code identifiers via tree-sitter; if code-specific coverage <0.3 EARLY DROP without recall; log rationale; skip 4b-4d. Coverage 0.2-0.4 may still proceed to calibrate threshold. | 1H:L193-L201 |
| P1H-E025 | E/CFG | 4b models pre-staged outside session; quant Jaccard top-50 terms A-full vs A-quant; if overlap>=0.95 prefer quant (saves 96MB + ~3x). Jaccard only for same-model quant, NOT cross-model. | 1H:L203-L211 |
| P1H-E026 | E/FLOW | 4b cross-model: quick recall@10 on 10 of 50 queries; pick higher mean; if within 1% pick faster; log to workflow/reasoning/phase-1H-measurements.md. | 1H:L212-L216 |
| P1H-E027 | E/TEST | 4c corpus: 100 code files, 50 queries; ≥15 vocab-mismatch, ≥15 distractors, ≥10 straightforward keyword; Wilcoxon signed-rank on paired recall@10; exclude bm25_recall==0 from delta %. | 1H:L218-L230 |
| P1H-E028 | E/FLOW | 4d KEEP if mean delta>=5% AND p<0.05; DROP if delta<5% OR p>=0.05; edge delta>=5% but p>=0.05: DROP default, preserve module, log inconclusive. | 1H:L232-L236 |
| P1H-E029 | E/TEST | Adversarial: ≥15 distractor queries; ≥15 code-syntax queries; all 50 queries run (no silent drop); clear caches between runs; ≥20 non-zero paired deltas else DROP; quant Jaccard >=0.80 vs full. | 1H:L248-L261 |
| P1H-E030 | E/STATE | Three exit outcomes: EARLY DROP vocab; DROP stats; KEEP. All preserve module behind cfg; DROP disables feature by default. KEEP: circuit breaker + RRF + winning model documented. | 1H:L273-L280 |
| P1H-E031 | E/FLOW | With splade disabled, SPARSE_ONLY cascade must degrade to BM25_ONLY (no dead path). | 1H:L281-L281 |
| P1H-E032 | E/OBS | Measurements: vocab_coverage_code, model_comparison table, recall_delta_percent mean+std, wilcoxon_p_value, splade_query_latency_ms p50/p95, model_load_time_s, binary_size_delta_mb. | 1H:L283-L290 |
| P1H-A001 | A/CFG | 0.3 vocab-coverage threshold is ungrounded heuristic, not published study; 0.2-0.4 band may still measure recall. | 1H:L201-L201 |
| P1H-A002 | A/TEST | If Candidate B unavailable, proceed with A only; full A/B protocol partially skipped. | 1H:L207-L207 |
| P1H-I001 | I/DEP | tokenizers may already exist from Phase 1G; step adds only if missing. | 1H:L128-L134 |

Category coverage: capabilities/actions ✓; workflows/states ✓; schemas/data ✓; flags/config ✓; integrations/events (none new) ✓; security (model path/circuit) ✓; failures/recovery ✓; determinism/stats ✓; observability/metrics ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No standalone UI panel; feature-flag and measurements.md are the operator surfaces.

### 3I — deterministic simulation testing

Source: `docs/specs/guide/steps/phase-3I.txt` (465/465 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3I-E001 | E/STATUS | Phase 3I deterministic simulation testing (DST); prerequisites 0.8 entropy quarantine + 3A-3D FSM/SSB/agents/BSP. NOT optional. | 3I:L1-L17 |
| P3I-E002 | E/CAP | Goal: prove FSM+SSB+BSP via deterministic simulation; reference model digital sibling is infallible oracle; same seed = same output. | 3I:L6-L9 |
| P3I-E003 | E/DEP | Pre-read orchestrator BSP/channels/invariants; core KEY TYPES + entropy traits L67-92; external-analysis DST L268-305. | 3I:L11-L14 |
| P3I-E004 | E/CAP | VCR pattern: intercept HTTP at transport (not LlmProvider trait); blake2s hash of full request body; store (hash,response) msgpack cassettes; DST lookup returns recorded response with simulated latency; missing hash fails (prompt drift). | 3I:L22-L40 |
| P3I-E005 | E/CAP | Trait-level LlmProvider mock is complementary unit pattern; VCR catches ollama cache anomalies trait mocks miss. | 3I:L42-L45 |
| P3I-E006 | E/CAP | TraceOps records entire agent trajectory (GRC, tools, branches) as structured trace; DST feeds trajectory instead of live LLM. | 3I:L47-L51 |
| P3I-E007 | E/CAP | Custom SQLite VFS traps write/fsync into in-memory buffer; sim PRNG dictates I/O latency; enables torn WAL, deterministic latency, crash recovery tests; ~200-400 LOC unsafe C FFI. | 3I:L59-L77 |
| P3I-E008 | E/DEP | Custom VFS must use madsim timer queue as single deterministic time source. SPIKE: evaluate if madsim libc I/O hooks make custom VFS unnecessary. | 3I:L72-L81 |
| P3I-E009 | E/FAIL | R1 madsim Windows unverified — spike first. R2 not drop-in tokio — needs cfg(madsim) and madsim-tokio swap. R3 must use proptest-state-machine ReferenceStateMachine not hand-rolled. R4 RUSTFLAGS madsim full rebuild. R5 10k cases need 10min timeout or reduced local count. | 3I:L84-L94 |
| P3I-E010 | E/TEST | 3I.1 madsim 0.2.34 dev-dep spike: two nodes send/receive on Windows with RUSTFLAGS; fail -> document manual SeededClock+SeededRng scheduler alternative. | 3I:L103-L119 |
| P3I-E011 | E/CAP | 3I.2 SimulatorRuntime wraps SeededClock+SeededRng+madsim; advance_time, inject_fault, snapshot_state. | 3I:L126-L127 |
| P3I-E012 | E/CAP | FsmReferenceModel implements ReferenceStateMachine; State=ReferenceState (FSM+SSB rows+agents+channels); Transition=FsmEvent; deliberately simpler than real system. | 3I:L128-L135 |
| P3I-E013 | E/DATA | FsmEvent 10 variants: PhaseTransition, AgentSpawn, AgentCrash, AgentCleanup, ToolCall, FileWrite, GrcStep, SSBWrite, ConfigChange, SuperstepBoundary. | 3I:L136-L144 |
| P3I-E014 | E/FLOW | SuperstepBoundary composite atomic: clear_non_accumulating_topics + execute + clear_ephemeral + evidence_gate_check. AgentCleanup models crashed->cleaned after crash. | 3I:L139-L144 |
| P3I-E015 | E/CAP | StateMachineTest: SUT=actual FSM+SSB; apply returns SUT by value (ownership model); check_invariants compares FSM state, SSB row counts, active agents, LastValue/Topic/Ephemeral channel state. | 3I:L145-L152 |
| P3I-E016 | E/CFG | dev-deps proptest v1.11+, proptest-state-machine v0.8+, madsim 0.2.34 cfg-gated; #[cfg(not(madsim))] stubs skip IPC sim gracefully. | 3I:L154-L157 |
| P3I-E017 | E/LIMIT | Reference model LOC <30% of real FSM LOC (oracle simplicity). Every FsmState reachable via BFS on reference state space. | 3I:L165-L169 |
| P3I-E018 | E/TEST | 3I.3 verify workspace: cargo test --workspace 0; warnings <= baseline. | 3I:L178-L188 |
| P3I-E019 | E/TEST | 3I.4 prop_state_machine sequential 1..200; PROPTEST_CASES=10000 CI 10min hard, 256 local; check_determinism takes bare static fn not closure; synthetic regression generates proptest-regressions/ checked into git; TDD red until 3I.8. | 3I:L190-L230 |
| P3I-E020 | E/TEST | 3I.5 PRE: entropy quarantine — zero Instant::now\|thread_rng\|SystemTime::now in orchestrator src (excl comments) else BLOCK. | 3I:L236-L240 |
| P3I-E021 | E/TEST | 3I.5 BSP: 100 identical checkpoints same seed; UntrackedValue never in checkpoint; EphemeralValue zero-write clears MISSING + write_count reset; Topic appends follow SSB seq; SuperstepBoundary two-phase Topic reset. | 3I:L241-L265 |
| P3I-E022 | E/TEST | 3I.6 madsim IPC (prep 4A): packet_loss_rate 0.3; send_latency 0..500ms Duration range; clog_link asymmetric; SIGKILL mid-message cleanup no orphaned SSB; burst 100% loss 5 msgs then 0%; seed replay; skip+document if madsim unavailable Windows. | 3I:L267-L300 |
| P3I-E023 | E/TEST | 3I.7 adversarial: anti-spiral 3x same GRC / >3 same-file edits STOP; evidence_gate_passes false without artifacts; context budget adaptive truncate; SSB concurrent same-path single-writer; demonic proptest interleavings. Whitespace-only triples still escalate; zero-byte evidence rejected. | 3I:L302-L328 |
| P3I-E024 | E/CAP | 3I.8 implement SimulatorRuntime with with_seed; FsmReferenceModel pure apply; SuperstepBoundary atomic clears; madsim::check_determinism bare fn; NetSim partition/delay/drop + buggify_with_prob; LOC ratio python assert. | 3I:L330-L376 |
| P3I-E025 | E/OBS | 3I.9 metrics phase_3I.toml: state_coverage_pct (>80% valid transition pairs), bugs_found, determinism_verified (100%), regression_count, ops_per_sec (ungrounded 100K target), compile_time_delta_sec, madsim_compile_time_sec. | 3I:L378-L401 |
| P3I-E026 | E/TEST | Exit: 10k+ sequences match; same seed identical; 100 BSP runs; IPC faults pass or documented skip; VCR cassette identical SSB; prompt hash mismatch reported; TraceOps all transitions; SQLite VFS spike done or hooks proven. AUTONOMOUS not enabled (4G). | 3I:L404-L417 |
| P3I-E027 | E/EVIDENCE | madsim drop-in claim FALSE; Windows UNGROUNDED; proptest-state-machine v0.8 VERIFIED; orchestrator.txt has NO DST section (SPEC_GAP — design in external-analysis); channel types GROUNDED orchestrator §§4-5. | 3I:L420-L468 |
| P3I-A001 | A/DEP | Whether custom SQLite VFS is required vs madsim hooks is unresolved until spike. | 3I:L79-L81 |
| P3I-A002 | A/CFG | Windows madsim availability and manual-scheduler equivalence details unresolved until 3I.1. | 3I:L85-L87, L110-L111 |
| P3I-I001 | I/DET | blake2s for VCR request hashing is the named hash in this phase (not blake3). | 3I:L30-L37 |

| P3I-E028 | E/DATA | ReferenceState includes FSM state, SSB row counts, agent lifecycle set, and channel state (LastValue current, Topic vec contents, Ephemeral presence); UntrackedValue excluded from reference channel state. | 3I:L129-L130, L339-L340 |
| P3I-E029 | E/TEST | 3I.4 adversarial: always-true preconditions must still produce at least one invariant failure; mutating one SSB row count in reference must be caught by check_invariants. | 3I:L222-L228 |
| P3I-E030 | E/TEST | 3I.5 adversarial: serialize checkpoint must not contain UntrackedValue marker; crash after ephemeral clear and SSB replay must restore EphemeralValue=MISSING. | 3I:L257-L261 |
| P3I-E031 | E/TEST | 3I.6 adversarial: crash agent mid-SSB-write yields no orphaned rows after cleanup; RUSTFLAGS madsim full rebuild expected and measured in compile_time metrics. | 3I:L286-L297 |
| P3I-E032 | E/CFG | madsim net::Config fields: packet_loss_rate f64, send_latency Range<Duration> with explicit Duration construction, NetSim::clog_link, buggify_with_prob. | 3I:L274-L277, L359-L360, L424-L426 |
| P3I-E033 | E/TEST | 3I.8 acceptance requires PROPTEST_CASES=10000 fsm_reference_model_matches within 10 min, all ipc_ tests under cfg(madsim), and cross-platform LOC ratio script. | 3I:L361-L376 |
Category coverage: capabilities/actions ✓; workflows/states ✓; schemas/data/lifecycle ✓; flags/config ✓; integrations/events ✓; security (entropy quarantine) ✓; failures/recovery ✓; determinism/concurrency ✓; observability/metrics ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No operator UI; metrics TOML and proptest-regressions are artifacts.

### 3K — RAG production hardening

Source: `docs/specs/guide/steps/phase-3K.txt` (317/317 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3K-E001 | E/STATUS | Phase 3K RAG production hardening; prerequisites 1E CatRAG/A-RAG, 1G cross-encoder, 3J sidecar for HyDE (3K.3 only). | 3K:L1-L3 |
| P3K-E002 | E/CAP | Upgrade crates/knowledge/ with adaptive chunking, query expansion, HyDE, quality-tier routing; better retrieval → smaller prompts for local LLMs. | 3K:L7-L14 |
| P3K-E003 | E/STATUS | Current stack: DKG 1A, BM25 1B, dual vectors 1C, PageRank 1D, CatRAG/A-RAG 1E, reranker 1G, SPLADE 1H KEEP/DROP done, resilience 1I. Missing: AST chunks, expansion, HyDE, tier routing, recall/MRR metrics. | 3K:L17-L35 |
| P3K-E004 | E/FAIL | R1 HyDE latency 1-3s mitigated by tier routing. R2 AST chunking reuses DKG LanguageGrammar. R3 expansion noise mitigated by template (not LLM) and 2-3 variants. | 3K:L38-L44 |
| P3K-E005 | E/CAP | 3K.1 adaptive chunking: replace 50-line windows; split Rust at fn/impl/struct/enum/mod; Python def/class; TS/Go function/class; non-code markdown headings or blank paragraphs; fallback 50-line on AST fail. | 3K:L49-L60 |
| P3K-E006 | E/CFG | Chunk targets 200-800 tokens; 50-token overlap; metadata file_path, start_line, end_line, chunk_type; Tantivy schema adds chunk_type + line range; re-index if chunk format version changes. | 3K:L61-L66 |
| P3K-E007 | E/TEST | 3K.1: boundaries align AST; no chunk >1000 tokens; overlap exists; parse fail falls back not crash; BM25 results include chunk_type; reindex without data loss. | 3K:L67-L76 |
| P3K-E008 | E/CAP | 3K.2 multi-query template expansion (no LLM): keyword extraction; CamelCase/snake_case symbol guess; synonym error→failure/exception; 2-3 variants for >5 words; skip <5 words. | 3K:L78-L93 |
| P3K-E009 | E/FLOW | 3K.2 run BM25+vector per expansion; merge dedupe by chunk ID; then cross-encoder rerank. Accept: Recall@5 improves; no >100ms latency regression on simple queries. | 3K:L91-L101 |
| P3K-E010 | E/CAP | 3K.3 HyDE: sidecar POST /v1/completions max_tokens=128 temperature=0.7 prompt Write short technical paragraph; embed hypothesis with 1C models; search with hypothesis embedding; RERANK against ORIGINAL query. | 3K:L103-L118 |
| P3K-E011 | E/CFG | HyDE cache hypothetical embeddings 5 minutes same query. Sidecar timeout 3s; unavailable/timeout degrade to direct search never block. Integrates 1I circuit breaker. | 3K:L119-L131 |
| P3K-E012 | E/TEST | 3K.3: HyDE for complex; simple skip zero latency regression; MRR improved on complex; graceful sidecar outage. | 3K:L123-L136 |
| P3K-E013 | E/CAP | 3K.4 QueryTier: FAST BM25-only <20ms; STANDARD hybrid+rerank no HyDE <200ms; DEEP expansion+HyDE+hybrid+rerank <5s. Distinct from 1I SearchMode degradation. | 3K:L138-L159 |
| P3K-E014 | E/CFG | Auto-detect: <5 words FAST; 5-15 code keywords STANDARD; >15 question/debug/arch DEEP. QueryPipeline::search optional tier; default AUTO backwards compatible. | 3K:L152-L157 |
| P3K-E015 | E/TEST | 3K.4: FAST skips reranker+vector; DEEP includes HyDE; three tiers measurably different latency; quality DEEP>=STANDARD>=FAST; existing callers unchanged. | 3K:L160-L168 |
| P3K-E016 | E/TEST | 3K.5 bench: 30 queries (10 simple/10 code/10 complex) known-correct chunks; Recall@5/@10, MRR, latency p50/p95 per tier; write .elai_cc/metrics/phase_3K_rag.toml; regress >5% blocks ship. | 3K:L170-L190 |
| P3K-E017 | E/EVIDENCE | Papers: A-RAG 2602.03442, CatRAG 2602.01965, Deep Research 2508.12752, Deep Search Agents 2508.05668; interface maps workflow/papers/*; independent of 4D web research. | 3K:L193-L199 |
| P3K-E018 | E/CAP | 3K.6 fix A-RAG keyword_search stub: Score_kw=SUM(count(k,text)*len(k)) case-insensitive; longer keywords weighted higher (arag-interface L72-77). | 3K:L202-L211 |
| P3K-E019 | E/CAP | 3K.6 fix semantic_search stub: dual-vector 1C; 3x oversample top_k*3; group by parent; score=MAX similarity among matches. | 3K:L212-L216 |
| P3K-E020 | E/CAP | 3K.6 progressive disclosure: search returns SNIPPETS not full chunks; ChunkRead full text only on explicit request; ContextTracker prevents re-read; ≥50% token savings goal. | 3K:L217-L224 |
| P3K-E021 | E/TEST | 3K.6 adversarial: short 10-line with 3 hits outscores 500-line with 3 (length domination); 60 candidates retrieval <100ms; arag contract tests pass. | 3K:L230-L246 |
| P3K-E022 | E/CAP | 3K.7 Symbolic Anchoring: rule-based CamelCase/UPPER_SNAKE entity extract; match DKG; weak PPR reset_weight(e)=epsilon*\|passages(e)\|^(-1); epsilon=0.2 from CatRagConfig. | 3K:L249-L260 |
| P3K-E023 | E/CAP | 3K.7 Key-Fact enhancement: w_hat=w*(1+beta*I(u,p in verified_triples)); beta=2.5; verified triples high-confidence DKG edges; pure algorithmic no LLM. | 3K:L261-L267 |
| P3K-E024 | E/DEFER | CatRAG Innovation 2 Dynamic Edge Weighting LLM-scored DEFERRED; too expensive for local code retrieval; re-evaluate after 4D web research LLM-in-loop. | 3K:L268-L271 |
| P3K-E025 | E/TEST | 3K.7: symbolic_anchor and key_fact_enhance symbols present; query 'how does the system work' extracts 0 entities; with beta=2.5 non-verified still in top-5; Recall@5 >= 3K.6. | 3K:L272-L294 |
| P3K-E026 | E/OBS | 3K.8 re-run same 30 queries; deltas Recall@5/@10/MRR/tokens-per-query/latency; phase_3K_rag_improved.toml; tokens-per-query >=30% reduction; no metric regress >5%. | 3K:L297-L317 |
| P3K-A001 | A/CFG | Exact synonym lexicon beyond error/failure/exception not fully specified. | 3K:L88-L90 |
| P3K-A002 | A/CFG | AUTO routing keyword lists for code/debug/architecture are descriptive not enumerated. | 3K:L152-L155 |
| P3K-I001 | I/SURFACE | 3K has no new operator UI; surfaces are QueryTier API, metrics TOML artifacts, and retrieval result snippet fields. | 3K:L156-L157, L181-L182, L308-L308 |

| P3K-E027 | E/INT | 3K.1 touches knowledge/src/bm25.rs chunking and knowledge/src/dkg.rs grammar reuse. | 3K:L76 |
| P3K-E028 | E/INT | 3K.2–3K.4 touch knowledge/src/pipeline.rs; QueryTier exported from knowledge/src/lib.rs. | 3K:L101, L136, L168 |
| P3K-E029 | E/INT | 3K.3 reuses providers/src/sidecar_client.rs; never blocks retrieval on sidecar outage. | 3K:L116-L122, L136 |
| P3K-E030 | E/INT | 3K.6–3K.7 touch knowledge/src/arag.rs and catrag.rs; contracts tests/contracts/arag_contract.rs must still pass. | 3K:L241-L246, L288-L294 |
| P3K-E031 | E/EVIDENCE | 3K.6–3K.7 grounding cites arag-interface.md and catrag-interface.md line ranges; progressive disclosure 90% paper figure is grounded interface claim, local acceptance uses ≥50%/≥30% token reductions. | 3K:L242-L245, L289-L293, L237-L239, L314 |
| P3K-E032 | E/TEST | Regression gate both 3K.5 and 3K.8: any metric regresses >5% vs prior baseline requires fix before complete. | 3K:L182-L185, L309-L312 |
Category coverage: capabilities/actions ✓; workflows/states ✓; schemas/data ✓; flags/config (QueryTier) ✓; integrations (sidecar HyDE) ✓; security (none new privacy) ✓; failures/fallback ✓; determinism ✓; observability/benchmarks ✓; tests/done gates ✓; constraints/deferred ✓. Surfaces: QueryTier API default AUTO, snippet fields, phase_3K_rag*.toml metrics.

### 4A — Tier-C agent process isolation

Source: `docs/specs/guide/steps/phase-4A.txt` (427/427 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4A-E001 | E/STATUS | Phase 4A Tier-C agent process isolation; prerequisite Phase 3 INTERACTIVE pipeline gates all pass. Foundational for all later Phase 4. | 4A:L1-L9 |
| P4A-E002 | E/CAP | Agents run as isolated OS processes; no agent sees enforcement, other agents, or harness; orchestrator communicates via unix socket / named pipe. | 4A:L6-L8 |
| P4A-E003 | E/DEP | Pre-read orchestrator §12, tools §8-9, reasoning §4 GroundingRequest IPC, audit-results-phase4 all 9 BLOCK+WARN, architecture-overview §9 roles. | 4A:L11-L16 |
| P4A-E004 | E/SEC | BLOCK-1: two seccomp profiles IPC_BASELINE (all) + NETWORK_EXTRA (researcher only); BPF checks socket domain arg not blanket socket. AF_UNIX baseline; AF_INET researcher only. | 4A:L19-L21 |
| P4A-E005 | E/SEC | BLOCK-2: researcher uses veth+iptables NOT CAP_NET_RAW; zero capabilities any role; prctl(PR_SET_NO_NEW_PRIVS) before exec. | 4A:L22-L23 |
| P4A-E006 | E/SEC | BLOCK-3: agent binary has NO LLM client; InferRequest→orchestrator→providers→InferResponse. | 4A:L24-L25 |
| P4A-E007 | E/SEC | BLOCK-4: GRC budget orchestrator-side; counts ThinkStep externally; agent budget advisory only. | 4A:L26-L27 |
| P4A-E008 | E/SEC | BLOCK-5: sockets in 0700 /tmp/elai_cc_{session}/; SO_PEERCRED on accept; unlink on agent death. | 4A:L28-L29 |
| P4A-E009 | E/SEC | BLOCK-6: auditor self-discovery → PROPOSED never auto-CANDIDATE; human approval; independent second auditor validates predicates. | 4A:L30-L32 |
| P4A-E010 | E/SEC | BLOCK-7: Windows weaker; AppContainer preferred; restricted token+ACL deny documented-weaker; Linux is production security target. | 4A:L33-L35 |
| P4A-E011 | E/SEC | BLOCK-8: self-mod locks BOTH IPC canonical realpath AND FS RO mount; BashRequest+GitCommit also filtered. | 4A:L36-L37 |
| P4A-E012 | E/SEC | BLOCK-9: adversarial IPC test battery required. | 4A:L38-L38 |
| P4A-E013 | E/CAP | 4A.1 AgentProcess wraps Command; IPC unix socket Linux / named pipe Windows; frame: 1-byte version + 4-byte length + postcard; MAX_FRAME_SIZE 10MB. | 4A:L43-L54 |
| P4A-E014 | E/DATA | Per-type size caps: AgentHeartbeat <1KB, ThinkStep <64KB, ToolResult <10MB, AgentResponse <context_limit. | 4A:L53-L54 |
| P4A-E015 | E/DATA | Exactly 15 message types: AgentRequest, AgentResponse, InferRequest, InferResponse, ToolRequest, ToolResult, BashRequest, BashResult, GroundingRequest, GroundingResult, ThinkStep, CheckpointSync, AgentHeartbeat, AgentKill, ReplanRequest. | 4A:L55-L59 |
| P4A-E016 | E/CAP | elai_cc_agent binary: GRC loop + IPC client + structured output parser; MUST NOT contain LLM client, reqwest, network, tools, FSM, SSB, rules, knowledge. | 4A:L60-L63 |
| P4A-E017 | E/CFG | ProcessConfig per role: allowed_paths, seccomp_profile, resource_limits. ToolFilter whitelist {allowed_tools}; not in list → generic "tool call rejected by policy". Defense-in-depth on PreToolUse. | 4A:L64-L70 |
| P4A-E018 | E/CFG | AgentDispatchConfig authoritative: role, model_preference, tool_filter, rule_domains, grc_config, sandbox_config. Role alone insufficient. | 4A:L71-L74 |
| P4A-E019 | E/EVENT | phase_complete SSB action_type at model-swap phase boundaries: {phase_id, agent_id, fsm_state, evidence_artifacts, commit_hashes, test_outcomes}; atomic durability marker; model unload only after fsync WAL. | 4A:L75-L79 |
| P4A-E020 | E/DATA | agent_id UUID v4 assigned at spawn by orchestrator. | 4A:L80-L80 |
| P4A-E021 | E/FLOW | Connection sequence listen-before-spawn: create 0700 dir → create socket listen → spawn with path → SO_PEERCRED/SecurityIdentification → version handshake mismatch kills. | 4A:L81-L87 |
| P4A-E022 | E/TEST | 4A.1: two binaries elai_cc + elai_cc_agent; cargo tree agent no reqwest/providers; 15 variants const_assert; postcard round-trip each type; 11MB frame rejected pre-alloc. | 4A:L88-L103 |
| P4A-E023 | E/TEST | 4A.2: cargo build/test/clippy -D warnings clean; cargo test 3x all pass (temp dir isolation). | 4A:L105-L120 |
| P4A-E024 | E/TEST | 4A.3 functional isolation TDD: cannot read .elai_cc ENOENT; NSpid==1 host-verified; AF_INET EPERM; AF_UNIX allowed; researcher HTTPS via veth; 15s heartbeat → SIGKILL; crash exit1 not crash orchestrator; cannot write outside scratch; concurrent agents isolated scratch and no peer AF_UNIX; auditor can read executor output via orchestrator copy; executor cannot read auditor; env only LANG/TERM/PATH/TMPDIR no HOME/USER/API/ELAI_CC_*; wrong PID reject; /proc masked /sys unmounted; RLIMIT_FSIZE 100MB fail write; RLIMIT_AS 512MB OOM agent only. | 4A:L122-L151 |
| P4A-E025 | E/TEST | 4A.3 LLM proxy: InferRequest path; no localhost:11434; oversize prompt rejected. | 4A:L153-L157 |
| P4A-E026 | E/TEST | 4A.3 GRC budget: orchestrator counts ThinkStep per topic; max_calls injects convergence; per-topic independent; orchestrator authoritative when agent local count disagrees. | 4A:L159-L166 |
| P4A-E027 | E/TEST | 4A.3 GroundingRequest proxy: FILE/WEB/SOURCE_CHECK; role-method matrix executor/auditor get FILE/SOURCE/WEB/TEST; context-compiler/prompt-engineer/memory-writer FILE+SOURCE only; TEST_CHECK from context-compiler rejected; counts to budget; no active ThinkStep topic rejected. | 4A:L168-L180 |
| P4A-E028 | E/TEST | 4A.3 test names test_4a_isolation_*/llm_proxy_*/grc_budget_*/grounding_*; one assert each; FAIL until 4A.5. | 4A:L182-L192 |
| P4A-E029 | E/TEST | 4A.4 adversarial IPC: >10MB reject; length=u32::MAX pre-alloc reject; malformed postcard no crash; unsolicited AgentResponse reject; ToolResult unrequested ToolId reject; 10k ThinkStep burst budget drop; second process SO_PEERCRED reject; SIGKILL socket cleanup respawn; mismatched agent_id reject; version mismatch kill; partial frame timeout cleanup. | 4A:L194-L211 |
| P4A-E030 | E/TEST | 4A.4 ReplanRequest: MINOR+evidence auditor fast-track; MAJOR human INTERACTIVE; max 1 replan per agent per phase; no evidence reject; evidence >500 tokens truncate/reject. | 4A:L213-L218 |
| P4A-E031 | E/TEST | 4A.4 self-mod locks: WriteFile locked path generic error; path traversal realpath reject; BashRequest locked RO mount block; GitCommit locked diff reject; symlink realpath reject. | 4A:L220-L225 |
| P4A-E032 | E/TEST | 4A.4 auditor discovery: PROPOSED not CANDIDATE; human before CANDIDATE; second auditor validates; inflated violation_patterns same-hash capped. | 4A:L227-L231 |
| P4A-E033 | E/TEST | 4A.4 crash: orchestrator crash restart SSB consistent; agent crash heartbeat respawn; SSB write BEFORE ToolResult send; partial frame discarded on fd close. | 4A:L233-L237 |
| P4A-E034 | E/TEST | 4A.4 Windows cfg: AppContainer no .elai_cc; Job limits memory/process/timeout; named pipe SecurityIdentification; fallback restricted token still tested. Every BLOCK 1-9 and WARN 1-9 has ≥1 test comment. | 4A:L239-L257 |
| P4A-E035 | E/CAP | 4A.5 agent: heartbeat every 5s; AgentKill 1s flush. Orchestrator gateway: routes Infer→providers, Tool→tools+rules+visibility, Bash→sandbox, Grounding→knowledge+web; GRC budget external; SSB sync before ToolResult. | 4A:L259-L280 |
| P4A-E036 | E/SEC | Linux: clone NEWNS\|NEWPID\|NEWNET; NO_NEW_PRIVS; seccomp IPC_BASELINE + NETWORK_EXTRA SOCK_STREAM AF_INET/6; bind-mount project RO + scratch RW; .elai_cc unmounted; /proc hidepid=2; socket dir NOT mounted into ns; self-mod locked RO; PID 1; RLIMIT_NPROC=1 FSIZE=100MB AS=512MB; env sanitized. | 4A:L282-L295 |
| P4A-E037 | E/SEC | Windows: AppContainer CreateAppContainerProfile preferred; fallback restricted token+Job+ACL; Job ACTIVE_PROCESS=1 PROCESS_MEMORY=512MB JOB_TIME configurable; named pipe SecurityIdentification. | 4A:L297-L302 |
| P4A-E038 | E/SEC | mutable=0 rules protected from all automated modification. Self-mod: IPC realpath + RO mount + GitCommit path filter. | 4A:L304-L312 |
| P4A-E039 | E/DEFER | ShortCoder post-process MUST run in orchestrator after AgentResponse BEFORE auditor view FS — not in agent; verify in 4B PRE_GATE (WARN-3). | 4A:L314-L317 |
| P4A-E040 | E/OBS | 4A.6 metrics phase_4A.toml: IPC p99 <1ms for <64KB; spawn <100ms; memory/agent <30MB; overhead vs in-process <5%; isolation escape fails. Each metric value/unit/timestamp/method. Separation cost estimates 0.1ms/50ms/20MB. | 4A:L337-L373 |
| P4A-E041 | E/INT | Every request-routing call site/struct must wire ssb, fsm, clock through single RuntimeClient; no None/uninitialized after construction. | 4A:L427-L427 |
| P4A-E042 | E/SURFACE | Operator-facing isolation surfaces: generic policy rejection string; human approval for MAJOR replan (INTERACTIVE gate); human approval for auditor PROPOSED rules; Windows vs Linux isolation strength documentation. | 4A:L67-L68, L215-L216, L228-L230, L33-L35 |
| P4A-A001 | A/OBS | IPC/spawn/memory/overhead SLO numbers are UNGROUNDED targets pending 4A.6 empirical measurement. | 4A:L346-L353, L376-L384 |
| P4A-A002 | A/CFG | WARN list 1-9 referenced for test mapping but individual WARN bodies not expanded in this phase file. | 4A:L257-L257 |
| P4A-I001 | I/SURFACE | No new dashboard panel is specified in 4A; isolation is infrastructure with generic IPC error strings as the agent-visible surface. | 4A:L67-L68 |

| P4A-E043 | E/CFG | Heartbeat interval 5s with 15s timeout → orchestrator SIGKILL. AgentKill grants 1s flush window for graceful shutdown. | 4A:L137, L269, L275 |
| P4A-E044 | E/SEC | Researcher NETWORK_EXTRA allows AF_INET/AF_INET6 SOCK_STREAM only via BPF argument inspection; other roles EPERM on AF_INET. | 4A:L134-L136, L285-L287 |
| P4A-E045 | E/SEC | Socket directory /tmp/elai_cc_{session}/ is NOT mounted into agent namespace; agent receives socket fd or path argument; per-agent scratch isolated. | 4A:L141-L142, L290-L291 |
| P4A-E046 | E/FLOW | Orchestrator routes: InferRequest→providers; ToolRequest→tools+rules+visibility_filter; BashRequest→sandbox; GroundingRequest→knowledge+web. | 4A:L276-L277 |
| P4A-E047 | E/TEST | Isolation escape acceptance: agent instructed to find .elai_cc/ must fail (measured in 4A.5 and 4A.6). | 4A:L335, L354-L365 |
| P4A-E048 | E/FAIL | Generic IPC policy rejection message is "tool call rejected by policy" with no rule ID leakage to agent. | 4A:L67-L68 |
| P4A-E049 | E/DATA | ReplanRequest max 1 per agent per phase; MINOR with evidence auditor fast-track; MAJOR requires human INTERACTIVE gate. | 4A:L213-L216 |
| P4A-E050 | E/EVENT | SSB write is synchronous with ToolResult dispatch: write BEFORE sending result (WARN-6 fix). | 4A:L236, L279, L404-L405 |
| P4A-E051 | E/DEP | Decision doc workflow/reasoning/phase-4A-decisions.md is PRE_GATE for 4A.5 IMPLEMENT. | 4A:L262 |
Category coverage: capabilities/actions ✓; workflows/states ✓; schemas/IPC data ✓; flags/config ✓; integrations/events ✓; security/permissions ✓; failures/recovery ✓; determinism ✓; observability/metrics ✓; tests/done gates ✓; constraints/deferred ✓. User-facing: generic policy rejection string, INTERACTIVE human approval for MAJOR replan and auditor PROPOSED rules, isolation strength docs.

### 4H — measured bounded self-modification

Source: `docs/specs/guide/steps/phase-4H.txt` (1292/1292 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P4H-E001 | E/STATUS | Phase 4H measured bounded self-modification (DGM/CEL/DarwinNet-derived), revised 2026-04-15 after adversarial audit (4 BLOCK+8 WARN resolved). Prerequisites 4A Tier C isolation. | 4H:L1-L24 |
| P4H-E002 | E/CAP | System improves configs/rules/data from pipeline outcomes via Layer1 metrics (no LLM), Layer1A data-table growth (no LLM), Layer2 staged config proposals, Layer3 structural human-only stubs. Modifying agent NEVER evaluates own modifications. | 4H:L7-L12 |
| P4H-E003 | E/LIMIT | Architectural divergence from DGM: 4H mutates CONFIG KEYS and DATA TABLES only, never agent/Rust source recompilation. | 4H:L36-L41 |
| P4H-E004 | E/CAP | Class A data-table additive low-risk Layer1A: deprecated_patterns, FailureTaxonomy, lazy-code regexes, source-tier entries; validate FPR over last N pipelines; no staged bench. | 4H:L43-L51 |
| P4H-E005 | E/CAP | Class B parameter systemic Layer2: ETC threshold, baby-step LOC, AB-MCTS temps, context weights, freshness half-life, retries, steering coeffs; staged 10/50/200. | 4H:L52-L56 |
| P4H-E006 | E/FAIL | R14 reward hacking — SWE-bench rotation + trigger task (DGM Appendix H). R16 resource contention idle-only lowest priority. R53 learned reward bias EVALUATE conditional. | 4H:L67-L72 |
| P4H-E007 | E/DATA | PipelineOutcome fields include pipeline_mode SURGICAL\|GREENFIELD\|ORACLE, first-try phases, audit findings, coverage, time, tokens, interventions, rules_triggered, test_outcomes, etc_trigger_events, stale_api_catches, context_budget_truncations, failure_taxonomy_misses. | 4H:L87-L103 |
| P4H-E008 | E/DATA | StaleApiCatch: file_path, deprecated_name, replacement_name, ecosystem, caught_by CatchLayer T1_HOOK\|T2_HOOK\|AUDITOR\|ETC. | 4H:L110-L116 |
| P4H-E009 | E/DATA | UnclassifiedError auto-proposes FailureTaxonomy when occurrence_count>=3 across pipelines. | 4H:L118-L123 |
| P4H-E010 | E/DATA | TestOutcomeReport 4-category: fail_to_pass, pass_to_pass, fail_to_fail, pass_to_fail (REGRESSION hard reject). | 4H:L125-L129 |
| P4H-E011 | E/CFG | Score always [0,1] weighted: first_try 0.3, coverage 0.2, efficiency 0.2, intervention 0.15, findings 0.15 via elai_cc.toml [self_mod.score_weights]. | 4H:L131-L140 |
| P4H-E012 | E/OBS | MSI = exponential-decay weighted moving avg of delta_score over N=10; lambda=0.3; weights oldest~0.07 newest 1.0; MSI in [-1,1]; inspired by DarwinNet PSI different formula. | 4H:L142-L152 |
| P4H-E013 | E/DATA | ConfigVariant: variant_id, parent_id, score [0,1], children_count, config_blob, created_at, pipeline_outcomes, pipeline_mode, status ACTIVE\|ARCHIVED\|REVERTED; SQLite table matches fields. | 4H:L154-L165 |
| P4H-E014 | E/FLOW | Parent selection DGM: s_i=sigmoid(10*(alpha_i-0.5)); h_i=1/(1+n_i); p_i proportional s*h; all non-zero; MODE FILTER — SURGICAL parent never for GREENFIELD eval. | 4H:L167-L175 |
| P4H-E015 | E/CFG | AllowedConfigKey closed enum Parameter keys: EtcThreshold, FreshnessHalfLife, FreshnessDocHalfLife, FreshnessBoostNorm, BabyStepMaxLoc{vram_tier}, ContextSlotWeight{slot}, RetryBudget, AbMctsMaxBranches, AbMctsTemperatures, MicroReflectionMaxTokens, SteeringVectorCoefficient, SelfModFrequency. | 4H:L177-L193 |
| P4H-E016 | E/CFG | AllowedConfigKey DataTable: DeprecatedPattern, FailureTaxonomyCategory, LazyCodePattern, SourceTierEntry. | 4H:L195-L199 |
| P4H-E017 | E/SEC | LOCKED never modifiable even human_approved: score_weights, gate_definitions, auditor_config, benchmark_harness, test_suite, config_variants_schema, safety_locks, lock_list, cloud_monthly_budget, cloud_opus_allowed, cloud_escalation_threshold. | 4H:L201-L207 |
| P4H-E018 | E/DATA | SelfModProposal: variant/parent ids, target_config_key AllowedConfigKey, ProposedChange InsertRow\|UpdateKey, ssb_evidence, reasoning_trace_evidence, rationale, optional ProjectScope. | 4H:L209-L228 |
| P4H-E019 | E/DATA | ProjectScope ecosystem/language/pipeline_mode filters prevent cross-ecosystem over-generalization. | 4H:L219-L224 |
| P4H-E020 | E/DATA | ReasoningTraceRef pinpoints GRC failure location (agent, phase, step index/type, failure_mode). StagedEvaluation Stage1(10)\|Stage2(50)\|Stage3(200) Class B only. | 4H:L230-L239 |
| P4H-E021 | E/DATA | DataTableValidation: FPR=false_positive_count/total; threshold default 0.05; window default 10; verdict PROMOTE\|REJECT\|INSUFFICIENT_DATA. | 4H:L241-L249 |
| P4H-E022 | E/DATA | ConfigVariantSimilarity ouroboros: 0.5*key_overlap+0.3*type_match+0.2*exact_match; STAGNANT if similarity>=0.95 over stagnation_window=3. | 4H:L256-L263 |
| P4H-E023 | E/STATE | OscVerdict: Stagnant{window,similarity}, Oscillating{period}, Evolving, Inconclusive{reason}. | 4H:L265-L269 |
| P4H-E024 | E/DATA | AcRegression + RegressionReport cross-pipeline AC history; never-passed AC is not regression; distinct from single-pipeline pass_to_fail. | 4H:L271-L290 |
| P4H-E025 | E/STATE | EvolutionGateResult Allow{evolved_count}\|Withhold{reason,identical_streak}; evolved_count pairs with similarity<0.95. | 4H:L292-L301 |
| P4H-E026 | E/CFG | SandboxConfig copy parent to .elai_cc/sandbox/{mod_id}/. Layer3 StructuralChange types FSM_STATE\|AGENT_ROLE\|GRC_PROTOCOL\|EVIDENCE_GATE\|PIPELINE_CONTROL\|ALLOWED_CONFIG_KEY_ADDITION require human_approved=true. | 4H:L303-L310 |
| P4H-E027 | E/SEC | Self-mod agent Tier C EXTRA restrictions: cannot write metrics/gates/auditor/benchmark/tests/schema/AllowedConfigKey def/locked keys; cannot READ lock list; four layers dispatch ToolFilter + IPC + FS + git. | 4H:L312-L319 |
| P4H-E028 | E/TEST | 4H.1 acceptance: AllowedConfigKey ≥12 Parameter + ≥4 DataTable; score [0,1]; parent_selection signature; config_variants schema match. | 4H:L321-L349 |
| P4H-E029 | E/TEST | 4H.3 config archive: max 50 variants; one ACTIVE per PipelineMode; mode-filtered parent; revert restores same-mode previous; overflow archives lowest non-ACTIVE; deceptive-dip ancestors survive prune even low score; ancestor-vs-cap collision keeps ancestor under cap. | 4H:L380-L433 |
| P4H-E030 | E/FLOW | 4H.3B deprecated_patterns: 3 catches → auto-propose PROPOSED scoped; FPR over 10 pipelines; <0.05 PROMOTE else REJECT. FailureTaxonomy: 3 misses → category+regex+fix template from successful resolution. Lazy-code: 3 auditor findings → GREP_PATTERN. Source-tier: domain 5+ as T3 with verified_facts/total>0.8 → propose T2. Scope: cargo rule not eval on TypeScript. | 4H:L442-L522 |
| P4H-E031 | E/LIMIT | Layer1A path does NOT trigger staged 10/50/200. Proposals use AllowedConfigKey DataTable variants only. | 4H:L517-L522 |
| P4H-E032 | E/FLOW | 4H.4 log-driven proposals read SSB full trace + reasoning_traces; must cite ssb_evidence>=1 and reasoning WHERE failed; target AllowedConfigKey; heuristics fallback only; DataTable→1A Parameter→staged. | 4H:L537-L581 |
| P4H-E033 | E/TEST | 4H.5 staged Class B: Stage1 10 discard on fail; Stage2 50 includes trigger_task; Stage3 200 if >40% on 60-subset AND top2 archive + 3 SWE rotation; PASS_TO_FAIL immediate reject; mode-aware tasks; Stage1 fail not archived; Stage2/3 fail stay for branching; Class A never enters. | 4H:L584-L652 |
| P4H-E034 | E/SURFACE | Trigger task surface: .elai_cc/benchmarks/trigger_task.toml with input, expected_output_hash sha256, metric_key — project-specific canary against objective hacking. | 4H:L589-L598 |
| P4H-E035 | E/SURFACE | Stage3 promotion gate requires better trigger task AND no SWE regression AND no auditor safety regressions AND no evidence-gate definition mods → ACTIVE demote previous ARCHIVED. | 4H:L612-L615 |
| P4H-E036 | E/SEC | 4H.6 four independent enforcement layers; generic error no path leak; cannot read lock list; locked keys rejected at dispatch; auto-revert >10% degrade over 3 pipelines; Layer3 human_approved enforced; cannot extend AllowedConfigKey; FPR threshold locked; objective-hacking trigger task. | 4H:L655-L723 |
| P4H-E037 | E/OBS | 4H.7 MSI pure arithmetic no LLM; IMPROVING MSI>0.3 frequency 1/3 pipelines; STABILIZING -0.1<MSI<=0.3 frequency 1/5; DIVERGING MSI<=-0.1 pause that mode + user alert + restore highest-score; per-mode independent; stored self_mod_history.db. | 4H:L734-L801 |
| P4H-E038 | E/SURFACE | DIVERGING MSI alerts user and pauses self-mod for that PipelineMode only (never silently continues). | 4H:L746-L746, L775-L779 |
| P4H-E039 | E/OBS | 4H.7B OSC: stagnation 3 consecutive sim>=0.95 → ConvergenceStagnant SSB + 2x exploration_bonus next 3 selections (does not block promotion). Oscillating period-2 full sim(N,N-2) AND sim(N-1,N-3) both >=0.95 needs ≥4 variants → ConfigOscillation SSB + freeze differing keys 5 pipelines. | 4H:L804-L848 |
| P4H-E040 | E/OBS | 4H.7C REG: cross-pipeline AC regression blocks 4H.5 promotion even if score up; 3 consecutive regressed pipelines on active → 4H.6 auto-revert; persistent never-passed not regression; per-mode. | 4H:L897-L970 |
| P4H-E041 | E/OBS | 4H.7D EVO: if evolved_count==0 AND latest_sim>=0.95 Withhold → EvolutionWithheld SSB and FORCE MSI STABILIZING (not IMPROVING) regardless of score noise; per-mode. | 4H:L973-L1050 |
| P4H-E042 | E/FLOW | 4H.8 Layer1 security/safety/architecture rules NEVER auto-adjusted. Layer1A four growth paths + ProjectScope + locked FPR. Layer2 log-driven + archive + staged + sandbox + rotation + 4-layer read-lock + AllowedConfigKey routing. Layer3 stubs only. | 4H:L1053-L1090 |
| P4H-E043 | E/FLOW | Self-referential alignment loop: pipeline→SSB→self-mod reads→propose→1A/2 validate→archive→next pipeline. | 4H:L1092-L1097 |
| P4H-E044 | E/SURFACE | Anti-staleness wiring: 4H triggers nightly elai dep-audit CLI; outdated deps → stale_api_catches → Layer1A deprecated_patterns → steering vector refresh metrics/compute_steering_vector.py. | 4H:L1099-L1106 |
| P4H-E045 | E/FLOW | Reasoning wiring: 3+ unclassified errors → FailureTaxonomy category + fix template from Layers2-6 resolution → future Layer1 deterministic sub-100ms. Token efficiency: truncations → ContextSlotWeight / BabyStepMaxLoc proposals. | 4H:L1108-L1121 |
| P4H-E046 | E/EVENT | SSB action_types: convergence_stagnant, config_oscillation, regression_detected, evolution_withheld. | 4H:L1133-L1134 |
| P4H-E047 | E/DEFER | Optional cross-transfer validation with different FM; FM-specific variants marked not universal. | 4H:L1136-L1139 |
| P4H-E048 | E/OBS | 4H.9 metrics phase_4H.toml: e2e self-mod time; Layer1A proposal/FPR/growth rates; Stage1 elimination >60% target / Stage3 <20% pattern; MSI trajectory; anti-fragility recovery; closed-loop tightness; stale catch trends; Layer1 classification rate. | 4H:L1186-L1238 |
| P4H-E049 | E/DEFER | EVALUATE SWE-World-style learned reward pre-screen ONLY if MSI shows >50% Stage1 fail (poor proposals); else skip. SWR accuracy numbers do NOT transfer; real Stage1 still runs after screen. Risk R53. | 4H:L1241-L1264 |
| P4H-E050 | E/INT | RuntimeClient ssb/fsm/clock wiring rule same as 4A exit. | 4H:L1290-L1292 |
| P4H-E051 | E/LIMIT | 4H specifies no new full dashboard panel or live cockpit UI; operator surfaces are alerts, CLI dep-audit, TOML metrics, and SQLite history only. | 4H:L1186-L1230 |
| P4H-A001 | A/CFG | max 50 archive cap is OUR DESIGN not DGM; resource-driven. | 4H:L437-L437 |
| P4H-A002 | A/CFG | MSI thresholds 0.3/-0.1 and 3 phases are OUR DESIGN vs DarwinNet 2 phases. | 4H:L796-L797 |
| P4H-A003 | A/CFG | Source-tier auto-promote verified_facts/total >0.8 and domain hit count 5+ are specified but broader tier-mapping rules remain design-local. | 4H:L483-L488 |
| P4H-I001 | I/SURFACE | self_mod_history.db and phase_4H.toml are operator-visible measurement/history surfaces for later cockpit consumption. | 4H:L751-L751, L1220-L1221 |
| P4H-C001 | C/LIMIT | DGM paper modifies agent code; 4H deliberately refuses code mutation — safety divergence must not be collapsed to "full DGM". | 4H:L36-L41 |

| P4H-E052 | E/CFG | Default score weights in elai_cc.toml [self_mod.score_weights]: first_try_rate_weight=0.3, coverage_weight=0.2, efficiency_weight=0.2, intervention_weight=0.15, findings_weight=0.15. | 4H:L134-L139 |
| P4H-E053 | E/CFG | Parent selection parameters lambda=10 and alpha_0=0.5 per DGM section 2.3; exploration h_i=1/(1+children_count). | 4H:L167-L172 |
| P4H-E054 | E/CFG | MSI exponential weights with N=10 lambda=0.3 listed as [0.07,0.08,0.10,0.13,0.17,0.22,0.29,0.37,0.48,0.61] then normalized. | 4H:L740-L742 |
| P4H-E055 | E/STATE | Only one variant status=ACTIVE per PipelineMode at a time; revert promotes previous highest-score ARCHIVED same-mode variant. | 4H:L391-L400 |
| P4H-E056 | E/FLOW | Archive overflow when count>50 moves lowest-score non-ACTIVE to cold storage; ancestor-of-active and children_count>0 protected from deceptive-dip prune. | 4H:L401-L425 |
| P4H-E057 | E/TEST | 4H.3B scope enforcement: cargo-scoped deprecated_patterns not evaluated on TypeScript pipelines; unscoped rules evaluate on all. | 4H:L490-L495 |
| P4H-E058 | E/TEST | 4H.3B FailureTaxonomy proposed pattern must match ≥3 distinct historical error instances, not 3 occurrences of one pipeline error. | 4H:L510-L513 |
| P4H-E059 | E/FLOW | Stage1 fail → variant discarded entirely (not archived). Stage2 fail → archived with Stage2 score. Stage3 fail → archived near-miss higher parent priority than Stage2 fails. | 4H:L616-L621 |
| P4H-E060 | E/TEST | Stage3 entry requires >40% success on 60-task subset AND top two performers in archive (DGM section 2.5). | 4H:L602-L604, L643 |
| P4H-E061 | E/SEC | Self-mod cannot write metrics code (objective hacking defense); trigger task known-answer catches metric gaming even if score improves. | 4H:L678-L684, L705-L708 |
| P4H-E062 | E/SEC | Layer1A cannot target Parameter keys; FPR threshold is locked and proposing change to it is rejected. | 4H:L685-L690, L714-L715 |
| P4H-E063 | E/OBS | Stagnant OSC does NOT block promotion (valid stop state) but multiplies exploration_bonus by 2.0 for next 3 parent_selection rounds in that mode. | 4H:L842-L846 |
| P4H-E064 | E/OBS | Oscillating freezes keys that differ between the two states into AllowedConfigKey runtime exclusion list for next 5 pipelines. | 4H:L847-L849 |
| P4H-E065 | E/EVENT | SSB rows ConvergenceStagnant / ConfigOscillation emitted on OSC verdicts; EvolutionWithheld on EVO Withhold. | 4H:L843-L848, L1007-L1008, L1133-L1134 |
| P4H-E066 | E/FLOW | REG used as additional gate at 4H.5 Stage3→ACTIVE promotion and every 3 pipelines on active variant for auto-revert. | 4H:L923-L927, L1127-L1130 |
| P4H-E067 | E/OBS | 4H.9 targets: Stage1 elimination rate >60% pattern, Stage3 reach <20% pattern (DGM cost pattern, measured as baseline not hard fail). | 4H:L1199-L1201, L1227 |
| P4H-E068 | E/FAIL | StructuralChange.apply without human_approved=true returns Err or panics; Layer3 is enforced not advisory. | 4H:L674-L675, L709-L711 |
| P4H-E069 | E/SURFACE | CLI surface: `elai dep-audit` nightly anti-staleness trigger invoked by 4H loop. | 4H:L1100-L1100, L1144 |
| P4H-E070 | E/DATA | self_mod_history.db stores MSI history and RegressionReport persistence. | 4H:L751, L1129-L1130 |
| P4H-E071 | E/TEST | 4H.8 adversarial: every proposal.ssb_evidence must reference logs from last COMPLETE pipeline with completed_at < proposal.created_at. | 4H:L1148-L1152 |
| P4H-E072 | E/LIMIT | Security/safety/architecture rules NEVER auto-adjusted by Layer1. | 4H:L1058-L1059 |
| P4H-E073 | E/DEFER | Layer3 structural execution deferred until system maturity proves need; only stubs+gate in this phase. | 4H:L1088-L1090 |
Category coverage: capabilities/actions ✓; workflows/states/lifecycle ✓; schemas/data ✓; flags/config/enums ✓; integrations/events ✓; security/permissions ✓; failures/revert/retry ✓; determinism ✓; observability/MSI/OSC/REG/EVO ✓; tests/done gates ✓; constraints/deferred ✓. Surfaces: user alert on MSI DIVERGING, Layer3 human_approved gate, `elai dep-audit` CLI, trigger_task.toml, phase_4H.toml, self_mod_history.db.

## Cross-source interactions and lifecycle

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| IX-001 | E | Retrieval ladder continuity: 1H SPLADE is optional sparse expansion between BM25 (1B) and dual vectors; 1I SearchMode cascade degrades SPARSE_ONLY→BM25_ONLY when splade feature absent (1H exit requires this). | P1H-E027, P1H-E032; P3K-E003 |
| IX-002 | E | 3K hardening composes 1E A-RAG/CatRAG stubs, 1G reranker, 1C embeddings, 1A DKG grammars, and 3J sidecar only for HyDE. | P3K-E001–P3K-E003, P3K-E010, P3K-E015–P3K-E018 |
| IX-003 | E | 3K.4 QueryTier intentional quality selection is explicitly distinct from 1I SearchMode fault-degradation cascade. | P3K-E012 |
| IX-004 | E | 3I DST proves FSM+SSB+BSP that 4A process isolation and 4H self-mod later depend on for crash recovery and durable evidence. | P3I-E001–P3I-E002; P4A-E036; P4H-E001 |
| IX-005 | E | 3I.6 madsim IPC fault tests are preparation for 4A agent IPC boundary. | P3I-E018; P4A-E013–P4A-E014 |
| IX-006 | E | 3I entropy quarantine (zero raw time/rng in orchestrator) is a hard PRE_GATE for BSP determinism and for any later seed-replay of isolation tests. | P3I-E016 |
| IX-007 | E | 4A isolation is prerequisite and sandbox boundary for 4H self-mod agent; 4H four enforcement layers include 4A IPC+FS+dispatch. | P4A-E001; P4H-E001, P4H-E027, P4H-E038 |
| IX-008 | E | 4A ToolFilter whitelist + generic policy rejection string is first barrier reused by 4H self-mod ToolFilter and locked-path messaging. | P4A-E016; P4H-E027, P4H-E038 |
| IX-009 | E | 4A BLOCK-6 auditor PROPOSED never auto-CANDIDATE + second auditor composes with 4H Layer3 human_approved and 3F self-discovery 3+ pattern. | P4A-E008; P4H-E026, P4H-E032 |
| IX-010 | E | 4A BLOCK-8 self-mod locks (realpath+RO+git) are the filesystem/git halves of 4H's four-layer enforcement; 4H cannot read lock list. | P4A-E010, P4A-E034; P4H-E027, P4H-E038 |
| IX-011 | E | 4A ShortCoder deferral to 4B orchestrator placement is a cross-phase WARN-3 handoff outside this chunk's implementation. | P4A-E035 |
| IX-012 | E | 4A phase_complete SSB durability marker and RuntimeClient ssb/fsm/clock wiring rule also close 4H exit criteria. | P4A-E018, P4A-E038; P4H-E048 |
| IX-013 | E | 4H Layer1A deprecated_patterns growth is fed by anti-staleness (T1/T2/AUDITOR/ETC catches) and `elai dep-audit`, completing Layer4 maintenance. | P4H-E009, P4H-E031, P4H-E043 |
| IX-014 | E | 4H FailureTaxonomy expansion closes reasoning-architecture Layer1 with fix templates from higher layers' successful resolutions. | P4H-E010, P4H-E032, P4H-E044 |
| IX-015 | E | 4H Class B staged eval SWE-bench rotation + trigger_task is distinct from 3K retrieval quality bench and 1H recall keep/drop. | P4H-E034–P4H-E036; P3K-E014; P1H-E024–P1H-E026 |
| IX-016 | E | 4H OSC/REG/EVO ouroboros ports operate per PipelineMode in parallel with MSI and feed SSB action types. | P4H-E040–P4H-E042, P4H-E045 |
| IX-017 | E | 3K progressive disclosure snippets and 4A agent isolation both shrink agent-visible context: 3K by snippet policy, 4A by mount/namespace denial of .elai_cc. | P3K-E017; P4A-E021 |
| IX-018 | E | Budget authority handoff: 2C in-process budgets become advisory under 4A orchestrator ThinkStep counting. | P4A-E006, P4A-E023 |
| IX-019 | I | 1H keep/drop outcome is recorded as prior state in 3K current-stack narrative; 3K does not re-run SPLADE evaluation. | P3K-E003; P1H-E027 |
| IX-020 | E | Both 4A and 4H require decision docs under workflow/reasoning/ (phase-4A-decisions.md, phase-4H-decisions.md) before IMPLEMENT. | P4A-E032; P4H-E041 |
| IX-021 | E | 3I VCR uses blake2s for prompt hashing while much of ELAI elsewhere bans blake2; this phase-local choice is intentional for cassette keys. | P3I-I001 |
| IX-022 | E | Researcher-only network (4A NETWORK_EXTRA/veth) is the only agent network path; 3K HyDE uses orchestrator-side SidecarClient not agent network. | P4A-E003–P4A-E004; P3K-E010 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | ambiguity | 1H 0.3 vocab threshold ungrounded; 0.2–0.4 band may still measure recall. | P1H-A001 |
| AC-002 | ambiguity | 1H Candidate B missing path skips full A/B comparison. | P1H-A002 |
| AC-003 | ambiguity/dependency | 3I custom SQLite VFS necessity vs madsim hooks unresolved until spike. | P3I-A001 |
| AC-004 | ambiguity | 3I Windows madsim availability and manual scheduler equivalence open until 3I.1. | P3I-A002 |
| AC-005 | ambiguity/config | 3K synonym lexicon and AUTO keyword lists incomplete. | P3K-A001–P3K-A002 |
| AC-006 | ambiguity/measurement | 4A IPC/spawn/memory/overhead SLOs are ungrounded until 4A.6. | P4A-A001 |
| AC-007 | ambiguity | 4A WARN 1–9 bodies not expanded in phase file despite required test mapping. | P4A-A002 |
| AC-008 | ambiguity/design | 4H archive cap 50 and MSI thresholds are local design choices, not paper transfers. | P4H-A001–P4H-A002 |
| AC-009 | ambiguity | 4H source-tier auto-promote thresholds (5 hits, 0.8 verified ratio) lack broader tier policy. | P4H-A003 |
| AC-010 | conflict | DGM-style code self-mod vs 4H config/data-only must not be flattened. | P4H-C001 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/01-phase-index-MASTER.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### 1H deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-1H-001 | **ADD** full risk register R24–R30 including air-gap ort download, tokenizers separate crate, and statistical power of 10 vs 50 queries. | MASTER:L135-L143 | P1H-E007–P1H-E009 |
| SD-1H-002 | **ADD** PrePooled vs RawMLM ONNX output shapes, RRF k=60 0.5/0.5 weights, circuit-breaker empty-expansion fallback, and SPLADE_MODEL_PATH default. | MASTER:L141 | P1H-E016–P1H-E021 |
| SD-1H-003 | **ADD** early-drop 0.2–0.4 calibration note, quant Jaccard≥0.95 prefer int8, cross-model within-1% pick-faster, Wilcoxon paired protocol, and ≥20 non-zero pairs power gate. | MASTER:L142-L143 | P1H-E022–P1H-E026, P1H-A001 |
| SD-1H-004 | **ADD** SPARSE_ONLY→BM25_ONLY cascade verification when splade disabled and full measurement field list including binary_size_delta_mb. | MASTER:L143 | P1H-E028–P1H-E029 |
| SD-1H-005 | **ADD** knowledge.txt typo/lock note and UNGROUNDED code-search recall labeling. | MASTER:L135-L143 | P1H-E006, P1H-E010 |

### 3I deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-3I-001 | **ADD** full VCR cassette design (blake2s HTTP body hash, msgpack, prompt-drift fail) and TraceOps trajectory layer. | MASTER:L349 | P3I-E004–P3I-E006 |
| SD-3I-002 | **ADD** custom SQLite VFS details, madsim timer coordination, and spike-to-possibly-skip VFS. | MASTER:L349 | P3I-E007–P3I-E008, P3I-A001 |
| SD-3I-003 | **ADD** complete FsmEvent ten-name list, SuperstepBoundary atomic substeps, AgentCleanup lifecycle, and by-value SUT apply semantics. | MASTER:L351 | P3I-E012–P3I-E014 |
| SD-3I-004 | **ADD** R1–R5 risk block, PROPTEST_CASES 10000/256 split, bare-fn check_determinism constraint, and proptest-regressions synthetic seed. | MASTER:L350-L354 | P3I-E009, P3I-E017 |
| SD-3I-005 | **ADD** entropy-quarantine BLOCK PRE_GATE, burst-loss 5×100%, whitespace anti-spiral, zero-byte evidence rejection, and ops/sec ungrounded 100K target. | MASTER:L355-L358 | P3I-E016, P3I-E018–P3I-E021 |
| SD-3I-006 | **ADD** SPEC_GAP that orchestrator.txt has no DST section; design lives in external-analysis. | MASTER:L346-L358 | P3I-E023 |
| SD-3I-007 | **ADD** exit note AUTONOMOUS not enabled (Phase 4G) and HITL walking-skeleton is multi-phase. | MASTER:L358 | P3I-E022 |

### 3K deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-3K-001 | **ADD** current-stack inventory (1A–1I) and explicit missing list before steps. | MASTER:L159-L161 | P3K-E003 |
| SD-3K-002 | **ADD** chunk metadata fields, Tantivy schema extension, format-version reindex migration, and 1000-token hard cap check. | MASTER:L162 | P3K-E005–P3K-E007 |
| SD-3K-003 | **ADD** exact HyDE sidecar prompt/max_tokens/temperature, 3s timeout, 5-minute cache, and rerank-against-original adversarial mitigation. | MASTER:L164 | P3K-E010–P3K-E011 |
| SD-3K-004 | **ADD** per-tier latency budgets FAST<20ms STANDARD<200ms DEEP<5s and AUTO word-count rules. | MASTER:L165 | P3K-E012–P3K-E013 |
| SD-3K-005 | **ADD** A-RAG Score_kw formula, 3× oversample MAX grouping, progressive disclosure ≥50% tokens, and length-domination adversarial. | MASTER:L167 | P3K-E015–P3K-E018 |
| SD-3K-006 | **ADD** epsilon/beta formulas, entity-extraction 0-entity adversarial, beta dominance adversarial, and post-improvement tokens ≥30% reduction criterion. | MASTER:L168-L170 | P3K-E019–P3K-E022 |
| SD-3K-007 | **QUALIFY** paper list and independence from 4D web research. | MASTER:L159-L170 | P3K-E015 |

### 4A deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-4A-001 | **ADD** per-type frame size caps, postcard version framing details, and exactly-15 const_assert. | MASTER:L380-L381 | P4A-E013–P4A-E015 |
| SD-4A-002 | **ADD** full listen-before-spawn sequence, UUID v4 agent_id, and version-handshake kill. | MASTER:L385 | P4A-E019–P4A-E020 |
| SD-4A-003 | **ADD** complete functional isolation test battery including env sanitization, /proc/sys masks, RLIMIT behaviors, concurrent agent isolation, and role-method grounding matrix. | MASTER:L392 | P4A-E021–P4A-E025 |
| SD-4A-004 | **ADD** full adversarial IPC/replan/self-mod/crash/Windows test inventory and WARN/BLOCK coverage requirement. | MASTER:L389 | P4A-E026–P4A-E031 |
| SD-4A-005 | **ADD** Linux clone flags, rlimits defaults, heartbeat 5s/15s, and Windows Job object numbers. | MASTER:L382-L388 | P4A-E032–P4A-E034 |
| SD-4A-006 | **ADD** phase_complete SSB fields and RuntimeClient wiring exit rule. | MASTER:L391 | P4A-E018, P4A-E038 |
| SD-4A-007 | **ADD** separation-cost planning numbers (0.1ms/50ms/20MB) vs SLO targets. | MASTER:L392 | P4A-E036–P4A-E037, P4A-A001 |
| SD-4A-008 | **QUALIFY** feasibility GRC 3/600 stub-write/bash timeouts appear in MASTER dispatch bullet but are not restated as 4A step numbers in phase-4A.txt (they live in later 4F); do not invent them as 4A.x steps. | MASTER:L390 | P4A-E017 |

### 4H deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-4H-001 | **ADD** revision history 2026-04-15 audit resolution (4 BLOCK/8 WARN), paper refs DGM/DarwinNet/SWE-World, and code-vs-config safety divergence. | MASTER:L480-L482 | P4H-E001–P4H-E003, P4H-C001 |
| SD-4H-002 | **ADD** complete PipelineOutcome field set including ETC/staleness/truncation/miss vectors and CatchLayer enum. | MASTER:L486-L488 | P4H-E007–P4H-E010 |
| SD-4H-003 | **ADD** full AllowedConfigKey Parameter/DataTable/LOCKED lists including cloud spend locks and Freshness* keys. | MASTER:L493 | P4H-E015–P4H-E017 |
| SD-4H-004 | **ADD** DataTableValidation FPR defaults, ProjectScope, ReasoningTraceRef, ProposedChange variants, and StructuralChange type set. | MASTER:L479-L493 | P4H-E018–P4H-E026 |
| SD-4H-005 | **ADD** ouroboros OSC/REG/EVO algorithms, SSB action types, exploration boost, key freeze, and MSI force-STABILIZING on Withhold. | MASTER:L480-L494 | P4H-E022–P4H-E025, P4H-E040–P4H-E042, P4H-E045 |
| SD-4H-006 | **ADD** staged-eval archive handling differences Stage1 discard vs Stage2/3 retain, trigger_task.toml schema, and mode-aware evaluation. | MASTER:L484 | P4H-E034–P4H-E036 |
| SD-4H-007 | **ADD** Layer1A four growth paths with numeric thresholds (3 catches, FPR 0.05, domain 5+, verified 0.8) and no-staged-eval path. | MASTER:L483 | P4H-E031–P4H-E033 |
| SD-4H-008 | **ADD** MSI frequency schedules (1/3 and 1/5 pipelines), per-mode pause, anti-fragility N=10 recovery bound, and exponential weight series. | MASTER:L491-L492 | P4H-E039 |
| SD-4H-009 | **ADD** anti-staleness `elai dep-audit`, reasoning fix-template loop, token-efficiency slot proposals, and optional FM cross-transfer. | MASTER:L494 | P4H-E043–P4H-E046 |
| SD-4H-010 | **ADD** conditional learned-reward EVALUATE section with domain-mismatch warnings and R53. | MASTER:L480-L494 | P4H-E047 |
| SD-4H-011 | **ADD** four-layer enforcement detail, lock-list unreadability, objective-hacking trigger, and FPR threshold lock. | MASTER:L494 | P4H-E027, P4H-E038 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **5/5 files**, **2,791/2,791 logical lines**, **174,259/174,259 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **238 unique atoms** = **221 explicit**, **005 inferred**, **011 ambiguous**, **001 conflicting/corrective**.
- Per-file atom counts: 1H **035**; 3I **036**; 3K **035**; 4A **054**; 4H **078**. Sum **238**.
- Relational interaction records: **22** (`IX-001`–`IX-022`). Summary-delta records: **038** (`SD-*`). Ambiguity/conflict register: **10** (`AC-001`–`AC-010`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `P1H/P3I/P3K/P4A/P4H-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the five exact DEEP-PHASE-006 manifest rows. The only orientation artifact consulted was `01-phase-index-MASTER.discovery.md`.
- Output metrics after finalization: `output_lines=0430`; `output_bytes=068574`.
- Canonical normalized output SHA-256: 39ed124bd678e2bc480fffaffe61e95dfa13cdaf7428607b9655743ac5463e97.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
