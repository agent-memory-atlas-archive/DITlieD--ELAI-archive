# DEEP-PHASE-005 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1383-L1392`, heading `### DEEP-PHASE-005`; the next chunk starts at L1394.
- Declared chunk: family PHASE; **4 files**, **2,735 logical lines**, **169,614 bytes**, all four classified `numbered/index original`; target status **within 1,800–2,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all four listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `01-phase-index-MASTER.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `1F:Lx-Ly`, `2A:Lx-Ly`, `3F:Lx-Ly`, and `3J:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 332 | 22,187 | 9420ec0aae677f6d990efa9a20c6c3974901a801ec9ab82a04598148883053b9 | docs/specs/guide/steps/phase-1F.txt | exact |
| numbered/index original | 382 | 24,128 | a536492e24d32c30904c7009542f0e5d55405a3f04c0481eb90f648c58fd8f0d | docs/specs/guide/steps/phase-2A.txt | exact |
| numbered/index original | 448 | 27,252 | 60b6a091ca87767dd3849eda51bc591a3c8bd00b5ab59e76544a0dca45559e99 | docs/specs/guide/steps/phase-3F.txt | exact |
| numbered/index original | 1,573 | 96,047 | 8b3ffcf2840b74cb88e1dddf02d9f7cea8bbfcac56cff662730d5f16f0a863a8 | docs/specs/guide/steps/phase-3J.txt | exact |
| **Aggregate** | **2,735** | **169,614** | — | **4 exact paths** | **exact** |

## Per-file atomic ledgers

### 1F — speculative decoding evaluation

