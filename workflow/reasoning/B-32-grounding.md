GROUNDING SWEEP -- phase B-32
Source: F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-32_minified_file_chunking.md
Date: 2026-05-16
FSM: PLANNING | Session: s-50960

SPEC REFERENCES
indexer.ts:283-465 | GROUNDED | lines 282-465 read; contains findAstBoundaries, applyCharCap (344-351), chunkByCharacters (363-411), chunkFileContent (420-439) — all on-topic
indexer.ts:344-351 | GROUNDED | applyCharCap function confirmed at lines 344-351; truncates via substring(0, MAX_CHUNK_CHARS)
indexer.ts:363-411 | GROUNDED | chunkByCharacters confirmed; backwards-scan loop at 379-385 checks \n, space, \t, ;, , — exact match to plan claim
indexer.ts:420 (chunkFileContent) | GROUNDED | export function chunkFileContent at line 420; dispatch order: minified detect (432-439) FIRST -> tiny files -> AST -> line fallback — matches plan
indexer.ts:432-439 | GROUNDED | avgLineLength computation and MAX_AVG_LINE_LENGTH guard at 432-439; returns applyCharCap(chunkByCharacters(...))
constants.ts:92-128 | GROUNDED | MAX_AVG_LINE_LENGTH=500 at line 115; MAX_CHUNK_CHARS=2000 at line 128; doc comment at 122-126 matches plan's inline-doc quote verbatim
phase-3K.txt | GROUNDED | file exists; grep confirms: AST-aware chunking, 50-line fallback, chunk_type metadata — on-topic for plan context claim
DETAILED_SUMMARY_02_current_state.md:697-722 | GROUNDED | lines 697-722 read; R@5=0.6333, R@10=0.7667 across 30 queries confirmed at line 712; AST-replaces-50-line confirmed 697-701
bm25.rs (chunk_file + make_chunks_from_content) | GROUNDED | chunk_file at 366-401: try_ast_chunk FIRST, line_block_chunks fallback — no minified detection, no char cap — plan's gap claim confirmed

CLAIM CLASSIFICATION
MAX_AVG_LINE_LENGTH=500 chars/line threshold | GROUNDED | constants.ts:115 exact match
MAX_CHUNK_CHARS=2000 universal cap | GROUNDED | constants.ts:128 exact match
backwards-scan boundary chars: \n space \t ; , | GROUNDED | indexer.ts:381 exact: ch==="\n"||" "||"\t"||";"||","
minified detect FIRST before AST attempt | GROUNDED | indexer.ts:432-439 before AST block at 456-461
applyCharCap runs on ALL strategies (AST/line/char) | GROUNDED | indexer.ts:438,443,460,464 — four call sites
chunk ID uses byte offset not line number | GROUNDED | indexer.ts:394 chunkId(relativePath, offset) — offset is byte position
R@5=0.6333 R@10=0.7667 across 30 queries | GROUNDED | DETAILED_SUMMARY_02:712 verbatim
ELAI bm25.rs has no minified detect or char cap | GROUNDED | bm25.rs:384-386 try_ast_chunk then line_block_chunks; no cap step — gap confirmed
avg_line_length = content.len()/max(line_count,1) | GROUNDED | indexer.ts:432 exact: content.length/lines.length
floor_char_boundary needed for UTF-8 safety | VERIFY | Rust std::str::floor_char_boundary is nightly-only as of Rust 1.80 — plan uses it without flagging stability; needs stable alternative (char_indices scan or str::is_char_boundary loop)
nomic-embed-text-768d has 8192 token window | UNGROUNDED | Decision doc mentions this; not verified in any read source
Granite has 512 token context window | UNGROUNDED | Decision doc mentions this; not verified in any read source

ACTIONS REQUIRED
- verify floor_char_boundary stability: grep "floor_char_boundary" in .elai_cc/Cargo.toml and rust toolchain file; if nightly-only, plan must specify stable alternative (char_indices backward scan or is_char_boundary loop) before TESTING_FIRST
- UNGROUNDED claim "nomic-embed-text-768d has 8192 token window" in B-32-decisions.md scope — verify via model card or source before committing the 2000-char cap rationale
- UNGROUNDED claim "Granite has 512 tokens" — verify embedding model context for ELAI's deployed instance before finalizing MAX_CHUNK_CHARS rationale

VALIDATION WARNINGS
DETAILED_SUMMARY_02 cited without full path in plan ("DETAILED_SUMMARY_02_current_state.md" is actual filename); path resolved correctly during sweep — grounding succeeded but plan should use full filename to avoid ambiguity
all other sources validated
