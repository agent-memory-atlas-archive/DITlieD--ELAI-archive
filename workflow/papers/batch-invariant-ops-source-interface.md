Source: batch_invariant_ops (github.com/thinking-machines-lab/batch_invariant_ops)
Reliability: MEDIUM (reviewed source, shallow clone, MIT license)
Version: 0.1.0
Files examined:
  F:\.ELAI_workflow\docs\.source\batch_invariant_ops\batch_invariant_ops\batch_invariant_ops.py
  F:\.ELAI_workflow\docs\.source\batch_invariant_ops\batch_invariant_ops\__init__.py
  F:\.ELAI_workflow\docs\.source\batch_invariant_ops\test_batch_invariance.py
  F:\.ELAI_workflow\docs\.source\batch_invariant_ops\deterministic_vllm_inference.py
  F:\.ELAI_workflow\docs\.source\batch_invariant_ops\pyproject.toml
  F:\.ELAI_workflow\docs\.source\batch_invariant_ops\README.md
RAG ingest: source-batch-invariant-ops project, 7 chunks (reranker live at ingest time)

---
ELAI CONTEXT NOTE
This map was extracted specifically for ELAI's HF-transformers inference sidecar on a single
RTX 3090 (Ampere sm_86, bfloat16 NF4 via bitsandbytes, model.generate + LogitsProcessor,
SDPA or FA2 attention). The steal hypothesis: wrap model.generate in set_batch_invariant_mode()
to get batch-invariant mm/addmm/log_softmax/mean. This map VERIFIES and CONSTRAINS that hypothesis.

---
PUBLIC API

set_batch_invariant_mode(enabled: bool = True) -> contextlib.AbstractContextManager
  Context manager. On __enter__ calls enable_batch_invariant_mode() if enabled=True, else
  disable_batch_invariant_mode(). On __exit__ destroys the Library object and restores prior
  state (prior _batch_invariant_MODE + _batch_invariant_LIB tuple). Thread-safety: NOT guaranteed
  (single global _batch_invariant_LIB). Logs Triton JIT compilation on first use (README notes
  "Just to get the logging out of the way").

enable_batch_invariant_mode() -> None
  Idempotent (early-return if already enabled). Registers four aten dispatch overrides using
  torch.library.Library("aten", "IMPL") at the dispatch_key derived from
  torch.accelerator.current_accelerator().type.upper() (e.g. "CUDA" on CUDA devices).
  After this call torch.mm, torch.addmm, torch._log_softmax, torch.mean.dim all route to
  Triton kernels for the current accelerator. GLOBAL MUTATION: modifies PyTorch's dispatch table.

disable_batch_invariant_mode() -> None
  Calls _batch_invariant_LIB._destroy() (removes the Library from the dispatch table),
  resets globals to False/None.

is_batch_invariant_mode_enabled() -> bool
  Returns the global _batch_invariant_MODE flag. Pure read.

matmul_persistent(a: torch.Tensor, b: torch.Tensor, bias: torch.Tensor | None = None) -> torch.Tensor
  Direct call to the persistent Triton matmul kernel. Asserts a.shape[1]==b.shape[0] and
  a.dtype==b.dtype. bias must be 1D if provided. Returns tensor of same dtype as inputs.
  Internally detects NUM_SMS via torch.cuda.get_device_properties(0).multi_processor_count
  (CUDA path) or torch.xpu (XPU path) or torch.get_num_threads() (CPU fallback).

log_softmax(input: torch.Tensor, dim: int = -1) -> torch.Tensor
  Direct Triton log_softmax. Only supports last dimension (dim==-1 or dim==input.ndim-1),
  raises ValueError otherwise. Reshapes to 2D, processes row-by-row, reshapes back.
  BLOCK_SIZE hardcoded to 1024.

mean_dim(input: torch.Tensor, dim: int, keepdim: bool = False, dtype: torch.dtype | None = None) -> torch.Tensor
  Triton mean over a single dimension. Asserts input.is_cuda. Handles negative dim. For
  integer inputs defaults dtype to float32. Multi-dim mean falls back to torch.sum+divide
  (not a Triton path) -- see GOTCHAS.

get_batch_invariant_attention_block_size() -> AttentionBlockSize
  Returns AttentionBlockSize(block_m=16, block_n=16). This is a RECOMMENDATION only for
  attention block tiling, not a kernel dispatch override. The attention kernel itself is NOT
  in this library (see GOTCHAS).

AttentionBlockSize: namedtuple with fields block_m (int), block_n (int).

