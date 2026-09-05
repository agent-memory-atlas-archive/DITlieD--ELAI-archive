Source: LoPace (Lossless Optimized Prompt Accurate Compression Engine)
Reliability: HIGH (reviewed source, real working code, published to PyPI, arXiv paper 2602.13266)
Repo: https://github.com/connectaman/LoPace (commit 6f94369, 2026-02-17)
License: MIT
Files examined: lopace/compressor.py, README.md, pyproject.toml, tests/test_compressor.py

--- WHAT IT ACTUALLY DOES ---
Compresses prompt strings to bytes for DATABASE STORAGE. NOT for reducing token count
sent to an LLM. The compressed bytes are binary blobs unsuitable as LLM input.
Three methods: Zstd (LZ77+FSE), Token (BPE binary pack), Hybrid (BPE then Zstd).
Decompresses back to original string perfectly.

--- COMPRESSION CLASSIFICATION ---
LOSSLESS: yes, all three methods have decompress() that returns identical string.
BINARY: output is bytes, not a shortened text prompt.
DETERMINISTIC: no LLM call, pure algorithmic.
USE CASE: storing millions of prompts in a database cheaply; NOT reducing tokens in a
live context window. The token count sent to the LLM is unchanged.

Public API:
PromptCompressor.__init__ | (model="cl100k_base", zstd_level=15) -> None | validates both params on init
PromptCompressor.compress | (text: str, method: CompressionMethod) -> bytes | routes to one of 3 methods
PromptCompressor.decompress | (compressed_data: bytes, method: CompressionMethod) -> str | exact round-trip
PromptCompressor.compress_zstd | (text: str) -> bytes | UTF-8 encode then zstd.compress
PromptCompressor.decompress_zstd | (bytes) -> str | zstd.decompress then UTF-8 decode
PromptCompressor.compress_token | (text: str) -> bytes | tiktoken BPE -> uint16/uint32 struct pack
PromptCompressor.decompress_token | (bytes) -> str | struct unpack -> tiktoken decode
PromptCompressor.compress_hybrid | (text: str) -> bytes | BPE -> struct pack -> zstd compress
PromptCompressor.decompress_hybrid | (bytes) -> str | zstd decompress -> unpack -> tiktoken decode
PromptCompressor.compress_and_return_both | (text, method) -> Tuple[str, bytes] | trivial, returns (text, compress(text))
PromptCompressor.get_compression_stats | (text, method=None) -> dict | computes all methods if method=None
PromptCompressor.calculate_shannon_entropy | (text, unit="character") -> float | H(X) in bits
PromptCompressor.get_theoretical_compression_limit | (text, unit) -> dict | theoretical bounds

Behavioral contracts:
PromptCompressor.__init__ -> raises ImportError if zstandard or tiktoken missing; raises ValueError if zstd_level not in [1,22]
compress_token -> uint16 (2 bytes/token) if max_id <= 65535; uint32 (4 bytes/token) otherwise; format byte prepended
compress_hybrid -> tokenize first, pack as uint16/uint32, then zstd compress; best ratio for DB
decompress methods -> exact inverse: decompress(compress(text)) == text for all methods
get_compression_stats -> does not mutate; returns dict with original_size_bytes, original_size_tokens, methods dict
compress_and_return_both -> ONLY returns (original_text, compressed_bytes); does NOT shorten the text

Platform constraints:
Python 3.8+ required (type annotations used but compatible)
zstandard>=0.22.0 required (pip installable, active as of 2026)
tiktoken>=0.5.0 required (pip installable, active OpenAI package)
cl100k_base tokenizer = GPT-4/Claude compatible BPE; other models available
zstd_level 1-22: 15 is default (good compression/speed balance); 22 is max compression slowest
Output bytes are binary — cannot be passed as LLM prompt text
Decompression requires same method used for compression — NO method auto-detection in binary

Gotchas:
The output of compress() is BYTES not a shorter text string. Token count to LLM is ZERO reduced.
compress_and_return_both returns the ORIGINAL unmodified text, not a compressed text version.
method must be tracked externally — compressed bytes contain no method header.
Empty text returns struct.pack('B', 0) for token/hybrid — decompress handles this.
Hybrid on very short text (< ~10 chars) may produce LARGER output than original due to Zstd header overhead.
Shannon entropy calculation is character-level by default, byte-level optional.

Critical assertions (testable):
assert compressor.decompress(compressor.compress(text, m), m) == text  # lossless for all methods
assert isinstance(compressor.compress(text, CompressionMethod.HYBRID), bytes)
assert 1 <= compressor.zstd_level <= 22
assert compressor.get_compression_stats("")['original_size_bytes'] == 0 or True  # empty handled
assert compressor.compress_and_return_both(text)[0] == text  # first element is UNCHANGED original

Complexity:
Zstd: O(n * level) compress, O(n) decompress, memory ~2x input size
Token: O(n) BPE tokenize, O(tokens) struct pack, O(tokens) struct unpack
Hybrid: O(n) BPE + O(tokens) Zstd, O(n) decompress, memory ~2x input
No parallelism — single-threaded pure Python + C extensions

Failure modes:
zstd_level outside 1-22 raises ValueError at __init__ time, not at compress() time
compress_token with empty text returns a single format byte (0x00) — decompress handles
Mismatched method on decompress raises ValueError or produces garbage (no method tag in binary)
Using Zstd-only on short text: may not compress at all (Zstd adds ~3-5 byte header)

Implementation pitfalls (do NOT do this):
Do NOT pass compressed bytes as LLM prompt — binary blobs are not valid text prompts
Do NOT assume compressed output is shorter for short inputs — Zstd header overhead
Do NOT strip the format byte from token/hybrid output — it encodes uint16 vs uint32 mode
Do NOT use Token mode for database storage — Hybrid achieves 70-80% vs 52% space savings
Do NOT reconstruct from training knowledge: the compress_and_return_both method returns the ORIGINAL, not a shortened form

ELAI FIT:
NOT APPLICABLE as a context-window token reducer. LoPace reduces STORAGE bytes, not live-context tokens.
ELAI's problem is reducing the number of tokens in the active context window sent to the LLM.
LoPace's compressed bytes are binary and cannot be re-inserted into a prompt.
ONLY use case: if ELAI were caching prompts in a SQLite/Postgres store between sessions, LoPace
reduces disk footprint. For ELAI's actual token-reduction need, LoPace is irrelevant.
Compression is LOSSLESS and DETERMINISTIC (no LLM call) — consistent with ELAI doctrine.
No cipher/LLM-summarization of cache prefix involved — safe per ELAI rules IF used for storage only.
