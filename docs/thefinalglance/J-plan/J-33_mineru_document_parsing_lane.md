# J-33: mineru document-parsing lane — wire the shipped-dark PDF parser backend into real ingestion (runtime + harness)

STATUS: PLANNED (WAVE DP-1)

- Track J | lever CANT-MISREAD+REACH | grounding STRONG (ELAI side code-verified 2026-06-13) / EXTERNAL (MinerU quality figures — self-reported, design-rationale only) | nature ACTIVATION + BUILD (wiring, not invention) | effort M (Tier-1) + gated Tier-2 | WAVE DP-1 | new external service dep = USER CONFIRMATION gate (house rule) + model-card license verify-at-edit

## Premise

The user asked "can MinerU improve the visual loop / add PDF support?" Verified answer: it is NOT a
visual-ui-loop unit (that rule is UI-screenshot-only — zero pdf/document hits, grep 2026-06-13);
it IS the missing wire of ELAI's DOCUMENT lane. Today every PDF entry point is text-blind:
- B-35 artifact indexer: index_single_file calls std::fs::read_to_string — no .pdf branch
  (crates/knowledge/src/artifacts/indexer.rs:236); a PDF artifact is garbled or skipped.
- harness local-rag ingest: _ipc_ingest_file does path.read_text (rag server.py:1555) — the
  paper-ingestor works off arXiv HTML, raw PDFs unsupported.
- E-36 document_vision_rag: SHIPPED-DARK behind ELAI_DOCUMENT_VISION_RAG_ENABLE (mod.rs:31), and —
  the accelerant — crates/knowledge/src/chunking/pdf_structured.rs ALREADY ships a ParserBackend
  enum with a MinerU variant (+ PaddleOcrVl / Dolphin / LlamaParseCloud / MistralCloud) plus
  check_backend_eligibility / compute_fallback_recommended, with ZERO production callers.
So J-33 is the classic ELAI shape: the type layer exists, the gap is the wire. MinerU
(opendatalab, ~67K stars, v3.3.1 2026-06-11, weekly releases) parses PDF/DOCX/PPTX/images ->
structured markdown + content_list.json (layout, 109-language OCR, formula->TeX, table->HTML,
reading order), with a CPU-capable pipeline backend and a GPU VLM backend (MinerU2.5 1.2B, min
8GB VRAM), and ships a FastAPI server (`mineru serve`, POST /file_parse) — the cleanest boundary
for a Rust host.

LICENSE (the make-or-break, verified 2026-06-13): the code is NO LONGER AGPL — v3.x ships the
"MinerU Open Source License" = Apache-2.0 + four clauses (free commercial use under 100M MAU /
$20M monthly revenue; attribution required in UI or public docs for service use) after the team
removed the AGPL model deps. Safe as an EXTERNAL opt-in service for closed-source ELAI; NEVER
vendor the code; do NOT use the Flash-MinerU fork (still AGPL). OPEN GATE: the VLM model card
(opendatalab/MinerU2.5-2509-1.2B on HF) license is UNVERIFIED — verify before any Tier-2 VLM leg;
the Tier-1 pipeline backend does not need it. Attribution clause = one line in the docs/about
surface, recorded as a deliverable.

## Steal/build (5 legs)

1. PARSER SERVICE CLIENT: a thin DocParserClient in crates/knowledge (reqwest against
   `mineru serve` POST /file_parse, multipart PDF -> markdown + content_list.json), behind the
   SHIPPED ParserBackend::MinerU variant — the enum finally gets its production arm.
   check_backend_eligibility decides per-host (service reachable? backend mode?); unreachable =
   deterministic fallback per compute_fallback_recommended, NEVER a silent skip (R1: a PDF that
   cannot be parsed is reported unparsed, not indexed as garbage).
2. B-35 ARTIFACT WIRE: a .pdf/.docx branch in index_single_file routing through
   pdf_structured + the parser client, then the EXISTING chunkers (B-32 cap discipline) and the
   EXISTING ArtifactStage — Cosmos artifacts (B-42) inherit PDF support for free since
   cosmos_worker already references the pdf_structured types.
3. E-36 PRE-STAGE: parsed markdown/structure becomes the deterministic FIRST leg of the document
   route — VisualArtifactCheck claims that resolve against parsed text/tables/formulas never pay
   a VLM call; the (dark) ColQwen/VLM leg stays the escalation for what parsing cannot answer
   (same doctrine as the visual loop: deterministic before vision). Composes the E-36 flag; this
   unit does NOT flip ELAI_DOCUMENT_VISION_RAG_ENABLE wholesale — it gives the parse leg its own
   wire and leaves the VLM leg's promotion to E-36's own done-gate.
