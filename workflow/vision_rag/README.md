workflow/vision_rag/

E-36 Tier-A scaffolding directory. Holds non-code artefacts for the document
vision RAG path (PDF / screenshot / diagram ingestion + VLM extraction +
ColPali-style late-interaction retrieval).

This directory was created empty by the s-orch-e36 run. Tier-A shipped the
type foundation (knowledge::chunking::image_artifact) + a contract test
(crates/contracts/tests/vision_chunk_types_contract.rs); Tier-B will populate
this directory with:

  - sidecar/        — PaddleOCR-VL + ColModernVBERT + Dolphin-v2 deployment notes
  - fixtures/       — sample PDFs / screenshots for the test suite
  - eval/           — vision-rag eval harness output
  - decisions/      — per-task decision logs

See workflow/reasoning/E-36-decisions.md for the full Tier-A / Tier-B split
and the 5 HIGH drift findings from the s-orch-e36 batch.
