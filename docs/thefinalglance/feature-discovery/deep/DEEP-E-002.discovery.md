# DEEP-E-002 feature discovery

## Scope and evidence rules

This is the deep pass for manifest chunk **DEEP-E-002**. The only original-plan sources read were the nine files in the exact manifest block reproduced below. **05-E-SUMMARY_INDEX.discovery.md** was used only for orientation and delta calculation; it is not evidence for a deep-source feature unless an assigned original independently states that feature. One previously generated discovery artifact, DEEP-D-001.discovery.md, was briefly consulted only for output structure and contributed no feature claim. No other original-plan source, companion report, implementation file, paper, or web source was opened.

Notation:

- **[E]** explicit in the named assigned original.
- **[I]** inference joining explicit atoms; never promoted to source fact.
- **[A]** ambiguous or underspecified.
- **[C]** conflicting statements, within one source, across this chunk, or between the summary and assigned source.
- Categories: **CAP** capability/action/surface; **WF** workflow/state/lifecycle; **DATA** schema/data; **CFG** flag/default/config; **INT** integration/event/dependency; **SEC** permission/security/privacy/compliance; **FAIL** failure/fallback/retry/recovery; **DET** determinism/idempotency/concurrency; **OBS** observability/evidence; **TEST** test/acceptance/done gate; **CON** constraint/non-goal; **STATUS** status/deferred/closure.

An atomic discovery unit is one uniquely keyed, source-local contract statement below. A complete type, decision matrix, test vector, or closely coupled inventory may remain one atom so its semantics are not fragmented. Repeated wording in task, wiring, deterministic-check, acceptance, and files-touched sections is counted once and cross-referenced rather than duplicated. Non-explicit analysis and summary deltas reference atoms and add no atomic count.

## Exact processed manifest and fingerprint verification

Manifest location: **DEEP-SOURCE-MANIFEST.md:529-543**; the next heading, **DEEP-E-003**, begins at line 545.

> ### DEEP-E-002
>
> Family **E**; totals: **9 files**, **2131 logical lines**, **251948 bytes**; target status: **within 1,800–2,800**.

| Chunk ID | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---|---:|---:|---|---|---|
| DEEP-E-002 | numbered/index original | 209 | 28213 | 3f4e0b3c189eeddef277c83c2277db24d1481f6eb2231e7df8e00a62c01f6d25 | docs/thefinalglance/E-plan/E-1_infinite_context_chat_illusion.md | exact match |
| DEEP-E-002 | numbered/index original | 200 | 24403 | 502b84c4ee3bfc91ee1c94f8b8b5d1e16c7560869015a9de3c3e8856427ddd35 | docs/thefinalglance/E-plan/E-3_model_handoff_ux.md | exact match |
| DEEP-E-002 | numbered/index original | 217 | 24950 | 4627c7cff910c274e50b71e9087d8c48958f1924d64173754df9ff08e3e0756b | docs/thefinalglance/E-plan/E-14_browser_e2e_agents.md | exact match |
| DEEP-E-002 | numbered/index original | 181 | 17170 | 562b0f096a64febd1dc82af1bedd4a63f75e54fe6e16c2bfef19a3ee59570e73 | docs/thefinalglance/E-plan/E-16_vlm_image_validation.md | exact match |
| DEEP-E-002 | numbered/index original | 251 | 22214 | b87ac81c7b9421ad6aab068e45a14e6b896a6c1ecbedea214c16697bc9adddfc | docs/thefinalglance/E-plan/E-17_rag_retrieval_quality.md | exact match |
| DEEP-E-002 | numbered/index original | 231 | 24911 | 480d16062314089e229723832c96a9f2f2b6afe58358ef585e9544b3d781ec6e | docs/thefinalglance/E-plan/E-34_rust_ai_ecosystem_2026.md | exact match |
| DEEP-E-002 | numbered/index original | 258 | 33705 | 54543d18aecca52d7331db3cb1ea2bf7f68fd0a4e2030d545f937993e6e2c4db | docs/thefinalglance/E-plan/E-39_cost_token_economics.md | exact match |
| DEEP-E-002 | numbered/index original | 280 | 35454 | 54868188d615640e830e53237928e84fc2c7cf666c3d5ccf4f893f2b380a94f5 | docs/thefinalglance/E-plan/E-49_coding_agent_metrics.md | exact match |
| DEEP-E-002 | numbered/index original | 304 | 40928 | 01f93a3d63fdd7b76a420a658f79433e3e3b7dae0896d65004169eec6df45f07 | docs/thefinalglance/E-plan/E-53_formal_verification_codegen.md | exact match |
| **Verified total** | **9 files** | **2131** | **251948** | — | — | **exact match** |

Verification used **sha256sum** and **wc -lc** against the exact absolute paths. There were zero missing files and zero hash, logical-line, byte, per-file, or aggregate mismatches.

## Per-file explicit ledgers

### E-1_infinite_context_chat_illusion.md — 209/209 lines

- **E1-001 [E][STATUS/CAP]** The completion-drive rollup claims eleven surfaces were already shipped on V2: the memory crate skeleton with twelve modules, bi-temporal memory_notes table, NoteStore CRUD/queries, CoreBlock/CoreBlockStore, SleepCompute FSM variant/string mapping, four typed memory/Core SSB actions with payload and bijection, HippoRAG PPR retrieval, and a registered get_memory_notes command plus mounted MemoryNotesPanel. (E-1_infinite_context_chat_illusion.md:3-4)
- **E1-002 [E][STATUS/CAP/INT]** The same rollup claims this session shipped evolve_on_insert, AttributeDiff, apply_diff, ppr_seeded_top_k, two memory tool definitions, additive task-runner injection, CLI deterministic preamble plus chat-memory-reflect dispatch, closed the core_block.rs stub, and documented eight SPEC_DRIFT resolutions. (E-1_infinite_context_chat_illusion.md:5)
- **E1-003 [E][STATUS/DATA/INT]** SleepComputeStarted and SleepComputeQueryCached were consolidated into the already-shipped MemoryEvolutionRun event, discriminated by phase values start and cached_query and carrying duration_ms plus notes_touched; SCHEMA_VERSION 103 is declared locked. (E-1_infinite_context_chat_illusion.md:6)
- **E1-004 [E][STATUS/OBS]** The rollup removes fabricated BeliefShift arXiv:2603.23848, corrects the minus-50-percent-token attribution from Mem0 to sleep-time compute, and marks the MemGPT paper ingestion pending while accepting a Letta blog as secondary grounding. (E-1_infinite_context_chat_illusion.md:7)
- **E1-005 [E][STATUS/INT]** Seven live-code anchors were refreshed, and chat_memory_reflect.rs rather than chat_reflect.rs is declared the canonical PRE-stage target because it already owns the memory-store imports. (E-1_infinite_context_chat_illusion.md:8-9)
- **E1-006 [E][STATUS/DEP]** Five cross-crate contract files, explicit user_typing_detected/user_input_committed SSB variants, and MemGPT paper ingestion are deferred; in-crate analogs are claimed to cover the deferred contract tests functionally. (E-1_infinite_context_chat_illusion.md:10)
- **E1-007 [E][CAP/WF]** The objective replaces flat 150-token MistakeBook injection with A-MEM notes over SSB, a MemGPT/Letta Core-plus-Archival split inside 25K context, and between-turn sleep compute so local Qwen appears continuous without raw-context cost. (E-1_infinite_context_chat_illusion.md:12-13)
- **E1-008 [E][DEP/STATUS]** Required foundations are B-1, B-6, B-9, B-22, and B-39 DONE plus E-5 memory schema; E-2/E-3/E-4 are declared parallel-safe subject to distinct touched surfaces/action variants. (E-1_infinite_context_chat_illusion.md:14-16)
- **E1-009 [E][DATA/CON]** Existing MistakeBook data is a per-baby-step diff-hash list occupying the 150-token PhaseSpec slot and does not retain user opinions, voice, style, expertise, or cross-session statements. (E-1_infinite_context_chat_illusion.md:18-20)
- **E1-010 [E][CAP/DATA]** The four conceptual primitives are A-MEM notes with context/keywords/tags/links and evolution, a roughly 1.5K-token editable Core block plus paged Archival memory, idle compute for user-model/retrieval/summary refresh, and Zep-style valid_at/invalid_at temporal history. (E-1_infinite_context_chat_illusion.md:22)
- **E1-011 [E][INT/CON]** Existing PageRank, SSB, and ChatReflectPipeline primitives are reused; MistakeBook remains only as a fact_type=mistake view while its injection path is replaced by note retrieval. (E-1_infinite_context_chat_illusion.md:24,28-36)
- **E1-012 [E][DEP/SEC]** SessionDag stores the stable intent vector as a snapshot annotation, SleepCompute mirrors autonomous evidence derivation, digest-writer defaults to the local Sidecar runtime, C-21 is a hard prerequisite for view compatibility, C-48 trust tiers are inherited, and temporal invalidation supports later sycophancy work. (E-1_infinite_context_chat_illusion.md:28-35)
- **E1-013 [E][DATA/CFG]** The planned memory_notes schema has sixteen named columns: note_id, scope, tags, keywords, context, body, links, fact_type, valid_from, valid_to, insertion_time, last_accessed, trust_score, decay_rate, evidence_pointers, and superseded_by; scope is session/project/global and the task text proposes schema 45 to 46. (E-1_infinite_context_chat_illusion.md:59)
- **E1-014 [E][CAP/DATA/DET]** insert_note fingerprints the body with blake3, finds top-K linked notes by tag intersection, evolves linked-note attributes into zero or more diffs, and commits the batch atomically; evolution depth is capped at one hop to prevent thrash. (E-1_infinite_context_chat_illusion.md:61)
- **E1-015 [E][CAP/DATA/CFG]** HippoRAG retrieval reuses the existing PageRank kernel, builds sparse adjacency from links JSON, seeds from TRIAGE entity extraction, performs one non-IRCoT PPR call per turn, and returns top eight notes. (E-1_infinite_context_chat_illusion.md:63)
- **E1-016 [E][CAP/CFG/CON]** PhaseSpec injection is renamed emit_memory_note_constraints, remains hard-capped at 150 tokens, and selects PPR-ranked live facts of types mistake, preference, opinion, or expertise; query_mistake_book remains a compatibility view. (E-1_infinite_context_chat_illusion.md:65)
- **E1-017 [E][DATA/CFG]** CoreBlock contains user_style, a topic-to-ExpertiseLevel map, active_goal, and opinions; it is capped at 1,500 tokens and persisted as one global fact_type=core row. (E-1_infinite_context_chat_illusion.md:67)
- **E1-018 [E][CAP/FAIL/OBS]** TRIAGE exposes core_memory_replace(section,new_text) and archival_search(query,top_k); typed validation turns an invalid section into a silent no-op plus an SSB warning rather than a hard error. (E-1_infinite_context_chat_illusion.md:67)
- **E1-019 [E][WF/CFG/INT]** The shipped production reflection trigger runs after every successful turn, calls should_reflect with K=4, dispatches chat_memory_reflect::reflect, and emits MemoryEvolutionRun start/cached_query phase rows. (E-1_infinite_context_chat_illusion.md:69)
- **E1-020 [E][WF/DATA/CFG]** Reflection inserts notes, appends CoreBlock, runs decay_tick, and reports duration_ms/notes_touched; ReflectConfig::max_turns_per_note defaults to 32, while explicit typing/commit events await a future schema bump. (E-1_infinite_context_chat_illusion.md:69)
- **E1-021 [E][DATA/CFG]** Retrieval drops invalidated rows, reweights trust by exp of negative decay_rate times age since last access, and updates last_accessed on retrieval. (E-1_infinite_context_chat_illusion.md:71)
- **E1-022 [E][FAIL/SEC/INT]** A same-tag-set note with embedding cosine below -0.4 is treated as structurally opposing; the system emits a conflict row, preserves the original without auto-update, and requires user adjudication. (E-1_infinite_context_chat_illusion.md:71)
- **E1-023 [E][CAP/OBS]** Before sidecar invocation, CLI synchronously emits a one-to-two-sentence preamble from the top three PPR notes, before knowledge augmentation/reflection, targeting user-visible bytes in under 50 ms; the dashboard SSE preamble is deferred. (E-1_infinite_context_chat_illusion.md:73)
- **E1-024 [E][STATUS/INT]** MemoryNoteInserted, MemoryNoteEvolved, MemoryConflictDetected, and CoreBlockReplaced are claimed shipped with SQL string conversion, payload, and round-trip coverage; the two old sleep actions are explicitly superseded by MemoryEvolutionRun phases. (E-1_infinite_context_chat_illusion.md:75)
- **E1-025 [E][CAP/OBS]** MemoryNotesPanel renders current-turn PPR top-K, Core sections, conflict markers, and a decay heat map through get_memory_notes(scope,top_k), following the RuntimePanel shape. (E-1_infinite_context_chat_illusion.md:77)
- **E1-026 [E][CFG/STATUS]** The decision record chooses a net-new memory crate, recommends lazy evolution during sleep except explicit Core replacement, and sets provisional daily decay defaults: opinion 0.0005, mistake 0.005, expertise 0.0001. (E-1_infinite_context_chat_illusion.md:79)
- **E1-027 [E][CAP/INT]** The wiring contract exports NoteStore, ppr_seeded_top_k, evolve_on_insert, CoreBlock/tool handling, SleepCompute, memory events, and get_memory_notes, and consumes the task-runner/context-budget/chat/CLI/PageRank sites named in the file. (E-1_infinite_context_chat_illusion.md:81-100)
- **E1-028 [E][WF/INT]** Reachability is CLI main or dashboard stream_chat into the same runtime/reflection path; the stale wiring text also says SleepCompute is reached from IDLE on user_typing_detected. (E-1_infinite_context_chat_illusion.md:98-101)
- **E1-029 [E][CON/SEC]** Literal lazy placeholders and any PhaseSpec injection path bypassing ppr_seeded_top_k are banned. (E-1_infinite_context_chat_illusion.md:102-106)
- **E1-030 [E][TEST/OBS]** The observable smoke records a hashing preference over two turns, starts a fresh session, expects memory/sleep/Core events within 0.5/4/8 seconds, recalls blake3 unprompted with prior-session attribution, and shows a live note with trust above 0.7. (E-1_infinite_context_chat_illusion.md:107-110)
- **E1-031 [E][TEST]** Deterministic checks require workspace memory dependency/imports, the memory_notes columns and proposed version 46, all six originally named SSB actions, only a delegating legacy MistakeBook shim, the FSM variant, and an idle smoke count at least one. (E-1_infinite_context_chat_illusion.md:112-137)
- **E1-032 [E][TEST/DATA]** Acceptance requires all sixteen columns plus insertion_time and live-row indexes, a clean workspace build, action round trips, PPR-ranked injection within 150 tokens, and both Core/Archival tools in TRIAGE. (E-1_infinite_context_chat_illusion.md:139-146)
- **E1-033 [E][TEST/WF/FAIL]** Acceptance also requires a measurable idle gap of at least two seconds, age-sensitive opinion decay, conflict-without-update, cross-session unprompted recall, live panel rendering, and no production unwrap in the memory crate. (E-1_infinite_context_chat_illusion.md:146-151)
- **E1-034 [E][TEST/DET]** Note tests require no more than ten-percent body-edit churn after step 50, stable top-eight retrieval from 100 notes under 50 ms, and monotonically decreasing weights for 30-day/7-day/1-hour notes. (E-1_infinite_context_chat_illusion.md:153-157)
- **E1-035 [E][TEST/FAIL]** Contract tests cover SSB field preservation, idle substate/time budget, oversized Core edit as no-op/warning/no panic, 150-token PPR injection, and conflict-event emission with the original untouched. (E-1_infinite_context_chat_illusion.md:158-162)
- **E1-036 [E][CAP/STATUS]** The file inventory creates the memory crate’s note/evolution/retrieval/store modules, MemoryNotesPanel, and decision doc, and modifies workspace membership, SSB/viewer/task/context/FSM/reasoning/CLI/Tauri/PageRank integration sites. (E-1_infinite_context_chat_illusion.md:164-183)
- **E1-037 [E][OBS/STATUS]** Grounding explicitly distinguishes primary architectural papers, the corrected sleep-compute attribution, LongMemEval evaluation, CTIM-Rover counter-evidence, and the temporary Letta secondary source while MemGPT ingestion remains pending. (E-1_infinite_context_chat_illusion.md:185-196)
- **E1-038 [E][TEST/A]** Note-evolution stability on local Qwen is unmeasured; the declared measurement threshold is under ten-percent churn after turn 50. (E-1_infinite_context_chat_illusion.md:198-200)
- **E1-039 [E][CFG/A]** Sleep-compute payoff is unknown; the source proposes a 4,000-token budget and one week of duration telemetry before recalibration. (E-1_infinite_context_chat_illusion.md:201)
- **E1-040 [E][SEC/A]** Core drift on a 7B model is unvalidated; mitigation is typed sections, JSON-schema validation, and rejection-rate telemetry. (E-1_infinite_context_chat_illusion.md:202)
- **E1-041 [E][A]** Stability of entity-seeded PageRank top-K on local Qwen is explicitly ungrounded and has no empirical floor. (E-1_infinite_context_chat_illusion.md:203)
- **E1-042 [E][CON]** Speculative decoding is out of scope and delegated to E-2; MIRIX-style eight-agent memory orchestration is explicitly rejected for a single 7B. (E-1_infinite_context_chat_illusion.md:204-205)
- **E1-043 [E][DEP/INT]** E-5 supplies scope/schema, C-21 consolidates the compatibility view, D-14 consumes conflict rows, and E-3 consumes CoreBlock for cross-runtime voice. (E-1_infinite_context_chat_illusion.md:207-209)