---
OPS ACTUALLY REPLACED (via torch.Library dispatch override)

aten::mm            -> mm_batch_invariant     -> matmul_persistent (Triton, persistent kernel)
aten::addmm         -> addmm_batch_invariant  -> matmul_persistent (Triton, bias via HAS_BIAS)
aten::_log_softmax  -> _log_softmax_batch_invariant -> log_softmax (Triton, row-per-block)
aten::mean.dim      -> mean_batch_invariant   -> mean_dim (Triton, single-dim) or torch.sum fallback

NOT REPLACED (no dispatch override exists in this library):
  - aten::bmm (batched matmul -- NOT overridden)
  - aten::scaled_dot_product_attention / flash_attention (SDPA/FA2 -- NOT overridden)
  - aten::linear (NOT overridden -- but bitsandbytes NF4 calls F.linear which decomposes to mm/addmm,
    so mm/addmm IS covered post-dequant)
  - aten::softmax (only _log_softmax is overridden, not softmax)
  - any layer norm, RMS norm, rotary embedding ops

KERNEL IMPLEMENTATION DETAILS

matmul_kernel_persistent (Triton, @triton.jit + launch_metadata):
  Persistent grid: grid size = min(NUM_SMS, ceil(M/BM)*ceil(N/BN)). Each SM processes multiple
  tiles. Accumulates in float32 (tl.float32) then casts to c_ptr.dtype on store.
  Configs per dtype:
    bfloat16: BLOCK_SIZE_M=128, N=128, K=64, GROUP_SIZE_M=8, num_stages=3, num_warps=8
    float16:  BLOCK_SIZE_M=128, N=256, K=64, GROUP_SIZE_M=8, num_stages=3, num_warps=8
    float32:  BLOCK_SIZE_M=128, N=128, K=32, GROUP_SIZE_M=8, num_stages=3, num_warps=8
  Large-tensor guard: offsets cast to int64 when tensor.numel() > 2**31.
  Bias: added to accumulator in float32 before final cast when HAS_BIAS=True.

_log_softmax_kernel (Triton, @triton.jit):
  3-pass: find max_val, compute sum_exp, compute x-max-log_sum_exp.
  One block per row of the flattened 2D view. BLOCK_SIZE fixed at 1024.
  Does NOT accumulate in float32 -- loads in native dtype, computes in native dtype.
  (log_softmax internal precision depends on input dtype -- bfloat16 inputs stay bfloat16.)

mean_kernel (Triton, @triton.jit):
  Views input as (M, N, K), one program per output element. BLOCK_SIZE fixed at 1024.
  Accumulator is float (acc = 0.0, Python float -> tl.float32 in Triton context).
  Single-dim only; multi-dim falls back to torch.sum.

GPU ARCH REQUIREMENT:
  Triton requires CUDA sm_50+ (Triton docs, training knowledge). RTX 3090 = sm_86 (Ampere).
  No sm_86-specific ops are used. bfloat16 Triton tl.dot requires sm_80+ (Ampere+ only --
  bfloat16 tensor cores not available before Ampere). The ELAI target (sm_86) satisfies this.
  XPU (Intel) path exists but is untested by this library (dispatch_key="XPU").
  CPU "fallback" (get_num_threads) would feed a CUDA Triton kernel launch, which would crash --
  the CPU path in get_compute_units is a dead branch when CUDA is present.

---
DTYPE COVERAGE (critical for NF4 bitsandbytes use case)

bfloat16: FULLY SUPPORTED for mm/addmm. matmul_persistent has explicit config for torch.bfloat16.
  Triton tl.dot accumulates in float32 internally (accumulator = tl.zeros(..., tl.float32)) and
  casts back to bfloat16 on store. This is numerically stable.

float16: FULLY SUPPORTED for mm/addmm.

float32: FULLY SUPPORTED for mm/addmm.

log_softmax with bfloat16: SUPPORTED by the kernel dispatch, but the Triton kernel (_log_softmax_kernel)
  does NOT explicitly upcast to float32. The tl.exp and tl.log calls operate in the input dtype.
  bfloat16 log_softmax may have precision loss vs float32 reference. The _half_to_float parameter
  (the PyTorch kernel's half->float promotion path) is asserted False and not implemented:
    assert not _half_to_float, "not implemented"
  If PyTorch's aten dispatcher calls _log_softmax with _half_to_float=True (which it does for
  float16 inputs when the output dtype is float32), this WILL RAISE AN ASSERTION ERROR.
  For bfloat16 inputs, _half_to_float is typically False, so this path is likely safe for ELAI's
  bfloat16 NF4 use case, but is UNVERIFIED.

