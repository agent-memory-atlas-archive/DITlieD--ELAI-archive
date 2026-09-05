Algorithm: SGLang (RadixAttention + Compressed FSM Constrained Decoding)
ArXiv: 2312.07104
Paper: SGLang: Efficient Execution of Structured Language Model Programs
Authors: Zheng et al. (Stanford / UC Berkeley), NeurIPS 2024
Foundational FSM reference: Willard & Louf 2023, arXiv:2307.09702 (Outlines library)
No separate 2402.* paper for compressed FSM -- blog post at lmsys.org/blog/2024-02-05-compressed-fsm/ is the supplemental reference.
Grammar backend in current SGLang: XGrammar (default), Outlines, Llguidance. Only one active at a time.

--- RADIXATTENTION ---

Inputs (RadixAttention):
token_sequence | list[int] | shape: [seq_len] | mandatory | the full prompt token IDs for the new request
image_hash | bytes/int | optional | SHA hash of input image(s) for VLM; used as tree key for image token prefix sharing
prefix_hint | list[int] | optional | sent by frontend interpreter on fork() primitive to pre-insert prefix into radix tree before remaining tokens arrive

Outputs (RadixAttention):
matched_prefix_len | int | >= 0 | number of prompt tokens reused from cache; 0 means cold start
kv_cache_tensors | tensor[num_layers, 2, seq_len, num_heads, head_dim] | paged, non-contiguous layout; one page per token; shared via reference counting

Data structure:
radix_tree | CPU-resident trie of token sequences | each node holds: token_seq_label, kv_cache_page_pointers, reference_count (int >= 0), last_used_timestamp
eviction_policy | LRU, leaf-first | a node is evictable only if reference_count == 0
reference_count invariant | reference_count incremented when request starts using node, decremented on completion; node never evicted while reference_count > 0

Hyperparameters (RadixAttention):
cache_size | dynamic | shares same memory pool with running requests; no fixed reservation | high sensitivity: when waiting queue is large enough, all cached tokens evicted in favor of larger batch
scheduling_order | longest-shared-prefix-first | equivalent to DFS on radix tree (Theorem 3.1) | high impact: cache hit rate drops significantly with FCFS or random scheduling
frontend_hint | enabled by default | fork() sends prefix hint before remaining tokens | disabling degrades cache hit rate; required for tree-structured workloads

Critical invariants (testable assertions):
assert node.reference_count >= 0, "reference count must never go negative"
assert matched_prefix_len <= len(token_sequence), "cannot match more tokens than the full sequence"
assert cache_hit_rate == (cached_prompt_tokens / total_prompt_tokens), "Sec 3.p6 definition"
# Theorem 3.1: DFS order achieves optimal cache hit rate when cache_size >= max_request_length
assert sorted(requests, key=lambda r: -r.matched_prefix_len) == dfs_order(requests), "longest-prefix-first == DFS"

VLM multi-image prefix sharing (Section 6.SS2.p5):
- Hash each input image (bytes hash or SHA) to produce a stable key
- Insert image token sequence into radix tree under that hash key
- Requests with identical images hit the cached image KV regardless of surrounding text
- Constraint: image tokens must form a contiguous prefix segment or be separably hashable
- Multiple images in one request: each image can be independently cached if their token positions are consistent

Complexity (RadixAttention):
Time per request: O(matched_prefix_len) for tree traversal; O(seq_len) worst case cold start
Space: O(total_cached_tokens * num_layers * 2 * num_heads * head_dim) in paged GPU memory
Tree maintenance overhead: 0.2s for 100 requests = <0.3% overhead (Section 6.SS3.p3)
Distributed tensor parallelism: each GPU holds sharded KV; tree operations identical, no extra sync needed
Data parallelism: router maintains meta-tree; workers maintain local sub-trees; weakly consistent

Failure modes (RadixAttention):
Cache thrashing: frequent switching between unrelated requests collapses hit rate to near 0; use cache-aware scheduling
Starvation: longest-prefix-first can indefinitely delay short-prefix requests; no fairness fix in paper (cited as future work)
Reference count leak: if request completion callback fails to decrement, nodes become non-evictable, filling GPU memory
Image hash collision: astronomically unlikely but two different images with same hash share KV incorrectly -- use strong hash (SHA256 or better)
Variable output length: unpredictable generation length causes KV recomputation because prefix boundary not known in advance (footnote 1)

