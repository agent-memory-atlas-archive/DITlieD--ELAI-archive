repomap-xgrammar
Target: C:\Users\user\Downloads\new\xgrammar-main  (mlc-ai/xgrammar, pyproject version 0.2.2, "XGrammar-2" era)
Mapped: 2026-06-15. Method-focused. Read selectively: all docs/, public Python API, core C++ headers + the load-bearing bodies of grammar_compiler.cc / grammar_matcher.cc / earley_parser.h / tokenizer_info.cc / the CUDA kernel. Data/fixtures/tests not read.


1. WHAT IT IS

XGrammar is a C++/CUDA engine (with Python/JS/Swift bindings) for constrained / grammar-guided decoding: forcing an LLM's token-by-token output to conform 100% to a structure (JSON Schema, regex, arbitrary EBNF context-free grammar, or a composite "structural tag" describing reasoning + free text + tool calls). The problem it solves is that an LLM left to free-sample produces structurally-broken output (a JSON int emitted as a string, a missing brace, a malformed tool-call wrapper). XGrammar guarantees structural correctness by computing, at every decode step, a token bitmask over the whole vocabulary that zeroes the probability of any token that would violate the grammar, and does so with near-zero per-token overhead via a precompiled "adaptive token mask cache". It is the default structured-output backend for vLLM, SGLang, TensorRT-LLM, MLC-LLM, and others.


2. STRUCTURE MAP (modules only, one-line role)

include/xgrammar/          public C++ API headers
  grammar.h                Grammar handle: from_ebnf / from_json_schema / from_regex / from_structural_tag
  compiler.h               GrammarCompiler + CompiledGrammar (per-model preprocessing result)
  matcher.h                GrammarMatcher (stateful per-request) + BatchGrammarMatcher; bitmask apply
  tokenizer_info.h         TokenizerInfo: vocab + vocab-type + stop tokens, per-model
  config.h / exception.h / object.h   recursion-depth config, error types, PImpl base

cpp/                       engine implementation
  grammar.cc / grammar_impl.h          Grammar object + internal IR (rules as choices/sequences of byte-strings, char-classes, rule refs, tag-dispatch FSM nodes)
  grammar_parser.cc / grammar_builder.cc / ebnf_script_creator.h   EBNF text -> Grammar IR
  grammar_functor.cc / grammar_functor.h  (2.7k LOC) grammar transform passes: inline, dead-rule elim, normalization, lookahead-assertion analysis, char-class handling
  grammar_compiler.cc      (1.5k LOC) THE PERF CORE: builds the AdaptiveTokenMask cache per parser state (accepted/rejected/uncertain split), first-character-mask speculative pruning, lookahead adaptation, multi-threaded + LRU-cached
  compiled_grammar.cc / compiled_grammar_impl.h   CompiledGrammar = Grammar + TokenizerInfo + adaptive_token_mask_cache map; serialization
  grammar_matcher.cc       (1.3k LOC) THE RUNTIME CORE: accept_token / accept_string, fill_next_token_bitmask using the cache, jump-forward string, draft-tree (speculative decoding) traversal, rollback/fork
  earley_parser.cc / .h    (1k LOC) Earley parser (replaced the old pushdown-automaton in v0.1.21): predict/scan/complete over byte input, multiple parallel states, char-class + UTF-8 + repeat + tag-dispatch-FSM edges
  fsm.cc / fsm_builder.cc  (2k LOC) finite-state-machine construction (Thompson/compaction) used for tag-dispatch and regex legs; Aho-Corasick-style multi-pattern dispatch
  json_schema_converter.cc / _ext.cc   (3.2k LOC) JSON Schema -> EBNF grammar (incl. XML-parameter styles for Qwen/MiniMax/DeepSeek/GLM)
  regex_converter.cc       regex -> EBNF grammar
  structural_tag.cc / .h   (2.4k LOC) the structural-tag tree (tag / triggered_tags / dispatch / token-level variants) -> grammar
  tokenizer_info.cc        vocab decode (raw / byte-fallback / byte-level), sorted-vocab + byte-trie subtree ranges, vocab-type + stop-token detection
  support/                 dynamic_bitset, compact_2d_array, int_set, thread_pool, thread_safe_cache (LRU), recursion_guard, reflection-based JSON serializer, encoding (UTF-8)
  tvm_ffi/                 FFI bridge exposing C++ to Python via apache-tvm-ffi
  testing.cc               debug/introspection helpers

