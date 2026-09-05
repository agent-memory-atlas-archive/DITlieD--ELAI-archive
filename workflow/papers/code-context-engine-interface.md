Source: code-context-engine (CCE)
Reliability: HIGH (reviewed source, active, v0.4.24, 112 PRs, commit 2026-06-21)
Repo: https://github.com/elara-labs/code-context-engine (commit 144332b, 2026-06-21)
License: MIT
Files examined: src/context_engine/compression/compressor.py,
  src/context_engine/compression/prompts.py, src/context_engine/compression/quality.py,
  src/context_engine/memory/compressor.py, src/context_engine/memory/grammar.py,
  src/context_engine/memory/extractive.py, README.md, pyproject.toml

--- WHAT IT ACTUALLY DOES ---
A FULL code-indexing + context-management system for AI coding agents (Claude Code, Cursor, etc.).
Hooks into editor sessions, captures tool events + prompts, stores in SQLite (memory.db),
indexes the codebase with embeddings, serves context via MCP.

Token reduction happens at MULTIPLE layers:
  1. grammar.py: deterministic article/filler dropper on stored turn summaries (lite/full/ultra)
  2. extractive.py: embedding-centroid sentence selection (top-K from turn) — LOSSY
  3. compression/compressor.py: LLM-based (Ollama) code chunk summarizer — LOSSY
  4. retrieval/retriever.py: vector + FTS retrieval — RETRIEVAL-BASED

--- COMPRESSION CLASSIFICATION ---
MULTI-MODAL — three distinct mechanisms with different loss profiles:

Layer A — grammar.py (deterministic, fast, default=lite):
  Tokenizes text into structured (code/path/URL/identifier) and prose tokens.
  Structured tokens: code fences, inline code, URLs, datetimes, versions, paths, identifiers
    -> all pass byte-for-byte through compress().
  Prose tokens: articles (lite), fillers (full), fillers+abbreviations (ultra) are dropped.
  DEFAULT_LEVEL = "lite" (drops only a/an/the).
  LOSSY on articles/fillers. NOT reversible (expand() restores abbreviations only, not dropped words).
  DETERMINISTIC: no LLM call. Pure Python, no IO.

Layer B — extractive.py (embedding-based, runtime cost):
  Splits turn text into sentences. Embeds each with bge-small (or passed embedder).
  Centroid = mean of all embeddings. Top-K sentences by cosine similarity to centroid.
  Returns selected sentences in source order — real text, no synthesis.
  LOSSY: non-central sentences are dropped permanently from the turn summary.
  NOT REVERSIBLE. Falls back to truncation if embedder unavailable.
  Fallback: truncation_summary() at max_chars=200.

Layer C — compression/compressor.py (LLM-based, optional Ollama):
  Used for CODE CHUNK indexing (not turn memory). Summarizes function/class/module chunks
  via a local Ollama model (default phi3:mini).
  Quality gate: checks that >= 40% of identifiers from original survive in summary.
  If quality fails or Ollama unavailable: falls back to _extract_signature() or truncation.
  LOSSY on code semantics — LLM-generated summary may miss edge cases.
  Cached per (chunk_id, level) in LocalBackend.
  Levels: "minimal" (truncation only, 100 chars), "standard" (LLM, 300 char fallback),
    "full" (passthrough if confidence_score > 0.8 OR no embedding).

Layer D — retrieval (RETRIEVAL-BASED):
  vector_store + fts_store query; confidence scoring; cross-encoder reranker.
  Full chunk text retrieved on demand. Not summarized in the retrieval path itself.

Public API:
grammar.compress(text, level="full") -> str | deterministic prose compression
grammar.compress_with_counts(text, level) -> tuple[str, int, int] | adds token counts for metrics
grammar.expand(text) -> str | reverses abbreviations only (ultra level), not dropped words
grammar.DEFAULT_LEVEL | "lite" | single source of truth; all compression paths must agree
extractive.extractive_summary(text, embedder, top_k=3) -> str | centroid-based sentence selection
extractive.truncation_summary(text, max_chars=200) -> str | hard fallback
extractive.split_sentences(text) -> list[str] | coarse sentence splitter (punctuation + newlines)
Compressor.__init__(ollama_url, model="phi3:mini", cache=None) -> None | duck-type cache check
Compressor.compress(chunks, level="standard") -> list[Chunk] | async, mutates chunk.compressed_content
compress_turn(conn, session_id, prompt_number, embedder) -> str | extractive then grammar
compress_session_rollup(conn, session_id, embedder) -> str | rollup from turn summaries
compression_loop(db_path, embedder, interval_seconds=5.0, stop_event) -> None | asyncio drain loop