Implementation pitfalls (do NOT do this):
Do not discard KV cache after each request completes -- this is the vLLM v0.2.5 behavior that SGLang explicitly avoids
Do not use FCFS scheduling -- throughput degradation is severe on shared-prefix workloads
Do not preallocate a fixed-size cache pool -- share the same memory pool with running requests dynamically
Do not skip frontend fork() hints -- runtime cannot infer prefix structure from requests alone without frontend co-design
Do not evict nodes with reference_count > 0 -- causes use-after-free of KV tensors in running requests
Do not use non-paged contiguous KV layout -- prevents sharing across requests without copying

--- COMPRESSED FSM CONSTRAINED DECODING ---

Inputs (Compressed FSM):
constraint_spec | one of: regex (str), json_schema (dict or Pydantic BaseModel), ebnf (str) | mandatory | only one may be set per request
input_tokens_so_far | list[int] | the tokens decoded so far in the constrained generation | used to determine current FSM state

Outputs (Compressed FSM):
allowed_token_mask | bool tensor, shape [vocab_size] | tokens with False are zeroed before sampling
jump_forward_tokens | list[int] | optional | token sequence for the entire compressed edge; all decoded in single forward pass when on singular path

API parameters (gen() primitive, paper Section 2.p3):
gen(variable_name: str, regex: str = None, stop: str = None, max_tokens: int = None) -> str
variable_name | str | mandatory | name to store result in s[variable_name]
regex | str | optional | regular expression constraint; compiled to FSM at first use, cached for batch reuse
stop | str | optional | stop condition string; ignored during compressed edge traversal (Section 5)

API parameters (OpenAI-compatible server endpoint):
response_format={"type": "json_schema", "json_schema": {"name": str, "schema": dict}} | for JSON schema
extra_body={"regex": str} or extra_body={"ebnf": str} | for regex or EBNF
sampling_params={"json_schema": json.dumps(dict), "regex": str, "ebnf": str} | for native /generate endpoint
Constraint: only one of json_schema, regex, ebnf may be set per request

Grammar backends and capabilities:
XGrammar (default, recommended): json_schema + regex + ebnf; best performance; select with --grammar-backend xgrammar
Outlines: json_schema + regex only; no ebnf; select with --grammar-backend outlines
Llguidance: json_schema + regex + ebnf; select with --grammar-backend llguidance

Compressed FSM algorithm (Appendix B, Section 4):
Step 1: Convert constraint to FSM (regex -> DFA via standard Thompson/subset construction; JSON schema -> regex -> DFA via Outlines/XGrammar)
Step 2: Identify singular transition edges: edge is singular iff (source_node.out_degree == 1) AND (exactly one valid character/string on edge)
Step 3: Compress: merge consecutive singular edges into single compressed edge; label = concatenation of individual edge labels
Step 4: At decode time: if current FSM state is on a compressed edge, emit all tokens for that edge in one forward pass (prefill/extend the string)
Step 5: Retokenize: after jump-forward, retokenize entire text (previous + compressed string) using original tokenizer to realign token boundaries; overhead ~4%
Step 6: At branching points (non-singular edges): decode one token at a time, masking disallowed tokens

Hyperparameters (Compressed FSM):
fsm_cache | per-constraint-string, shared across batch | mandatory to precompute once | recomputing per request causes 2.4x throughput loss (Section 6.SS3.p4)
jump_forward | default enabled | multi-token single-pass on compressed edges | provides 1.6x throughput improvement over token-by-token FSM (Section 6.SS3.p4)

Critical invariants (testable assertions):
assert len([c for c in [json_schema, regex, ebnf] if c is not None]) <= 1, "only one constraint type per request"
assert all(token in allowed_tokens_at_state(fsm_state) for token in jump_forward_tokens), "all jump-forward tokens must be valid per FSM"
assert fsm_compiled[constraint_key] is fsm_compiled[same_constraint_key], "FSM object must be cached and reused, not recomputed per request"
# After retokenization, full token sequence must decode to same text as before retokenization:
assert tokenizer.decode(retokenized_ids) == original_text + compressed_edge_string