4. HARNESS LEG: .claude/tools/pdf_ingest.py — a thin wrapper the paper-ingestor agent MAY call
   (gated: service reachable, else degrade to today's HTML path) converting a paper PDF ->
   markdown BEFORE mcp_ingest_file, so formula/table-heavy papers stop ingesting as noise.
   JD-1 evidence = the tool's log + the ingested chunk count delta.
5. EVAL, NOT FAITH: MinerU's 90.67 OmniDocBench score is SELF-reported on a SELF-created
   benchmark (arXiv:2509.22186; the org authored both) — [EXTERNAL], never claimed. The done-gate
   ships a small ELAI-measured fixture eval (5-10 PDFs incl. a formula paper + a table-heavy
   spec): parsed-vs-read_to_string retrieval delta on the B-35 query path, recorded in a metrics
   TOML (ID-5). dots.ocr / Qwen3-VL comparisons stay J-8's eval legs (VR-8/VR-9) — J-33 does not
   pre-empt them; it gives them the baseline lane to compare against.

## Existing ground truth (code-verified 2026-06-13; re-grep at edit, W-7)

- ParserBackend{MinerU,...} + eligibility/fallback machinery SHIPPED, zero production callers:
  crates/knowledge/src/chunking/pdf_structured.rs. document_vision_rag/ SHIPPED-DARK (mod.rs:31);
  VerificationMethod::VisualArtifactCheck at crates/reasoning/src/types.rs:421 (no prod caller);
  contract test vision_artifact_check_dispatch_contract.rs exists.
- B-35 ArtifactStage WIRED (stage.rs:48) but index_single_file is text-only (indexer.rs:236).
  B-42 cosmos_worker SHIPPED. Harness rag ingest text-only (server.py:1555).
- J-1/J-8 PLANNED-ONLY (no perception/ dir) — J-33 lands in the document_vision_rag sibling, NOT
  the perception module, and carries NO J-1/J-8 dependency (their doc-route composes later).
- GD-1: only plan mention of MinerU is E-36 naming the enum variant; no unit ships the ingestion
  wire — genuinely unplanned residue.
- MinerU external: license + maturity + API shapes per the 2026-06-13 web grounding (sources in
  workflow/handoff/j-canvas-frontdoor-grounding.md): v3.3.1, FastAPI server, pipeline=CPU-ok,
  VLM=8GB VRAM min, effort parameter 35-220% speed tradeoff (no absolute wall-clock published —
  measure ours).

## REVISED scope (W-10 tiered)

Tier-1 (ships): pipeline-backend-only via the HTTP service (CPU-capable — runnable anywhere,
no VRAM claim), legs 1+2+4 + the fixture eval. The service is an OPT-IN external dep: user
confirmation + anti-slopsquat (pip/registry verify) at edit; unconfigured = today's behavior,
loudly reported (the dead path screams, R2).
Tier-2 (gated): VLM backend on the remote 3090 (HARDWARE-HEAVY rule: flag in chat before boot;
never beside the sidecar model without VRAM math) + the model-card license verify + leg 3's
deeper E-36 composition + async /tasks batching for Cosmos refresh (rides H-5 batch doctrine).

## Integration

- crates/knowledge: DocParserClient (~150 LOC) + the .pdf branch in artifacts/indexer.rs +
  pdf_structured production arm. crates/orchestrator: cosmos_worker inherits via the same path.
- config: [knowledge.doc_parser] block (service_url, backend=pipeline|vlm, enabled=false default
  BUT registered in wire-dark-allowlist.toml with promotion plan = this file's Tier-1 done-gate
  (R9/W-2: default-off is a gate, not a graveyard)).
- harness: .claude/tools/pdf_ingest.py (stdlib + requests; degrade-to-passthrough per JC-4).
- SSB: prefer-reuse the shipped E-36/pdf_structured rows if any exist (verify at edit); else
  +DocumentParsedEvent (Pattern-2 single-variant; Kind: Parsed / ParseFailed / FallbackUsed /
  EvalMeasured). net 0..+1 from LIVE 241/550 (re-grep at edit, W-9).
- frontend_pair: parsed-artifact rows/badges in KnowledgeDrawer (artifact list gains a
  parsed-by-MinerU badge + parse-failure surface); W-4 at edit.

## observable_effect (done-gate, W-1)

With the service configured, `elai` indexing a fixture dir containing a real formula-heavy PDF
lands artifact chunks whose text contains the TeX of a known formula (grep-provable against the
journal/index, impossible via read_to_string) + a Parsed row; a /research query against the PDF's
content retrieves it (the B-35 query path, scored in the fixture eval TOML). The harness leg:
paper-ingestor on a raw PDF produces a markdown ingest whose chunk count and formula fidelity
beat the HTML path on the fixture (measured, ID-5). NEGATIVE (R2): service down -> ParseFailed +
deterministic fallback recommendation, the PDF is NOT silently indexed as binary garbage; the
Flash-MinerU fork or any vendored MinerU code in the tree FAILS review (license boundary); an
unconfigured install behaves exactly as today AND says so in the drawer (no phantom capability).

## Riders (recorded, not in-scope)

- dots.ocr / Qwen3-VL eval = J-8 VR-8/VR-9 (this unit supplies their comparison baseline).
- DOCX/PPTX office formats ride the same wire (MinerU supports them) — fixture eval is PDF-first.
- An OCR'd-screenshot leg for the visual loop = DECLINED here (the visual loop's perception
  scaffold is the AX-tree, not OCR; if ever wanted, it is a J-1 gateway leg, not J-33).
- MinerU MCP server mode = noted; ELAI consumes the FastAPI shape, not MCP, to keep the G-4/E-24
  pin surface out of a Tier-1 ingestion path.