Behavioral contracts:
grammar.compress -> structured tokens (code/path/URL/version/ident) ALWAYS survive byte-for-byte
grammar.compress -> prose words in drop_set are removed; punctuation attached to dropped word is kept
grammar.compress -> pure function, no IO, no LLM, deterministic
grammar.expand -> restores abbreviations only at ultra level; NOT a lossless inverse of compress
grammar.DEFAULT_LEVEL = "lite" -> only articles dropped; must be consistent across all 5 call sites
extractive_summary -> returns real sentences from input, never synthesizes new text
extractive_summary -> returns verbatim input if len(sentences) <= top_k (no truncation below threshold)
Compressor.compress -> checks Ollama liveness (cached 30s TTL); falls back to truncation silently
Compressor._fallback_compress -> extracts function signature for FUNCTION/CLASS chunks, truncates others
compress_turn -> two-pass: extractive first, then grammar compress, then PII scrub
compress_turn -> PII scrub (scrub_pii) runs BEFORE grammar compress on the extractive summary
compression_loop -> drains at most 5 items per batch (BACKLOG_BATCH), sleeps 5s between empty polls
compression_loop -> runs heavy work via asyncio.to_thread to avoid blocking MCP stdio loop

Platform constraints:
Python 3.12+ (from pyproject.toml type annotations: str | None etc.)
SQLite for memory.db storage — all session/turn/decision/vector data
Ollama required for LLM-based code compression (optional — falls back gracefully)
bge-small or compatible embedder required for extractive summarization (falls back to truncation)
asyncio required — compression_loop is a long-running async task
grammar.py: pure stdlib, no external deps, no IO
extractive.py: pure stdlib (no numpy — manual cosine by design), no IO
Chunk.embedding: None signals chunk was not retrieved via retrieval pipeline (passthrough in "full" mode)

Gotchas:
grammar.expand() is NOT the inverse of grammar.compress(). Dropped articles/fillers are GONE.
  expand() is a read-side cosmetic step, not a decompressor.
Compressor at level="full" with confidence_score > 0.8: passes chunk UNCOMPRESSED (passthrough).
  Chunks ingested outside retrieval pipeline (embedding=None) also pass through — designed intent.
_approx_tokens heuristic: chars // 4. NOT actual BPE count. Savings metrics are approximate.
compression_loop uses a stashed embedder on _drain_one_threaded._embedder — a function attribute
  used as cross-thread state. Not thread-safe if multiple embedders are used simultaneously.
QualityChecker threshold: 40% identifier retention. A summary retaining 39% is SILENTLY rejected
  and falls back to truncation — not an error, just a tier downgrade.
LLM compression (Ollama) on code is lossy — the summary may omit edge cases, error handling paths,
  or parameter constraints not surfaced in the 2-3 sentence summary.
turn_summaries table stores post-grammar-compress form; expand() is applied at READ time only.
PII scrub on turn summaries: emails/IPs/SSNs scrubbed before indexing.

Critical assertions (testable):
# grammar structured-token preservation
import re; code_block = "```python\ndef f(x): return x\n```"; result = grammar_compress(code_block); assert code_block in result
import re; url = "https://example.com/api"; result = grammar_compress("check " + url); assert url in result
# grammar idempotence (lite level only — ultra abbreviations may not be idempotent)
t = grammar_compress("use the JWT token", level="lite"); assert grammar_compress(t, level="lite") == t
# extractive returns subset of input sentences
sentences = split_sentences(text); summary = extractive_summary(text, embedder=e, top_k=3)
assert all(s in text for s in split_sentences(summary))  # only real sentences
# quality gate
qc = QualityChecker(); assert qc.check("def foo(bar): pass", "foo takes bar") == True
assert qc.check("def authenticate(user, password): pass", "x y z") == False  # < 40% identifier retention