mean.dim with bfloat16: multi-dim mean (len(dim)>1) has an explicit dtype check:
    assert input.dtype in {torch.float16, torch.bfloat16, torch.float32}
  Single-dim mean goes through mean_kernel where acc is tl.float32 and result cast back to dtype.

bitsandbytes NF4 compatibility: bitsandbytes Linear4bit dequantizes weights and calls F.linear
  on the dequantized tensor. F.linear decomposes to aten::addmm (with bias) or aten::mm (no bias).
  Both are overridden. The dequantized weight dtype is bfloat16 when compute_dtype=bfloat16 (standard
  NF4 config on Ampere). matmul_persistent asserts a.dtype==b.dtype. F.linear passes the activation
  and dequantized weight in the same compute_dtype, so this assertion holds. VERDICT: NF4 linear
  layers ARE covered by the batch-invariant mm/addmm override as long as compute_dtype=bfloat16.

---
VLLM-SPECIFICITY vs FRAMEWORK-AGNOSTIC

The library itself is FRAMEWORK-AGNOSTIC. It uses only:
  torch.library.Library("aten", "IMPL") -- standard PyTorch public API
  triton -- standard Triton package
  No vllm imports, no vllm-specific tensor layout assumptions, no PagedAttention coupling.

deterministic_vllm_inference.py: a SEPARATE demo script, not part of the library. It requires
  PyTorch 2.9.0 or higher (comment at top of file) and a vLLM upstream PR
  (github.com/vllm-project/vllm/pull/24583). This demo is vLLM-specific; the library is not.

For ELAI's HF-transformers use: no vLLM dependency needed. set_batch_invariant_mode() wraps
  model.generate() directly. The dispatch override applies to all aten::mm calls regardless of
  whether they come from vLLM, HF transformers, or any other framework.

Tensor layout: the library does not assume any special tensor layout (no strides constraints
  beyond a.shape[1]==b.shape[0] for mm). Triton kernels use stride arguments from input tensors.
  Contiguous is required for log_softmax input (input_2d.contiguous() called explicitly).

---
BEHAVIORAL CONTRACTS

set_batch_invariant_mode context manager:
  - On enter with enabled=True: calls enable_batch_invariant_mode(). If already enabled,
    enable_ returns early without nesting (NOT a stack). This means nested calls to
    set_batch_invariant_mode(True) inside an already-enabled context are safe but the inner
    context manager restores state on exit, which will DISABLE the mode. See GOTCHAS.
  - On exit: _destroy() + restore old_data. The Library destructor removes the dispatch override.
  - Side effect: Triton JIT compilation on first kernel invocation (one-time, cached).

enable_batch_invariant_mode:
  - Calls torch.library.Library("aten", "IMPL") -- this is a GLOBAL registration.
  - Not reentrant. Second call when already enabled is a no-op.
  - dispatch_key is detected per-call from current_accelerator(). If called in an environment
    without a CUDA device, dispatch_key="CPU" and the Triton kernels would be invoked on CPU
    tensors, which will fail (Triton requires CUDA). No guard against this.

disable_batch_invariant_mode:
  - Calls _batch_invariant_LIB._destroy() which removes the override from PyTorch's dispatch.
  - After destroy, torch.mm etc. revert to native PyTorch implementation.

matmul_persistent:
  - Does NOT check that inputs are on CUDA. Will crash at Triton launch if inputs are on CPU.
  - Does NOT check for contiguous inputs. Triton uses stride arguments so non-contiguous tensors
    are handled (strides are passed), but performance may degrade.
  - Accumulates in float32 regardless of input dtype; result cast back to input dtype.

log_softmax:
  - Calls input_2d.contiguous() explicitly, safe for non-contiguous inputs.
  - Only supports last dimension; ValueError for others.
  - _half_to_float=True asserts False (not implemented).

mean_dim:
  - Asserts input.is_cuda (hard failure on CPU input).
  - Single-dim only for the Triton path; multi-dim uses torch.sum (not batch-invariant).
  - dtype=None preserves input dtype for floats; promotes integer to float32.
  - dtype check in mean_batch_invariant: "assert dtype is None or dtype == torch.float32".
    PyTorch may call mean.dim with dtype=torch.float32 (common for layer norm precompute).
    dtype != float32 and dtype != None will assert.