python/xgrammar/           Python API (thin wrappers over _core FFI)
  grammar.py / compiler.py / matcher.py / tokenizer_info.py   mirror the C++ classes
  structural_tag.py / builtin_structural_tag.py   structural-tag builders; get_model_structural_tag (per-model tool-call formats)
  openai_tool_call_schema.py   pydantic models for OpenAI tools / tool_choice shapes
  kernels/                 apply_token_bitmask_inplace in 6 backends: cpu, cuda(.cu), triton, torch, torch_compile, mlx
  contrib/hf.py, contrib/mlxlm.py   integration helpers (HF LogitsProcessor, MLX)
  config.py / exception.py / load_binding.py / base.py

docs/                      tutorials (workflow, constrained_decoding, engine_integration, json/ebnf generation, advanced_topics), structural_tag/, xgrammar_features/ (serialization, runtime_safeguards, javascript_api)
3rdparty/                  cpptrace, dlpack, googletest, picojson (vendored)


3. ARCHITECTURE + METHOD (the core)

3.1 Pipeline (per-model setup vs per-request runtime)
PER-MODEL (once): TokenizerInfo  ->  GrammarCompiler(tokenizer_info)  ->  CompiledGrammar (reusable, cacheable).
PER-REQUEST: GrammarMatcher(compiled_grammar) (carries mutable parse state).
PER-DECODE-STEP loop (docs/tutorials/workflow_of_xgrammar.md):
  1. matcher.fill_next_token_bitmask(bitmask)   -> bitmask is int32 bitset, shape (batch, ceil(vocab/32)), on CPU
  2. apply_token_bitmask_inplace(logits, bitmask.to(device))  -> sets disallowed logits to -inf (CUDA/Triton/CPU kernel)
  3. sample next_token from softmax(logits)
  4. matcher.accept_token(next_token)  -> advances parse state
  5. repeat until matcher.is_terminated() (stop token accepted) or completed.
Separation of concerns: mask GENERATION is CPU/grammar-side (it has to be, the parse state lives there), mask APPLICATION is a GPU kernel fused into the logits. The bitmask lives on CPU because it is filled by CPU grammar logic, then moved to the logits' device for the masked-fill.

3.2 Grammar representation
All input forms normalize to one EBNF/CFG IR (grammar_impl.h). A rule is a set of CHOICES; a choice is a SEQUENCE; a sequence element is one of: byte-string literal, character-class, character-class-star (repetition), a rule reference, or a tag-dispatch FSM node. JSON Schema (json_schema_converter.cc) and regex (regex_converter.cc) are compiled DOWN to this same IR, so the matcher has one execution model. grammar_functor.cc runs transform passes (inline, dead-rule elimination, normalization, lookahead-assertion extraction) before compilation.

3.3 Parser: Earley over BYTES (earley_parser.h/.cc)
Since v0.1.21 the matcher uses an Earley parser instead of a recursive pushdown automaton (docs/xgrammar_features/runtime_safeguards.md: this removes the stack-overflow / recursion-limit hazard entirely; no recursion during parsing). It runs on raw BYTES (uint8), not codepoints, with explicit partial-UTF-8-codepoint accumulation in ParserState.partial_codepoint. A ParserState is (rule_id, sequence_id, element_id, rule_start_pos, sub_element_id, repeat_count, partial_codepoint). The parser keeps MULTIPLE PARALLEL states (the nondeterminism of a CFG) and does the classic three operations:
  - PREDICT: expand a rule reference into its choices' start states.
  - SCAN: consume one input byte, advancing states whose next element matches (byte-string / char-class / char-class-star / tag-dispatch-FSM edge / atomic-token edge).
  - COMPLETE: when a rule finishes, advance the parent states that were waiting on it (rule_id_to_completable_states_, indexed by start position).
scanable_state_history_ is the per-position stack of live states; this is what enables O(1) ROLLBACK (PopLastStates) and FORK (deep-copy of state only, grammar/tokenizer shared) for beam search / speculative decoding. A RepeatDetector (hash-set + ring buffer, threshold 50) stops duplicate states from re-entering the work queue. is_completed_[i] records, per input position, whether the stop token is acceptable there.

3.4 THE KEY PERF TRICK: Adaptive Token Mask Cache (grammar_compiler.cc + compiled_grammar_impl.h)
Naively, generating the mask each step would require running the parser forward against every one of ~150k vocab tokens — far too slow. XGrammar precomputes, at COMPILE time, for each reachable parser state, an AdaptiveTokenMask that partitions the vocabulary into THREE sets (compiled_grammar_impl.h struct AdaptiveTokenMask):
  - ACCEPTED: tokens the current state alone proves valid (context-free of the parent stack).
  - REJECTED: tokens the current state alone proves invalid.
  - UNCERTAIN: tokens whose validity depends on the PARENT stack (what comes after this rule) — only these need real work at runtime.