### E-3_model_handoff_ux.md — 200/200 lines

- **E3-001 [E][CAP/WF]** The objective makes local-to-cloud-to-local swaps invisible through one cross-runtime voice preamble, role-first UI, constant-size digested handoff, predictive adapter warmup, and an optional local persona-vector gate. (E-3_model_handoff_ux.md:1-3)
- **E3-002 [E][DEP/STATUS]** B-37 RuntimeAdapter/PerRoleRuntime and E-1 CoreBlock/memory_notes are prerequisites; C-15 timing is optional; E-2/E-4 are parallel-safe only with coordinated CommandWindow bubble-header edits. (E-3_model_handoff_ux.md:4-6)
- **E3-003 [E][CAP/DATA]** The existing runtime set has eleven backends: Sidecar, AnthropicApi, seven named CLI ports, Ollama, and Vllm; roles map designer/auditor/researcher to Anthropic and executor/memory_writer/context_compiler/prompt_engineer to Sidecar. (E-3_model_handoff_ux.md:8-10)
- **E3-004 [E][CON]** Cross-backbone KV-cache reuse is explicitly out of scope; a compressed HandoffPayload over E-1’s digest is the cross-model substitute. (E-3_model_handoff_ux.md:12-16)
- **E3-005 [E][CFG/CON]** Persona-vector activation steering is local-Qwen-only because cloud activations are inaccessible, and remains optional. (E-3_model_handoff_ux.md:16)
- **E3-006 [E][INT/STATUS]** E-3 extends shipped B-37/B-22/B-39/B-9 surfaces without changing ChatReflectPipeline stage isolation, treats E-1 CoreBlock as canonical voice anchor, and adds predictive prewarm/persona behavior net-new. (E-3_model_handoff_ux.md:18-27)
- **E3-007 [E][CAP/DATA/CFG]** build_role_prompt(role,voice,task) prepends an approximately 80–120-token VoiceAnchor containing tone, formality, forbidden phrasings, confidence calibration, sycophancy guard, and terse-default policy to every role prompt. (E-3_model_handoff_ux.md:48-50)
- **E3-008 [E][DATA/CFG]** The single voice source is .elai_cc/agents/voice.toml and applies uniformly upstream of all eleven RuntimeAdapter implementations. (E-3_model_handoff_ux.md:50-52)
- **E3-009 [E][WF/OBS]** Every RuntimeOrchestrator::send builds the prompt before stream_step and emits UnifiedPreambleApplied with role, runtime_backend, and preamble_blake3; adapters themselves need no prompt change. (E-3_model_handoff_ux.md:52)
- **E3-010 [E][CAP/SEC]** Each chat bubble shows action-oriented role text, hides the model by default, and exposes the backend only when the user clicks the role badge to open a drawer. (E-3_model_handoff_ux.md:54)
- **E3-011 [E][WF/DET]** FSM transitions produce predicted_next_role from the canonical transition table and asynchronously/non-blockingly prewarm the predicted adapter. (E-3_model_handoff_ux.md:56)
- **E3-012 [E][CFG/FAIL]** Prewarm is a no-op on 12GB, uses RuntimeAdapter::warm on 24GB+, and emits RuntimePrewarmStarted/Complete. (E-3_model_handoff_ux.md:56)
- **E3-013 [E][CAP/FAIL]** RuntimeAdapter::warm has an immediate no-op default; Sidecar sends a one-token CUDA-graph warmup and AnthropicApi sends a one-token connection-pool warmup; nine other backends retain the default. (E-3_model_handoff_ux.md:58)
- **E3-014 [E][CAP/DATA/OBS]** compress_handoff_payload uses token frequency plus the existing cross-encoder reranker without an LLM, targets 5–20x compression across PerRoleRuntime boundaries, and logs original/compressed tokens and ratio. (E-3_model_handoff_ux.md:60)
- **E3-015 [E][CFG/CAP]** The local sidecar persona hook is controlled by ELAI_PERSONA_VECTOR=1, derives an ELAI vector at boot from contrastive in-voice/neutral pairs, and applies it through the existing steering processor during decode. (E-3_model_handoff_ux.md:62)
- **E3-016 [E][CFG/CAP/CON]** ELAI_STYLE_NORMALIZE=1 enables a default-off display-only pass for cross-model outputs, stripping role-specific presentation tells while never stripping auditor BLOCK from agent-to-agent traffic. (E-3_model_handoff_ux.md:64,135-136)
- **E3-017 [E][CFG/CON]** vLLM must have enable_prefix_caching true for its own immutable local prefix; this is explicitly not cross-model cache reuse. (E-3_model_handoff_ux.md:66)
- **E3-018 [E][DATA/CFG]** voice.toml defines anchor rules including lowercase i, no em dashes, no robotic prefixes, and apostrophe-free contractions, plus Verified/High confidence/Uncertain calibration, a citation-backed sycophancy guard, and forbidden prefixes feat:, chore:, and Phase N:. (E-3_model_handoff_ux.md:68)
- **E3-019 [E][CFG/STATUS]** The decision doc centralizes the prompt module, keeps persona and style normalization off pending evidence, and proposes prewarm tiers of 12GB no-op, 24GB enabled, and 48GB aggressive. (E-3_model_handoff_ux.md:70)
- **E3-020 [E][CAP/INT]** The wiring contract exports build_role_prompt, VoiceAnchor loading, warm, predict_next_role, six named SSB actions, and the Python persona hook. (E-3_model_handoff_ux.md:72-80)
- **E3-021 [E][INT/WF]** The integration consumes send/handoff/CommandWindow/Sidecar/Vllm surfaces and is reachable from CLI or dashboard dispatch and the task-runner FSM transition handler. (E-3_model_handoff_ux.md:81-89)
- **E3-022 [E][CON]** Missing voice anchors, hardcoded Sonnet, prompt paths bypassing build_role_prompt, and DroidSpeak-style cross-architecture KV code are banned. (E-3_model_handoff_ux.md:90-95)
- **E3-023 [E][TEST/OBS]** The observable cycle EXECUTING to AUDITING to MEMORIZING expects at least three stable-hash preamble rows, immediate prewarm-start rows, role badges with drawer-only models, and consistent lowercase/no-em-dash/terse voice across backends. (E-3_model_handoff_ux.md:96-99)
- **E3-024 [E][TEST/DET]** Deterministic checks require the role module/import, all four TOML sections, all six action identifiers, a default warm method, one distinct preamble hash over the cycle, and zero cross-model-KV/DroidSpeak hits. (E-3_model_handoff_ux.md:101-125)
- **E3-025 [E][TEST/CFG]** Acceptance makes voice.toml the only voice source, requires a stable preamble hash per send, role-first drawer behavior, and two specialized warm implementations plus nine defaults. (E-3_model_handoff_ux.md:127-133)
- **E3-026 [E][TEST/CFG]** Acceptance requires at least 5x handoff compression, persona steering off by default with at least ten-percent voice-drift reduction when enabled, and display-only style normalization off by default. (E-3_model_handoff_ux.md:134-136)
- **E3-027 [E][TEST/CON]** Acceptance forbids cross-architecture KV reuse and any mid-stream model swap path. (E-3_model_handoff_ux.md:137-138)
- **E3-028 [E][TEST/DET]** Tests cover one stable anchor hash across eight AgentRole values, the FSM prediction table, compile compatibility for eleven adapters, event shape/hash stability, compression at least 5x, persona extraction/no decode crash, and role-badge drawer interaction. (E-3_model_handoff_ux.md:140-148)
- **E3-029 [E][CAP/STATUS]** The file inventory creates role_prompt.rs, voice.toml, and a decision doc, then changes agents/runtime/FSM/provider/handoff/rendering/dashboard/Python/SSB surfaces. (E-3_model_handoff_ux.md:150-167)
- **E3-030 [E][A/OBS]** Per-transition cold-start decomposition is unknown; prewarm stays tier-gated until telemetry measures load, graph-capture, and prefix-reencode costs. (E-3_model_handoff_ux.md:189-191)
- **E3-031 [E][A/TEST]** Persona effects on abliterated Qwen 3B/7B are unvalidated and require an A/B coherence test before promotion. (E-3_model_handoff_ux.md:192)
- **E3-032 [E][STATUS/A]** The voice-coherence human-plus-judge harness is deferred to E-4 or a follow-up; the digest payoff threshold is co-investigated with E-1. (E-3_model_handoff_ux.md:193-194)
- **E3-033 [E][CON/STATUS]** Self-MoA versus cloud swapping is not addressed, and all DroidSpeak hopes are explicitly rejected as wasted engineering. (E-3_model_handoff_ux.md:195-196)
- **E3-034 [E][DEP/INT]** E-1 supplies the anchor, E-4 layers confidence UI on the shared header, a future LoRA may address residual drift, and D-14 consumes the voice guard as an evaluation signal. (E-3_model_handoff_ux.md:198-200)

### E-14_browser_e2e_agents.md — 217/217 lines

- **E14-001 [E][CAP/WF]** The objective combines AXTree-first Playwright Tier 1, axe gates, constrained downgrade-only VLM Tier 2, SSIM-first regional escalation, and a Cosmos deploy-preview worker with deployment polling and cURL backoff. (E-14_browser_e2e_agents.md:1-3)
- **E14-002 [E][DEP/STATUS]** B-24 and B-42 are shipped prerequisites, C-7/C-16 are planned, and the work augments those surfaces while being parallel-safe with E-11/E-12/E-13/E-15. (E-14_browser_e2e_agents.md:4-6)
- **E14-003 [E][SEC/CON]** Cloudflare/Turnstile bypass is prohibited; detection must skip protected flows because headless automation is fingerprinted and bypass poses legal/ToS risk. (E-14_browser_e2e_agents.md:8-10)
- **E14-004 [E][WF/FAIL]** The closed veto rule is asymmetric: Tier 2 may downgrade a Tier 1 pass to HumanReview but can never upgrade a Tier 1 failure. (E-14_browser_e2e_agents.md:10)
- **E14-005 [E][CAP/STATUS]** Net-new surfaces are PlaywrightWebAdapter, StagehandWebAdapter, DeployPreviewWorker, axe injection, and hybrid SSIM routing; existing B-24 models remain unchanged. (E-14_browser_e2e_agents.md:12-21)
- **E14-006 [E][DATA/CAP]** ax_snapshot.rs defines AxNode with role/name/state, AxSnapshot, and AxSnapshotReader::from_disk; every Tier 1 invocation returns AX snapshot, screenshot, and console-log paths, and downstream rules match role/name rather than pixels. (E-14_browser_e2e_agents.md:38-40)
- **E14-007 [E][WF/CFG/DET]** PlaywrightWebAdapter invokes Playwright 1.56+ snapshot-to-disk in a Tier-C sandbox and writes ax-snapshot.json, screenshot.png, console.log, and trace.zip with retries=2, trace_on_retry=true, animations disabled, and deterministic seed. (E-14_browser_e2e_agents.md:42)
- **E14-008 [E][SEC/FAIL]** Axe runs on every Tier 1 web check: serious/critical immediately fail, moderate passes Tier 1 but advises Tier 2/dashboard, and minor is logged only. (E-14_browser_e2e_agents.md:44)
- **E14-009 [E][CAP/FAIL/DET]** Stagehand act/extract/observe is a Tier-C local-Node fallback only when AXTree role/name lookup fails; action sequences are cached by url_hash plus goal_hash at ~/.cache/elai/stagehand and replayed across DOM drift. (E-14_browser_e2e_agents.md:46)
- **E14-010 [E][DATA/FAIL]** Tier 2 must return schema-constrained JSON with pass and failures containing region/reason/confidence in [0,1]; free text raises VlmInvalidOutput and degrades to HumanReview. (E-14_browser_e2e_agents.md:48)
- **E14-011 [E][WF/CFG]** HybridDiff avoids VLM when SSIM_delta is below configurable threshold 0.02; at or above threshold it crops delta-map bounding boxes and sends only changed regions to Tier 2. (E-14_browser_e2e_agents.md:50)
- **E14-012 [E][OBS]** HybridDiff emits hybrid_diff_short_circuit when no VLM is needed and hybrid_diff_vlm_dispatched otherwise. (E-14_browser_e2e_agents.md:50)
- **E14-013 [E][WF/FAIL]** Verdict aggregation is exhaustive: T1 fail stays Fail; T1 pass/T2 pass is Pass; T1 pass/T2 fail is HumanReview; T1 pass/T2 unavailable is Degraded; absent T1 with either T2 result is HumanReview. (E-14_browser_e2e_agents.md:52)
- **E14-014 [E][INT/CFG]** DeployPreviewWorker mirrors cosmos_worker, polls Vercel/Netlify checks every 30 seconds using provider/api_token/project_ids, may fall back to cURL retry/backoff, and starts live smoke validation on the first HTTP 200. (E-14_browser_e2e_agents.md:54)
- **E14-015 [E][CFG/OBS]** Deploy-preview execution is project-opt-in with enabled=false by default and emits polling_started, ready, failed, and app_validation_complete actions. (E-14_browser_e2e_agents.md:54)
- **E14-016 [E][WF/DET]** SaasJourneyRunner creates one isolated browser context per simulated user, captures outbound mail/webhooks through network mocks, asserts captured notifications, and loads project steps/assertions from .elai_cc/journeys/{project}.toml. (E-14_browser_e2e_agents.md:56)
- **E14-017 [E][SEC/FAIL/OBS]** CfDetector checks challenge HTML, Turnstile widgets, and JA4 mismatch banners, returns Skipped(ReasonExternalProtection), never bypasses, and emits cf_protection_detected. (E-14_browser_e2e_agents.md:58)
- **E14-018 [E][INT/FAIL]** DeployPreviewArtifact is added to the Cosmos types; until C-16 ships, the worker runs through a standalone task-runner tick loop and is migrated into CosmosStage later. (E-14_browser_e2e_agents.md:60)
- **E14-019 [E][CAP/OBS]** MediaValidationPanel adds an AX role/name tree, severity-filtered axe list, SSIM/VLM gauge, deployment badge, multi-context journey timeline, and protection-skip banner. (E-14_browser_e2e_agents.md:62)
- **E14-020 [E][CAP/INT]** Three commands expose recent AX snapshots, axe findings, and deploy status. (E-14_browser_e2e_agents.md:62)
- **E14-021 [E][STATUS/TEST]** VLM Brier/ECE calibration is a follow-up after ship and feeds the SSIM default rather than blocking the initial implementation. (E-14_browser_e2e_agents.md:64)
- **E14-022 [E][CAP/INT]** The produced symbol set comprises AX reader, axe gate, refactored Playwright, Stagehand, constrained VLM, hybrid diff, SaaS journeys, CF detection, and deploy worker. (E-14_browser_e2e_agents.md:66-77)
- **E14-023 [E][INT/WF]** Task-runner media validation, research CosmosStage, and dashboard rendering are the stated entry paths. (E-14_browser_e2e_agents.md:78-87)
- **E14-024 [E][CON/SEC]** New modules prohibit todo/unimplemented, bypass logic, free-text VLM acceptance, Box of dynamic Error, and production unwrap. (E-14_browser_e2e_agents.md:88-94)
- **E14-025 [E][TEST/OBS]** The cloud todo-app smoke expects render, AX, axe, hybrid, and deploy events in order and durable ax-snapshot.json/axe-results.json artifacts, with T1 failure dominating any T2 result. (E-14_browser_e2e_agents.md:95-98)
- **E14-026 [E][TEST]** Deterministic presence checks cover four core types, axe integration, deploy worker, nine named action identifiers, constrained-output errors, absent-T1 HumanReview, and protection skip behavior. (E-14_browser_e2e_agents.md:100-129)
- **E14-027 [E][TEST/FAIL]** Three executable smokes verify below-threshold short-circuit, T1-fail dominance, and protected-page skip without bypass. (E-14_browser_e2e_agents.md:130-140)
- **E14-028 [E][TEST]** Acceptance requires AX snapshot-to-disk, the exact axe severity policy, fallback-only Stagehand cache, schema rejection to HumanReview, SSIM threshold behavior, and the asymmetric matrix. (E-14_browser_e2e_agents.md:143-150)
- **E14-029 [E][TEST/INT]** Acceptance additionally requires API-plus-cURL deployment polling, first-200 smoke, multi-context mocked notifications, protection skip, at least nine actions, three commands, the panel extensions, and three configuration tables. (E-14_browser_e2e_agents.md:150-156)
- **E14-030 [E][TEST/CON]** Acceptance bans NoopSsbNotifier, Box of dynamic Error, and production unwrap. (E-14_browser_e2e_agents.md:155)
- **E14-031 [E][TEST/DET]** AX parser tests cover well formed, missing role, empty tree, and malformed JSON; axe tests cover critical/serious/moderate/minor/clean; hybrid tests cover below/above/equal threshold. (E-14_browser_e2e_agents.md:158-162)
- **E14-032 [E][TEST/FAIL]** Veto tests cover five T1/T2 combinations, VLM tests cover full/free/partial schema, Stagehand tests cover miss/hit/stale, and CF tests cover challenge/widget/clean/JA4. (E-14_browser_e2e_agents.md:162-165)
- **E14-033 [E][TEST/WF]** Deployment tests cover 200, 404 retry, timeout, and API error; SaaS tests cover happy path, captured webhook, and context-isolation regression. (E-14_browser_e2e_agents.md:166-167)
- **E14-034 [E][CAP/STATUS]** The file inventory creates eight implementation modules plus a reference journey and modifies media aggregation, Playwright, SSB, config, dashboard/commands, and temporary task-runner spawn wiring. (E-14_browser_e2e_agents.md:169-185)
- **E14-035 [E][A/TEST]** Threshold 0.02 is explicitly heuristic; planned follow-up is a 50-app measurement toward the report’s requested 500-app cost curve. (E-14_browser_e2e_agents.md:204-205)
- **E14-036 [E][A/TEST]** The claimed fourfold snapshot token saving is vendor-task-specific and requires an ELAI A/B before publicizing it. (E-14_browser_e2e_agents.md:206)
- **E14-037 [E][A/FAIL]** Stagehand cache survival is uncalibrated and must be measured over 30 regeneration runs. (E-14_browser_e2e_agents.md:207)
- **E14-038 [E][SEC/CFG/FAIL]** Air-gapped mode must disable deploy preview and emit deploy_preview_disabled_air_gapped. (E-14_browser_e2e_agents.md:208)
- **E14-039 [E][CFG/STATUS]** InternVL Mode B does not fit the 12GB host and is cloud-only deferred; constrained decoding must be validated against Qwen Mode A first. (E-14_browser_e2e_agents.md:209)
- **E14-040 [E][DEP/SEC]** Tier-C must allow node/npx/Chromium through E-15, and CF signatures require quarterly source refresh. (E-14_browser_e2e_agents.md:210-211)
- **E14-041 [E][STATUS/DEP]** Follow-ups calibrate 200-app VLM probabilities, migrate the worker after C-16, and measure DNS/edge delay between provider readiness and first HTTP 200. (E-14_browser_e2e_agents.md:213-217)

