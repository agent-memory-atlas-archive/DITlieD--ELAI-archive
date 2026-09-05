ProgramAsWeights (PAW) - repo + paper analysis
Repo: github.com/programasweights/programasweights-python @ 0.4.2 (shallow-cloned 2026-07-04)
Paper: arXiv:2607.02512 "Program-as-Weights: A Programming Paradigm for Fuzzy Functions"
  (Zhang, Hotsko, Kim, Nie, Shieber, Deng; Waterloo/Cornell/Harvard). No arxiv link in repo; found via WebSearch.
Authors = da03 / yuntian-deng (Yuntian Deng, implicit-CoT line of work).

PURPOSE
Compile a natural-language spec ("fix malformed JSON", "classify urgency") into a tiny frozen-base
neural function that runs locally, deterministically, offline after first download, with no runtime API
key. Target niche: "fuzzy" text tasks between regex (too rigid) and a full LLM call (too heavy/costly/
non-local) - classification, extraction, format-repair, triage, intent-routing. The unit of reuse is a
~5-23 MB program file plus a shared small base model; inference is ~one-fiftieth the memory of prompting
a 32B model. The pip SDK is a THIN CLIENT: real compilation (the training) happens server-side on the
hosted PAW API; the client resolves/downloads/caches and runs local inference.

CORE MECHANISM
Two representations of a "neural program" coexist in the repo; understand both.
1. Paper/continuous form (compiler/__init__.py, the local training-time path):
   Phase 1 "generate pseudo-program": a trained compiler model (Qwen3-4B; paper adds a frozen off-the-
     shelf Qwen3-4B "pseudo-compiler" that first restates spec + synthesizes I/O examples) greedily
     decodes a pseudo-program text from the spec. compiler/__init__.py:118-165 (AutoModelForCausalLM.
     generate, do_sample=False). Prompt format compiler_prompt() lines 30-40: "[SPEC]..[END_SPEC]
     [PSEUDO_PROGRAM]".
   Phase 2 "extract prefix KV / LoRA": 64 special <prefix_i> tokens are appended (lines 172-181), the
     compiler's hidden states are pulled from uniformly-spaced layers (get_prefix_hidden_states, imported
     from an EXTERNAL research script not in the repo), then an MLPProgramPrefixMapper (imported from
     `utils`, also NOT in the repo) maps those hidden states into either an interpreter KV-cache prefix
     (compiler/__init__.py:207-238) or, per the paper, LoRA mixing-coefficients over N=64 shared learnable
     rank-r=64 bases. Interpreter is a FROZEN Qwen3-0.6B. Program = hybrid: discrete pseudo-program text +
     continuous LoRA/KV weights. Only the LoRA-compiler is trained (SFT); loss = neg mean-token
     log-likelihood of target output under the frozen interpreter. Pseudo-compiler + interpreter stay frozen.
2. Shipped local runtime form (runtime_llamacpp.py, the actually-installed inference path):
   The downloaded bundle is a dir with adapter.gguf (Q4_0 LoRA), prompt_template.txt (pseudo-program +
   {INPUT_PLACEHOLDER}), meta.json. PawFunction.__init__ (runtime_llamacpp.py:34-140): load base GGUF via
   llama_cpp.Llama, apply LoRA via llama_adapter_lora_init + llama_set_adapters_lora (lines 118-125,
   _apply_adapter 142-151), split template on {INPUT_PLACEHOLDER} into prefix/suffix, tokenize prefix.
   KEY PERF TRICK: prefix KV cache is evaluated once and persisted to prefix_kv_cache.bin via
   llama_state_seq_save_file, reloaded on later runs via llama_state_seq_load_file (_load_or_eval_prefix,
   lines 153-196) - eliminates ~2-3s cold-start prefix eval. __call__ (198-247): reset n_tokens to prefix
   len, eval input+suffix tokens, greedy sample loop to EOS. Deterministic (temperature=0 default).
There is ALSO a pure-torch interpreter (runtime/interpreter.py, 485 lines) that consumes the continuous
KV-cache program directly (load_paw_program -> attach kv_layers as past_key_values), and an ONNX variant
(interpreter_onnx.py, 534 lines) - these are the research/alt runtimes, not the default llama.cpp path.

KEY INTERFACES
Public API (programasweights/__init__.py):
  compile(spec, compiler=None, name, tags, public=True, slug=None, ephemeral=False) -> Program
    (server-side compile via PAWClient; Program has id, slug, status, version, timings).
  function(program_id, n_ctx=2048, n_gpu_layers=None, verbose=False, offline=False) -> PawFunction
    (resolves slug->id, downloads .paw bundle if uncached, returns local callable). __init__.py:113-236.
  compile_and_load(spec, ...) -> PawFunction   (compile + function convenience).
  list_versions / list_programs / list_compilers / login.