get_batch_invariant_attention_block_size:
  - Returns hardcoded AttentionBlockSize(16, 16). Pure data, no side effects.
  - README/source: "The attention leg is assumed NOT portable." This function is advisory only;
    no attention kernel dispatch override exists in the library.

---
PLATFORM CONSTRAINTS

Python: requires >=3.8 per pyproject.toml. match/case in get_compute_units requires Python 3.10+
  (structural pattern matching). On Python 3.8/3.9 get_compute_units will raise SyntaxError at
  import time. ELAI sidecar (Ubuntu 24.04, venv) must be Python 3.10+ for safe use.

CUDA: required in practice. XPU path exists but is untested. CPU path will crash at Triton launch.

Triton: required dependency. triton version not pinned (pyproject.toml: "triton"). RTX 3090 sm_86
  is compatible. Triton must match the installed PyTorch version; the PyTorch wheel bundles a
  compatible triton for most standard installs. bfloat16 tl.dot requires sm_80+ (satisfied).

torch.accelerator API: get_compute_units() calls torch.accelerator.current_accelerator() which
  requires PyTorch >= 2.1 (accelerator API introduced 2.1). The vLLM demo requires >= 2.9.0.

Thread safety: the global _batch_invariant_MODE and _batch_invariant_LIB are NOT protected by
  a lock. Concurrent calls to enable/disable from multiple threads are unsafe. model.generate
  with SDPA is typically single-threaded on the Python side, so this is likely not an issue for
  ELAI's use case, but it is unverified.

---
GOTCHAS

1. NESTING BUG: set_batch_invariant_mode is NOT a stack. enable_batch_invariant_mode() returns
   early if already enabled (does not push state). The context manager saves old_data =
   (_batch_invariant_MODE, _batch_invariant_LIB) on enter. If mode was already True on enter
   (outer caller enabled it), on exit the context manager will _destroy() the Library and
   restore old_data = (True, old_lib_ref). If old_lib_ref was already destroyed by the outer
   context manager's exit... this is a use-after-free on the Library object. DO NOT nest
   set_batch_invariant_mode() calls unless you control the outer context.
   SAFE PATTERN for ELAI: enable once around the entire model.generate() call, do not nest.