### E-16_vlm_image_validation.md — 181/181 lines

- **E16-001 [E][CAP/STATUS]** E-16 is explicitly a delta on shipped B-24: add AXTree YAML, anti-sycophancy prompting, swap-order evaluation, and bounded LPIPS convergence without re-authoring the existing tier architecture. (E-16_vlm_image_validation.md:1-3)
- **E16-002 [E][DEP/STATUS]** B-24 and B-37 are shipped prerequisites, C-7 is planned, and E-17/C-7/C-8 are parallel-safe; D-13 is a sibling cross-family design rather than being pre-empted. (E-16_vlm_image_validation.md:4-6,19-26)
- **E16-003 [E][STATUS/TEST]** Existing B-24 is described as already having pHash/SSIM/LPIPS, axe, OCR/media tooling, two VLM modes, an RCE loop, and 113 passing tests, but lacking the four new controls. (E-16_vlm_image_validation.md:8-17)
- **E16-004 [E][FAIL/SEC]** The motivating failures are leading-prompt sycophancy, greater-than-ten-point pairwise position bias, missing deterministic AXTree snapshots, and unbounded/no-convergence RCE iteration. (E-16_vlm_image_validation.md:10-17)
- **E16-005 [E][CAP/DATA/DET]** AXTreeAdapter calls page.ariaSnapshot after web rendering, stores YAML in the Tier 1 result, compares by render task ID, emits axtree_mismatch on drift, targets under 100 ms CPU-only, and is skipped for non-web adapters. (E-16_vlm_image_validation.md:39-41)
- **E16-006 [E][CAP/DATA]** neutralize_query removes leading executor claims while preserving the semantic request; contrastive_decode compares neutral and leading variants and returns agree/divergent plus divergence_score. (E-16_vlm_image_validation.md:43)
- **E16-007 [E][WF/SEC]** Primary Tier 2 always uses neutral framing; the leading variant is confined to contrastive comparison. (E-16_vlm_image_validation.md:43)
- **E16-008 [E][WF/FAIL/OBS]** Before/after candidate comparison runs in [A,B] and [B,A] order; only agreement can pass, disagreement enters HumanReview and emits swap_order_disagreement. (E-16_vlm_image_validation.md:45)
- **E16-009 [E][WF/CFG/FAIL]** RCE converges when LPIPS(previous,current) is below 0.02, the critic reports no actionable differences, or iteration reaches the hard cap of five; the existing CPU ONNX LPIPS model is reused. (E-16_vlm_image_validation.md:47)
- **E16-010 [E][WF/FAIL]** InternVL3.5 is a second opinion when Qwen’s verdict has low log probability or sycophancy score exceeds a threshold; both must agree to pass and disagreement routes to HumanReview. (E-16_vlm_image_validation.md:49)
- **E16-011 [E][CFG/INT]** The second opinion reuses B-37 runtime configuration and adds second_opinion_backend rather than introducing a runtime implementation. (E-16_vlm_image_validation.md:49)
- **E16-012 [E][DATA/FAIL/OBS]** XGrammar must constrain VlmValidationOutput; a fuzz integration test proves mask compliance, while any SGLang fallback that ignores the schema emits vlm_schema_unconstrained. (E-16_vlm_image_validation.md:51)
- **E16-013 [E][WF/SEC/OBS]** sycophancy_probe reruns every user-aligned Tier 2 claim with inverted framing; agreement is robust, divergence emits sycophancy_flag and quarantines to HumanReview. (E-16_vlm_image_validation.md:53)
- **E16-014 [E][CAP/DATA]** Dashboard row schema preserves AXTree, sycophancy_flag, swap_order_disagreement, and convergence_history as separate fields, and MediaValidationDrawer adds three display fields rather than collapsing tier evidence. (E-16_vlm_image_validation.md:55)
- **E16-015 [E][TEST/CFG]** cloud_mode_live is to be unignored after neutralization, swap-order, and probe land, with a distinct needs-real-vlm ignored guard preventing accidental CI cloud calls. (E-16_vlm_image_validation.md:57)
- **E16-016 [E][TEST/FAIL]** Twenty identical renders measure AXTree YAML drift; above one percent triggers fallback to an axe-only deterministic gate. (E-16_vlm_image_validation.md:59)
- **E16-017 [E][CAP/INT]** Exported symbols are AXTreeAdapter, neutralize_query, contrastive_decode, lpips_delta_converged, SycophancyVerdict, AxTreeResult, and four SSB variants: SycophancyFlag, SwapOrderDisagreement, AxtreeMismatch, VlmSchemaUnconstrained. (E-16_vlm_image_validation.md:61-73)
- **E16-018 [E][INT/WF]** validate/Web::render/RuntimeAdapter/dashboard row schema are consumed; task-runner media_validation_phase and get_media_validation_recent are the two declared reachability paths. (E-16_vlm_image_validation.md:74-81)
- **E16-019 [E][CON]** The source bans TODO/unimplemented in sycophancy, lingering ignore on cloud_mode_live, iteration counts over five, and pixel difference as the primary gate. (E-16_vlm_image_validation.md:82-87)
- **E16-020 [E][TEST/OBS]** The cloud TODO-app smoke expects no AX mismatch on a clean run, sycophancy-probe evidence, no more than five RCE iterations, an axtree.yaml artifact, and monotonically decreasing LPIPS deltas. (E-16_vlm_image_validation.md:88-91)
- **E16-021 [E][TEST]** Deterministic checks require AX integration, both anti-sycophancy functions, MAX_RCE_ITERATIONS=5, all four event variants, no ignored cloud_mode_live attribute, and smoke counts of at least one probe/no more than five RCE/no clean AX mismatch. (E-16_vlm_image_validation.md:93-121)
- **E16-022 [E][TEST]** Acceptance requires YAML for every web render, neutral primary prompts, swap agreement, five-iteration convergence, conditional second opinion, enforced JSON mask, inverted-frame quarantine, a passing live cloud test, at most one-percent AX drift, and no production unwrap/expect. (E-16_vlm_image_validation.md:123-134)
- **E16-023 [E][TEST/DET]** Tests require YAML round-trip and sub-100-ms timing, fifty leading-prompt neutralizations, A/B disagreement routing, an LPIPS sequence converging at iteration four plus seven-step cap fixture, and a four-event contract guard. (E-16_vlm_image_validation.md:136-142)
- **E16-024 [E][CAP/STATUS]** The file inventory creates sycophancy and AX modules plus a decision doc, and modifies tier dispatch, feedback loop, aggregation, web rendering, SSB, Tauri schema, and drawer display. (E-16_vlm_image_validation.md:144-155)
- **E16-025 [E][A/FAIL]** AX stability remains an open gate: hydration drift may force axe-only fallback. (E-16_vlm_image_validation.md:168-170)
- **E16-026 [E][A/SEC]** Neutral prompts may not drive Qwen-family sycophancy below five percent; model-level contrastive decoding may be deferred upstream. (E-16_vlm_image_validation.md:171)
- **E16-027 [E][CFG/FAIL]** InternVL Q4 is borderline on 24GB and must be sequenced rather than co-resident with Qwen through PerRoleRuntime. (E-16_vlm_image_validation.md:172)
- **E16-028 [E][FAIL/CFG]** XGrammar schema compilation must be warmed at sidecar boot because first-call CPU mask generation can stall for seconds. (E-16_vlm_image_validation.md:173)
- **E16-029 [E][CON]** Existing pixel metrics remain only flake suppressors, never the primary gate. (E-16_vlm_image_validation.md:174)
- **E16-030 [E][DEP/STATUS]** D-13 may consume the cross-family agreement primitive, D-14 may reuse the neutralization module after promotion to a shared crate, and AX/VLM escalation ROC work is deferred to a C-20 metrics task. (E-16_vlm_image_validation.md:176-181)

### E-17_rag_retrieval_quality.md — 251/251 lines

- **E17-001 [E][STATUS/CAP]** The completion rollup claims fifteen surfaces shipped on V2: knowledge-model doctor, DEEP cap/RRF floor/constants, NL-only HyDE gate/env, four RRF/DEEP/HyDE actions, ReRankerModel variant selection, three-retriever RRF, five config groups, four dashboard commands/panels, a RAG eval harness, and three tests. (E-17_rag_retrieval_quality.md:3-21)
- **E17-002 [E][STATUS/CAP/TEST]** It claims this session shipped contextual_retrieval with heuristic floor and LLM-template scaffold plus thirteen tests, cAST recursive split/sibling merge with four language adapters and ten tests, a disabled late-interaction flag, three missing contract/load/smoke tests, and an optional-reranker doctor footer plus BGE staging recipe. (E-17_rag_retrieval_quality.md:23-32)
- **E17-003 [E][STATUS/FAIL]** Jina is explicitly pending because no official ONNX was found: ReRankerModel defaults to MsMarcoMiniLmL12, load failure would otherwise leave results unranked, and BGE-v2-m3 is merely opt-in stageable. (E-17_rag_retrieval_quality.md:34-40)
- **E17-004 [E][STATUS/INT]** ContextualChunkEmitted and ChunkRegenSkipped producers are ready but their SSB variants are deferred because SCHEMA_VERSION 103 is locked. (E-17_rag_retrieval_quality.md:42-45)
- **E17-005 [E][STATUS]** Three declared spec drifts are resolved as Jina-pending, shipped cAST/contextual modules, and a documented disabled late-interaction flag whose producer remains deferred. (E-17_rag_retrieval_quality.md:47-50)
- **E17-006 [E][CAP/STATUS]** The objective prioritizes verifying/staging three ONNX models before any technique, then contextual ingest, reranker modernization, and DEEP cap/RRF gating to restore MRR monotonicity. (E-17_rag_retrieval_quality.md:53-57)
- **E17-007 [E][OBS/FAIL]** Baseline measurements are DEEP MRR 0.206 versus STANDARD 0.575; diagnosed causes are candidate dilution, missing ONNX artifacts causing BM25-only degradation, unwired code_vec, and HyDE noise on precise code queries. (E-17_rag_retrieval_quality.md:59-65)
- **E17-008 [E][DEP/STATUS]** B-1 is done, B-4 is done but code_vec is opt-in/unqueried and model staging absent, C-8 will surface produced rows, Phase 3K is superseded for HyDE/reranking, and D-11 is orthogonal. (E-17_rag_retrieval_quality.md:67-74)
- **E17-009 [E][CAP/OBS/FAIL]** elai doctor knowledge-models loads the knowledge engine, checks cross-encoder/GraphCodeBERT/Granite under models_dir_path, stages missing files, and reports size plus tokenizer hash for present files before other work. (E-17_rag_retrieval_quality.md:88-90)
- **E17-010 [E][CAP/DATA/DET]** Contextual ingest generates a 50–100-token file/class/purpose prefix locally with prompt caching, prepends it before BM25 and embeddings, stores it with the chunk, and regenerates only when the blake3 source hash changes. (E-17_rag_retrieval_quality.md:92)
- **E17-011 [E][CFG/CAP]** The planned reranker enum supports JinaV3, BgeV2M3, and legacy MsMarcoMiniLmL12; model selection is configurable per task and the task text says Jina should become default only once staging is confirmed. (E-17_rag_retrieval_quality.md:94,112-125)
- **E17-012 [E][CAP/CFG/OBS]** DEEP caps candidates at 100, drops RRF scores below 0.005 with k=60, exposes both values in knowledge.deep config, and emits DeepPoolCapped and RrfQualityGateFired per query. (E-17_rag_retrieval_quality.md:96)
- **E17-013 [E][WF/CFG/OBS]** HyDE is enabled only for natural-language classification, disabled for PL/code symbols under default-true ELAI_HYDE_NL_ONLY, and emits HydeSkippedCodeQuery. (E-17_rag_retrieval_quality.md:98)
- **E17-014 [E][CAP/DATA]** cAST retains tree-sitter but replaces chunk emission with recursive split plus sibling merge. (E-17_rag_retrieval_quality.md:100)
- **E17-015 [E][CAP/CFG/OBS]** Three-ranker RRF must query BM25, code_vec, and text_vec with configurable defaults 1.0/1.0/0.8 and emit component weights per query. (E-17_rag_retrieval_quality.md:102)
- **E17-016 [E][CFG/STATUS]** PLAID/FastPlaid is only optional follow-up wiring; knowledge.late_interaction.enabled defaults false and may never be enabled if the simpler fixes restore monotonicity. (E-17_rag_retrieval_quality.md:104,243,245-251)
- **E17-017 [E][CAP/OBS]** KnowledgeDrawer is intended to show RRF weights, cap/floor gates, HyDE skips, contextual emits, and regen skips; C-8 owns commands while E-17 owns production rows. (E-17_rag_retrieval_quality.md:106)
- **E17-018 [E][TEST/DATA]** The private harness uses a 135-file DKG and 50-query goldset, reports R@5/R@10/MRR by STANDARD/DEEP before and after each task, and targets STANDARD at least 0.55, DEEP greater than STANDARD, non-regressed HyDE-NL, and cAST Recall@5 lift. (E-17_rag_retrieval_quality.md:108)
- **E17-019 [E][CAP/INT]** Produced symbols include contextualize_chunk, CastChunker, ReRankerModel, deep_pool_cap, rrf_quality_gate, should_use_hyde, doctor probe, and six named SSB variants. (E-17_rag_retrieval_quality.md:110-125)
- **E17-020 [E][INT/WF]** Existing query, reranker/degradation/trace/path primitives are consumed; CLI doctor, task-runner knowledge query, and C-8 dashboard query trace are the declared entry paths. (E-17_rag_retrieval_quality.md:126-135)
- **E17-021 [E][CON]** The original stub ban forbids MiniLM as default after task 3, HyDE on code, DEEP pools over 100 without explicit override, non-contextual new chunks after task 2 unless the source hash is unchanged, and production unwrap. (E-17_rag_retrieval_quality.md:136-142)
- **E17-022 [E][TEST/OBS]** The research smoke expects all three RRF weights, a capped count at most 100, a code-query HyDE skip, DEEP at least STANDARD, and a trace populated by BM25/code/text/rerank. (E-17_rag_retrieval_quality.md:143-146)
- **E17-023 [E][TEST]** Deterministic checks require a healthy doctor with three nonempty model files/tokenizer hashes, contextual integration, Jina-or-BGE default binding, constants 100 and 0.005, HyDE gate, cAST, two harness files, and a run restoring monotonicity. (E-17_rag_retrieval_quality.md:148-182)
- **E17-024 [E][TEST]** Acceptance requires staged models, every new chunk contextualized with hash-stable skip, modern default reranker with MiniLM legacy access, cap/floor events, code HyDE off, cAST active, all RRF components, and monotonicity. (E-17_rag_retrieval_quality.md:184-193)
- **E17-025 [E][TEST/C]** The cAST acceptance line demands at least four Recall@5 points but parenthetically calls at least two conservative; PLAID must exist only behind its disabled flag and chunk hashes must use blake3. (E-17_rag_retrieval_quality.md:194-196)
- **E17-026 [E][TEST/DET]** Contract tests cover contextual prefix/hash skip, 250-to-100 pool/floor drops, twenty query classes, reproducible reranker scoring over three trials with standard deviation below 1e-6, ten-file cAST lift at least two, and DEEP-over-STANDARD synthetic monotonicity. (E-17_rag_retrieval_quality.md:198-205)
- **E17-027 [E][CAP/STATUS]** The planned file inventory creates contextual, cAST, deep-gate, eval, and decision artifacts and modifies reranker/pipeline/HyDE/trace/doctor/SSB/config. (E-17_rag_retrieval_quality.md:207-221)
- **E17-028 [E][A/FAIL]** Missing ONNX may be packaging rather than code; repair may be installer download or Git-LFS, and doctor must distinguish it. (E-17_rag_retrieval_quality.md:236-238)
- **E17-029 [E][A/FAIL]** If cAST fails to reach a two-point private-corpus lift, retain the legacy chunker behind a flag. (E-17_rag_retrieval_quality.md:239)
- **E17-030 [E][A/PERF]** Contextual ingest adds one sidecar call per chunk; prompt caching mitigates but does not remove ingest-time cost, which must be measured before ship. (E-17_rag_retrieval_quality.md:240)
- **E17-031 [E][CON]** BM42 is explicitly rejected. (E-17_rag_retrieval_quality.md:241)
- **E17-032 [E][TEST/FAIL]** Query-type classification must reach at least 95-percent F1; below that, HyDE falls back to off everywhere. (E-17_rag_retrieval_quality.md:242)
- **E17-033 [E][STATUS/DEP]** Follow-ups cover optional late interaction, E-19 consumption, C-8 visualization, a future ModernColBERT tier, and deferred HippoRAG concept-seeded PageRank. (E-17_rag_retrieval_quality.md:245-251)