Complexity:
grammar.compress: O(n) tokenize + O(n * P) pattern matching where P=8 patterns; O(n) prose scan
extractive_summary: O(S * D) embed where S=sentences, D=embedding dim; O(S^2) cosine scoring -> O(S log S) sort
Compressor._llm_compress: O(HTTP round-trip to Ollama) ~100-500ms per chunk
compression_loop: drains 1 item/tick, 5s sleep when empty, 50ms breath every 5 items
SQLite: all reads/writes serialized per connection; no concurrent write support

Failure modes:
Ollama unavailable: Compressor silently falls back to truncation (log at INFO level)
extractive_summary embedder raises: falls back to truncation_summary at 200 chars
QualityChecker < 40% identifier retention: silently falls back to truncation
grammar.expand collapses "b/c" -> "because" but "b/c" in a path would be structured token
  and pass through — path detection is priority-ordered above prose
grammar at "ultra": abbreviation collisions possible (resp = both "responsible" and "response")

Implementation pitfalls (do NOT do this):
Do NOT call grammar.expand() expecting to recover the original text — dropped words are gone
Do NOT use Compressor level="minimal" expecting LLM summarization — it uses truncation-only
Do NOT trust _approx_tokens (chars//4) as accurate BPE counts — only suitable for relative comparisons
Do NOT assume LLM compression (Ollama) preserves all code semantics — quality gate only checks identifiers
Do NOT run compression_loop on more than one thread with different embedders — _drain_one_threaded._embedder is a shared function attribute
Do NOT skip the QualityChecker fallback in custom Compressor extensions — silent truncation is intentional UX

ELAI FIT:
STRONG FIT on architecture — CCE's memory pipeline (turn capture -> extractive summary -> grammar compress
  -> SQLite + vector index -> retrieval on demand) is architecturally aligned with ELAI's
  append-only journal + DKG + BM25 + vector + cross-encoder reranker stack.

DIVERGENCES from ELAI doctrine:
  1. LLM compression (Ollama/phi3:mini) for code chunks: BANNED by ELAI's "ciphers + LLM-summarization
     of cache prefix BANNED" rule. The summary is lossy and non-reversible.
     VERDICT: the code chunk LLM compressor (compression/compressor.py) is NOT adoptable as-is.
  2. Extractive summarizer for turn memory: retrieval-based (real sentences selected, not synthesized).
     This is acceptable under ELAI's "append-only journal = lossless record" if the ORIGINAL turn text
     is retained in the journal and the summary is a retrieval index, not a replacement.
     VERDICT: extractive.py is adoptable if ELAI keeps the raw journal and uses the summary only for retrieval.
  3. grammar.py (deterministic article/filler dropper): fully deterministic, structured tokens preserved
     byte-for-byte. At DEFAULT_LEVEL="lite" only articles dropped — low-risk lossy prose compression.
     VERDICT: grammar.py is the most directly adoptable component. Maps to ELAI's "deterministic-reversible"
     preference (reversible at ultra level for abbreviations; at lite level, only articles lost).

BEST STEAL: grammar.py tokenizer + structured-token protection pattern. The priority-ordered regex
  approach (fence > inline_code > URL > datetime > version > path > identifier > prose) is cleaner
  than honey-for-devs' PROTECT array and handles more structured-token types including identifiers
  and versions. Low code volume, pure stdlib, no external deps.

SECOND STEAL: the extractive.py cosine-centroid sentence selector. Deterministic once embedder output
  is fixed. No numpy dep. Returns only real source sentences. Maps to ELAI's retrieval stack.

LLM compressor: DO NOT STEAL — violates ELAI's deterministic-reversible-only rule.
Compression type by layer: grammar=LOSSY-PROSE/LOSSLESS-CODE, extractive=LOSSY-RETRIEVAL-BASED,
  LLM-chunk=LOSSY-IRREVERSIBLE, retrieval=RETRIEVAL-BASED-LOSSLESS.