Storage is adaptive (StoreType, the "adaptive" in the name): store only ACCEPTED indices, or only REJECTED indices, or a full ACCEPTED bitset — whichever is smallest. USE_BITSET_THRESHOLD = 1000: when both accepted and rejected sets are large, fall back to a DynamicBitset of accepted ids. Indices are stored as positions in a SORTED-decoded vocab, not raw token ids, which makes the runtime trie-walk contiguous.
Compile-time pruning tricks (grammar_compiler.cc):
  - First-character mask: a bitset<256> of which leading bytes any continuation can start with; tokens whose first byte is excluded are bulk-rejected without parsing (GetFirstCharacterMask / GetSpeculativeCalculation).
  - Byte-trie subtree ranges (tokenizer_info.cc trie_subtree_nodes_range_): the sorted vocab forms a prefix trie; when a token prefix is rejected, every token in its trie subtree is rejected in one range-skip (last_rejected_uncertain_range in the matcher).
  - Lookahead-assertion adaptation (AdaptCacheWithLookahead): folds grammar lookahead constraints into the cached mask so the uncertain set shrinks.
  - Multi-threaded compile (thread_pool.h) + LRU CompiledGrammar cache (thread_safe_cache.h), keyed on the raw EBNF/schema/regex STRING so re-compiling an identical grammar is a cache hit. compile_* releases the GIL, so Python can compile grammars in parallel via asyncio.to_thread.

3.5 Runtime mask fill (grammar_matcher.cc FillNextTokenBitmask, lines ~809-962)
At each step the matcher takes its set of latest scanable states and, per state, looks up the precomputed AdaptiveTokenMask. The final mask = UNION of accepted sets across all parallel states, and INTERSECTION of rejected sets. The only real computation is resolving the UNCERTAIN tokens: it walks them in sorted-vocab order, exploiting the trie structure to (a) skip a token already accepted by another state, (b) range-skip an entire rejected subtree, and (c) reuse the longest-common-prefix with the previous token (std::mismatch + PopLastStates) so it only Advances the parser over the NEW suffix bytes of each candidate. This is what collapses per-step cost from "parse against 150k tokens" to "parse against the small uncertain set, mostly via prefix reuse". Result is OR-ed into tmp_accepted_bitset_, then SetTokenBitmask writes the int32 bitset. can_reach_end (IsCompleted) controls whether the stop token is allowed.

3.6 Mask application kernel (python/xgrammar/kernels/apply_token_bitmask_inplace_cuda.cu)
LogitsBitmaskKernel: one thread block per (vocab-chunk x batch-row); each thread owns kBitsPerThread bits, uses PACKED vectorized loads (PackedT over half/bf16/fp32) and writes -inf (type-specialized NegativeInfinity for fp16/bf16) where the bit is 0. Supports an optional indices[] gather so a batched bitmask can map to a subset of logits rows (continuous-batching friendly). Same op exists in Triton, Torch, torch.compile, CPU, and MLX backends — chosen at runtime by device.

3.7 Structural Tag = composite grammar for agentic output (structural_tag.cc, docs/structural_tag/)
The big XGrammar-2 feature for tool-calling. A StructuralTag is a TREE of format objects that compiles to one grammar:
  - Primitive: const_string, json_schema (with json | qwen_xml | minimax_xml | deepseek_xml | glm_xml parameter styles), grammar (raw EBNF), regex, any_text.
  - Composition: sequence, or, optional, plus, star, repeat(min,max).
  - Tagging/dispatch: tag(begin,content,end), triggered_tags(triggers->tags, with at_least_one / stop_after_first), tags_with_separator, dispatch(rules, loop, excludes) — uses Aho-Corasick for multi-pattern trigger detection in free text.
  - Token-level twins: token, exclude_token, any_tokens, token_triggered_tags, token_dispatch — dispatch on special tokens (e.g. <|tool_call_start|>) instead of text strings.
The pattern: model emits free text (any_text) until a TRIGGER prefix appears (e.g. "<function="), then decoding SWITCHES into the schema-constrained region for that tool's arguments, then returns to free text. "Unlimited" formats (any_text/any_tokens/triggered_tags) inside a tag auto-use the enclosing end marker as their stop condition, so <think>...</think> stops cleanly. builtin_structural_tag.py::get_model_structural_tag(model, tools, tool_choice, reasoning) emits the correct wrapper for a named family (llama, qwen_3/_coder, kimi, deepseek_r1/v3_1/v3_2/v4, harmony/gpt-oss, glm_4_7, minimax) from OpenAI-shape tools, mapping tool_choice auto/required/forced/none and parallel_tool_calls onto the tree flags. register_model_structural_tag extends it.