### E-34_rust_ai_ecosystem_2026.md — 231/231 lines

- **E34-001 [E][STATUS/CON]** The primary deliverable corrects a hard-rule/code contradiction: ort stable-only is replaced with permission for an exact-pinned 2.x release candidate and an explicit ban on abandoned 1.13.1. (E-34_rust_ai_ecosystem_2026.md:1-4,8-10)
- **E34-002 [E][DEP/STATUS]** E-34 must precede planned B-16, extends shipped B-37, and is parallel-safe with E-31/E-32/E-33/E-35. (E-34_rust_ai_ecosystem_2026.md:3-6,16-20)
- **E34-003 [E][CON]** The ecosystem disposition is rmcp canonical for a future MCP server, genai as a light multi-provider option, mistralrs replacing archived llm, fastembed as a current ONNX wrapper, swiftide rejected, async-std rejected, rkyv preferred for zero-copy, and postcard retained for bounded IPC. (E-34_rust_ai_ecosystem_2026.md:12-14)
- **E34-004 [E][STATUS/CON]** ELAI currently only consumes MCP; exposing an MCP server is deferred, so this session adopts an rmcp-ready dependency/minimal crate without a server surface. (E-34_rust_ai_ecosystem_2026.md:21)
- **E34-005 [E][CON]** All anti-stale rules other than ort remain unchanged, and swiftide is marked do-not-adopt despite not being in the dependency tree. (E-34_rust_ai_ecosystem_2026.md:22-23)
- **E34-006 [E][CAP/STATUS]** CLAUDE.md must replace the old ort sentence with exact rc-track guidance citing current rc.12 and the abandoned stable; this amendment is ordered before dependency adoption. (E-34_rust_ai_ecosystem_2026.md:39-41,209-211)
- **E34-007 [E][OBS/STATUS]** outdated-tech-audit.md gains a dated ort appendix with download/caller/no-stable evidence. (E-34_rust_ai_ecosystem_2026.md:43)
- **E34-008 [E][STATUS/DEP]** B-16’s summary is corrected to preserve =2.0.0-rc.12 at knowledge/Cargo.toml:50 and reevaluate only on a stable 2.0 release. (E-34_rust_ai_ecosystem_2026.md:45)
- **E34-009 [E][CAP/CFG]** GenAiAdapter is a new RuntimeAdapter variant covering Anthropic, Gemini, Groq, xAI, Ollama, DeepSeek, and Copilot under pinned genai 0.6.0-beta.20; existing CloudGroundingClient remains until its retry/streaming behavior is audited. (E-34_rust_ai_ecosystem_2026.md:47-49)
- **E34-010 [E][SEC/DEP]** Before adding genai, cargo search must confirm a recent 0.6 beta and its identity; workspace-versus-crate dependency placement is discovered first. (E-34_rust_ai_ecosystem_2026.md:49)
- **E34-011 [E][CAP/STATUS]** rmcp 1.7.0 is added either to orchestrator or a new mcp crate with a real re-export and compile test, while tool/server design remains a follow-up. (E-34_rust_ai_ecosystem_2026.md:51)
- **E34-012 [E][CAP/STATUS]** The initial GenAiAdapter wires one Anthropic completion method; remaining methods may use registered unimplemented markers explicitly owned by E-34.B rather than pretending complete migration. (E-34_rust_ai_ecosystem_2026.md:53)
- **E34-013 [E][CON/FAIL]** Any archived llm dependency is removed and replaced with mistralrs 0.8.1 or candle-core 0.10.2; if absent, that absence is recorded in the decision doc. (E-34_rust_ai_ecosystem_2026.md:55)
- **E34-014 [E][OBS/STATUS]** cargo deny/audit records actual rkyv/postcard versions; rkyv expansion is only a future measured choice and postcard remains the bounded-IPC format. (E-34_rust_ai_ecosystem_2026.md:57)
- **E34-015 [E][TEST/CAP]** insta and proptest demos snapshot synthetic SSB payloads and property-test FSM replay; crate identity/currency is checked before adoption. (E-34_rust_ai_ecosystem_2026.md:59)
- **E34-016 [E][CON]** CLAUDE.md explicitly rejects swiftide as dormant and directs agent-framework needs toward rig-core or hand-rolled code. (E-34_rust_ai_ecosystem_2026.md:61)
- **E34-017 [E][STATUS]** The decision doc records the ort contradiction and defers rkyv expansion, MCP server design, full genai migration, and rig-core adoption. (E-34_rust_ai_ecosystem_2026.md:63)
- **E34-018 [E][CFG/STATUS/CON]** oxc is optional only as a second-pass TS/JS refinement, tree-sitter remains canonical, and work is deferred if integration exceeds half a day. (E-34_rust_ai_ecosystem_2026.md:65)
- **E34-019 [E][CAP/INT]** Produced artifacts are GenAiAdapter, rmcp re-export crate, two demo tests, decision record, two rule/audit amendments, and B-16 correction; CloudGroundingClient and the ort pin remain in place. (E-34_rust_ai_ecosystem_2026.md:67-83)
- **E34-020 [E][WF/TEST]** GenAiAdapter is reachable through runtime selection, rmcp through a future server, and the demos through workspace tests; the observable proof has no SSB requirement and is artifact/build based. (E-34_rust_ai_ecosystem_2026.md:84-100)
- **E34-021 [E][CON/SEC]** Archived llm, swiftide, and async-std must be absent; unimplemented genai methods need registry ownership; every new crate needs search/license/recency evidence; production unwrap and non-blake3 cache keys are banned. (E-34_rust_ai_ecosystem_2026.md:88-95)
- **E34-022 [E][TEST]** Deterministic checks prove new and old rule text, swiftide rejection, audit appendix, B-16 correction, unchanged ort pin, genai/rmcp modules/deps, absent llm, demo tests, and a green workspace build. (E-34_rust_ai_ecosystem_2026.md:102-146)
- **E34-023 [E][TEST]** Acceptance repeats the rule/audit/B-16/unchanged-pin gates and requires exact genai beta, one working adapter method plus registered stubs, rmcp linking, demo tests, absent llm/async-std, green build, decision coverage, dependency evidence, no unwrap, and blake3 cache keys. (E-34_rust_ai_ecosystem_2026.md:148-165)
- **E34-024 [E][TEST/DET]** Tests cover offline mocked GenAi completion, rmcp import/link, snapshot stability, 100 random FSM sequences, and a lint contract requiring new ort wording and rejecting the old wording. (E-34_rust_ai_ecosystem_2026.md:167-173)
- **E34-025 [E][CAP/STATUS]** The file inventory modifies CLAUDE.md, the audit, B-summary, and two dependency manifests, and creates adapter, mcp crate, five tests, and the decision doc. (E-34_rust_ai_ecosystem_2026.md:175-190)
- **E34-026 [E][A/STATUS]** CloudGroundingClient capability is unaudited; if it already has retry/backoff/SSE behavior, genai is cleanup rather than a new capability. (E-34_rust_ai_ecosystem_2026.md:212)
- **E34-027 [E][CFG/STATUS]** genai migration is deliberately two-phase because beta versions can break; exact-beta pin rationale must be documented and reassessed at each bump. (E-34_rust_ai_ecosystem_2026.md:213,217)
- **E34-028 [E][STATUS]** MCP server tooling remains a separate design problem rather than an implicit consequence of adding rmcp. (E-34_rust_ai_ecosystem_2026.md:214)
- **E34-029 [E][TEST/STATUS]** rkyv expansion proceeds only if decode latency beats postcard by more than twofold. (E-34_rust_ai_ecosystem_2026.md:215)
- **E34-030 [E][DEP/STATUS]** B-16 must enforce the amended rule and ship after E-34. (E-34_rust_ai_ecosystem_2026.md:216,222-230)
- **E34-031 [E][A/STATUS]** Swiftide rejection is preemptive and may be revisited if its organization produces a successor. (E-34_rust_ai_ecosystem_2026.md:218)
- **E34-032 [E][CON]** CLAUDE.md edits must preserve all unrelated anti-stale constraints. (E-34_rust_ai_ecosystem_2026.md:219-220)
- **E34-033 [E][STATUS/DEP]** Deferred follow-ups are full genai migration, MCP server design, measured rkyv expansion, rig-core evaluation, and the oxc decision; B-37 hosts the adapter and C-13 is independent. (E-34_rust_ai_ecosystem_2026.md:222-231)

### E-39_cost_token_economics.md — 258/258 lines

- **E39-001 [E][CAP/SEC]** The objective converts soft task/day spend warnings into a hard cooperative abort, adds self-verification, request/dollar velocity limits, local difficulty routing, prompt-cache discipline, and three low-cost/free runtime routes. (E-39_cost_token_economics.md:1-3)
- **E39-002 [E][DEP/STATUS]** E-39 extends planned B-27 and C-5 and shipped B-37; it is parallel-safe with E-36/E-37/E-38/E-40 with their costs/legal posture cross-referenced. (E-39_cost_token_economics.md:4-6)
- **E39-003 [E][SEC/FAIL]** The safety premise is that telemetry alone cannot stop wallet loops or retry storms; the system must refuse subsequent calls and terminate the active task. (E-39_cost_token_economics.md:8-10,21-25)
- **E39-004 [E][CFG/OBS]** The plan records May-2026 price/cache assumptions, a five-minute cache TTL hazard, local marginal-cost assumptions, and claimed cascade savings; two paper identifiers are explicitly marked for verification. (E-39_cost_token_economics.md:12-19)
- **E39-005 [E][DEP/STATUS]** B-27 supplies mode/spend schema, C-5 supplies cap errors/warning default/retry cap/estimator/BYOE, C-46 may supply the classifier, B-37 hosts backends, and C-4 hosts related spend display. (E-39_cost_token_economics.md:28-35)
- **E39-006 [E][WF/FAIL/OBS]** When task spend exceeds its cap, abort_task_on_cap_breach waits for the current paid call, records its cost, drops pending writes, returns a partial result marked aborted_by_spend_cap, and emits TaskAbortedBySpendCap with spent/cap/last_action. (E-39_cost_token_economics.md:57-59)
- **E39-007 [E][CFG/DATA]** Per-role caps are configured for executor, auditor, scope_planner, and memory_writer and tracked in a task-context role-to-spend map; the proposed starting relationship is auditor at twice executor pending user feedback. (E-39_cost_token_economics.md:61,251)
- **E39-008 [E][CAP/CFG/FAIL]** VelocityCapEnforcer maintains OneMin, TenMin, OneHour, and TwentyFourHour rolling windows over requests and dollars; a breach aborts and emits window/count/limit evidence. (E-39_cost_token_economics.md:63,81-85)
- **E39-009 [E][WF/DATA]** Before hybrid cloud escalation, local Qwen verifies failed output into Continue, Retry, or Escalate JSON; Continue accepts it, Retry makes one more local attempt, and Escalate calls cloud. (E-39_cost_token_economics.md:65)
- **E39-010 [E][OBS/TEST]** Every AutoMix decision emits AutomixVerifierVerdict, and acceptance targets more than half of failed-output cases avoiding cloud on a thirty-step ablation. (E-39_cost_token_economics.md:65,175-180)
- **E39-011 [E][CAP/DATA]** Cost routing combines a local [0,1] difficulty score from task trigrams, touched-symbol centrality, and reference-task cosine with remaining budget. (E-39_cost_token_economics.md:67)
- **E39-012 [E][WF/CFG/SEC]** Routing is deterministic by matrix: easy routes local regardless of budget; hard+ample uses Sonnet; hard+tight uses Haiku or DeepSeekFlash; very-hard+critical uses Opus only with explicit confirmation. (E-39_cost_token_economics.md:67)
- **E39-013 [E][OBS]** CostRouterDecision records difficulty, remaining budget, selected backend, and rationale before every cloud dispatch. (E-39_cost_token_economics.md:67)
- **E39-014 [E][CFG/CAP]** Every cloud request marks system prompt, project spec, repository manifest, and tool schema cacheable while keeping variable content suffix-only; ttl accepts 5min or 1hour and defaults to 5min. (E-39_cost_token_economics.md:69)
- **E39-015 [E][OBS/TEST]** MonitorWindow exposes per-session cache hit rate with a target above seventy percent. (E-39_cost_token_economics.md:69)
- **E39-016 [E][CAP/SEC]** RuntimeAdapter gains DeepSeekV4Flash, CerebrasFree, and GroqFree only after official endpoint/model/date verification; router uses them for tight-budget cases. (E-39_cost_token_economics.md:71)
- **E39-017 [E][CFG/FAIL/OBS]** Free-tier usage is counted per backend in UTC days, warns at eighty percent, refuses at or beyond ninety-five percent, and emits Approaching/Exceeded evidence. (E-39_cost_token_economics.md:73)
- **E39-018 [E][DATA/OBS]** Pre-call estimates become p50/p95 bands and are stored beside actual cost; an actual-to-estimate ratio above 2.0 or below 0.5 emits CostEstimateInaccurate. (E-39_cost_token_economics.md:75)
- **E39-019 [E][CFG/SEC/OBS]** Default layered caps are a $50/day soft alert, $100/day hard cutoff, and $1000/month typed-confirmation manual override; elai cloud override-cap emits session, amount, and confirmation string. (E-39_cost_token_economics.md:77)
- **E39-020 [E][CAP/DATA]** Exported APIs include AbortDecision/TaskAbortReason, VelocityCapEnforcer/VelocityWindow, AutomixVerdict, route decision variants, QuotaStatus, three Backend variants, override CLI, and eight SSB actions. (E-39_cost_token_economics.md:79-103)
- **E39-021 [E][INT/WF]** cloud_dispatch runs cost router/free-tier checks before a call, velocity checks per call, and ceiling/abort checks afterward; CLI shows override/status and three dashboard commands expose cap/router/AutoMix state. (E-39_cost_token_economics.md:104-115)
- **E39-022 [E][CON/SEC]** Hard caps cannot be warn-only, estimates cannot be point values, cloud calls cannot bypass logged routing, and cost paths prohibit production unwrap. (E-39_cost_token_economics.md:116-120)
- **E39-023 [E][CON]** The plan bans LLM gateways, all-Opus defaults, Batch API for interactive work, and one-hour cache writes without at least two reads. (E-39_cost_token_economics.md:121-124)
- **E39-024 [E][CAP/OBS]** CostEnforcementPanel shows velocity counters, free quota, AutoMix savings, estimate accuracy, and layered-cap state. (E-39_cost_token_economics.md:125)
- **E39-025 [E][TEST/OBS]** A 100-dispatch/60-second synthetic storm must trip a minute velocity cap and possibly spend cap, retain a partial marked result, log routing for each dispatch, and bound total burn. (E-39_cost_token_economics.md:126-129)
- **E39-026 [E][TEST]** Deterministic checks require abort APIs, four windows, pre-escalation AutoMix, five route variants, three backends, two quota events, four cache_control sites, eight SSB variants, $100/day config, and override CLI. (E-39_cost_token_economics.md:131-173)
- **E39-027 [E][TEST/FAIL]** Acceptance requires cooperative spend abort, enforced role caps, four velocity windows, logged deterministic routing, and the measured AutoMix threshold. (E-39_cost_token_economics.md:175-181)
- **E39-028 [E][TEST/CFG]** Acceptance requires cache marks on all four stable blocks plus the hit-rate target, verified backend registrations, 80/95 quota behavior, estimate ratio evidence and p50/p95 shape, layered defaults, and typed override. (E-39_cost_token_economics.md:182-187)
- **E39-029 [E][TEST/CON]** Acceptance repeats the no-all-Opus/no-interactive-Batch/no-unjustified-one-hour-cache rules and bans new gateway dependencies. (E-39_cost_token_economics.md:188-189)
- **E39-030 [E][TEST/FAIL]** Spend and velocity contract fixtures use a $0.05 task and 50 calls against 10/min, verifying abort, partial result, and typed rows. (E-39_cost_token_economics.md:191-195)
- **E39-031 [E][TEST/WF]** AutoMix’s ten fixtures expect 5 Continue, 3 Retry, 2 Escalate; router tests cover sixteen difficulty-by-budget cells; cache tests inspect four cache blocks in five calls. (E-39_cost_token_economics.md:195-197)
- **E39-032 [E][TEST/CFG]** Quota fixtures warn at 11,520/14,400 and reject the next call at 13,680/14,400; estimate fixtures distinguish 3x from 1.2x; role fixtures distinguish executor $0.05 from auditor $0.10. (E-39_cost_token_economics.md:198-200)
- **E39-033 [E][TEST/SEC]** Override requires the typed string, and a 100-decision default-routing test requires fewer than ten percent Opus routes. (E-39_cost_token_economics.md:201-202)
- **E39-034 [E][CAP/STATUS]** The file inventory creates four orchestration modules, the CostEnforcementPanel, ten contract tests, and a decision record while changing spend/client/runtime/task/SSB/CLI/config/Tauri surfaces. (E-39_cost_token_economics.md:204-220)
- **E39-035 [E][A/STATUS]** Two pivotal quantitative citations—agent token variance and AutoMix savings—must be verified before final adoption; fallback wording/source strength is prescribed if verification fails. (E-39_cost_token_economics.md:244-247)
- **E39-036 [E][SEC/A]** DeepSeek’s export-control/GDPR posture must be surfaced to the user before restricted-jurisdiction routing. (E-39_cost_token_economics.md:248)
- **E39-037 [E][FAIL/STATUS]** If Cerebras revokes free service, the guard must fail closed and reroute to DeepSeek or Sonnet. (E-39_cost_token_economics.md:249)
- **E39-038 [E][TEST/STATUS]** AutoMix is opt-in if its added local verifier call does not remove more than half of prospective cloud calls in measurement. (E-39_cost_token_economics.md:250)
- **E39-039 [E][SEC/DEP]** All three new provider identities are mandatory anti-slopsquatting checks; E-37 safety-classifier and E-36 document-parser costs receive distinct role buckets. (E-39_cost_token_economics.md:252-253)
- **E39-040 [E][DEP/FAIL]** If C-46 is absent, cost routing starts from ten hand-curated references and later migrates to C-46’s corpus. (E-39_cost_token_economics.md:254)
- **E39-041 [E][STATUS/CON]** A full three/four-tier FrugalGPT cascade, C3PO optimization, and PRM spend monitoring remain follow-ups; this E-N implements only a two-tier cascade. (E-39_cost_token_economics.md:256-258)