Source: `docs/specs/guide/steps/phase-1F.txt` (332/332 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P1F-E001 | E/STATUS | Phase 1F is SPECULATIVE DECODING EVALUATION: an evaluation phase whose output is a measured ADOPT/DROP decision, not a feature ship. Status stamp 2026-04-07. | 1F:L1-L14, L24-L27 |
| P1F-E002 | E/STATUS | Steps 1F.1+1F.2 shipped: SpeculativeConfig, OllamaConfig nesting, ProbeResult, probe_speculative_support in ollama.rs with unit+live tests. | 1F:L3-L6 |
| P1F-E003 | E/EVIDENCE | Live probe on ollama v0.20.2 returned NOT_AVAILABLE: PR #8134 closed never-merged; draft_model silently ignored; accepted_tokens always null. | 1F:L6-L8, L56-L59 |
| P1F-E004 | E/DEFER | Steps 1F.3 / 1F.3b / 1F.4 / 1F.5 are DEFERRED-BY-DESIGN to Phase 2+ (vLLM provider); not missing work. | 1F:L8-L10, L58-L60 |
| P1F-E005 | E/OBS | Per-step TOMLs (correctness/throughput/vram/prompt_lookup) intentionally not produced; consolidated `.elai_cc/metrics/phase_1F.toml` records decision=UNAVAILABLE with reason. | 1F:L10-L12 |
| P1F-E006 | E/STATUS | EXIT CRITERIA "Decision made and documented: ADOPT or DROP" is satisfied via DROP path. | 1F:L13-L14, L309-L310 |
| P1F-E007 | E/DEP | Prerequisite corrected to Phase 1C complete (vector search + VRAM accounting); dependency graph L141 shows 1F branches from 1C; original "Phase 0 independent of 1A-1E" was wrong. | 1F:L16-L20, L100-L101 |
| P1F-E008 | E/CAP | Goal: evaluate whether speculative decoding improves throughput enough to justify VRAM cost; adopt if >1.3x throughput on target hardware, drop if less. | 1F:L24-L26, L321-L322 |
| P1F-E009 | E/DEP | Mandatory pre-reads: providers.txt L87-97, external-analysis EAGLE-3/TIDE, architecture-overview §7 HARDWARE TARGET L233-252. | 1F:L29-L32 |
| P1F-E010 | E/LIMIT | R4 VRAM: Gemma 4 E2B ~7.2GB Q4_K_M leaves ~4.8GB headroom on RTX 3060 12GB (not ~10GB as providers.txt L47 stated). Scenario A ~2GB draft → ~2.8GB free; B ~3GB delta → ~1.8GB free. | 1F:L34-L41, L76-L78 |
| P1F-E011 | E/OBS | Monitor VRAM via `nvidia-smi --query-gpu=memory.used` with polling during benchmark, not static snapshots. | 1F:L40-L41, L287-L289, L293-L295 |
| P1F-E012 | E/FAIL | R-NEW-1: Ollama speculative support unconfirmed; fallback (a) llama.cpp speculative only in Phase 1 (vLLM Phase 2+), (b) defer until ollama merges; else DROP/defer Phase 2 re-eval. | 1F:L42-L47 |
| P1F-E013 | E/LIMIT | R-NEW-2: EAGLE-3 published 2x-6.5x is for large (70B+) models; gains on 2.3B consumer GPU will be substantially lower; 1.3x threshold accounts for this. | 1F:L48-L50, L328-L329 |
| P1F-E014 | E/LIMIT | R-NEW-3: Gemma 4 EAGLE-3 draft may not exist (vLLM issue #38893); if missing, use same-family smaller draft, not EAGLE-3 head. Step 1F.2 verifies availability. | 1F:L51-L55 |
| P1F-E015 | E/STATUS | RESOLVED 2026-04-07: DROP speculative decoding for Phase 1; Prompt Lookup Decoding remains viable zero-VRAM alternative; revisit Phase 2+ with vLLM. | 1F:L56-L64 |
| P1F-E016 | E/CAP | OllamaProvider + LlmProvider trait abstraction from 1F.1 means vLLM integration is a new provider impl, not a rewrite. | 1F:L63-L64 |
| P1F-E017 | E/EVIDENCE | Grounding: EAGLE-3 corrected to 2x-6.5x (arxiv 2503.01840); TIDE 1.15x over static verified (arxiv 2602.05145); "Ollama supports natively" UNVERIFIED until probe; >1.3x threshold UNGROUNDED design decision. | 1F:L71-L85 |
| P1F-E018 | E/CFG | Draft model size guidance from llama.cpp #10466: smaller drafts often better net speedup; prefer 0.5B-1.5B over 2B-3B for Gemma 4 E2B target. | 1F:L86-L90, L145-L149 |
| P1F-E019 | E/CFG | num_draft_tokens default 5 is suboptimal; step 1F.4 uses parameter sweep instead of fixed k=5. | 1F:L91-L93, L114-L116, L229-L232 |
| P1F-E020 | E/OBS | Acceptance rate is a required metric; below 0.5 speculative decoding hurts throughput (early kill signal). | 1F:L94-L96, L234-L237, L324-L324 |
| P1F-E021 | E/CAP | Prompt lookup decoding added as step 1F.3b: zero-VRAM, up to 2.8x on summarization (vLLM blog); baseline before committing VRAM to draft. | 1F:L97-L99, L194-L219 |
| P1F-E022 | E/CFG | 1F.1 fields: speculative_decoding bool default false; draft_model Option<String>; num_draft_tokens u8 default 5; nested SpeculativeConfig or on OllamaConfig; serde round-trip. | 1F:L106-L117, L128-L130 |
| P1F-E023 | E/TEST | 1F.1 adversarial: serde name mismatch silent drop (round-trip test); num_draft_tokens=0 invalid (validate >=1). | 1F:L122-L126 |
| P1F-E024 | E/TEST | 1F.1 PRE_GATE: Phase 1C cargo test knowledge lib; providers cargo check; OllamaConfig exists via grep. | 1F:L107-L110 |
| P1F-E025 | E/CAP | 1F.2 probe checks ollama draft model param on /api/generate or /api/chat; outputs SUPPORTED/NOT_SUPPORTED; if neither ollama nor llama.cpp viable → DROP document. | 1F:L132-L162 |
| P1F-E026 | E/SURFACE | Decision artifact path: workflow/reasoning/phase-1F-decisions.md documents fallback and ADOPT/DROP with measurements. | 1F:L152-L153, L161-L161, L285-L285, L315-L315 |
| P1F-E027 | E/TEST | 1F.2 adversarial: silent ignore of unknown draft_model (false positive) must be disproved by timing/headers; record ollama --version. | 1F:L153-L158 |
| P1F-E028 | E/TEST | 1F.3 correctness: 10 prompts min; baseline vs speculative same prompt+seed; soft text match 10/10; token ID divergence WARN not fail; temp=0 + fixed seed; unique prompts/clear cache. | 1F:L164-L192 |
| P1F-E029 | E/DATA | Correctness metrics file `.elai_cc/metrics/phase_1F_correctness.toml`. | 1F:L179-L179, L192-L192 |
| P1F-E030 | E/CAP | 1F.3b prompt lookup: same 5 prompts as 1F.4, 3 runs each (15 measurements); ratio vs baseline; if ollama lacks support check llama.cpp --lookup-ngram-min; else UNAVAILABLE. | 1F:L194-L219 |
| P1F-E031 | E/DATA | Prompt lookup metrics: `.elai_cc/metrics/phase_1F_prompt_lookup.toml`. | 1F:L209-L209 |
| P1F-E032 | E/FLOW | Decision preference: if prompt_lookup >1.3x AND spec_decode also >1.3x, prefer prompt_lookup for zero VRAM; if both >1.3x and spec better, document both. | 1F:L217-L219, L277-L281 |
| P1F-E033 | E/TEST | 1F.4 throughput: 5 diverse prompts; k sweep {1,2,3,4,5,7}; 3 runs per prompt per k per mode; record acceptance rate; early kill if acceptance <0.5 across all k for 3+ prompts → DROP. | 1F:L221-L263 |
| P1F-E034 | E/OBS | Per run: prompt/output length, wall time, tokens/sec, GPU util, acceptance rate; mean+stddev per mode at best-k; ratio single number; stddev <20% of mean per mode independently. | 1F:L233-L245, L256-L261 |
| P1F-E035 | E/DATA | Throughput metrics: `.elai_cc/metrics/phase_1F_throughput.toml`. | 1F:L241-L241 |
| P1F-E036 | E/TEST | 1F.4 adversarial: GPU temp >85C discard + 60s cooldown; ≥2 prompts with >200 output tokens; warm-up exclude first run; timing proxy if acceptance not exposed. | 1F:L246-L255 |
| P1F-E037 | E/FLOW | 1F.5 decision gate top-to-bottom: NOT_AVAILABLE→DROP; acceptance<0.5→DROP; prompt_lookup>1.3x and spec≤lookup→DROP spec ADOPT lookup; spec>1.3x and delta_vram_mb<3000→ADOPT with best-k; else DROP; boundary ratio==1.3x or delta==3000 → DROP (conservative). | 1F:L265-L284 |
| P1F-E038 | E/CFG | If ADOPT: providers config default enables speculative decoding, best-k and draft model set; if DROP: default remains disabled. | 1F:L290-L291, L305-L306, L316-L318 |
| P1F-E039 | E/DATA | VRAM metrics: `.elai_cc/metrics/phase_1F_vram.toml` with PEAK baseline and PEAK speculative via nvidia-smi polling. | 1F:L287-L289 |
| P1F-E040 | E/LIMIT | ADOPT decision applies to Phase 0 hardware only; production re-evaluation needed in Phase 4 when hardware changes. | 1F:L296-L298 |
| P1F-E041 | E/TEST | Marginal ratio within 0.05 of 1.3x: run 5 additional prompts and recompute. | 1F:L299-L300 |
| P1F-E042 | E/TEST | Exit: Phase 0 + 1A-1C tests still pass (no regression); decision binary ADOPT or DROP (no maybe). | 1F:L301-L304, L319-L319 |
| P1F-E043 | E/OBS | Measurement targets: throughput >1.3x to adopt; acceptance >0.5; VRAM delta <3000MB; draft size 0.5B-1.5B expected optimal; TIDE only if static adopted and further opt desired. | 1F:L321-L332 |
| P1F-E044 | E/SURFACE | No dashboard UI in 1F; operator-visible surfaces are decision markdown, metrics TOMLs, and probe SUPPORTED/NOT_SUPPORTED console output. | 1F:L10-L14, L151-L152, L285-L289 |
| P1F-I001 | I/DEP | "Phase 2+" revisit wording implies any post-Phase-1 vLLM path; concrete provider step is Phase 3J, not a Phase-2 numbered phase. | 1F:L8-L10, L59-L60 |
| P1F-A001 | A/CFG | Exact draft model name/artifact for Gemma 4 family is unresolved until 1F.2 availability check. | 1F:L51-L55, L145-L149 |
| P1F-A002 | A/TEST | Soft text match algorithm (normalization, whitespace) is not fully specified beyond "identical text content". | 1F:L175-L176 |
| P1F-C001 | C/DEP | Original prerequisite "Phase 0 independent of 1A-1E" conflicts with dependency graph 1C branch; corrected to Phase 1C. | 1F:L16-L20, L100-L101 |
| P1F-C002 | C/STATUS | Opening evaluation plan describes full 1F.3-1F.5 suite, but RESOLVED block permanently skips them for Phase 1 after NOT_AVAILABLE probe. | 1F:L3-L14, L56-L60, L164-L306 |

Category coverage: capabilities/actions ✓; surfaces (CLI/metrics/decision docs) ✓; workflows/decision gates ✓; schemas/metrics data ✓; defaults/config ✓; integrations (ollama/llama.cpp/nvidia-smi) ✓; security/privacy N/A named; failures/fallback ✓; determinism (seed/temp) ✓; observability/evidence ✓; tests/done gates ✓; constraints/deferred ✓. No formal retry/idempotency API; concurrency not in scope.

### 2A — GRC state machine (basic)

Source: `docs/specs/guide/steps/phase-2A.txt` (382/382 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P2A-E001 | E/STATUS | Phase 2A: GRC STATE MACHINE (basic). Prerequisites: Phase 0 complete (contract tests pass, cargo check clean). | 2A:L1-L2 |
| P2A-E002 | E/CAP | Goal: GRC engine core — all reasoning cites grounding; budget enforces per-topic call/char limits; state machine validates transition order; rejected steps produce typed errors. | 2A:L5-L9 |
| P2A-E003 | E/LIMIT | Scope excludes grounding verification methods (2B), per-agent config (2C), CSQ (2D). | 2A:L11-L13 |
| P2A-E004 | E/DEP | Pre-reads: reasoning.txt §§1-3,10,11 items 1-3; architecture-overview §4 GrcEngine; prompt-architecture §1. | 2A:L16-L21 |
| P2A-E005 | E/STATE | D1: GrcState with allowed_next(); observe→hypothesize→plan_action→act→verify; reject invalid (e.g. hypothesize→verify). | 2A:L26-L32, L72-L76 |
| P2A-E006 | E/FAIL | D2: GrcError { UngroundedStep, BudgetExhausted, InvalidTransition, MalformedGrounding } with context; thiserror. | 2A:L34-L37, L96-L100 |
| P2A-E007 | E/CFG | D3: GrcBudget tracks remaining_calls/chars; when remaining_calls<=1 engine forces convergence (must produce conclusion). | 2A:L39-L44, L89-L93 |
| P2A-E008 | E/DATA | D4: grounding string parsed into source citations format "source_type:reference" (e.g. file:src/foo.rs:L42); 2A format-only, 2B verifies sources. | 2A:L46-L52, L273-L285 |
| P2A-E009 | E/STATE | D5: GroundingLevel VERIFIED > HIGH > UNCERTAIN (PartialOrd); UNCERTAIN flagged warn log, not rejected in 2A. | 2A:L54-L57, L94-L95, L279-L283 |
| P2A-E010 | E/CAP | 2A.1 greenfield: create crates/reasoning with Cargo.toml dep core; add to workspace. | 2A:L62-L70 |
| P2A-E011 | E/STATE | GrcState enum: Observe, Hypothesize, PlanAction, Act, Verify, Converge, Status; Converge and Status allowed from any state; first non-Status/non-Converge step must be Observe. | 2A:L72-L76, L217-L220, L269-L271 |
| P2A-E012 | E/DATA | ThinkStep { step_type, content, topic, grounding: Option<String>, conclusion: Option<String>, scope: Option<Vec<String>> }. | 2A:L77-L85 |
| P2A-E013 | E/CFG | scope: when AgentGrcConfig.require_scope_enumeration=true, Observe must populate scope; orchestrator validates every scope item appears in subsequent plan_action; unaddressed → WARN into agent context. | 2A:L81-L85, L222-L237, L364-L364 |
| P2A-E014 | E/DATA | ThinkResult { step, budget_remaining: GrcBudget, grounding_level: Option<GroundingLevel>, warnings: Vec<String> }. | 2A:L87-L88, L305-L307 |
| P2A-E015 | E/CFG | GrcBudget default max_calls=5, max_chars=800 ("moderate"); methods remaining_calls, remaining_chars, is_exhausted, must_converge (remaining_calls<=1). | 2A:L89-L93, L211-L215 |
| P2A-E016 | E/CAP | GrcEngine trait 2A methods: think, set_budget, get_budget, reset_topic; 2B stubs unimplemented: verify_claim, ground_plan; full 6-method trait for stability. | 2A:L101-L112, L142-L144 |
| P2A-E017 | E/DATA | 2B forward stubs: VerificationResult, GroundingResult with #[allow(dead_code)]. | 2A:L113-L116 |
| P2A-E018 | E/STATE | BasicGrcEngine: HashMap<String, TopicState> with budget, last_state, steps, grounding_log; default budget applied on first think if no set_budget. | 2A:L117-L120, L300-L303 |
| P2A-E019 | E/DEP | 2A.2: reasoning depends only on core/, thiserror, serde — no providers/, tools/, knowledge/; pure logic, no LLM calls. | 2A:L146-L164 |
| P2A-E020 | E/TEST | 2A.3 TDD: 12 failing tests T1-T12 covering ungrounded observe, budget overflow/forced converge (max_calls=3), per-topic isolation, invalid transition, must_converge, malformed grounding, free Status, default budget, first-step Observe, scope present/missing/optional. | 2A:L166-L255 |
| P2A-E021 | E/TEST | T2 uses max_calls=3 not 2 so must_converge at remaining<=1 does not mask overflow test. | 2A:L179-L189 |
| P2A-E022 | E/TEST | T7 Status is free: does not consume budget (calls_used stays 0). | 2A:L207-L209, L362-L362 |
| P2A-E023 | E/TEST | Stubs must use todo!/unimplemented!, not Ok placeholders; zero timing-dependent tests. | 2A:L247-L254 |
| P2A-E024 | E/FLOW | I1 transitions: last_state.allowed_next(); first step Observe; Status does not update last_state; Converge allowed any state. | 2A:L265-L271 |
| P2A-E025 | E/DATA | I2 grounding required on Observe, Hypothesize, Verify; optional on PlanAction/Act; none on Status. Parse comma-separated type:reference. Valid types: file, grep, test, search, tool, training. | 2A:L273-L285 |
| P2A-E026 | E/DATA | GroundingLevel map: test/search→Verified; file/grep/tool→High; training→Uncertain; warn on Uncertain, do not reject. | 2A:L279-L283 |
| P2A-E027 | E/FLOW | I3 budget check-then-increment: exhaust calls, chars overflow, or must_converge without Converge → BudgetExhausted before increment; then saturating_add. | 2A:L287-L298 |
| P2A-E028 | E/STATE | I4 reset_topic clears all state for topic; entry fully removed from map (not zeroed). | 2A:L300-L303, L322-L323 |
| P2A-E029 | E/TEST | 2A.4 acceptance: cargo test -p reasoning 0 failures ≥12; no todo!/unimplemented!/unwrap() in engine.rs. | 2A:L309-L327 |
| P2A-E030 | E/OBS | 2A.5 metrics: validation_overhead_us <5000; memory per topic <1KB base + history; write `.elai_cc/metrics/phase_2A.toml`; median of 1000 iterations; no criterion without approval. | 2A:L329-L354 |
| P2A-E031 | E/TEST | Exit criteria: grounding enforced; budget overflow → converge or BudgetExhausted; InvalidTransition; MalformedGrounding; free Status; independent topics; scope WARN when required; 12+ tests. | 2A:L357-L365 |
| P2A-E032 | E/EVIDENCE | Budget range grounding: max_calls 1-12, max_chars 200-1500 from reasoning.txt §3; defaults are moderate 5/800. | 2A:L369-L370, L93-L93 |
| P2A-E033 | E/SURFACE | No dashboard panel in 2A; operator/engine surface is GrcEngine API and typed errors; Status free-echo aligns with mcp_think(step_type="status"). | 2A:L207-L209, L373-L373 |
| P2A-I001 | I/INT | AgentGrcConfig and orchestrator scope validation are named but config crate/module location is outside 2A skeleton (types only reference require_scope_enumeration). | 2A:L81-L85, L222-L237 |
| P2A-A001 | A/STATE | Exact allowed_next() table for every state (beyond Observe→not Verify and Verify terminal note) is not fully enumerated in the phase file. | 2A:L72-L76, L136-L138, L317-L319 |
| P2A-A002 | A/TEST | T11 says "Err or ThinkResult with warning" for missing scope — hard fail vs soft warn unresolved. | 2A:L229-L232 |
| P2A-A003 | A/FAIL | InvalidTransition "from=None indication" for first non-Observe step lacks a dedicated error variant shape. | 2A:L217-L220 |
| P2A-C001 | C/DATA | Header says "Grounding required on every step" in grounding summary item while body requires grounding only on Observe/Hypothesize/Verify (PlanAction/Act optional, Status free). | 2A:L273-L285, L358-L362, L371-L371 |

Category coverage: capabilities ✓; surfaces (API/errors, no UI) ✓; workflows/states ✓; schemas/types ✓; defaults/config ✓; integrations (trait stubs for 2B) ✓; failures/typed errors ✓; determinism (no timing tests) ✓; observability/metrics ✓; tests/TDD ✓; constraints/deferred ✓. Concurrency/retry/idempotency not specified. Privacy/SEC N/A.

### 3F — rules engine (self-improving, three-tier enforcement)

Source: `docs/specs/guide/steps/phase-3F.txt` (448/448 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3F-E001 | E/STATUS | Phase 3F: RULES ENGINE. Prerequisites: Phase 3D complete (parallel with 3E); implicit Phase 1A tree-sitter for AST_PATTERN. | 3F:L1-L3 |
| P3F-E002 | E/CAP | Goal: deterministic rule enforcement replacing probabilistic prompts; Predicate DSL with 5 types; self-discovery from auditor violations; rules DATA in SQLite evaluated by Rust; three-tier enforcement (Tier1 code, Tier2 constrained decoding, Tier3 prompt+auditor). | 3F:L7-L12 |
| P3F-E003 | E/DEP | Pre-reads: rules.txt full; prompt-architecture §§1-2,4.3,5,6; risk-register R25. | 3F:L14-L22 |
| P3F-E004 | E/SEC | R25 LLM ignores domain rules — this phase is the mitigation. | 3F:L24-L25 |
| P3F-E005 | E/FAIL | Malformed self-discovered predicates mitigated by compile-time validation before DB insert (3F.7). | 3F:L26-L27, L271-L272 |
| P3F-E006 | E/INT | tree-sitter: rules/ must NOT add separate dep; knowledge exposes AstEvaluator facade; recommended trait Box<dyn AstEvaluator> injected at construction (option b preserves dep graph). | 3F:L28-L42 |
| P3F-E007 | E/FLOW | Evaluation order cheapest-first short-circuit on BLOCK: METRIC_GATE → GREP_PATTERN → AST_PATTERN → COMPOSITE; within type severity DESC (BLOCK before WARN). | 3F:L44-L55, L133-L134 |
| P3F-E008 | E/FLOW | WARN collection: WARNs before a BLOCK preserved with BLOCK as context. | 3F:L52-L53 |
| P3F-E009 | E/DEP | Dev deps: proptest, criterion; runtime jsonschema; tree-sitter via knowledge AstEvaluator. | 3F:L57-L61 |
| P3F-E010 | E/DATA | 3F.1 tables: rules, rule_metrics, contract_tests, violation_patterns; schema_version migration; columns per rules.txt §2 including phase_mask, severity, mutable, source; scoring score_clarity/frequency/precision; counterfactual causal_catches/spurious_catches. | 3F:L66-L93 |
| P3F-E011 | E/TEST | CHECK constraints reject invalid tier/status/source; NULL predicate INSERT fails; migration idempotent preserves v1 data. | 3F:L81-L93 |
| P3F-E012 | E/CAP | 3F.2 five predicate types: AST_PATTERN, GREP_PATTERN, SCHEMA_REQUIRED, METRIC_GATE, COMPOSITE. | 3F:L95-L100, L119-L119 |
| P3F-E013 | E/STATE | Verdict: ALLOW, WARN, BLOCK, ESCALATE(EscalationTarget); EscalationTarget: Auditor, Human, NextTierRule(RuleId). | 3F:L101-L102, L120-L121 |
| P3F-E014 | E/DATA | Predicate serde to/from TEXT (JSON or postcard); round-trip for all types. | 3F:L103-L103, L109-L109 |
| P3F-E015 | E/CAP | AstEvaluator: evaluate_query(language, source, query)→Vec<AstMatch>; validate_query→Result. | 3F:L104-L106 |
| P3F-E016 | E/FAIL | COMPOSITE circular graph returns error within 1ms; serde version drift explicit error or defaults. | 3F:L112-L117 |
| P3F-E017 | E/CAP | 3F.3 RuleEngine trait: evaluate(action, context)→ALLOW\|BLOCK\|WARN; get_active_rules(fsm_state, file_context); compile_predicate at insertion (regex, tree-sitter, jsonschema). Object-safe dyn. | 3F:L123-L148 |
| P3F-E018 | E/STATE | 3F.4 lifecycle: PROPOSED → CANDIDATE → SHADOW → ACTIVE → RETIRED. | 3F:L150-L155, L189-L189 |
| P3F-E019 | E/FLOW | Guards: PROPOSED→CANDIDATE ≥3 contract tests all pass; CANDIDATE→SHADOW human OR independent second-auditor (finding-source auditor cannot approve); SHADOW→ACTIVE FPR<5% AND TPR>80% AND counterfactual_causal_rate>50% AND no safety regressions; ACTIVE→RETIRED FPR>30% over 10 pipelines OR superseded OR human. | 3F:L155-L162 |
| P3F-E020 | E/SEC | mutable=0 rejects auto-retirement; mutable=0 exempt from causal rate gate (human-promotable only). | 3F:L163-L164, L177-L177, L191-L191 |
| P3F-E021 | E/OBS | Every transition logged with timestamp, metrics snapshot, reason. | 3F:L165-L165, L178-L178 |
| P3F-E022 | E/FAIL | Division-by-zero safe rates: 0 shadow data → "insufficient shadow data" error not panic; RETIRED kept in DB not deleted. | 3F:L179-L184, L192-L192 |
| P3F-E023 | E/SEC | PROPOSED source=AUDITOR cannot auto-promote without human/second-auditor. | 3F:L175-L187, L276-L277 |
| P3F-E024 | E/CAP | 3F.5 interpreter: AST via AstEvaluator; GREP compile-once regex; SCHEMA_REQUIRED jsonschema; METRIC_GATE lt/gt/eq/between; COMPOSITE AND/OR/NOT max depth 8. | 3F:L194-L229 |
| P3F-E025 | E/DATA | Evaluation returns Verdict { rule_id, result, evidence: String }. | 3F:L205-L205 |
| P3F-E026 | E/FLOW | ESCALATE routing: Auditor for SHADOW rules; Human for mutable=0 promotion; NextTierRule for tier chain. | 3F:L206-L207, L215-L215 |
| P3F-E027 | E/DET | Same code + same predicate = same verdict over 100 repetitions. | 3F:L216-L216, L229-L229, L408-L408 |
| P3F-E028 | E/FAIL | COMPOSITE depth 100 → Err(DepthExceeded); ReDoS patterns complete within timeout (regex linear-time). | 3F:L217-L222 |
| P3F-E029 | E/CFG | 3F.6 phase_mask bitmask + domain detection; get_active_rules status=ACTIVE AND phase_mask AND domain; SQL includes OR domain IN ('engineering','security'); elai_cc.toml [rules.domain_mapping]. | 3F:L231-L242 |
| P3F-E030 | E/CFG | Engineering and security domains always active (cross-cutting, security mutable=0). | 3F:L239-L242, L247-L248, L257-L258 |
| P3F-E031 | E/TEST | Domain detection covers 5 domains + engineering + security; case-insensitive path matching; phase_mask=0 never active. | 3F:L243-L258 |
| P3F-E032 | E/CAP | 3F.7 self-discovery on orchestrator side: normalize, blake2s hash, count, auto-propose at occurrences≥3; generate AST then regex then Tier3-only; compile_predicate before insert; invalid logged discarded. | 3F:L260-L298 |
| P3F-E033 | E/TEST | Auto-contract tests: each example_code → BLOCK test; orchestrator generates ALLOW counterexamples (not auditor). | 3F:L273-L275, L285-L285 |
| P3F-E034 | E/OBS | Violation scoring 3-axis composite clarity/frequency/precision in [0,15]. | 3F:L278-L278, L298-L298 |
| P3F-E035 | E/SEC | Normalization runs in orchestrator process, not auditor (circular trust). | 3F:L265-L266, L292-L293 |
| P3F-E036 | E/DATA | 3F.8 seed: rules_seed.db ~33 highest-value rules; status=ACTIVE source=SEED skip lifecycle; merge global+project rule_id override; mutable=0 seeds cannot be overridden by mutable=1 project rules. | 3F:L300-L337 |
| P3F-E037 | E/SURFACE | Post-tool completeness GREP rules (Tier1, engineering, PostToolUse): web_fetch <img> → WARN "N images not extracted"; <table> → WARN "N tables not extracted"; file_read TODO/FIXME → INFO "N TODO/FIXME items"; <1ms, no pre-exec I/O. | 3F:L309-L314 |
| P3F-E038 | E/TEST | Every seed Tier1 rule ≥3 contract tests; Tier2 ≥2; absent seed DB explicit error with path. | 3F:L322-L328 |
| P3F-E039 | E/TEST | 3F.9 ≥254 static contract tests; proptest 256 cases/rule; generators violation/valid/mutation_invariant/fix_mutation; COMPOSITE prop_recursive depth 8 size 64; De Morgan + associativity; shrunk counterexamples → contract_tests source=GENERATED; ~1400 proptest lines / 7 domains. | 3F:L339-L371 |
| P3F-E040 | E/SEC | contract_tests READ-LOCK during self-modification (INSERT rejected when flag true). | 3F:L364-L371 |
| P3F-E041 | E/OBS | 3F.10 criterion benchmarks: GREP median <1ms; METRIC <0.1ms; AST <5ms target; 10 mixed <10ms; store `.elai_cc/metrics/phase_3F.toml`. | 3F:L373-L397 |
| P3F-E042 | E/DEFER | Counterfactual shadow RUNTIME integration deferred to orchestrator+auditor (3C+3A) and 3I DST model; 3F only models rates/guards. | 3F:L399-L405 |
| P3F-E043 | E/TEST | Exit: determinism 100x; lifecycle all guards; self-discovery ≥3; ≥254+proptest; GREP/METRIC latency; auditor trust boundary. | 3F:L407-L414 |
| P3F-E044 | E/EVIDENCE | 254 arithmetic verified from rules.txt §9A; COMPOSITE depth=8 UNGROUNDED defensive (not in rules.txt); policy_engine.rs/permission_enforcer.rs LOC verified present. | 3F:L422-L448 |
| P3F-E045 | E/SURFACE | No dedicated rules dashboard panel in 3F phase file; enforcement surfaces are evaluate verdicts, seed DB, PostToolUse WARN/INFO strings, and transition logs. | 3F:L309-L314, L165-L165 |
| P3F-I001 | I/INT | SCHEMA_REQUIRED may interact with provider-tier escalation only if rules.txt says so; phase-3F body defines schema validation, not provider-tier escalation. | 3F:L99-L100, L202-L202 |
| P3F-A001 | A/DATA | Predicate TEXT encoding choice JSON vs postcard left open. | 3F:L103-L103 |
| P3F-A002 | A/CFG | Exact five business domains for detection listed only as "all 5 domains + engineering + security" without full enum in this file. | 3F:L256-L258 |
| P3F-A003 | A/FLOW | Verdict trait evaluate signature shows ALLOW\|BLOCK\|WARN while enum also has ESCALATE — escalation path vs evaluate return shape underspecified. | 3F:L128-L129, L101-L102, L206-L207 |
| P3F-C001 | C/DEP | Crate map says rules depends on core only, while architecture decision requires knowledge AstEvaluator or knowledge dep — resolved by recommended trait injection but dual wording remains. | 3F:L31-L41 |

Category coverage: capabilities/predicates ✓; surfaces (PostToolUse messages, errors) ✓; lifecycle states ✓; schema/data ✓; config/domain mapping ✓; integrations (knowledge AstEvaluator, orchestrator discovery) ✓; security/trust ✓; failures/depth/ReDoS ✓; determinism ✓; observability/metrics ✓; tests/proptest ✓; deferred counterfactual runtime ✓. Privacy formal policy N/A; retry only via compile discard.

### 3J — vLLM + advanced providers + plugins + orchestrator hardening

Source: `docs/specs/guide/steps/phase-3J.txt` (1573/1573 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3J-E001 | E/STATUS | Phase 3J: vLLM integration + advanced providers + plugins + orchestrator hardening. Prerequisites: Phase 3I complete; Phase 1F speculative eval. | 3J:L1-L2 |
| P3J-E002 | E/CAP | Goal: vLLM production backend; layer opts DoLa/TurboQuant/steering; ModelPool GPU multiplex/preload/profiles; MCP bridge; tiered static hooks; AppContainer; advanced SSB types; MILP eval. | 3J:L6-L11 |
| P3J-E003 | E/CFG | Primary test model Gemma4 E2B on RTX 3060 12GB; final 3J.14/15 apply to Gemma4 31B via CPU offload on 64GB RAM (~0.5-2 tok/s); EAGLE-3 draft training not feasible on CPU. | 3J:L13-L30, L99-L101 |
| P3J-E004 | E/EVIDENCE | Installed 2026-04-08: gemma4:31b 19GB Q4 262144 ctx vision+tools+thinking; gemma4:e2b 7.2GB; gemma3:4b fallback; no separate VLM needed. | 3J:L32-L41 |
| P3J-E005 | E/CFG | vLLM 0.19.0 in WSL2 ~/vllm-env/ PyTorch 2.10+cu128; serve example port 8000; Rust on Windows connects via HTTP localhost auto-forward. | 3J:L42-L51 |
| P3J-E006 | E/DEP | Pre-reads: providers.txt vLLM/LayerConfig/ModelPool; orchestrator SSB/AppContainer/self-mod; plugins MCP+hooks; paper summaries DoLa/TurboQuant/EAGLE-3/TIDE/ActAdd/Murakkab. | 3J:L54-L64 |
| P3J-E007 | E/DEP | Phase is prerequisite for 4A isolation and 4G autonomous mode (placement/optimization must stabilize first). | 3J:L67-L69 |
| P3J-E008 | E/FAIL | Risks R1-R11: HTTP latency, DoLa layer access, TurboQuant deep integration, ActAdd coupled, MILP Windows build, MCP 1751 LOC, AppContainer weaker-than-Linux, VRAM overcommit, small-model transfer, 31B CPU speed, CPU hook availability. | 3J:L72-L105 |
| P3J-E009 | E/CAP | 3J.1 VllmProvider: /v1/completions, /v1/models, SSE (reuse sse.rs), health GET /health, LlmProvider complete/embed/token_count; cfg feature "vllm"; latency 100 req p50/p95/p99 in vllm_spike.toml; timeout 5s on dead server. | 3J:L110-L141 |
| P3J-E010 | E/CAP | 3J.2 InferenceBackend::{Ollama(OllamaConfig), Vllm(VllmConfig)}; VllmConfig server_url, tensor_parallelism, served_model_names; ModelPool does not load/unload under Vllm (vLLM scheduler); POST/DELETE load/unload model APIs. | 3J:L143-L174 |
| P3J-E011 | E/CFG | elai_cc.toml [providers.backend] = "ollama"\|"vllm"; fallback to Ollama emits WARNING including "layer-level optimizations unavailable". | 3J:L158-L169 |
| P3J-E012 | E/CAP | 3J.3 DolaDecoder: JSD premature layer, alpha=0.1, theta=1.2 repetition; model-specific candidate buckets; DolaConfig enabled/alpha/buckets/repetition_penalty. | 3J:L176-L199 |
| P3J-E013 | E/CFG | Per-role DoLa: Executor on, Auditor amplified, Prompt-engineer off; elai_cc.toml [agents.{role}.layer_config.dola]. | 3J:L200-L204, L234-L234 |
| P3J-E014 | E/LIMIT | Guard: model_params <1B auto-disable DoLa with WARNING; 1-7B EVALUATE empirically; degraded top-N logprobs path if full logits unavailable. | 3J:L211-L219, L226-L227 |
| P3J-E015 | E/TEST | E2B: 20 factuality prompts baseline vs DoLa; latency budget ~5-8% (accept within 10%). | 3J:L204-L209, L236-L237 |
| P3J-E016 | E/DEFER | 3J.4 TurboQuant FEASIBLE/DEFER decision; QR rotation + Lloyd-Max; b-bit codebooks; d>=64; unit-norm + orthogonal Pi invariants; decision file phase-3J-turboquant.md. | 3J:L239-L291 |
| P3J-E017 | E/CAP | 3J.5 SteeringVectorEngine ActAdd: h_A^l = h+^l − h-^l; SteeringVector {layer, vector, coefficient}; config vectors/enabled; elai_cc.toml [agents.{role}.layer_config.steering]; mid-layer default model_layers/3; c in [3,20] start 5. | 3J:L293-L350 |
| P3J-E018 | E/DEP | Steering coupled to DoLa layer hook: if DoLa deferred, steering deferred. Local models only (no API models). Composable multi-vector. | 3J:L323-L329, L350-L350 |
| P3J-E019 | E/DATA | Store vectors .safetensors or .npz; E2B ≥2 contrast pairs skepticism+completeness; 5 prompts/vector eval. | 3J:L330-L334 |
| P3J-E020 | E/CAP | 3J.6 MultiplexingPolicy: safety_margin_bytes default 1GB; max_colocated 3; min_model_size_for_dedicated_gpu 4GB; ModelPool.can_colocate; Phase 0-1 single GPU no multiplex. | 3J:L352-L369, L401-L404 |
| P3J-E021 | E/FLOW | Predictive preload on FSM entry: EXECUTING→auditor; AUDITING→executor; PLANNING→prompt-engineer; TESTING_FIRST→executor; tokio::spawn background; blocked if VRAM insufficient. | 3J:L370-L378 |
| P3J-E022 | E/CFG | PreloadPolicy map FsmState→Vec<ModelId> in [providers.preload]; SloTier BEST/GOOD/FAIR/BASIC per [agents.{role}.slo_tier]; tier mismatch WARNING. | 3J:L377-L386 |
| P3J-E023 | E/EVENT | 3J.7 WorkloadProfile SSB action_type='workload_profile' at phase_complete: role, phase_id, tokens_in/out, latency_ms, vram_peak_mb, model_id, gpu_id, timestamp ClockProvider; budget <1ms. | 3J:L406-L442 |
| P3J-E024 | E/CAP | 3J.8 MILP eval vs greedy: good_lp+HiGHS preferred; ADOPT if >10% swap latency improvement; DROP if heuristic <5% worse; EVALUATE if 5-10%; always implement GreedyModelPlacer; decision phase-3J-milp.md; solve budget <50ms; infeasible→greedy WARNING. | 3J:L444-L487 |
| P3J-E025 | E/CAP | 3J.9 McpBridge: connect/list_tools/call_tool/disconnect; transport stdio/SSE/streamable HTTP; feature mcp; port claw-code 912+839 LOC. | 3J:L489-L533 |
| P3J-E026 | E/RECOVERY | MCP retry 3 attempts 1s/2s/4s; heartbeat timeout 30s; tool call timeout 30s configurable; graceful degradation explicit errors to agent. | 3J:L504-L509, L525-L526 |
| P3J-E027 | E/SEC | MCP untrusted: sanitize results; treat as data not instructions; version handshake reject incompatible; agent never connects to MCP directly (orchestrator proxy). | 3J:L509-L528 |
| P3J-E028 | E/CAP | 3J.10 T1 RULES <1ms every SSB write; T2 AST <10ms on file_write only, circuit break files>5000 lines; T3 PATTERN <5ms only when WARN and no BLOCK; no LLM hot path. | 3J:L535-L567 |
| P3J-E029 | E/SEC | BLOCK/WARN → SSB finding; orchestrator blake2s normalizes for violation_patterns; agent sees sanitized "Tool call rejected by policy" only (no rule ID/tier/pattern). | 3J:L556-L563, L582-L584 |
| P3J-E030 | E/SEC | 3J.11 AppContainerConfig profile elai_cc_agent_{id}; granted_paths; network_allowed false except researcher; Job limits ACTIVE_PROCESS=1, PROCESS_MEMORY=512MB, JOB_TIME; env only LANG/TERM/PATH/TMPDIR. | 3J:L586-L614 |
| P3J-E031 | E/SEC | Self-mod locks: IPC realpath Write/Edit reject; FS not granted; GitCommit filter on locked paths; fallback restricted token documented weaker (junctions). | 3J:L605-L611, L624-L625 |
| P3J-E032 | E/CAP | AgentSandbox trait create/launch_agent/cleanup; Windows AppContainer cfg; Linux clone+seccomp Phase 4A. | 3J:L615-L617 |
| P3J-E033 | E/EVENT | 3J.12 adds typed SSB actions: model_swap, knowledge_degradation, isolation_metric, plan_update, replan_request, stub_reference, reasoning_trace, memory_monitoring, phase_complete, workload_profile; total variants ≥17; write-time type validation. | 3J:L637-L681 |
| P3J-E034 | E/OBS | isolation_metric: context_savings_from_isolation = sum(agent_grc_tokens)/model.max_context at phase transition. | 3J:L663-L665 |
| P3J-E035 | E/CAP | 3J.PIPELINE InferenceDispatcher trait; ApiDispatcher + LocalDispatcher; [backend] mode local\|api\|hybrid\|auto; role_override map. | 3J:L683-L712 |
| P3J-E036 | E/FLOW | FSM tool filtering: PLANNING read-only; EXECUTING full; AUDITING read-only+test; local max 15 tools truncate descriptions; API all tools. | 3J:L714-L721 |
| P3J-E037 | E/CFG | ContextBudget: local 80% of 131K; API 200K; slots system/tools/messages. | 3J:L723-L730 |
| P3J-E038 | E/SURFACE | Slash /model and /mode switch backend at runtime; elai --local uses orchestrator pipeline; default elai uses API same pipeline. | 3J:L737-L737, L765-L767 |
| P3J-E039 | E/FLOW | Auto-start sidecar when mode local/auto; health wait max 120s; kill sidecar on CLI exit; auto→API if start fails. | 3J:L739-L744 |
| P3J-E040 | E/CAP | 3J.UNIFIED single RuntimeClient: auth/backend/tool_policy/budget_policy; one stream(); max 3 backend branch points; BackendConfig max 10 fields. | 3J:L769-L848 |
| P3J-E041 | E/CFG | ToolPolicy Full \| FsmFiltered {max_tools, fsm_state_path} \| None(Idle); BudgetPolicy ApiDefault{ceiling:200K} \| Local{ceiling:131K, fraction:80%}. | 3J:L800-L807 |
| P3J-E042 | E/SURFACE | --local flag sets BackendConfig mode Local; no flag → Api from env auth; elai.bat simplified to --local + HOME fallback. | 3J:L814-L815, L862-L862 |
| P3J-E043 | E/STATUS | 3J.PIPELINE session summary 2026-04-09: dispatcher.rs, Fsm::read_persisted_state, LocalRuntimeClient, start_sidecar windows_to_wsl_path, /shutdown, real tokenizer counting, 7/7 tests, patches 0.3/0.4. | 3J:L850-L875 |
| P3J-E044 | E/CAP | 3J.WIRE.FOUNDATION: LiveCli owns Ssb, Arc<Fsm>, ClockProvider; paths sessions/{sid}/ssb.db and .fsm-state.msgpack. | 3J:L877-L907 |
| P3J-E045 | E/SURFACE | CliInteractiveGate: present_gate prints EvidenceSummary+RiskLevel; prompt Approve/Reject/Modify/Redirect; LOW risk auto-advances. | 3J:L895-L898 |
| P3J-E046 | E/EVENT | SessionTracer backed by Ssb: turn_started, tool_execution, turn_completed → SsbPayload; Ssb::shutdown on exit. | 3J:L902-L907 |
| P3J-E047 | E/CFG | 3J.WIRE.BUDGET: real fixed_slot_sizes for slots 0/1/2; max_tokens = min(model max, remaining); ApiDefault skips ContextBudget; [pipeline] telemetry logs slots. | 3J:L925-L963 |
| P3J-E048 | E/CAP | 3J.WIRE.RULES: load seed rules (~30-40 eng+security); DefaultRuleEngine on PreToolUse; BLOCK reject; WARN→Ssb; ESCALATE→InteractiveGate; AstEvaluator/DkgQuerier stubs initially; strip CLAUDE.md rules from local system prompt only. | 3J:L966-L1015 |
| P3J-E049 | E/OBS | Rule eval <5ms per tool call Tier1 target; seed rules versioned under .elai_cc/data/rules/ TOML. | 3J:L1004-L1007 |
| P3J-E050 | E/CAP | 3J.WIRE.SESSION: SessionDag sessions/{sid}/dag.db; auto-snapshot each turn; trim at 85% budget 3-pass; /resume via load_snapshot+parent_chain; /session fork create_branch max 3; recover_stubs from Ssb. | 3J:L1017-L1060 |
| P3J-E051 | E/LIMIT | SessionDag sync SQLite not WAL; separate from Ssb WAL connection; keep /compact as manual override. | 3J:L1045-L1052 |
| P3J-E052 | E/SURFACE | 3J.WIRE.PLAN: optional PlanSkeleton from .elai_cc/plans/{pipeline_id}.toml; /plan displays skeleton; /plan phase <id> shows PhaseDetail; Slot 2 phase context; contract satisfied vs pending display. | 3J:L1063-L1103 |
| P3J-E053 | E/LIMIT | Plan load optional (None ok); do not auto-create plans. | 3J:L1089-L1096 |
| P3J-E054 | E/CAP | 3J.WIRE.STUBS: TreeSitterAstEvaluator + DkgQuerierAdapter replace PassAstEvaluator/EmptyDkgQuerier; max_depth<=20; visited set on BFS; lazy DKG init; fallback EmptyDkgQuerier on init fail with warning. | 3J:L1106-L1201 |
| P3J-E055 | E/DEFER | Stub inventory defers OllamaProvider/Embedder todos, NetworkVerifier, NoopTestRunner, OrtSession.embed, VectorIndex readiness to other phases. | 3J:L1120-L1127 |
| P3J-E056 | E/OBS | 3J.13 metrics phase_3J.toml fields: latencies, dola_*, turboquant_*, milp_*, steering_*, multiplexing_enabled, ssb_action_type_count, t1/t2/t3 hook medians, compile_time_delta_sec; non-vllm build still passes. | 3J:L1204-L1244 |
| P3J-E057 | E/SURFACE | 3J.14 WIRE.E2E: elai.bat single command; primary sidecar DoLa+TurboQuant+NF4; ollama fallback; QueryPipeline retrieval; release elai.exe. | 3J:L1246-L1253 |
| P3J-E058 | E/FLOW | Ollama fallback on sidecar health fail or mid-session HTTP error; re-probe each turn; both down → clear error no crash; user sees "[degraded] using ollama fallback" prompt prefix. | 3J:L1263-L1277 |
| P3J-E059 | E/CFG | elai_cc.toml [backend] fallback_url default http://localhost:11434. | 3J:L1277-L1277 |
| P3J-E060 | E/CAP | Knowledge retrieval: BM25 index cwd; QueryPipeline BM25+CatRag+Arag; optional CrossEncoder; inject top-K into Slot 4 CodeContext; update index on file_write/edit; DKG PageRank if dkg.db exists. | 3J:L1279-L1297 |
| P3J-E061 | E/FLOW | Startup sequence: probe/start sidecar or ollama; BM25; QueryPipeline; Ssb/Fsm/SessionDag/RuleEngine/ContextBudget/PlanSkeleton; banner backend+index+session; REPL. | 3J:L1307-L1314 |
| P3J-E062 | E/TEST | E2E accept: REPL ≤60s excluding model load; both backends complete; BM25 context in prompt; non-zero CodeContext slot; release binary path. | 3J:L1341-L1348 |
| P3J-E063 | E/CAP | 3J.15 31B CPU offload: ≥2 steering vectors stored; DoLa 20 prompts buckets [0,20),[20,40),[40,60); TurboQuant codebook validate; EAGLE-3 architecture only, training deferred 48GB; metrics phase_3J_31b.toml. | 3J:L1350-L1428 |
| P3J-E064 | E/DEFER | 100B Q3 validation gated on 48GB GPU; SUBSTITUTE_MODEL escalation path test; metrics phase_3J_100b.toml if executed. | 3J:L1430-L1441 |
| P3J-E065 | E/TEST | Exit criteria list: VllmProvider+ModelPool; DoLa/steering E2B+31B; TurboQuant decision; multiplexing+preload; workload profiles; MILP decision; MCP; hooks SLOs; AppContainer; 17 SSB types; zero 0-3I regressions; EAGLE-3 arch deferred. | 3J:L1446-L1462 |
| P3J-E066 | E/STATUS | 3J.ADDENDUM shipped post-baseline: A-9 ETCLogitsProcessor; A-17/A-25 GG mean_logprob+entropy_mean; A-10 ConstrainedDecodingConfig grammar/JSON; A-26 routing_rules; A-27 bash safety_filter; A-28 git_commit; A-31 action_classifier; A-8 steering batch mode. | 3J:L1517-L1569 |
| P3J-E067 | E/SURFACE | Addendum surfaces: /research JSON-mode uses constrained decoding; COMMITTING FSM phase invokes git_commit tool; calibrate_etc_threshold.py; data/steering_contrast_pairs.toml. | 3J:L1525-L1569 |
| P3J-E068 | E/SURFACE | User-facing CLI verbs/flags in 3J body: elai / elai --local; elai.bat; /mode local\|api\|hybrid; /model; /plan; /plan phase <id>; /resume; /session fork; /compact; /shutdown (sidecar); REPL banner; degraded prefix; InteractiveGate Approve/Reject/Modify/Redirect. | 3J:L737-L767, L814-L831, L895-L898, L1027-L1052, L1078-L1079, L1268-L1273, L1307-L1314 |
| P3J-I001 | I/DEP | 3J.COMPOSE and 3J.LOCAL are PRE_GATE prerequisites for PIPELINE/E2E but are not defined as step headings in this file (likely baseline/other chunk). | 3J:L685-L686, L1257-L1257 |
| P3J-I002 | I/CFG | Summary "can_fit_onnx()" for multiplexing does not appear in 3J.6; source API is can_colocate (and can_fit may live elsewhere). | 3J:L368-L369 |
| P3J-A001 | A/CAP | Exact vLLM plugin/hook API for full DoLa logits remains VERIFY; degraded path always required until proven. | 3J:L76-L79, L210-L214, L1491-L1493 |
| P3J-A002 | A/CFG | Contrast prompt pairs for Auditor skepticism and Context-compiler completeness marked TBD at implementation. | 3J:L308-L313 |
| P3J-A003 | A/FLOW | hybrid/auto role_override vs later UNIFIED removal of Hybrid/Auto dead variants (session summary) leaves production mode enum history ambiguous. | 3J:L705-L712, L870-L870 |
| P3J-A004 | A/TEST | 3J.14 "NF4" in sidecar primary stack is named without algorithm detail in this file. | 3J:L1249-L1249 |
| P3J-C001 | C/STATUS | Step numbers in body (3J.6 multiplex+preload, 3J.7 profiles, 3J.8 MILP, 3J.9 MCP…) differ from MASTER summary bundling (3J.6 can_fit_onnx; 3J.7-8 preload+profiles; 3J.9 MILP; 3J.10-13 MCP/hooks/AppContainer/SSB). | 3J:L352-L681 vs MASTER:L370-L373 |
| P3J-C002 | C/STATUS | Opening says final step 3J.14 applies 31B; body renumbers 31B validation to 3J.15 and 3J.14 becomes WIRE.E2E. | 3J:L15-L16, L1246-L1350 |

Category coverage: capabilities ✓; user surfaces (CLI/flags/slash/gate/banner/degraded) ✓; workflows/FSM/preload ✓; schemas/SSB/types ✓; config flags ✓; integrations vLLM/MCP/sidecar/WSL ✓; security AppContainer/MCP/self-mod ✓; failures/fallback/retry ✓; determinism feature-gates ✓; observability metrics ✓; tests/exit ✓; deferred TurboQuant/MILP/EAGLE-3/100B ✓. Formal privacy policy N/A beyond env sanitization.

## Cross-source interactions and lifecycle

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| IX-001 | E | 1F DROP on Ollama reopens layer-level speculative/DoLa path when 3J delivers vLLM; 1F trait work avoids rewrite. | P1F-E004, P1F-E015, P1F-E016; P3J-E001, P3J-E009 |
| IX-002 | E | 1F requires 1C ModelPool VRAM accounting for draft overhead measurement. | P1F-E007, P1F-E010 |
| IX-003 | E | 2A freezes GrcEngine six-method trait so 2B can implement verify_claim/ground_plan without breaking BasicGrcEngine. | P2A-E016, P2A-E017 |
| IX-004 | E | 2A scope enumeration WARNs feed agent context; orchestrator (outside 2A) enforces plan_action coverage. | P2A-E013 |
| IX-005 | E | 3F depends on 3D complete and 1A tree-sitter via AstEvaluator; self-discovery needs 3C agent/auditor IPC. | P3F-E001, P3F-E006, P3F-E032 |
| IX-006 | E | 3F lifecycle PROPOSED→…→ACTIVE is the machine I-12 curator must not reimplement (cross-family note; source owns SM). | P3F-E018, P3F-E019 |
| IX-007 | E | 3J.10 tiered hooks consume 3F predicates and feed violation_patterns via orchestrator blake2s (same trust boundary as 3F.7). | P3F-E032, P3F-E035; P3J-E028, P3J-E029 |
| IX-008 | E | 3J.WIRE.RULES loads 3F seed engine into PreToolUse; WIRE.STUBS replaces AST/DKG stubs with knowledge adapters. | P3J-E048, P3J-E054; P3F-E036 |
| IX-009 | E | 3J workload_profile + advanced SSB types extend 3B core actions to ≥17 including phase_complete atomic boundary used later by 4A. | P3J-E023, P3J-E033 |
| IX-010 | E | 3J.WIRE.SESSION composes 3H SessionDag trim 85% + stub recovery from SSB. | P3J-E050, P3J-E051 |
| IX-011 | E | 3J.WIRE.PLAN composes 3G PlanSkeleton into CLI Slot 2 and /plan surfaces. | P3J-E052 |
| IX-012 | E | 3J.WIRE.FOUNDATION InteractiveGate is the 3A/4G-shared human gate surface (Approve/Reject/Modify/Redirect). | P3J-E045 |
| IX-013 | E | 3J.14 retrieval injects 1B/1E/1G pipeline pieces (BM25, CatRAG, ARAG, optional cross-encoder) into prompt Slot 4. | P3J-E060 |
| IX-014 | E | 3J.15 SUBSTITUTE_MODEL 100B path anticipates 4G escalation; anti-spiral 3× same error swap. | P3J-E064 |
| IX-015 | E | Hash policy: 3F/3J.10 discovery hashing is blake2s; other families may require blake3 — phase-local exception. | P3F-E032; P3J-E029 |
| IX-016 | E | 2A BudgetExhausted/forced Converge is engine-local; 3J ContextBudget is prompt token allocation — distinct budget layers. | P2A-E007, P2A-E027; P3J-E037, P3J-E047 |
| IX-017 | I | 1F prompt-lookup preference and 3J layer hooks both aim at throughput without assuming Ollama internals. | P1F-E021, P1F-E032; P3J-E002 |
| IX-018 | E | 3J MCP tools join native tools under uniform ToolRequest/ToolResult after 3C agents exist. | P3J-E025, P3J-E027 |
| IX-019 | E | AppContainer self-mod locks prefigure 4A BLOCK-8 realpath+FS+git filters. | P3J-E031 |
| IX-020 | E | DoLa disabled for prompt-engineer aligns with constrained-decoding-only role (addendum A-10). | P3J-E013, P3J-E066 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Note | Evidence atoms |
|---|---|---|---|
| AC-001 | conflict | 1F prerequisite Phase 0-independent vs 1C dependency graph. | P1F-C001 |
| AC-002 | conflict | 1F full eval suite vs RESOLVED skip 1F.3-1F.5. | P1F-C002 |
| AC-003 | conflict | 2A "grounding every step" summary line vs required subset. | P2A-C001 |
| AC-004 | ambiguity | 2A T11 hard Err vs warning for missing scope. | P2A-A002 |
| AC-005 | ambiguity | 2A full allowed_next table incomplete. | P2A-A001 |
| AC-006 | conflict | 3F rules→core-only crate map vs AstEvaluator knowledge coupling. | P3F-C001 |
| AC-007 | ambiguity | 3F evaluate return ALLOW\|BLOCK\|WARN vs ESCALATE enum arm. | P3F-A003 |
| AC-008 | conflict | 3J step numbering vs MASTER summary bundles. | P3J-C001 |
| AC-009 | conflict | 3J.14 31B in intro vs 3J.15 body renumber + E2E as 3J.14. | P3J-C002 |
| AC-010 | ambiguity | DoLa full-logit access method still VERIFY. | P3J-A001 |
| AC-011 | ambiguity | hybrid/auto BackendMode vs session removal of dead variants. | P3J-A003 |
| AC-012 | dependency | 3J.COMPOSE/3J.LOCAL referenced but not defined in this file. | P3J-I001 |
| AC-013 | qualification | MASTER can_fit_onnx for 3J.6 vs source can_colocate. | P3J-I002 |
| AC-014 | ambiguity | SCHEMA_REQUIRED provider-tier escalation is MASTER-only, not 3F body. | P3F-I001 |
| AC-015 | ambiguity | 3F.5 NaN/Inf metric guard appears in MASTER not phase-3F body. | MASTER:L314 vs P3F-E024 |

## Summary additions and corrections

### 1F deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-1F-001 | **ADD** full RESOLVED timeline: ollama 0.20.2 NOT_AVAILABLE, PR #8134 never merged, consolidated phase_1F.toml decision=UNAVAILABLE, EXIT satisfied via DROP. | MASTER:L114-L117 | P1F-E002–P1F-E006, P1F-E015 |
| SD-1F-002 | **ADD** complete decision gate order, boundary DROP at ==1.3x/3000MB, prompt-lookup preference, acceptance early kill, k-sweep, GPU temp rules, marginal retest. | MASTER:L120-L120 | P1F-E020, P1F-E032–P1F-E037, P1F-E041 |
| SD-1F-003 | **ADD** hardware scenarios A/B remaining VRAM, nvidia-smi polling, Phase-0-hardware-only ADOPT caveat, TIDE conditional later opt. | MASTER:L121-L121 | P1F-E010-P1F-E011, P1F-E040, P1F-E043 |
| SD-1F-004 | **ADD** adversarial serde/num_draft_tokens=0, silent-ignore false positive, cache uniqueness, and all metrics file paths. | MASTER:L118-L120 | P1F-E022-P1F-E031, P1F-E039 |
| SD-1F-005 | **QUALIFY** "Phase 2+" revisit as post-Phase-1/vLLM (3J), not a Phase-2 numbered dependency. | MASTER:L117, L659 | P1F-I001, P1F-E004 |

### 2A deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-2A-001 | **ADD** design D1-D5 paper grounding (BudgetThinker, RARR/SAFE, DSPy Assert/Suggest) and check-then-increment rationale. | MASTER:L172-L186 | P2A-E005-P2A-E009, P2A-E027 |
| SD-2A-002 | **ADD** full T1-T12 catalog notes including max_calls=3 trick, free Status, default 5/800, scope three-way tests. | MASTER:L182-L182 | P2A-E020-P2A-E023 |
| SD-2A-003 | **ADD** valid grounding type list and level mapping including tool→High and search→Verified. | MASTER:L184-L184 | P2A-E025, P2A-E026 |
| SD-2A-004 | **ADD** metrics methodology (1000-iter median, no criterion without approval) and reset_topic full removal. | MASTER:L186-L186 | P2A-E028, P2A-E030 |
| SD-2A-005 | **FLAG** T11 Err-vs-warn, incomplete allowed_next table, and "every step grounding" wording conflict. | MASTER:L172-L186 | P2A-A001-P2A-A002, P2A-C001 |

### 3F deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-3F-001 | **ADD** WARN preservation with BLOCK, severity DESC within type, and evaluation-order rationale. | MASTER:L305-L306 | P3F-E007, P3F-E008 |
| SD-3F-002 | **ADD** full transition thresholds, mutable=0 dual invariants, transition log fields, insufficient-shadow-data error. | MASTER:L311-L313 | P3F-E018-P3F-E023 |
| SD-3F-003 | **ADD** PostToolUse seed GREP surfaces (img/table/TODO strings), merge override rules, seed contract-test minimums. | MASTER:L317-L317 | P3F-E036-P3F-E038 |
| SD-3F-004 | **ADD** proptest De Morgan/associativity, READ-LOCK on contract_tests, COMPOSITE depth 8 UNGROUNDED note, counterfactual runtime deferral. | MASTER:L318-L319 | P3F-E039-P3F-E042, P3F-E044 |
| SD-3F-005 | **CORRECT/QUALIFY** MASTER SCHEMA_REQUIRED "escalates provider tier" and NaN/Inf guard: not stated in phase-3F body (schema validate + metric ops only). | MASTER:L306, L314 | P3F-I001, AC-014, AC-015 |
| SD-3F-006 | **ADD** AstEvaluator injection option (b) recommended, domain_mapping config, scoring [0,15], blake2s discovery. | MASTER:L303-L316 | P3F-E006, P3F-E029, P3F-E032, P3F-E034 |

### 3J deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-3J-001 | **CORRECT** step numbering: multiplex+preload are 3J.6; profiles 3J.7; MILP 3J.8; MCP 3J.9; hooks 3J.10; AppContainer 3J.11; SSB 3J.12; verify 3J.13; E2E 3J.14; 31B 3J.15. | MASTER:L370-L374 | P3J-C001, P3J-C002, P3J-E020-P3J-E063 |
| SD-3J-002 | **CORRECT** 3J.6 API is can_colocate + MultiplexingPolicy fields, not can_fit_onnx in this file. | MASTER:L370 | P3J-E020, P3J-I002 |
| SD-3J-003 | **ADD** complete user-facing surface set: --local, elai.bat, /mode,/model,/plan,/resume,/session fork,/compact, InteractiveGate four choices, degraded prefix, startup banner. | MASTER:L675-L681 | P3J-E038, P3J-E042, P3J-E045, P3J-E050, P3J-E052, P3J-E058, P3J-E061, P3J-E068 |
| SD-3J-004 | **ADD** full WIRE.* chain FOUNDATION→BUDGET→RULES→SESSION→PLAN→STUBS and UNIFIED RuntimeClient constraints. | MASTER:L675-L681 | P3J-E040-P3J-E055 |
| SD-3J-005 | **ADD** MCP security/retry, hook sanitization string, AppContainer job limits/env allowlist, isolation_metric formula, SSB ≥17 write-time validation. | MASTER:L373-L373 | P3J-E025-P3J-E034 |
| SD-3J-006 | **ADD** 3J.14 Ollama fallback re-probe, QueryPipeline Slot 4, BM25 update-on-edit, 60s REPL SLO, release binary checks. | MASTER:L675-L681 | P3J-E057-P3J-E062 |
| SD-3J-007 | **ADD** 3J.15 31B buckets, EAGLE-3 skip reason, optional 100B SUBSTITUTE_MODEL gate, metrics file names. | MASTER:L374 | P3J-E063, P3J-E064 |
| SD-3J-008 | **ADD** entire 3J.ADDENDUM shipped extensions A-8/9/10/17/25/26/27/28/31 with file paths. | MASTER:L360-L375 (absent) | P3J-E066, P3J-E067 |
| SD-3J-009 | **ADD** hardware/model install snapshot (vLLM 0.19.0 WSL2, gemma4 sizes, native vision) and R1-R11 risk register. | MASTER:L363-L364 | P3J-E003-P3J-E008 |
| SD-3J-010 | **FLAG** undefined 3J.COMPOSE/LOCAL steps, TBD contrast pairs, NF4 detail, hybrid mode history. | MASTER:L675-L681 | P3J-I001, P3J-A002-P3J-A004, AC-011-AC-012 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **4/4 files**, **2,735/2,735 logical lines**, **169,614/169,614 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **213 unique atoms** = **190 explicit**, **005 inferred**, **012 ambiguous**, **006 conflicting/corrective**.
- Per-file atom counts: 1F **049**; 2A **038**; 3F **050**; 3J **076**. Sum **213**.
- Relational interaction records: **20** (`IX-001`–`IX-020`). Summary-delta records: **026** (`SD-*`). Ambiguity/conflict register: **15** (`AC-001`–`AC-015`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `P1F/P2A/P3F/P3J-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them.
- User-facing surface audit: 1F probe output + decision/metrics artifacts; 2A engine API/typed errors (no panel); 3F PostToolUse WARN/INFO strings + evaluate verdicts; 3J CLI flags/slash commands/InteractiveGate/degraded banner/REPL/startup sequence/elai.bat — each captured as explicit atoms, not folded into backend bullets.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta/I records and are not elevated to explicit requirements.
- Manifest-boundary audit: the only original sources opened were the four exact DEEP-PHASE-005 manifest rows. The only orientation artifact consulted was `01-phase-index-MASTER.discovery.md`.
- Output metrics after finalization: `output_lines=0381`; `output_bytes=057830`.
- Canonical normalized output SHA-256: 064e0ec251d167638c6413d375d69a3492e20c2df0013d2bf135c9ae56c5e552
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
