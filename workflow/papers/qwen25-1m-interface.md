Algorithm: Qwen2.5-1M Context Extension (YaRN + Dual Chunk Attention + Sparse Prefill)
ArXiv: 2501.15383
Paper: Qwen2.5-1M Technical Report (Qwen Team, Alibaba Group, Jan 2026)

Inputs:
model_id | str | scalar | huggingface.co/Qwen/Qwen2.5-7B-Instruct-1M or Qwen2.5-14B-Instruct-1M
rope_scaling | dict | optional | {"type":"yarn","factor":<f>,"original_max_position_embeddings":32768} for base Qwen2.5-Coder-7B
  -- note: the 1M variant ships with rope_theta=1e7 and max_position_embeddings=1010000 (no rope_scaling needed)
  -- for base Coder-7B (rope_theta=1e6, max_position_embeddings=32768): YaRN factor must be applied via AutoConfig
tokens | int[] | (T,) | T up to 1M for 1M variant; T up to 32K for base Coder-7B without YaRN
chunk_size | int | 262144 | Dual Chunk Attention chunk boundary; local_size=8192 for intra-chunk local attention

Outputs:
logits | tensor | (T, vocab_size) float | next-token distribution
Invariant: quality validated through 256K tokens in training; 1M is extrapolated (not training-validated)
Invariant: Qwen2.5-1M uses direct position scaling (no rope_scaling dict) via rope_theta=1e7; rope_scaling=null in config.json

Hyperparameters:
yarn_factor | null (off) | {null, 2.0, 4.0, 8.0} | HIGH sensitivity: factor=4 for 128K target on base Coder-7B (32768*4=131072); MUST validate with eval before enabling default-on (E-28 Open Question 3)
original_max_position_embeddings | 32768 | fixed | base Qwen2.5-Coder-7B max_position_embeddings from config.json; assert this in code
rope_theta (base Coder-7B) | 1000000.0 (1e6) | fixed | verified from HF config.json Qwen/Qwen2.5-Coder-7B-Instruct
max_position_embeddings (base Coder-7B) | 32768 | fixed | verified from HF config.json
num_attention_heads | 28 | fixed | GQA: 28 Q heads, 4 KV heads
num_key_value_heads | 4 | fixed | verified from HF config.json; head_dim=128 (3584/28)
chunked_prefill_chunk_tokens | 32768 | tunable | reduces activation VRAM by 96.7% at 1M context; recommended max_num_batched_tokens=131072

Critical invariants (testable assertions):
assert config.max_position_embeddings >= 32768  # guard against silent mis-config (E-28 Task 3 explicit requirement)
assert config.rope_theta == 1e6  # base Coder-7B; 1M variant uses 1e7
assert config.num_key_value_heads == 4  # GQA shape; head_dim=128 power-of-two; FA2 compatible
assert config.head_dim == 128  # config.hidden_size // config.num_attention_heads = 3584 // 28
# YaRN rope_scaling dict shape validation:
if yarn_factor is not None:
    assert rope_scaling["type"] == "yarn"
    assert rope_scaling["original_max_position_embeddings"] == 32768
    assert rope_scaling["factor"] in (2.0, 4.0, 8.0)
# Quality gate: do not enable YaRN default-on until needle-in-haystack + code-edit eval passes human review
assert yarn_enabled == False  # default; override only after Task 9 eval sweep completes

Complexity:
VRAM at 1M (1M variant): 7B = 120GB total; 14B = 320GB total; requires multi-GPU
VRAM at 25K on base Coder-7B + YaRN factor=4: within single RTX 3090 24GB envelope (measured 1.92GB for 7B NF4 at 25K)
Prefill speedup at 1M: 3x-7x vs naive (chunked prefill + sparse attention)
Chunked prefill activation VRAM reduction: 96.7% at chunk_size=32768

Failure modes:
Quality degradation at T >> training window: validated to 256K; 1M is extrapolated; quality at 512K-1M unverified without needle eval
YaRN factor too large without eval: factor=8 at base 32K model = 256K context; quality unverified; MUST run eval sweep first
rope_scaling applied to wrong model variant: the 1M variant already has rope_theta=1e7 and does not need rope_scaling dict; applying YaRN on top of 1M variant is untested
Silent mis-config from wrong original_max_position_embeddings: if 131072 passed instead of 32768, position encoding breaks silently; always assert 32768
Multi-GPU requirement at 1M: single RTX 3090 cannot hold 1M context at 7B; E-28 scope is 25K-128K on single 3090

Implementation pitfalls (do NOT do this):
Do NOT apply rope_scaling dict to Qwen2.5-7B-Instruct-1M model -- it already uses rope_theta=1e7 and max_pos=1010000
Do NOT set original_max_position_embeddings != 32768 for base Coder-7B YaRN application
Do NOT enable YaRN default-on without running Task 9 needle-in-haystack eval sweep first
Do NOT assume factor=4 is safe without per-position-bucket quality validation (E-28 Open Question 3)
Do NOT use rope_scaling with type != "yarn" -- type "dynamic" is a different algorithm with different guarantees
Do NOT target 1M context on single RTX 3090 -- minimum VRAM is 120GB for 7B model at 1M tokens