### E-49_coding_agent_metrics.md — 280/280 lines

- **E49-001 [E][CAP/CON]** The objective replaces single-seed vanity reporting with stable multi-run, cost, horizon, attribution, private-eval, production-revert, safety, awareness, context-position, and high-reliability coverage metrics, and stops headline HumanEval/MBPP/SWE-bench Verified measurement. (E-49_coding_agent_metrics.md:1-3)
- **E49-002 [E][DEP/STATUS]** B-23 TestOutcomeReport, C-50 contract tasks, C-51 polyglot tasks, and B-37 runtime ablation are prerequisites; E-9/E-13/E-46 are parallel or signal suppliers. (E-49_coding_agent_metrics.md:4-6)
- **E49-003 [E][OBS/CON]** The motivating evidence treats single-run variance, public-set leakage, harness dominance, cost-quality Pareto, real PR acceptance/revert, human-time horizons, self-report bias, middle-context decay, and eval awareness as distinct measurement failures. (E-49_coding_agent_metrics.md:8-12)
- **E49-004 [E][DATA/STATUS]** Existing pass_at_1 remains only for compatibility; phase files gain model_id, harness_version, harness_commit_sha, and the credible metric blocks. (E-49_coding_agent_metrics.md:14-21)
- **E49-005 [E][DATA/DET]** StablePass1 stores default n_seeds=5, task_id, per-seed TestOutcomeReport values, blake3 identical-output count over stdout plus diff, pass-rate standard deviation, and stable_score. (E-49_coding_agent_metrics.md:44-46)
- **E49-006 [E][DATA]** stable_score equals pass_rate times one minus twice standard deviation, clamped to [0,1]; stable_pass_1_n5 is additive while pass_at_1 is deprecated. (E-49_coding_agent_metrics.md:46)
- **E49-007 [E][CFG/OBS]** Repeated runs use deterministic seeds with temperature zero or a HAL-recommended schedule and emit StablePass1Aggregated per task. (E-49_coding_agent_metrics.md:46)
- **E49-008 [E][DATA/CAP]** CostPerSolve stores task/runtime/total USD/correct count/dollars-per-correct/tier; A is local, B Haiku-equivalent, C Sonnet-equivalent, D Opus-equivalent. (E-49_coding_agent_metrics.md:48)
- **E49-009 [E][OBS/FAIL]** Phase files store per-tier dollar_per_correct, emit CostPerSolveEmitted, and trip CostPerSolve2xBaseline for B/C/D above twice baseline; absent runtime cost must emit CostPerSolveNoCostData rather than zero. (E-49_coding_agent_metrics.md:48,96-107)
- **E49-010 [E][DATA/WF]** TimeHorizonAt50Pct records frozen pool version, HCAST-style tasks, current 50-percent human-time horizon, and timestamp in metrics/time_horizon.toml on a quarterly cadence. (E-49_coding_agent_metrics.md:50)
- **E49-011 [E][CFG/CON]** The initial horizon pool derives up to fifty tasks from the 31-task Horizon 1.1 set or matched ELAI-curated equivalents, freezes each version, and never substitutes ELAI wall-clock for human time. (E-49_coding_agent_metrics.md:50,264-273)
- **E49-012 [E][CAP/OBS]** TimeHorizonPanel graphs horizon history. (E-49_coding_agent_metrics.md:50)
- **E49-013 [E][DATA/DET]** AttributionTuple records model, harness version/commit, baseline model/harness, and isolated harness/model deltas; the ablation holds one axis fixed while varying the other. (E-49_coding_agent_metrics.md:52)
- **E49-014 [E][FAIL/CFG]** Phase publication refuses any empty model_id/harness_version/harness_commit_sha; baselines are configurable and default to a frozen E-49 landing snapshot; missing both deltas emits a blocking row. (E-49_coding_agent_metrics.md:52)
- **E49-015 [E][SEC/DATA]** metrics/private_eval is never Git-tracked, is manually or later automatically regenerated quarterly, and initially contains 20–50 internal tasks per Rust/Python/TS language. (E-49_coding_agent_metrics.md:54)
- **E49-016 [E][OBS/FAIL]** Every phase reports public and private pass rates; public-minus-private over ten points warns and over twenty points blocks. (E-49_coding_agent_metrics.md:54)
- **E49-017 [E][DATA/DET]** RevertCanary uses a default thirty-day auto-merge window, counts reverted self-mod proposals and revert rate, and computes ninety-day AI-to-human churn using Git numstat partitioned by author. (E-49_coding_agent_metrics.md:56)
- **E49-018 [E][OBS/CON]** Revert rate over 0.12 alerts, target churn is under 1.5x, and the module may read but never mutate Git state. (E-49_coding_agent_metrics.md:56,96-105)
- **E49-019 [E][SEC/TEST]** Prompt-injection safety runs roughly 100 vendored prompts, measures successful exfiltration/override, and targets under five percent for 2-shot and under fifteen percent for 8-shot adaptive attacks. (E-49_coding_agent_metrics.md:58)
- **E49-020 [E][SEC/TEST]** Refusal accuracy uses 150 malicious/benign coding requests, counts correct refusal and correct compliance, targets above 98 percent, runs per release, writes two safety TOMLs, and emits a regression tripwire. (E-49_coding_agent_metrics.md:58)
- **E49-021 [E][DATA/CFG]** EvalAwareness stores run totals/count/rate; a configurable nonempty base regex set detects five named evaluation phrases and warns above ten percent. (E-49_coding_agent_metrics.md:60,104-105)
- **E49-022 [E][DATA/CFG/TEST]** ContextRotProbe defaults to 12,000 tokens, supports up to 25,000, probes positions 0/.25/.5/.75/1, stores per-release results, targets middle recall above seventy percent, and trips below it. (E-49_coding_agent_metrics.md:62)
- **E49-023 [E][DATA]** coverage_at_tau_95 is the fraction of tasks whose stable_score is at least 0.95 and is emitted in the phase coverage block. (E-49_coding_agent_metrics.md:64)
- **E49-024 [E][CON/TEST]** A stdlib-only CI tool fails new phase files containing humaneval_pass_at_1, mbpp_pass_at_1, or swe_bench_verified_pass_at_1, while documentation hits only warn. (E-49_coding_agent_metrics.md:66)
- **E49-025 [E][CAP/INT]** Five panels and five Tauri commands expose stable pass, cost Pareto, human-time horizon, harness/model attribution, and ASR/refusal metrics under a metrics drawer. (E-49_coding_agent_metrics.md:68)
- **E49-026 [E][CAP/DATA]** The produced type surface includes StablePass1, CostPerSolve/RunTier, TimeHorizonAt50Pct, AttributionTuple, RevertCanary, PromptInjectionAsr, RefusalAccuracy, EvalAwareness, ContextRotProbe, and CoverageAtTau. (E-49_coding_agent_metrics.md:70-83)
- **E49-027 [E][INT/WF]** Metrics consume TestOutcomeReport, RuntimeOrchestrator/PerRoleRuntime, E-46 process reward, blake3, and read-only Git; CLI provides run-suite/horizon-update/safety-probe and AUDITING wraps test runs. (E-49_coding_agent_metrics.md:84-95)
- **E49-028 [E][CON]** New modules ban unimplemented and production unwrap; single-seed pass cannot be emitted alone; human-time cannot be substituted; public docs cannot headline the three deprecated suites; modified vendored safety prompts block. (E-49_coding_agent_metrics.md:96-106)
- **E49-029 [E][TEST/OBS]** A ten-task, five-seed Tier-A polyglot smoke expects at least ten stable rows, ten cost rows, one attribution per phase, optional contamination warnings, and populated phase metric/identity/delta fields. (E-49_coding_agent_metrics.md:109-112)
- **E49-030 [E][TEST]** Deterministic checks cover n=5, four tiers, HCAST pool file, required attribution, Git ignore, 0.12 revert threshold, safety types, awareness markers, five positions, Cover@tau, lint self-test, and blake3 without blake2/sha2. (E-49_coding_agent_metrics.md:114-178)
- **E49-031 [E][TEST]** Acceptance requires StablePass formula inputs, runtime-sourced four-tier costs with 2x tripwire, frozen human-time horizon, mandatory two-axis attribution, private gap thresholds, revert/churn, safety, awareness, context rot, Cover@.95, lint, five panels/commands, and TestOutcomeReport compatibility. (E-49_coding_agent_metrics.md:180-194)
- **E49-032 [E][TEST/DET]** Stable tests compare 3-pass/2-fail aggregation and equal-pass-rate variance; cost tests cover four tiers and absent-cost evidence; attribution tests distinguish empty versus populated identity. (E-49_coding_agent_metrics.md:196-202)
- **E49-033 [E][TEST/FAIL]** Canary tests exercise a twenty-point contamination gap, thirteen-percent revert, ASR regression from three to eight percent, one awareness phrase, exactly five positions, and Cover@.95 rejecting lucky single runs. (E-49_coding_agent_metrics.md:203-209)
- **E49-034 [E][TEST/INT]** Five contract files guard stable pass, cost, attribution, revert, and safety surfaces. (E-49_coding_agent_metrics.md:209)
- **E49-035 [E][CAP/STATUS]** The file inventory creates eight metric modules, five panels, lint, three frozen fixture/TOML artifacts, decision doc, and modifies ignore/auditor/config/CLI/Tauri files. (E-49_coding_agent_metrics.md:211-235)
- **E49-036 [E][CFG/A]** Five seeds is provisional; standard error is accumulated and k=3/5/10/20 is recalibrated at the first quarterly review. (E-49_coding_agent_metrics.md:264-266)
- **E49-037 [E][STATUS/A]** Harness/model ablation, first revert baseline, first safety baseline, and public/private gap are unmeasured operator work; the last is blocked on C-51. (E-49_coding_agent_metrics.md:267-270)
- **E49-038 [E][SEC/FAIL]** HCAST and prompt-injection suite redistribution licenses must be verified; if unavailable, use matched curated or regenerated pinned-spec equivalents. (E-49_coding_agent_metrics.md:269-271)
- **E49-039 [E][A]** Regex awareness detection is brittle and may later need a classifier. (E-49_coding_agent_metrics.md:272)
- **E49-040 [E][CFG/FAIL]** Context probes above 12K depend on sidecar capacity. (E-49_coding_agent_metrics.md:273)
- **E49-041 [E][DEP/FAIL]** If RuntimeOrchestrator lacks per-call cost data, the cost metric is blocked and requires a TokenCostMeter extension outside E-49. (E-49_coding_agent_metrics.md:274)
- **E49-042 [E][FAIL]** If self-mod commits lack the expected author, revert partitioning falls back to an [elai-self-mod] commit-message marker. (E-49_coding_agent_metrics.md:275)
- **E49-043 [E][FAIL/CON]** Existing historical Verified references are annotated rather than deleted; phase files fail, docs only warn. (E-49_coding_agent_metrics.md:276)
- **E49-044 [E][STATUS/INT]** The intended credible evidence stack is stable private pass, cost Pareto, real thirty-day revert, and human-time horizon with reproducible harness/model axes; E-46/C-50/C-51/D-3 are downstream or upstream consumers. (E-49_coding_agent_metrics.md:278-280)

### E-53_formal_verification_codegen.md — 304/304 lines