2. _log_softmax _half_to_float=True not implemented: if PyTorch dispatches _log_softmax with
   _half_to_float=True (common for float16 inputs), the assert fires and the call raises
   AssertionError. For bfloat16 inputs (ELAI's case), PyTorch typically passes _half_to_float=False.
   VERIFY: run a single forward pass before production use and watch for AssertionError.

3. log_softmax precision in bfloat16: the Triton kernel does NOT accumulate in float32.
   exp/log in bfloat16 has ~8 bits mantissa vs float32's 24. For very large vocabulary sizes
   (Qwen2.5-Coder vocab = 151,936) this may introduce numerical differences vs the native
   PyTorch path (which promotes to float32 for _log_softmax). Determinism within a run is
   preserved; absolute accuracy vs float32 reference may degrade.

4. mean.dim dtype assertion: mean_batch_invariant asserts dtype is None or float32. If the
   model calls torch.mean(..., dtype=torch.bfloat16) the override will AssertionError.
   Layer norm in transformers typically uses float32 accumulation; this is likely safe in practice.

5. SDPA/FA2 attention NOT covered: the attention mechanism in ELAI's sidecar (SDPA or FA2)
   is NOT batch-invariant via this library. get_batch_invariant_attention_block_size() provides
   a block size hint but there is no attention kernel override. Batched decoding with SDPA
   may still produce batch-size-dependent results due to flash-attention's tiling.
   IMPLICATION: full token reproducibility across batch sizes requires EITHER using this library
   for non-attention ops AND ensuring greedy decoding with temperature=0 (no sampling variance),
   OR using batch_size=1 always for the attention leg.

6. matmul_persistent dtype mismatch: asserts a.dtype==b.dtype. bitsandbytes NF4 dequant
   produces bfloat16 when compute_dtype=bfloat16. Input activations must also be bfloat16.
   Mixed-dtype inputs (e.g. float32 activations + bfloat16 weights) will assert.

7. Multi-dim mean is NOT batch-invariant: len(dim)>1 falls back to torch.sum+divide, which
   is the standard PyTorch summation (potentially not batch-invariant for large tensors).
   Models using multi-dim mean reduction will not be fully covered.

8. Triton JIT compilation on first use: first invocation of each kernel triggers Triton
   compilation (~seconds). Subsequent calls use the cached PTX. Warm-up call recommended
   before timing or before expecting determinism (README demonstrates this pattern).

9. No float32 intermediate for log_softmax: batch-invariance guarantee for log_softmax is
   achieved by the deterministic row-per-block kernel structure, but the kernel does not
   internally upcast. The guarantee is that SAME input + SAME kernel produces SAME output
   regardless of batch size, not that output matches float32-reference log_softmax.

---
DETERMINISM VERIFICATION (from test_batch_invariance.py and README)

Pattern: compare out = model(input[:1]) vs model(input)[:1] where model uses mm.
Test covers bfloat16 and float32.
Method: (out1 - out2).abs().max() == 0 (bitwise identical, not just close).
Runs 10 iterations to check run-to-run stability.
The test does NOT cover log_softmax or mean in isolation; it tests mm via torch.mm directly.

VERIFICATION FOR ELAI: after pip install -e . and importing set_batch_invariant_mode, run:
  with set_batch_invariant_mode(True):
      run_iters()  # from test_batch_invariance.py
Expected: "Batch Deterministic: True" for both float32 and bfloat16.

---
KNOWN LIMITATIONS (from README and source comments)

- Attention is not covered (no batch-invariant attention kernel in library).
- vLLM determinism requires upstream PR (github.com/vllm-project/vllm/pull/24583) in addition
  to this library; the PR is not merged as of library v0.1.0.
- _half_to_float not implemented in log_softmax override.
- Multi-dim mean not batch-invariant.
- bias.dim()==1 only for addmm; 2D bias will assert.
- Python 3.10+ required in practice (match/case).
- Triton JIT compilation lag on first use.
- Library version 0.1.0, alpha status per pyproject.toml classifiers ("Development Status :: 3 - Alpha").

---
STEAL VERDICT FOR ELAI

CONFIRMED VIABLE with caveats:
  1. set_batch_invariant_mode() wrapping model.generate() is the correct API. VERIFIED from source.
  2. bfloat16 mm/addmm ARE covered and numerically stable (float32 accumulation, cast back to bf16).
  3. bitsandbytes NF4 Linear4bit IS covered post-dequant (F.linear -> addmm, same dtype, asserts hold).
  4. SDPA/FA2 attention IS NOT covered. Full cross-batch-size bitwise reproducibility requires
     temperature=0 greedy decoding AND batch_size=1 for the attention pass, OR accepting that
     attention outputs may still vary.
  5. log_softmax IS covered for bfloat16 with _half_to_float=False, but precision is bfloat16 (not
     float32-promoted). Acceptable for determinism; may differ from float32 reference.
  6. Do NOT nest set_batch_invariant_mode() calls.
  7. Warm up with one forward pass before the actual inference run to trigger Triton JIT compilation.
  8. Requires Python 3.10+ on the sidecar (match/case).

---
Critical assertions (testable):

assert is_batch_invariant_mode_enabled() == False  # initial state
with set_batch_invariant_mode(True):
    assert is_batch_invariant_mode_enabled() == True
assert is_batch_invariant_mode_enabled() == False  # restored on exit

# mm batch invariance for bfloat16 (the core ELAI steal)
import torch
B, D = 128, 512
a = torch.randn(B, D, dtype=torch.bfloat16, device='cuda')
b = torch.randn(D, D, dtype=torch.bfloat16, device='cuda')
with set_batch_invariant_mode(True):
    out1 = torch.mm(a[:1], b)
    out2 = torch.mm(a, b)[:1]
assert torch.equal(out1, out2), "mm not batch-invariant under set_batch_invariant_mode"

# log_softmax NOT dispatched with _half_to_float=True for bfloat16 (guard test)
inp = torch.randn(4, 32000, dtype=torch.bfloat16, device='cuda')
with set_batch_invariant_mode(True):
    out = torch.log_softmax(inp, dim=-1)  # must not AssertionError
assert out.shape == inp.shape

# mean_dim single-dim on bfloat16
inp = torch.randn(4, 512, dtype=torch.bfloat16, device='cuda')
with set_batch_invariant_mode(True):
    out = torch.mean(inp, dim=-1)
assert out.shape == (4,)

# nesting guard -- do NOT nest; verify outer context still active after inner exits
with set_batch_invariant_mode(True):
    with set_batch_invariant_mode(True):  # inner no-op on enter, but destroys on exit
        pass
    # DANGER: mode is now False (inner exit destroyed lib), outer has stale ref
    # assert is_batch_invariant_mode_enabled() == False  # this is the gotcha
