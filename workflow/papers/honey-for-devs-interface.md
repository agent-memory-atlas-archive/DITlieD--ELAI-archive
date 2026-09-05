Source: honey-for-devs (Honey — I Shrunk the AI)
Reliability: HIGH (reviewed source, active, well-tested, 18 closed PRs, commit 2026-06-25)
Repo: https://github.com/Green-PT/honey-for-devs (commit 9682ea3, 2026-06-25 — ACTIVE TODAY)
License: MIT
Files examined: hooks/precompress.js, hooks/logcompress.js, eso/ccr.js, eso/index.js,
  skills/honey/SKILL.md, bench/eso/VERDICT.md, bench/input/RESULTS.md,
  hooks/precompress.test.js

--- WHAT IT ACTUALLY DOES (THREE DISTINCT SYSTEMS) ---

System 1: precompress.js — deterministic prose filler stripper for USER INPUT PROMPTS
System 2: logcompress.js — deterministic ANSI-strip + consecutive-repeat collapser for TOOL OUTPUT
System 3: ESO / CCR — structured wire format + retrieval-based array offloading for AGENT-TO-AGENT messages

--- COMPRESSION CLASSIFICATION PER SYSTEM ---

System 1 (precompress):
  LOSSY on prose content — drops "please", "thanks", "basically", greetings, filler phrases,
  duplicate sentences, duplicate lines. Compression is NOT reversible.
  LOSSLESS on technical content — ALL protected spans pass byte-for-byte:
    fenced code blocks, inline code, URLs, paths, filenames, double-quoted strings, digit runs.
  DETERMINISTIC: no LLM call.
  MEASURED RESULT (verified by bench/input/RESULTS.md): On real human-typed prompts from
    35 sessions, median saving = 0.0%, 219/266 prompts cut nothing, total saving 2.5%.
    On verbose padded prompts: median 13% cut with 100% unit test pass rate preserved.
  QUALITY VERDICT: Safe on code-correctness. The bench proves (Run B, 10 tasks, 100%->100%
    unit test pass) that removing filler from a prompt does NOT degrade code output quality.
    The critical safety property: protected spans (code blocks, identifiers, paths) survive
    verbatim. Code correctness is NOT at risk because the code spec cannot be in an
    unprotected prose position that would be stripped.

System 2 (logcompress):
  LOSSY on volatile per-line values (timestamps, hex addresses, UUIDs) — these are masked
    to templates before deduplication; original values in collapsed runs are dropped.
  COUNT is preserved: "line ×N" marker records how many times the line repeated.
  LOSSLESS on distinct lines — only consecutive identical-template runs collapse.
  DETERMINISTIC: no LLM call. ANSI stripping is always lossless.
  Gate: only fires for tool output >=25 lines. Below that only ANSI is stripped.
  Expand function restores the ×N count but NOT per-line volatile values.

System 3 (ESO + CCR):
  ESO: a tab-separated columnar wire format. Lossless encoding: encode()/decode() are
    exact inverses. Saves ~28% tokens vs pretty JSON. REQUIRES format primer to parser.
  CCR (Compress-Cache-Retrieve): RETRIEVAL-BASED. Keeps head + tail + change-point rows
    of an array visible; offloads dropped rows to a hash-keyed cache. The agent can
    retrieve the full array by hash when it needs a dropped row. NOT lossless in the
    context window (some rows are not visible) but original is recoverable.

--- HONEY SKILL (SKILL.md) ---
The SKILL.md is a writing discipline for LLM output, NOT a compression algorithm.
Lever 1: minimum code (YAGNI, stdlib-first, no speculative generality).
Lever 2: terse prose output (no hedging, no preamble, fragments OK).
Lever 3: agent-to-agent wire format (columnar/ESO, compact JSON, never pretty).
Lever 3b: read less input (grep before read, offset+limit reads, offload via ESO stash).
This is a BEHAVIORAL GUIDELINE, not a programmatic transform. No API.

Public API:
compress(input: string) -> {text: string, saved: number} | precompress.js | deterministic filler strip
logcompress.compress(text: string) -> {view: string, saved: number, dropped: number} | logcompress.js
logcompress.expand(view: string) -> string | restores ×N markers, NOT volatile values
ccr.crush(array: any[], options?) -> {view: any[], hash: string|null, dropped: number} | array offload
ccr.strip(view: any[]) -> any[] | removes CCR sentinel markers from a crushed view
eso.encode(obj: object) -> string | columnar text format, lossless
eso.decode(source: string) -> object | exact inverse of encode

Behavioral contracts:
precompress.compress -> fails open if input contains sentinel char (returns input unchanged)
precompress.compress -> idempotent: compress(compress(x)).text === compress(x).text
precompress.compress -> all code spans, URLs, paths, digit runs survive verbatim (safety contract)
logcompress.compress -> ANSI stripping always applied; collapse only at >=25 lines
ccr.crush -> passthrough (hash=null) if array.length < minItems (default 5)
ccr.crush -> keeps first + last + change-point rows up to maxItems (default 15)
ccr.crush -> appends sentinel {_ccr: "<<ccr:HASH N_rows_offloaded>>"} to view
eso.encode -> raises if root is not a plain object; raises on cyclic values or sparse arrays
eso.encode -> record arrays must have a single uniform schema (all rows same keys in same order)