- **E53-001 [E][CAP/CON]** The objective uses formal methods only where latency/assurance justify them: Critical-path Z3 in D-3 consensus, Kani in AUDITING, Verus in PR CI, Dafny IR for Critical Python, and no routine Lean proof loop. (E-53_formal_verification_codegen.md:1-3)
- **E53-002 [E][DEP/STATUS]** D-3 and C-52 are planned prerequisites, CSQ is claimed shipped, and E-51/E-52/E-54/E-55 are parallel-safe; E-53 additively extends D-3 and promotes a narrow subset of C-52. (E-53_formal_verification_codegen.md:4-6,12-20)
- **E53-003 [E][SEC/OBS]** The justification distinguishes fast SMT/bounded/deductive checks from expensive theorem proving and emphasizes that compute-based verification resists plausible-but-wrong reasoning missed by PRMs/static analyzers. (E-53_formal_verification_codegen.md:8-10)
- **E53-004 [E][SEC/CON]** The DAFNYCOMP guard requires the specification to come from CSQ counterfactual, D-13 Adversary, or a human—not the model that authored the implementation. (E-53_formal_verification_codegen.md:10,18)
- **E53-005 [E][CAP/INT]** Z3SmtCheck extends D-3 VerificationResult as a VerifierKind and supplies one of the at-least-two consensus votes without changing existing verifier surfaces. (E-53_formal_verification_codegen.md:14)
- **E53-006 [E][CAP/STATUS]** C-52’s research spike remains reference material while Dafny SpecValidator becomes a narrow production path for auth/money/parser Python. (E-53_formal_verification_codegen.md:15)
- **E53-007 [E][CFG/CON]** Formal checks are gated by CSQ Critical classification to avoid overhead on ordinary edits; the named Critical classes are auth, money, crypto, parser, and untrusted input, with some task-specific subsets narrower. (E-53_formal_verification_codegen.md:16,48-56)
- **E53-008 [E][DEP/SEC]** Z3, kani-verifier, Verus, Dafny, and Lean Copilot are all new dependencies and require identity/license/recency/pinned-release verification. (E-53_formal_verification_codegen.md:20,93-97,117)
- **E53-009 [E][CAP/DATA/FAIL]** Z3 encodes a Rust signature plus independent counterfactual postcondition to SMT-LIB2, times out after five seconds, maps Sat to BLOCK with counterexample, Unsat to a positive vote, and Unknown/Timeout to abstention. (E-53_formal_verification_codegen.md:48-50)
- **E53-010 [E][CFG/STATUS]** If an existing CSQ Critical enum cannot be found, CounterfactualClass Critical/NonCritical is added; D-3 file presence is likewise discovered before extension. (E-53_formal_verification_codegen.md:50,255-256)
- **E53-011 [E][WF/SEC]** Every Z3 specification carries origin and same-model origin is refused before solver execution. (E-53_formal_verification_codegen.md:50)
- **E53-012 [E][CAP/WF]** KaniHarness invokes cargo kani for functions marked with the Kani proof attribute during AUDITING, with a thirty-second cap per harness. (E-53_formal_verification_codegen.md:52)
- **E53-013 [E][DATA/FAIL/OBS]** Kani outcomes are Verified, Falsified(counterexample), Timeout, and NotApplicable; falsification blocks and feeds the fixer, timeout only warns, and verified/falsified/timeout actions are emitted. (E-53_formal_verification_codegen.md:52)
- **E53-014 [E][SEC/DEP]** Kani properties may come from CSQ, Adversary, or a human proof file; the AWS canonical install is version-pinned and identity-checked. (E-53_formal_verification_codegen.md:52)
- **E53-015 [E][CAP/DET]** verus_invariants encodes no-skipped FSM transitions, IPC v2 payload/header length bounds, and total KV-slot bytes not exceeding KV_CACHE_BUDGET_BYTES. (E-53_formal_verification_codegen.md:54)
- **E53-016 [E][WF/FAIL]** verus-check runs in a dedicated PR job with a five-minute cap and gates COMMITTING rather than per-audit execution; violations or persistent timeout fail CI. (E-53_formal_verification_codegen.md:54,115,299)
- **E53-017 [E][WF/DATA/FAIL]** Critical Python flow is independent spec, parallel implementation, thirty-second Dafny verification, then semantic SpecValidator comparison; outcomes are Verified, NotVerified, Timeout, or SpecMismatch and failure rows are emitted. (E-53_formal_verification_codegen.md:56)
- **E53-018 [E][CAP/WF]** The Sonnet SpecValidator agent receives function plus Critical classification, selects Z3/Kani/Verus/Dafny by language/property shape, consumes the independent postcondition, runs the tool, and returns only verdict plus grounding from its own context. (E-53_formal_verification_codegen.md:58)
- **E53-019 [E][CON/SEC]** SpecValidator never authors both spec and proof. (E-53_formal_verification_codegen.md:58)
- **E53-020 [E][CAP/CON/FAIL]** LeanPremiseHint is optional, read-only, asynchronous, runs only after another formal BLOCK, never votes or applies changes, has a sixty-second background budget, and must not delay the BLOCK. (E-53_formal_verification_codegen.md:60)
- **E53-021 [E][TEST/OBS]** A nightly PRMBench canary contains twenty human-validated deceptive diffs; with Z3/Kani enabled, every diff must yield at least one finding and longitudinal results are written to E-53-prmbench-results.md. (E-53_formal_verification_codegen.md:62)
- **E53-022 [E][DATA/SEC/OBS]** SpecOrigin has CsqCounterfactual, Adversary, Human, and SameLlmAsImpl; the last is refused with spec_origin_violation_blocked. (E-53_formal_verification_codegen.md:64)
- **E53-023 [E][TEST/CON]** Contracts require no Lean VerifierKind/default/consensus path and zero formal-tool invocation for NonCritical work. (E-53_formal_verification_codegen.md:64)
- **E53-024 [E][WF/FAIL/DET]** Formal retries cap at three; the third failure becomes a user-visible FINDING with cumulative hypothesis trail, no fourth attempt occurs, and identical query/spec retries are rejected as no progress. (E-53_formal_verification_codegen.md:66)
- **E53-025 [E][CAP/OBS/CON]** FormalVerificationPanel shows recent Z3/Kani/Verus/Dafny outcomes and counterexample previews, CriticalPathClassificationPanel shows classified/skipped functions, two Tauri commands feed them, and Lean is excluded. (E-53_formal_verification_codegen.md:68)
- **E53-026 [E][CAP/DATA]** Exported types are Z3/Kani outcomes, Verus invariants, Lean hint, three-retry controller, SpecOrigin, Python Dafny module, agent, CI files, commands, and thirteen named SSB action types. (E-53_formal_verification_codegen.md:70-86)
- **E53-027 [E][INT/DEP]** The system consumes D-3 aggregation, EvidenceGate, CSQ, D-13 origin, five formal dependencies, and blake3 for counterexample dedup. (E-53_formal_verification_codegen.md:87-98)
- **E53-028 [E][WF/INT]** Reachability is D-3 run_verifiers for Z3, auditor AUDITING for Kani, GitHub PR CI for Verus, and Python server module loading for Dafny. (E-53_formal_verification_codegen.md:99-103)
- **E53-029 [E][CON]** Production unwrap, todo/unimplemented, same-model spec/proof, Lean consensus/default use, and routine theorem-prover loops are prohibited. (E-53_formal_verification_codegen.md:104-109)
- **E53-030 [E][FAIL/CON]** Z3 silence abstains, Kani timeout warns, Dafny runs only on Critical Python, retries are bounded/different, Verus timeout fails, Dafny is capped, dependency checks are mandatory, and only blake3 hashes counterexamples. (E-53_formal_verification_codegen.md:110-119)
- **E53-031 [E][TEST/OBS]** The synthetic constant-time-password smoke expects Z3 Sat and optionally Kani Falsified counterexamples, a D-3 BLOCK against consensus, fixer routing, and only a separate asynchronous Lean hint. (E-53_formal_verification_codegen.md:121-124)
- **E53-032 [E][TEST]** Deterministic checks cover four Z3 outcomes, D-3 variant, four Kani outcomes, three Verus invariant families, CI files, four Dafny outcomes, Sonnet agent, read-only Lean and absent Lean consensus, four origins, three contracts, retry cap, blake3-only, no unwrap, panels without Lean, and seven smoke tests. (E-53_formal_verification_codegen.md:126-200)
- **E53-033 [E][TEST]** Acceptance requires Critical-only Z3 with Unknown/Timeout abstention and Kani in AUDITING with thirty-second timeout, falsification BLOCK, and counterexample. (E-53_formal_verification_codegen.md:202-205)
- **E53-034 [E][TEST]** Acceptance requires three Verus invariant groups in COMMITTING-gating CI and Critical-only Dafny with thirty-second cap and SpecMismatch BLOCK. (E-53_formal_verification_codegen.md:206-207)
- **E53-035 [E][TEST/SEC]** Acceptance requires the Sonnet agent, asynchronous non-voting Lean, origin refusal, three distinct retries, and a nightly every-diff PRMBench finding. (E-53_formal_verification_codegen.md:208-212)
- **E53-036 [E][TEST/CON]** Acceptance also requires all five dependency verifications, both panels/commands, three anti-pattern contracts, blake3-only, no production unwrap, green workspace check, and additive preservation of D-3. (E-53_formal_verification_codegen.md:213-220)
- **E53-037 [E][TEST/FAIL]** Z3 tests prove Unknown abstention and Critical-only dispatch; Kani tests prove falsification blocks and timeout does not. (E-53_formal_verification_codegen.md:222-227)
- **E53-038 [E][TEST/WF]** Verus tests simulate CI failure, Dafny tests require mismatch BLOCK, Lean tests prove post-BLOCK non-consensus arrival, and retry tests prove stop-at-three and same-spec rejection. (E-53_formal_verification_codegen.md:228-232)
- **E53-039 [E][TEST/SEC]** Contracts test same-model origin refusal, absent Lean kind, NonCritical zero-invocation, twenty deceptive findings, and no Lean rows in the dashboard. (E-53_formal_verification_codegen.md:233-237)
- **E53-040 [E][CAP/STATUS]** The file inventory creates five Rust modules, one Python validator, agent, two CI artifacts, two panels, fourteen tests, decision/results docs, and modifies D-3/CSQ/auditor/Tauri/dependency manifests. (E-53_formal_verification_codegen.md:239-260)
- **E53-041 [E][A/TEST]** Open measurements are Z3 false-positive effect, scalable autoformalization beyond three invariants, actual Z3/Kani wall time, Dafny-versus-tests bug rate, and whether local 3B can write Verus annotations. (E-53_formal_verification_codegen.md:287-293)
- **E53-042 [E][SEC/CFG]** The five dependencies require explicit user confirmation before merge and ship behind a formal-verification workspace feature that defaults off. (E-53_formal_verification_codegen.md:294)
- **E53-043 [E][FAIL]** Missing cargo-kani becomes NotApplicable with warning; missing Dafny likewise becomes NotApplicable with an event, and platform paths must be probed. (E-53_formal_verification_codegen.md:295-296)
- **E53-044 [E][CFG/FAIL]** Lean hints require ELAI_LEAN_PREMISE_HINT=1, default off; missing installation produces neither row nor error. (E-53_formal_verification_codegen.md:297)
- **E53-045 [E][SEC/DEP]** Origin enforcement compares implementation-author and spec-author roles/backends already tracked by PerRoleRuntime. (E-53_formal_verification_codegen.md:298)
- **E53-046 [E][FAIL]** Verus CI retries once on timeout, then auto-files an issue rather than silently skipping. (E-53_formal_verification_codegen.md:299)
- **E53-047 [E][DATA/TEST]** The canary fixture stores twenty diffs with deceptive=true and an expected-finding reason, each human curated. (E-53_formal_verification_codegen.md:300)
- **E53-048 [E][STATUS/INT]** Follow-ups tune thresholds, extend invariant scope, or add a verifier kind based on misses; E-52 shares EvidenceVariant integration. (E-53_formal_verification_codegen.md:302-304)

## Interactions and workflows

### Explicit cross-source and upstream/downstream interactions

- **X-01 [E] Memory-to-handoff:** E-1 identifies E-3 as a CoreBlock consumer, while E-3 makes E-1’s CoreBlock/memory_notes its required voice-and-digest anchor. (E1-043; E3-002, E3-006, E3-034)
- **X-02 [E] Shared media-validator substrate:** E-14 and E-16 both extend shipped B-24’s media validator and touch its web adapter, verdict aggregation, SSB, and dashboard evidence. E-14 adds deployment/journey/accessibility breadth; E-16 adds prompt-bias/pair-order/convergence depth. (E14-002, E14-005, E14-034; E16-001, E16-002, E16-024)
- **X-03 [E] Browser sandbox/deploy sequence:** E-14 requires E-15’s Tier-C node/npx/Chromium allowance and temporarily hosts DeployPreviewWorker in task_runner until C-16 CosmosStage exists. (E14-018, E14-040, E14-041)
- **X-04 [E] VLM-to-adversary sequence:** E-16’s intra-VLM disagreement primitive is explicitly reusable by D-13 and its neutralization functions by D-14 after promotion to a shared crate. (E16-002, E16-030)
- **X-05 [E] Retrieval-to-view/reasoning:** E-17 produces rows for C-8 to render and feeds later E-19 reasoning; its task trace and private harness are the evidence boundary. (E17-008, E17-017, E17-020, E17-033)
- **X-06 [E] Rule-before-enforcement:** E-34 must amend CLAUDE.md and B-16 before B-16 runs, while B-37 hosts the new GenAiAdapter without deleting CloudGroundingClient. (E34-001, E34-002, E34-008, E34-009, E34-030)
- **X-07 [E] Cost-control layering:** E-39 composes B-27 mode/spend schema, C-5 hardening, optional C-46 classification, B-37 runtime registration, and C-4 display; E-36/E-37 costs receive separate role buckets. (E39-002, E39-005, E39-039, E39-040)
- **X-08 [E] Metrics supply chain:** E-49 aggregates B-23 outcomes, wraps C-50/C-51 task pools, uses B-37 for model/harness ablation and cost, and may consume E-46 reward evidence; C-51 gates the public/private canary. (E49-002, E49-004, E49-027, E49-037, E49-044)
- **X-09 [E] Formal-verifier supply chain:** E-53 consumes D-3 consensus, C-52 research, shipped CSQ classification, D-13 independent specs, and E-52’s evidence surface; it remains additive to D-3. (E53-002, E53-004-E53-007, E53-027, E53-048)
- **X-10 [E] Evidence surfaces:** Every detailed plan except E-34 defines typed runtime evidence plus dashboard/CLI or durable artifacts; E-34 explicitly uses build/rule artifacts and expects no SSB row. (E1-024-E1-030; E3-020-E3-023; E14-019-E14-025; E16-013-E16-020; E17-017-E17-022; E34-020; E39-020-E39-025; E49-025-E49-029; E53-025-E53-031)

### Inferences

- **I-01 [I]** E-14 and E-16 need one canonical AX snapshot format/adapter rather than two parallel implementations. This follows from their shared B-24 integration points but neither source declares a merge order. (X-02; C-08 below)
- **I-02 [I]** E-39’s richer token/cost accounting would be a natural provider for E-49 CostPerSolve, but E-49 names RuntimeOrchestrator rather than E-39 and no explicit interface joins them. (E39-018, E39-021; E49-008, E49-041)
- **I-03 [I]** GenAiAdapter and E-39’s three new backends will need consistent per-call identity/cost/cache metadata if they coexist, although neither E-34 nor E-39 specifies that shared adapter contract. (E34-009; E39-013-E39-018)
- **I-04 [I]** A single handoff voice likely needs a precedence rule among E-1 retrieved-note preamble, E-1 CoreBlock, and E-3 voice.toml VoiceAnchor; both sources require these inputs but do not define one combined builder/order. (E1-017, E1-023, E1-043; E3-007-E3-009)
- **I-05 [I]** E-17’s RAG before/after measurements could be recorded through E-49’s stable/attribution metrics, but no assigned source explicitly makes the RAG harness an E-49 task pool. (E17-018; E49-005-E49-014)
- **I-06 [I]** E-53 verifier outcomes could become additional E-49 reliability signals alongside E-46 process reward, but E-49 does not name the formal evidence and E-53 only names EvidenceVariant. (E49-027; E53-027, E53-048)

## Ambiguities and underspecified behavior

### Ambiguous or incomplete contracts

