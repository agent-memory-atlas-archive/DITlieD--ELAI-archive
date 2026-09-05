E-36 — Document vision RAG — decisions
session: s-orch-e36 (campaign continuation)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-36_document_vision_rag.md

(a) SCOPE DETERMINATION — TIER-A ONLY (6-of-6 campaign drift confirmed)

E-36 returned PLAN_DRIFTED_REQUIRES_IMPROVEMENT from both dispatched agents. Drift findings:

  DRIFT-1 (HIGH): plan title says "8th GroundingMethod" — symbol GroundingMethod (orchestrator/ipc_gateway.rs:57) has only 3 variants. The 8-variant enum the plan actually means is VerificationMethod at reasoning/types.rs:331 (FileCheck/SourceCheck/TestCheck/WebCheck/ApiCheck/ApiRegistryCheck/ArtifactCheck/CosmosCheck). The new variant would be the 9th VerificationMethod, not the 8th GroundingMethod. reasoning/ is AVOIDED this run.

  DRIFT-2 (HIGH): plan cites verification.rs:452 — real location is reasoning/types.rs:331. Line numbers stale.

  DRIFT-3 (HIGH): plan task 4 + DETERMINISTIC_CHECK #3 reference an ArtifactType enum inside cosmos_worker.rs — symbol does not exist.

  DRIFT-4 (HIGH): plan claims cosmos_worker.rs is ~700 LOC — real value is 1459 LOC.

  DRIFT-5 (HIGH): plan claims "C-7 PLANNED Cosmos Worker Wiring" — C-7 was the ChangeDecision/ReviewWindow ship (SHIPPED 2026-05-17). CosmosRefreshStarted / CosmosArtifactUpdated / CosmosArtifactDecayed / cosmos_grounding_used are LIVE in ssb.rs:2126-2134 (from B-42 SCHEMA bump 43 -> 44).

  DRIFT-6 (HIGH): 4 HuggingFace / arXiv handles cited without verification (ColModernVBERT, ColQwen2.5-v0.2, PaddleOCR-VL 0.9B, Dolphin-v2). arXiv:2602.12510 implausible year — needs WebSearch verification (anti-slopsquatting).

  DRIFT-7 (MEDIUM): VRAM ~1.8 GB free per memory:project_gpu_context, plan requires ~3.8 GB for simultaneous VLM + reranker — sidecar capacity insufficient on local hardware.

  DRIFT-8 (MEDIUM): plan task 8 wants `crates/dashboard-v2/src/lib/components/VisionRAGDrawer.svelte` — drawer path is src/lib/drawers/ not src/lib/components/. Same path-drift pattern as E-33 D5 (MediaValidationDrawer didn't exist) and E-29 (TranscriptDrawer location).

  DRIFT-9 (MEDIUM): tasks 3, 5, 8, 10 require AVOIDED-crate edits (orchestrator/{root} for SSB schema bump, reasoning for VerificationMethod expansion, providers/runtime for VLM dispatch, python/elai_inference for PaddleOCR sidecar).

  DRIFT-10 (MEDIUM): plan's claim_class: visual_spatial routing predicate has zero producers in the codebase today.

6-of-6 plans now confirmed PLAN_DRIFTED_REQUIRES_IMPROVEMENT (E-44, E-16, E-31, E-29, E-21, E-36).

(b) WHAT TIER-A E-36 SHIPPED

Three deliverables, all in OWNED crates. Zero AVOID-zone touches, zero new SsbActionType variants, zero new dependencies.

  - crates/knowledge/src/chunking/image_artifact.rs (NEW, ~115 LOC)
    Defines VisionArtifactKind enum (Page | Screenshot | Diagram) + VisionArtifact struct (source_path, page_index, kind, content_hash_hex, byte_size) + hash_bytes helper that uses blake3 per ELAI anti-stale rule. Locked at 3 kinds for Tier-A. Includes 3 unit tests covering: variant-str round-trip, blake3 hex format pin, full artifact serde round-trip.

  - crates/knowledge/src/chunking/mod.rs (MODIFIED, +3 LOC)
    Adds `pub mod image_artifact;` declaration alongside the existing constants / detect / cap / character modules. Comment block points future implementers at the decisions doc.

  - crates/contracts/tests/vision_chunk_types_contract.rs (NEW, ~80 LOC)
    Pins the Tier-A surface — 5 tests: 3-variant lock, snake_case wire-form, serde round-trip, page_index = None contract for standalone Screenshot, blake3-hex format (64 char lower-case + empty-input digest pin so an accidental blake2 / sha2 swap fails the contract).

  - workflow/vision_rag/README.md (NEW)
    Creates the OWNED workflow/vision_rag/ directory the plan referenced. Lists the four subdirectories Tier-B will populate (sidecar/, fixtures/, eval/, decisions/).

  - workflow/reasoning/E-36-decisions.md (NEW, this file)

(c) WHAT TIER-B DEFERRED + WHY

Per the 10 DRIFT findings + the 6-of-6 campaign pattern:

  Task 3 (5 new SsbActionType variants for vision_artifact_indexed / chunked / extracted / verified / failed) — defer reasons: (a) 7+ atomic touchpoints × 5 variants in ssb.rs grey-zone; (b) 288-variant action_type_lock.golden alphabetical regen via `cargo test regen_golden_file --ignored`; (c) co-staging variants without producers (sidecar VLM is AVOIDED) violates the B-23 4I.1 dead-schema-rows precedent.

  Task 4 (VerificationMethod::VisionCheck variant expansion in reasoning crate) — defer reasons: AVOIDED crate (reasoning/). Belongs in a session that owns reasoning + can coordinate with PerLevelAuditVerdict + the auditor mode dispatch surface (B-41 SHIPPED).

  Task 5 (PaddleOCR-VL sidecar wiring in python/elai_inference/server.py) — defer reasons: AVOIDED crate; HARDWARE-HEAVY per CLAUDE.md sidecar rule; VRAM headroom insufficient on local pod (memory:project_gpu_context says ~1.8 GB free, plan needs ~3.8 GB).

  Task 6 (ColModernVBERT / ColPali late-interaction retrieval) — defer reasons: (a) needs the SsbActionType variants from Task 3; (b) needs the producer trait surface from Task 4; (c) anti-slopsquatting WebSearch verification of the cited HuggingFace handles.

  Task 7 (RRF fusion-4 extending knowledge::pipeline::rrf_rerank) — defer reasons: knowledge crate is OWNED, but this builds on Task 4's VerificationMethod expansion + the SSB variants from Task 3. Standalone shipping would create dead code paths.

  Task 8 (VisionRAGDrawer.svelte) — defer reasons: needs the get_vision_artifact_status Tauri command + the SsbViewer surface from Task 3; without those the drawer renders only the empty state. Pattern matches E-21.A PagerankPanel + E-29.A drawers — graceful-empty over real producer is the right shape, but the producer must exist first.

  Task 10 (visual_spatial claim_class routing predicate) — defer reasons: AVOIDED reasoning crate; needs the VerificationMethod expansion from Task 4.

(d) WHY VisionArtifact + BLAKE3 + 3-VARIANT LOCK

Even though the full RAG pipeline is Tier-B, the artifact types lock the wire shape NOW so future producers + consumers (sidecar / dashboard / contract tests) cannot diverge. blake3 is the workspace standard per CLAUDE.md anti-stale rule (SHA-256 / blake2 forbidden for non-RFC uses). The 3-kind variant set (Page / Screenshot / Diagram) is the minimum-viable taxonomy:

  - Page — multi-page document (PDF, slide deck) where a single artifact maps to one page.
  - Screenshot — standalone capture, no parent document.
  - Diagram — embedded figure inside a larger document (chart, flowchart, architecture diagram), routed differently than text-bearing pages.

Tier-B may add Photograph / WhiteboardCapture / RawScan if needed; growth is gated on the contract test which will fail loudly.

(e) WHY THE EMPTY-INPUT BLAKE3 HEX IS PINNED

The contract test has a literal assertion: `VisionArtifact::hash_bytes(b"") == "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"`. This is the canonical blake3 empty-input digest. If a future refactor "improves" the helper to use blake2 / sha2 / md5, this assertion fails and the anti-stale rule is enforced at the contract layer — no silent algorithm drift.

(f) NO SSB / NO REASONING EDIT — DELIBERATE

E-36 plan would add ~50 SSB-related touchpoints (5 action_types × 7-8 lockstep sites + the 288-variant golden regen) AND would touch the reasoning crate (AVOIDED). This run touches NEITHER. Tier-B will land them atomically with the producers.

The campaign-wide discipline emerging across 6 plans:
  - Tier-A = shipping over EXISTING producers without resolving every drift.
  - Tier-B = adding new producers + their SSB / SCHEMA / contract surface together.
  - Plan-improvement protocol mandatory before Tier-B EXECUTING.

(g) TIER-B ADDENDUM (completion-drive session 4/4 closure)

Following the 6-of-6 PLAN_DRIFTED campaign + the Tier-A 3-variant lock, this Tier-B addendum lands the document+vision RAG producer substrate while keeping the dashboard surface excluded per the user's "NO dashboard" directive for this run.

  TIER-B SHIPPED — Producers, types, contracts (Tier-A complement, no dashboard touch):

  - crates/knowledge/src/chunking/image_artifact.rs (EXTENDED, ~310 LOC total)
    Grew VisionArtifact with 4 Option<>-wrapped fields (multi_vec / pooled / vlm_caption / parser_confidence) — back-compat preserved (Tier-A serde round-trip stays green via #[serde(default, skip_serializing_if)]). Added ImageArtifactChunker with chunk_image + pool_mean + attach_pooled_vectors. Mean-pool is the conservative fallback for the Visual RAG Toolkit (arXiv:2602.12510 — VERIFIED real, Feb 2026) pooling story until the encoder lands; pool_mean reduces 1030->100 with deterministic windowing.

  - crates/knowledge/src/chunking/pdf_structured.rs (NEW, ~265 LOC)
    PdfStructuredChunker types-only API + ParserBackend enum (6 arms: PaddleOcrVl / MinerU / Dolphin local; MistralCloud / ReductoCloud / LlamaParseCloud cloud). compute_fallback_recommended + check_backend_eligibility are pure-compute; the cross-language sidecar parse step lives in orchestrator::document_vision_rag (NOT in python/elai_inference this run — that remains AVOIDED per VRAM gating + cross-language drift policy).

  - crates/knowledge/src/cosmos_worker.rs (EXTENDED, ~+185 LOC)
    Introduced ArtifactType (4 arms: Diagram / ArchImage / Screenshot / ScannedDoc) + DiagramSourceLang (Mermaid/PlantUml/D2/Excalidraw) + ParserBackend (6 arms; same shape as the chunking::pdf_structured peer for taxonomy consistency — kept separate to avoid pulling chunking into the cosmos module path). is_stale() helper enforces the per-type stale-check policy: Diagram fires on source_text change OR rendered_png_blake3 change; ArchImage/Screenshot/ScannedDoc fire on blake3 mismatch of the source file.

  - crates/reasoning/src/types.rs (EXTENDED, ~+80 LOC)
    Added VerificationMethod::VisualArtifactCheck (the 10th variant, not 8th per plan drift; the Tier-A header comment said 9th but pre-counted EmpireEventCheck which IS live — actual count is 10). Reuses `claim_category: String` rather than introducing a parallel `claim_class` taxonomy (GAP-7 from context-compiler). Added requires_vision_index() helper. Added VisualArtifactVerdict (Confirm/Refute/Indeterminate) + ArtifactReliability (VlmCaption/OcrParse/DiagramSource) enums with is_low() helper. Updated BOTH default arrays (AgentGrcConfig + GrcDefaults) so dispatch is reachable.

  - crates/orchestrator/src/document_vision_rag/ (NEW subtree, ~525 LOC across 3 files)
    mod.rs — ELAI_DOCUMENT_VISION_RAG_ENABLE_ENV gate + emit_enabled/emit_disabled helpers.
    policy.rs — VisionRagPolicy (default disabled, 500MB budget, Local mode) + CloudMode (Local/Hybrid/Cloud) + check_cloud_parser_blocked + decide_policy. Pure-compute; producer call path wires the SSB row.
    visual_verifier.rs — verify_pure_compute kernel implementing the HARD downweight rule: VlmCaption-tier verifiers NEVER issue Confirm verdicts; OcrParse requires top score >= 0.8; DiagramSource confirms with any candidate.

  - crates/orchestrator/src/ssb.rs (EXTENDED)
    SCHEMA_VERSION 181 -> 182 atomic bump.
    SsbActionType::DocumentVisionRagEvent variant added (single-variant Pattern-2, 34th application — collapses the plan's "5 SSB variants" requirement into one variant carrying a 5-arm DocumentVisionRagEventKind discriminator: VisionChunkEmitted / VisionFallbackTriggered / VisionRagPolicyDecision / VisualArtifactCheckVerdict / DocParserCloudFallbackBlockedByMode).
    DocumentVisionRagEventPayload { kind, session_id, task_id, details_json, ts_ms } + ELAI_DOCUMENT_VISION_RAG_ENABLE_ENV const.
    All 7 lockstep sites updated: SQL CHECK constraint (also CLOSED THE E-21 OVERSIGHT — peer_systems_event was missing from the CHECK clause), enum variant, as_sql_str arm, from_str arm, SsbPayload variant, SsbPayload::action_type() arm, DocumentVisionRagEventKind enum.

  - crates/orchestrator/tests/action_type_lock.rs + .golden (UPDATED)
    TOTAL_VARIANTS 454 -> 455. ALL_VARIANTS array + exhaustive_match_guard match block both gain DocumentVisionRagEvent. Golden file alphabetically re-numbered: document_vision_rag_event lands at ordinal 0136 (between dkg_rebuild_started 0135 and durability_lock_blocked 0137). All 3 action_type_lock tests GREEN.

  - crates/knowledge/src/vector/mod.rs (EXTENDED)
    Added public rrf_score_4_weighted helper + RrfWeights4 struct (default 1.0/1.0/0.8/0.6 per plan task 6) + RRF_K=60 const. The production RRF accumulator in pipeline.rs already loops over Vec<PipelineStage> so adding a "vision_vec" stage to that vector composes naturally; this helper ships the per-stage formula for the rrf_4list_fusion_contract test.

  - crates/contracts/tests/ (5 NEW + 1 EXTENDED)
    vision_chunk_types_contract.rs — Tier-A test extended for new VisionArtifact fields. 5 GREEN.
    vision_artifact_check_dispatch_contract.rs — 5 GREEN. Locks the 10th VerificationMethod variant + requires_vision_index + dual default-array updates + claim_category-not-claim_class field name.
    vision_chunk_pooling_contract.rs — 8 GREEN. Locks MAX_VECTORS_PER_PAGE <= 128 + 1030->100 reduction + determinism across 3 runs.
    paddleocr_vl_parser_confidence_contract.rs — 8 GREEN. Locks DEFAULT_CLOUD_FALLBACK_THRESHOLD == 0.7 + Hybrid-triggers-Local-blocks fallback semantics + cloud-backend eligibility.
    visual_artifact_reliability_tier_contract.rs — 10 GREEN. Locks 3-tier taxonomy + HARD downweight rule (VlmCaption-tier verifiers never confirm).
    rrf_4list_fusion_contract.rs — 9 GREEN. Locks 4-list weighted RRF + vision_vec <= text_vec invariant + determinism across 3 runs.

  - crates/contracts/tests/routing_e20_contract.rs (FIXED)
    schema_version_at_180 was an exact-equality pin (`assert_eq!(SCHEMA_VERSION_PUBLIC, 180)`) broken since E-21 bumped to 181. E-36 bumps to 182 — same kind of fail. Rewrote as a FLOOR (`>= 180`) per the precedent set by adversary_contract.rs / adv_rm_farming_contract.rs so the E-20 contract surface stays orthogonal to subsequent SCHEMA bumps.

  Net test deltas (this session):
    - 5 new contract test files, 40 new tests GREEN.
    - 1 contract test file extended (vision_chunk_types_contract.rs) — Tier-A 5 still GREEN.
    - 1 contract test file fixed (routing_e20_contract.rs) — 8/8 GREEN (was 7/8 broken).
    - 15 new orchestrator subtree unit tests GREEN.
    - 13 new knowledge::chunking unit tests GREEN.
    - 0 regression: cosmos_worker 16/16, ssb 56/56, reasoning 40/40, action_type_lock 3/3 (including golden match) all GREEN.

  TIER-B DEFERRED — remaining out-of-scope items per user "NO dashboard" + the campaign discipline:

  - Task 1 (--vision flag on `elai doctor knowledge-models`) — CLI surface; could be added in a follow-on without touching this Tier-B substrate.
  - Task 5 PaddleOCR-VL sidecar wiring in python/elai_inference/server.py — HARDWARE-HEAVY (VRAM gap: ~1.8 GB free, ~3.8 GB needed); cross-language pair MUST mirror SCHEMA bump 181 -> 182 in python/elai_inference/schemas/messages.py (the python side has been at 138 for many sessions; that drift is tracked separately by the cross_lang_drift detector and is not made worse by this run).
  - Task 10 dashboard surface (VisionRAGDrawer.svelte + Tauri commands get_vision_artifact_status / get_vision_artifact_check_recent) — EXCLUDED THIS RUN per user "NO dashboard" directive.
  - Task 9 eval/vision_rag/queries.toml + run.py — eval-harness substrate; deferred to its own session with the goldset construction.

(h) HARD RULES enforced by this Tier-B landing:

  - blake3-only hashing (CLAUDE.md anti-stale): every new hash site uses `blake3::hash(bytes).to_hex()` directly. vision_chunk_types_contract.rs pins the empty-input digest.
  - No .unwrap() in production vision pipelines: all chunker / verifier code uses Result/Option propagation. .unwrap() in tests is acceptable per CLAUDE.md.
  - HARD opt-in default: VisionRagPolicy::default() is `enabled = false`, mode = `Local`. Operators must explicitly flip per-project after the eval gate (Task 9) passes.
  - HARD downweight rule: ArtifactReliability::VlmCaption is_low()==true. verify_pure_compute NEVER returns Confirm for VlmCaption-tier candidates. visual_artifact_reliability_tier_contract.rs pins this invariant.
  - No cloud parser when --mode local: check_backend_eligibility + check_cloud_parser_blocked enforce. paddleocr_vl_parser_confidence_contract.rs + the policy tests pin this.
  - SCHEMA atomic +1 not +5: Pattern-2 single-variant collapse keeps the SCHEMA bump at one (181 -> 182) rather than the plan's naive 5 (181 -> 186); matches E-20 / E-21 / E-4 / E-50 precedent. action_type_lock.golden insertion shifts ordinals from 0136 forward by +1 only.

(i) CLOSED OVERSIGHTS

E-21 added SsbActionType::PeerSystemsEvent to the enum + golden + as_sql_str + from_str but missed adding 'peer_systems_event' to the CREATE TABLE CHECK constraint (`action_type IN (...)` allowlist at ssb.rs::CREATE_SSB_TABLE). This caused EVERY peer_systems_event insert to fail with CHECK violation at runtime. E-36 closes the gap during the 181 -> 182 bump by adding BOTH peer_systems_event AND document_vision_rag_event to the CHECK clause in the same edit. The CHECK constraint is regenerated on every SCHEMA bump (run_migration drops + recreates the table when user_version mismatches), so this fix activates the moment SCHEMA 182 ships.

END E-36-decisions.md