3.8 Extra runtime features
  - Jump-forward decoding (FindJumpForwardString): when the grammar forces a deterministic continuation (e.g. the next chars MUST be `": "`), return that string so the engine can emit it WITHOUT an LLM call — free tokens.
  - Speculative / draft-tree decoding (TraverseDraftTree): DFS over a draft token tree (retrieve_next_token / retrieve_next_sibling), filling a per-node bitmask, with a wall-clock time_threshold guard.
  - Batch path: BatchGrammarMatcher multi-threads fill/accept across many matchers (default hw_concurrency/2 threads).
  - Serialization: Grammar / TokenizerInfo / CompiledGrammar each serialize to versioned JSON (reflection.h member tables); CompiledGrammar stores only TokenizerInfo METADATA and re-binds a provided TokenizerInfo on load, so a compiled grammar can be cached to disk or shipped cross-process. Version mismatch raises DeserializeVersionError.
  - Padding handling: vocab_size = the MODEL's logits width, which can exceed the tokenizer vocab (e.g. DeepSeek-V3 128815 tokens vs 129280 logits); pass config.vocab_size, not tokenizer.vocab_size.
  - Recursion safeguard (config.h): set_max_recursion_depth / max_recursion_depth context manager (default 10000) — now mostly moot post-Earley.


4. KEY TECHNIQUES PORTABLE TO ELAI

ELAI runs a local sidecar (Qwen2.5-Coder, HF transformers / SDPA / NF4) speaking the Anthropic protocol, and cares about tool-call schemas, JSON output, and grounded/structured generation. XGrammar is BUILT for exactly the local-sidecar decode loop. Portability notes:

(T1) Grammar-constrained decoding on the local sidecar — HIGHLY PORTABLE, lowest effort.
  Files: python/xgrammar/{compiler,matcher,tokenizer_info}.py, docs/tutorials/workflow_of_xgrammar.md.
  XGrammar ships on PyPI (pip install xgrammar, Apache-2.0, dep on torch>=1.10 + transformers>=4.38, both already in the sidecar env). The integration is the documented HF loop: build TokenizerInfo.from_huggingface(tokenizer, vocab_size=config.vocab_size) once, GrammarCompiler once, then per request a GrammarMatcher; in the sidecar's generation loop (where ELAI already runs a LogitsProcessor per CLAUDE.md "model.generate()+LogitsProcessor"), call fill_next_token_bitmask + apply_token_bitmask_inplace before sampling, accept_token after. contrib/hf.py provides a ready HF LogitsProcessor adapter. This makes the local sidecar's JSON / tool-call output 100% structurally valid instead of best-effort. Maps directly onto ELAI's interest in "tool-call schemas, JSON output, grammar-guided decoding".