1. **A-01 [A] Source status is not implementation proof.** E-1 and E-17 contain strong shipped rollups while their acceptance boxes remain unchecked; no implementation was in scope for this read, so shipped remains an explicit document claim only. (E1-001-E1-006, E1-032-E1-036; E17-001-E17-005, E17-023-E17-027)
2. **A-02 [A] E-1 reflection trigger has no single formal predicate.** K=4 after successful run_turn, an idle gap of at least two seconds, max_turns_per_note=32, and a proposed 4,000-token budget appear in separate sections without precedence or one config schema. (E1-019, E1-020, E1-033, E1-039)
3. **A-03 [A] E-1 invalid Core edit warning is unnamed.** The no-op plus SSB warning is required, but no action type or payload is included in the action inventory. (E1-018, E1-024)
4. **A-04 [A] E-1 recall attribution is not typed.** The smoke asks for prior-session citation, but no tag syntax, session-id field, or evidence-pointer rendering contract is defined. (E1-030)
5. **A-05 [A] E-1 decay units are only implied.** Decision defaults are stated per day while the exponential formula uses raw now-minus-last_accessed; conversion and timestamp unit are not specified. (E1-021, E1-026)
6. **A-06 [A] E-1 conflict comparison is underspecified.** “Same tag set,” the embedding model’s opposite-cosine semantics, and user adjudication UI/authorization have no normalization or state-transition schema. (E1-022)
7. **A-07 [A] E-3 names seven mapped roles but tests eight.** The eighth AgentRole and its runtime/default text are not enumerated. (E3-003, E3-028)
8. **A-08 [A] Prewarm failure behavior is absent.** Started/complete exist, but timeout, failure, cancellation, duplicate hints, and stale-transition invalidation are not defined. (E3-011-E3-013, E3-020)
9. **A-09 [A] Role badge input is untyped.** CommandWindow consumes a current_role SSB signal, but the plan adds no current-role action/payload and gives no reconnect/stale-state rules. (E3-010, E3-021)
10. **A-10 [A] Handoff compression is algorithmically thin.** Target_tokens, loss bounds, selection of 5x versus 20x, failure fallback, deterministic ties, and CompressedPayload fields are not specified. (E3-014)
11. **A-11 [A] Persona-vector lifecycle is missing.** The fixture set, layer choice, persistence/versioning, vector integrity, and fallback when extraction fails are unspecified. (E3-015, E3-031)
12. **A-12 [A] E-14 says the Rust AX reader performs a “plain TS parse.”** It is unclear whether TS means TypeScript, typed-structure parsing, or a wording error. (E14-006)
13. **A-13 [A] Deploy retry policy is incomplete.** Poll cadence is 30 seconds and cURL backoff exists, but maximum wait, retry schedule, terminal timeout, provider failover, and idempotency are absent. (E14-014, E14-033)
14. **A-14 [A] Deployment secrets lack a policy.** api_token is configured, but storage, redaction, command permissions, log suppression, and dashboard exposure are not defined. (E14-014, E14-019)
15. **A-15 [A] E-14’s nine-plus SSB actions lack payload/schema contracts.** Several appear only in deterministic checks/observable text, and schema-version/golden-file/idempotency rules are absent. (E14-012, E14-015, E14-017, E14-025-E14-030)
16. **A-16 [A] Stagehand cache validity is underdefined.** URL/goal hashes are named, but DOM-version input, TTL, privacy, corruption behavior, and stale-entry eviction are not. (E14-009, E14-037)
17. **A-17 [A] Temporary-to-Cosmos migration is unbounded.** The source does not say how duplicate task-runner/Cosmos workers are prevented during C-16 rollout or how state is migrated. (E14-018, E14-041)
18. **A-18 [A] E-16 trigger thresholds are missing.** “Low logprob,” “sycophancy above threshold,” and “user-aligned claim” lack numeric defaults, classifiers, and config keys. (E16-010, E16-013)
19. **A-19 [A] “No actionable diffs” is not schema-bound.** RCE convergence accepts critic prose/field semantics that are not defined in VlmValidationOutput. (E16-009)
20. **A-20 [A] XGrammar fallback disposition is incomplete.** vlm_schema_unconstrained is logged, but the corresponding verdict, retry, escalation, and whether the unmasked output is discarded are not stated. (E16-012)
21. **A-21 [A] E-17 staging artifacts are not reproducibly pinned.** Doctor names three model families and sizes/tokenizer hashes, but exact URLs, licenses, filenames, model-file hashes, and atomic-download recovery are absent. (E17-009, E17-028)
22. **A-22 [A] Contextual retrieval’s shipped depth is unclear.** The rollup says heuristic ground floor plus LLM-template scaffold; task text requires a live sidecar call on every changed chunk. It does not explicitly say the LLM generation path is wired in the shipped claim. (E17-002, E17-010)
23. **A-23 [A] E-17 config precedence is unstated.** ELAI_CODE_VEC and ELAI_HYDE_NL_ONLY coexist with TOML groups, but environment/TOML/default precedence is not defined. (E17-001, E17-013, E17-015)
24. **A-24 [A] RRF tie/order semantics are absent.** Weight values and floor are explicit, but deterministic tie-breaking, duplicate document fusion, and override validation are not. (E17-012, E17-015)
25. **A-25 [A] GenAiAdapter’s partial method boundary is vague.** “complete” is the only named implementation; RuntimeAdapter’s other methods and when selecting the adapter may hit a registered stub are not enumerated. (E34-009, E34-012)
26. **A-26 [A] “Current at adoption time” can differ from acceptance.** The source both pins beta.20 and tells the executor to use whatever current beta is found, without saying which text controls. (E34-009, E34-010, E34-023)
27. **A-27 [A] E-39 velocity dimensions do not map cleanly to windows.** req/min, dollars/min/hour/day and OneMin/TenMin/OneHour/TwentyFourHour are listed, but which metric applies to TenMin and how multiple breaches compose are not stated. (E39-008)
28. **A-28 [A] Estimate-ratio denominator is undefined.** After estimates become p50/p95, actual divided by “estimate” does not select p50, p95, or a range rule. (E39-018)
29. **A-29 [A] Cooperative abort under concurrency is incomplete.** Multiple in-flight calls, cancellation, bank ordering, dropped writes, partial-result consistency, and exactly-once abort evidence are not specified. (E39-006)
30. **A-30 [A] Override/Opus confirmation authorization is thin.** Typed strings are required, but user identity, permissions, expiration, replay prevention, and whether Opus confirmation reuses cap override are absent. (E39-012, E39-019)
31. **A-31 [A] AutoMix Continue accepts an output already described as failed.** “Structurally valid” and the boundary between a recoverable failed-output and unsafe/incorrect output are not formalized. (E39-009)
32. **A-32 [A] CostPerSolve zero-correct behavior is undefined.** No infinity/null/error convention is supplied for total_usd divided by zero correct solves. (E49-008, E49-009)
33. **A-33 [A] Attribution baseline persistence is incomplete.** A frozen landing snapshot is required, but its artifact path, hash, mutation guard, and behavior before the two ablations run are missing. (E49-013, E49-014, E49-037)
34. **A-34 [A] Private-eval confidentiality is only Git exclusion.** Access controls, encryption, task authoring/review, contamination monitoring during manual regeneration, and deletion policy are absent. (E49-015, E49-038)
35. **A-35 [A] Safety success labels are underspecified.** Exfiltration/override success, harm/benign labels, partial refusal, scorer reproducibility, and prior-measurement compatibility are not typed. (E49-019, E49-020)
36. **A-36 [A] Horizon licensing/provenance may alter the promised pool.** The source alternates between vendoring METR tasks and curated equivalents without defining equivalence acceptance. (E49-011, E49-038)
37. **A-37 [A] Awareness regex has no escaping/language policy.** Case sensitivity, substrings such as legitimate “benchmark” discussion, non-English outputs, and false-positive review are unspecified. (E49-021, E49-039)
38. **A-38 [A] Kani property authorship is unclear.** “LLM proposes” is allowed only if it is not the implementation author, but the invoking role/model comparison for Kani proposals is not explicitly joined in Task 2. (E53-004, E53-014, E53-045)
39. **A-39 [A] Dafny semantic equivalence is asserted, not defined.** No input relation, state model, oracle, failure proof, or supported Python subset explains SpecValidator’s equivalence judgment. (E53-017, E53-018)
40. **A-40 [A] Retry can weaken a specification.** Requiring a different predicate prevents identical loops but no monotonic-strength rule stops retries from relaxing the property until verification passes. (E53-024)
41. **A-41 [A] Missing-tool telemetry differs by tool.** Kani warns, Dafny emits an unspecified row, Lean is silently absent, and Verus installation/failure fallback is not normalized into one outcome model. (E53-043, E53-044, E53-046)
42. **A-42 [A] Feature-off completion is unclear.** Formal verification defaults off but acceptance requires all paths and a green build; the source does not define whether “shipped” means compiled-but-disabled or enabled in CI. (E53-036, E53-042)
43. **A-43 [A] D-3 may not exist.** The plan allows creating a “minimal D-3 surface stub” if its planned file is absent, but does not define that stub’s scope, ownership, or compatibility with eventual D-3. (E53-005, E53-010, E53-040)

### Conflicting statements

1. **C-01 [C] E-1 schema version conflict.** The original task/check says 45 to 46, while the completion header says SCHEMA_VERSION 103 is locked and the discarded actions cannot be added. (E1-003 versus E1-013, E1-031)
2. **C-02 [C] E-1 event inventory is stale.** Wiring/deterministic/observable/acceptance retain SleepComputeStarted and SleepComputeQueryCached, while the rollup/task says they are consolidated into MemoryEvolutionRun. (E1-003, E1-019, E1-024 versus E1-027-E1-033)
3. **C-03 [C] E-1 trigger model conflicts.** Current production is turn-count K=4 after success, while wiring and checks expect user_typing_detected/idle; those explicit typing events are themselves deferred. (E1-006, E1-019 versus E1-028, E1-031, E1-033)
4. **C-04 [C] E-1 code target drift.** The rollup declares chat_memory_reflect canonical, but Task 5, wiring, and files-touched still instruct edits/imports in chat_reflect. (E1-005, E1-019 versus E1-017, E1-027, E1-036)
5. **C-05 [C] E-1 action-count acceptance is obsolete.** Acceptance demands six new round trips, but only four are shipped as new variants and the remaining two are phases of an existing variant. (E1-024 versus E1-032)
6. **C-06 [C] E-1 status presentation is mixed.** The top declares numerous items shipped, but the full acceptance remains unchecked and deterministic checks reference superseded schema/events. (E1-001-E1-006 versus E1-031-E1-036)
7. **C-07 [C] E-14 and E-16 disagree on AX technology.** E-14 uses Playwright 1.56+ CLI JSON ax-snapshot.json and ax_snapshot.rs; E-16 uses Playwright 1.59+ ariaSnapshot YAML axtree.yaml and axtree.rs. (E14-006, E14-007, E14-025, E14-034 versus E16-005, E16-020, E16-024)
8. **C-08 [C] B-24 axe state differs.** E-14 says axe injection is missing/net-new; E-16 says shipped B-24 already has axe-core. (E14-005, E14-008 versus E16-003)
9. **C-09 [C] Dashboard ownership differs.** E-14 modifies a MediaValidationPanel under windows, while E-16 extends MediaValidationDrawer under components; both present as the existing B-24 surface. (E14-019, E14-034 versus E16-014, E16-024)
10. **C-10 [C] E-16 says preserve then replace.** Objective/existing-state language says not to re-author and to preserve feedback_loop, while Task 4 explicitly says replace its iteration logic. (E16-001, E16-003 versus E16-009)
11. **C-11 [C] E-16 live-test ignore wording is tense.** cloud_mode_live must be unignored and deterministic grep expects zero ignore hits, while the task also adds a needs-real-vlm ignored guard; whether that is the same test or a separate test is not named. (E16-015, E16-021)
12. **C-12 [C] E-17 reranker default reverses.** Task/stub/acceptance require Jina or BGE default, but the completion rollup explicitly retains MiniLM because Jina ONNX is unavailable. (E17-003 versus E17-011, E17-021, E17-023, E17-024)
13. **C-13 [C] E-17 SSB readiness differs.** Tasks/wiring/acceptance expect ContextualChunkEmitted and ChunkRegenSkipped, while the completion rollup says their schema variants are deferred. (E17-004 versus E17-017, E17-019, E17-021, E17-024)
14. **C-14 [C] E-17 cAST threshold has two gates.** Acceptance says at least +4 points yet labels +2 conservative, and the actual smoke/risk uses +2. (E17-018, E17-025, E17-026, E17-029)
15. **C-15 [C] E-34 adapter scope conflicts.** Task 4 describes a roughly 250-line multi-provider RuntimeAdapter implementation, while Task 7/acceptance define only one working method plus registered unimplemented methods. (E34-009 versus E34-012, E34-023)
16. **C-16 [C] E-34’s “not a Lazy Code stub” still permits unimplemented.** The task calls the adapter real/non-lazy and immediately authorizes unimplemented markers for remaining methods. (E34-012, E34-021)
17. **C-17 [C] E-39 action count drifts.** Existing-state prose says six new variants, while the wiring/acceptance/deterministic sections enumerate eight. (E39-005 versus E39-020, E39-026)
18. **C-18 [C] Spend boundary differs.** Task aborts only when spend is greater than cap, whereas summary orientation and its exact-cap example say block at the cap; the detailed acceptance does not resolve equality. (E39-006; summary delta SD-E39-10 below)
19. **C-19 [C] Free-tier boundary vocabulary differs.** Task/acceptance refuse at or above 95 percent but distinguish Approaching from actual Exceeded; the test expects Exceeded on the next call when current usage is exactly 95 percent. (E39-017, E39-028, E39-032)
20. **C-20 [C] E-49 contamination boundary differs.** Task says greater than twenty points blocks, but the test labels an exact twenty-point gap as BLOCK. (E49-016 versus E49-033)
21. **C-21 [C] E-49 “replace” remains additive.** The objective says replace single-seed reporting, while task/acceptance retain pass_at_1 and merely forbid emitting it alone. (E49-001 versus E49-004-E49-007, E49-028, E49-031)
22. **C-22 [C] E-53 “4th verifier” count is unclear.** The title/task calls Z3 the fourth kind, but the stated D-3 surface already lists deterministic, PPM, intrinsic, and cross_context before Z3. (E53-001, E53-005)
23. **C-23 [C] Critical-only rule conflicts with global Verus CI.** Contracts say only CSQ Critical triggers Z3/Kani/Verus/Dafny, yet fixed FSM/IPC/KV invariants run on every PR regardless of the edited function’s classification. (E53-007, E53-016, E53-023, E53-030)
24. **C-24 [C] SpecValidator selects one tool but smoke expects two.** Task 5 says the agent chooses one tier, while the observable example expects Z3 and, if harnessed, Kani for one function. (E53-018 versus E53-031)
25. **C-25 [C] Dafny mismatch test wording drifts.** Acceptance says SpecMismatch blocks; the test named blocks-on-mismatch injects NotVerified, treating different outcomes as equivalent. (E53-017, E53-034, E53-038)
26. **C-26 [C] Verus workflow path drifts.** Files-touched creates .elai_cc/.github/workflows/verus-check.yml, while reachability names .github/workflows/verus-check.yml without the .elai_cc prefix. (E53-016, E53-028, E53-040)

## Summary-layer additions and corrections

Comparison target: **05-E-SUMMARY_INDEX.discovery.md**, used only as orientation. “Addition” means explicit assigned-original detail absent from or materially thinner than the summary. “Correction” means the assigned original contradicts or does not support the summary wording. Matching summary claims are already represented by the ledgers and are not repeated here.

### E-1 deltas against summary lines 15-26

- **SD-E1-01 Addition:** Add the completion rollup’s exact shipped-on-V2/session/consolidated/deferred state, including SCHEMA_VERSION 103 lock, four shipped new actions, MemoryEvolutionRun phase reuse, canonical chat_memory_reflect target, and deferred explicit typing events. (E1-001-E1-006)
- **SD-E1-02 Addition:** Add the sixteen-column memory_notes schema, blake3/tag-linked atomic one-hop evolution, single-call entity-seeded PPR top eight, live-fact filter, and 150-token cap/back-compat view. (E1-013-E1-016)
- **SD-E1-03 Addition:** Add CoreBlock’s exact fields/cap/storage, two tool APIs, invalid-section no-op/warning, K=4 reflection path, output telemetry, and max_turns_per_note default. (E1-017-E1-020)
- **SD-E1-04 Addition:** Add decay formula/access update, cosine-less-than-minus-0.4 conflict/no-update/user-adjudication behavior, top-three under-50-ms CLI preamble, panel fields/command, and provisional decay defaults. (E1-021-E1-026)
- **SD-E1-05 Addition:** Add the full observable timing/artifact gates, deterministic and contract tests, churn/latency/decay thresholds, no-unwrap rule, rejected MIRIX design, and open measurement risks. (E1-029-E1-043)
- **SD-E1-06 Correction:** Replace the summary’s E-5 memory_facts storage wording with E-1’s memory_notes/CoreBlock model; E-5 is a schema dependency, not the table name used by this source. (summary E-1.F04; E1-008, E1-013)
- **SD-E1-07 Correction:** Remove the summary’s BeliefShift promotion gate. This source deletes the cited BeliefShift ID as fabricated and uses CTIM-Rover only as counter-evidence for bulk injection; it defines no promotion threshold. (summary E-1.F06; E1-004, E1-037)
- **SD-E1-08 Correction:** Replace memory_promoted/memory_evicted with the original’s MemoryNoteInserted/Evolved/ConflictDetected/CoreBlockReplaced plus MemoryEvolutionRun phase actions. (summary E-1.F08-F09; E1-003, E1-024)
- **SD-E1-09 Correction:** Replace “after five turns” with the shipped K=4 after-successful-turn trigger, while preserving the source’s separate unresolved two-second-idle wording as conflict C-03. (summary E-1.F10; E1-019, C-03)
- **SD-E1-10 Correction:** Remove the unsupported turn-six and [memory:K] tag assertion. The source’s smoke asks a fresh session to recall blake3 unprompted and cite the prior session, without that tag or turn number. (summary E-1.F11; E1-030)

### E-3 deltas against summary lines 46-57

- **SD-E3-01 Addition:** Add VoiceAnchor’s exact fields, 80–120-token composition, sole voice.toml authority and sections/rules, build_role_prompt placement, and per-send stable-hash evidence. (E3-007-E3-009, E3-018)
- **SD-E3-02 Addition:** Add predictive FSM hinting, 12/24/48GB policy, default/two-specialized warm behavior, asynchronous semantics, and prewarm actions. (E3-011-E3-013, E3-019-E3-020)
- **SD-E3-03 Addition:** Add non-LLM 5–20x handoff compression, local persona-vector gate, display-only style normalizer, vLLM prefix cache, and all negative requirements. (E3-014-E3-017, E3-022)
- **SD-E3-04 Addition:** Add cycle smoke, stable-hash/warm/adapter/compression/UI tests, A/B thresholds, and open telemetry/deferred harness work. (E3-023-E3-034)
- **SD-E3-05 Correction:** The source does not read E-5 voice_style directly into this preamble; it consumes E-1 CoreBlock and the canonical voice.toml VoiceAnchor. (summary E-3.F04; E3-002, E3-006-E3-009)
- **SD-E3-06 Correction:** The summary’s documented/Hopper-deferred CUDA warm pool is not this original’s contract. The original implements predicted prewarm, no-op on 12GB and enabled on 24GB+, with no Hopper deferral. (summary E-3.F05-F06; E3-011-E3-013)
- **SD-E3-07 Correction:** Replace SettingsWindow/ResponseCardPayload with CommandWindow, RuntimePanel, HandoffPayload, role_prompt, and runtime-orchestrator surfaces. (summary E-3.F07; E3-010, E3-014, E3-020-E3-021, E3-029)
- **SD-E3-08 Correction:** role_shown_model_hidden and the exact Designer/settings-toggle smoke are absent; the original emits UnifiedPreambleApplied and tests an executor/auditor/memory-writer cycle with click-to-open drawer. (summary E-3.F08-F10; E3-020, E3-023)

### E-14 deltas against summary lines 239-260

- **SD-E14-01 Addition:** Add AX node/snapshot schemas, exact four disk artifacts, Tier-C CLI invocation, and deterministic render defaults. (E14-006-E14-007)
- **SD-E14-02 Addition:** Add axe’s four-impact disposition and Stagehand’s fallback-only url/goal cache behavior. (E14-008-E14-009)
- **SD-E14-03 Addition:** Add the exact VLM JSON schema/error fallback, 0.02 SSIM regional-crop routing, and complete six-case T1/T2 verdict matrix. (E14-010-E14-013)
- **SD-E14-04 Addition:** Add deploy config/cadence/events/default-off behavior, first-200 validation, C-16 fallback lifecycle, SaaS context isolation/network mocks/journey schema, and air-gap disable rule. (E14-014-E14-018, E14-038)
- **SD-E14-05 Addition:** Add CF signatures/skipped outcome, full dashboard sections/commands, nine-event evidence chain, three config groups, and no-bypass/no-dynamic-error/no-unwrap gates. (E14-017, E14-019-E14-030)
- **SD-E14-06 Addition:** Add exhaustive parser/severity/threshold/veto/schema/cache/protection/deploy/journey test matrices and calibration/hardware/sandbox/refresh deferrals. (E14-031-E14-041)
- **SD-E14-07 Correction:** Replace the summary’s crates/orchestrator/src/browser_e2e/ surface with media_validator modules plus knowledge::deploy_preview_worker and dashboard integration. (summary E-14.F11; E14-022-E14-024, E14-034)
- **SD-E14-08 Correction:** Replace axtree_snapshot/axe_violation with the source’s ax_snapshot_emitted/axe_findings_count and full hybrid/deploy/CF action inventory; exact payloads remain ambiguous. (summary E-14.F12-F14; E14-012, E14-015, E14-017, E14-025-E14-029)
- **SD-E14-09 Correction:** The source requires durable snapshot artifacts and a deploy-ready worker but does not require committing an AXTree diff for a greenfield build. (summary E-14.F15; E14-025, E14-028-E14-029)

