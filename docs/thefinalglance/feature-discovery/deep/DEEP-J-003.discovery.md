# DEEP-J-003 exhaustive feature discovery

## Result and evidence discipline

- Chunk: DEEP-J-003, family J.
- Outcome: all 10 assigned files were read completely; all manifest hashes, physical line counts, byte counts, and aggregate totals match.
- Evidence tags: **[E]** explicit source statement; **[I]** bounded inference from multiple explicit statements; **[A]** ambiguity, unresolved choice, moving status, or conditional contract; **[C]** internal conflict or correction that must not be silently reconciled.
- Anchors are physical line numbers in the named assigned source, not anchors in the J summary.
- The grounding map is treated as first-class evidence. The orientation file, 10-J-SUMMARY_INDEX.discovery.md, was consulted only at its nine relevant entry blocks; it is not source evidence.
- Repetition inside a plan is normalized into one canonical atom with all relevant anchors. Interaction and delta sections refer back to canonical atoms instead of redefining them.

## Exact processed manifest and verification

Manifest section: DEEP-SOURCE-MANIFEST.md lines 909-925. Declared chunk total: **10 files, 1,229 logical lines, 134,050 bytes**, target status **under 1,800**.

| # | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---:|---|---:|---:|---|---|---|
| 1 | auxiliary/grounding/fixture | 100 | 7,977 | 98910181c58542183e5619761e63ba9f7a4c56fffc365f6e2782434969d308f7 | docs/thefinalglance/J-plan/_grounding/J-HARNESS-GROUNDING-MAP.md | exact |
| 2 | numbered/index original | 230 | 16,685 | a4c241e9a9946b185a218220d450d257dbac8d35638d04382883353dd12f89cd | docs/thefinalglance/J-plan/J-8_vlm_perception_research_upgrade.md | exact |
| 3 | numbered/index original | 229 | 17,490 | 1b516ccc21c0181f06337677ffef148b99f2c3d02ac2ec6f49daebe519f46e38 | docs/thefinalglance/J-plan/J-10_insight_backprop_tree_memory.md | exact |
| 4 | numbered/index original | 125 | 9,341 | 76408a6e4ec22c7fab76dbaf6b4df545a6c5b8e9b9ca8b0229d1c87fbb6f139a | docs/thefinalglance/J-plan/J-33_mineru_document_parsing_lane.md | exact |
| 5 | numbered/index original | 71 | 16,351 | 1c3073dbb831d8e809844d07c349801ebbd5875daa7f0d630e14eab78e1e4fdd | docs/thefinalglance/J-plan/J-35_intent_control_layer.md | exact |
| 6 | numbered/index original | 66 | 7,764 | 6b8cd27ea659dc10f932751cc88e77833be45655b373154be32ec65e09edca0c | docs/thefinalglance/J-plan/J-75_rrf_ablation_harness.md | exact |
| 7 | numbered/index original | 79 | 12,417 | 3ec78ac4c80c068d60d18e01153185a18251cd44eb8fa27da81df535fc7c6907 | docs/thefinalglance/J-plan/J-79_grill_signal_interlace.md | exact |
| 8 | numbered/index original | 76 | 11,669 | 20377cde67d5ebaf3facefc3effc439c9686679ed926c69efd6c6d5613eade62 | docs/thefinalglance/J-plan/J-87_journal_completeness_substrate.md | exact |
| 9 | numbered/index original | 135 | 17,381 | 8369ee3a2091f0c723af3c005a963bc587a5c77035a784441796d8be8818af04 | docs/thefinalglance/J-plan/J-96_verified_case_graph_core.md | exact |
| 10 | numbered/index original | 118 | 16,975 | f8a354e89efa1bcf777d5c1dc82c096209ec106488af9e2bb2ba321f64ad4c4c | docs/thefinalglance/J-plan/J-107_longcot_recursive_scaffold_bench.md | exact |
| **Total** | **1 auxiliary + 9 numbered originals** | **1,229** | **134,050** | — | — | **exact** |

Verification commands were read-only sha256sum and wc -l -c over the ten literal paths. Read coverage was 1-100, 1-230, 1-229, 1-125, 1-71, 1-66, 1-79, 1-76, 1-135, and 1-118 respectively. No other numbered/original J-plan file was opened.

## Per-file atomic ledgers

### J-HARNESS-GROUNDING-MAP.md — 100/100 lines

- [E] **HGM-001** [L1-L5] The map is the single-track grounding for J-1..J-7, dated 2026-06-10; it combines a 14-file preread with live harness and targeted SSB greps, requires J detail files to cite its IDs, and marks PLAN-only anchors for executor re-grep.
- [E] **HGM-002** [L10-L13] The harness has 26 stdlib-only Python tools, including impact, wiring, freeze, status, link, skill-validation, cross-language, taxonomy, and codebase-map tools, but no undo, handoff-ledger, or backup tool; J-3/J-4 are builds.
- [E] **HGM-003** [L14-L17] The harness has exactly 11 named skills and no incremental-eval or see skill; J-6 and J-1's harness leg are net-new skills.
- [E] **HGM-004** [L18-L23] The live hook seam is the stated pre_tool_check.py path; registration is through settings.local.json; supported families are PreToolUse, PostToolUse, Stop, and PreCompact; per-session hook-executions.log has a global fallback.
- [E] **HGM-005** [L24-L28] PostToolUse already writes per-session change state, evidenced by the wire-remediation handoff; J-4 must parse both state.json and hook-log shapes defensively and fail visibly.
- [E] **HGM-006** [L29-L30] J-plan was absent before the authoring session, B through I were occupied, and J was collision-free.
- [E] **HGM-007** [L31-L34] The missing F: prefix in the hook gate is a real, deliberately unfixed bug; an attempted repair was byte-faithfully reverted, J-3 must not smuggle in the fix, and the issue remains a user decision/open question.
- [E] **HGM-008** [L39-L43] The 2026-06-10 runtime snapshot is SCHEMA_VERSION 221 and TOTAL_VARIANTS 532; the rapid 234/511 through 221/532 history makes hard-coding forbidden and edit-time rereading mandatory.
- [E] **HGM-009** [L44-L47] PLAN-asserted runtime anchors describe discarded pre-images, editor_fs_override=None, a missing streaming validator, flat 100K compaction, and a PreCompact no-op; every anchor must be re-grepped.
- [E] **HGM-010** [L48-L51] CLI MCP client/glue files exist and partially land C-39's client direction; J-2 is the missing server inverse and should first try to reuse cli_mcp_dispatch/server_started/server_stopped rows.
- [E] **HGM-011** [L52-L54] B-25's ElaiNeuron task_submit/task_status/pipeline_result/health_check list is authored but Empire-blocked; J-2 may reuse the list without touching or un-deferring B-25.
- [E] **HGM-012** [L55-L59] J-2 has a moving security pre-gate: E-24 was shipped, G had eight wired/two dark items, E-30 and G-3 needed fixtures, and C-42 was unverified; all require live recheck.
- [E] **HGM-013** [L64-L66] SRC-1 is the harness redesign recommendation plus its per-file reading log.
- [E] **HGM-014** [L67-L70] SRC-2 is the external-repo steal recommendation/evaluation, the skillspector interface map, and the named external clone/summary roots.
- [E] **HGM-015** [L71-L72] SRC-3 is the vision research summary plus screenshot; its figures are external design rationale, never ELAI-measured.
- [E] **HGM-016** [L73-L75] SRC-4 is the wire-remediation handoff plus FINISH master index; status is moving and every status citation must be rechecked at execution.
- [E] **HGM-017** [L80-L81] J-3 substitutes stdlib gzip for the redesign's non-stdlib zstd unless a compression dependency is approved.
- [E] **HGM-018** [L82-L86] Harness hashing defaults to keyed hashlib.blake2b because py-blake3 is an approval-gated dependency; stored algorithm prefixes make a later blake3 migration additive, and the Rust blake3 rule is explicitly distinguished.
- [E] **HGM-019** [L87-L89] Harness J-3..J-7 add zero SSB variants; J-1 adds PerceptionScaffoldEvent; J-2 is constrained to reuse or at most one new row; anything more is a scope violation for that J-1..J-7 track.
- [E] **HGM-020** [L90-L92] J-1's see skill must anti-slopsquat-check Pillow, degrade to raw-image passthrough plus a warning if absent, and never hard-fail an image read.
- [E] **HGM-021** [L93-L95] Every new skill must pass agentskills_validator.py frontmatter validation and ship positive.jsonl and negative.jsonl evals; this binds J-6 and the see skill.
- [E] **HGM-022** [L96-L98] J-5 is a port checklist, not a build; its first gate is rechecking seven liteGAME compatibility files.