Runtime: PawFunction(program_dir, n_ctx, n_gpu_layers, verbose, ...).__call__(input_text, max_tokens=None,
  temperature=0.0) -> str. Stateless: one text in, one text out, ~2048-token shared window.
Server client (client.py): compile()->POST /api/v1/compile; resolve_slug()->GET /api/v1/programs/resolve/
  {slug}; download_paw()->GET /api/v1/programs/{id}/download, handles 202 Retry-After polling (still
  generating) + 302 HF-CDN redirect (client.py:52-160).
Format (paw_format.py): .paw v2 binary = magic b"PAW\x02" + uint32 version + uint32 metadata-len + JSON
  metadata + safetensors tensor blob. save_paw_program()/load_paw_program()/load_paw_lora()/
  validate_paw_file(). Holds KV layers (layer_i_key/value), optional LoRA tensors (lora_*), pseudo-program,
  generation_config, hub metadata. validate_paw_file (paw_format.py:255-330) enforces 500MB cap,
  prefix_steps<=256, lora_rank<=128, and greps metadata for __import__/eval(/exec(/os.system/subprocess.
Data flow: spec -> (server) compiler+mapper -> .paw(LoRA/KV+pseudo-program) -> download+cache -> llama.cpp
  base+LoRA+cached-prefix-KV -> greedy decode.
Interop: convert_peft_to_paw.py from_peft() wraps any HF PEFT/LoRA adapter dir into a .paw (source="peft",
  forkable=False) - a clean bridge from standard PEFT training to the PAW distribution format.

EVIDENCE
Paper (arXiv:2607.02512) numbers:
  FuzzyBench exact-match: PAW on frozen Qwen3-0.6B = 73.78% vs Qwen3-32B direct prompting = 68.70%
    (small compiled program BEATS 32B prompting on the in-distribution benchmark).
  Program size ~23 MB (Q4_0 LoRA) + shared base ~484 MB (Q4_K_M). Latency 31.6 tok/s on M3 Metal.
  Inference memory ~1.2 GB vs ~60 GB for Qwen3-32B (~50x).
  Cross-dataset (PAW vs Qwen3-32B): YouTube 90.40 vs 93.60; SMS-F1 80.77 vs 89.04; Yelp 95.82 vs 98.11;
    IMDB 90.64 vs 94.64 (in-domain PAW wins; OOD transfer trails the big model by 3-8 pts).
  Training data FuzzyBench-10M: gpt-5.2-generated, 29 thematic versions, 800+ task categories, 8 I/O pairs
    per spec, 80/10/10 split by spec; verified test needs gpt-5-mini AND gpt-5.2 agreement. Only LoRA
    compiler trained (SFT); interpreter + pseudo-compiler frozen.
Repo does NOT contain: the eval harness, FuzzyBench data, the compiler/mapper training code, or the
  trained checkpoints. compiler/__init__.py imports `utils.MLPProgramPrefixMapper` and
  `main_no_spec_..._vllm.get_prefix_hidden_states` which are ABSENT (research repo, not shipped). The
  compact demo compiler/dummy.py is a placeholder. So the repo is a client SDK + file format + local
  runtimes; the science (training, benchmarking) lives elsewhere and is only reproducible via the paper.
Repo tests: tests/test_sdk.py (526 lines), test_offline_cache.py (97), test_runtime_registry_sdk.py (81) -
  SDK/cache/registry behavior, not model quality.

DEPENDENCIES + LICENSE
LICENSE: MIT (Copyright 2026 ProgramAsWeights) - verified, clean, permissive, no attribution burden beyond
  notice. pyproject license = MIT.
Runtime deps (pyproject): httpx>=0.27,<1.0 (server client), llama-cpp-python>=0.3.0,<=0.3.19 (local
  inference - HEAVY: native llama.cpp build, GGUF, Metal/CUDA). Test extra: pytest.
Implicit heavy deps used by the compiler/torch-interpreter paths but NOT declared in pyproject: torch,
  transformers, safetensors (compiler/__init__.py, paw_format.py, runtime/interpreter.py), PIL optional.
  These matter only if you run the local training/torch-interpreter path; the pip-installed llama.cpp path
  avoids torch entirely. Weight verdict: SDK core is light (httpx + llama-cpp); the research paths pull the
  full torch/transformers stack.

MATURITY
Dev status "3 - Alpha" (pyproject). Version 0.4.2, first public release 0.2.0 (2026-04). CHANGELOG shows
  active iteration Mar-Apr 2026 (download-race fixes, LoRA-switch KV-clear bug, py3.9 compat). Code quality
  is good: typed signatures, docstrings, offline/cache handling, stderr suppression, dual-API llama_cpp
  compat shims (_apply_adapter handles old+new adapter API), format validation with a code-injection grep.
  Tests present (704 LOC) but cover SDK plumbing not model behavior. Smells: compiler/__init__.py depends
  on out-of-repo modules + hardcoded checkpoint dir paths (train_runs_stage2/compiler) = not runnable
  standalone; sys.path hacking (lines 15-20). Small (3.4K LOC pkg). Backed by a live commercial SaaS
  (hosted compile API, HF asset CDN, billing/rate-limits) - so the local runtime is durable but compile
  depends on their server staying up.

PORTABILITY (to a Rust FSM-gated multi-agent coding runtime w/ local Qwen2.5-Coder sidecar)
Note: PAW's DOMAIN (fuzzy single-shot text classify/extract/repair) is narrower than an agent runtime, so
  the steal is at the MECHANISM level, not "adopt PAW wholesale". 4 concrete stealable ideas:
1. Persisted prefix-KV cache to kill cold-start on a fixed prompt prefix. runtime_llamacpp.py:153-196
   (_load_or_eval_prefix via llama_state_seq_save_file / _load_file). ELAI's sidecar already has a
   PrefixKVCache (server.py:1819) that is BYPASSED on the stream:true path (per memory H-50 finding). PAW's
   pattern - eval the stable system/agent-preamble prefix ONCE, serialize the seq state to disk keyed by
   prefix-token hash, reload per call - is the exact fix, and it is llama.cpp-native (the sidecar is Qwen
   NF4/llama-family). This is the single highest-value, lowest-risk port. Reference: the save/load ctypes
   dance at runtime_llamacpp.py lines 157-196.
2. Per-task compiled LoRA adapters for recurring narrow sub-tasks (spec->deterministic tiny function). An
   agent runtime repeatedly does the SAME fuzzy micro-decisions (classify a tool result, route intent,
   repair a JSON tool-arg, triage a build-log line). Instead of a full sidecar generate each time, compile
   a stable, deterministic, ~5-23MB adapter once and swap it in via llama_adapter_lora_init +
   llama_set_adapters_lora (runtime_llamacpp.py:118-151). Determinism (temp=0) + off-agent reproducibility
   aligns with ELAI's grounded/verifiable posture. Reference impl: PawFunction + convert_peft_to_paw.py.
3. The .paw single-file program format (paw_format.py): magic+versioned-JSON-metadata+safetensors, with a
   validate step that caps size, bounds rank/prefix, and GREPS metadata for code-injection tokens
   (paw_format.py:305-322). A clean template for ELAI to package a "compiled skill/adapter" as one
   distributable, cache-keyable, hub-shareable artifact with provenance (source=compiled|peft|custom,
   forkable flag). Portable to Rust as a struct + safetensors reader.
4. Hybrid discrete+continuous "program" = pseudo-program TEXT prepended to a LoRA. Portable idea for ELAI's
   self-improvement loop: represent a learned behavior as (a human-readable/auditable pseudo-program the
   auditor can read) + (a weight delta), not weights alone. The text half stays inspectable/greppable/
   diffable, satisfying wiring+audit doctrine; the weight half carries the fuzzy capability. Reference:
   save_paw_program(pseudo_program=..., lora_weights=...) paw_format.py:110-250.
Bonus (5): PEFT->distribution-format bridge (convert_peft_to_paw.from_peft) - if ELAI ever fine-tunes
   task adapters, wrapping HF PEFT output into a validated portable artifact is a 1-file pattern to copy.
DO NOT port: the server-side compiler/mapper training (out-of-repo, needs FuzzyBench-10M + 4B compiler
   training). The paradigm's compile step is a hosted paid service, not a local capability.

UNKNOWNS
- MLPProgramPrefixMapper internals + get_prefix_hidden_states (the actual mapper architecture) are NOT in
  the repo; only the paper's prose description (mean-pool -> shallow MLP -> mixing coeffs over 64 rank-64
  bases). Cannot verify from code.
- Whether shipped programs use the continuous-KV form or the LoRA form by default: runtime_llamacpp.py
  (the pip default) is LoRA-adapter-based; the KV-prefix path is the torch/research runtime. Server may
  emit either; not verifiable from client code.
- llama.cpp LoRA-hotswap latency/quality on a Qwen2.5-Coder base specifically (paper uses Qwen3-0.6B
  interpreter) - untested here.
- Exact FuzzyBench license/availability and reproduction cost - paper says gpt-5.2-generated; not in repo.
- Paper numbers taken from arXiv HTML abstract/results via WebFetch, not independently reproduced.
