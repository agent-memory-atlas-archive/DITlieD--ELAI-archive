Phase 1B Grounding Log
Session: s-1181060
Date: 2026-04-06
Iteration: 3 (post-audit fix pass)

AUDIT RESULTS (adversarial-auditor)

4 BLOCKs found, all fixed:

BLOCK-1: path field schema TEXT -> STRING
  Finding: tantivy TEXT fields are tokenized (split on / . -). delete_term requires exact match.
  Using TEXT for path causes silent deletion failures.
  Evidence: tantivy deleting_updating_documents.rs example uses STRING for primary-key fields.
  Fix: changed schema to path: STRING | STORED everywhere (1B.2, 1B.4). Added test (j) as regression guard.
  Added R7 risk. Added D6 decision.

BLOCK-2: "tantivy_v21" is fabricated identifier
  Finding: tantivy uses INDEX_FORMAT_VERSION (u32 constant), not string labels.
  "tantivy_v21" in knowledge.txt L200 is a project-level label, not a tantivy API value.
  Fix: updated R1 risk and 1B.1 TASK to explain distinction. Metrics now record actual
  tantivy_version and index_format_version. GROUNDING section notes correction.
  NOTE: knowledge.txt L200 itself should be updated (out of scope for this phase file).

BLOCK-3: Chunk struct undefined between phases
  Finding: Phase 1A exports DkgNode (AST metadata: kind, name, start/end lines) but NOT content.
  Phase 1B referenced "Chunk struct provided by Phase 1A" — no such type exists.
  Fix: defined BM25Chunk as phase-1B-owned type with content field. Added D7 decision explaining
  that DkgNode -> BM25Chunk conversion (reading file content at DKG line ranges) is BM25's job.

BLOCK-4: crate path inconsistency
  Finding: Phase 1A canonical path is .elai_cc/crates/knowledge/ (phase-1A.txt L3).
  Phase 1B used .elai_cc/rust/crates/knowledge/ everywhere.
  Fix: all paths in phase-1B updated to .elai_cc/crates/knowledge/.

5 WARNs found, all addressed:

WARN-1: project_id derivation unspecified
  Fix: changed constructor to new(project_root, project_id) with project_id as explicit parameter.
  Added D8 decision citing knowledge.txt L274.

WARN-2: test (b) references real codebase path contradicting "no hardcoded paths" rule
  Fix: rewrote ALL tests to use synthetic files in tempdir. No real codebase dependency in unit tests.
  Benchmark in 1B.5 still uses real codebase (acceptable — it's a benchmark, not a unit test).

WARN-3: CI/dev threshold inconsistency in DETERMINISTIC_CHECK
  Fix: DETERMINISTIC_CHECK now uses CI env var: CI=true -> 50ms threshold, else -> 10ms.

WARN-4: update_file writer serialization not documented
  Fix: added writer serialization note in 1B.2 update_file docs and 1B.4 INDEX_DIRECTORY.
  Added note to D5 decision.

WARN-5: missing contract tests
  Fix: added contract test creation to step 1B.2 TASK. File at .elai_cc/tests/contracts/bm25_contract.rs.

4 INFOs noted:

INFO-1: ChunkKind enum approach is correct (confirmed).
INFO-2: 10-line overlap is UNGROUNDED for BM25 specifically. Noted in D1 and GROUNDING section.
INFO-3: snippet uses simple truncation, not tantivy SnippetGenerator. Noted in D9.
INFO-4: scope boundary with Phase 1I is clean (confirmed).


SUBSTANTIVE IMPROVEMENTS (from iteration 2, preserved)

1. ZERO ORPHAN INVARIANT (knowledge.txt L177, L254) — tests (e), (f) added
2. INCREMENTAL UPDATE (knowledge.txt L265) — update_file method and test (g) added
3. CHUNKING STRATEGY (knowledge.txt L172-173) — dual-mode with BM25Chunk type
4. INDEX PATH CONVENTION (knowledge.txt L195) — .elai_cc/projects/{id}/bm25/
5. QUERY RETURN TYPE — BM25Hit struct with metadata for downstream integration


CLAIM GROUNDING (full list)

"tantivy full-text index" | VERIFIED | knowledge.txt L36, L103
"Sub-ms queries" | VERIFIED | knowledge.txt L36
"<10ms for indexes <100K documents" | VERIFIED | knowledge.txt L255
"Per-project index" | VERIFIED | knowledge.txt L37
"tantivy_v21 format" | CORRECTED | spec label, not tantivy API. Actual: INDEX_FORMAT_VERSION (u32)
"query fn complete -> returns provider files" | CORRECTED | tests now synthetic, no real paths
"Section 4 (KnowledgeEngine trait)" | VERIFIED | Section 4 "TRAIT BOUNDARIES" at line ~80, trait at ~94. Earlier SPEC_GAP was wrong.
"knowledge/ crate exists" | VERIFIED | phase-1A.txt L21
"Chunk struct from Phase 1A" | CORRECTED | DkgNode has no content. BM25Chunk is BM25-owned
"Crate path" | CORRECTED | .elai_cc/crates/knowledge/ not .elai_cc/rust/crates/knowledge/
"69 .rs files" | VERIFIED | implementation-guide.txt L29
"Zero orphan files" | VERIFIED | knowledge.txt L177, L254
"50-line sliding window" | VERIFIED | knowledge.txt L173
"10-line overlap" | UNGROUNDED | benefits embedding, not BM25. Kept for cross-layer consistency
"Incremental update <5s" | VERIFIED | knowledge.txt L265
"Index path .elai_cc/projects/{id}/" | VERIFIED | knowledge.txt L195
"path field STRING not TEXT" | VERIFIED | tantivy examples, auditor verification
"project_id as first-class input" | VERIFIED | knowledge.txt L274