### J-8_vlm_perception_research_upgrade.md — 230/230 lines

- [E] **J008-001** [L1-L5] J-8 is PLANNED for WAVE 3+, upgrades J-1 only after J-1 is DONE, splits Tier 1 rules/schema/data from Tier 2 model evaluations, and gates the latter on sidecar/RunPod availability.
- [E] **J008-002** [L6-L18] Its concrete scope is resize data, normalized coordinate schema, DOM-owned counts, multi-image/micro-text routing, MoAI citation, I-10 cache composition, two candidate evals, a prompt template, and J-9 catalog seeds; all corpus numbers remain design-only until ELAI measures them.
- [E] **J008-003** [L15-L18] J-8 must not edit the J-1/E-36/H-14/E-51/B-24 plan files or reimplement J-1 gateway structs.
- [E] **J008-004** [L20-L25] Tier 1 is estimated at 6-8 hours plus a gated eval session; J-8 is parallel-safe with J-3/4/5/6/7, not with J-1, and must sequence or coordinate its shared vision-ladder seed with J-9.
- [E] **J008-005** [L29-L38] The source corpus contributes DATA, RULES, and EVALS; all quantitative corpus content is external, while fixture deltas are ELAI measurements.
- [E] **J008-006** [L40-L55] J-8 composes planned J-1, shipped B-24, planned/partly in-tree E-36, harness visual-loop law, planned I-10/H-14/E-51, and the then-live SSB 226/537 snapshot.
- [E] **J008-007** [L57-L64] Coding is gated on reading the full perception map, original three corpus files rather than their summary, live J-1 structs, E-36 baseline code, visual-loop law, and anti-slopsquat protocol.
- [E] **J008-008** [L67-L71] sweet_spot.rs reads sweet_spots.toml rows with longest_side_px and native_res; initial family data are Claude 1568, GPT-4o 2048, Qwen-VL 1280, and Gemini native, each provider-rechecked; native_res=true bypasses resizing.
- [E] **J008-009** [L72-L75] Scaffold schema v2 requires coordinates ordered as [ymin,xmin,ymax,xmax] normalized to 0-1000; values over 1000 or px-tagged output fail, re-prompt once, then return a typed refusal; instruction precedes image.
- [E] **J008-010** [L76-L78] A count-shaped request over ten elements uses AXTree/DOM or a deterministic script, never a VLM; absent DOM support yields typed D-16 refusal instead of guessing.
- [E] **J008-011** [L79-L81] More than one evidence image routes to a tiled composite or B-24 Mode B; micro-text routes through tiling and then a confidence-gated typed refusal; both journal route Kind and honest residue.
- [E] **J008-012** [L82-L84] A debug assertion places image plus scaffold after the last cache breakpoint; J-8 composes with I-10 but does not create another cache guard.
- [E] **J008-013** [L85-L89] Tier 2 adds elai bench vlm-eval with ocr and vision-rag lanes, comparing the E-36 baseline with anti-slopsquat-resolved dots.ocr/Qwen3-VL-Embedding; it records accuracy/latency in metrics TOML plus scheduled/result rows, and adoption remains a separate user decision.
- [E] **J008-014** [L90-L92] prompt_shapes.rs exports a component-inventory template with type, verbatim text, hierarchy, and layout cues plus a MoAI citation for future H-14 use.
- [E] **J008-015** [L93-L95] vision_ladder.toml receives VR-13 and FastVLM seed rows for J-9; the file is shared and requires coordination.
- [A] **J008-016** [L96-L99] The event is either a new VlmPerceptionUpgradeEvent with SchemaApplied/ResizeRuleApplied/RouteRuleApplied/EvalScheduled or a payload extension of J-1's event if its carrier exists; the net SSB delta is unresolved until edit.
- [E] **J008-017** [L100-L102] J-8-decisions.md records chosen table data and provider checks, schema v2, candidate-resolution status, and which figures remain external.
- [E] **J008-018** [L104-L123] Produced symbols are the two TOML datasets, sweet-spot/prompt modules, schema/router changes, eval harness, and event; consumed seams are J-1 gateway types, B-24 Mode B, E-36 baseline, D-16 refusal, and I-10; runtime reachability is every J-1 image dispatch plus the bench CLI.
- [E] **J008-019** [L124-L130] Stub bans prohibit magic pixel constants in Rust, any VLM counting call, candidate adoption without measured row and user assent, corpus figures as product metrics, forbidden plan edits/reimplementation, production unwrap/expect, and unapproved dependencies.
- [E] **J008-020** [L131-L134] Upgrade and eval rows reuse J-1's perception drawer tile, or inherit and restate J-1's justified frontend absence.
- [E] **J008-021** [L135-L147] The positive run must journal validated normalized coordinates, table/native resize routing, and DOM count routing; proof artifacts show zero VLM count call and no resize for native models.
- [E] **J008-022** [L144-L147] Negative arms require legacy pixels to fail, forced single-image routing to degrade relative to proper multi-image routing, and unresolved candidate identity to refuse bench startup.
- [E] **J008-023** [L149-L158] Deterministic checks run the wiring trigger, prove TOML wiring/no pixel magic in Rust, locate normalized schema, ban production unwrap/expect, and require the seed and decisions files.
- [E] **J008-024** [L160-L172] Acceptance additionally requires J-1 DONE, native passthrough, one-retry refusal, DOM-derived counts, route residues, cache assertion, measure-only eval/user gate, no external figures as claims, complete event lockstep, workspace/auditor green, and no new dependency.
- [E] **J008-025** [L174-L185] Rules/schema are default ON on the gateway, eval is an invoked entrypoint rather than a flag, originals supply tables, all paths/baselines are re-read, all rows are journal-recomputable under ELAI_SEED, and harness JD clauses are N/A.
- [E] **J008-026** [L187-L194] The contract suite covers family resolution/native passthrough, bad coordinates and bounded retry, DOM count dispatch, multi-image discrimination, unresolved-candidate refusal, and corrupt/missing-family TOML degrading to DegradedPassthrough without panic.
- [E] **J008-027** [L196-L203] Planned file effects create or modify the named perception modules/data, CLI eval, SSB lock files, contract test, and decision document.
- [E] **J008-028** [L205-L213] Corpus and paper citations are grounding only; ELAI ground truth is the E-36 baseline/J-1 contract/live snapshot, while anti-slopsquat, no-new-deps, blake3, measured-only, and token discipline bind execution.
- [A] **J008-029** [L215-L217] Exact candidate package/HF identities are unresolved; Tier 2 is blocked on anti-slopsquat, but Tier 1 may ship independently.
- [E] **J008-030** [L218-L219] Provider preprocessing drift is handled by TOML rows carrying a recheck date and edit-time provider verification.
- [A] **J008-031** [L220-L221] J-1 is still planned, so its final structs and whether it supplies a reusable event carrier are unknown and must be rediscovered.
- [E] **J008-032** [L222-L223] Schema repair cost is bounded to one retry then refusal and must be measured before stricter behavior.
- [E] **J008-033** [L225-L228] J-8 closes J-1's abstract calibration gap, seeds J-9, and lets measured eval rows inform later user-gated B-24 repinning.

### J-10_insight_backprop_tree_memory.md — 229/229 lines