Platform constraints:
Node.js runtime required (CommonJS modules, require())
node:crypto used for sha256 hashing in CCR (stdlib, no external dep)
No Python version; no browser support (uses node:crypto)
CCR file cache lives in bin/eso.js (separate from the pure ccr.js module)
precompress sentinel uses private-use Unicode codepoint — inputs containing it fail-open

Gotchas:
precompress on real prompts: 0% median saving on real traffic (bench-verified negative result).
  The verbose benchmark (16.5%) is NOT representative of real usage.
logcompress drops volatile per-line values (timestamps, hex addrs, UUIDs) in collapsed runs.
  If those values matter (e.g. a specific failing UUID), logcompress must NOT be applied.
CCR is retrieval-based: dropped rows require a file-system cache read to recover.
  If the cache is unavailable, the dropped rows are gone from context.
ESO breaks-even vs columnar JSON only after ~2 cached messages (primer cost 126 tokens).
  For scalar-heavy or short payloads, ESO LOSES vs compact JSON.
Single-quoted strings are NOT protected by precompress (apostrophe ambiguity).
Honey SKILL.md is a behavioral guideline, not a runnable API — cannot be imported.

Critical assertions (testable):
# precompress safety
result = compress(text_with_code_block); assert code_block in result['text']
result = compress(text_with_url); assert url in result['text']
result = compress(x); assert compress(result['text'])['text'] == result['text']  # idempotent
# logcompress
out = logcompress_compress("line\nline\nline\n"); assert "×3" in out['view']  # collapse marker
# ccr
v = ccr_crush(list(range(20))); assert v['dropped'] > 0; assert v['hash'] is not None
# eso
assert eso_decode(eso_encode({'x': [1,2,3]})) == {'x': [1,2,3]}

Complexity:
precompress: O(n * P) where P = number of protect patterns (constant 7); O(n) prose scan
logcompress: O(n) line scan; O(n) for template matching
CCR: O(n log n) for sort; O(n) for SHA-256 hash of full array via JSON.stringify
ESO encode/decode: O(n) rows * O(k) fields per row

Failure modes:
precompress fails open when sentinel char in input (returns input unchanged, saved=0)
precompress single-quoted strings NOT protected — '`x`' is safe, "'x'" is not
logcompress below GATE_LINES (25): only ANSI stripped, no collapse
CCR with no file cache: dropped rows are irrecoverable in that context window
ESO decode on malformed input: throws SyntaxError — no graceful fallback built in

Implementation pitfalls (do NOT do this):
Do NOT apply precompress to tool outputs expecting meaningful token savings — 0% median on real prompts
Do NOT use CCR without a file-system cache for the originals — dropped rows become irrecoverable
Do NOT use ESO for scalar-heavy or short payloads — primer cost (126 tokens) exceeds savings
Do NOT apply logcompress when specific UUID/timestamp values in collapsed runs are needed
Do NOT treat the Honey SKILL.md as code you can call — it is a behavioral instruction for an LLM

--- HONEY-FOR-DEVS QUALITY VERDICT (user concern: "reduce tokens but NEVER reduce code quality") ---

VERDICT: SAFE FOR PROSE FILLER REMOVAL, WITH IMPORTANT CAVEATS.

The precompress bench (bench/input/RESULTS.md Run B) proves code-correctness is preserved:
10 coding tasks, verbose filler added around real prompts, then compressed. Unit tests:
100% pass rate before and after compression. The safety contract is real and verified.

WHY IT IS SAFE: the code spec lives in protected spans (fenced/inline code, paths, quoted
strings, identifiers). The only things removed are courtesy phrases, duplicate lines, and
redundant connectives — none of which change what code the LLM writes.

WHY REAL-WORLD SAVINGS ARE MARGINAL: median saving on real human-typed prompts is 0%.
Real developers already write terse prompts. The compressor is safe but mostly inert.

THE QUALITY RISK ZONE: logcompress IS lossy on volatile per-line values. If a developer
needs a specific failing UUID or timestamp from a build log, logcompress on that log
corrupts the diagnostic value. Risk is limited to tool output, not code prompts.

ELAI FIT:
precompress: moderate fit — input filler removal is deterministic and lossless on code spans.
  But real savings are negligible (0% median on real traffic). Not worth hook integration cost
  unless user population generates verbose prompts. The safety claim is grounded.
logcompress: moderate fit — deterministic ANSI strip is always useful. The consecutive-repeat
  collapse maps naturally to ELAI's tool output reduction need. Lossy on volatile values —
  acceptable for progress bars and retry storms, NOT for single-occurrence diagnostic events.
ESO: limited fit — ELAI's existing DKG+BM25+vector stack already handles the retrieval use case.
  ESO wire format competes with ELAI's own structured SSB payload serialization.
CCR: moderate fit — the retrieval-based array offloading maps directly to ELAI's cross-encoder
  reranker use case (show top-K, retrieve rest on demand). Clean design.
Compression type by system: precompress=LOSSY-PROSE/LOSSLESS-CODE, logcompress=LOSSY-VOLATILE/LOSSLESS-DISTINCT, ESO=LOSSLESS-ENCODING, CCR=RETRIEVAL-BASED.
No LLM calls in any compression path — fully deterministic — consistent with ELAI's
deterministic-reversible-only doctrine for the active context window.