(T2) Structural-tag tool-call enforcement matched to the model family — HIGHLY PORTABLE, highest ELAI value.
  Files: python/xgrammar/builtin_structural_tag.py (get_model_structural_tag), docs/structural_tag/tool_calling_and_reasoning.md.
  ELAI's local lane is Qwen2.5-Coder; get_model_structural_tag has qwen_3 / qwen_3_coder styles plus deepseek_v4 (ELAI's cheap cloud floor). Feed ELAI's existing OpenAI-shape tool catalog (the production_agent_rpc_tool_catalog) straight in; the helper emits the correct begin/end wrapper + per-tool JSON-Schema-constrained argument region + reasoning <think> handling, and maps tool_choice required/auto/forced and parallel_tool_calls. This structurally guarantees the local model emits a parseable tool call (right tool name, schema-valid args) — directly attacking the failure mode where a small local model emits a malformed tool call. CAVEAT: the exact wrapper depends on the served model's chat template; verify the style name matches the deployed Qwen2.5-Coder template (the builtin styles target Qwen3/3-coder; 2.5-coder's call format must be checked, or a custom style registered via register_model_structural_tag, or a hand-written triggered_tags tree used).

(T3) Jump-forward decoding for free deterministic tokens — PORTABLE, efficiency win.
  Files: GrammarMatcher.FindJumpForwardString (grammar_matcher.cc:964).
  When the schema forces literal continuation (JSON keys, braces, the tool wrapper boilerplate), emit it without an LLM forward pass. On a slow local sidecar this cuts real latency/tokens on structured output. Aligns with ELAI's efficiency-direction memory (cheaper via fewer model calls). Requires the sidecar loop to support injecting tokens between forward passes.

(T4) The adaptive-mask + byte-trie design pattern — PORTABLE AS A DESIGN, not as code to reimplement.
  Files: compiled_grammar_impl.h (AdaptiveTokenMask), grammar_compiler.cc, grammar_matcher.cc fill body.
  The accepted/rejected/UNCERTAIN three-way split + "precompute per-state, only resolve uncertain at runtime" + sorted-vocab trie subtree skipping is the reusable INSIGHT for any per-step vocab-filtering ELAI might build (not just grammars — e.g. constraining to a fixed enum of next actions). Do NOT hand-roll the engine; use XGrammar. But the pattern informs any "mask the vocab cheaply each step" feature.

(T5) Serializable CompiledGrammar cache — PORTABLE, ops win.
  Files: docs/xgrammar_features/serialization.md, compiled_grammar.cc.
  Compiling a grammar is the expensive part; CompiledGrammar.serialize_json lets ELAI cache compiled tool-call grammars to disk keyed by (tool catalog hash + tokenizer), re-binding the shared TokenizerInfo on load. Fits ELAI's grammar-per-tool-catalog reuse and its caching posture. Note the versioned format invalidates on XGrammar upgrade.

NOT PORTABLE / caveats:
  - The CUDA kernel (apply_token_bitmask_inplace_cuda.cu) needs a GPU; the remote 3090 sidecar has one, but the local Anthropic-protocol path streams (runtime_client.rs) — mask application must happen inside the SIDECAR's torch decode loop, NOT in the Rust orchestrator. The Rust side cannot apply masks; this is a Python-sidecar feature only. (Community Rust bindings xgrammar-rs exist but only wrap the same C++; they still operate on logits, which the orchestrator does not hold.)
  - Constrained decoding only shapes SAMPLING; it cannot make a weak model's underlying distribution correct. Docs explicitly recommend ALSO describing the structure in the prompt. So this complements, does not replace, good tool-call prompting.
  - BatchGrammarMatcher / draft-tree speculative decoding assume an engine that owns batched logits + a draft tree; ELAI's sidecar would need that machinery first.
  - It does NOT verify SEMANTIC correctness (right tool for the task, sane arg VALUES) — only structural/schema validity. ELAI's wiring/grounding gates remain orthogonal and still needed.


5. BENCHMARK NUMBERS

This repo's prose makes only QUALITATIVE claims (no figures in README/docs):
  - README.md:36 "near-zero overhead in JSON generation ... one of the fastest structured generation engines available" and "100% structural correctness".
  - docs/index.rst:5 mission: "flexible zero-overhead structure generation everywhere".
The quantitative headline numbers (per-token mask overhead in microseconds, the order-of-magnitude mask-generation speedup, end-to-end serving throughput vs unconstrained) live in the technical report arXiv:2411.15100 (Dong et al., "XGrammar", MLSys 7, 2024) and the XGrammar-2 paper (Li et al., ACM AI & Agentic Systems 2026, doi 10.1145/3786335.3813124) — both cited in README.md:107-128 — NOT in the repo text. Adoption signal as a proxy (README News): default backend in vLLM, SGLang, TensorRT-LLM, MLC-LLM; integrated into Modular MAX, OpenVINO GenAI, Mirai. No fabricated figures: any specific number must come from those two papers, which are outside this repo.


6. DEPENDENCIES / STACK

Languages: C++ (engine, ~16k LOC core), CUDA (.cu mask kernel), Python (bindings + kernels + structural-tag builders), with JS (web/) and Swift (Package.swift) APIs also shipped.
Build: scikit-build-core + CMake (RelWithDebInfo), wheels via cibuildwheel for Linux x86_64/aarch64, macOS x86_64/arm64, Windows AMD64. Python >=3.8,<4. pyproject version 0.2.2.
Python runtime deps: apache-tvm-ffi>=0.1.9 (the C++<->Python FFI bridge), pydantic (tool schemas), torch>=1.10.0, transformers>=4.38.0, numpy, typing-extensions; triton (Linux x86_64 only) for the Triton kernel. Optional: sentencepiece + tiktoken (extra tokenizer backends), mlx-lm (Apple Silicon "metal" extra).
Vendored 3rdparty: dlpack (tensor ABI for the bitmask/logits DLTensor interface), picojson (JSON serialize), cpptrace (stack traces), googletest (tests).
Platforms/HW: Linux/macOS/Windows; CPU, NVIDIA GPU, AMD GPU, Apple Silicon, TPU. Mask-apply kernel backends: cuda / triton / torch / torch.compile / cpu / mlx, selected by device.
License: Apache-2.0.