- [E] **J010-001** [L1-L5] J-10 is PLANNED WAVE 3, with node field and audit extraction first and ideation injection only after I-4 v1; it is described as the Arbor steal-list's genuinely new capability.
- [E] **J010-002** [L7-L18] It ports four mechanisms only: bounded insight extraction, verdict-time ancestor synthesis to a root memory, lineage injection into execution, and a three-part ideation constraints block; H-49/SessionDag, I-4 receipts, and I-11 handoff digest remain owners of their existing concerns.
- [E] **J010-003** [L9-L11] Arbor's -27pp/-18pp ablations are external rationale and may not become ELAI targets or claims.
- [E] **J010-004** [L20-L27] Field/extraction/up-synthesis can start from H-49/SessionDag; ideation injection depends on I-4; J-10 is parallel-safe with J-3/4/6/7/9 and coordinates with I-4/I-11.
- [E] **J010-005** [L31-L36] Extraction uses one small LLM call, a verbatim structured-fields contract, a 1-3 sentence result, and deterministic head+tail truncation around 12K source characters.
- [E] **J010-006** [L37-L39] Every verdict walks to root and resynthesizes each ancestor from child insights under a roughly 200-word render, making root live global memory.
- [E] **J010-007** [L40-L46] Execution gathers ancestor lineage; ideation renders ROOT INSIGHT priors, PRUNED LESSONS with an explicit no-repeat-without-countermeasure instruction, and VALIDATED FINDINGS treated as trunk assumptions.
- [E] **J010-008** [L48-L52] ELAI adds anti-gaming absent from Arbor: insight source-row hashes are recomputed, mismatches are flagged, and unverified lessons never silently gain trust.
- [E] **J010-009** [L54-L66] J-10 generalizes I-5 contrastive constraints, keeps I-11 cross-handoff distinct, enriches I-4 loser receipts, feeds but does not prebuild I-12 playbooks, reuses H-49 storage, and found no existing insight action type at the 226/537 snapshot.
- [E] **J010-010** [L68-L85] Mandatory reads use the Arbor source code—not the steal-list prose—as ground truth and re-grep the live node, receipt, constraint, digest, prompt, and SSB seams.
- [E] **J010-011** [L87-L90] The node gains insight text plus hashes of source SSB rows in the existing store, using Rust blake3 only.
- [E] **J010-012** [L91-L95] Every audit/experiment verdict triggers fixed-contract extraction; malformed LLM output is a typed error, never an invented insight.
- [E] **J010-013** [L96-L98] Ancestor synthesis is bounded by ELAI tree depth at most four and targets the local sidecar.
- [E] **J010-014** [L99-L104] Both executor and post-I-4 ideation injection occur after the last cache breakpoint; ideation reuses I-5's render and cited wording.
- [E] **J010-015** [L105-L107] Provenance verification is deterministic; absent or forged hashes produce visible UNTRUSTED state and suppress consumption.
- [E] **J010-016** [L108-L111] InsightPropagated is a Pattern-2 event with Extracted, AncestorResynthesized, ConstraintsInjected, and PrunedLessonBound Kinds and full six-part lock discipline.
- [E] **J010-017** [L112-L114] J-10-decisions.md freezes prompt/schema, local truncation budget, synthesis bound, external-only figures, and provenance rationale.
- [E] **J010-018** [L116-L133] The pipeline produces the field, extract/synthesize/inject module, constraints renderer, and event; it consumes H-49, I-4/I-5, dispatch/ranker seams, and verdict SSB rows; it fires on auditing, executing, and ideation production paths.
- [E] **J010-019** [L134-L141] Stub bans forbid model-authored trust, unproven injection, external ablation claims, production unwrap/expect, and non-blake3 provenance hashes.
- [E] **J010-020** [L142-L144] The frontend is an insight column/lineage tooltip on an existing plan or branch drawer, with exact drawer decided at edit or explicitly waived.
- [E] **J010-021** [L145-L154] A two-sibling fixture must show pruned lessons in the next prompt and both outcomes in root; forged provenance is excluded, and no-insight must re-propose the pruned idea to prove load-bearing effect.
- [E] **J010-022** [L156-L164] Deterministic checks validate reachability, both SSB locks, absence of production unwrap and non-blake3 hashing, and existence of the decision record.
- [E] **J010-023** [L166-L179] Acceptance requires sequencing, provenance/trust separation, cache stability, ELAI-only metrics, SSB lockstep, frontend cross-language checks, two short local 7B sidecar operations, workspace/auditor green, and no unapproved dependency.
- [E] **J010-024** [L181-L189] Contract tests cover bounded extraction, sibling-to-root synthesis, pruned lesson reinjection, forged-hash rejection, all event Kinds, a no-insight control, and journal-content tracing assertions.
- [E] **J010-025** [L191-L198] Planned edits create insight modules and a decisions file and modify existing tree, receipt, prompt-dispatch, and SSB lock structures.
- [E] **J010-026** [L200-L209] Source-code anchors outrank the steal-list; external token/ablation facts are rationale; all live plan status is stale-by-construction and rechecked.
- [E] **J010-027** [L211-L214] If I-4 is late, field/extract/up-synthesis/executor injection still ship; only the ideation block waits.
- [A] **J010-028** [L20-L22,L91-L98,L176-L177] The estimate calls the work “two short LLM calls,” while the algorithm explicitly resynthesizes every ancestor and can therefore invoke more than two calls per verdict; implementation must clarify whether “two” means call classes or actual invocation count.
- [E] **J010-029** [L215-L216] Arbor's 12K truncation assumption must be retuned for the local 32K lane and recorded.
- [E] **J010-030** [L217-L218] I-11 overlap must be audited so the same insight is not both synthesized and redundantly digested into one handoff row.
- [A] **J010-031** [L219-L221] Weak 7B ideation quality may route only the ideation hypothesis turn to cloud; extraction/synthesis remain local, and always-on behavior waits for measurement.
- [E] **J010-032** [L223-L227] The result is a teaching role for loser branches, an I-12 precursor, and no port of Arbor's large architecture; remaining steal-list amendments stay in J-11.

### J-33_mineru_document_parsing_lane.md — 125/125 lines

- [E] **J033-001** [L1-L5] J-33 is PLANNED WAVE DP-1, an activation/build of a shipped-dark parser backend; Tier 1 is medium effort, Tier 2 gated, and any external service dependency needs user confirmation plus license verification.
- [E] **J033-002** [L7-L20] MinerU is not a UI-screenshot visual-loop unit; it fills the document lane where B-35 and harness ingestion are text-only and E-36's ParserBackend::MinerU plus eligibility/fallback machinery has zero production callers.
- [E] **J033-003** [L21-L25] MinerU is described as parsing PDF/DOCX/PPTX/images to structured markdown and content_list.json with layout, multilingual OCR, TeX formulae, HTML tables, reading order, CPU pipeline, 8GB-minimum 1.2B VLM mode, and FastAPI POST /file_parse.
- [E] **J033-004** [L27-L34] The v3 code license is a “MinerU Open Source License” based on Apache-2.0 plus four clauses, with stated commercial thresholds and service attribution; code may only be consumed as an external opt-in service, never vendored; Flash-MinerU is banned.
- [A] **J033-005** [L31-L34] The MinerU2.5 model-card license remains unverified and blocks Tier 2; Tier 1 pipeline does not need it; an about/docs attribution line is a deliverable.
- [E] **J033-006** [L38-L43] DocParserClient uses reqwest multipart HTTP against mineru serve; eligibility checks reachability/mode; failure follows compute_fallback_recommended and reports unparsed rather than indexing binary garbage.
- [E] **J033-007** [L44-L47] B-35 adds PDF/DOCX routing through structured parsing, existing capped chunkers, and ArtifactStage; Cosmos inherits support through its existing path.
- [E] **J033-008** [L48-L53] Parsed text/table/formula output becomes E-36's deterministic first leg; only unresolved claims escalate to VLM, and J-33 does not flip the whole ELAI_DOCUMENT_VISION_RAG_ENABLE gate.
- [E] **J033-009** [L54-L57] Harness pdf_ingest.py may parse raw papers to markdown before mcp_ingest_file; if the service is unavailable it falls back to the present HTML path, with tool logs and chunk-count delta as off-agent evidence.
- [E] **J033-010** [L58-L63] MinerU's OmniDocBench figure is self-reported design rationale; a 5-10 PDF ELAI fixture set measures parsed-vs-read_to_string retrieval, including formula and table cases, and stores metrics in TOML.
- [E] **J033-011** [L62-L63] J-8 retains ownership of dots.ocr/Qwen3-VL comparisons; J-33 only supplies their baseline lane.
- [E] **J033-012** [L65-L80] The code snapshot marks parser types shipped but unwired, document vision dark, ArtifactStage and Cosmos wired, harness/B-35 text-only, and J-1/J-8 planned-only; all anchors and external maturity/performance claims require edit-time recheck or local measurement.
- [E] **J033-013** [L82-L87] Tier 1 ships CPU pipeline HTTP service, artifact wire, harness leg, and fixture eval; it is opt-in, anti-slopsquat checked, default behavior is retained when unconfigured, and the absence is made loud.
- [E] **J033-014** [L88-L90] Tier 2 requires remote-3090 hardware notice and VRAM accounting, model-license verification, deeper E-36 composition, and H-5-style async task batching for Cosmos refresh.
- [E] **J033-015** [L92-L98] Runtime integration is in crates/knowledge plus inherited Cosmos routing; knowledge.doc_parser has service_url, backend pipeline|vlm, and enabled=false, and requires a wire-dark allowlist entry with owner/promotion plan/smoke.
- [C] **J033-016** [L99] The harness line says “stdlib + requests”; requests is not Python stdlib. Dependency policy and the intended no-new-dependency/fallback implementation are therefore internally inconsistent and require resolution.
- [A] **J033-017** [L100-L102] SSB should reuse existing E-36/pdf rows if present; otherwise add DocumentParsedEvent with Parsed, ParseFailed, FallbackUsed, and EvalMeasured, for a net 0..+1 after rereading the live baseline.
- [E] **J033-018** [L103-L104] KnowledgeDrawer adds parser badges and a parse-failure surface.
- [E] **J033-019** [L106-L116] Done means real formula-heavy PDF indexing produces known TeX, a Parsed row, retrievable B-35 content, and an ELAI metrics delta; paper ingestion must beat its HTML fixture on chunk count/formula fidelity.
- [E] **J033-020** [L113-L116] Negative proof requires service-down ParseFailed plus deterministic fallback, no binary-garbage index, rejection of Flash/vendored code, and explicit unchanged-mode reporting on unconfigured installs.
- [E] **J033-021** [L118-L125] Office formats may ride the wire but evaluation is PDF-first; visual-loop screenshot OCR is declined and belongs to J-1 if revived; ELAI uses FastAPI rather than MinerU MCP to avoid G-4/E-24 pins.