Complexity (Compressed FSM):
FSM compilation: O(|regex| * |alphabet|) one-time cost; must be amortized across batch
Per-token decode (non-compressed): O(vocab_size) for mask application
Per-compressed-edge decode: O(|edge_string| / avg_tokens_per_string) forward passes instead of O(|edge_string|)
Retokenization: O(total_token_count) per jump-forward; ~4% overhead per compressed edge traversal
Space: O(|states| * |vocab_size|) for FSM transition table; can be large for complex schemas

Failure modes (Compressed FSM):
Distorted probability (Appendix B.3): compressed edges with alternatives cause model to assign inaccurate probabilities at branching points; workaround: include choices directly in prompt text
Tokenization misalignment: naive string split without retokenization produces subtly wrong token sequences; retokenization step is non-optional
Complex schema compilation timeout: very large JSON schemas can produce massive FSMs; no timeout in paper; implementation must guard
Grammar backend mismatch: EBNF not supported by Outlines backend; silently fails or errors if wrong backend selected

Implementation pitfalls (do NOT do this):
Do not recompute the FSM per request -- must cache compiled FSM by constraint string as key; 2.4x throughput penalty if not cached
Do not use naive token-by-token FSM decode when on a singular compressed edge -- defeats the jump-forward optimization
Do not skip retokenization after jump-forward -- tokenization boundary misalignment corrupts subsequent generation
Do not allow more than one constraint parameter (json_schema/regex/ebnf) per request -- behavior is undefined
Do not run compressed edge prefill through the sampling path -- it is a deterministic extend/prefill, not a sample

--- PERFORMANCE NUMBERS (verified from paper/blog) ---

Throughput vs baselines (Llama-7B, A10G 24GB, Section 6):
Up to 6.4x throughput improvement over vLLM, Guidance, LMQL across all benchmarks
Up to 3.7x latency reduction
JSON decoding specifically: compressed FSM gives 1.6x throughput gain (Section 6.SS3.p4)
Multi-modal (LLaVA-v1.5-7B image): 0.18 -> 1.15 image/s = 6.4x (Table 2)
Multi-modal (LLaVA-NeXT-34B video): 0.02 -> 0.10 frame/s = 5x (Table 2)
Production deployment (Chatbot Arena): 52.4% cache hit rate on LLaVA-Next-34B, 74.1% on Vicuna-33B; 1.7x TTFT reduction
Cache hit rates in benchmarks: 50% to 99%; cache-aware scheduling achieves 96% of optimal on average
RadixAttention tree management overhead: <0.3% (Section 6.SS3.p3)
Compressed FSM blog: up to 2x latency reduction, up to 2.5x throughput vs guidance+llama.cpp and outlines+vLLM

--- PHASE 4I SPECIFICS (InternVL3.5-38B Mode B) ---

RadixAttention for multi-image KV cache:
Each image must be hashed to produce a stable key before adding to radix tree
The image token block in the prompt must be positionally stable (same token positions across requests using same image)
System prompt + image tokens form the shared prefix; the question/instruction text is the per-request suffix
With multiple images: each image independently hashable; all image KV blocks can be cached simultaneously
Batch requests about the same image(s) together to maximize cache hit rate; use longest-prefix-first scheduling

Compressed FSM for VlmValidationOutput JSON schema:
Convert VlmValidationOutput Pydantic model to JSON schema via .model_json_schema()
Pass as json_schema parameter to OpenAI-compatible API or sampling_params for native API
Use XGrammar backend (default) for best performance; fallback to Outlines if XGrammar unavailable
The FSM is compiled once per unique schema; cache it -- do not recompile per inference call
Constant fields in schema (e.g., required key names like "summary":) become compressed edges = decoded in single forward pass
Variable fields (e.g., string values) are branching = decoded token-by-token with mask

Constraint: JSON schema must be expressible as a regular language; recursive/self-referential schemas are not supported by FSM-based backends
Pydantic Optional fields and Union types expand regex considerably; test compilation time before deployment
