"""E-28 contract tests for the attention-kernels 2026 helpers.

Tests the pure-Python surface added in workflow/reasoning/E-28-decisions.md:

  * refuse_fa3_on_ampere — loud refusal of FA3/FA4/cuDNN-attention imports
    on Ampere (RTX 3090/3060 sm_86/sm_80); no-op on Hopper/Blackwell.
  * pin_sdpa_backend — returns (ctx_mgr, backend_name, pin_active) tuple;
    pins FLASH_ATTENTION when torch>=2.3, nullcontext fallback otherwise.
  * apply_streaming_llm_mask — attention-sink mask per arXiv:2309.17453.
    Critical: position_ids re-indexing is the CALLER's responsibility.
  * apply_yarn_rope_scaling_at_boot — harness-only helper; gated behind
    ELAI_YARN_EVAL_GATE=1; the live boot path still refuses YaRN per
    E-32 refuse_yarn_at_boot.

Paper labels (per memory:feedback_paper_citation_verify):
  * arXiv:2309.17453 Xiao et al. ICLR 2024 — DIRECT GROUNDING for
    StreamingLLM sink-count default (4) per Sec 4.2.
  * arXiv:2501.15383 Qwen2.5-1M tech report — DIRECT GROUNDING for
    Qwen2.5 family YaRN recipe (factor=4 targets 128K from 32768).
  * pytorch#133780 — DIRECT GROUNDING for SDPA backend-pin necessity.

Run: python -m pytest elai_inference/test_e28_attention_contract.py
"""
from __future__ import annotations

import os
import sys
from contextlib import contextmanager

import pytest


@contextmanager
def _env(**overrides):
    saved = {}
    try:
        for k, v in overrides.items():
            saved[k] = os.environ.get(k)
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v
        yield
    finally:
        for k, prev in saved.items():
            if prev is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = prev


# Top-level import is deferred to inside tests so the module can be
# discovered by pytest in environments without torch+transformers
# (cargo workspace builds skip Python).
def _load_server():
    from elai_inference import server as srv  # type: ignore
    return srv


# --------------------------------------------------------------------------
# Test 1: refuse_fa3_on_ampere refuses FA3/FA4 imports on Ampere
# (E-28 Rules schema row dp-pip-fa3-fa4-ampere)
# --------------------------------------------------------------------------
def test_refuse_fa3_on_ampere_blocks_fa3_module_on_sm86():
    srv = _load_server()
    # Simulate FA3 module loaded by injecting a fake into sys.modules.
    fake_name = "flash_attn_3"
    sys.modules[fake_name] = type(sys)("flash_attn_3_fake")  # placeholder module
    try:
        with _env(ELAI_REFUSE_FA3="1"):
            with pytest.raises(srv.SidecarBootError) as exc:
                srv.refuse_fa3_on_ampere(86)  # RTX 3090 Ampere SM_86
            assert "FA3/FA4" in str(exc.value)
            assert "Hopper" in str(exc.value) or "Blackwell" in str(exc.value)
    finally:
        sys.modules.pop(fake_name, None)


def test_refuse_fa3_on_ampere_passes_through_on_hopper():
    srv = _load_server()
    fake_name = "flash_attn_3"
    sys.modules[fake_name] = type(sys)("flash_attn_3_fake")
    try:
        with _env(ELAI_REFUSE_FA3="1"):
            # SM_90 == Hopper; FA3 is the optimized path, no refusal.
            srv.refuse_fa3_on_ampere(90)
    finally:
        sys.modules.pop(fake_name, None)


def test_refuse_fa3_on_ampere_opt_out_via_env():
    srv = _load_server()
    fake_name = "flash_attn_4"
    sys.modules[fake_name] = type(sys)("flash_attn_4_fake")
    try:
        with _env(ELAI_REFUSE_FA3="0"):
            # Explicit opt-out — cloud Hopper deployments only.
            srv.refuse_fa3_on_ampere(86)
    finally:
        sys.modules.pop(fake_name, None)


# --------------------------------------------------------------------------
# Test 2: pin_sdpa_backend returns the (ctx_mgr, backend, pin_active) tuple
# (E-28 Rules schema row dp-pip-sdpa-no-backend-pin)
# --------------------------------------------------------------------------
def test_pin_sdpa_backend_returns_three_tuple():
    srv = _load_server()
    ctx_mgr, backend_name, pin_active = srv.pin_sdpa_backend()
    # On torch>=2.3 we get FLASH_ATTENTION; on older torch nullcontext.
    assert isinstance(backend_name, str)
    assert isinstance(pin_active, bool)
    # The ctx_mgr is always usable as a with-statement target.
    with ctx_mgr:
        pass