### E-16 deltas against summary lines 280-294

- **SD-E16-01 Addition:** Add YAML/time/web-only AX behavior and the twenty-render one-percent stability fallback. (E16-005, E16-016, E16-020-E16-025)
- **SD-E16-02 Addition:** Add neutralize_query/contrastive_decode schemas, neutral-primary rule, inverted-frame probe, quarantine behavior, and four exact SSB types. (E16-006-E16-008, E16-013, E16-017)
- **SD-E16-03 Addition:** Add swap-order double invocation/agreement gate, LPIPS-or-critic-or-five convergence, and conditional InternVL second opinion. (E16-008-E16-011)
- **SD-E16-04 Addition:** Add XGrammar compliance/fallback evidence, distinct dashboard columns, cloud feature/ignore discipline, and exact deterministic/acceptance/test gates. (E16-012, E16-014-E16-023)
- **SD-E16-05 Addition:** Add VRAM sequencing, schema warmup, pixel-metric-only-as-flake-suppressor, and D-13/D-14/C-20 follow-ups. (E16-025-E16-030)
- **SD-E16-06 Correction:** “Preserve feedback_loop” requires qualification because Task 4 explicitly replaces its iteration logic with convergence/cap behavior. (summary E-16.F05; C-10)
- **SD-E16-07 Correction:** Replace rce_convergence_step/intra_modal_disagreement with SycophancyFlag, SwapOrderDisagreement, AxtreeMismatch, and VlmSchemaUnconstrained; rce_iteration/sycophancy_probe appear in smoke queries but not the produced-variant list. (summary E-16.F08-F09; E16-017, E16-021)
- **SD-E16-08 Correction:** The detailed smoke is a clean TODO render with no AX mismatch and probe evidence, not a specified deliberate-fail screenshot producing intra_modal_disagreement or an “honest verdict” action. (summary E-16.F10-F11; E16-020)

### E-17 deltas against summary lines 296-314

- **SD-E17-01 Addition:** Add the exact completion status: fifteen V2 surfaces, contextual/cAST session additions and tests, Jina-model pending rationale, two schema-deferred events, and disabled late-interaction producer. (E17-001-E17-005)
- **SD-E17-02 Addition:** Add doctor verify-first behavior and failure diagnosis, source-hash contextual regeneration, exact reranker enum/config, candidate cap 100, RRF floor .005, and NL-only HyDE gate. (E17-007-E17-013)
- **SD-E17-03 Addition:** Add cAST split/merge, three-ranker weights 1/1/.8, disabled PLAID flag, six exact SSB types, and the complete query trace/reachability/negative rules. (E17-014-E17-022)
- **SD-E17-04 Addition:** Add the 135-file/50-query R@5/R@10/MRR harness and every deterministic/acceptance/test threshold including reproducible reranker standard deviation. (E17-018, E17-023-E17-026)
- **SD-E17-05 Addition:** Add packaging, ingest-cost, cAST fallback, BM42 rejection, classifier-F1 fallback, and E-17.B/C/D deferred routes. (E17-028-E17-033)
- **SD-E17-06 Correction:** Jina-rerank-v3 is not the current default; the completion header keeps MiniLM and exposes BGE-v2-m3 as opt-in because official Jina ONNX is unavailable. (summary E-17.F06; E17-003, C-12)
- **SD-E17-07 Correction:** Found-in-the-Middle calibration/lift is absent from this original; its quality fixes are cap/floor, HyDE gating, cAST, contextual retrieval, and three-way RRF. (summary E-17.F07, E-17.F13; E17-010-E17-018)
- **SD-E17-08 Correction:** Replace scripts/setup_onnx.sh with the rollup’s scripts/stage_onnx_models.py BGE recipe and elai doctor knowledge-models. (summary E-17.F09; E17-001-E17-003, E17-009)
- **SD-E17-09 Correction:** Replace onnx_model_staged/rerank_active/fitm_calibrated with DeepPoolCapped, RrfQualityGateFired, HydeSkippedCodeQuery, RrfComponentWeights, plus two deferred contextual actions. (summary E-17.F11-F13; E17-001, E17-004, E17-019)
- **SD-E17-10 Correction:** The source uses blake3 for chunk regeneration, not an explicit blake3 verification contract for all three staged model binaries. (summary E-17.F14; E17-009-E17-010, E17-025)
- **SD-E17-11 Correction:** Monotonicity is proved by .elai_cc/eval/rag with fifty queries, not a named phase_3K_rag.toml artifact. (summary E-17.F15; E17-018, E17-022-E17-026)

### E-34 deltas against summary lines 609-623

- **SD-E34-01 Addition:** Add exact old/new ort wording, unchanged rc.12 code pin, audit appendix, corrected B-16 text, and strict E-34-before-B-16 ordering. (E34-001-E34-008)
- **SD-E34-02 Addition:** Add pinned beta.20 GenAiAdapter as a coexisting partial migration with one real method, explicit stubs, identity checks, and two-phase follow-up. (E34-009-E34-012, E34-025-E34-027)
- **SD-E34-03 Addition:** Qualify rmcp as compile-ready dependency/re-export only; MCP server/tool design is deferred. (E34-004, E34-011, E34-028)
- **SD-E34-04 Addition:** Add archived-llm replacement/absence handling, rkyv/postcard decision gate, swiftide and async-std bans, optional oxc threshold, and all dependency hygiene constraints. (E34-003, E34-013-E34-018, E34-021)
- **SD-E34-05 Addition:** Add offline adapter/rmcp/snapshot/property/contract tests, green-build gate, no-runtime-SSB proof mode, and five explicit deferred branches. (E34-020-E34-024, E34-029-E34-033)
- **SD-E34-06 Correction:** “Adopt rmcp” means dependency plus minimal compile surface, not a functioning ELAI MCP server. (summary E-34.F02; E34-004, E34-011)
- **SD-E34-07 Correction:** “Adopt genai” means a pinned-beta, one-method parallel adapter, not deletion/consolidation of CloudGroundingClient in this session. (summary E-34.F03; E34-009, E34-012, E34-026-E34-027)
- **SD-E34-08 Correction:** ort_rc_amended and rmcp_adopted are not SSB actions in this original; observable_effect explicitly expects no SSB. (summary E-34.F09-F10; E34-020)

### E-39 deltas against summary lines 700-716

- **SD-E39-01 Addition:** Add cooperative post-call banking/partial-result abort semantics and the exact abort payload. (E39-006)
- **SD-E39-02 Addition:** Add per-role caps/map, four velocity windows/evidence, and the exact AutoMix Continue/Retry/Escalate workflow plus measured gate. (E39-007-E39-010)
- **SD-E39-03 Addition:** Add the full difficulty feature vector, five-route budget matrix, explicit Opus confirmation, and mandatory decision payload. (E39-011-E39-013)
- **SD-E39-04 Addition:** Add four cacheable blocks/suffix-only rule, 5min default/two TTLs, hit-rate target, three provider routes, and 80/95 UTC quota policy. (E39-014-E39-017)
- **SD-E39-05 Addition:** Add p50/p95 estimates and ratio tripwire, exact layered defaults/override payload, eight action names, commands/panel, and complete negative rules. (E39-018-E39-025)
- **SD-E39-06 Addition:** Add all ten contract matrices and the citation/legal/free-tier/AutoMix/bootstrap contingencies plus two-tier-only follow-up boundary. (E39-026-E39-041)
- **SD-E39-07 Correction:** The detailed source gates per agent role, not separately per tool. (summary E-39.F04; E39-007)
- **SD-E39-08 Correction:** Replace cost_gate.rs with cloud_spend.rs plus velocity_cap, automix_gate, cost_router, and free_tier_guard modules. (summary E-39.F09; E39-020-E39-021, E39-034)
- **SD-E39-09 Correction:** Replace hard_ceiling_blocked/retry_budget_exhausted with TaskAbortedBySpendCap, VelocityCapBreached, AutomixVerifierVerdict, CostRouterDecision, CostEstimateInaccurate, two quota actions, and LayeredCapManualOverride. (summary E-39.F10-F11; E39-020)
- **SD-E39-10 Correction:** The summary’s block-at-exactly-$0.05 is not supported by Task 1’s greater-than comparison; equality is unresolved and recorded as C-18. (summary E-39.F12; E39-006, C-18)

### E-49 deltas against summary lines 890-906

- **SD-E49-01 Addition:** Add StablePass1’s complete schema, n=5 run/hash/standard-deviation formula, clamp, compatibility field, and absent-cost behavior. (E49-005-E49-009)
- **SD-E49-02 Addition:** Add frozen quarterly human-time horizon schema/pool/cadence and the mandatory two-axis attribution schema/refusal/baseline rules. (E49-010-E49-014)
- **SD-E49-03 Addition:** Add private-set location/composition/Git exclusion and exact 10/20-point contamination gates, plus 30/90-day revert/churn schemas and thresholds. (E49-015-E49-018)
- **SD-E49-04 Addition:** Add prompt-injection and refusal suite sizes/targets/cadence/artifacts, configurable awareness schema/threshold, five-position context probe, and Cover@.95 formula. (E49-019-E49-023)
- **SD-E49-05 Addition:** Add phase-versus-doc STOP lint behavior, five panels/commands, all produced/consumed types/actions, and complete negative requirements. (E49-024-E49-029)
- **SD-E49-06 Addition:** Add every deterministic/acceptance/test vector, provisional-seed calibration, licensing/fallbacks, cost-data blocker, Git-author fallback, and downstream evidence story. (E49-030-E49-044)
- **SD-E49-07 Correction:** The implementation surface modifies CLI main and adds orchestrator metric modules; it does not extend crates/cli/src/metrics.rs as the summary says. (summary E-49.F12; E49-027, E49-035)
- **SD-E49-08 Correction:** Replace stable_pass_at_1 and harness_attribution_tuple with stable_pass_1_aggregated and attribution_tuple_emitted; stop_verified is a CI lint result, not a declared SSB action. (summary E-49.F13-F15; E49-005-E49-007, E49-013-E49-014, E49-024)
- **SD-E49-09 Correction:** STOP-Verified is scoped: phase metric fields fail, documentation references warn, and historical references are annotated; pass_at_1 itself remains only when accompanied by stable_pass_1_n5. (summary E-49.F11; E49-004, E49-024, E49-028, E49-043)
- **SD-E49-10 Correction:** The exact twenty-point contamination example conflicts with the prose’s strictly-greater-than block threshold and must not be normalized silently. (E49-016, E49-033, C-20)

### E-53 deltas against summary lines 984-1002

- **SD-E53-01 Addition:** Add exact Z3 input/origin, five-second timeout, Sat/Unsat/Unknown/Timeout meanings, abstention behavior, and one-of-at-least-two vote semantics. (E53-004-E53-011)
- **SD-E53-02 Addition:** Add Kani trigger/attribute/thirty-second outcomes and fixer routing; add three exact Verus invariants plus five-minute PR-CI behavior. (E53-012-E53-016)
- **SD-E53-03 Addition:** Add the four Dafny outcomes/semantic validator flow, Sonnet SpecValidator selection contract, and same-model refusal. (E53-017-E53-019, E53-022)
- **SD-E53-04 Addition:** Add optional asynchronous/non-voting Lean hint, twenty-diff nightly canary, three-distinct-retry/no-progress gates, thirteen exact actions, panels/commands, and evidence entry paths. (E53-020-E53-031)
- **SD-E53-05 Addition:** Add all deterministic/acceptance/test matrices, feature-off-by-default and explicit-confirmation policy, missing-tool fallbacks, Lean env behavior, origin comparison, Verus retry, and fixture schema. (E53-032-E53-047)
- **SD-E53-06 Addition:** Add measured open questions and follow-ups for false positives, invariant scope, latency, Dafny benefit, local-Verus capability, and new verifier kinds. (E53-041, E53-048)
- **SD-E53-07 Correction:** Replace z3.rs/kani.rs/verus.rs/dafny.rs with z3_smt_check.rs, kani_harness.rs, verus_invariants.rs, and Python dafny_spec_validator.py plus the agent/CI modules. (summary E-53.F10; E53-009, E53-012, E53-015, E53-017-E53-018, E53-040)
- **SD-E53-08 Correction:** Replace z3_consensus/kani_proved/autoverus_proved with the thirteen source actions; successful Z3 is z3_smt_unsat and Kani is kani_harness_verified. (summary E-53.F11-F13; E53-026)
- **SD-E53-09 Correction:** Z3 is not sole required consensus before promotion; Unsat contributes one vote to D-3’s at-least-two rule, while Sat blocks and Unknown/Timeout abstain. (summary E-53.F14; E53-005, E53-009)
- **SD-E53-10 Correction:** Lean is excluded by contracts, absent VerifierKind/consensus paths, opt-in hint-only behavior, and dashboard exclusion—not by the summary’s asserted CI lint. (summary E-53.F15; E53-020, E53-023, E53-025, E53-029-E53-030, E53-044)

## Zero-omission and zero-duplicate audit

### Coverage totals

| Assigned source | Lines read | Explicit atomic units | First-last ID |
|---|---:|---:|---|
| E-1_infinite_context_chat_illusion.md | 209/209 | 43 | E1-001..E1-043 |
| E-3_model_handoff_ux.md | 200/200 | 34 | E3-001..E3-034 |
| E-14_browser_e2e_agents.md | 217/217 | 41 | E14-001..E14-041 |
| E-16_vlm_image_validation.md | 181/181 | 30 | E16-001..E16-030 |
| E-17_rag_retrieval_quality.md | 251/251 | 33 | E17-001..E17-033 |
| E-34_rust_ai_ecosystem_2026.md | 231/231 | 33 | E34-001..E34-033 |
| E-39_cost_token_economics.md | 258/258 | 41 | E39-001..E39-041 |
| E-49_coding_agent_metrics.md | 280/280 | 44 | E49-001..E49-044 |
| E-53_formal_verification_codegen.md | 304/304 | 48 | E53-001..E53-048 |
| **Total** | **2131/2131** | **347** | **347 unique IDs** |

Audit assertions:

- **Manifest completeness:** 9/9 exact assigned files, 2,131/2,131 logical lines, 251,948/251,948 bytes, and 9/9 SHA-256 values verified twice with zero mismatch.
- **Source completeness:** every title/objective/effort/prerequisite/parallel-status/context/existing-state/read-list/task/wiring/produced/consumed/reachability/stub-ban/frontend/observable/deterministic/acceptance/test/files/citations/risk/next block present in each assigned source was read. E-1 and E-17 completion-rollup comments were included rather than skipped.
- **Feature-class completeness:** capabilities/actions/surfaces; workflows/states/lifecycle; schemas/data; flags/defaults/config; integrations/events/dependencies; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/concurrency/idempotency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; and status/deferred items are represented in the ledgers or explicitly identified as absent/ambiguous.
- **Structural-line handling:** headings, blank lines, Markdown separators, code-fence delimiters, and citation-only repetition were read but are not separate atoms. Read lists and file inventories are represented as integration/status atoms without duplicating each already-described capability.
- **Duplicate control:** a machine audit found 347 atomic-ID occurrences, 347 distinct values, zero duplicates, and contiguous sequences for every source prefix. Cross-source interactions, six inferences, 43 ambiguities, 26 conflicts, and 84 summary additions/corrections reference the atoms and add no second atomic count.
- **Explicitness control:** shipped/done/current-code assertions remain attributed document claims, not independently verified implementation facts. Inferences are isolated under their own heading; ambiguities and conflicts are not silently resolved.
- **Boundary control:** the only original files read were the nine manifest members. The manifest and summary were administrative/orientation inputs, and one completed generated D discovery was consulted only for formatting. No claim was imported from it; no companion report, implementation file, other E original, other original chunk, paper, or web source was read.
- **Independent completion re-audit:** source hashes/counts were rerun after extraction; every source’s end line was reached; all nine first/last ID ranges and per-prefix counts were machine-counted; the summary’s nine relevant session records were reconciled into 84 labeled additions/corrections.

### Output fingerprint

- Canonical output SHA-256, computed over this UTF-8 file with the 64-hex digest on this line replaced by sixty-four ASCII zeroes: 6418f137b047971f42798a0308f4e3392a692e5a9a2198a43c2cfed9fe7196e1
- A literal whole-file SHA-256 cannot be embedded as its own content without changing itself; the canonical replacement rule above makes the recorded digest reproducible. The final whole-file SHA-256 is reported in the completion handoff.