### J-35_intent_control_layer.md — 71/71 lines

- [E] **J035-001** [L1-L5] J-35 is PLANNED WAVE CFW-3, a large build on shipped fidelity/rules/journal substrate, with soft J-32 routing and Tier-2 J-31 card dependencies plus B-40.
- [E] **J035-002** [L7-L9] It sits before decomposition because ELAI's resident skeleton can faithfully propagate a wrong aim; existing prompt-transform, dark IdeaFramer, user-only grill/cook, and lane routing do not reconstruct or validate intent.
- [E] **J035-003** [L11-L17] True stakeholder aim cannot be machine-verified; the product may force a separate premise surface, build an aim-adjacent floor, and expose residuals, but must never emit an aim-verified/right-problem verdict.
- [E] **J035-004** [L14-L17] Premise checking is a separate structural pass because spontaneous model objection is framing-sensitive; the LLM proposes reconstruction/premises, deterministic checks and user/journal dispose, and proposal nondeterminism is recorded.
- [E] **J035-005** [L19-L21] IntentSpec is a visible typed artifact with goal, constraints, hidden_assumptions, non_goals, and validation_contract, reusing BabyStepSpec/IntentPair/WiringContract if possible and introducing types only when absent.
- [E] **J035-006** [L21] Reconstruction happens before retrieval and plan-skeleton decomposition; it is shown, because a hidden rewrite can impose the system's framing.
- [E] **J035-007** [L23] Candidate premises are extracted independent of requested output format and deterministically checked against DKG plus FILE/SOURCE/TEST/WEB methods; a contradiction surfaces with evidence.
- [E] **J035-008** [L23] In active blocking mode, a false-premise flag stops decomposition until user confirmation/correction rather than tunneling into execution.
- [E] **J035-009** [L25] RouteDecision deterministically combines J-32 confidence, AmbiguityScorer, premise flags, reversibility/cost, GateMode, and RiskLevel into ask, proceed-under-assumptions, or refuse/escalate.
- [E] **J035-010** [L25] Interactive ASK is reserved for questions expected to improve execution when the action is costly or irreversible; lower-cost reversible ambiguity proceeds under logged assumptions.
- [E] **J035-011** [L27] Autonomous ambiguous defaults are derived from similar user outcomes in SSB, never a population prior; absent signal selects the least-presumptive reversible action, while high/irreversible work uses high_risk_pauses.
- [E] **J035-012** [L27] Every injected assumption becomes an auditable, overridable AssumptionLogged row.
- [E] **J035-013** [L29] The decision card contains reconstructed goal, route/lane, assumptions, premise flags, reversibility, and intervention handle; it exposes decisions before decomposition, not chain-of-thought.
- [E] **J035-014** [L31] Premise rules and sparse user-family aim patterns are data in the existing rules engine, learned from outcomes, garbage-collected, and promoted only PROPOSED/CANDIDATE/SHADOW/ACTIVE/RETIRED with fpr<5%, tpr>80%, causal>50%.
- [E] **J035-015** [L33-L42] Verified substrate includes Slot-3 skeleton context, one-hop adjacent contracts, staleness warnings, planned J-31/J-32, shipped-dark B-40, shipped rules/D-16/GateMode/RiskLevel, journal outcomes, typed specs, and phase-2B verifiers; all are re-grepped.
- [E] **J035-016** [L44-L46] Tier 1 ships model-light Rust spec, premise, route, and assumption functions without J-31/J-32, initially OBSERVE-ONLY for at least three real runs to bound false positives/over-blocking.
- [E] **J035-017** [L46] ELAI_INTENT_CONTROL_ENABLE defaults off and must have a complete experimental allowlist row; without J-31/J-32 the card uses chat/CLI and routing visibly falls back to current decomposition.
- [E] **J035-018** [L47] Tier 2 adds J-31/J-32 card/chip integration, SHADOW-to-ACTIVE aim-pattern learning, and conformal-tau calibration from chip tuples.
- [E] **J035-019** [L49-L54] intent_control.rs owns assembly, premise checks, routing, assumption journaling, and journal defaults at the task-lane seam between J-32 and decomposition; UI requires cross_lang_diff and svelte-check.
- [E] **J035-020** [L54] IntentControlEvent Kinds are SpecReconstructed, PremiseFlagged, RouteDecided, and AssumptionLogged; payload fields are spec_id, premise_flag_count, route_decision, assumption_count, reversibility_class, and gate_mode, never a “true goal.”
- [E] **J035-021** [L54] IntentControlEvent follows IntentRouted rather than replacing it; lane choice and within-lane intent control are explicitly distinct; full SSB/golden lockstep is required.
- [C] **J035-022** [L54] The source corrects stale 241/550 figures (including J-32's) to a then-live SCHEMA 268 / action types 577 and warns that TOTAL_VARIANTS auto-followed beyond a hand-tracked 564; both lock sites must be reread.
- [E] **J035-023** [L56-L58] Positive scenarios cover DKG-proven false premise, journal-derived reversible autonomous default, high-risk autonomous escalation, recomputability, and visible reconstruction.
- [E] **J035-024** [L58-L59] Observe-only initially emits a nonblocking premise notice; only after the three-run bound does the same flag block decomposition.
- [E] **J035-025** [L59] Negative contracts reject silent flag-off pseudo-checks, format-conditioned checking, population defaults, aim-verification verdicts, and hidden goal rewrite.
- [E] **J035-026** [L61-L67] Riders reuse J-26/D-6 evaluation, await J-28 lane tags for cloud defaults, defer B-39 reflection, preserve user-only grill/cook, and leave multi-turn state to SessionDag rather than rearchitecting memory.
- [E] **J035-027** [L69-L71] External front-door papers remain verify-pending design rationale; load-bearing evidence is ELAI code plus validation/verification logic, and no external number becomes a target.

### J-75_rrf_ablation_harness.md — 66/66 lines

- [E] **J075-001** [L1-L5] J-75 is a WAVE CR-3 instrumentation extension, not deletion; it tests the over-fused retrieval stack using ELAI data because cited paper gains/costs are design rationale only.
- [E] **J075-002** [L7-L8] Bench flags ablate PageRank, the second dense encoder, or graph ranking/navigation participation and measure MRR and Recall@5 over J-72's gold set.
- [E] **J075-003** [L9] elai bench rrf-ablate emits deterministic per-configuration deltas and never uses an LLM judge.
- [E] **J075-004** [L10-L11] Any actual prune requires both favorable ELAI measurement and explicit user sign-off; cross-encoder is never a prune target, silent/paper-number-driven removal is excluded.
- [E] **J075-005** [L13-L17] The measured subject is BM25, two ONNX dense encoders, PageRank, RRF k=60, cross-encoder, and E-40 SCIP; existing conditional HyDE/deferred SPLADE establish measure-first precedent, and J-72 supplies fixtures/plumbing.
- [E] **J075-006** [L19-L21] Tier 1 is the complete deliverable: an in-process model-free harness and report; this unit has no prune tier.
- [E] **J075-007** [L23-L25] Ablation variants are allowlisted, bench-only, and forbidden from production defaults; CLI implementation may live in knowledge or orchestrator but reuses J-72 plumbing.
- [A] **J075-008** [L26] Event ownership is decided at edit: prefer ConvergenceCurveMeasured/C-20 measurement rows for zero new action types; the report is the persistent artifact.
- [E] **J075-009** [L27] UI uses ExternalBenchPanel/CodingMetrics or documents a W-6 absence; no cross-language pair is otherwise needed.
- [E] **J075-010** [L29-L39] Produced symbols are three RRF toggles plus the bench/report; consumed symbols are J-72 gold/delta plumbing, shipped full cascade, and I-27/C-20 measurement shape; only real CLI reachability counts.
- [E] **J075-011** [L40-L46] Stub bans restate no removal, no cross-encoder target, no paper threshold, no LLM judge, and no production-default toggle leakage.
- [E] **J075-012** [L47-L50] The done run executes all three ablations against full stack, journals deterministic MRR/Recall rows, and proves no production config changed.
- [E] **J075-013** [L52-L57] CMU/CodeRAG figures and stack survey are grounding only; the live SSB estimate is moving and must be reread.
- [E] **J075-014** [L59-L60] J-75 composes J-72/J-74, existing conditional/deferred retrieval controls, I-27/C-20, and E-49 without restating them.
- [A] **J075-015** [L62-L63] Delta validity depends on gold-set representativeness; reports must disclose corpus composition rather than transfer the paper's predicted loss.
- [E] **J075-016** [L64] A contract assertion must keep production RRF full-stack until a separately authorized prune lands.

### J-79_grill_signal_interlace.md — 79/79 lines

- [E] **J079-001** [L1-L5] J-79 wires existing ambiguity signals to a grill offer; consent, not trigger source, is the invariant, so signals may offer but never run grill.
- [E] **J079-002** [L8-L10] Clarify rung 0 preserves J-32's single chip for one missing arg span; rung 1 offers one-line grill and waits when multi-branch B-38 is high, B-40 blocks, or C-29 cannot supply three intent pairs.
- [E] **J079-003** [L11] Grill output closes the detector/resolver loop: resolved decisions/glossary lower B-38, branch resolutions supply C-29 pairs, and contradiction resolutions clear B-40 after deterministic reevaluation.
- [E] **J079-004** [L12] Autonomous mode never runs grill mid-flight; when AutonomousGate escalates for ambiguity-grade evidence failure, the escalation names grill as the human resolution path.
- [E] **J079-005** [L13] The first harness change sharpens grill SKILL.md to distinguish forbidden auto-run from explicitly permitted signal-triggered offer.
- [E] **J079-006** [L14] Exclusions are auto-run, ubiquitous offers, and new detectors; offer thresholds must avoid consent fatigue.
- [E] **J079-007** [L16-L22] Existing status is planned J-32, wired B-38/C-29, dark B-40 and autonomous gate, and user-invoked grill; the interlace extends rather than rebuilds each.
- [E] **J079-008** [L24-L27] Tier 1 first sharpens the skill and enables only the strongest B-40-block offer, conservatively/observationally, while also shipping the full structured return path.
- [E] **J079-009** [L27] B-38/C-29 trigger widening waits until false-offer rate is bounded over at least three real runs; G2 follows interactive calibration.
- [E] **J079-010** [L29-L35] Integration adds a signal-to-rung selector, one-line wait surface, autonomous escalation payload, and deterministic gate consumers; IntentRouted stores offer/decline calibration with net zero new action types.
- [E] **J079-011** [L37-L52] Produced/consumed symbols bind J-32/B-38/B-40/C-29/B-22 and the skill text; real chat/plan or autonomous escalation is required, never test-only reachability.
- [E] **J079-012** [L53-L59] Stub bans forbid auto-run, unbounded false offers, new detection, new SSB action type, or accepting grill output as an LLM gate verdict.
- [E] **J079-013** [L60-L63] A live positive run offers once, waits, accepts resolved decisions, and passes only after gate rescore; negative proof records a clear-task decline, proceeds, and keeps autonomous behavior escalation-only.
- [E] **J079-014** [L65-L69] Grounding is ELAI signals/skill law, the three-run FP discipline, anti-sycophancy consent, and a moving SSB baseline.
- [E] **J079-015** [L71-L72] The feature is only the missing wire plus calibrated threshold; all signal and resolver owners remain unchanged.
- [A] **J079-016** [L74-L76] B-40 is both the first/strongest Tier-1 trigger and dark; until it is active, the file says the same-flag wire plus wired C-29 carries interaction, creating a sequencing ambiguity about whether C-29 is enabled before the stated widening gate.
- [E] **J079-017** [L77] The skill edit is harness-only, adds no SSB evidence, and is proven by its diff; it should land first.

### J-87_journal_completeness_substrate.md — 76/76 lines

- [E] **J087-001** [L1-L6] J-87 is PLANNED WAVE 1, XL, a K-21-pre-gated green-field keystone: hash-chain integrity cannot prove all real side effects emitted rows, so journal completeness is the missing trust substrate.
- [E] **J087-002** [L8-L9] The mediate leg routes a declared high-value set—write opens, egress connect/sendmsg, clone/execve, privilege change—through existing Tier-C seccomp user notification.
- [E] **J087-003** [L9] The supervisor commits the SSB row before releasing the kernel notification, guaranteeing atomic row/effect approval but explicitly not pointer-argument interception soundness because of TOCTOU.
- [E] **J087-004** [L10] A Linux-only independent eBPF/LSM-class feed observes effects outside the executor's authorship; cited overhead/tooling is design rationale.
- [E] **J087-005** [L11] Normalization handles temp churn, path aliases, and retries before a deterministic bounded-window observed-effect-to-row match; a covered missing row is FAIL, not unknown/skip.
- [E] **J087-006** [L12] Dropped counters, map/ring overrun, or observer crash yield ObserverImpaired and fail closed; startup self-tests the evidence feed.
- [E] **J087-007** [L13] Missing-row effects become canvas ANOMALY nodes for J-94/J-60.
- [E] **J087-008** [L14] Universal completeness, privileged host/kernel resistance, Windows observer parity, and any LLM reconcile role are explicitly excluded.
- [E] **J087-009** [L16-L21] The plan extends wired Tier-C, G-3, and single-writer SSB WAL, is governed by planned K-21 JC-1..4, and grounds kernel primitives externally with edit-time rechecks.
- [E] **J087-010** [L23-L25] New journal_completeness modules implement reconciliation/normalization, Linux observer/health, and mediation; the auditor owns the AUDITING and live-cli post-tick checks, not the main model.
- [E] **J087-011** [L26] JournalCompletenessEvent Kinds are EffectMediated, EffectObserved, ReconcileMatched, ReconcileMissingFAIL, and ObserverImpaired, with full nine-tuple lockstep after rereading the baseline.
- [A] **J087-012** [L27] ELAI_JOURNAL_COMPLETENESS_ENABLE is called a kill switch with default OBSERVE-ONLY; the file fixes the mode but does not unambiguously say whether an unset Boolean means enabled-observe or disabled.
- [E] **J087-013** [L27] The flag requires a kill-switch allowlist row with owner, expiry, J-87-promotion, and an ON-path smoke.
- [E] **J087-014** [L28] The audit/canvas completeness panel shows matched, missing-fail, and observer-impaired state and requires cross-language verification when surfaced.
- [E] **J087-015** [L30-L43] Named symbols are reconcile_effects, normalize_observed_effect, ObserverFeed, observer_health, and mediate_effect; they consume Tier-C, G-3 posture, and SSB and are reachable only through real auditing/live ticks.
- [E] **J087-016** [L44-L51] Stub bans enforce covered-scope wording, deterministic reconciliation, loud observer impairment, observe-before-block over at least three runs, Windows mediation-only truth, and missing=FAIL.
- [E] **J087-017** [L52-L55] Positive proof observes write/connect and matches rows; a suppressed-row fault blocks FSM advance; killing observer fails, normalized temp churn does not false-fail, and an empty feed fails startup.
- [E] **J087-018** [L57-L62] Reports, kernel mechanisms, and overhead numbers are design evidence; the only done proof is ELAI's injected missing-row failure; anti-duplication found zero prior equivalents.
- [E] **J087-019** [L64-L65] Completeness composes with but is orthogonal to hash-chain integrity and feeds RRA, non-LLM labels, canvas anomalies, and higher-level evidence gates.
- [E] **J087-020** [L67-L68] A privileged host/kernel can defeat both witnesses; Rekor/TPM-IMA is only a rider elsewhere, and no stronger claim is permitted.
- [E] **J087-021** [L69] Normalization failure creates an irreducible schema-mismatch false-positive floor; claims remain restricted to covered classes.
- [E] **J087-022** [L70] Windows is mediation-only, Linux remote execution gets the observer, and cross-lane guarantees must stay explicit.
- [E] **J087-023** [L71] Covered classes are frozen, user-reviewed data; adding a class is deliberate, every run logs coverage, and silent scope narrowing is forbidden.
- [E] **J087-024** [L73-L74] J-91, J-93, and J-94 are direct downstream consumers of the completed substrate.

### J-96_verified_case_graph_core.md — 135/135 lines

- [E] **J096-001** [L1-L8] J-96 is PLANNED WAVE 1, observe-only and mint/show only; it builds a typed verified-case graph over existing ingredients, while prompt reuse is J-97 and canvas altitude is J-98.
- [A] **J096-002** [L5] It depends on proof rows and J-10 insight text; it may sequence after J-10 legs 1-3 or ship the insight field independently, so exact scheduling is unresolved.
- [E] **J096-003** [L6-L7] Only gate-proven fixes enter; auditor/critic/adversary never receive case memory; superseded/stale cases are rederived and hard-filtered; use does not score or evolve the core.
- [E] **J096-004** [L10-L15] Minting requires an AUDITING red-to-green transition with matching F-4 stdout hash and a sealed G-2 RRA; I-37 requirements and B-29/J-16 symbols provide scope; model claims can never mint.
- [E] **J096-005** [L17-L23] CaseNode has six top-level fields: trigger_signature, blast_radius_cone, fix_delta, proof_receipt, provenance_hash, and insight; trigger_signature itself includes symptom, failing-test ID, and taxonomy class.
- [E] **J096-006** [L20-L23] fix_delta is structural edit/search-replace; proof_receipt contains F-4 stdout hash plus RRA blake3 root; provenance hashes source SSB rows; insight is 1-3 sentence model-written text, never verdict.
- [E] **J096-007** [L25-L31] Deterministic edge kinds are DKG/cone overlap, same session/plan-unit, same failure taxonomy, and same-symbol supersession; weights are recomputable from journal rows and never model-authored.
- [E] **J096-008** [L30] Superseded nodes remain for replay integrity and are filtered, not deleted.
- [E] **J096-009** [L33-L34] Every new failure gets deterministic typed-signature hit/miss matching; recurrence-hit-rate is matched/total and measures potential usefulness before any prompt influence.
- [E] **J096-010** [L34] K-18 bench case-reuse consumes this safe arm; reuse-on iteration/cost remains pending on J-97.
- [E] **J096-011** [L36-L44] The plan reuses wired F-4/G-2/B-29/taxonomy/I-37, planned J-16/J-10, partial C-45/C-47 and SessionDag, keeps C-25 pipelines distinct, and leaves I-12/H-43..48 reuse promotion/scoring to J-97.
- [E] **J096-012** [L46-L51] Mandatory reads include the source synthesis, SF rules, live proof/store/insight seams, taxonomy tool, and SSB locks, all re-grepped.
- [E] **J096-013** [L53-L55] CaseNode persists as journal-derived rows over the existing case base with blake3 proof/provenance, and the mint assembler combines proof rows, cone, taxonomy, and J-10 insight.
- [E] **J096-014** [L56] Edge derivation runs at idle/audit tick, never the hot path.
- [E] **J096-015** [L57] elai case-graph stats exposes node count and recurrence measure.
- [E] **J096-016** [L58] The SF firewall blocks reads from auditor/critic/adversary, superseded nodes are filtered, and proof-hash drift marks a case STALE.
- [E] **J096-017** [L59] CaseNodeMinted Kinds are CaseMinted, EdgeDerived, RecurrenceMatched, and CaseStaled under full lock discipline.
- [E] **J096-018** [L60] J-96-decisions.md records signature similarity, four edge rules, mint-only rationale, and firewall assertion point.
- [E] **J096-019** [L62-L79] Produced symbols are typed nodes, derivation, stats, SF guards, and event; consumed symbols are F-4, G-2, B-29, taxonomy, J-10, C-45/C-47, and SessionDag; only live audit/new-failure paths count.
- [E] **J096-020** [L80-L87] Stub bans forbid claim-based mint, model edges/weights, verifier-context exposure, stale/superseded reuse, any prompt injection, production unwrap/expect, and non-blake3 proof hashing.
- [E] **J096-021** [L87] UI shows node count and recurrence on an existing trust/audit panel; the full memory/case canvas belongs to J-98.
- [E] **J096-022** [L88-L91] A red-green fixture mints and a matching second failure records recurrence; negative arms cover failed F-4, forged provenance, verifier injection, superseded read, and self-report-derived edges.
- [E] **J096-023** [L93-L102] Checks prove real production caller reachability, event locks, firewall code, blake3-only hashing, and decision artifact.
- [E] **J096-024** [L104-L110] Acceptance enforces proof-only minting, byte-recomputable deterministic edges, observable stats with zero reuse, firewall/filtering, insight provenance, lockstep, workspace/auditor green, and no new dependency.
- [E] **J096-025** [L112-L113] The contract suite covers all six node fields, no-mint failure, byte-identical edges under ELAI_SEED, recurrence, firewall, supersession, forged provenance, and traced journal rows.
- [E] **J096-026** [L115-L119] Planned files create case_graph modules and a decision record and modify existing case store, audit/failure paths, and SSB locks.
- [E] **J096-027** [L121-L125] External self-improvement research is rationale; ELAI gates/rules supply ground truth; K-18 packages economics and K-19 supplies the binding pre-gate.
- [A] **J096-028** [L127-L129] Signature threshold is deliberately unset pending deterministic fixture tuning; v1 proof scope is red-green tests only, with wider completed-plan/research cases deferred.
- [E] **J096-029** [L130] Edit-time review must prevent parallel records with C-45/C-47 and duplicate extraction with J-10.
- [E] **J096-030** [L132-L133] The safe half only makes proven fixes visible and measurable; “improves as used” is exclusively the later J-97 gate.

### J-107_longcot_recursive_scaffold_bench.md — 118/118 lines

- [E] **J107-001** [L1-L5] J-107 is PLANNED WAVE RLM-1, a build on the K-15/I-27 runner and the gate for J-108; it compares constrained recursive scaffold with flat reasoning per model/lane using receipt outcomes.
- [E] **J107-002** [L5] It ships no new reasoning architecture or model-chosen orchestration and must reuse K-15, I-27, J-34, and I-38 rather than fork them.
- [E] **J107-003** [L7-L10] The adapter/driver/curve/knee is one session S-M; fixture/cloud work ships now, live local work is RunPod-gated; J-107 and J-108 may coauthor disjoint surfaces but the harness/flat arm comes first.
- [A] **J107-004** [L8,L22-L26] Required K-15 is described as PLANNED/BENCH-PENDING and I-27 as DARK while also treated as reusable runner/event/UI substrate; live availability must be re-grepped before implementation.
- [E] **J107-005** [L12-L13] The benchmark turns one-off RLM evidence into repeatable per-model/lane calibration and guards small local models against the J-55 CodeAct collapse.
- [E] **J107-006** [L15-L17] LB-1 is flat single-shot OFF at imposed budget; LB-2 is J-108's named-op slice, at most one child spawn, and HippoRAG hop, explicitly not free Python or model-chosen orchestration.
- [E] **J107-007** [L18-L20] Y axes are verified-outcome receipts, abstention rate, and tokens per correct; X axes are 0.5W/2W/8W-style bands and/or recursion depth; the knee is where more work stops turning abstentions into correct answers.
- [E] **J107-008** [L20] The per-model/lane knee configures J-108 and overthinking ceilings; paper 6K/7K values are forbidden as product constants.
- [E] **J107-009** [L22-L27] Existing K-15, I-27, J-34, and I-38 are siblings/dependencies; LongCoT GitHub/HF assets are MIT and are adapted rather than vendored wholesale.
- [E] **J107-010** [L29-L34] Mandatory reads include the full RLM grounding, source synthesis, experiment/result files as reference implementation, live runners/CLI, and measurement/reproducibility doctrines.
- [E] **J107-011** [L36-L37] The adapter fetches/caches LongCoT, uses deterministic exact-match/receipt scoring with no LLM judge, and mirrors imposed bands.
- [A] **J107-012** [L38] If J-108 is absent, LB-2 is a registered entrypoint stub and the gate explicitly runs flat-only; it may not silently imply two-arm coverage.
- [E] **J107-013** [L39] Curve rows are per model/lane/band/arm, may add only a discriminator Kind if necessary, and use at least five paired seeds.
- [E] **J107-014** [L40] args_bench.rs supplies the real elai bench longcot entrypoint and ConvergencePanel supplies or hosts the UI.
- [E] **J107-015** [L41-L42] The recorded gate artifact contains knee, LB-2-minus-LB-1 verified lift, and the requirement that local scaffold fabrication/code-error rate not exceed flat; J-108 reads this artifact.
- [E] **J107-016** [L44-L54] Produced symbol is longcot_runner plus CLI; consumed symbols are shared runner/curve/UI, J-34/I-38, J-108 primitive, and HF data; only the production CLI invocation counts.
- [E] **J107-017** [L55-L61] Stub bans forbid LLM judging, paper results, runner fork, wholesale harness copy, single-seed claims, treating unreachable abstention as failure, local free-Python CodeAct, and production unwrap.
- [E] **J107-018** [L62-L65] The live run uses five seeds and emits one row per tuple; artifacts include curve, knee, lift, and abstention; fabricated rowless results or worse scaffold fabrication fail.
- [E] **J107-019** [L67-L75] Deterministic checks inspect two-arm/knee/abstention code, assert no judge or paper constants, prove production reachability, and require a decisions file.
- [E] **J107-020** [L77-L82] Acceptance requires runner reuse, J-108 seam or loud flat-only registration, receipt/abstention/token metrics, measured gate artifact, paired seeds, deterministic scoring, license-clean adaptation, build/auditor green, and production reachability.
- [E] **J107-021** [L84-L90] Done is a real CLI run with rows and knee; no default-off runtime flag is needed for an on-demand bench; UI reuses ConvergencePanel; fixture/cloud is now and local GPU is a separate gate; all results are row-recomputable.
- [E] **J107-022** [L92-L93] Contract tests cover tuple rows, scorer determinism, distinct abstention, seed-stable knee, rejection of rowless results, collapse detection, and rejection of fewer than five seeds.
- [E] **J107-023** [L95-L101] Planned changes create the bench/contract/decision files, edit CLI/UI, and touch SSB locks only if a discriminator Kind is added.
- [E] **J107-024** [L103-L108] Sources are the RLM synthesis, LongCoT interface, existing runner surfaces, and ELAI experiment results; all paper figures remain non-product rationale.
- [A] **J107-025** [L110-L111] Subjectively scored LongCoT subsets must be excluded; the deterministic gate's covered subset and residue must be stated.
- [E] **J107-026** [L112] J-107 cannot claim scaffold lift before the real J-108 primitive exists.
- [E] **J107-027** [L113] Local Qwen-class calibration waits for RunPod with advance hardware notice; fixture/cloud measures structure, not that local knee.
- [E] **J107-028** [L115-L116] J-107 becomes the third calibration axis beside reasoning-budget/context-fill and feeds J-84/J-85 only with its measured knee.

## Cross-source interactions

The following are references to the canonical atoms above, not additional feature atoms.

| Interaction | Producer / rule | Consumer / consequence |
|---|---|---|
| Harness evidence discipline | HGM-001, HGM-004, HGM-005, HGM-021 | Explains why harness legs such as J033-009 and J079-017 require off-agent artifacts rather than runtime SSB rows. |
| Moving schema baselines | HGM-008, HGM-016 | J008-016, J010-016, J033-017, J035-022, J075-008, J079-010, J087-011, J096-017, and J107-013 all require edit-time lock rereads or prefer-reuse decisions. |
| Perception/document boundary | J008-003, J008-018 | J033-002, J033-008, and J033-021 keep PDF parsing in document_vision_rag and decline a screenshot-OCR side path. |
| Evaluation ownership | J008-013, J008-029 | J033-010/J033-011 supply the canonical parsed-document baseline but leave dots.ocr and Qwen3-VL adoption to J-8. |
| Insight-to-case memory | J010-011, J010-015 | J096-002, J096-005, J096-013, and J096-029 reuse the provenance-bound insight field and forbid re-extraction or a parallel store. |
| Human decision boundary | J035-003, J035-009, J035-013 | J079-001 through J079-004 preserve consent by offering grill and rerunning deterministic gates, never using grill as verdict. |
| Observe-before-block | J035-016/J035-024, J079-008/J079-009, J087-012/J087-016 | All three begin nonblocking or conservative and promote only after measured false-positive/over-block evidence. |
| No-removal discipline | J075-004, J075-006, J075-016 | Graph rank-to-navigation or any stage prune is a later user-authorized change, not an ablation output. |
| Journal trust stack | J087-001 through J087-006 | J096 proof-born cases and J107 receipt curves are only as trustworthy as complete, independently reconciled event evidence. |
| Safe self-improvement split | J010-008, J096-003, J096-009, J096-030 | Text may be model-produced, but trust, birth, edges, recurrence, and later reuse promotion remain deterministic/off-agent. |
| Measurement-gated recursive reasoning | J107-006 through J107-008, J107-015, J107-026 | J-108 may become local-default only after measured lift with no J-55 collapse; a flat-only stub cannot authorize it. |

## Explicit versus inferred, ambiguous, and conflicting

### Explicit

All canonical feature atoms tagged [E] are direct requirements, statuses, constraints, schemas, failure behaviors, gates, surfaces, or evidence statements in the assigned source. External metrics and repository facts remain source assertions/design rationale; this discovery did not independently web-verify them.

### Bounded inferences

- [I] **INF-001** From HGM-008/HGM-016 plus the later and mutually different “live” baselines, numeric SSB baselines in these plans are snapshots, not implementation constants; the only stable contract is edit-time discovery plus lockstep update.
- [I] **INF-002** From J033-008/J033-011 and J008-013, MinerU parsing is a baseline/pre-stage integration and not evidence that either VLM candidate should be adopted.
- [I] **INF-003** From J035-003/J035-013 and J079-003/J079-012, cards and grill artifacts are decision inputs; neither is an authoritative aim or gate verdict.
- [I] **INF-004** From J087-005/J087-023, “complete” is parameterized by the frozen covered-effect schema. Changing that schema changes the claim and therefore requires review/evidence, not a silent config edit.
- [I] **INF-005** From J096-009/J096-030 and J107-015/J107-026, observe-only utility or lift measurement can justify a later gate but cannot itself enable prompt reuse or the recursive runtime default.

### Ambiguities and unresolved choices

- [A] **AMB-001** J008-016/J008-031: new event versus reuse of J-1's carrier is edit-time conditional.
- [A] **AMB-002** J008-029: exact dots.ocr and Qwen3-VL-Embedding identities remain unresolved and block Tier 2 only.
- [A] **AMB-003** J010-028: “two short calls” conflicts with per-ancestor resynthesis unless it names call types rather than invocation cardinality.
- [A] **AMB-004** J010-031: weak local ideation may selectively use cloud, but no default is chosen.
- [A] **AMB-005** J033-005/J033-017: model license and event reuse are unresolved.
- [A] **AMB-006** J075-008: reuse of measurement SSB versus a discriminator is deferred to edit.
- [A] **AMB-007** J079-016: dark B-40-first sequencing versus C-29 carrying the case before widening needs an explicit implementation order.
- [A] **AMB-008** J087-012: observe-only mode is clear; the enable-flag's unset Boolean behavior is not.
- [A] **AMB-009** J096-002/J096-028: J-10 sequencing and signature threshold remain open, while v1 proof scope is explicitly red-green only.
- [A] **AMB-010** J107-004/J107-012/J107-025: planned/dark runner dependencies, a registered flat-only arm, and deterministic-subset selection all require live resolution before a two-arm claim.

### Conflicts and corrections

- [C] **CON-001** J033-016: “stdlib + requests” is internally false because requests is third-party; the harness dependency/fallback contract must be corrected before coding.
- [C] **CON-002** J035-022: the numbered original explicitly supersedes stale 241/550 SSB counts with a then-current 268/577 snapshot and still requires rereading; neither figure is safe to hard-code.
- [C] **CON-003** J035-008 versus J035-024 is not an unconditional runtime contradiction: active mode blocks, but initial Tier 1 observe-only only notices. Any summary saying “premise contradiction halts decomposition” without this phase condition is overbroad.
- [C] **CON-004** J033-004/J033-005 correct the shorthand “Apache-era MinerU”: the source says a custom Apache-2.0-plus-four-clauses code license with thresholds/attribution, while the VLM model license is separately unverified.

## Summary additions and corrections

Comparison baseline: only the relevant blocks in 10-J-SUMMARY_INDEX.discovery.md (J-8 at lines 106-117, J-10 at 129-138, J-33 at 370-379, J-35 at 393-404, J-75 at 797-804, J-79 at 837-845, J-87 at 916-926, J-96 at 1053-1062, and J-107 at 1129-1138).

| Delta ID | Entry | Addition or correction supplied by the deep source |
|---|---|---|
| **DELTA-001** | grounding | The summary has no first-class ledger for HGM-001..HGM-022: exact hook/log seams, tool/skill census, deliberate F:-bug non-fix, moving schema history, dependency substitutions, skill eval convention, and J-1..J-7 SSB scope are additions. |
| **DELTA-002** | J-8 | Adds exact resize rows, coordinate order, >10 counting discriminator, one-reprompt policy, route Kinds/residue, metric artifacts, default-on runtime rules, deterministic checks, corrupt-TOML DegradedPassthrough, shared-file coordination, and all edit-time uncertainties (J008-008..J008-032). |
| **DELTA-003** | J-8 | Corrects any impression that candidate eval means adoption: anti-slopsquat identity and hardware gate Tier 2, and adoption is separately user-gated (J008-013/J008-029). |
| **DELTA-004** | J-10 | Adds head+tail truncation, ~200-word/depth<=4 bounds, cache-breakpoint placement, exact event Kinds, local-sidecar acceptance, malformed-output handling, no-I-4 partial shipment, and the call-count ambiguity (J010-005..J010-031). |
| **DELTA-005** | J-33 | Adds exact HTTP/multipart/output contract, supported formats, license thresholds/attribution, VLM-license block, config fields, 5-10-PDF metric design, Cosmos inheritance, harness evidence, FastAPI-not-MCP boundary, and request-library conflict (J033-003..J033-021). |
| **DELTA-006** | J-33 | Corrects “Apache-2.0-era” to the source's custom Apache-plus-four-clauses statement and separates code-license acceptability from unresolved model-card license (CON-004). |
| **DELTA-007** | J-35 | Adds the complete route input tuple, exact typed fields/event payload, no-signal autonomous fallback, rules thresholds/lifecycle, UI fallback, Tier-2 composition, stale-lock correction, and explicit riders (J035-005..J035-027). |
| **DELTA-008** | J-35 | Corrects unconditional blocking: first release is default-off OBSERVE-ONLY and emits a nonblocking notice for at least three real runs; blocking follows bounded FP/over-block evidence (CON-003). |
| **DELTA-009** | J-75 | Adds exact full-stack subject, bench-only allowlisted variants, zero-prune tier, real CLI reachability, persistent report/event uncertainty, corpus-representativeness caveat, and full-stack default assertion (J075-005..J075-016). |
| **DELTA-010** | J-79 | Adds skill-text deliverable, exact one-line/wait behavior, deterministic gate rescore, autonomous escalation constraint, no-verdict rule, harness-only evidence, and the B-40-dark/C-29 sequencing ambiguity (J079-003..J079-017). |
| **DELTA-011** | J-87 | Adds exact covered syscalls, atomic release order, normalization examples/window, startup self-test, module/symbol names, fault arms, frozen user-reviewed covered-set data, platform/host ceilings, and enable-flag ambiguity (J087-002..J087-024). |
| **DELTA-012** | J-96 | Corrects node shape to six top-level fields with taxonomy nested in trigger_signature; adds idle edge timing, stats CLI, STALE versus UNTRUSTED behaviors, exact event Kinds, proof-scope decision, live-path tests, and J-10 sequencing uncertainty (J096-002..J096-030). |
| **DELTA-013** | J-107 | Adds exact named scaffold operations, 0.5W/2W/8W bands, tokens/correct, collapse comparison, real CLI syntax, >=5-seed rejection, deterministic subset rule, licensed adaptation/no vendoring, RunPod notice, registered flat-only behavior, and prohibition on lift claims before J-108 (J107-006..J107-028). |

## Zero-omission and zero-duplicate audit

- Manifest membership: 10 expected paths; 10 processed; 0 missing; 0 extra original sources.
- Integrity: 10/10 hashes exact; 10/10 line counts exact; 10/10 byte counts exact; aggregate exact at 1,229 lines and 134,050 bytes.
- Full-read boundary: every physical line from each assigned file's line 1 through its explicit end line was displayed with line numbers. Blank lines, headings, citations, file lists, shell checks, acceptance boxes, risks, and end markers were included in the read; substantive requirements from those sections were normalized into the ledgers.
- Atomic-definition audit: **242 canonical per-source atoms** (22 HGM + 33 J-8 + 32 J-10 + 21 J-33 + 27 J-35 + 16 J-75 + 17 J-79 + 24 J-87 + 30 J-96 + 28 J-107), plus 5 bounded inferences, 10 ambiguity records, 4 conflict/correction records, and 13 summary deltas = **274 uniquely defined IDs**.
- Duplicate policy: each canonical source fact is defined once. Repeated task/contract/acceptance/test wording is merged into its canonical atom; later sections reference IDs only. ID uniqueness and prefix totals were mechanically checked after writing.
- Classification audit: explicit, inferred, ambiguous, and conflicting material is separated above; conditional status is not promoted to shipped behavior, external figures are not treated as ELAI results, and unresolved choices remain unresolved.
- Orientation audit: the summary supplied comparison structure only. No summary statement was used to override a numbered source or the grounding map.
- Output SHA-256 convention: the value below is computed over the final file after replacing the 64 hexadecimal characters in the output_sha256 field with 64 zeroes. This avoids an impossible self-referential whole-file digest while keeping the artifact independently verifiable.
- output_sha256: 0000000000000000000000000000000000000000000000000000000000000000