# --------------------------------------------------------------------------
# Test 3: apply_streaming_llm_mask preserves sinks + window
# (E-28 plan Task 2; arXiv:2309.17453 Sec 4.2 sink_count >= 4)
# --------------------------------------------------------------------------
def test_apply_streaming_llm_mask_short_context_is_passthrough():
    srv = _load_server()
    import torch
    mask = torch.ones(1, 100, dtype=torch.float32)
    out = srv.apply_streaming_llm_mask(mask, sink_count=4, window_size=50, context_position=20)
    # context_position < sink_count + window_size — passthrough (no eviction).
    assert torch.equal(out, mask), "short context should be unmodified"


def test_apply_streaming_llm_mask_masks_eviction_region():
    srv = _load_server()
    import torch
    seq_len = 200
    mask = torch.ones(1, seq_len, dtype=torch.float32)
    sink_count = 4
    window_size = 50
    context_position = 150
    out = srv.apply_streaming_llm_mask(
        mask, sink_count=sink_count, window_size=window_size, context_position=context_position
    )
    # Sinks [0, 4) kept; window [100, 150) kept; positions [4, 100) and
    # [150, 200) are masked OUT (zeroed in the additive HF convention).
    assert out[0, 0].item() == 1.0
    assert out[0, 3].item() == 1.0
    assert out[0, 4].item() == 0.0  # evicted from sink region
    assert out[0, 99].item() == 0.0  # evicted between sink + window
    assert out[0, 100].item() == 1.0  # window start
    assert out[0, 149].item() == 1.0  # window end (exclusive)
    assert out[0, 150].item() == 0.0  # past context_position


def test_apply_streaming_llm_mask_rejects_zero_sinks():
    srv = _load_server()
    import torch
    mask = torch.ones(1, 100, dtype=torch.float32)
    with pytest.raises(ValueError):
        srv.apply_streaming_llm_mask(mask, sink_count=0, window_size=50, context_position=30)


# --------------------------------------------------------------------------
# Test 4: apply_yarn_rope_scaling_at_boot is gated by ELAI_YARN_EVAL_GATE
# (E-28 reconciliation with E-32 refuse_yarn_at_boot)
# --------------------------------------------------------------------------
def test_apply_yarn_rope_scaling_refuses_without_eval_gate():
    srv = _load_server()

    class FakeConfig:
        rope_scaling = None

    with _env(ELAI_YARN_EVAL_GATE=None):  # unset
        with pytest.raises(srv.SidecarBootError) as exc:
            srv.apply_yarn_rope_scaling_at_boot(FakeConfig(), 4.0)
        assert "ELAI_YARN_EVAL_GATE" in str(exc.value)


def test_apply_yarn_rope_scaling_injects_when_gate_set():
    srv = _load_server()

    class FakeConfig:
        rope_scaling = None

    cfg = FakeConfig()
    with _env(ELAI_YARN_EVAL_GATE="1"):
        out = srv.apply_yarn_rope_scaling_at_boot(cfg, 4.0, original_max_position=32_768)
    assert out is cfg
    assert cfg.rope_scaling == {
        "type": "yarn",
        "factor": 4.0,
        "original_max_position_embeddings": 32_768,
    }


# --------------------------------------------------------------------------
# Test 5: NO FA3 / FA4 / cuDNN-attention imports in server.py
# (E-28 plan Task DETERMINISTIC_CHECK #6; lock the absence)
# --------------------------------------------------------------------------
def test_no_fa3_fa4_imports_in_server_py():
    server_path = os.path.join(
        os.path.dirname(__file__), "server.py"
    )
    with open(server_path, "r", encoding="utf-8") as f:
        body = f.read()
    forbidden_literals = [
        "from flash_attn_3",
        "import flash_attn_3",
        "from flash_attn_4",
        "import flash_attn_4",
        "from cudnn_attention",
        "import cudnn_attention",
        "FlashAttention3",
        "FlashAttention4",
    ]
    for needle in forbidden_literals:
        assert needle not in body, (
            f"server.py contains forbidden FA3/FA4/cuDNN-attention literal "
            f"'{needle}'; E-28 BLOCK rule dp-pip-fa3-fa4-ampere prohibits "
            f"these on Ampere. See workflow/reasoning/E-28-decisions.md."
        )
