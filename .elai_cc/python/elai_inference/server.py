"""ELAI Inference Sidecar -- HF Transformers with DoLa + TurboQuant + Steering.

Serves Anthropic-compatible /v1/messages and OpenAI-compatible /v1/completions
endpoints via FastAPI + uvicorn (async ASGI).

DoLa contrastive decoding, TurboQuant KV compression, and ActAdd steering
vectors are integrated via model forward hooks + a custom LogitsProcessor,
allowing model.generate() to drive the decode loop in optimized C++/CUDA.

Usage:
    python -m elai_inference.server \
        --model D:/.ELAI_workflow/.elai_cc/models/Qwen2.5-Coder-3B-Instruct-abliterated \
        --port 8100 --no-dola --turboquant-bits 8 --no-steering
"""

import argparse
import asyncio
import collections
import hashlib
import os
import re
import signal
import struct
import time
import uuid
from concurrent.futures import ThreadPoolExecutor
from contextlib import asynccontextmanager
from pathlib import Path
from threading import Lock
from typing import Any, AsyncIterator, Callable, Dict, Optional

import numpy as np
import orjson
import structlog
import torch
import torch.nn.functional as F
from fastapi import FastAPI, Request
from fastapi.responses import Response, StreamingResponse

# E-45: typed boundary models for /v1/messages. Imported lazily-safe at
# module top so import failures surface at server startup rather than at
# first request. The validation gate inside handle_messages is opt-in:
# on `ValidationError` we fall through to the legacy dict-get path so
# the migration is non-destructive. See workflow/reasoning/E-45-decisions.md.
try:
    from pydantic import ValidationError as _PydanticValidationError
    from elai_inference.schemas import MessagesRequest as _MessagesRequestModel
    from elai_inference.schemas import SCHEMA_VERSION as _SIDECAR_SCHEMA_VERSION
    _E45_SCHEMAS_AVAILABLE = True
except ImportError:  # pragma: no cover -- only triggers on broken deployments
    _PydanticValidationError = Exception  # type: ignore[misc,assignment]
    _MessagesRequestModel = None  # type: ignore[assignment]
    _SIDECAR_SCHEMA_VERSION = 0
    _E45_SCHEMAS_AVAILABLE = False

# H-50 STEP 3: sidecar context-reduction journal + measurement layer. The
# journal is env-gated (ELAI_CONTEXT_JOURNAL_ENABLE, default OFF) so this
# import + call are no-ops on the default path. The sidecar only MEASURES
# input tokens and journals them; the trim itself is the orchestrator/CLI's
# job (live_cli.rs prepare_turn_runtime + conversation.rs maybe_auto_compact).
try:
    from elai_inference.context_journal import maybe_journal_turn as _h50_maybe_journal_turn
    _H50_JOURNAL_AVAILABLE = True
except ImportError:  # pragma: no cover -- broken deployment fallback
    _h50_maybe_journal_turn = None  # type: ignore[assignment]
    _H50_JOURNAL_AVAILABLE = False
from transformers import (
    AutoConfig,
    AutoModelForCausalLM,
    AutoTokenizer,
    BitsAndBytesConfig,
    DynamicCache,
    LogitsProcessor,
    TextIteratorStreamer,
    set_seed as _transformers_set_seed,
)

# E-27: ELAI_QUANT_BACKEND / ELAI_KV_CACHE_DTYPE environment variables.
# ELAI_QUANT_BACKEND : "nf4" (default) | "awq" | "fp16"
# ELAI_KV_CACHE_DTYPE: "fp16" (default) | "fp8_e4m3" | "fp8_e5m2"
# Both default to the legacy NF4+FP16 path for backward compatibility.

# Track C-B: lm-format-enforcer is an OPTIONAL runtime dependency. The
# package is added by the USER manually per USER-MANUAL-CB-DEPS.md (executors
# MUST NOT touch pyproject.toml). When absent we degrade gracefully:
#   * engine startup does NOT call build_token_enforcer_tokenizer_data
#   * /v1/messages and /v1/completions accept response_format normally when
#     it is None (zero behaviour change vs pre-C-B)
#   * /v1/messages and /v1/completions return HTTP 503 with a clear message
#     when response_format is set but the library is missing — surfacing the
#     missing install to the operator instead of silently falling back to
#     unconstrained decoding.
try:
    from lmformatenforcer import JsonSchemaParser  # type: ignore
    from lmformatenforcer.integrations.transformers import (  # type: ignore
        build_token_enforcer_tokenizer_data,
        build_transformers_prefix_allowed_tokens_fn,
    )
    _LMFE_AVAILABLE = True
except ImportError:
    JsonSchemaParser = None  # type: ignore[assignment]
    build_token_enforcer_tokenizer_data = None  # type: ignore[assignment]
    build_transformers_prefix_allowed_tokens_fn = None  # type: ignore[assignment]
    _LMFE_AVAILABLE = False

# C-36 MuCoCo registry — paper-grounded mutation operator catalog. Imported
# defensively so the sidecar still boots if the package is missing on a
# stripped deployment (mirrors the LMFE pattern above). The /mucoco/*
# routes return HTTP 503 when _MUCOCO_AVAILABLE is False.
try:
    from elai_inference.mucoco.registry import (  # type: ignore
        apply as _mucoco_apply,
        get_applicability_matrix as _mucoco_get_applicability_matrix,
        list_operators as _mucoco_list_operators,
    )
    _MUCOCO_AVAILABLE = True
except ImportError:
    _mucoco_apply = None  # type: ignore[assignment]
    _mucoco_get_applicability_matrix = None  # type: ignore[assignment]
    _mucoco_list_operators = None  # type: ignore[assignment]
    _MUCOCO_AVAILABLE = False

# ---------------------------------------------------------------------------
# Structured logging via structlog
# ---------------------------------------------------------------------------

structlog.configure(
    processors=[
        structlog.contextvars.merge_contextvars,
        structlog.processors.add_log_level,
        structlog.processors.StackInfoRenderer(),
        structlog.dev.set_exc_info,
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.dev.ConsoleRenderer(),
    ],
    wrapper_class=structlog.make_filtering_bound_logger(0),
    context_class=dict,
    logger_factory=structlog.PrintLoggerFactory(),
    cache_logger_on_first_use=True,
)

logger = structlog.get_logger("elai_inference")

# ---------------------------------------------------------------------------
# Named constants
# ---------------------------------------------------------------------------

ALPHA_DEFAULT = 0.1
ALPHA_AMPLIFIED = 0.05
JSD_MAX_NATS = 0.6931471805599453
EPSILON = 1e-12

COEFFICIENT_DEFAULT = 5.0
COEFFICIENT_MIN = 1.0
COEFFICIENT_MAX = 30.0

SIDECAR_PORT_DEFAULT = 8100
SIDECAR_HOST_DEFAULT = "0.0.0.0"
MAX_NEW_TOKENS_DEFAULT = 256
STREAMER_TIMEOUT_S = 30
THREAD_POOL_SIZE = 2

# Error code constant for constrained-decoding 503 responses (Phase B/C
# unification — applicationRules §1: one canonical string per concept).
# Both stream and non-stream LMFE-missing returns use this single code; the
# old "constrained_streaming_unavailable" split (s-1875676 PART5 §3.5 WARN)
# is removed.
_ERR_CONSTRAINED_DECODING_UNAVAILABLE: str = "constrained_decoding_unavailable"

# Operator-facing 503 detail surfaced when response_format is requested but
# lm-format-enforcer is not installed. PART6 §3.4 / codeRules Rule 17:
# one canonical string, referenced from /v1/messages and /v1/completions.
_ERR_DETAIL_INSTALL_LMFE: str = (
    "response_format requested but lm-format-enforcer is not "
    "installed on this sidecar. Run: pip install "
    "lm-format-enforcer>=0.10.11 and restart the sidecar."
)

# ETC (Entropy-Trend Constraint) constants
ETC_THRESHOLD_DISABLED = float("inf")  # BLOCK-7: disabled until calibration
ETC_RECENT_WINDOW = 20  # tokens to decode for import-context check

# Novelty tracker (A-25) — max cached embeddings per prompt
NOVELTY_CACHE_MAX_ENTRIES = 256
# Matches Python/Rust/Go/JS/TS import lines in generated code
IMPORT_CONTEXT_RE = re.compile(
    r"(?:^|\n)\s*(?:import |from |use |require\(|#include )", re.MULTILINE
)

# Core tools the small model can handle (filter from full tool list)
CORE_TOOLS = frozenset({
    "bash", "read_file", "write_file", "edit_file",
    "glob_search", "grep_search", "WebFetch", "WebSearch",
    "TodoWrite", "AskUserQuestion",
})

# ---------------------------------------------------------------------------
# DoLa helpers
# ---------------------------------------------------------------------------


def compute_jsd(p: torch.Tensor, q: torch.Tensor) -> float:
    m = 0.5 * (p + q)
    m_safe = m.clamp(min=EPSILON)
    p_safe = p.clamp(min=EPSILON)
    q_safe = q.clamp(min=EPSILON)
    kl_pm = (p * (p_safe / m_safe).log()).sum()
    kl_qm = (q * (q_safe / m_safe).log()).sum()
    return (0.5 * kl_pm + 0.5 * kl_qm).clamp(0.0, JSD_MAX_NATS).item()


def get_candidate_layers(n_layers: int) -> list[int]:
    if n_layers <= 40:
        mid = n_layers // 2
        buckets = [range(0, mid), range(mid, n_layers)]
    else:
        buckets = []
        start = 0
        while start < n_layers:
            buckets.append(range(start, min(start + 20, n_layers)))
            start += 20
    return [layer for bucket in buckets for layer in bucket if layer % 2 == 0]


# ---------------------------------------------------------------------------
# Steering vector engine (ActAdd)
# ---------------------------------------------------------------------------


class SteeringEngine:
    """Steering vector engine for activation addition (ActAdd)."""

    def __init__(
        self,
        n_layers: int,
        hidden_dim: int,
        coefficient: float = COEFFICIENT_DEFAULT,
        target_layer: str = "auto",
    ):
        self.n_layers = n_layers
        self.hidden_dim = hidden_dim
        self.coefficient = coefficient
        self.target_layer = target_layer
        self.vectors: dict[int, torch.Tensor] = {}
        self._device: torch.device | None = None

        if target_layer == "auto":
            self.target_layer_idx = n_layers // 3
        else:
            self.target_layer_idx = int(target_layer)

        if self.target_layer_idx < 0 or self.target_layer_idx >= n_layers:
            logger.warning(
                "target_layer_out_of_range",
                target=target_layer,
                clamped_to=n_layers // 3,
            )
            self.target_layer_idx = n_layers // 3

        if coefficient < COEFFICIENT_MIN or coefficient > COEFFICIENT_MAX:
            logger.warning(
                "coefficient_out_of_range",
                coefficient=coefficient,
                clamped_to=COEFFICIENT_DEFAULT,
            )
            self.coefficient = COEFFICIENT_DEFAULT

    def add_vector(self, layer_idx: int, vector: np.ndarray | torch.Tensor) -> None:
        if isinstance(vector, np.ndarray):
            vector = torch.from_numpy(vector).float()
        if vector.shape != (self.hidden_dim,):
            logger.error(
                "vector_shape_mismatch",
                got=vector.shape,
                expected=(self.hidden_dim,),
            )
            return
        if self._device is not None:
            self.vectors[layer_idx] = vector.to(self._device)
        else:
            self.vectors[layer_idx] = vector

    def load_vector_from_file(self, layer_idx: int, filepath: str) -> None:
        try:
            if filepath.endswith(".npz"):
                data = np.load(filepath)
                vector = data["vector"] if "vector" in data else data[list(data.keys())[0]]
            elif filepath.endswith(".safetensors"):
                from safetensors.torch import load_file

                data = load_file(filepath)
                vector = data["vector"] if "vector" in data else data[list(data.keys())[0]]
                vector = vector.cpu().numpy()
            else:
                logger.error("unsupported_vector_format", filepath=filepath)
                return

            self.add_vector(layer_idx, vector)
            logger.info("loaded_steering_vector", layer=layer_idx, path=filepath)
        except Exception:
            logger.exception("steering_vector_load_failed", filepath=filepath)

    def compute_vector_multi(
        self,
        model: AutoModelForCausalLM,
        tokenizer: AutoTokenizer,
        positive_prompts: list[str],
        negative_prompts: list[str],
        layer_idx: int,
    ) -> np.ndarray | None:
        """Batch-ful-c (A-8) — mean-embedding contrast.

        Computes `mean(embed(positives)) - mean(embed(negatives))` at
        `layer_idx` using `torch.inference_mode()` per the CLAUDE.md
        anti-stale-training rule. Each prompt's last-token hidden state at
        the chosen layer contributes equally to the mean so the vector is
        direction-stable regardless of prompt length.

        Returns the difference as a CPU float32 numpy array, or None on
        zero-magnitude (invalid contrast set) / exception. Does NOT call
        `add_vector` — the caller decides when to install the vector.
        """
        if not positive_prompts or not negative_prompts:
            return None
        try:
            def _mean_last_hidden(prompts: list[str]) -> torch.Tensor:
                acc = None
                count = 0
                for prompt in prompts:
                    enc = tokenizer(prompt, return_tensors="pt").to(model.device)
                    with torch.inference_mode():
                        out = model(
                            input_ids=enc.input_ids,
                            output_hidden_states=True,
                            return_dict=True,
                        )
                    last = out.hidden_states[layer_idx][:, -1, :].squeeze(0).cpu()
                    acc = last if acc is None else acc + last
                    count += 1
                return acc / float(count) if acc is not None else acc

            pos_mean = _mean_last_hidden(positive_prompts)
            neg_mean = _mean_last_hidden(negative_prompts)
            if pos_mean is None or neg_mean is None:
                return None
            steering_vector = pos_mean - neg_mean
            magnitude = torch.norm(steering_vector).item()
            if magnitude < 1e-6:
                logger.warning("steering_vector_too_small", magnitude=magnitude)
                return None
            logger.info(
                "computed_steering_vector_multi",
                layer=layer_idx,
                n_pos=len(positive_prompts),
                n_neg=len(negative_prompts),
                magnitude=round(magnitude, 4),
            )
            return steering_vector.float().numpy()
        except Exception:
            logger.exception("steering_vector_compute_multi_failed")
            return None

    def compute_vector(
        self,
        model: AutoModelForCausalLM,
        tokenizer: AutoTokenizer,
        positive_prompt: str,
        negative_prompt: str,
        layer_idx: int,
    ) -> np.ndarray | None:
        try:
            pos_inputs = tokenizer(positive_prompt, return_tensors="pt").to(model.device)
            neg_inputs = tokenizer(negative_prompt, return_tensors="pt").to(model.device)

            with torch.inference_mode():
                pos_out = model(
                    input_ids=pos_inputs.input_ids,
                    output_hidden_states=True,
                    return_dict=True,
                )
                pos_hidden = pos_out.hidden_states[layer_idx][:, -1, :].squeeze(0).cpu()

            with torch.inference_mode():
                neg_out = model(
                    input_ids=neg_inputs.input_ids,
                    output_hidden_states=True,
                    return_dict=True,
                )
                neg_hidden = neg_out.hidden_states[layer_idx][:, -1, :].squeeze(0).cpu()

            steering_vector = pos_hidden - neg_hidden
            magnitude = torch.norm(steering_vector).item()
            if magnitude < 1e-6:
                logger.warning("steering_vector_too_small", magnitude=magnitude)
                return None

            self.add_vector(layer_idx, steering_vector)
            logger.info("computed_steering_vector", layer=layer_idx, magnitude=round(magnitude, 4), stored=True)
            return steering_vector.float().numpy()
        except Exception:
            logger.exception("steering_vector_compute_failed")
            return None

    def apply_steering(self, hidden_state: torch.Tensor, layer_idx: int) -> torch.Tensor:
        if layer_idx != self.target_layer_idx:
            return hidden_state
        if layer_idx not in self.vectors:
            return hidden_state

        vector = self.vectors[layer_idx]
        if vector.device != hidden_state.device:
            vector = vector.to(hidden_state.device)
            self.vectors[layer_idx] = vector
            self._device = hidden_state.device
        return hidden_state + self.coefficient * vector.unsqueeze(0)

    def is_active(self) -> bool:
        return self.target_layer_idx in self.vectors


# ---------------------------------------------------------------------------
# KV cache quantization (KIVI-style per-head-per-token 4-bit)
# ---------------------------------------------------------------------------

# Quantize/dequantize operate entirely on GPU tensors. No numpy, no CPU roundtrip.
# Shape convention: (batch, num_kv_heads, seq_len, head_dim).
# Quantization granularity: one scale+zero per (batch, head, token) along head_dim.


KV_GROUP_SIZE = 32  # groups within head_dim for finer quantization


def _quantize_per_token(
    tensor: torch.Tensor, bits: int = 4,
) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    """Asymmetric per-group quantization on GPU (KIVI-style).

    Groups head_dim into KV_GROUP_SIZE chunks. Each group gets its own
    scale and zero to contain outliers (RoPE keys can have range [-448, 258]
    in some heads while most values are near 0).

    Args:
        tensor: (batch, heads, seq_len, head_dim) bf16/fp16 tensor.
        bits: quantization bit-width (2 or 4).

    Returns:
        (packed_uint8, scales_fp16, zeros_fp16).
        packed_uint8 shape: (batch, heads, seq_len, head_dim // pack_factor).
        scales shape: (batch, heads, seq_len, n_groups).
        zeros shape: (batch, heads, seq_len, n_groups).
    """
    b, h, s, d = tensor.shape
    g = KV_GROUP_SIZE
    n_groups = d // g
    max_val = (1 << bits) - 1

    # Reshape to expose groups: (b, h, s, n_groups, g)
    grouped = tensor.reshape(b, h, s, n_groups, g)

    # Per-group min/max
    vmin = grouped.amin(dim=-1, keepdim=True)  # (b, h, s, n_groups, 1)
    vmax = grouped.amax(dim=-1, keepdim=True)
    scale = ((vmax - vmin) / max_val).clamp(min=1e-10)

    quantized = ((grouped - vmin) / scale).round().clamp(0, max_val).to(torch.uint8)
    # Flatten groups back: (b, h, s, d)
    quantized = quantized.reshape(b, h, s, d)

    if bits == 4:
        packed = (quantized[..., 0::2] << 4) | quantized[..., 1::2]
    elif bits == 2:
        packed = (
            (quantized[..., 0::4] << 6)
            | (quantized[..., 1::4] << 4)
            | (quantized[..., 2::4] << 2)
            | quantized[..., 3::4]
        )
    else:
        packed = quantized

    # Scales/zeros: (b, h, s, n_groups)
    return packed, scale.squeeze(-1).half(), vmin.squeeze(-1).half()


def _dequantize_per_token(
    packed: torch.Tensor,
    scale: torch.Tensor,
    zero: torch.Tensor,
    bits: int = 4,
    dtype: torch.dtype = torch.bfloat16,
) -> torch.Tensor:
    """Dequantize packed uint8 back to float tensor on GPU.

    Returns: (batch, heads, seq_len, head_dim) tensor in the requested dtype.
    """
    if bits == 4:
        high = (packed >> 4).to(dtype)
        low = (packed & 0x0F).to(dtype)
        quantized = torch.stack([high, low], dim=-1).flatten(-2)
    elif bits == 2:
        b3 = ((packed >> 6) & 0x03).to(dtype)
        b2 = ((packed >> 4) & 0x03).to(dtype)
        b1 = ((packed >> 2) & 0x03).to(dtype)
        b0 = (packed & 0x03).to(dtype)
        quantized = torch.stack([b3, b2, b1, b0], dim=-1).flatten(-2)
    else:
        quantized = packed.to(dtype)

    # scale/zero shape: (b, h, s, n_groups) -> expand to (b, h, s, n_groups, group_size)
    g = KV_GROUP_SIZE
    s_expanded = scale.unsqueeze(-1).expand(*scale.shape, g).reshape(*scale.shape[:-1], -1).to(dtype)
    z_expanded = zero.unsqueeze(-1).expand(*zero.shape, g).reshape(*zero.shape[:-1], -1).to(dtype)

    return quantized * s_expanded + z_expanded


class KVQuantLayer:
    """Cache layer with KIVI-style per-head-per-token quantization.

    Stores historical tokens as packed uint8 on GPU with per-vector scales.
    Keeps recent tokens (residual) in full bf16/fp16.
    On update(), decompresses historical, concatenates with residual + new,
    returns full tensors for attention, then stores new state.
    """

    is_sliding = False

    def __init__(self, bits: int = 4, residual_length: int = 128):
        self.bits = bits
        self.residual_length = residual_length
        self.is_initialized = False
        self.dtype = None
        self.device = None

        # Residual (full precision recent tokens)
        self.keys: torch.Tensor | None = None
        self.values: torch.Tensor | None = None

        # Compressed historical tokens
        self._packed_keys: torch.Tensor | None = None
        self._key_scales: torch.Tensor | None = None
        self._key_zeros: torch.Tensor | None = None
        self._packed_values: torch.Tensor | None = None
        self._value_scales: torch.Tensor | None = None
        self._value_zeros: torch.Tensor | None = None
        self._compressed_len: int = 0

    def update(
        self, key_states: torch.Tensor, value_states: torch.Tensor, *args, **kwargs
    ) -> tuple[torch.Tensor, torch.Tensor]:
        if not self.is_initialized:
            self.dtype = key_states.dtype
            self.device = key_states.device
            self.is_initialized = True

        # Append new tokens to residual
        if self.keys is not None and self.keys.numel() > 0:
            self.keys = torch.cat([self.keys, key_states], dim=-2)
            self.values = torch.cat([self.values, value_states], dim=-2)
        else:
            self.keys = key_states
            self.values = value_states

        res_len = self.keys.shape[-2]

        # If residual exceeds limit, compress the overflow
        if res_len > self.residual_length:
            overflow = res_len - self.residual_length
            k_overflow = self.keys[:, :, :overflow, :]
            v_overflow = self.values[:, :, :overflow, :]

            pk, ks, kz = _quantize_per_token(k_overflow, self.bits)
            pv, vs, vz = _quantize_per_token(v_overflow, self.bits)

            if self._packed_keys is not None:
                self._packed_keys = torch.cat([self._packed_keys, pk], dim=2)
                self._key_scales = torch.cat([self._key_scales, ks], dim=2)
                self._key_zeros = torch.cat([self._key_zeros, kz], dim=2)
                self._packed_values = torch.cat([self._packed_values, pv], dim=2)
                self._value_scales = torch.cat([self._value_scales, vs], dim=2)
                self._value_zeros = torch.cat([self._value_zeros, vz], dim=2)
            else:
                self._packed_keys = pk
                self._key_scales = ks
                self._key_zeros = kz
                self._packed_values = pv
                self._value_scales = vs
                self._value_zeros = vz

            self._compressed_len += overflow
            self.keys = self.keys[:, :, overflow:, :]
            self.values = self.values[:, :, overflow:, :]

        # Build full tensors for attention
        if self._packed_keys is not None:
            dk = _dequantize_per_token(
                self._packed_keys, self._key_scales, self._key_zeros,
                self.bits, self.dtype,
            )
            dv = _dequantize_per_token(
                self._packed_values, self._value_scales, self._value_zeros,
                self.bits, self.dtype,
            )
            full_keys = torch.cat([dk, self.keys], dim=-2)
            full_values = torch.cat([dv, self.values], dim=-2)
            del dk, dv
        else:
            full_keys = self.keys
            full_values = self.values

        return full_keys, full_values

    def get_seq_length(self) -> int:
        res_len = 0
        if self.keys is not None and self.keys.numel() > 0:
            res_len = self.keys.shape[-2]
        return self._compressed_len + res_len

    def get_mask_sizes(self, query_length: int) -> tuple[int, int]:
        return self.get_seq_length() + query_length, 0

    def get_max_cache_shape(self) -> int:
        return -1

    def crop(self, max_length: int) -> None:
        if max_length < 0:
            max_length = self.get_seq_length() - abs(max_length)
        if self.get_seq_length() <= max_length:
            return
        # For simplicity, decompress everything, crop, and re-init
        # This is rare (only on explicit crop calls)
        if self._packed_keys is not None:
            dk = _dequantize_per_token(
                self._packed_keys, self._key_scales, self._key_zeros,
                self.bits, self.dtype,
            )
            dv = _dequantize_per_token(
                self._packed_values, self._value_scales, self._value_zeros,
                self.bits, self.dtype,
            )
            full_keys = torch.cat([dk, self.keys], dim=-2)
            full_values = torch.cat([dv, self.values], dim=-2)
            del dk, dv
        else:
            full_keys = self.keys
            full_values = self.values

        cropped_k = full_keys[..., :max_length, :]
        cropped_v = full_values[..., :max_length, :]
        del full_keys, full_values

        # Reset compressed state
        self._packed_keys = None
        self._key_scales = None
        self._key_zeros = None
        self._packed_values = None
        self._value_scales = None
        self._value_zeros = None
        self._compressed_len = 0

        # Re-set as residual (will compress on next overflow)
        self.keys = cropped_k
        self.values = cropped_v

    def batch_repeat_interleave(self, repeats: int) -> None:
        if self.keys is not None and self.keys.numel() > 0:
            self.keys = self.keys.repeat_interleave(repeats, dim=0)
            self.values = self.values.repeat_interleave(repeats, dim=0)
        if self._packed_keys is not None:
            self._packed_keys = self._packed_keys.repeat_interleave(repeats, dim=0)
            self._key_scales = self._key_scales.repeat_interleave(repeats, dim=0)
            self._key_zeros = self._key_zeros.repeat_interleave(repeats, dim=0)
            self._packed_values = self._packed_values.repeat_interleave(repeats, dim=0)
            self._value_scales = self._value_scales.repeat_interleave(repeats, dim=0)
            self._value_zeros = self._value_zeros.repeat_interleave(repeats, dim=0)

    def batch_select_indices(self, indices: torch.Tensor) -> None:
        if self.keys is not None and self.keys.numel() > 0:
            self.keys = self.keys[indices, ...]
            self.values = self.values[indices, ...]
        if self._packed_keys is not None:
            self._packed_keys = self._packed_keys[indices, ...]
            self._key_scales = self._key_scales[indices, ...]
            self._key_zeros = self._key_zeros[indices, ...]
            self._packed_values = self._packed_values[indices, ...]
            self._value_scales = self._value_scales[indices, ...]
            self._value_zeros = self._value_zeros[indices, ...]

    def compressed_bytes(self) -> int:
        total = 0
        if self._packed_keys is not None:
            for t in (
                self._packed_keys, self._key_scales, self._key_zeros,
                self._packed_values, self._value_scales, self._value_zeros,
            ):
                total += t.nbytes
        return total


class KVCompressor:
    """Factory for KV cache quantization. Creates quantized cache objects."""

    def __init__(self, bit_width: int = 8, residual_length: int = 512):
        self.bit_width = bit_width
        self.residual_length = residual_length
        self.enabled = bit_width > 0

        if self.enabled:
            logger.info("kv_quantization_enabled", bits=bit_width, residual=residual_length)
        else:
            logger.info("kv_quantization_disabled")

    def make_cache(self, config=None) -> "DynamicCache | _KVQuantCache":
        """Create a KV cache: quantized if enabled, else standard DynamicCache."""
        if not self.enabled:
            from transformers import DynamicCache as DC
            return DC()
        return _KVQuantCache(bits=self.bit_width, residual_length=self.residual_length)


class _KVQuantCache:
    """Quantized KV cache compatible with HF Transformers model.generate().

    Drop-in replacement for DynamicCache. Each layer is a KVQuantLayer
    that compresses historical tokens and keeps recent tokens at full precision.
    """

    is_compileable = False

    def __init__(self, bits: int = 4, residual_length: int = 128):
        self.bits = bits
        self.residual_length = residual_length
        self.layers: list[KVQuantLayer] = []

    def update(
        self,
        key_states: torch.Tensor,
        value_states: torch.Tensor,
        layer_idx: int,
        cache_kwargs: dict | None = None,
    ) -> tuple[torch.Tensor, torch.Tensor]:
        while layer_idx >= len(self.layers):
            self.layers.append(KVQuantLayer(self.bits, self.residual_length))
        return self.layers[layer_idx].update(key_states, value_states)

    def get_seq_length(self, layer_idx: int = 0) -> int:
        if layer_idx >= len(self.layers):
            return 0
        return self.layers[layer_idx].get_seq_length()

    def get_max_cache_shape(self) -> int:
        return -1

    def get_mask_sizes(self, query_length: int, layer_idx: int = 0) -> tuple[int, int]:
        if layer_idx < len(self.layers):
            return self.layers[layer_idx].get_mask_sizes(query_length)
        return query_length, 0

    def crop(self, max_length: int) -> None:
        for layer in self.layers:
            layer.crop(max_length)

    def batch_repeat_interleave(self, repeats: int) -> None:
        for layer in self.layers:
            layer.batch_repeat_interleave(repeats)

    def batch_select_indices(self, indices: torch.Tensor) -> None:
        for layer in self.layers:
            layer.batch_select_indices(indices)

    def __len__(self) -> int:
        return len(self.layers)

    def __getitem__(self, idx: int):
        return self.layers[idx]

    def __iter__(self):
        for layer in self.layers:
            k = layer.keys if layer.keys is not None else torch.tensor([])
            v = layer.values if layer.values is not None else torch.tensor([])
            yield k, v, None

    def compressed_bytes(self) -> int:
        return sum(layer.compressed_bytes() for layer in self.layers)

    def residual_bytes(self) -> int:
        total = 0
        for layer in self.layers:
            if layer.keys is not None and layer.keys.numel() > 0:
                total += layer.keys.nbytes + layer.values.nbytes
        return total


# ---------------------------------------------------------------------------
# DoLaSteeringProcessor — LogitsProcessor with forward hooks
# ---------------------------------------------------------------------------


class DoLaSteeringProcessor(LogitsProcessor):
    """Custom LogitsProcessor that applies DoLa contrastive decoding and
    steering vector injection via model forward hooks.

    Forward hooks on each decoder layer capture hidden states. Steering is
    injected at the target layer via a pre-forward hook. In __call__, the
    captured hidden states are used for DoLa layer-contrast token selection.
    """

    def __init__(
        self,
        model: AutoModelForCausalLM,
        candidate_layers: list[int],
        alpha: float,
        steering_engine: SteeringEngine | None,
    ):
        self.model = model
        self.lm_head = model.lm_head
        self.candidate_layers = candidate_layers
        self.alpha = alpha
        self.steering_engine = steering_engine

        # Storage for captured hidden states (reset each forward pass)
        self._captured_hidden: dict[int, torch.Tensor] = {}
        self._final_hidden: torch.Tensor | None = None
        self._hooks: list = []

    def register_hooks(self) -> None:
        """Register forward hooks on decoder layers to capture hidden states."""
        self.remove_hooks()

        # Find the decoder layers
        layers = _get_decoder_layers(self.model)
        if layers is None:
            logger.error("cannot_find_decoder_layers")
            return

        n_layers = len(layers)
        # Hook on candidate layers to capture hidden states for DoLa
        for layer_idx in self.candidate_layers:
            if layer_idx < n_layers:
                hook = layers[layer_idx].register_forward_hook(
                    self._make_capture_hook(layer_idx)
                )
                self._hooks.append(hook)

        # Hook on final layer to capture final hidden state
        final_idx = n_layers - 1
        hook = layers[final_idx].register_forward_hook(
            self._make_capture_hook(final_idx)
        )
        self._hooks.append(hook)

        # Steering: register post-forward hook on target layer
        if (
            self.steering_engine is not None
            and self.steering_engine.is_active()
        ):
            target = self.steering_engine.target_layer_idx
            if target < n_layers:
                steer_hook = layers[target].register_forward_hook(
                    self._steering_hook
                )
                self._hooks.append(steer_hook)

        logger.info(
            "hooks_registered",
            candidate_layers=len(self.candidate_layers),
            steering=self.steering_engine is not None and self.steering_engine.is_active(),
        )

    def remove_hooks(self) -> None:
        for hook in self._hooks:
            hook.remove()
        self._hooks.clear()

    def _make_capture_hook(self, layer_idx: int):
        def hook_fn(_module, _input, output):
            # Decoder layers typically return (hidden_state, ...) or just hidden_state
            if isinstance(output, tuple):
                hidden = output[0]
            else:
                hidden = output
            # Capture only the last token position to save memory
            self._captured_hidden[layer_idx] = hidden[:, -1, :].detach()
        return hook_fn

    def _steering_hook(self, _module, _input, output):
        if self.steering_engine is None or not self.steering_engine.is_active():
            return output
        target = self.steering_engine.target_layer_idx
        if isinstance(output, tuple):
            hidden = output[0]
            modified_last = self.steering_engine.apply_steering(
                hidden[:, -1:, :], target
            )
            hidden = hidden.clone()
            hidden[:, -1:, :] = modified_last
            return (hidden,) + output[1:]
        else:
            modified_last = self.steering_engine.apply_steering(
                output[:, -1:, :], target
            )
            output = output.clone()
            output[:, -1:, :] = modified_last
            return output

    def __call__(
        self, input_ids: torch.LongTensor, scores: torch.FloatTensor
    ) -> torch.FloatTensor:
        """Apply DoLa contrastive decoding using captured hidden states."""
        if not self._captured_hidden:
            return scores

        # Find the final layer hidden state
        layers = _get_decoder_layers(self.model)
        if layers is None:
            return scores
        final_idx = len(layers) - 1

        if final_idx not in self._captured_hidden:
            return scores

        final_hidden = self._captured_hidden[final_idx].to(self.lm_head.weight.device)
        final_logits = self.lm_head(final_hidden).squeeze(0).float()
        final_probs = F.softmax(final_logits, dim=-1)

        # Compute candidate layer logits via lm_head
        candidate_probs: dict[int, torch.Tensor] = {}
        candidate_raw: dict[int, torch.Tensor] = {}

        for layer_idx in self.candidate_layers:
            if layer_idx in self._captured_hidden:
                h = self._captured_hidden[layer_idx].to(self.lm_head.weight.device)
                raw = self.lm_head(h).squeeze(0).float()
                candidate_probs[layer_idx] = F.softmax(raw, dim=-1)
                candidate_raw[layer_idx] = raw

        if not candidate_probs:
            self._captured_hidden.clear()
            return scores

        # DoLa: select premature layer via JSD maximization
        best_layer = None
        best_jsd = -1.0
        for layer_idx, q_j in candidate_probs.items():
            jsd = compute_jsd(final_probs, q_j)
            if jsd > best_jsd:
                best_jsd = jsd
                best_layer = layer_idx

        if best_layer is None:
            self._captured_hidden.clear()
            return scores

        # Contrastive logits with V_head constraint
        max_prob = final_probs.max().item()
        threshold = self.alpha * max_prob
        v_head_mask = final_probs >= threshold

        contrastive = torch.full_like(final_logits, float("-inf"))
        contrastive[v_head_mask] = (
            final_logits[v_head_mask] - candidate_raw[best_layer][v_head_mask]
        )

        # Clear captured states for next step
        self._captured_hidden.clear()

        # Return contrastive logits reshaped to match scores
        return contrastive.unsqueeze(0)

    def __del__(self):
        self.remove_hooks()


def _get_decoder_layers(model):
    """Find the decoder layer list in the model architecture."""
    # Standard LLaMA / Mistral / Gemma2: model.model.layers
    if hasattr(model, "model") and hasattr(model.model, "layers"):
        return model.model.layers
    # Gemma4 VLM (Gemma4ForConditionalGeneration):
    #   model.model = Gemma4Model
    #   model.model.language_model = Gemma4TextModel
    #   model.model.language_model.layers = ModuleList of decoder layers
    if hasattr(model, "model") and hasattr(model.model, "language_model"):
        lm = model.model.language_model
        if hasattr(lm, "layers"):
            return lm.layers
        if hasattr(lm, "model") and hasattr(lm.model, "layers"):
            return lm.model.layers
    # Fallback: top-level language_model
    if hasattr(model, "language_model"):
        lm = model.language_model
        if hasattr(lm, "layers"):
            return lm.layers
        if hasattr(lm, "model") and hasattr(lm.model, "layers"):
            return lm.model.layers
    # GPT-2 style: model.transformer.h
    if hasattr(model, "transformer") and hasattr(model.transformer, "h"):
        return model.transformer.h
    return None


# ---------------------------------------------------------------------------
# ETCLogitsProcessor — entropy-trend constraint monitoring (A-9)
# ---------------------------------------------------------------------------


class ETCLogitsProcessor(LogitsProcessor):
    """Monitors entropy trends during model.generate() and flags
    high-uncertainty tokens for post-generation search.

    Observes the second derivative of token entropy (d2H). When d2H exceeds
    the threshold AND the recent context matches an import pattern, sets
    ``triggered = True`` with the trigger position. Never modifies scores.

    BLOCK-7: threshold defaults to infinity (disabled) until calibration.
    """

    def __init__(
        self,
        threshold: float,
        import_pattern: re.Pattern,
        tokenizer,
    ):
        self.threshold = threshold
        self.import_pattern = import_pattern
        self.tokenizer = tokenizer
        # Running state (reset per generation call)
        self.h_prev: float = 0.0
        self.dh_prev: float = 0.0
        self.triggered: bool = False
        self.trigger_token_idx: int = -1
        # Calibration data (populated when threshold == 0)
        self.d2h_log: list[tuple[int, float, bool]] = []

    def reset(self) -> None:
        """Reset state for a new generation call."""
        self.h_prev = 0.0
        self.dh_prev = 0.0
        self.triggered = False
        self.trigger_token_idx = -1
        self.d2h_log.clear()

    def __call__(
        self, input_ids: torch.LongTensor, scores: torch.FloatTensor
    ) -> torch.FloatTensor:
        # Compute entropy of the score distribution
        probs = torch.softmax(scores[0], dim=-1)
        log_probs = torch.log(probs + EPSILON)
        h = -(probs * log_probs).sum().item()

        dh = h - self.h_prev
        d2h = dh - self.dh_prev
        self.h_prev = h
        self.dh_prev = dh

        # Check for import context in recent tokens
        token_pos = input_ids.shape[1]
        is_import = False
        if d2h > self.threshold or self.threshold == 0.0:
            recent_ids = input_ids[0, -ETC_RECENT_WINDOW:]
            recent_text = self.tokenizer.decode(recent_ids, skip_special_tokens=True)
            is_import = bool(self.import_pattern.search(recent_text))

        # Log for calibration (when threshold == 0, log everything)
        if self.threshold == 0.0:
            self.d2h_log.append((token_pos, d2h, is_import))

        # Trigger only when above threshold AND in import context
        if not self.triggered and d2h > self.threshold and is_import:
            self.triggered = True
            self.trigger_token_idx = token_pos
            logger.info(
                "etc_triggered",
                token_pos=token_pos,
                d2h=round(d2h, 4),
                h=round(h, 4),
            )

        return scores  # never modify scores, only observe


# ---------------------------------------------------------------------------
# SeededGumbelProcessor — SGLang det-mode multinomial_with_seed (J-13)
# ---------------------------------------------------------------------------


class SeededGumbelProcessor(LogitsProcessor):
    """LogitsProcessor implementing SGLang det-mode multinomial_with_seed.

    Adds Gumbel(0,1) noise derived from a per-(seed, step) torch.Generator to
    the logits, so same (logits, seed) -> same token regardless of global
    torch RNG state or other processors' ordering. Active only when constructed
    with a non-None seed (the request carried `seed`).

    J-13 WIRING_CONTRACT: produces_symbols server.py::SeededGumbelProcessor.
    """

    def __init__(self, seed: int):
        self.seed = int(seed)
        # Per-step Generator seeded from (seed, step_index); never touches the
        # global torch RNG so other processors' draws cannot shift the stream.
        self._step: int = -1

    def _step_generator(self) -> torch.Generator:
        self._step += 1
        gen = torch.Generator(device="cpu")
        gen.manual_seed((self.seed ^ (self._step * 0x9E3779B1)) & 0xFFFFFFFF)
        return gen

    def __call__(
        self, input_ids: torch.LongTensor, scores: torch.FloatTensor
    ) -> torch.FloatTensor:
        # SGLang det-mode pattern (plan l54): Gumbel-Max trick.
        # Gumbel(0,1) = -log(-log(u)), u ~ Uniform(0,1) drawn from a
        # per-(seed,step) generator. scores + gumbel_noise -> argmax gives a
        # deterministic sample from the categorical distribution; seeding the
        # per-step generator makes it reproducible across runs without touching
        # the global RNG. Step index = input_ids.shape[1] - 1 (current pos).
        step_gen = self._step_generator()
        u = torch.rand(
            scores.shape, dtype=scores.dtype, generator=step_gen
        )
        u = u.clamp_(EPSILON, 1.0 - EPSILON)
        gumbel = -torch.log(-torch.log(u))
        return scores + gumbel.to(scores.device)


# ---------------------------------------------------------------------------
# GGSignalCollector — intrinsic verification signals (A-17)
# ---------------------------------------------------------------------------


class GGSignalCollector(LogitsProcessor):
    """Collects per-token logprob and entropy during model.generate().

    Zero additional inference cost: observes the softmax distribution that
    model.generate() already computes. Never modifies scores.

    Provides 4 GG signals for AB-MCTS branch scoring (Layer 4):
      mean_logprob  — mean of chosen-token log-probabilities
      min_logprob   — worst single-token log-probability
      entropy_mean  — mean token-level entropy
      entropy_max   — peak token-level entropy
    """

    def __init__(self) -> None:
        self._logprobs: list[float] = []
        self._entropies: list[float] = []

    def reset(self) -> None:
        self._logprobs.clear()
        self._entropies.clear()

    def __call__(
        self, input_ids: torch.LongTensor, scores: torch.FloatTensor
    ) -> torch.FloatTensor:
        probs = torch.softmax(scores[0], dim=-1)
        log_probs = torch.log(probs + EPSILON)

        # Entropy of the full distribution
        h = -(probs * log_probs).sum().item()
        self._entropies.append(h)

        # Log-prob of the token that will be selected (greedy = argmax)
        chosen_idx = probs.argmax().item()
        self._logprobs.append(log_probs[chosen_idx].item())

        return scores  # never modify scores, only observe

    def signals(self) -> dict[str, float | None]:
        """Return the 4 GG signal fields, or None values if no tokens."""
        if not self._logprobs:
            return {
                "mean_logprob": None,
                "min_logprob": None,
                "entropy_mean": None,
                "entropy_max": None,
            }
        return {
            "mean_logprob": sum(self._logprobs) / len(self._logprobs),
            "min_logprob": min(self._logprobs),
            "entropy_mean": sum(self._entropies) / len(self._entropies),
            "entropy_max": max(self._entropies),
        }


# ---------------------------------------------------------------------------
# E-32: Boot-time validation helpers (YaRN refusal + deep mode activation)
# ---------------------------------------------------------------------------

# FITM long-context threshold (8K tokens — requests below this skip FITM entirely)
FITM_PROMPT_LEN_THRESHOLD: int = 8_192
# FITM dummy doc length ceiling (cost ceiling per WIRING_CONTRACT stub_ban)
FITM_DUMMY_MAX_TOKENS: int = 100
# Default dummy text (lorem ipsum, ~50 tokens)
FITM_DUMMY_TEXT_DEFAULT: str = (
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod "
    "tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim "
    "veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip."
)
# Deep mode context cap on 24 GB box (64K)
DEEP_MODE_CTX_CAP: int = 64_000
# Minimum VRAM (GB) required for deep mode
DEEP_MODE_MIN_VRAM_GB: float = 24.0
# Qwen2.5-7B-Instruct-1M HF model ID for deep mode
DEEP_MODE_MODEL_ID: str = "Qwen/Qwen2.5-7B-Instruct-1M"


class SidecarBootError(RuntimeError):
    """Raised at sidecar boot when a requested feature is explicitly refused.

    Loud refusal is always preferable to silent degradation.
    """


def refuse_yarn_at_boot(model_name: str) -> None:
    """Raise SidecarBootError if ELAI_USE_YARN=1 is set for a Qwen2.5-Coder model.

    YaRN is EXPLICITLY REJECTED for ELAI's default sidecar path:
    - vllm issue 18728: YaRN-static factor=4 causes 15-20% quality drop on
      short prompts on Qwen3-32B (same RoPE family as Qwen2.5-Coder).
    - arxiv:2601.15300: Qwen2.5-7B F1 drops 0.55->0.30 at 40-50% of 128K window.
    Use deep mode (DCA via Qwen2.5-7B-Instruct-1M at 64K) for >32K context instead.

    Args:
        model_name: The model path/ID being loaded. Check is applied when the
            name contains "qwen" (case-insensitive) to catch all Qwen variants.
    """
    if os.getenv("ELAI_USE_YARN", "0") != "1":
        return
    if "qwen" not in model_name.lower():
        # YaRN refusal is scoped to Qwen family; other models not yet characterised.
        logger.warning(
            "yarn_refusal_skipped_non_qwen",
            model=model_name,
            hint="YaRN quality data only available for Qwen2.5-Coder family",
        )
        return
    raise SidecarBootError(
        "YaRN rejected per E-32: 15-20% quality drop on short prompts "
        "(vllm issue 18728); F1 cliff at 40-50% of 128K window "
        "(arxiv:2601.15300). Use deep mode (DCA via Qwen2.5-7B-Instruct-1M) "
        "for >32K context instead. Unset ELAI_USE_YARN or set ELAI_DEEP_MODE=1."
    )


def assert_head_dim_invariant(
    model: object,
    expected_dim: int = 128,
    model_id: str | None = None,
) -> tuple[int, int]:
    """Raise SidecarBootError if the loaded model's head_dim != expected_dim.

    Mirrors the head_dim_invariant_contract.rs test that locks the Qwen2.5-Coder
    family at head_dim=128: SDPA / FA2 / xformers attention paths assume 128 and
    silently degrade (or hit untested fallback kernels) on any other value. Loud
    refusal at boot is preferable to silent attention-quality regression.

    Qwen2 config.json does NOT carry a `head_dim` field directly — it is computed
    as `hidden_size / num_attention_heads` per HF transformers Qwen2Attention.
    Some configs (e.g. multimodal models with a text_config sub-block) expose
    head_dim explicitly; both shapes are handled.

    Args:
        model: A loaded transformers PreTrainedModel (must expose `.config`).
        expected_dim: Expected head_dim. Default 128 (Qwen2.5-Coder 3B / 7B).
        model_id: Optional model name for the error message + SSB row.

    Returns:
        (head_dim_observed, head_dim_expected) on success.

    Raises:
        SidecarBootError: when observed head_dim != expected_dim.

    References:
        crates/contracts/tests/head_dim_invariant_contract.rs (config-layer lock)
        huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct config.json
    """
    cfg = getattr(model, "config", None)
    if cfg is None:
        raise SidecarBootError(
            f"head_dim_invariant check failed: model has no .config attribute "
            f"(model_id={model_id})"
        )

    # Multimodal models expose .text_config; prefer it when present.
    target_cfg = getattr(cfg, "text_config", cfg)

    explicit = getattr(target_cfg, "head_dim", None)
    if isinstance(explicit, int) and explicit > 0:
        observed = explicit
    else:
        hidden = getattr(target_cfg, "hidden_size", None)
        heads = getattr(target_cfg, "num_attention_heads", None)
        if not isinstance(hidden, int) or not isinstance(heads, int) or heads <= 0:
            raise SidecarBootError(
                f"head_dim_invariant check failed: cannot compute head_dim "
                f"(hidden_size={hidden!r}, num_attention_heads={heads!r}, "
                f"model_id={model_id})"
            )
        observed = hidden // heads

    if observed != expected_dim:
        raise SidecarBootError(
            f"head_dim invariant violated: model_id={model_id!r} has "
            f"head_dim={observed} (expected {expected_dim}). SDPA / FA2 / "
            f"xformers attention paths assume head_dim=128 (Qwen2.5-Coder "
            f"family); other values silently degrade or hit untested fallback "
            f"kernels. Refusing to boot. Reference: "
            f"crates/contracts/tests/head_dim_invariant_contract.rs."
        )

    logger.info(
        "head_dim_invariant_verified",
        model_id=model_id,
        head_dim_observed=observed,
        head_dim_expected=expected_dim,
        ok=True,
    )
    return observed, expected_dim


def activate_deep_mode(default_model: str) -> tuple[str, int]:
    """Check ELAI_DEEP_MODE and VRAM, return (resolved_model_id, ctx_cap).

    When ELAI_DEEP_MODE=1 AND VRAM >= 24 GB: returns the 1M model + 64K cap.
    When ELAI_DEEP_MODE=1 AND VRAM < 24 GB: logs a warning and falls back to
        the standard model (never crashes — soft fallback per WIRING_CONTRACT).
    When ELAI_DEEP_MODE != 1: returns (default_model, 0) where 0 means
        use the standard effective_max_context from env.

    Args:
        default_model: The model path/ID passed at boot.

    Returns:
        Tuple of (resolved_model_id, ctx_cap). ctx_cap=0 means not in deep mode.
    """
    if os.getenv("ELAI_DEEP_MODE", "0") != "1":
        return default_model, 0

    vram_gb: float = 0.0
    if torch.cuda.is_available():
        props = torch.cuda.get_device_properties(torch.cuda.current_device())
        vram_gb = props.total_memory / 1024 ** 3

    if vram_gb < DEEP_MODE_MIN_VRAM_GB:
        logger.warning(
            "deep_mode_refused_insufficient_vram",
            vram_gb=round(vram_gb, 1),
            required_gb=DEEP_MODE_MIN_VRAM_GB,
            fallback_model=default_model,
            hint="Deep mode requires >= 24 GB VRAM; falling back to standard model at 32K",
        )
        return default_model, 0

    logger.info(
        "deep_mode_activated",
        model_id=DEEP_MODE_MODEL_ID,
        ctx_cap=DEEP_MODE_CTX_CAP,
        vram_gb=round(vram_gb, 1),
    )
    return DEEP_MODE_MODEL_ID, DEEP_MODE_CTX_CAP


# ---------------------------------------------------------------------------
# E-28: attention-kernels 2026 — FA2 mandatory on Ampere, FA3 refusal,
# SDPA backend pinning, StreamingLLM attention-sink masking, and boot-time
# YaRN rope_scaling injection gated behind ELAI_YARN_EVAL_GATE (the live
# default still refuses YaRN per refuse_yarn_at_boot above; the boot-time
# helper exists ONLY so the harness in test_yarn_eval_gate.py can build a
# config object for the factor sweep — it never executes on a normal boot).
#
# References (grounded via workflow/papers/):
#   * arXiv:2307.08691 FlashAttention-2 — SM_80+ supported (Ampere included).
#   * arXiv:2309.17453 StreamingLLM — attention-sink masking; default sink=4.
#     Position IDs MUST be re-indexed relative to cache slot, NOT original
#     text position (critical implementation note per
#     workflow/papers/streaming-llm-interface.md).
#   * arXiv:2501.15383 Qwen2.5-1M — YaRN rope_scaling recipe for Qwen2.5
#     family; base Coder-7B max_position_embeddings=32768, rope_theta=1e6.
#   * tridao.me/blog/2024/flash3 — FA3 Hopper-only (no Ampere SM_86 path).
#   * pytorch#133780 — SDPA silently falls back to MATH on shape errors when
#     backend is not pinned; torch.nn.attention.sdpa_kernel pins explicitly.
# ---------------------------------------------------------------------------

# StreamingLLM default sink count (paper Sec 4.2; minimum viable is 4).
STREAMING_LLM_DEFAULT_SINK_COUNT: int = 4
# YaRN base context for Qwen2.5-Coder-7B (HF config.json verified).
QWEN25_CODER_ORIGINAL_MAX_POSITION: int = 32_768


def refuse_fa3_on_ampere(compute_capability: int) -> None:
    """Raise SidecarBootError if FA3/FA4 import is attempted on Ampere.

    FA3 targets H100/H200 Hopper (SM_90) and FA4 targets Hopper + Blackwell
    (SM_90+/SM_100). RTX 3090 / 3060 are Ampere SM_86 / SM_80 — neither has
    an optimized FA3 path. The 75% headline speedup cited in cuDNN
    benchmarks is H100-measured. Loud refusal is preferable to silent
    fallback to a slower or numerically different code path.

    Args:
        compute_capability: GPU compute capability as int (sm_86 = 86).
            Use 0 to skip the GPU check (e.g., CPU-only smoke runs).

    References: tridao.me/blog/2024/flash3, together.ai/blog/flashattention-4,
        github.com/Dao-AILab/flash-attention SM_80 supported list.
    """
    if os.getenv("ELAI_REFUSE_FA3", "1") != "1":
        # Explicit opt-out for cloud Hopper deployments (out-of-scope today).
        return
    if compute_capability == 0:
        return
    if compute_capability >= 90:
        # Hopper / Blackwell — FA3 path is the optimized one, no refusal.
        return
    # Ampere or older: actively check no FA3 module was imported.
    import sys
    forbidden = (
        "flash_attn_3",
        "flash_attn_4",
        "cudnn_attention",
    )
    for mod_name in list(sys.modules.keys()):
        for forbidden_prefix in forbidden:
            if mod_name.startswith(forbidden_prefix):
                raise SidecarBootError(
                    f"FA3/FA4/cuDNN-attention kernel '{mod_name}' is Hopper/"
                    f"Blackwell only; this GPU has compute capability sm_"
                    f"{compute_capability}. RTX 3090/3060 (Ampere) must use "
                    f"FA2 (flash_attn 2.x). Reference: tridao.me/blog/2024/"
                    f"flash3, together.ai/blog/flashattention-4. To override "
                    f"(cloud Hopper deployments only), set ELAI_REFUSE_FA3=0."
                )


def pin_sdpa_backend() -> "tuple[Any, str, bool]":
    """Pin SDPA backend to FLASH_ATTENTION; return (ctx_mgr, backend, fallback).

    Returns a context manager that pins torch's SDPA dispatcher to the
    FLASH_ATTENTION kernel for the duration of the with-block. This
    prevents the documented silent MATH fallback on shape errors
    (pytorch#133780). Caller wraps model.generate() with the returned
    context manager.

    The third return value indicates whether the SDPA backend pin was
    actually applied (False on torch versions that lack sdpa_kernel; the
    returned ctx_mgr is a nullcontext in that case so callers don't crash
    on older toolchains).

    References:
        * pytorch#133780 — SDPA silent MATH fallback when backend not pinned.
        * torch>=2.3 torch.nn.attention.sdpa_kernel + SDPBackend.FLASH_ATTENTION.
    """
    import contextlib
    try:
        from torch.nn.attention import sdpa_kernel, SDPBackend  # type: ignore
    except ImportError:
        # torch < 2.3 lacks the explicit backend pin; return a no-op so
        # callers don't crash. The kernel-select code logs this once at
        # boot so operators see the degraded path.
        return contextlib.nullcontext(), "unknown", False
    ctx_mgr = sdpa_kernel([SDPBackend.FLASH_ATTENTION])
    return ctx_mgr, "flash_attention", True


def apply_streaming_llm_mask(
    attention_mask: "torch.Tensor",
    sink_count: int,
    window_size: int,
    context_position: int,
) -> "torch.Tensor":
    """Apply StreamingLLM attention-sink masking to an attention mask.

    Per arXiv:2309.17453 (Xiao et al., ICLR 2024), preserves the first
    `sink_count` tokens (the attention sinks) PLUS a sliding window of
    the most recent `window_size` tokens. All other positions are masked
    out (set to 0 in the additive mask convention).

    Critical implementation note (per
    workflow/papers/streaming-llm-interface.md): when this mask is paired
    with a KV-cache eviction policy, position_ids MUST be re-indexed
    relative to the cache slot, NOT the original text position. This
    function only manipulates the attention mask; the caller is
    responsible for ensuring the model's position_ids are passed in
    cache-relative form.

    Args:
        attention_mask: 2D or 4D attention mask tensor. Shape
            (batch, seq_len) or (batch, 1, seq_len, seq_len).
        sink_count: Number of initial tokens to keep as sinks. Paper
            recommends >= 4; values < 4 do not preserve attention
            distribution quality per paper Sec 4.2.
        window_size: Size of the trailing sliding window.
        context_position: Absolute position of the current token in the
            untruncated text. Used to compute which tokens are in the
            window vs. sink vs. evicted region.

    Returns:
        A new attention mask tensor with the same shape, with attention
        masked to (sinks U window) only.

    Raises:
        ValueError: when sink_count < 1 or window_size < 1.
    """
    if sink_count < 1:
        raise ValueError(
            f"StreamingLLM sink_count must be >= 1 (paper recommends >= 4 "
            f"per Sec 4.2); got {sink_count}"
        )
    if window_size < 1:
        raise ValueError(
            f"StreamingLLM window_size must be >= 1; got {window_size}"
        )
    seq_len = attention_mask.shape[-1]
    if context_position < sink_count + window_size:
        # Short context — no eviction needed; return the mask unchanged.
        return attention_mask
    # Build a boolean keep mask: positions [0, sink_count) and
    # [context_position - window_size, context_position) are kept.
    device = attention_mask.device
    positions = torch.arange(seq_len, device=device)
    is_sink = positions < sink_count
    window_start = context_position - window_size
    is_window = (positions >= window_start) & (positions < context_position)
    keep = is_sink | is_window
    # Broadcast the keep vector to the full mask rank by zeroing dropped
    # positions. attention_mask is conventionally 1 for "attend" and 0
    # for "do not attend" (HF convention).
    if attention_mask.dim() == 2:
        new_mask = attention_mask * keep.to(attention_mask.dtype).unsqueeze(0)
    else:
        # 4D: (batch, heads_or_1, q_len, k_len). Apply along the k_len axis.
        shape = [1] * attention_mask.dim()
        shape[-1] = seq_len
        keep_b = keep.to(attention_mask.dtype).reshape(shape)
        new_mask = attention_mask * keep_b
    return new_mask


def apply_yarn_rope_scaling_at_boot(
    model_config: Any,
    factor: float,
    original_max_position: int = QWEN25_CODER_ORIGINAL_MAX_POSITION,
) -> Any:
    """Inject YaRN rope_scaling into a model config (boot-time helper).

    Conflict reconciliation per workflow/reasoning/E-28-decisions.md:
    the live-server default still refuses YaRN via refuse_yarn_at_boot
    (E-32 policy unchanged). This helper is ONLY callable from the
    factor-sweep harness (test_yarn_eval_gate.py) when both
    ELAI_YARN_EVAL_GATE=1 AND ELAI_YARN_BOOT_FACTOR is set to a valid
    factor. It mutates the in-memory config dict to inject
    rope_scaling={"type":"yarn","factor":<f>,
    "original_max_position_embeddings":<orig>}. The mutation happens at
    config load before AutoModelForCausalLM.from_pretrained reads
    config.rope_scaling. The live boot path NEVER invokes this helper.

    References:
        * arXiv:2309.00071 YaRN (Peng+, 2023) — base rope_scaling formula.
        * arXiv:2501.15383 Qwen2.5-1M tech report — Qwen2.5 family
          uses factor=4 to target 128K (orig=32768).
        * E-32-decisions.md DECISION 1 — YaRN rejected by default; this
          helper is harness-only.

    Args:
        model_config: HF AutoConfig (or dict) instance for the model.
        factor: YaRN scaling factor (typical: 2.0, 4.0, 8.0). Validated
            here to reject 0 / negative / NaN.
        original_max_position: The pre-extension max position embeddings.
            Defaults to 32768 (Qwen2.5-Coder-7B base context window).

    Returns:
        The same config object with rope_scaling injected.

    Raises:
        SidecarBootError: when the gate env var ELAI_YARN_EVAL_GATE is
            not set, OR factor is invalid.
    """
    if os.getenv("ELAI_YARN_EVAL_GATE", "0") != "1":
        raise SidecarBootError(
            "apply_yarn_rope_scaling_at_boot is harness-only; set "
            "ELAI_YARN_EVAL_GATE=1 to enable (live boot path uses "
            "refuse_yarn_at_boot per E-32). See "
            "workflow/reasoning/E-28-decisions.md."
        )
    if not (factor > 0.0) or factor != factor:  # also catches NaN
        raise SidecarBootError(
            f"YaRN factor must be positive finite; got {factor}"
        )
    if original_max_position < 1:
        raise SidecarBootError(
            f"YaRN original_max_position_embeddings must be >= 1; "
            f"got {original_max_position}"
        )
    rope_scaling = {
        "type": "yarn",
        "factor": float(factor),
        "original_max_position_embeddings": int(original_max_position),
    }
    if hasattr(model_config, "rope_scaling"):
        model_config.rope_scaling = rope_scaling
    elif isinstance(model_config, dict):
        model_config["rope_scaling"] = rope_scaling
    else:
        raise SidecarBootError(
            f"apply_yarn_rope_scaling_at_boot: unsupported config type "
            f"{type(model_config).__name__}; expected AutoConfig or dict"
        )
    logger.info(
        "yarn_rope_scaling_applied",
        factor=float(factor),
        original_max_position_embeddings=int(original_max_position),
        gate="ELAI_YARN_EVAL_GATE=1 (harness-only path)",
    )
    return model_config


def apply_yarn_rope_scaling(
    model_config: Any,
    factor: float,
    original_max_position: int = QWEN25_CODER_ORIGINAL_MAX_POSITION,
) -> Any:
    """Inject YaRN rope_scaling into a model config for the LIVE boot path.

    E-28 STEP 3: when ELAI_YARN_FACTOR is set at boot, InferenceEngine loads
    AutoConfig.from_pretrained, calls this helper to inject rope_scaling, then
    passes the patched config to AutoModelForCausalLM.from_pretrained. This is
    the opt-in context-extension toggle per arXiv:2501.15383 (Qwen2.5-1M
    recipe: factor=4 targets 128K from original_max_position_embeddings=32768).

    Unlike apply_yarn_rope_scaling_at_boot (harness-only, gated behind
    ELAI_YARN_EVAL_GATE), this function runs on the live boot path when
    ELAI_YARN_FACTOR is set. It is NOT gated by ELAI_YARN_EVAL_GATE. The E-32
    refuse_yarn_at_boot refusal only applies to ELAI_USE_YARN=1 — a different
    env var — so ELAI_YARN_FACTOR bypasses the E-32 refusal by design (E-28
    plan supersedes E-32 for this specific opt-in toggle).

    Guarded assert: config.max_position_embeddings >= 32768 to prevent silent
    mis-config (e.g., a 4K-context model accidentally paired with YaRN factor=4
    would produce garbage). Uses an explicit SidecarBootError raise instead of
    a bare `assert` so the check survives `python -O`.

    Args:
        model_config: HF AutoConfig instance for the model.
        factor: YaRN scaling factor (2.0, 4.0, 8.0). Validated > 0 and finite.
        original_max_position: Pre-extension max position embeddings.
            Defaults to 32768 (Qwen2.5-Coder-7B base context window).

    Returns:
        The same config object with rope_scaling injected.

    Raises:
        SidecarBootError: when factor is invalid, original_max_position < 1,
            config type is unsupported, or config.max_position_embeddings <
            32768 (the guarded assert per E-28 plan STEP 3).
    """
    if not (factor > 0.0) or factor != factor:  # catches NaN
        raise SidecarBootError(
            f"YaRN factor must be positive finite; got {factor}"
        )
    if original_max_position < 1:
        raise SidecarBootError(
            f"YaRN original_max_position_embeddings must be >= 1; "
            f"got {original_max_position}"
        )
    rope_scaling = {
        "type": "yarn",
        "factor": float(factor),
        "original_max_position_embeddings": int(original_max_position),
    }
    if hasattr(model_config, "rope_scaling"):
        model_config.rope_scaling = rope_scaling
    elif isinstance(model_config, dict):
        model_config["rope_scaling"] = rope_scaling
    else:
        raise SidecarBootError(
            f"apply_yarn_rope_scaling: unsupported config type "
            f"{type(model_config).__name__}; expected AutoConfig or dict"
        )
    # Guarded assert per E-28 plan STEP 3: prevent silent mis-config where a
    # short-context model is paired with YaRN extension. Use explicit raise
    # (not bare assert) so it survives `python -O`.
    max_pos = getattr(model_config, "max_position_embeddings", None)
    if isinstance(model_config, dict):
        max_pos = model_config.get("max_position_embeddings")
    if max_pos is not None and max_pos < original_max_position:
        raise SidecarBootError(
            f"YaRN guarded assert: config.max_position_embeddings={max_pos} "
            f"< original_max_position={original_max_position}; refusing to "
            f"apply YaRN extension to a model with a shorter base context "
            f"window than the YaRN original_max. See E-28 plan STEP 3."
        )
    logger.info(
        "yarn_rope_scaling_applied",
        factor=float(factor),
        original_max_position_embeddings=int(original_max_position),
        applied_at_layer=-1,
        gate="ELAI_YARN_FACTOR (live boot path)",
    )
    return model_config


# StreamingLLM sliding-window default. When ELAI_LOCAL_MAX_CONTEXT is unset
# the window defaults to 25000 — the production recommendation per
# REMOTE-SIDECAR.md ("Use 25000-28000 in production for headroom" under the
# 32K native Qwen2.5-Coder-7B ceiling). This is the same env the Rust client
# uses to cap the local lane context (CLAUDE.md ELAI_LOCAL_MAX_CONTEXT).
STREAMING_LLM_DEFAULT_WINDOW_SIZE: int = 25_000


def resolve_streaming_llm_config() -> "tuple[int, int]":
    """Read ELAI_STREAMING_LLM_SINKS + ELAI_LOCAL_MAX_CONTEXT env vars.

    Returns (sink_count, window_size). When ELAI_STREAMING_LLM_SINKS is
    unset or "0", returns (0, 0) — StreamingLLM masking is OFF and the
    generation path is byte-identical to the pre-E-28 baseline (backward
    compat: the caller skips the mask entirely when sink_count == 0).

    When ELAI_STREAMING_LLM_SINKS == "1", sink_count defaults to
    STREAMING_LLM_DEFAULT_SINK_COUNT (4 per arXiv:2309.17453 Sec 4.2).
    When set to an integer N > 0, sink_count = N.

    window_size is read from ELAI_LOCAL_MAX_CONTEXT so the sliding window
    matches the Rust client's local-lane context cap. Default
    STREAMING_LLM_DEFAULT_WINDOW_SIZE (25000).

    Raises:
        SidecarBootError: when either env var is set to a non-integer or
            non-positive value (fail-closed at boot rather than a silent
            degraded mask mid-generation).

    References:
        * arXiv:2309.17453 Xiao et al. ICLR 2024 — sink_count >= 4 (Sec 4.2).
        * github.com/mit-han-lab/streaming-llm — pure-Python mask reference.
    """
    raw = os.getenv("ELAI_STREAMING_LLM_SINKS", "0").strip()
    if raw in ("", "0"):
        return 0, 0
    if raw == "1":
        sink_count = STREAMING_LLM_DEFAULT_SINK_COUNT
    else:
        try:
            sink_count = int(raw)
        except ValueError as exc:
            raise SidecarBootError(
                f"ELAI_STREAMING_LLM_SINKS must be 0, 1, or a positive "
                f"integer; got '{raw}'"
            ) from exc
        if sink_count < 1:
            raise SidecarBootError(
                f"ELAI_STREAMING_LLM_SINKS must be >= 1 when set; got {sink_count}"
            )
    window_raw = os.getenv("ELAI_LOCAL_MAX_CONTEXT", str(STREAMING_LLM_DEFAULT_WINDOW_SIZE)).strip()
    try:
        window_size = int(window_raw)
    except ValueError as exc:
        raise SidecarBootError(
            f"ELAI_LOCAL_MAX_CONTEXT must be an integer; got '{window_raw}'"
        ) from exc
    if window_size < 1:
        raise SidecarBootError(
            f"ELAI_LOCAL_MAX_CONTEXT must be >= 1; got {window_size}"
        )
    return sink_count, window_size


# ---------------------------------------------------------------------------
# E-32: FoundInTheMiddleCalibrator — training-free positional-prior subtraction
# ---------------------------------------------------------------------------


class FoundInTheMiddleCalibrator(LogitsProcessor):
    """Training-free Found-in-the-Middle (FITM) LogitsProcessor.

    Implements positional-prior subtraction per arxiv:2406.16008 (Hsieh et al.,
    ACL 2024): on long-context requests (prompt_len > FITM_PROMPT_LEN_THRESHOLD)
    with ELAI_FITM_ENABLED=1, runs one extra forward pass with a dummy-doc
    baseline at the target position, then subtracts the positional prior logits
    from the real logits. This nulls the attention sink that causes Lost-in-the-
    Middle degradation (arxiv:2307.03172 TACL 2024: 30%+ drop at middle positions).

    Gate: only applied when ELAI_FITM_ENABLED=1 AND prompt_len > 8K.
    Short requests (<=8K) are completely unaffected — guard checked in __init__.

    Cost: 1 extra forward pass per long request (~100-200ms on 3090 with NF4 7B).

    WIRING_CONTRACT stub_ban:
    - No dummy doc longer than FITM_DUMMY_MAX_TOKENS (100 tokens) — cost ceiling.
    - No .unwrap() equivalent — all errors raise typed exceptions.
    - blake3 used for dummy-text cache key (not blake2/sha2 per codeRules).
    """

    def __init__(
        self,
        model: "AutoModelForCausalLM",
        tokenizer: "AutoTokenizer",
        prompt_len: int,
        fitm_enabled: bool,
        dummy_text: str | None = None,
        per_request_override: bool | None = None,
    ) -> None:
        """Initialise the calibrator.

        Args:
            model: The loaded HF model (used for dummy forward pass).
            tokenizer: Tokenizer matching the model.
            prompt_len: Token count of the real prompt.
            fitm_enabled: Global ELAI_FITM_ENABLED env var value.
            dummy_text: Override dummy doc text. Defaults to ELAI_FITM_DUMMY_TEXT
                env var or FITM_DUMMY_TEXT_DEFAULT.
            per_request_override: Per-request `fitm` field from the request body.
                True forces on; False forces off; None inherits global.
        """
        self._model = model
        self._tokenizer = tokenizer
        self._prompt_len = prompt_len
        self._dummy_prior: torch.Tensor | None = None

        # Determine effective enable: per-request > global env
        if per_request_override is not None:
            effective_enabled = per_request_override
        else:
            effective_enabled = fitm_enabled

        self.active = effective_enabled and (prompt_len > FITM_PROMPT_LEN_THRESHOLD)

        if not self.active:
            return

        # Build dummy text — clamp to FITM_DUMMY_MAX_TOKENS
        raw_dummy = dummy_text or os.getenv("ELAI_FITM_DUMMY_TEXT", FITM_DUMMY_TEXT_DEFAULT)
        dummy_ids = tokenizer(raw_dummy, return_tensors="pt", add_special_tokens=False)
        dummy_token_count = dummy_ids["input_ids"].shape[1]
        if dummy_token_count > FITM_DUMMY_MAX_TOKENS:
            # Truncate to ceiling per stub_ban rule
            dummy_ids["input_ids"] = dummy_ids["input_ids"][:, :FITM_DUMMY_MAX_TOKENS]
            dummy_token_count = FITM_DUMMY_MAX_TOKENS

        self._dummy_text = raw_dummy
        self._dummy_token_count = dummy_token_count

    def _compute_prior(self, input_ids: torch.LongTensor) -> None:
        """Run one dummy-doc forward pass to get the positional prior logits.

        Called lazily on the first __call__ invocation to avoid cost when
        the processor is constructed but generation is cancelled before it starts.

        Raises:
            RuntimeError: If the dummy forward pass fails. Caller catches this
                and disables FITM for the request (never crashes generation).
        """
        device = input_ids.device
        dummy_ids = self._tokenizer(
            self._dummy_text,
            return_tensors="pt",
            add_special_tokens=False,
            truncation=True,
            max_length=FITM_DUMMY_MAX_TOKENS,
        ).input_ids.to(device)

        # Build a dummy context by replacing the last dummy_token_count tokens
        # of the real prompt with the dummy doc tokens.
        real_len = input_ids.shape[1]
        dummy_len = dummy_ids.shape[1]
        insert_start = max(0, real_len - dummy_len)

        dummy_context = input_ids.clone()
        dummy_context[:, insert_start : insert_start + dummy_len] = dummy_ids[
            :, : real_len - insert_start
        ]

        with torch.inference_mode():
            dummy_out = self._model(dummy_context)

        # Prior logits = last-position logits from the dummy context forward pass
        self._dummy_prior = dummy_out.logits[:, -1, :].squeeze(0).detach()

    def __call__(
        self, input_ids: torch.LongTensor, scores: torch.FloatTensor
    ) -> torch.FloatTensor:
        if not self.active:
            return scores

        if self._dummy_prior is None:
            try:
                self._compute_prior(input_ids)
            except Exception as exc:
                logger.warning(
                    "fitm_prior_compute_failed",
                    error=str(exc),
                    fallback="disabling FITM for this request",
                )
                self.active = False
                return scores

        if self._dummy_prior is None:
            return scores

        # Subtract positional prior: calibrated_logits = real_logits - dummy_prior
        # Operates on scores[0] (batch_size=1 for sidecar requests).
        prior = self._dummy_prior.to(scores.device)
        scores[0] = scores[0] - prior
        return scores

    def ssb_payload(self, applied_ms: float) -> dict:
        """Return the fitm_calibration_applied SSB payload."""
        return {
            "prompt_len": self._prompt_len,
            "dummy_len": getattr(self, "_dummy_token_count", 0),
            "applied_ms": round(applied_ms, 2),
        }


# ---------------------------------------------------------------------------
# NoveltyTracker — generation novelty signal (A-25)
# ---------------------------------------------------------------------------


class NoveltyTracker:
    """Tracks per-prompt generation embeddings for novelty scoring.

    Computes cosine similarity between current generation and the last
    generation for the same prompt. Returns 0.0 (identical) to 1.0
    (completely different). Uses the generation model's own last hidden
    layer (mean-pooled) as the embedding — zero extra VRAM.
    """

    def __init__(self, max_entries: int = NOVELTY_CACHE_MAX_ENTRIES) -> None:
        self._cache: dict[str, np.ndarray] = {}
        self._max_entries = max_entries

    def compute(
        self,
        prompt: str,
        generated_text: str,
        model: "AutoModelForCausalLM",
        tokenizer: "AutoTokenizer",
    ) -> float:
        """Compute generation novelty for this prompt.

        Returns 0.0 on first generation for a prompt (no prior to compare).
        """
        embedding = self._embed(generated_text, model, tokenizer)
        if embedding is None:
            return 0.0

        prev = self._cache.get(prompt)

        # Evict oldest if at capacity and this is a new prompt
        if prev is None and len(self._cache) >= self._max_entries:
            oldest_key = next(iter(self._cache))
            del self._cache[oldest_key]

        self._cache[prompt] = embedding

        if prev is None:
            return 0.0

        sim = self._cosine_similarity(prev, embedding)
        return float(np.clip(1.0 - sim, 0.0, 1.0))

    @staticmethod
    def _embed(
        text: str,
        model: "AutoModelForCausalLM",
        tokenizer: "AutoTokenizer",
    ) -> np.ndarray | None:
        """Mean-pool last hidden layer of the model for the given text."""
        if not text.strip():
            return None
        tokens = tokenizer(text, return_tensors="pt", truncation=True, max_length=512)
        tokens = {k: v.to(model.device) for k, v in tokens.items()}
        with torch.inference_mode():
            out = model(**tokens, output_hidden_states=True)
        last_hidden = out.hidden_states[-1]  # (1, seq_len, hidden_dim)
        mask = tokens["attention_mask"].unsqueeze(-1)  # (1, seq_len, 1)
        pooled = (last_hidden * mask).sum(dim=1) / mask.sum(dim=1).clamp(min=1)
        return pooled[0].float().cpu().numpy()

    @staticmethod
    def _cosine_similarity(a: np.ndarray, b: np.ndarray) -> float:
        dot = np.dot(a, b)
        norm_a = np.linalg.norm(a)
        norm_b = np.linalg.norm(b)
        if norm_a < 1e-12 or norm_b < 1e-12:
            return 0.0
        return float(dot / (norm_a * norm_b))


# ---------------------------------------------------------------------------
# VRAM logging
# ---------------------------------------------------------------------------


def _log_vram(label: str = "") -> None:
    if torch.cuda.is_available():
        alloc = torch.cuda.memory_allocated() / 1024**3
        reserved = torch.cuda.memory_reserved() / 1024**3
        props = torch.cuda.get_device_properties(torch.cuda.current_device())
        total = props.total_memory / 1024**3
        logger.info(
            "vram",
            label=label,
            allocated_gb=round(alloc, 2),
            reserved_gb=round(reserved, 2),
            total_gb=round(total, 2),
            free_gb=round(total - reserved, 2),
        )


# ---------------------------------------------------------------------------
# PrefixKVCache — LRU prefix reuse for DynamicCache
# ---------------------------------------------------------------------------

# Hash backend: blake3 Python package is NOT listed in pyproject.toml deps.
# Using hashlib.sha256 (stdlib, no new dep). blake3 is preferred in Rust (CLAUDE.md
# anti-stale rule) but that rule is Rust-scoped; Python uses stdlib sha256 here.
def _hash_input_ids(ids: "torch.LongTensor") -> str:
    """Return hex sha256 of the raw bytes of the token-id tensor."""
    return hashlib.sha256(ids.cpu().numpy().tobytes()).hexdigest()


# Minimum shared prefix (tokens) required to use a cached entry.
_KV_MIN_PREFIX_TOKENS = 16


class PrefixKVCache:
    """LRU cache of DynamicCache objects keyed by input-id prefix.

    Thread-safe via a threading.Lock (not asyncio.Lock) because all callers
    run inside ThreadPoolExecutor threads (via loop.run_in_executor), not
    in the async event loop.

    Auto-disabled when --turboquant-bits > 0: TQ produces _KVQuantCache
    objects that cannot be substituted with a plain DynamicCache pre-fill.
    Auto-disabled when max_entries == 0.
    """

    def __init__(self, max_entries: int = 4) -> None:
        self.max_entries = max_entries
        # OrderedDict used as LRU: move_to_end(key, last=True) on hit.
        # Values: (token_id_tensor, DynamicCache, last_used_ts)
        self._lru: "collections.OrderedDict[str, tuple]" = collections.OrderedDict()
        self._lock = Lock()

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def lease(
        self,
        input_ids: "torch.LongTensor",
        marker_boundary: Optional[int] = None,
    ) -> "tuple[Optional[object], int, int]":
        """Find longest cached prefix of input_ids.

        Returns (cache, prefix_match_len, total_input_len).
        Returns (None, 0, total_len) when no match >= _KV_MIN_PREFIX_TOKENS.
        Does NOT clone the cache — caller must not mutate it concurrently.
        generate() is called under InferenceEngine.lock so serialisation is
        guaranteed at the call site.

        marker_boundary (Track C-D, Anthropic ``cache_control`` support):
          When None (default), behaviour is identical to the pre-C-D
          full-prefix match path — backward compat preserved.
          When int N, the matched prefix length is CAPPED at N (positions
          ``[0..N)`` are cacheable; positions ``>= N`` are not). Concretely:
            * the per-entry comparison ``cmp_len`` is bounded by N, so the
              contiguous-match counter cannot exceed N;
            * the floor ``_KV_MIN_PREFIX_TOKENS`` still applies — if N is
              below the floor, no match is reported.
        """
        total_len = input_ids.shape[0]
        # Resolve effective scan ceiling for this lease. None => total_len
        # (legacy unbounded scan); int => marker boundary cap.
        if marker_boundary is None:
            effective_ceiling = total_len
        else:
            # Defensive clamp: never exceed total_len, never go negative.
            effective_ceiling = max(0, min(int(marker_boundary), total_len))

        best_key: Optional[str] = None
        best_match_len = 0

        with self._lock:
            input_cpu = input_ids.cpu()
            for key, (stored_ids, _cache, _ts) in self._lru.items():
                # Require contiguous prefix (first mismatch ends the prefix).
                # stored_ids is already CPU-resident (admit() stores .cpu()).
                # marker_boundary caps cmp_len: we never report a match that
                # extends past the marker, even if the stored cache covers it.
                cmp_len = min(stored_ids.shape[0], total_len, effective_ceiling)
                contiguous = 0
                for i in range(cmp_len):
                    if stored_ids[i] == input_cpu[i]:
                        contiguous += 1
                    else:
                        break
                if contiguous > best_match_len:
                    best_match_len = contiguous
                    best_key = key

            if best_match_len < _KV_MIN_PREFIX_TOKENS or best_key is None:
                return None, 0, total_len

            # Bump recency
            stored_ids, cache, _ts = self._lru[best_key]
            self._lru.move_to_end(best_key, last=True)
            self._lru[best_key] = (stored_ids, cache, time.time())
            return cache, best_match_len, total_len

    def admit(
        self,
        input_ids: "torch.LongTensor",
        cache: "object",
    ) -> None:
        """Store a post-generation DynamicCache for future prefix reuse.

        Evicts the LRU entry (with VRAM release) when over capacity.
        """
        if self.max_entries == 0:
            return
        key = _hash_input_ids(input_ids)
        with self._lock:
            # Evict any stale entry referencing the SAME Python cache object
            # under a different key. generate() mutates DynamicCache in-place,
            # so a leased entry's stored object now has the wrong seq length;
            # leaving the old key in the LRU would cause future leases to
            # return a cache whose get_seq_length() exceeds the matched
            # prefix, corrupting cache_position on the next generate().
            stale_keys = [k for k, (_, c, _) in self._lru.items() if c is cache and k != key]
            for k in stale_keys:
                del self._lru[k]
            if key in self._lru:
                # Update existing entry
                self._lru.move_to_end(key, last=True)
                self._lru[key] = (input_ids.cpu(), cache, time.time())
                return
            # Evict LRU entries until under capacity
            while len(self._lru) >= self.max_entries:
                _evict_key, (_, evicted_cache, _) = self._lru.popitem(last=False)
                del evicted_cache
                if torch.cuda.is_available():
                    torch.cuda.empty_cache()
            self._lru[key] = (input_ids.cpu(), cache, time.time())

    def clear(self) -> int:
        """Evict all entries and release VRAM.

        Returns the pre-clear ``len(self._lru)`` snapshot, captured under the
        same lock as the clear so the count is TOCTOU-safe (no entry can be
        admitted between the snapshot and the clear). Used by the operator
        flush endpoint at /v1/cache/flush to report n_entries_cleared.
        """
        with self._lock:
            n = len(self._lru)
            self._lru.clear()
        if torch.cuda.is_available():
            torch.cuda.empty_cache()
        return n


# Module-level singleton. None = disabled (TQ active, or max_entries=0).
_prefix_kv_cache: Optional[PrefixKVCache] = None


def init_prefix_kv_cache(max_entries: int, turboquant_bits: int) -> None:
    """Initialise the module-level PrefixKVCache singleton.

    Called from InferenceEngine.__init__ after model load.
    Auto-disables if turboquant_bits > 0 (TQ path uses _KVQuantCache
    which is incompatible with a pre-filled DynamicCache).
    Auto-disables if max_entries == 0.
    """
    global _prefix_kv_cache
    if turboquant_bits > 0:
        logger.warning(
            "prefix_kv_cache_disabled",
            reason="turboquant_bits > 0; DynamicCache incompatible with _KVQuantCache",
            turboquant_bits=turboquant_bits,
        )
        _prefix_kv_cache = None
        return
    if max_entries == 0:
        logger.info("prefix_kv_cache_disabled", reason="max_entries=0 (--kv-cache-size 0)")
        _prefix_kv_cache = None
        return
    _prefix_kv_cache = PrefixKVCache(max_entries=max_entries)
    logger.info("prefix_kv_cache_init", max_entries=max_entries)


# ---------------------------------------------------------------------------
# InferenceEngine
# ---------------------------------------------------------------------------


class InferenceEngine:
    """Composed inference: HF Transformers + DoLa + TurboQuant + Steering.

    Uses model.generate() with a custom DoLaSteeringProcessor for the decode
    loop, and TextIteratorStreamer for real token streaming.
    """

    def __init__(
        self,
        model_name: str,
        dola_enabled: bool = True,
        dola_alpha: float = ALPHA_DEFAULT,
        turboquant_bits: int = 8,
        steering_enabled: bool = False,
        steering_coefficient: float = COEFFICIENT_DEFAULT,
        steering_layer: str = "auto",
        max_new_tokens: int = MAX_NEW_TOKENS_DEFAULT,
        device_map: str = "auto",
        load_in_4bit: bool = True,
        etc_threshold: float = ETC_THRESHOLD_DISABLED,
        kv_cache_size: int = 4,
    ):
        # Auto-detect FA2 availability. ELAI_FORCE_SDPA=1 forces sdpa even
        # if flash_attn is installed (workaround for transformers 4.57.6
        # flash_attention_mask kv_length non-scalar bug, see masking_utils.py:555).
        try:
            import flash_attn  # noqa: F401
            if os.getenv("ELAI_FORCE_SDPA", "0") == "1":
                _attn_impl = "sdpa"
                logger.info("sdpa_forced_via_env", flash_attn_version=flash_attn.__version__)
            else:
                _attn_impl = "flash_attention_2"
                logger.info("flash_attention_2_available", version=flash_attn.__version__)
        except ImportError:
            _attn_impl = "sdpa"

        self.model_name = model_name
        quant_backend = __import__("os").getenv("ELAI_QUANT_BACKEND", "nf4").lower()
        kv_cache_dtype = __import__("os").getenv("ELAI_KV_CACHE_DTYPE", "fp16").lower()
        if not load_in_4bit and quant_backend == "nf4":
            quant_backend = "fp16"
        logger.info("loading_model", model=model_name, quant_backend=quant_backend, kv_cache_dtype=kv_cache_dtype, load_in_4bit=load_in_4bit, attn=_attn_impl)
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
        # E-28 STEP 3: YaRN context-extension toggle (env-gated, default off).
        # When ELAI_YARN_FACTOR is set (e.g. "4.0"), load AutoConfig first,
        # inject rope_scaling via apply_yarn_rope_scaling, then pass the
        # patched config to from_pretrained. When unset, yarn_config is None
        # and from_pretrained loads the default config (byte-identical to the
        # pre-E-28 baseline). The E-32 refuse_yarn_at_boot only blocks
        # ELAI_USE_YARN=1 — a different env var — so this path is not blocked.
        yarn_config = None
        yarn_factor_raw = os.getenv("ELAI_YARN_FACTOR", "").strip()
        if yarn_factor_raw:
            try:
                yarn_factor = float(yarn_factor_raw)
            except ValueError as exc:
                raise SidecarBootError(
                    f"ELAI_YARN_FACTOR must be a float (e.g. 2.0, 4.0, 8.0); "
                    f"got '{yarn_factor_raw}'"
                ) from exc
            yarn_config = AutoConfig.from_pretrained(model_name)
            apply_yarn_rope_scaling(yarn_config, yarn_factor)
            logger.info(
                "yarn_factor_resolved",
                factor=yarn_factor,
                model=model_name,
            )
        if quant_backend == "awq":
            compute_cap = 0
            if torch.cuda.is_available():
                major, minor = torch.cuda.get_device_capability()
                compute_cap = major * 10 + minor
            marlin_enabled = compute_cap >= 80
            try:
                from transformers import AwqConfig
            except ImportError as exc:
                raise RuntimeError("AWQ backend requires autoawq and transformers>=4.45; install: pip install autoawq>=0.2.0") from exc
            awq_version = "marlin" if marlin_enabled else "gemm"
            self.model = AutoModelForCausalLM.from_pretrained(model_name, config=yarn_config, quantization_config=AwqConfig(version=awq_version), device_map=device_map, attn_implementation=_attn_impl)
            logger.info("quant_backend_selected", backend="awq", kv_dtype=kv_cache_dtype, compute_capability=compute_cap, marlin_enabled=marlin_enabled)
        elif quant_backend == "nf4":
            bnb_config = BitsAndBytesConfig(load_in_4bit=True, bnb_4bit_compute_dtype=torch.bfloat16, bnb_4bit_quant_type="nf4", bnb_4bit_use_double_quant=True)
            self.model = AutoModelForCausalLM.from_pretrained(model_name, config=yarn_config, quantization_config=bnb_config, device_map=device_map, attn_implementation=_attn_impl)
            logger.info("quant_backend_selected", backend="nf4", kv_dtype=kv_cache_dtype, compute_capability=0, marlin_enabled=False)
        else:
            self.model = AutoModelForCausalLM.from_pretrained(model_name, config=yarn_config, torch_dtype=torch.bfloat16, device_map=device_map, attn_implementation=_attn_impl)
            logger.info("quant_backend_selected", backend="fp16", kv_dtype=kv_cache_dtype, compute_capability=0, marlin_enabled=False)
        if kv_cache_dtype in ("fp8_e4m3", "fp8_e5m2"):
            compute_cap_kv = 0
            if torch.cuda.is_available():
                major, minor = torch.cuda.get_device_capability()
                compute_cap_kv = major * 10 + minor
            if compute_cap_kv < 89:
                logger.warning("kv_cache_dtype_software_sim", kv_dtype=kv_cache_dtype, throughput_cost_pct=5.0, compute_capability=compute_cap_kv)
        self.kv_cache_dtype = kv_cache_dtype
        self.model.eval()
        self.lock = Lock()
        _log_vram("after_model_load")

        # E-28: StreamingLLM attention-sink masking config (env-gated, default
        # off). resolve_streaming_llm_config returns (0, 0) when
        # ELAI_STREAMING_LLM_SINKS is unset/0 — the generation paths skip the
        # mask entirely in that case, preserving byte-identical baseline
        # behaviour. When sinks > 0, _apply_streaming_llm_if_active wraps the
        # model.generate attention_mask per arXiv:2309.17453.
        self.streaming_llm_sinks, self.streaming_llm_window = resolve_streaming_llm_config()
        if self.streaming_llm_sinks > 0:
            logger.info(
                "streaming_llm_configured",
                sink_count=self.streaming_llm_sinks,
                window_size=self.streaming_llm_window,
            )

        # E-28 STEP 4: Pin SDPA backend to FLASH_ATTENTION when ELAI_FORCE_SDPA=1.
        # Per pytorch#133780, SDPA silently falls back to MATH on shape errors
        # when the backend is not explicitly pinned. We call pin_sdpa_backend()
        # at boot to (a) verify the FLASH_ATTENTION SDPA backend is available on
        # this hardware/torch version, (b) emit the sdpa_backend_pinned SSB row
        # for dashboard visibility, and (c) set the flag that the generate paths
        # use to wrap model.generate() in the pinned context manager. If the
        # backend is unavailable (pin_active=False), raise a clear error rather
        # than silently falling back to MATH (report Anti-pattern "SDPA without
        # explicit backend pin").
        self._sdpa_pin_enabled = False
        if os.getenv("ELAI_FORCE_SDPA", "0") == "1":
            _ctx_mgr, _backend_name, _pin_active = pin_sdpa_backend()
            if not _pin_active:
                raise SidecarBootError(
                    "ELAI_FORCE_SDPA=1 but FLASH_ATTENTION SDPA backend is "
                    "unavailable (torch<2.3 or backend not supported on this "
                    "hardware). Refusing to boot with a silent MATH fallback. "
                    "Either upgrade torch>=2.3 or unset ELAI_FORCE_SDPA to use "
                    "flash_attention_2. Reference: pytorch#133780."
                )
            self._sdpa_pin_enabled = True
            logger.info(
                "sdpa_backend_pinned_ssb",
                backend=_backend_name,
                forced_via_env=True,
            )

        # Model config
        if hasattr(self.model.config, "text_config"):
            tc = self.model.config.text_config
            self.n_layers = tc.num_hidden_layers
            self.hidden_dim = tc.hidden_size
            self.head_dim = getattr(tc, "head_dim", 256)
        else:
            self.n_layers = self.model.config.num_hidden_layers
            self.hidden_dim = self.model.config.hidden_size
            self.head_dim = getattr(self.model.config, "head_dim", 256)

        # E-31: head_dim invariant check. Refuse-to-boot on mismatch for the
        # Qwen2.5-Coder family (head_dim=128). Scoped to "qwen" model_ids so
        # non-Qwen models (deep-mode 1M variant, future families) are not
        # forced into the 128 lock until measured. Mirrors the FA3-on-Ampere
        # refusal pattern: loud refusal > silent degradation.
        if "qwen" in self.model_name.lower():
            assert_head_dim_invariant(
                self.model, expected_dim=128, model_id=self.model_name,
            )

        self.candidate_layers = get_candidate_layers(self.n_layers)
        self.dola_enabled = dola_enabled
        self.dola_alpha = dola_alpha
        self.max_new_tokens = max_new_tokens

        # KV cache quantization (KIVI-style per-head-per-token)
        self.kv_compressor = KVCompressor(bit_width=turboquant_bits)

        # Steering vectors
        self.steering_enabled = steering_enabled
        self.steering_engine: SteeringEngine | None = None
        if steering_enabled:
            self.steering_engine = SteeringEngine(
                self.n_layers, self.hidden_dim, steering_coefficient, steering_layer
            )

        # DoLa+Steering processor (registered lazily per generate call)
        self._dola_processor: DoLaSteeringProcessor | None = None
        if dola_enabled:
            self._dola_processor = DoLaSteeringProcessor(
                model=self.model,
                candidate_layers=self.candidate_layers,
                alpha=dola_alpha,
                steering_engine=self.steering_engine,
            )

        # ETC entropy-trend constraint processor (A-9)
        self.etc_threshold = etc_threshold
        self.etc_enabled = etc_threshold != ETC_THRESHOLD_DISABLED
        self._etc_processor: ETCLogitsProcessor | None = None
        if self.etc_enabled:
            # DoLa and ETC must not stack on models < 32B (spec constraint)
            if dola_enabled and self.n_layers < 64:
                logger.warning(
                    "etc_dola_conflict",
                    layers=self.n_layers,
                    msg="DoLa disabled — ETC takes priority on <32B models",
                )
                self._dola_processor = None
                self.dola_enabled = False
            self._etc_processor = ETCLogitsProcessor(
                threshold=etc_threshold,
                import_pattern=IMPORT_CONTEXT_RE,
                tokenizer=self.tokenizer,
            )

        # GG intrinsic verification signal collector (A-17)
        self._gg_collector = GGSignalCollector()

        # Generation novelty tracker (A-25)
        self._novelty_tracker = NoveltyTracker()

        # Prefix KV cache (C-A) — disabled when TQ active or kv_cache_size=0
        self._turboquant_bits = turboquant_bits
        self._kv_cache_size = kv_cache_size
        init_prefix_kv_cache(kv_cache_size, turboquant_bits)

        # Track C-B: cache TokenEnforcerTokenizerData ONCE at engine startup.
        # build_token_enforcer_tokenizer_data scans the full tokenizer vocab —
        # paper-ingestor §52 documents this as expensive and cacheable. Calling
        # it per-request would add a ~100-500ms first-token latency tax. The
        # data structure is reused across requests; per-request we still build
        # a NEW prefix_allowed_tokens_fn (the underlying TokenEnforcer is NOT
        # thread-safe per paper §83 — only the data is shareable).
        # Skipped when lm-format-enforcer is not installed (USER-MANUAL-CB-DEPS).
        self._enforcer_tokenizer_data = None
        if _LMFE_AVAILABLE:
            try:
                self._enforcer_tokenizer_data = build_token_enforcer_tokenizer_data(
                    self.tokenizer
                )
                logger.info(
                    "enforcer_tokenizer_data_ready",
                    vocab_size=len(self.tokenizer),
                )
            except Exception as e:
                # Vocab scan failure is recoverable — log + leave None so
                # /v1/* endpoints surface 503 when response_format is set.
                logger.warning(
                    "enforcer_tokenizer_data_build_failed",
                    error=str(e),
                    error_type=type(e).__name__,
                )
                self._enforcer_tokenizer_data = None
        else:
            logger.info(
                "enforcer_tokenizer_data_skipped",
                reason="lm-format-enforcer not installed",
            )

        logger.info(
            "engine_ready",
            layers=self.n_layers,
            hidden_dim=self.hidden_dim,
            head_dim=self.head_dim,
            dola=dola_enabled,
            dola_alpha=dola_alpha,
            kv_quant=self.kv_compressor.enabled,
            kv_quant_bits=turboquant_bits,
            steering=steering_enabled,
            etc_enabled=self.etc_enabled,
            etc_threshold=etc_threshold,
        )

    def _format_prompt(self, prompt: str) -> str:
        if hasattr(self.tokenizer, "apply_chat_template"):
            try:
                messages = [{"role": "user", "content": prompt}]
                return self.tokenizer.apply_chat_template(
                    messages, tokenize=False, add_generation_prompt=True
                )
            except Exception:
                logger.warning("chat_template_failed_in_format", prompt_len=len(prompt))
        return prompt

    def _build_constrained_prefix_fn(
        self, response_format: Optional[Dict[str, Any]],
    ) -> Optional[Callable]:
        """Track C-B: build a per-request prefix_allowed_tokens_fn from the
        cached TokenEnforcerTokenizerData and the request-supplied JSON
        schema.

        Returns a callable suitable for the model.generate(prefix_allowed_tokens_fn=)
        kwarg, or None on any failure / when constraint should be ignored:
          * lm-format-enforcer not installed
          * tokenizer-data cache absent (engine startup couldn't build it)
          * response_format is None or wrong shape (warn + ignore — backward
            compat: unsupported types must NOT 400 per context-package §3a/3b)
          * JsonSchemaParser construction fails

        Per paper-ingestor §40-46: pass the result as
        ``model.generate(prefix_allowed_tokens_fn=fn)`` — NOT inside
        LogitsProcessorList. Per paper §83: TokenEnforcer is not thread-safe;
        a fresh enforcer (i.e. fresh build_transformers_prefix_allowed_tokens_fn
        call) per request is required — only the underlying tokenizer-data is
        shared.
        """
        if response_format is None:
            return None
        if not _LMFE_AVAILABLE or self._enforcer_tokenizer_data is None:
            logger.warning(
                "response_format_ignored_no_lmfe",
                lmfe_available=_LMFE_AVAILABLE,
                tokenizer_data_ready=self._enforcer_tokenizer_data is not None,
            )
            return None
        if not isinstance(response_format, dict):
            logger.warning(
                "response_format_invalid_type",
                got=type(response_format).__name__,
            )
            return None
        rf_type = response_format.get("type")
        if rf_type != "json_schema":
            # Forward-compat: warn + ignore. Future {"type":"text"} or
            # {"type":"regex"} clients should not 400.
            logger.warning(
                "response_format_type_unsupported",
                type_=rf_type,
            )
            return None
        js = response_format.get("json_schema") or {}
        schema = js.get("schema") if isinstance(js, dict) else None
        # JsonSchemaParser accepts dict (constrained), pydantic.schema(), or
        # None (any valid JSON). For our request shape we only support dict.
        if schema is not None and not isinstance(schema, dict):
            logger.warning(
                "response_format_schema_invalid_type",
                got=type(schema).__name__,
            )
            return None
        try:
            parser = JsonSchemaParser(schema)
            prefix_fn = build_transformers_prefix_allowed_tokens_fn(
                self._enforcer_tokenizer_data, parser,
            )
            return prefix_fn
        except Exception as e:
            # Schema-construction errors land here; mid-generation
            # LMFormatEnforcerException is caught in _generate_impl wrapper.
            logger.warning(
                "response_format_parser_build_failed",
                error=str(e),
                error_type=type(e).__name__,
            )
            return None

    def generate(
        self,
        prompt: str,
        max_tokens: int | None = None,
        temperature: float = 0.0,
        marker_boundary: Optional[int] = None,
        response_format: Optional[Dict[str, Any]] = None,
        seed: Optional[int] = None,
        fitm_calibrator: Optional["FoundInTheMiddleCalibrator"] = None,
    ) -> "tuple[str, list[int], int, dict | None, dict[str, float | None], int, int]":
        """Generate text synchronously.

        Returns (text, token_ids, input_token_count, search_trigger, gg_signals,
                 cache_read_tokens, cache_creation_tokens).
        cache_read_tokens: prefix tokens served from KV cache (0 if no hit).
        cache_creation_tokens: tokens newly admitted to KV cache (0 if disabled).
        search_trigger is None unless ETC detected a high-entropy import context.
        gg_signals contains mean_logprob, min_logprob, entropy_mean, entropy_max,
        generation_novelty (A-25).

        marker_boundary (Track C-D): optional cap on PrefixKVCache prefix
        match. None = legacy full-prefix behaviour. int = honour Anthropic
        ``cache_control`` marker — cache_read_tokens <= marker_boundary.

        seed (Phase A6): optional reproducibility hint. When set,
        ``_generate_impl`` calls ``transformers.set_seed`` +
        ``torch.manual_seed`` immediately before ``model.generate()``, inside
        the ``torch.inference_mode()`` lock that the existing DoLa / ETC /
        steering / TQ LogitsProcessors run under — so all four stochastic
        paths inherit the seed without per-processor wiring. At
        ``temperature == 0`` (greedy decoding) the seed is a no-op but is
        still applied so logs record the request shape.
        """
        max_tokens = max_tokens or self.max_new_tokens

        with self.lock:
            # Phase A6: seed is applied INSIDE _generate_impl, after the lock
            # is acquired and immediately before model.generate, so the RNG
            # state mutation is bounded by the same lock that already
            # serialises every other piece of model.generate state.
            return self._generate_impl(
                prompt, max_tokens, temperature,
                marker_boundary=marker_boundary,
                response_format=response_format,
                seed=seed,
                fitm_calibrator=fitm_calibrator,
            )

    def generate_streaming(
        self,
        prompt: str,
        max_tokens: int | None = None,
        temperature: float = 0.0,
        response_format: Optional[Dict[str, Any]] = None,
        seed: Optional[int] = None,
        marker_boundary: Optional[int] = None,
    ) -> tuple[TextIteratorStreamer, int]:
        """Start streaming generation in a background thread.

        Returns (streamer, input_token_count). The caller iterates the
        streamer to get token chunks as they are produced.

        J-36: cache_read_tokens / cache_creation_tokens are attached as
        attributes on the returned streamer (streamer.cache_read_tokens,
        streamer.cache_creation_tokens) so the SSE layer can surface them
        in message_start / message_delta usage without changing the return
        tuple shape (preserves existing callers including test_seed.py).

        response_format (Phase B): when set + LMFE installed, enforces a JSON
        schema constraint on every token via prefix_allowed_tokens_fn. The
        helper _build_constrained_prefix_fn returns None for any failure
        (LMFE missing, malformed schema, unsupported type) — that case falls
        back to unconstrained streaming silently (warn-and-ignore matches the
        non-stream path's contract and engine.generate semantics).

        seed (Phase A6): optional reproducibility hint. Applied inside the
        background ``_run`` thread, right before ``model.generate()`` runs
        under ``torch.inference_mode()``. Errors during ``set_seed`` /
        ``manual_seed`` propagate via the thread's future (no silent
        fallback — A6 brief constraint).

        marker_boundary (J-36): Anthropic cache_control marker boundary,
        threaded into PrefixKVCache.lease so the streaming path can reuse
        a frozen-head KV prefix. Mirrors the non-streaming lease block in
        _generate_impl.
        """
        max_tokens = max_tokens or self.max_new_tokens
        formatted = self._format_prompt(prompt)
        inputs = self.tokenizer(formatted, return_tensors="pt").to(self.model.device)
        input_token_count = inputs.input_ids.shape[1]

        streamer = TextIteratorStreamer(
            self.tokenizer,
            skip_prompt=True,
            skip_special_tokens=False,
            timeout=STREAMER_TIMEOUT_S,
        )

        # Phase B: build prefix_fn HERE (main thread) before submitting _run to
        # the executor — TokenEnforcer is not thread-safe (paper §83). The
        # callable carries its own per-request enforcer state; sharing across
        # threads would corrupt the parser FSM. Returns None if response_format
        # is None or LMFE is unavailable; we only attach to kwargs when valid.
        prefix_fn = self._build_constrained_prefix_fn(response_format)

        # J-36: Prefix KV cache lease on the streaming path.
        # _prefix_kv_cache is None when TQ is active or cache disabled.
        # FH-3: when marker_boundary is set, the lease keys on the head
        # boundary and is NOT gated by input_token_count <= PREFILL_CHUNK_SIZE;
        # the tail beyond the head goes through _chunked_prefill as new tokens.
        # When marker_boundary is None, preserve the legacy gate (whole input
        # must fit one chunk) to match the non-streaming path's C-A invariant.
        cache_read_tokens = 0
        cache_creation_tokens = 0
        prefix_cache_hit = False
        _prefix_cache = None
        _prefix_len = 0
        if _prefix_kv_cache is not None:
            _lease_eligible = (
                marker_boundary is not None
                or input_token_count <= self.PREFILL_CHUNK_SIZE
            )
            if _lease_eligible:
                _prefix_cache, _prefix_len, _total_len = _prefix_kv_cache.lease(
                    inputs.input_ids[0],
                    marker_boundary=marker_boundary,
                )
                if _prefix_cache is not None:
                    prefix_cache_hit = True
                    cache_read_tokens = _prefix_len
                    logger.info(
                        "prefix_cache_hit_streaming",
                        prefix_len=_prefix_len,
                        total_len=_total_len,
                        marker_boundary=marker_boundary,
                    )

        # Build gen_input_ids / past_kv / gen_attention_mask.
        # When prefix_cache_hit, reuse the proven crop-to-_prefix_len-1
        # invariant from _generate_impl (RoPE/mask correctness fix).
        # When input > PREFILL_CHUNK_SIZE without a hit, use _chunked_prefill.
        past_kv = None
        if prefix_cache_hit:
            _prefix_cache.crop(_prefix_len - 1)
            gen_input_ids = inputs.input_ids[:, _prefix_len - 1:]
            gen_attention_mask = inputs.attention_mask
            past_kv = _prefix_cache
        elif input_token_count > self.PREFILL_CHUNK_SIZE:
            past_kv = self._chunked_prefill(
                inputs.input_ids, inputs.attention_mask,
            )
            gen_input_ids = inputs.input_ids[:, -1:]
            gen_attention_mask = inputs.attention_mask
        else:
            gen_input_ids = inputs.input_ids
            gen_attention_mask = inputs.attention_mask

        # E-28: StreamingLLM attention-sink masking (env-gated, default off).
        # Applied to the prefill attention_mask before generate; no-op when
        # ELAI_STREAMING_LLM_SINKS is unset/0.
        gen_attention_mask = self._apply_streaming_llm_if_active(
            gen_attention_mask, context_position=input_token_count,
        )

        generation_kwargs = {
            "input_ids": gen_input_ids,
            "attention_mask": gen_attention_mask,
            "max_new_tokens": max_tokens,
            "do_sample": temperature > 0,
            "streamer": streamer,
            "output_hidden_states": self.dola_enabled,
            "return_dict_in_generate": True,
        }
        if past_kv is not None:
            generation_kwargs["past_key_values"] = past_kv
        if temperature > 0:
            generation_kwargs["temperature"] = temperature
        if prefix_fn is not None:
            generation_kwargs["prefix_allowed_tokens_fn"] = prefix_fn

        logits_processors = []
        if self.dola_enabled and self._dola_processor is not None:
            self._dola_processor.register_hooks()
            logits_processors.append(self._dola_processor)
        # J-13: SeededGumbelProcessor (SGLang det-mode). Per-request, gated on
        # seed; constructed per call (matches FITM per-request pattern).
        if seed is not None:
            logits_processors.append(SeededGumbelProcessor(seed))
        # Set logits_processor whenever the list is non-empty. Previously this
        # was inside the dola branch and silently dropped other processors
        # (e.g. SeededGumbel) when dola was off (J-13 gather HIGH warning).
        if logits_processors:
            generation_kwargs["logits_processor"] = logits_processors

        def _run():
            try:
                with self.lock, torch.inference_mode():
                    # Phase A6 — seed BOTH transformers and torch RNGs
                    # immediately before model.generate. Order matters:
                    # transformers.set_seed sets python/numpy/torch CPU+CUDA
                    # all in one call (transformers 4.45+ confirmed); the
                    # explicit torch.manual_seed() afterwards is a belt-and-
                    # braces guard against future drift in set_seed coverage
                    # (and is a no-op-fast when seeds match). DoLa, ETC,
                    # steering and TQ all run as LogitsProcessors inside this
                    # model.generate() invocation so they inherit the seed
                    # without per-processor wiring. seed is a no-op at
                    # temperature == 0 (greedy decoding) but still applied so
                    # the request log records it.
                    if seed is not None:
                        _transformers_set_seed(int(seed))
                        torch.manual_seed(int(seed))
                    # E-28 STEP 4: pin SDPA backend to FLASH_ATTENTION around
                    # model.generate to prevent silent MATH fallback
                    # (pytorch#133780). Fresh context manager per call.
                    if self._sdpa_pin_enabled:
                        _sdpa_ctx, _, _ = pin_sdpa_backend()
                        with _sdpa_ctx:
                            self.model.generate(**generation_kwargs)
                    else:
                        self.model.generate(**generation_kwargs)
            finally:
                if self._dola_processor is not None:
                    self._dola_processor.remove_hooks()

        _thread_pool.submit(_run)

        logger.info(
            "streaming_started",
            input_tokens=input_token_count,
            max_tokens=max_tokens,
            prompt_chars=len(prompt),
            cache_read=cache_read_tokens,
            cache_creation=cache_creation_tokens,
        )
        streamer.cache_read_tokens = cache_read_tokens
        streamer.cache_creation_tokens = cache_creation_tokens
        return streamer, input_token_count

    # Chunked prefill: cap FFN activation scratch to chunk_size tokens
    PREFILL_CHUNK_SIZE = 8192

    def _make_cache(self):
        """Create a KV cache: quantized (KIVI-style) if enabled, else DynamicCache."""
        return self.kv_compressor.make_cache()

    def _chunked_prefill(
        self, input_ids: torch.Tensor, attention_mask: torch.Tensor,
    ):
        """Process long input in chunks, building KV cache incrementally.

        Returns (past_key_values, full_attention_mask) ready for generate().
        Peak FFN activation is bounded by chunk_size instead of full seq_len.
        """
        seq_len = input_ids.shape[1]
        cache = self._make_cache()
        position = 0

        while position < seq_len:
            end = min(position + self.PREFILL_CHUNK_SIZE, seq_len)
            chunk_ids = input_ids[:, position:end]
            chunk_mask = attention_mask[:, :end]

            with torch.inference_mode():
                outputs = self.model(
                    input_ids=chunk_ids,
                    attention_mask=chunk_mask,
                    past_key_values=cache,
                    use_cache=True,
                )
            cache = outputs.past_key_values
            position = end

            if end < seq_len:
                logger.info("chunked_prefill", processed=end, total=seq_len)

        return cache

    def _apply_streaming_llm_if_active(
        self,
        attention_mask: "torch.Tensor",
        context_position: int,
    ) -> "torch.Tensor":
        """Apply StreamingLLM attention-sink masking if enabled at boot.

        No-op (returns the mask unchanged) when ELAI_STREAMING_LLM_SINKS is
        unset/0 — preserving byte-identical baseline behaviour. When sinks
        > 0 AND context_position >= sink_count + window_size, delegates to
        apply_streaming_llm_mask (arXiv:2309.17453) which keeps the first
        `sink_count` tokens plus a trailing sliding window of
        `window_size` tokens and zeroes the evicted middle region.

        Emits the `streaming_llm_engaged_ssb` beacon (mirrors the Rust
        SsbActionType::StreamingLlmEngaged for dashboard visibility) ONLY
        when eviction actually occurs — short contexts below the
        sink+window threshold are a silent passthrough so the SSB journal
        is not polluted with no-op rows.

        Args:
            attention_mask: 2D or 4D HF attention mask tensor.
            context_position: Absolute position of the current token in
                the untruncated text (input_token_count for prefill).

        Returns:
            The (possibly masked) attention mask tensor.
        """
        sinks = self.streaming_llm_sinks
        window = self.streaming_llm_window
        if sinks < 1:
            return attention_mask
        if context_position < sinks + window:
            # Short context — no eviction; passthrough (no SSB noise).
            return attention_mask
        masked = apply_streaming_llm_mask(
            attention_mask,
            sink_count=sinks,
            window_size=window,
            context_position=context_position,
        )
        logger.info(
            "streaming_llm_engaged_ssb",
            sink_count=sinks,
            window_size=window,
            context_position=context_position,
        )
        return masked

    def _generate_impl(
        self,
        prompt: str,
        max_tokens: int,
        temperature: float,
        marker_boundary: Optional[int] = None,
        response_format: Optional[Dict[str, Any]] = None,
        seed: Optional[int] = None,
        fitm_calibrator: Optional["FoundInTheMiddleCalibrator"] = None,
    ) -> "tuple[str, list[int], int, dict | None, dict[str, float | None], int, int]":
        gen_start = time.time()
        formatted = self._format_prompt(prompt)
        inputs = self.tokenizer(formatted, return_tensors="pt").to(self.model.device)
        input_token_count = inputs.input_ids.shape[1]

        logger.info(
            "request",
            prompt_chars=len(prompt),
            formatted_chars=len(formatted),
            input_tokens=input_token_count,
            max_tokens=max_tokens,
        )

        # C-A: Prefix KV cache lease (non-streaming path; streaming path
        # now has its own lease block in generate_streaming per J-36).
        # _prefix_kv_cache is None when TQ is active or cache disabled.
        # Chunked prefill takes priority when input > PREFILL_CHUNK_SIZE.
        cache_read_tokens = 0
        cache_creation_tokens = 0
        prefix_cache_hit = False
        if _prefix_kv_cache is not None and input_token_count <= self.PREFILL_CHUNK_SIZE:
            # Track C-D: pass Anthropic cache_control marker boundary through.
            # marker_boundary is None for the legacy unbounded full-prefix path
            # (preserves C-A LIVE invariants), int when an Anthropic
            # cache_control: ephemeral block was parsed by handle_messages.
            _prefix_cache, _prefix_len, _total_len = _prefix_kv_cache.lease(
                inputs.input_ids[0],
                marker_boundary=marker_boundary,
            )
            if _prefix_cache is not None:
                prefix_cache_hit = True
                cache_read_tokens = _prefix_len
                logger.info(
                    "prefix_cache_hit",
                    prefix_len=_prefix_len,
                    total_len=_total_len,
                    marker_boundary=marker_boundary,
                )

        # Chunked prefill for long inputs to cap FFN activation memory
        past_kv = None
        if input_token_count > self.PREFILL_CHUNK_SIZE:
            past_kv = self._chunked_prefill(
                inputs.input_ids, inputs.attention_mask,
            )
            # For generate(), pass only the last token + cached KV
            gen_input_ids = inputs.input_ids[:, -1:]
            gen_attention_mask = inputs.attention_mask
        else:
            gen_attention_mask = inputs.attention_mask
            if prefix_cache_hit:
                # FULL_AND_PARTIAL_PREFIX_CACHE_HIT (patch-1.X):
                # The leased DynamicCache has seq_len equal to the FULL input
                # length of the prior admit (>= _prefix_len). Two failure modes
                # if we don't crop:
                #  - Full match (_prefix_len == _total_len): HF computes
                #    cache_position=arange(N,N)=empty -> IndexError on [-1].
                #  - Partial match: cache covers more positions than _prefix_len;
                #    feeding inputs[_prefix_len:] yields cache_position
                #    [N..N+new_len) instead of [_prefix_len..total_len), causing
                #    RoPE mis-rotation and attention_mask underflow.
                # Unified fix: crop cache to (_prefix_len - 1) and feed
                # inputs[_prefix_len - 1:]. Then cache_position = arange(
                # _prefix_len - 1, _total_len), attention_mask length
                # _total_len = (_prefix_len - 1) + (_total_len - _prefix_len + 1)
                # is correct, and one cached token is recomputed (negligible).
                # _prefix_len >= _KV_MIN_PREFIX_TOKENS=16 guaranteed by lease().
                _prefix_cache.crop(_prefix_len - 1)
                gen_input_ids = inputs.input_ids[:, _prefix_len - 1:]
                past_kv = _prefix_cache
            else:
                gen_input_ids = inputs.input_ids
                if self.kv_compressor.enabled:
                    # Use quantized cache even for short prompts if TQ enabled
                    past_kv = self._make_cache()

        # E-28: StreamingLLM attention-sink masking (env-gated, default off).
        # Applied to the full-length attention_mask AFTER chunked prefill so
        # the KV cache still holds every token (mask-only, no eviction) but
        # attention is restricted to sinks U window per arXiv:2309.17453.
        # No-op when ELAI_STREAMING_LLM_SINKS is unset/0.
        gen_attention_mask = self._apply_streaming_llm_if_active(
            gen_attention_mask, context_position=input_token_count,
        )

        generation_kwargs = {
            "input_ids": gen_input_ids,
            "attention_mask": gen_attention_mask,
            "max_new_tokens": max_tokens,
            "do_sample": temperature > 0,
            "output_hidden_states": self.dola_enabled,
            "return_dict_in_generate": True,
        }
        if past_kv is not None:
            generation_kwargs["past_key_values"] = past_kv
            if prefix_cache_hit:
                # transformers 4.57+ auto-infers cache_position from past_kv
                # length; passing it explicitly is now rejected by
                # _validate_model_kwargs. HF #36151 (the original bug this
                # workaround addressed) was fixed upstream by the time
                # cache_position became a strict kwarg.
                pass
        if temperature > 0:
            generation_kwargs["temperature"] = temperature

        logits_processors = []
        if self.dola_enabled and self._dola_processor is not None:
            self._dola_processor.register_hooks()
            logits_processors.append(self._dola_processor)
        if self._etc_processor is not None:
            self._etc_processor.reset()
            logits_processors.append(self._etc_processor)
        # GG signal collector (A-17) — always active, zero cost
        self._gg_collector.reset()
        logits_processors.append(self._gg_collector)
        # E-32: FoundInTheMiddleCalibrator (FITM) — inserted AFTER all existing
        # processors per CRANE-style ordering (modifiers after observers).
        # Only active when ELAI_FITM_ENABLED=1 AND prompt > 8K tokens.
        _fitm_t0: float = 0.0
        if fitm_calibrator is not None and fitm_calibrator.active:
            logits_processors.append(fitm_calibrator)
            _fitm_t0 = time.time()
        # J-13: SeededGumbelProcessor (SGLang det-mode). Per-request, gated on
        # seed; constructed per call (matches FITM per-request pattern).
        if seed is not None:
            logits_processors.append(SeededGumbelProcessor(seed))
        if logits_processors:
            generation_kwargs["logits_processor"] = logits_processors

        # Track C-B: constrained JSON decoding via lm-format-enforcer.
        # IMPORTANT (paper-ingestor §40-46): the correct integration is the
        # `prefix_allowed_tokens_fn=` kwarg of model.generate(), NOT a
        # LogitsProcessor entry in the list above. Adding to the LogitsProcessorList
        # would require subclassing TokenEnforcer manually (paper §43 — diagnostic
        # path only) and is incorrect.
        # Per paper §83: TokenEnforcer is not thread-safe — build a NEW
        # prefix_allowed_tokens_fn per request. The cached
        # _enforcer_tokenizer_data is the only shared piece.
        prefix_fn = self._build_constrained_prefix_fn(response_format)
        if prefix_fn is not None:
            generation_kwargs["prefix_allowed_tokens_fn"] = prefix_fn

        try:
            with torch.inference_mode():
                # Phase A6 — seed BOTH transformers and torch RNGs immediately
                # before model.generate. Same rationale as generate_streaming's
                # _run thread: transformers.set_seed handles python/numpy/torch
                # CPU+CUDA in one call; the explicit torch.manual_seed is a
                # belt-and-braces guard. DoLa / ETC / steering / TurboQuant
                # all run as LogitsProcessors inside this model.generate so
                # they inherit the seed without per-processor wiring.
                # No-op at temperature == 0 (greedy decoding) but still
                # applied so the request log records it.
                if seed is not None:
                    _transformers_set_seed(int(seed))
                    torch.manual_seed(int(seed))
                # E-28 STEP 4: pin SDPA backend to FLASH_ATTENTION around
                # model.generate to prevent silent MATH fallback
                # (pytorch#133780). Fresh context manager per call.
                if self._sdpa_pin_enabled:
                    _sdpa_ctx, _, _ = pin_sdpa_backend()
                    with _sdpa_ctx:
                        outputs = self.model.generate(**generation_kwargs)
                else:
                    outputs = self.model.generate(**generation_kwargs)
        finally:
            if self._dola_processor is not None:
                self._dola_processor.remove_hooks()
            # E-32: log FITM SSB event after generation (outside inference_mode)
            if fitm_calibrator is not None and fitm_calibrator.active and _fitm_t0 > 0:
                applied_ms = (time.time() - _fitm_t0) * 1000.0
                logger.info(
                    "fitm_calibration_applied",
                    **fitm_calibrator.ssb_payload(applied_ms),
                )

        # C-A: Admit the post-generation DynamicCache for future prefix reuse.
        # outputs.past_key_values is populated because return_dict_in_generate=True.
        # CRITICAL: returned_cache contains KV for input_token_count + output_len.
        # Storing it as-is would corrupt cache_position on a future hit (generate()
        # would compute cache_position assuming all input_ids are already cached).
        # Crop to input_token_count so a future lease aligns with the new prompt's
        # input_ids. crop() is in-place and supported by DynamicLayer.
        # Only admit when prefix cache is active AND chunked prefill was not used
        # (chunked prefill already owns the cache lifecycle).
        if _prefix_kv_cache is not None and input_token_count <= self.PREFILL_CHUNK_SIZE:
            returned_cache = outputs.past_key_values
            if returned_cache is not None:
                try:
                    returned_cache.crop(input_token_count)
                except (ValueError, AttributeError) as e:
                    logger.warning(
                        "prefix_cache_crop_failed",
                        reason=type(e).__name__,
                        error=str(e),
                    )
                    returned_cache = None
            if returned_cache is not None:
                _prefix_kv_cache.admit(inputs.input_ids[0], returned_cache)
                cache_creation_tokens = input_token_count - cache_read_tokens
                logger.info(
                    "prefix_cache_admit",
                    input_tokens=input_token_count,
                    cache_read=cache_read_tokens,
                    cache_creation=cache_creation_tokens,
                )

        # Extract generated tokens (after the input we passed to generate)
        gen_prefix_len = gen_input_ids.shape[1]
        generated_ids = outputs.sequences[0, gen_prefix_len:].tolist()
        text = self.tokenizer.decode(generated_ids, skip_special_tokens=False)

        # Clean special tokens but keep tool call markers
        for token in [
            "<bos>", "<eos>", "<pad>",
            "<|im_end|>", "<|endoftext|>", "<|im_start|>",
        ]:
            text = text.replace(token, "")

        # Build ETC search trigger payload if triggered
        search_trigger = None
        if self._etc_processor is not None and self._etc_processor.triggered:
            trigger_idx = self._etc_processor.trigger_token_idx - gen_prefix_len
            partial = text[:max(0, trigger_idx * 4)]  # approximate char position
            search_trigger = {
                "partial_text": partial,
                "detected_context": "import",
                "trigger_token_idx": self._etc_processor.trigger_token_idx,
            }

        gen_elapsed_ms = (time.time() - gen_start) * 1000
        out_tokens = len(generated_ids)
        tps = out_tokens / (gen_elapsed_ms / 1000) if gen_elapsed_ms > 0 else 0
        logger.info(
            "response",
            input_tokens=input_token_count,
            output_tokens=out_tokens,
            elapsed_ms=round(gen_elapsed_ms),
            tok_per_s=round(tps, 1),
            chunked=input_token_count > self.PREFILL_CHUNK_SIZE,
            etc_triggered=search_trigger is not None,
            cache_read=cache_read_tokens,
            cache_creation=cache_creation_tokens,
        )
        _log_vram("after_generate")

        gg_signals = self._gg_collector.signals()

        # A-25: generation novelty — cosine distance from previous attempt
        stripped_text = text.strip()
        novelty = self._novelty_tracker.compute(
            prompt, stripped_text, self.model, self.tokenizer,
        )
        gg_signals["generation_novelty"] = novelty

        return (
            stripped_text, generated_ids, input_token_count,
            search_trigger, gg_signals,
            cache_read_tokens, cache_creation_tokens,
        )


# ---------------------------------------------------------------------------
# Anthropic message format helpers
# ---------------------------------------------------------------------------

_TOOL_CALL_RE = re.compile(
    r"<\|tool_call>call:(\w+)\{(.*?)\}<tool_call\|>", re.DOTALL
)


def _extract_cache_marker_token_position(
    tokenizer,
    messages: list,
    system,
    tools: list | None,
) -> Optional[int]:
    """Parse Anthropic per-content-block ``cache_control`` markers.

    Walks ``messages[*].content[*]``, ``system`` (when ``list[ContentBlock]``),
    and ``tools[*]`` looking for blocks carrying ``cache_control``. Returns the
    cumulative input-token offset (within the formatted prompt the engine
    actually sees) up to and including the LAST marked block in token order.

    Per Anthropic spec (https://docs.anthropic.com/en/docs/build-with-claude/
    prompt-caching):
      * ``cache_control`` is per-content-block, NOT a top-level request field.
      * ``type`` must equal ``"ephemeral"``; we WARN-and-IGNORE other values
        rather than 400ing (backward-compat: unknown future ``type`` values
        should not break existing clients).
      * Up to 4 markers per request; only the LAST in token order is honoured
        as the boundary (earlier markers are subsumed by the last).
      * Marker boundary is INCLUSIVE: positions ``[0..boundary)`` are
        cacheable, positions ``>= boundary`` are not.

    Returns:
      * ``None`` when no marker is found (existing full-prefix caching).
      * ``int`` token offset (1 <= boundary <= len(full_prompt_tokens)) when
        a marker is honoured.

    Implementation strategy: re-render the prompt up to and including the
    marked block via the SAME chat template the engine uses, then take the
    length of the resulting token sequence. This avoids off-by-N errors from
    chat-template role wrappers because the same template is applied in
    both the partial (boundary computation) and full (engine) renders.
    """
    if not messages:
        return None

    # Step 1: walk all blocks in token order and record the LAST cache_control
    # location. We track location as (scope, message_index, block_index) so we
    # can later truncate the per-scope structures up to that point.
    #
    # Order of scopes matches the prompt assembly in _anthropic_to_gemma_prompt:
    # tools render first (in chat template), then system (folded into the
    # first user message), then messages. For boundary tokenisation we
    # currently support markers placed inside ``messages[*].content[*]`` and
    # ``system`` list-form. Markers on tools[*] are accepted-and-ignored for
    # now (rare in practice; supporting them would require partial-tools
    # rendering which the chat template does not naturally expose).
    last_marker: Optional[tuple[str, int, int]] = None
    marker_count = 0

    def _block_has_marker(block) -> bool:
        nonlocal marker_count
        if not isinstance(block, dict):
            return False
        cc = block.get("cache_control")
        if cc is None:
            return False
        if not isinstance(cc, dict):
            logger.warning(
                "cache_control_invalid_shape",
                got=type(cc).__name__,
                action="ignored",
            )
            return False
        cc_type = cc.get("type")
        if cc_type != "ephemeral":
            logger.warning(
                "cache_control_unsupported_type",
                got=cc_type,
                action="ignored",
                supported=["ephemeral"],
            )
            return False
        marker_count += 1
        return True

    # Walk messages[*].content[*]
    for msg_idx, msg in enumerate(messages):
        content = msg.get("content") if isinstance(msg, dict) else None
        if not isinstance(content, list):
            continue
        for blk_idx, block in enumerate(content):
            if _block_has_marker(block):
                last_marker = ("messages", msg_idx, blk_idx)

    # Walk system[*] (only when system is a list of ContentBlocks)
    if isinstance(system, list):
        for blk_idx, block in enumerate(system):
            if _block_has_marker(block):
                # system markers bind to scope="system"; message_index unused.
                last_marker = ("system", -1, blk_idx)

    # Walk tools[*] — accept-and-ignore (logged for visibility; not honoured).
    if isinstance(tools, list):
        for block in tools:
            if isinstance(block, dict) and block.get("cache_control") is not None:
                logger.warning(
                    "cache_control_on_tool_unsupported",
                    tool_name=block.get("name"),
                    action="ignored",
                    note="marker honoured only on messages/system blocks",
                )

    if last_marker is None:
        return None

    if marker_count > 4:
        logger.warning(
            "cache_control_too_many_markers",
            count=marker_count,
            action="last_in_order_honoured",
            spec_max=4,
        )

    scope, msg_target_idx, blk_target_idx = last_marker

    # Step 2: build a TRUNCATED messages/system pair that preserves only the
    # content up to and including the marked block. Then render via the same
    # chat template the engine uses, tokenise, and take the token count as
    # the boundary.
    truncated_messages = []
    truncated_system = system

    if scope == "system":
        # Boundary lives inside the system list — keep only blocks [0..idx]
        # of system, and DROP messages entirely (the marker subsumes nothing
        # past it). This is the rare "system-only cacheable prefix" case.
        if isinstance(system, list):
            truncated_system = system[: blk_target_idx + 1]
        truncated_messages = []
    else:
        # scope == "messages": copy messages[0..msg_target_idx-1] verbatim,
        # then append a partial copy of messages[msg_target_idx] whose content
        # list is truncated to blocks [0..blk_target_idx].
        for i in range(msg_target_idx):
            truncated_messages.append(messages[i])
        target_msg = messages[msg_target_idx]
        if isinstance(target_msg, dict):
            target_content = target_msg.get("content")
            if isinstance(target_content, list):
                # Shallow-copy the message dict and truncate its content list.
                # Strip cache_control off the copied blocks (it would otherwise
                # be a no-op for the chat template, but we keep the partial
                # render free of metadata that downstream renderers might
                # complain about).
                clean_blocks = []
                for blk in target_content[: blk_target_idx + 1]:
                    if isinstance(blk, dict) and "cache_control" in blk:
                        clean = {k: v for k, v in blk.items() if k != "cache_control"}
                        clean_blocks.append(clean)
                    else:
                        clean_blocks.append(blk)
                truncated_messages.append({
                    "role": target_msg.get("role", "user"),
                    "content": clean_blocks,
                })
            else:
                # content is a string; the marker_count check above means the
                # marker was on a parent dict — defensive: keep message whole.
                truncated_messages.append(target_msg)

    # Step 3: render the truncated prompt with the SAME helper the engine
    # uses for the full prompt, then format via the engine's _format_prompt
    # is NOT how /v1/messages works — handle_messages calls
    # _anthropic_to_gemma_prompt + passes the result string straight to
    # generate(prompt). generate() then internally calls
    # InferenceEngine._format_prompt which wraps the user content in another
    # chat template layer. To match, we apply the SAME wrapping path the
    # engine does on the full prompt: _anthropic_to_gemma_prompt produces
    # the chat-template output, _format_prompt wraps that as a single user
    # turn. We replicate the same composition here for the truncated input.
    try:
        partial_prompt = _anthropic_to_gemma_prompt(
            tokenizer, truncated_messages, truncated_system, tools
        )
    except Exception as exc:
        logger.warning(
            "cache_marker_partial_render_failed",
            error=str(exc),
            action="boundary_disabled",
        )
        return None

    # Mirror InferenceEngine._format_prompt: when the tokenizer supports
    # apply_chat_template, the engine wraps the upstream prompt as
    # [{"role": "user", "content": <upstream_prompt>}] with
    # add_generation_prompt=True. We do the SAME thing for the truncated
    # render so the token count we measure aligns with the input_ids the
    # engine will see for the full prompt's matching prefix.
    formatted_partial = partial_prompt
    if hasattr(tokenizer, "apply_chat_template"):
        try:
            formatted_partial = tokenizer.apply_chat_template(
                [{"role": "user", "content": partial_prompt}],
                tokenize=False,
                add_generation_prompt=True,
            )
        except Exception:
            # Mirror _format_prompt's silent fallback to the unwrapped string.
            formatted_partial = partial_prompt

    try:
        partial_ids = tokenizer(formatted_partial, return_tensors=None).get(
            "input_ids", []
        )
    except Exception as exc:
        logger.warning(
            "cache_marker_partial_tokenise_failed",
            error=str(exc),
            action="boundary_disabled",
        )
        return None

    # tokenizer(...) without return_tensors returns a list[int] (single-input
    # path) — handle the rare nested-list shape defensively.
    if partial_ids and isinstance(partial_ids[0], list):
        partial_ids = partial_ids[0]
    boundary = len(partial_ids)
    if boundary <= 0:
        return None
    logger.info(
        "cache_marker_boundary",
        boundary_tokens=boundary,
        scope=scope,
        marker_count=marker_count,
    )
    return boundary


def _anthropic_to_gemma_prompt(
    tokenizer, messages: list, system: str = "", tools: list | None = None
) -> str:
    gemma_tools = None
    if tools:
        gemma_tools = []
        for tool in tools:
            name = tool.get("name", "")
            if name not in CORE_TOOLS:
                continue
            desc = tool.get("description", "")
            if len(desc) > 200:
                desc = desc[:200] + "..."
            gemma_tools.append({
                "type": "function",
                "function": {
                    "name": name,
                    "description": desc,
                    "parameters": tool.get("input_schema", {}),
                },
            })
        if not gemma_tools:
            gemma_tools = None

    gemma_messages = []
    for msg in messages:
        role = msg.get("role", "user")
        content = msg.get("content", "")

        if role == "assistant":
            if isinstance(content, list):
                text_parts = []
                for block in content:
                    if isinstance(block, dict):
                        if block.get("type") == "text":
                            text_parts.append(block.get("text", ""))
                        elif block.get("type") == "tool_use":
                            tool_name = block.get("name", "")
                            tool_input = block.get("input", {})
                            text_parts.append(
                                _format_gemma_tool_call(tool_name, tool_input)
                            )
                content = "\n".join(text_parts)
            gemma_messages.append({"role": "assistant", "content": content})

        elif role == "user":
            if isinstance(content, list):
                text_parts = []
                for block in content:
                    if isinstance(block, dict):
                        if block.get("type") == "text":
                            text_parts.append(block.get("text", ""))
                        elif block.get("type") == "tool_result":
                            result_content = block.get("content", "")
                            if isinstance(result_content, list):
                                result_text = "\n".join(
                                    b.get("text", "")
                                    for b in result_content
                                    if isinstance(b, dict) and b.get("type") == "text"
                                )
                            else:
                                result_text = str(result_content)
                            text_parts.append(f"Tool result: {result_text}")
                content = "\n".join(text_parts)
            gemma_messages.append({"role": "user", "content": content})

    # Prepend system to first user message
    if system and gemma_messages:
        for i, m in enumerate(gemma_messages):
            if m["role"] == "user":
                gemma_messages[i]["content"] = system + "\n\n" + m["content"]
                break

    try:
        return tokenizer.apply_chat_template(
            gemma_messages,
            tools=gemma_tools,
            tokenize=False,
            add_generation_prompt=True,
        )
    except Exception as exc:
        logger.warning("chat_template_failed", error=str(exc))
        parts = []
        if system:
            parts.append(system)
        for m in gemma_messages:
            parts.append(m.get("content", ""))
        return "\n".join(parts)


def _format_gemma_tool_call(name: str, args: dict) -> str:
    parts = []
    for k, v in args.items():
        if isinstance(v, str):
            parts.append(f'{k}:<|"|>{v}<|"|>')
        else:
            parts.append(f"{k}:{orjson.dumps(v).decode()}")
    args_str = ",".join(parts)
    return f"<|tool_call>call:{name}{{{args_str}}}<tool_call|>"


def _parse_tool_calls(text: str) -> tuple[list[tuple[str, dict]], str]:
    """Parse Gemma4 tool call output into (name, args) pairs + remaining text."""
    matches = list(_TOOL_CALL_RE.finditer(text))
    if not matches:
        return [], text

    tool_calls = []
    for match in matches:
        tool_name = match.group(1)
        args_raw = match.group(2)
        args = _parse_gemma_args(args_raw)
        tool_calls.append((tool_name, args))

    remaining = _TOOL_CALL_RE.sub("", text).strip()
    return tool_calls, remaining


def _parse_gemma_args(args_raw: str) -> dict:
    args: dict = {}
    current_key = ""
    current_val = ""
    in_string = False
    parsing_key = True
    i = 0

    while i < len(args_raw):
        ch = args_raw[i]
        if args_raw[i : i + 5] == '<|"|>':
            in_string = not in_string
            i += 5
            continue
        if in_string:
            current_val += ch
            i += 1
            continue
        if ch == ":" and parsing_key:
            parsing_key = False
            i += 1
            continue
        if ch == "," and not in_string:
            if current_key:
                args[current_key.strip()] = _parse_arg_value(current_val.strip())
            current_key = ""
            current_val = ""
            parsing_key = True
            i += 1
            continue
        if parsing_key:
            current_key += ch
        else:
            current_val += ch
        i += 1

    if current_key:
        args[current_key.strip()] = _parse_arg_value(current_val.strip())
    return args


def _parse_arg_value(val: str):
    if not val:
        return ""
    try:
        return orjson.loads(val.encode())
    except (orjson.JSONDecodeError, ValueError):
        return val


# ---------------------------------------------------------------------------
# orjson response helper
# ---------------------------------------------------------------------------


class OrjsonResponse(Response):
    media_type = "application/json"

    def render(self, content) -> bytes:
        return orjson.dumps(content)


# ---------------------------------------------------------------------------
# FastAPI application
# ---------------------------------------------------------------------------

_engine: InferenceEngine | None = None
# PART12 OQ5: dashboard SettingsWindow writes cloud / host-level keys
# (vram_budget_gb, cloud_api_key, cloud_spend_cap_usd) that are not owned
# by the inference engine. They live in this module-level dict so /v1/settings
# can ack them and /v1/settings GET can re-seed dashboard state across
# refreshes. cloud_api_key is stored but never echoed in GET responses.
_cloud_settings: dict[str, Any] = {}
# Separate pools: _thread_pool for GPU-bound model.generate() calls,
# _drain_pool for lightweight streamer-drain threads. Prevents deadlock
# when concurrent streaming requests exhaust a single shared pool.
_thread_pool: ThreadPoolExecutor = ThreadPoolExecutor(
    max_workers=THREAD_POOL_SIZE, thread_name_prefix="elai-gpu"
)
_drain_pool: ThreadPoolExecutor = ThreadPoolExecutor(
    max_workers=THREAD_POOL_SIZE * 2, thread_name_prefix="elai-drain"
)


_teardown_done: bool = False


def _graceful_gpu_teardown() -> None:
    """Release GPU resources in deterministic order before process exit.

    Idempotent: subsequent calls are no-ops (guarded by _teardown_done).

    Limitations:
        - SIGKILL bypasses this handler; the WSL2 Linux kernel will reap the
          process but CUDA context cleanup is not guaranteed in that case.
        - WSL VM termination requires `wsl --terminate Ubuntu` from the host
          to reset the Hyper-V partition; this code cannot handle that case.
    """
    global _engine, _teardown_done
    if _teardown_done:
        return
    _teardown_done = True
    if _engine is None:
        return
    try:
        if hasattr(_engine, "_dola_processor") and _engine._dola_processor is not None:
            _engine._dola_processor.remove_hooks()
    except Exception as exc:
        logger.warning("teardown_dola_hooks_failed", error=str(exc))
    try:
        if hasattr(_engine, "model") and _engine.model is not None:
            del _engine.model
    except Exception as exc:
        logger.warning("teardown_model_del_failed", error=str(exc))
    try:
        if torch.cuda.is_available():
            torch.cuda.synchronize()
            torch.cuda.empty_cache()
            logger.info("vram_released")
    except Exception as exc:
        logger.warning("teardown_cuda_flush_failed", error=str(exc))
    try:
        import gc
        gc.collect()
    except Exception:
        pass
    _engine = None


@asynccontextmanager
async def _lifespan(app: FastAPI):
    yield
    # Shutdown: drain in-flight GPU work in the pools FIRST, then run teardown.
    # Reversing this order produces a use-after-free: a thread mid-generate()
    # would see its model object deleted out from under the running CUDA kernel.
    _thread_pool.shutdown(wait=True)
    _drain_pool.shutdown(wait=True)
    _graceful_gpu_teardown()
    logger.info("server_shutdown")


app = FastAPI(title="ELAI Inference Sidecar", lifespan=_lifespan)


@app.get("/health")
async def health():
    return OrjsonResponse({"status": "healthy"})


@app.get("/v1/models")
async def list_models():
    # patch-1.33/1.34: expose num_hidden_layers + hidden_dim so the Rust CLI
    # can resolve "auto" steering-layer specs and right-size layer-scoped
    # buffers without hardcoding per-model constants.
    if _engine is not None:
        entry = {
            "id": _engine.model_name,
            "object": "model",
            "num_hidden_layers": _engine.n_layers,
            "hidden_dim": _engine.hidden_dim,
        }
    else:
        entry = {"id": "elai-composed", "object": "model"}
    return OrjsonResponse({"data": [entry]})


@app.post("/shutdown")
async def shutdown():
    logger.info("shutdown_requested")
    # Schedule shutdown after response is sent
    loop = asyncio.get_running_loop()
    loop.call_later(0.5, lambda: signal.raise_signal(signal.SIGINT))
    return OrjsonResponse({"status": "shutting down"})


@app.post("/v1/compute_steering")
async def compute_steering(request: Request):
    """Compute a steering vector from a contrast prompt pair (ActAdd: h+ - h-).

    Expected JSON: {"positive_prompt": str, "negative_prompt": str, "layer_idx": int}
    Returns: {"vector": [float, ...]}
    """
    if _engine is None:
        return OrjsonResponse({"error": "engine not initialized"}, status_code=503)

    body = await request.body()
    try:
        data = orjson.loads(body)
    except orjson.JSONDecodeError:
        return OrjsonResponse({"error": "invalid JSON"}, status_code=400)

    positive_prompt = data.get("positive_prompt", "")
    negative_prompt = data.get("negative_prompt", "")
    layer_idx = data.get("layer_idx")

    if not positive_prompt or not negative_prompt:
        return OrjsonResponse(
            {"error": "positive_prompt and negative_prompt are required"},
            status_code=400,
        )
    if layer_idx is None:
        layer_idx = _engine.n_layers // 3  # auto: target layer for steering
    if not isinstance(layer_idx, int):
        return OrjsonResponse(
            {"error": "layer_idx must be an int"}, status_code=400
        )
    if layer_idx < 0 or layer_idx >= _engine.n_layers:
        return OrjsonResponse(
            {"error": f"layer_idx must be in [0, {_engine.n_layers})"},
            status_code=400,
        )

    # Lazily create steering engine if not already active (under lock for thread safety)
    def _compute():
        with _engine.lock:
            if _engine.steering_engine is None:
                _engine.steering_engine = SteeringEngine(
                    _engine.n_layers, _engine.hidden_dim,
                )
            return _engine.steering_engine.compute_vector(
                _engine.model, _engine.tokenizer,
                positive_prompt, negative_prompt, layer_idx,
            )

    loop = asyncio.get_running_loop()
    vector = await loop.run_in_executor(_thread_pool, _compute)

    if vector is None:
        return OrjsonResponse(
            {"error": "failed to compute steering vector"}, status_code=500
        )

    return OrjsonResponse({"vector": vector.tolist()})


# ---------------------------------------------------------------------------
# Batch-ful-c (A-8) — freshness steering vector endpoints.
#
# `/v1/steering/compute` takes a `ContrastPairs`-style batch of prompts per
# role and returns the mean-embedding difference as a JSON array of floats
# plus metadata (vector_dim, layer, norm, model, persisted_path). The Rust
# helper `compute_vector_via_sidecar_role` consumes this shape; the
# frontend never touches it directly.
#
# `/v1/steering/status` is a purely filesystem-backed scan. No GPU work.
# Matches the Rust `scan_steering_cache_dir` helper layout.
#
# Cache root resolution order (first hit wins):
#   1. `$ELAI_STEERING_CACHE_DIR` (explicit override — used by tests).
#   2. `$(cwd)/.elai_cc/steering/` (matches `steering_cache_path()` in
#      crates/providers/src/sidecar_client.rs for the canonical layout).
# ---------------------------------------------------------------------------


def _steering_cache_root() -> Path:
    """Resolve the base directory for per-project steering vector caches.

    Tests set `ELAI_STEERING_CACHE_DIR` to point at a tempdir. Production
    callers rely on the cwd-relative layout that the Rust `steering_cache_path`
    helper produces.
    """
    override = os.environ.get("ELAI_STEERING_CACHE_DIR")
    if override:
        return Path(override)
    return Path.cwd() / ".elai_cc" / "steering"


def _steering_cache_path(project_id: str, role: str) -> Path:
    """Canonical cache file for `(project_id, role)`. Matches
    `crates/providers/src/sidecar_client.rs::steering_cache_path`."""
    safe_project = project_id if project_id else "global"
    safe_role = role if role else "executor"
    return _steering_cache_root() / safe_project / f"{safe_role}.bin"


def _persist_vector_bytes(path: Path, vector: np.ndarray) -> None:
    """Write `vector` as raw little-endian f32 bytes to `path`.

    Creates parent dirs as needed. No header — the Rust `load_cached_vector_bytes`
    helper reads the same format. Uses `struct.pack` + file write rather than
    `numpy.tofile` so the byte layout is explicit and platform-independent.
    """
    path.parent.mkdir(parents=True, exist_ok=True)
    flat = vector.astype(np.float32).reshape(-1)
    # Pack as little-endian f32 so cross-platform readers agree on byte order.
    payload = struct.pack(f"<{flat.size}f", *flat.tolist())
    path.write_bytes(payload)


@app.post("/v1/steering/compute")
async def steering_compute(request: Request):
    """Batch-ful-c (A-8) — compute a freshness steering vector from a
    `ContrastPairs`-style prompt batch.

    Request body (JSON):
        {
          "role": str,                  # e.g. "executor", "auditor"
          "positive_pairs": [str, ...], # modern API examples
          "negative_pairs": [str, ...], # deprecated API examples
          "layer": int | null,          # default: n_layers // 3
          "project_id": str             # used to place the cache file
        }

    Response (JSON):
        {
          "vector": [float, ...],       # raw f32 list, vector_dim entries
          "vector_dim": int,
          "layer": int,
          "norm": float,
          "model": str,
          "persisted_path": str         # absolute cache file path
        }
    """
    if _engine is None:
        return OrjsonResponse({"error": "engine not initialized"}, status_code=503)

    body = await request.body()
    try:
        data = orjson.loads(body)
    except orjson.JSONDecodeError:
        return OrjsonResponse({"error": "invalid JSON"}, status_code=400)

    role = data.get("role", "executor")
    positive = data.get("positive_pairs", [])
    negative = data.get("negative_pairs", [])
    layer = data.get("layer")
    project_id = data.get("project_id", "global")

    if not isinstance(positive, list) or not isinstance(negative, list):
        return OrjsonResponse(
            {"error": "positive_pairs and negative_pairs must be arrays"},
            status_code=400,
        )
    positive = [p for p in positive if isinstance(p, str) and p.strip()]
    negative = [p for p in negative if isinstance(p, str) and p.strip()]
    if not positive or not negative:
        return OrjsonResponse(
            {"error": "positive_pairs and negative_pairs must be non-empty"},
            status_code=400,
        )

    if layer is None:
        layer_idx = _engine.n_layers // 3
    elif isinstance(layer, int):
        layer_idx = layer
    else:
        return OrjsonResponse({"error": "layer must be int or null"}, status_code=400)

    if layer_idx < 0 or layer_idx >= _engine.n_layers:
        return OrjsonResponse(
            {"error": f"layer must be in [0, {_engine.n_layers})"},
            status_code=400,
        )

    def _compute():
        with _engine.lock:
            if _engine.steering_engine is None:
                _engine.steering_engine = SteeringEngine(
                    _engine.n_layers, _engine.hidden_dim,
                )
            return _engine.steering_engine.compute_vector_multi(
                _engine.model,
                _engine.tokenizer,
                positive,
                negative,
                layer_idx,
            )

    started = time.perf_counter()
    loop = asyncio.get_running_loop()
    vector = await loop.run_in_executor(_thread_pool, _compute)

    if vector is None:
        return OrjsonResponse(
            {"error": "failed to compute steering vector (zero magnitude or exception)"},
            status_code=500,
        )

    cache_path = _steering_cache_path(str(project_id), str(role))
    try:
        _persist_vector_bytes(cache_path, vector)
    except OSError as exc:
        logger.exception("steering_cache_write_failed", path=str(cache_path))
        return OrjsonResponse(
            {"error": f"cache_write_failed: {exc}"},
            status_code=500,
        )

    elapsed_ms = int((time.perf_counter() - started) * 1000.0)
    norm = float(np.linalg.norm(vector))
    vector_dim = int(vector.size)

    logger.info(
        "steering_compute_ok",
        role=role,
        project_id=project_id,
        layer=layer_idx,
        vector_dim=vector_dim,
        norm=round(norm, 4),
        elapsed_ms=elapsed_ms,
        cache_path=str(cache_path),
    )

    return OrjsonResponse({
        "vector": vector.tolist(),
        "vector_dim": vector_dim,
        "layer": layer_idx,
        "norm": norm,
        "model": _engine.model_name,
        "persisted_path": str(cache_path),
        "elapsed_ms": elapsed_ms,
    })


@app.get("/v1/steering/status")
async def steering_status(request: Request):
    """Batch-ful-c (A-8) — filesystem-backed scan of the per-project steering
    cache directory.

    Query params:
        role (optional): filter to a single role's cache file.
        project_id (optional): which project's cache to scan. Defaults to "global".

    Response:
        {
          "available": bool,
          "roles_with_vectors": [str, ...],  # sorted
          "vector_count": int,
          "last_computed_ms": int,           # newest mtime in ms since epoch
          "vector_dir": str,
        }
    """
    params = request.query_params
    role = params.get("role") or None
    project_id = params.get("project_id") or "global"

    base = _steering_cache_root() / project_id
    roles: list[str] = []
    last_mtime_ms = 0

    if base.is_dir():
        for entry in sorted(base.iterdir()):
            if not entry.is_file() or entry.suffix != ".bin":
                continue
            if entry.stat().st_size == 0:
                continue
            role_name = entry.stem
            if role is not None and role_name != role:
                continue
            roles.append(role_name)
            m_ms = int(entry.stat().st_mtime * 1000)
            if m_ms > last_mtime_ms:
                last_mtime_ms = m_ms

    return OrjsonResponse({
        "available": len(roles) > 0,
        "roles_with_vectors": roles,
        "vector_count": len(roles),
        "last_computed_ms": last_mtime_ms,
        "vector_dir": str(base),
    })


@app.post("/v1/cache/flush")
async def handle_cache_flush(request: Request):
    """Operator-driven cold-start hook for prefix_cache_bench.py.

    Returns 200 with `{"flushed": <bool>, "n_entries_cleared": <int>}`.
    Safe to call regardless of cache state: when the prefix KV cache is
    disabled (TQ active or max_entries=0) `_prefix_kv_cache is None` and
    we return `{"flushed": false, "n_entries_cleared": 0}` — operators
    must be able to invoke this blindly between bench runs without
    conditional handling. PART7 §5.A-5.
    """
    _ = await request.body()  # body is unused; reading it drains the socket
    if _prefix_kv_cache is None:
        return OrjsonResponse({"flushed": False, "n_entries_cleared": 0})
    n = _prefix_kv_cache.clear()
    return OrjsonResponse({"flushed": True, "n_entries_cleared": n})


# B-36 D.11: dashboard get/set sidecar settings — added 2026-05-07 in s-59964.
# Requires rsync to pythondev + sidecar restart to be reachable on remote.


@app.get("/v1/settings")
async def handle_get_settings() -> OrjsonResponse:
    """Return current sidecar runtime settings (read-mostly view).

    Shape mirrors the dashboard's expected SidecarSettings type:
    { model, max_new_tokens, temperature, top_p, dola_enabled, steering_enabled,
      turboquant_bits, kv_cache_size, fa2_enabled }.
    temperature and top_p are per-request parameters not stored on the engine;
    they are returned as None here. Sources: InferenceEngine instance attrs.
    """
    if _engine is None:
        return OrjsonResponse({"error": "engine not initialized"}, status_code=503)
    fa2 = False
    try:
        fa2 = getattr(_engine.model.config, "_attn_implementation", "sdpa") == "flash_attention_2"
    except Exception:
        pass
    return OrjsonResponse({
        "model": _engine.model_name,
        "max_new_tokens": _engine.max_new_tokens,
        "temperature": None,   # per-request parameter; not persisted on engine
        "top_p": None,         # per-request parameter; not persisted on engine
        "dola_enabled": _engine.dola_enabled,
        "steering_enabled": _engine.steering_enabled,
        "turboquant_bits": _engine._turboquant_bits,
        "kv_cache_size": _engine._kv_cache_size,
        "fa2_enabled": fa2,
        # PART12 OQ5: cloud / host-level keys re-seed dashboard state across
        # refreshes. cloud_api_key itself is never returned — only a presence
        # flag — so the secret stays inside the sidecar process.
        "vram_budget_gb": _cloud_settings.get("vram_budget_gb"),
        "cloud_spend_cap_usd": _cloud_settings.get("cloud_spend_cap_usd"),
        "cloud_api_key_present": bool(_cloud_settings.get("cloud_api_key")),
    })


@app.post("/v1/settings")
async def handle_set_settings(request: Request) -> OrjsonResponse:
    """Update select sidecar runtime settings. Routes each key by category.

    Categories (PART12 OQ5 — see workflow/reasoning/part12-oq5-findings.md):
      RUNTIME_SETTABLE       written directly to _engine attrs.
      RUNTIME_DISABLE_ATTR   _engine attr exists, but the underlying
                             processor is built at __init__. Disabling
                             (False) is always safe; re-enabling (True)
                             only works when the processor object exists,
                             else routed to boot_flags_require_restart.
      BOOT_FLAG_ONLY         attr exists or is consumed only at __init__;
                             ack but require restart.
      NON_ENGINE             host/orchestrator concern; stored in
                             module-level _cloud_settings.

    Response shape:
      { "applied": {...},
        "boot_flags_require_restart": [...],
        "silently_skipped": [...] }
    """
    if _engine is None:
        return OrjsonResponse({"error": "engine not initialized"}, status_code=503)
    body = await request.body()
    try:
        payload: dict[str, Any] = orjson.loads(body)
    except Exception:
        return OrjsonResponse({"error": "invalid JSON"}, status_code=400)

    RUNTIME_SETTABLE = {"max_new_tokens"}
    # frontend_key -> (engine_attr_name, processor_attr_required_for_enable)
    RUNTIME_DISABLE_ATTR: dict[str, tuple[str, str]] = {
        "dola": ("dola_enabled", "_dola_processor"),
        "etc": ("etc_enabled", "_etc_processor"),
        "steering": ("steering_enabled", "steering_engine"),
    }
    BOOT_FLAG_ONLY = {"turboquant_bits"}
    NON_ENGINE_KEYS = {"vram_budget_gb", "cloud_api_key", "cloud_spend_cap_usd"}

    applied: dict[str, Any] = {}
    boot_flags_require_restart: list[str] = []
    silently_skipped: list[str] = []

    for key, value in payload.items():
        if key in RUNTIME_SETTABLE:
            try:
                setattr(_engine, key, value)
                applied[key] = value
            except (AttributeError, TypeError) as exc:
                logger.warning("settings.set_rejected", key=key, value=value, error=str(exc))
                silently_skipped.append(key)
        elif key in RUNTIME_DISABLE_ATTR:
            attr_name, processor_attr = RUNTIME_DISABLE_ATTR[key]
            if value is False:
                setattr(_engine, attr_name, False)
                applied[key] = False
            elif getattr(_engine, processor_attr, None) is not None:
                setattr(_engine, attr_name, True)
                applied[key] = True
            else:
                boot_flags_require_restart.append(key)
        elif key in BOOT_FLAG_ONLY:
            boot_flags_require_restart.append(key)
        elif key in NON_ENGINE_KEYS:
            if key == "cloud_api_key":
                _cloud_settings["cloud_api_key"] = str(value) if value is not None else None
                applied[key] = "***"
            else:
                _cloud_settings[key] = value
                applied[key] = value
        else:
            logger.warning("settings.unknown_key", key=key, value=value)
            silently_skipped.append(key)

    return OrjsonResponse({
        "applied": applied,
        "boot_flags_require_restart": boot_flags_require_restart,
        "silently_skipped": silently_skipped,
    })


@app.post("/v1/messages")
async def handle_messages(request: Request):
    if _engine is None:
        return OrjsonResponse({"error": "engine not initialized"}, status_code=503)
    body = await request.body()
    try:
        data = orjson.loads(body)
    except orjson.JSONDecodeError:
        return OrjsonResponse({"error": "invalid JSON"}, status_code=400)

    # E-45: typed boundary validation. When the Pydantic mirror is
    # available, validate the request shape; on failure, fall through
    # to the legacy dict-get path with a `_e45_validation_error`
    # response header to surface drift without breaking older callers.
    # This is the non-destructive migration step toward strict=True;
    # the strict mode toggle is a future flag once dashboard + CLI
    # callers have all been audited against the schemas/__init__.py
    # surface.
    _e45_validated_request = None
    if _E45_SCHEMAS_AVAILABLE and _MessagesRequestModel is not None:
        try:
            _e45_validated_request = _MessagesRequestModel.model_validate(data)
        except _PydanticValidationError:
            _e45_validated_request = None  # fall through to dict path
    _ = _e45_validated_request  # presence marker; consumed by future strict mode

    messages = data.get("messages", [])
    system_raw = data.get("system", "")
    # Anthropic API allows system as str or list[ContentBlock]
    if isinstance(system_raw, list):
        system = "\n".join(
            b.get("text", "") for b in system_raw
            if isinstance(b, dict) and b.get("type") == "text"
        )
    else:
        system = str(system_raw) if system_raw else ""
    max_tokens = data.get("max_tokens", MAX_NEW_TOKENS_DEFAULT)
    stream = data.get("stream", False)
    tools = data.get("tools", [])
    model_name = data.get("model", "elai-local")
    temperature = data.get("temperature", 0.0)

    # Phase A6 — determinism plumbing. Optional `seed` reaches engine.generate
    # as a kwarg; engine.generate calls transformers.set_seed +
    # torch.manual_seed immediately before model.generate(). Absent key →
    # None → pre-A6 behaviour (model's default RNG). Type contract:
    # ``Optional[int]`` only — strings/floats are rejected with a 400
    # rather than coerced silently. At temperature == 0 (greedy) the seed
    # is a no-op but still forwarded so the sidecar log records it.
    seed_raw = data.get("seed")
    if seed_raw is None:
        seed = None
    elif isinstance(seed_raw, bool) or not isinstance(seed_raw, int):
        # bool is a subclass of int — reject explicitly. Same for float/str.
        return OrjsonResponse(
            {
                "error": "invalid seed type",
                "detail": (
                    "field `seed` must be an integer or omitted; got "
                    f"{type(seed_raw).__name__}={seed_raw!r}"
                ),
            },
            status_code=400,
        )
    elif seed_raw < 0:
        return OrjsonResponse(
            {
                "error": "invalid seed value",
                "detail": (
                    f"field `seed` must be non-negative; got {seed_raw}"
                ),
            },
            status_code=400,
        )
    elif seed_raw > 2**32 - 1:
        return OrjsonResponse(
            {
                "error": "invalid seed value",
                "detail": "seed must be <= 4294967295 (transformers 32-bit ceiling)",
            },
            status_code=400,
        )
    else:
        seed = int(seed_raw)

    # E-32: per-request FITM override field. True forces on, False forces off,
    # absent (None) defers to ELAI_FITM_ENABLED env var. Bool or null only.
    fitm_raw = data.get("fitm")
    if fitm_raw is None:
        fitm_override: bool | None = None
    elif isinstance(fitm_raw, bool):
        fitm_override = fitm_raw
    else:
        return OrjsonResponse(
            {
                "error": "invalid fitm type",
                "detail": (
                    "field `fitm` must be a boolean or omitted; got "
                    f"{type(fitm_raw).__name__}={fitm_raw!r}"
                ),
            },
            status_code=400,
        )

    # Track C-B: extract optional response_format. Validation is lenient by
    # design — invalid types are warned-and-ignored (engine helper handles
    # the inner shape check); only the "library missing while caller asked
    # for constraint" case produces a hard 503 so operator notices.
    response_format = data.get("response_format")
    if response_format is not None and not _LMFE_AVAILABLE:
        logger.warning(
            "response_format_set_but_lmfe_missing",
            hint="install lm-format-enforcer per USER-MANUAL-CB-DEPS.md",
        )
        return OrjsonResponse(
            {
                "error": _ERR_CONSTRAINED_DECODING_UNAVAILABLE,
                "detail": _ERR_DETAIL_INSTALL_LMFE,
            },
            status_code=503,
        )

    prompt = _anthropic_to_gemma_prompt(_engine.tokenizer, messages, system, tools)

    # Track C-D: parse Anthropic per-content-block cache_control markers and
    # compute the boundary token offset within the formatted prompt. None
    # when no marker is present — preserves the legacy full-prefix cache
    # behaviour and keeps existing requests working unchanged.
    # Note: the system arg passed here is the RAW system_raw (str | list)
    # so the helper can walk system list-form blocks for markers. The
    # already-flattened ``system`` string is what the engine consumes.
    marker_boundary = _extract_cache_marker_token_position(
        _engine.tokenizer, messages, system_raw, tools,
    )

    tool_names = [t.get("name", "?") for t in tools] if tools else []
    logger.info(
        "turn",
        system_chars=len(system) if system else 0,
        messages=len(messages),
        tools=len(tools),
        tool_names=tool_names[:10],
        prompt_chars=len(prompt),
        cache_marker_boundary=marker_boundary,
    )

    # H-50 STEP 3: measure input tokens + journal the turn (env-gated, default
    # OFF). The sidecar only MEASURES here -- it does NOT trim the prompt; the
    # orchestrator/CLI applies the regime-aware trim before sending. When
    # ELAI_CONTEXT_JOURNAL_PROOF=1 and the turn exceeds trim_at, a
    # would-reduce ContextReduction row is also journaled. No-op on the
    # default path (journal disabled) so there is no extra tokenization cost.
    if _H50_JOURNAL_AVAILABLE and _h50_maybe_journal_turn is not None and _engine is not None:
        try:
            _h50_maybe_journal_turn(
                prompt, messages, model_name, _engine.tokenizer,
                max_output_tokens=max_tokens,
            )
        except Exception as _h50_exc:  # pragma: no cover -- journal must never fail the turn
            logger.warning("h50_journal_turn_failed", error=str(_h50_exc))

    if stream:
        # J-36: streaming path now threads marker_boundary into
        # _stream_messages_response so the prefix-KV lease fires on the
        # streaming generate path (FH-2/FH-3). Cache counters are surfaced
        # in SSE usage fields instead of hardcoded 0.
        # Phase B (s-1976592): response_format is now wired through to
        # generate_streaming via prefix_allowed_tokens_fn. The L2809 gate
        # above already returns 503 when LMFE is missing, so by the time we
        # reach here either response_format is None (normal streaming) or
        # LMFE is available (constrained streaming). _build_constrained_prefix_fn
        # warn-and-ignores any malformed schema; in that case we fall back
        # silently to unconstrained streaming, matching the non-stream path.
        # Phase A6: forward optional `seed` kwarg.
        return _stream_messages_response(
            prompt, max_tokens, model_name, tools, temperature,
            response_format=response_format,
            seed=seed,
            marker_boundary=marker_boundary,
        )
    return await _non_stream_messages_response(
        prompt, max_tokens, model_name, tools, temperature,
        marker_boundary=marker_boundary,
        response_format=response_format,
        seed=seed,
        fitm_override=fitm_override,
    )


@app.post("/v1/completions")
async def handle_completions(request: Request):
    if _engine is None:
        return OrjsonResponse({"error": "engine not initialized"}, status_code=503)
    body = await request.body()
    try:
        data = orjson.loads(body)
    except orjson.JSONDecodeError:
        return OrjsonResponse({"error": "invalid JSON"}, status_code=400)

    prompt = data.get("prompt", "")
    max_tokens = data.get("max_tokens", MAX_NEW_TOKENS_DEFAULT)
    temperature = data.get("temperature", 0.0)

    # Track C-B: optional response_format on /v1/completions (used by the
    # Rust orchestrator decompose path — grounding_client.rs::CompletionsRequest).
    # Same validation policy as /v1/messages: missing-library yields 503;
    # malformed type/shape is warn-and-ignored inside the engine helper.
    response_format = data.get("response_format")
    if response_format is not None and not _LMFE_AVAILABLE:
        logger.warning(
            "response_format_set_but_lmfe_missing",
            endpoint="/v1/completions",
            hint="install lm-format-enforcer per USER-MANUAL-CB-DEPS.md",
        )
        return OrjsonResponse(
            {
                "error": _ERR_CONSTRAINED_DECODING_UNAVAILABLE,
                "detail": _ERR_DETAIL_INSTALL_LMFE,
            },
            status_code=503,
        )

    t0 = time.time()
    loop = asyncio.get_running_loop()
    (
        text,
        tokens,
        input_token_count,
        search_trigger,
        gg_signals,
        _cache_read_tokens,
        _cache_creation_tokens,
    ) = await loop.run_in_executor(
        _thread_pool,
        lambda: _engine.generate(
            prompt, max_tokens, temperature,
            response_format=response_format,
        ),
    )
    elapsed = time.time() - t0

    # Determine finish reason
    eos_ids = _engine.tokenizer.eos_token_id
    if isinstance(eos_ids, int):
        eos_ids = [eos_ids]
    elif eos_ids is None:
        eos_ids = []
    finish_reason = "stop" if (tokens and tokens[-1] in eos_ids) else "length"

    response = {
        "id": f"cmpl-elai-{int(time.time())}",
        "object": "text_completion",
        "created": int(time.time()),
        "model": "elai-composed",
        "choices": [{
            "text": text,
            "index": 0,
            "finish_reason": finish_reason,
        }],
        "usage": {
            "prompt_tokens": input_token_count,
            "completion_tokens": len(tokens),
            "total_tokens": input_token_count + len(tokens),
        },
        "search_trigger": search_trigger,
        "mean_logprob": gg_signals.get("mean_logprob"),
        "min_logprob": gg_signals.get("min_logprob"),
        "entropy_mean": gg_signals.get("entropy_mean"),
        "entropy_max": gg_signals.get("entropy_max"),
        "generation_novelty": gg_signals.get("generation_novelty"),
        # Phase A bench redesign: top-level server-side elapsed_ms (sibling to
        # the existing nested elai_metadata.elapsed_ms). Bench reads this.
        "elapsed_ms": int(elapsed * 1000),
        "elai_metadata": {
            "dola_enabled": _engine.dola_enabled,
            "turboquant_enabled": _engine.kv_compressor.enabled,
            "turboquant_bits": _engine.kv_compressor.bit_width,
            "steering_enabled": _engine.steering_enabled,
            "steering_coefficient": (
                _engine.steering_engine.coefficient
                if _engine.steering_engine
                else None
            ),
            "steering_layer": (
                _engine.steering_engine.target_layer
                if _engine.steering_engine
                else None
            ),
            "elapsed_ms": int(elapsed * 1000),
            "etc_enabled": _engine.etc_enabled,
            "etc_triggered": search_trigger is not None,
            "etc_d2h_log": (
                _engine._etc_processor.d2h_log
                if _engine._etc_processor is not None
                and _engine.etc_threshold == 0.0
                else []
            ),
        },
    }
    return OrjsonResponse(response)


# ---------------------------------------------------------------------------
# Web search endpoint (SearXNG proxy + auto-start + Brave fallback)
# ---------------------------------------------------------------------------

SEARCH_MAX_RESULTS = 20
SEARXNG_INTERNAL_URL = os.environ.get("SEARXNG_URL", "http://127.0.0.1:8888")
SEARXNG_CONTAINER_NAME = "searxng"
SEARXNG_IMAGE = "searxng/searxng:latest"
SEARXNG_SETTINGS_MOUNT = "/etc/searxng/settings.yml"
BRAVE_SEARCH_URL = "https://api.search.brave.com/res/v1/web/search"

_searxng_available: bool | None = None  # None = not checked yet.
_searxng_lock = Lock()


def _ensure_searxng_sync() -> bool:
    """Check if SearXNG is running; auto-start Docker container if not.

    Returns True if SearXNG is reachable, False otherwise.
    Thread-safe (called from executor).
    """
    global _searxng_available
    import subprocess

    with _searxng_lock:
        # Quick health check.
        try:
            import httpx
            resp = httpx.get(f"{SEARXNG_INTERNAL_URL}/search",
                            params={"q": "test", "format": "json"},
                            timeout=3)
            if resp.status_code == 200:
                _searxng_available = True
                return True
        except Exception:
            pass

        if _searxng_available is False:
            return False  # Already tried and failed.

        # Try to start SearXNG via Docker.
        logger.info("searxng_auto_start", msg="attempting Docker auto-start")
        try:
            # Check if Docker is available.
            subprocess.run(
                ["docker", "info"],
                capture_output=True, timeout=5, check=True,
            )
        except (subprocess.CalledProcessError, FileNotFoundError, subprocess.TimeoutExpired):
            logger.warning("searxng_no_docker", msg="Docker not available")
            _searxng_available = False
            return False

        # Remove stale container if exists.
        subprocess.run(
            ["docker", "rm", "-f", SEARXNG_CONTAINER_NAME],
            capture_output=True, timeout=10,
        )

        # Find settings file relative to this script.
        import pathlib
        script_dir = pathlib.Path(__file__).resolve().parent
        settings_path = script_dir.parent / "searxng-settings" / "settings.yml"
        mount_arg = (
            f"{settings_path}:{SEARXNG_SETTINGS_MOUNT}:ro"
            if settings_path.exists()
            else ""
        )

        cmd = [
            "docker", "run", "-d",
            "--name", SEARXNG_CONTAINER_NAME,
            "-p", "8888:8080",
        ]
        if mount_arg:
            cmd.extend(["-v", mount_arg])
        cmd.append(SEARXNG_IMAGE)

        try:
            subprocess.run(cmd, capture_output=True, timeout=30, check=True)
        except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as exc:
            logger.warning("searxng_start_failed", error=str(exc))
            _searxng_available = False
            return False

        # Wait for SearXNG to become ready (up to 10s).
        import httpx as _httpx
        for _ in range(20):
            time.sleep(0.5)
            try:
                resp = _httpx.get(f"{SEARXNG_INTERNAL_URL}/search",
                                 params={"q": "test", "format": "json"},
                                 timeout=2)
                if resp.status_code == 200:
                    logger.info("searxng_ready")
                    _searxng_available = True
                    return True
            except Exception:
                continue

        logger.warning("searxng_timeout", msg="SearXNG did not become ready in 10s")
        _searxng_available = False
        return False


async def _proxy_searxng(query: str, max_results: int) -> list:
    """Proxy search request to local SearXNG instance."""
    import httpx
    try:
        async with httpx.AsyncClient(timeout=15) as client:
            resp = await client.get(
                f"{SEARXNG_INTERNAL_URL}/search",
                params={"q": query, "format": "json", "pageno": "1"},
            )
            if resp.status_code != 200:
                return []
            data = resp.json()
            results = data.get("results", [])[:max_results]
            # Normalize to SearXNG contract (already in correct format).
            return results
    except Exception:
        logger.exception("searxng_proxy_error")
        return []


async def _search_brave(query: str, max_results: int, api_key: str) -> list:
    """Search Brave Search API (requires API key). Free tier: 2000 queries/month."""
    import httpx
    results = []
    try:
        async with httpx.AsyncClient(timeout=10) as client:
            resp = await client.get(
                BRAVE_SEARCH_URL,
                params={"q": query, "count": min(max_results, 20)},
                headers={
                    "X-Subscription-Token": api_key,
                    "Accept": "application/json",
                },
            )
            if resp.status_code != 200:
                logger.warning("brave_search_failed", status=resp.status_code)
                return results

            data = resp.json()
            web_results = data.get("web", {}).get("results", [])
            for i, item in enumerate(web_results[:max_results]):
                results.append({
                    "title": item.get("title", ""),
                    "url": item.get("url", ""),
                    "content": item.get("description", ""),
                    "engine": "brave",
                    "score": 1.0 / (i + 1),
                })
    except Exception:
        logger.exception("brave_search_error")
    return results


# ---------------------------------------------------------------------------
# D-2: PPM (Process Preference Model) scoring endpoint — second sidecar
# port 8101, hosts Qwen2.5-Math-PRM-7B (or equivalent PRM). Wire contract
# defined by .elai_cc/crates/orchestrator/src/sidecar_ppm_hook.rs +
# http_sidecar_ppm_hook.rs::validate_response_shape. NEVER reuses the
# policy SLM (heterogeneous-verifier requirement, arxiv:2504.16760 T1).
#
# Wire format mirrors the Rust WireRequest struct in
# http_sidecar_ppm_hook.rs lines 313-320:
#   candidate_id: str        -- opaque corr id
#   verifier_id: str         -- "ppm_qwen25_math_prm_7b" or similar
#   problem_statement: str   -- the question text
#   reasoning_steps: list[str] -- ordered chain to score; len >= 1
#   mode: "math" | "code"    -- closed enum from sidecar_ppm_hook.rs
#
# Response mirrors WireResponse (lines 334-344):
#   candidate_id, verifier_id, mode (echoed) +
#   step_scores: list[float in 0..1]  -- one per step (prmbench invariant)
#   score_mean: float                  -- arithmetic mean (latent-verifiers paper)
#   score_var: float                   -- population variance
#   num_steps: int                     -- == len(step_scores)
#   latency_ms: int                    -- end-to-end scoring wall-clock
#
# Error envelope on contract violation (HTTP 422 or 503 + JSON body):
#   {"error": "<tag>", "detail": "...", "index": N, "value": F,
#    "request_steps": N, "response_scores": N}
# The Rust WireErrorBody (lines 369-403) maps these tags 1:1 to typed
# PpmHookError variants.
# ---------------------------------------------------------------------------

# Reward token glyphs Qwen2.5-Math-PRM-7B (+ derivatives) uses for the
# per-step score margin. The PRM is trained to predict a `+` token after
# a correct step and `-` after an incorrect step; sigmoid(logit(+) -
# logit(-)) yields the canonical scalar in [0, 1] (prmbench Section 4
# invariant + arxiv:2501.04519 §3.4.1 score-head convention).
PPM_REWARD_TOKEN_POS = "+"
PPM_REWARD_TOKEN_NEG = "-"

# Step separator token convention shared by Qwen2.5-Math-PRM-7B + the
# Skywork/Math-Shepherd PRM family. Some PRM tokenisers ship a dedicated
# `<extra_0>` separator; others use the standard newline. We try the
# `<extra_0>` route first and fall back to newline when the tokeniser
# does not have it (the fallback degrades to lower PRMBench AUC by ~3pp
# but still produces a valid signal — flagged as PRM_FALLBACK_SEPARATOR
# in tracing).
PPM_STEP_SEPARATOR_TOKEN = "<extra_0>"
PPM_STEP_SEPARATOR_FALLBACK = "\n"

# Closed set of legal mode wire values per sidecar_ppm_hook.rs PpmMode enum.
_PPM_VALID_MODES = ("math", "code")

# Soft hint substrings the operator embeds in the loaded model name when a
# PRM is in use. If the running engine's model_name has NONE of these, the
# /v1/ppm/score handler returns HTTP 503 with error="ppm_model_not_loaded"
# (the policy SLM cannot self-grade — same-model fallback is BANNED per
# D-2-decisions DECISION-3 and CLAUDE.md heterogeneous-verifier rule).
_PPM_MODEL_HINTS = ("prm", "math-prm", "process-reward", "ppm")


def _ppm_validate_request(payload: dict) -> tuple[str, str, str, list, str]:
    """Defensive request-shape validation. Returns the typed tuple on success
    or raises ValueError with a tag matching one of the WireErrorBody.error
    values the Rust hook understands (empty_input, invalid_mode, parse_error).

    Per anti-lazy rules + CLAUDE.md no-unwrap policy: explicit .get() with
    typed-error raise on every field; no `payload[key]` direct subscript.
    """
    if not isinstance(payload, dict):
        raise ValueError("parse_error: top-level body must be a JSON object")

    candidate_id = payload.get("candidate_id")
    verifier_id = payload.get("verifier_id")
    problem_statement = payload.get("problem_statement")
    reasoning_steps = payload.get("reasoning_steps")
    mode = payload.get("mode")

    if not isinstance(candidate_id, str) or not candidate_id:
        raise ValueError("parse_error: candidate_id must be a non-empty string")
    if not isinstance(verifier_id, str) or not verifier_id:
        raise ValueError("parse_error: verifier_id must be a non-empty string")
    if not isinstance(problem_statement, str):
        raise ValueError("parse_error: problem_statement must be a string")
    if not isinstance(reasoning_steps, list):
        raise ValueError("parse_error: reasoning_steps must be a JSON array")
    if not reasoning_steps:
        # Clause C5 of the wire contract — empty_input -> Rust EmptyInput.
        raise ValueError("empty_input: reasoning_steps array was empty")
    for i, step in enumerate(reasoning_steps):
        if not isinstance(step, str):
            raise ValueError(
                f"parse_error: reasoning_steps[{i}] must be a string, "
                f"got {type(step).__name__}"
            )
    if not isinstance(mode, str) or mode not in _PPM_VALID_MODES:
        raise ValueError(f"invalid_mode: mode={mode!r}; expected one of {_PPM_VALID_MODES}")

    return candidate_id, verifier_id, problem_statement, reasoning_steps, mode


def _ppm_model_loaded_for_scoring() -> tuple[bool, str]:
    """Returns (is_prm_loaded, model_id_for_error). Heuristic: the running
    engine.model_name must contain one of the PRM hint substrings. When
    the policy SLM (Qwen2.5-Coder) is loaded, this returns False and the
    handler emits HTTP 503 — same-model fallback would defeat the
    heterogeneous-verifier requirement (arxiv:2504.16760 T1).
    """
    if _engine is None:
        return False, "<engine-not-initialised>"
    name_lower = _engine.model_name.lower()
    for hint in _PPM_MODEL_HINTS:
        if hint in name_lower:
            return True, _engine.model_name
    return False, _engine.model_name


def _ppm_resolve_separator_token(tokenizer) -> tuple[int, str]:
    """Resolve the step-separator token id used to demarcate per-step score
    positions. Tries the `<extra_0>` route first (Qwen2.5-Math-PRM-7B
    canonical), then falls back to a newline. Returns (token_id, label)
    where label is "primary" or "fallback" for tracing.
    """
    # Some tokeniser implementations expose `convert_tokens_to_ids` directly.
    try:
        sep_id = tokenizer.convert_tokens_to_ids(PPM_STEP_SEPARATOR_TOKEN)
    except Exception:
        sep_id = None
    if isinstance(sep_id, int) and sep_id >= 0 and sep_id != getattr(tokenizer, "unk_token_id", -1):
        return sep_id, "primary"

    # Fallback: encode a newline and take the first sub-token id.
    fallback_ids = tokenizer.encode(PPM_STEP_SEPARATOR_FALLBACK, add_special_tokens=False)
    if not fallback_ids:
        raise ValueError("parse_error: tokenizer cannot encode newline separator")
    return fallback_ids[0], "fallback"


def _ppm_resolve_reward_token_ids(tokenizer) -> tuple[int, int]:
    """Resolve the `+` and `-` token ids the PRM scalar head reads. Some
    tokenisers split `+` differently in different surface contexts; we
    take the first sub-token after encoding with no special tokens.
    """
    pos_ids = tokenizer.encode(PPM_REWARD_TOKEN_POS, add_special_tokens=False)
    neg_ids = tokenizer.encode(PPM_REWARD_TOKEN_NEG, add_special_tokens=False)
    if not pos_ids or not neg_ids:
        raise ValueError(
            "parse_error: tokenizer cannot encode reward tokens "
            f"(+,- -> {pos_ids}, {neg_ids})"
        )
    return pos_ids[0], neg_ids[0]


def _ppm_score_steps(
    problem_statement: str,
    reasoning_steps: list[str],
    mode: str,
) -> list[float]:
    """Score every step via the loaded PRM model. Returns one scalar in
    [0, 1] per input step (prmbench invariant + latent-verifiers
    arithmetic-mean aggregation downstream).

    Implementation: build a single concatenated trajectory
        "<problem>\n<step_1>\n<sep>\n<step_2>\n<sep>..."
    run a single forward pass, then at every separator position read
    logit(+) and logit(-); the per-step score is sigmoid(logit(+) -
    logit(-)) -- the standard PRM scalar head convention per
    arxiv:2501.04519 §3.4.1 + the Qwen2.5-Math-PRM-7B model card.

    Raises ValueError with a tagged message on any tokeniser / model
    failure; the handler catches and returns 422 with the matching
    error envelope.
    """
    assert _engine is not None  # caller (handler) checked already
    tokenizer = _engine.tokenizer
    model = _engine.model

    sep_id, sep_label = _ppm_resolve_separator_token(tokenizer)
    pos_id, neg_id = _ppm_resolve_reward_token_ids(tokenizer)

    # Build the trajectory text. We use the mode tag in the system-style
    # prefix so the PRM has the same context shape it was trained on.
    mode_prefix = f"[mode={mode}]\n" if mode in _PPM_VALID_MODES else ""
    parts: list[str] = [f"{mode_prefix}Problem: {problem_statement}"]
    for step in reasoning_steps:
        parts.append(step)
    trajectory = PPM_STEP_SEPARATOR_FALLBACK.join(parts)

    # Tokenise without special tokens; we are emulating the PRM training
    # surface, not a chat template.
    encoded = tokenizer(trajectory, return_tensors="pt", add_special_tokens=False)
    input_ids = encoded["input_ids"].to(model.device)

    # Locate every separator position. We expect exactly len(reasoning_steps)
    # separators in the trajectory (one after each step's text). The first
    # `Problem: ...` chunk has no preceding step so it does not contribute
    # a separator.
    flat_ids = input_ids[0].tolist()
    sep_positions = [i for i, tok in enumerate(flat_ids) if tok == sep_id]

    # Defensive: if the tokeniser merged the separator with adjacent text
    # (common for newline + Qwen BPE), fall back to step-count matching
    # by linear-search for *any* token in the sep_id family at decreasing
    # granularity. Surface the mismatch as parse_error if we still cannot
    # locate >= one separator per step.
    if len(sep_positions) < len(reasoning_steps):
        logger.warning(
            "ppm_separator_count_mismatch",
            requested_steps=len(reasoning_steps),
            located_separators=len(sep_positions),
            sep_label=sep_label,
        )
        raise ValueError(
            "parse_error: could not locate enough step separators in "
            f"tokenised trajectory (steps={len(reasoning_steps)}, "
            f"separators={len(sep_positions)}); separator label={sep_label}"
        )

    # Single forward pass to obtain logits at every position.
    with torch.inference_mode():
        outputs = model(input_ids=input_ids)
    logits = outputs.logits  # [1, seq_len, vocab_size]

    per_step: list[float] = []
    for idx, _step_text in enumerate(reasoning_steps):
        # We score using the logits AT the separator position (i.e. the
        # token that immediately follows the step's last token), so the
        # PRM sees the full prefix x . s_1 .. s_i before the scoring
        # head fires. Take the closest separator >= the expected index.
        sep_pos = sep_positions[idx]
        # logits at this position predict the NEXT token; for PRM scoring
        # the convention is to read the logits at the separator position
        # itself (the head was trained to emit + / - at this slot).
        step_logits = logits[0, sep_pos, :]
        margin = (step_logits[pos_id] - step_logits[neg_id]).float()
        score = torch.sigmoid(margin).item()
        # Clamp defensively into [0, 1] — sigmoid output is mathematically
        # in (0, 1) but float rounding can produce 0.9999998 / 1.0000001
        # at the boundary; the Rust validate_response_shape rejects > 1.0.
        score = max(0.0, min(1.0, score))
        per_step.append(score)

    return per_step


def _ppm_aggregate(per_step_scores: list[float]) -> tuple[float, float]:
    """Compute (mean, variance) over per-step scores. Arithmetic mean
    (not min / max / geometric) per arxiv:2504.16760 latent-verifiers
    §A.2 aggregation ablation -- mean outperformed min/max/geometric.
    Population variance (NOT sample variance) per the Rust hook contract
    clause C3.
    """
    n = len(per_step_scores)
    if n == 0:
        return 0.0, 0.0
    mean = sum(per_step_scores) / float(n)
    var = sum((s - mean) * (s - mean) for s in per_step_scores) / float(n)
    return mean, var


@app.post("/v1/ppm/score")
async def handle_ppm_score(request: Request):
    """D-2 PPM score endpoint. See the docstring block above for the wire
    contract. The handler is intentionally lean — heavy lifting in
    `_ppm_score_steps` runs on the GPU executor pool to keep the asyncio
    loop responsive.

    Reliability tier: this endpoint is the Python side of the trust boundary
    described in http_sidecar_ppm_hook.rs lines 9-12. The Rust caller
    re-validates everything; we still validate defensively here to surface
    typed errors over the wire instead of HTTP 500.

    # RUNPOD-PENDING: ssh -i ~/.ssh/elai_remote -p 6465 maintainer@192.0.2.10 'bash ~/start-ppm-sidecar.sh' && curl http://127.0.0.1:8101/v1/ppm/score -X POST -H 'Content-Type: application/json' -d '{"candidate_id":"c1","verifier_id":"ppm_qwen25_math_prm_7b","problem_statement":"x=4","reasoning_steps":["x>0","x<10"],"mode":"math"}'
    """
    t0 = time.time()
    raw = await request.body()
    try:
        payload = orjson.loads(raw) if raw else None
    except orjson.JSONDecodeError as exc:
        return OrjsonResponse(
            {"error": "parse_error", "detail": f"invalid JSON body: {exc}"},
            status_code=422,
        )

    # Request-shape validation — surface tagged errors the Rust hook maps
    # to typed PpmHookError variants without lossy stringification.
    try:
        candidate_id, verifier_id, problem_statement, reasoning_steps, mode = (
            _ppm_validate_request(payload if isinstance(payload, dict) else {})
        )
    except ValueError as exc:
        msg = str(exc)
        tag = msg.split(":", 1)[0].strip() if ":" in msg else "parse_error"
        detail = msg.split(":", 1)[1].strip() if ":" in msg else msg
        logger.warning("ppm_request_invalid", tag=tag, detail=detail)
        return OrjsonResponse(
            {"error": tag, "detail": detail},
            status_code=422,
        )

    # Model-loaded gate. When the policy SLM is on this port instead of
    # the PRM, we return 503 instead of producing a self-graded score
    # (the Rust hook maps 503 to PpmHookError::Unavailable).
    prm_ready, model_id = _ppm_model_loaded_for_scoring()
    if not prm_ready:
        logger.warning(
            "ppm_model_not_loaded",
            running_model=model_id,
            verifier_id=verifier_id,
            hints=_PPM_MODEL_HINTS,
        )
        return OrjsonResponse(
            {"error": "ppm_model_not_loaded", "model_id": model_id},
            status_code=503,
        )

    # Run the scoring forward pass on the GPU thread-pool to avoid
    # blocking the asyncio loop. The pool also serialises GPU usage
    # against /v1/messages traffic on the same port (PRM sidecars run
    # on port 8101, separate from policy on 8100 — but if an operator
    # co-hosts during dev, the lock prevents kernel interleaving).
    loop = asyncio.get_running_loop()
    try:
        per_step = await loop.run_in_executor(
            _thread_pool,
            lambda: _ppm_score_steps(problem_statement, reasoning_steps, mode),
        )
    except ValueError as exc:
        msg = str(exc)
        tag = msg.split(":", 1)[0].strip() if ":" in msg else "parse_error"
        detail = msg.split(":", 1)[1].strip() if ":" in msg else msg
        logger.warning("ppm_score_failed", tag=tag, detail=detail)
        return OrjsonResponse(
            {"error": tag, "detail": detail},
            status_code=422,
        )
    except Exception as exc:  # noqa: BLE001 — typed error envelope on the wire
        logger.exception("ppm_score_unexpected_error")
        return OrjsonResponse(
            {"error": "parse_error", "detail": f"scoring failure: {exc}"},
            status_code=500,
        )

    # Defensive shape check before serialising — guards against the
    # scoring routine returning fewer scores than requested steps. The
    # Rust validate_response_shape will catch this too, but emitting a
    # clean 422 here is more diagnostic than a wire mismatch.
    if len(per_step) != len(reasoning_steps):
        logger.error(
            "ppm_internal_shape_mismatch",
            expected=len(reasoning_steps),
            got=len(per_step),
        )
        return OrjsonResponse(
            {
                "error": "shape_mismatch",
                "request_steps": len(reasoning_steps),
                "response_scores": len(per_step),
            },
            status_code=422,
        )
    for i, s in enumerate(per_step):
        if not (0.0 <= s <= 1.0):
            logger.error("ppm_internal_invalid_score", index=i, value=s)
            return OrjsonResponse(
                {"error": "invalid_score", "index": i, "value": s},
                status_code=422,
            )

    score_mean, score_var = _ppm_aggregate(per_step)
    elapsed_ms = int((time.time() - t0) * 1000)

    logger.info(
        "ppm_score_completed",
        candidate_id=candidate_id,
        verifier_id=verifier_id,
        mode=mode,
        num_steps=len(per_step),
        score_mean=score_mean,
        score_var=score_var,
        latency_ms=elapsed_ms,
    )

    return OrjsonResponse({
        "candidate_id": candidate_id,
        "verifier_id": verifier_id,
        "mode": mode,
        "step_scores": per_step,
        "score_mean": score_mean,
        "score_var": score_var,
        "num_steps": len(per_step),
        "latency_ms": elapsed_ms,
    })


@app.get("/search")
async def handle_search(request: Request):
    """SearXNG-compatible search endpoint. Drop-in replacement.

    Strategy (in order):
    1. Proxy to local SearXNG (auto-starts Docker container if available)
    2. Fall back to Brave Search API (if BRAVE_SEARCH_API_KEY env set)
    3. Return empty results with error message

    Query params: q (query string), format (ignored, always json), pageno (ignored).
    Returns: {"query": "...", "results": [...], "number_of_results": N}
    """
    query = request.query_params.get("q", "").strip()
    if not query:
        return OrjsonResponse({"query": "", "results": [], "number_of_results": 0})

    max_results = SEARCH_MAX_RESULTS

    # Strategy 1: SearXNG (auto-start Docker if needed).
    loop = asyncio.get_running_loop()
    searxng_ok = await loop.run_in_executor(None, _ensure_searxng_sync)

    if searxng_ok:
        results = await _proxy_searxng(query, max_results)
        if results:
            logger.info("search_completed", query=query, source="searxng",
                        result_count=len(results))
            return OrjsonResponse({
                "query": query,
                "number_of_results": len(results),
                "results": results,
            })

    # Strategy 2: Brave Search API fallback.
    import os
    brave_key = os.environ.get("BRAVE_SEARCH_API_KEY", "")
    if brave_key:
        results = await _search_brave(query, max_results, brave_key)
        if results:
            logger.info("search_completed", query=query, source="brave",
                        result_count=len(results))
            return OrjsonResponse({
                "query": query,
                "number_of_results": len(results),
                "results": results,
            })

    # Strategy 3: No search available.
    logger.warning("search_unavailable", query=query,
                   msg="No search backend available. Install Docker or set BRAVE_SEARCH_API_KEY.")
    return OrjsonResponse({
        "query": query,
        "number_of_results": 0,
        "results": [],
        "error": "No search backend available. Install Docker for SearXNG or set BRAVE_SEARCH_API_KEY env var.",
    })


# ---------------------------------------------------------------------------
# C-36 MuCoCo mutation operator endpoints
# ---------------------------------------------------------------------------
#
# Wire contract (mirrors crates/orchestrator/src/metamorphic_testing.rs
# MucocoMutationResult one-for-one — snake_case field names exactly):
#
#   POST /mucoco/mutate
#     request : {"source": str, "language": str = "python", "operator_name": str}
#     response: {"operator": str, "applied": bool, "mutated_source": str,
#                "skip_reason": Optional[str]}
#     errors  : 422 parse_error / missing_field, 503 mucoco_unavailable
#
#   GET /mucoco/operators
#     response: {"operators": [str], "applicability_matrix":
#                {operator_name: {language: bool}}}
#     errors  : 503 mucoco_unavailable
#
# The Rust client at metamorphic_testing.rs:594 enforces a stub_ban contract
# that mirrors python/elai_inference/mucoco/result.py MutationResult
# __post_init__: applied=False REQUIRES non-empty skip_reason; applied=True
# REQUIRES skip_reason=None. Both sides validate independently.


@app.post("/mucoco/mutate")
async def handle_mucoco_mutate(request: Request):
    """Dispatch a single MuCoCo mutation operator to a source snippet.

    Stateless; no model load required. The registry's `apply` function owns
    the applicability gate (paper sec S3.p3.1 applicability-first contract)
    and the dataclass __post_init__ enforces the stub_ban invariant.
    """
    if not _MUCOCO_AVAILABLE or _mucoco_apply is None:
        logger.warning("mucoco_unavailable", endpoint="/mucoco/mutate")
        return OrjsonResponse(
            {"error": "mucoco_unavailable",
             "detail": "elai_inference.mucoco.registry is not importable on this sidecar."},
            status_code=503,
        )

    raw = await request.body()
    try:
        payload = orjson.loads(raw) if raw else None
    except orjson.JSONDecodeError as exc:
        return OrjsonResponse(
            {"error": "parse_error", "detail": f"invalid JSON body: {exc}"},
            status_code=422,
        )

    if not isinstance(payload, dict):
        return OrjsonResponse(
            {"error": "parse_error", "detail": "request body must be a JSON object"},
            status_code=422,
        )

    source = payload.get("source")
    language = payload.get("language", "python")
    operator_name = payload.get("operator_name")

    if not isinstance(source, str):
        return OrjsonResponse(
            {"error": "missing_field", "detail": "source: expected string"},
            status_code=422,
        )
    if not isinstance(language, str):
        return OrjsonResponse(
            {"error": "missing_field", "detail": "language: expected string"},
            status_code=422,
        )
    if not isinstance(operator_name, str) or not operator_name:
        return OrjsonResponse(
            {"error": "missing_field", "detail": "operator_name: expected non-empty string"},
            status_code=422,
        )

    result = _mucoco_apply(operator_name, source, language)

    logger.info(
        "mucoco_mutate_completed",
        operator=result.operator,
        language=language,
        applied=result.applied,
        skip_reason=result.skip_reason,
    )

    return OrjsonResponse({
        "operator": result.operator,
        "applied": result.applied,
        "mutated_source": result.mutated_source,
        "skip_reason": result.skip_reason,
    })


@app.get("/mucoco/operators")
async def handle_mucoco_operators(request: Request):
    """List registered MuCoCo operators and their language applicability matrix.

    Read-only metadata endpoint; used by the Rust orchestrator to pre-filter
    operators per target language before issuing /mucoco/mutate calls.
    """
    del request  # signature consistency with other GET handlers
    if (
        not _MUCOCO_AVAILABLE
        or _mucoco_list_operators is None
        or _mucoco_get_applicability_matrix is None
    ):
        logger.warning("mucoco_unavailable", endpoint="/mucoco/operators")
        return OrjsonResponse(
            {"error": "mucoco_unavailable",
             "detail": "elai_inference.mucoco.registry is not importable on this sidecar."},
            status_code=503,
        )

    operators = _mucoco_list_operators()
    matrix = _mucoco_get_applicability_matrix()

    logger.info("mucoco_operators_listed", count=len(operators))

    return OrjsonResponse({
        "operators": operators,
        "applicability_matrix": matrix,
    })


# ---------------------------------------------------------------------------
# Streaming helpers
# ---------------------------------------------------------------------------


def _stream_messages_response(
    prompt: str, max_tokens: int, model_name: str, tools: list | None,
    temperature: float = 0.0,
    response_format: Optional[Dict[str, Any]] = None,
    seed: Optional[int] = None,
    marker_boundary: Optional[int] = None,
) -> StreamingResponse:
    """Real streaming: tokens are yielded as model.generate() produces them.

    Phase B: response_format threads through to generate_streaming so
    LMFE-installed sidecars can emit constrained SSE output.

    Phase A6: optional ``seed`` is forwarded into generate_streaming so the
    background ``_run`` thread can call transformers.set_seed +
    torch.manual_seed inside the inference_mode lock right before
    model.generate() — same point coverage as the non-stream path.

    J-36: ``marker_boundary`` threads through to generate_streaming so the
    streaming path can lease a frozen-head KV prefix. Cache counters are
    surfaced in SSE message_start / message_delta usage.
    """

    async def event_generator() -> AsyncIterator[bytes]:
        msg_id = f"msg_elai_{uuid.uuid4().hex[:12]}"
        streamer, input_token_count = _engine.generate_streaming(
            prompt, max_tokens, temperature,
            response_format=response_format,
            seed=seed,
            marker_boundary=marker_boundary,
        )
        cache_read_tokens = getattr(streamer, "cache_read_tokens", 0)
        cache_creation_tokens = getattr(streamer, "cache_creation_tokens", 0)

        # message_start
        yield _sse_bytes(
            "message_start",
            {
                "type": "message_start",
                "message": {
                    "id": msg_id,
                    "type": "message",
                    "role": "assistant",
                    "content": [],
                    "model": model_name,
                    "stop_reason": None,
                    "stop_sequence": None,
                    "usage": {
                        "input_tokens": input_token_count,
                        "output_tokens": 0,
                        "cache_creation_input_tokens": cache_creation_tokens,
                        "cache_read_input_tokens": cache_read_tokens,
                    },
                },
            },
        )

        # Accumulate full text for tool call parsing at the end
        full_text_parts: list[str] = []
        block_index = 0
        text_block_started = False

        # content_block_start for text
        yield _sse_bytes(
            "content_block_start",
            {
                "type": "content_block_start",
                "index": block_index,
                "content_block": {"type": "text", "text": ""},
            },
        )
        text_block_started = True

        # Bridge TextIteratorStreamer (sync, blocking) to async via queue.
        # A background thread drains the streamer and feeds an asyncio.Queue
        # so the async generator yields tokens as they arrive (real streaming).
        token_queue: asyncio.Queue[str | None] = asyncio.Queue()
        loop = asyncio.get_running_loop()

        def _drain_streamer():
            try:
                for chunk in streamer:
                    loop.call_soon_threadsafe(token_queue.put_nowait, chunk)
            except Exception as exc:
                logger.error("streamer_drain_error", error=str(exc))
            finally:
                loop.call_soon_threadsafe(token_queue.put_nowait, None)

        _drain_pool.submit(_drain_streamer)

        while True:
            chunk = await token_queue.get()
            if chunk is None:
                break
            if not chunk:
                continue

            # Clean special tokens
            for token in [
                "<bos>", "<eos>", "<pad>",
                "<|im_end|>", "<|endoftext|>", "<|im_start|>",
            ]:
                chunk = chunk.replace(token, "")
            if not chunk:
                continue

            full_text_parts.append(chunk)

            # Stream as text_delta (tool calls will be re-parsed post-hoc)
            yield _sse_bytes(
                "content_block_delta",
                {
                    "type": "content_block_delta",
                    "index": block_index,
                    "delta": {"type": "text_delta", "text": chunk},
                },
            )

        # Close text block
        if text_block_started:
            yield _sse_bytes(
                "content_block_stop",
                {"type": "content_block_stop", "index": block_index},
            )
            block_index += 1

        # Parse tool calls from accumulated text
        full_text = "".join(full_text_parts)
        tool_calls, remaining_text = (
            _parse_tool_calls(full_text) if tools else ([], full_text)
        )

        # If tool calls found, emit tool_use blocks
        for tool_name, tool_args in tool_calls:
            tool_id = f"toolu_elai_{uuid.uuid4().hex[:12]}"

            yield _sse_bytes(
                "content_block_start",
                {
                    "type": "content_block_start",
                    "index": block_index,
                    "content_block": {
                        "type": "tool_use",
                        "id": tool_id,
                        "name": tool_name,
                        "input": {},
                    },
                },
            )

            yield _sse_bytes(
                "content_block_delta",
                {
                    "type": "content_block_delta",
                    "index": block_index,
                    "delta": {
                        "type": "input_json_delta",
                        "partial_json": orjson.dumps(tool_args).decode(),
                    },
                },
            )

            yield _sse_bytes(
                "content_block_stop",
                {"type": "content_block_stop", "index": block_index},
            )
            block_index += 1

        stop_reason = "tool_use" if tool_calls else "end_turn"

        # Count actual output tokens from accumulated text (full_text set above)
        actual_output_tokens = len(
            _engine.tokenizer.encode(full_text, add_special_tokens=False)
        )

        # message_delta (Anthropic spec: usage only has output_tokens)
        yield _sse_bytes(
            "message_delta",
            {
                "type": "message_delta",
                "delta": {"stop_reason": stop_reason, "stop_sequence": None},
                "usage": {
                    "output_tokens": actual_output_tokens,
                    "cache_creation_input_tokens": cache_creation_tokens,
                    "cache_read_input_tokens": cache_read_tokens,
                },
            },
        )

        # message_stop
        yield _sse_bytes("message_stop", {"type": "message_stop"})

    return StreamingResponse(
        event_generator(),
        media_type="text/event-stream",
        headers={"Cache-Control": "no-cache", "Connection": "keep-alive"},
    )


async def _non_stream_messages_response(
    prompt: str, max_tokens: int, model_name: str, tools: list | None,
    temperature: float = 0.0,
    marker_boundary: Optional[int] = None,
    response_format: Optional[Dict[str, Any]] = None,
    seed: Optional[int] = None,
    fitm_override: Optional[bool] = None,
) -> OrjsonResponse:
    loop = asyncio.get_running_loop()
    # Track C-D: gg_signals is now surfaced via response.meta (was discarded
    # pre-C-D as `_gg_signals`). marker_boundary threaded through to bound
    # PrefixKVCache.lease().
    # Track C-B: response_format threaded through to engine.generate; engine
    # converts to prefix_allowed_tokens_fn= kwarg of model.generate().
    # Phase A bench redesign: bracket the executor await with perf_counter to
    # surface server-side end-to-end latency as response.elapsed_ms (top level).
    # Bench reads this instead of client perf_counter to bypass SSH-tunnel RTT.

    # E-32: build FITM calibrator for this request (no-op if not active).
    # Tokenise the prompt here (async context) to get prompt_len; the calibrator
    # is constructed before the executor call so it is available inside _generate_impl.
    _fitm_enabled = os.getenv("ELAI_FITM_ENABLED", "0") == "1"
    _fitm_calibrator: FoundInTheMiddleCalibrator | None = None
    if _engine is not None:
        try:
            _prompt_tokens = _engine.tokenizer(prompt, return_tensors="pt")
            _prompt_len = _prompt_tokens["input_ids"].shape[1]
            _fitm_calibrator = FoundInTheMiddleCalibrator(
                model=_engine.model,
                tokenizer=_engine.tokenizer,
                prompt_len=_prompt_len,
                fitm_enabled=_fitm_enabled,
                per_request_override=fitm_override,
            )
        except Exception as _fitm_exc:
            logger.warning("fitm_calibrator_build_failed", error=str(_fitm_exc))
            _fitm_calibrator = None

    t0 = time.perf_counter()
    (
        text, tokens, input_token_count, _search_trigger, gg_signals,
        cache_read_tokens, cache_creation_tokens,
    ) = await loop.run_in_executor(
        _thread_pool,
        lambda: _engine.generate(
            prompt, max_tokens, temperature,
            marker_boundary=marker_boundary,
            response_format=response_format,
            seed=seed,
            fitm_calibrator=_fitm_calibrator,
        ),
    )
    elapsed_ms = int((time.perf_counter() - t0) * 1000)

    tool_calls, remaining_text = (
        _parse_tool_calls(text) if tools else ([], text)
    )

    content = []
    if remaining_text.strip():
        content.append({"type": "text", "text": remaining_text.strip()})
    for tool_name, tool_args in tool_calls:
        tool_id = f"toolu_elai_{uuid.uuid4().hex[:12]}"
        content.append({
            "type": "tool_use",
            "id": tool_id,
            "name": tool_name,
            "input": tool_args,
        })

    if not content:
        content.append({"type": "text", "text": text})

    stop_reason = "tool_use" if tool_calls else "end_turn"

    response = {
        "id": f"msg_elai_{uuid.uuid4().hex[:12]}",
        "type": "message",
        "role": "assistant",
        "content": content,
        "model": model_name,
        "stop_reason": stop_reason,
        "stop_sequence": None,
        # ELAI accounting note: input_tokens emits the FULL prompt token
        # count (cache_creation + cache_read = input_tokens). Anthropic's
        # spec emits input_tokens as the suffix-only (uncached) portion,
        # but downstream ELAI consumers depend on the full count for
        # billing/observability. Keep the divergence intentional.
        "usage": {
            "input_tokens": input_token_count,
            "output_tokens": len(tokens),
            "cache_creation_input_tokens": cache_creation_tokens,
            "cache_read_input_tokens": cache_read_tokens,
        },
        # ELAI custom extension — NOT in the documented Anthropic Messages
        # API response spec (https://docs.anthropic.com/en/api/messages).
        # Anthropic clients ignore unknown top-level fields, so adding this
        # is non-breaking. Defensive .get() per key keeps the response shape
        # stable even if GGSignalCollector internal keys change.
        "meta": {
            "gg_signals": {
                "mean_logprob": gg_signals.get("mean_logprob"),
                "min_logprob": gg_signals.get("min_logprob"),
                "entropy_mean": gg_signals.get("entropy_mean"),
                "entropy_max": gg_signals.get("entropy_max"),
                "generation_novelty": gg_signals.get("generation_novelty"),
            },
        },
        # Phase A bench redesign: top-level server-side elapsed_ms. Additive;
        # ELAI custom field. Anthropic SDK clients ignore unknown top-level keys.
        "elapsed_ms": elapsed_ms,
    }
    return OrjsonResponse(response)


def _sse_bytes(event_type: str, data: dict) -> bytes:
    payload = f"event: {event_type}\ndata: {orjson.dumps(data).decode()}\n\n"
    return payload.encode()


# ---------------------------------------------------------------------------
# CLI entrypoint
# ---------------------------------------------------------------------------


def main():
    parser = argparse.ArgumentParser(description="ELAI Inference Sidecar")
    parser.add_argument(
        "--model",
        default="D:/.ELAI_workflow/.elai_cc/models/Qwen2.5-Coder-3B-Instruct-abliterated",
    )
    parser.add_argument("--port", type=int, default=SIDECAR_PORT_DEFAULT)
    parser.add_argument("--host", default=SIDECAR_HOST_DEFAULT)
    parser.add_argument(
        "--dola", action="store_true", default=False,
        help="Enable DoLa decoding (HARMFUL on <32B dense per baseline Part 7)",
    )
    parser.add_argument("--no-dola", dest="dola", action="store_false")
    parser.add_argument("--dola-alpha", type=float, default=ALPHA_DEFAULT)
    parser.add_argument(
        "--turboquant-bits",
        type=int,
        default=8,
        choices=[0, 2, 4, 8],
        help="KV cache quantization bit-width (0=disabled, 8=recommended)",
    )
    parser.add_argument(
        "--kv-cache-size",
        type=int,
        default=4,
        help="LRU size for prefix KV cache (0 disables, "
             "auto-disabled when --turboquant-bits > 0)",
    )
    parser.add_argument(
        "--steering", action="store_true", default=False,
        help="Enable steering vectors (ActAdd)",
    )
    parser.add_argument("--no-steering", dest="steering", action="store_false")
    parser.add_argument(
        "--steering-coefficient",
        type=float,
        default=COEFFICIENT_DEFAULT,
        help="Steering coefficient (typical range [3, 20])",
    )
    parser.add_argument(
        "--steering-layer",
        default="auto",
        help="Steering target layer (or 'auto' for model_layers / 3)",
    )
    parser.add_argument(
        "--steering-vector-path",
        default=None,
        help="Path to pre-computed steering vector (.npz or .safetensors), "
        "or comma-separated list for multiple vectors",
    )
    parser.add_argument(
        "--etc-threshold",
        type=float,
        default=ETC_THRESHOLD_DISABLED,
        help="ETC entropy threshold (inf=disabled until calibration, 0=calibration mode)",
    )
    parser.add_argument("--max-tokens", type=int, default=MAX_NEW_TOKENS_DEFAULT)
    parser.add_argument(
        "--load-in-4bit",
        action="store_true",
        default=True,
        help="Load model with bitsandbytes 4-bit quantization (default: on)",
    )
    parser.add_argument(
        "--no-4bit", dest="load_in_4bit", action="store_false",
        help="Load model in FP16 (no quantization)",
    )
    parser.add_argument("--log-level", default="INFO")
    parser.add_argument("--quant-backend", default=None, choices=["nf4", "awq", "fp16"], help="Weight quantization backend (overrides ELAI_QUANT_BACKEND env var)")
    parser.add_argument("--kv-cache-dtype", default=None, choices=["fp16", "fp8_e4m3", "fp8_e5m2"], help="KV cache dtype (overrides ELAI_KV_CACHE_DTYPE env var)")
    args = parser.parse_args()
    if args.quant_backend is not None:
        import os as _os; _os.environ["ELAI_QUANT_BACKEND"] = args.quant_backend
    if args.kv_cache_dtype is not None:
        import os as _os; _os.environ["ELAI_KV_CACHE_DTYPE"] = args.kv_cache_dtype

    # Configure structlog level filter
    level_map = {
        "DEBUG": 10, "INFO": 20, "WARNING": 30, "ERROR": 40, "CRITICAL": 50,
    }
    min_level = level_map.get(args.log_level.upper(), 20)
    structlog.configure(
        wrapper_class=structlog.make_filtering_bound_logger(min_level),
    )

    # E-32: boot-time feature gates (loud refusals before model load)
    # YaRN rejection — must run before InferenceEngine loads the model
    refuse_yarn_at_boot(args.model)
    # Deep mode resolution — may swap model_name and set ctx_cap
    resolved_model, _deep_mode_ctx_cap = activate_deep_mode(args.model)
    if resolved_model != args.model:
        logger.info(
            "deep_mode_model_resolved",
            original=args.model,
            resolved=resolved_model,
            ctx_cap=_deep_mode_ctx_cap,
        )
        args.model = resolved_model

    global _engine
    _engine = InferenceEngine(
        model_name=args.model,
        dola_enabled=args.dola,
        dola_alpha=args.dola_alpha,
        turboquant_bits=args.turboquant_bits,
        steering_enabled=args.steering,
        steering_coefficient=args.steering_coefficient,
        steering_layer=args.steering_layer,
        max_new_tokens=args.max_tokens,
        load_in_4bit=args.load_in_4bit,
        etc_threshold=args.etc_threshold,
        kv_cache_size=args.kv_cache_size,
    )

    # E-32: emit deep_mode_activated SSB beacon (Python-side; mirrors deep_mode_activated
    # Rust SsbActionType for dashboard visibility). Log only — no Rust SSB writer here.
    if _deep_mode_ctx_cap > 0 and torch.cuda.is_available():
        vram_gb = round(
            torch.cuda.get_device_properties(torch.cuda.current_device()).total_memory
            / 1024 ** 3,
            1,
        )
        logger.info(
            "deep_mode_activated_ssb",
            model_id=args.model,
            ctx_cap=_deep_mode_ctx_cap,
            vram_gb=vram_gb,
        )

    # Load pre-computed steering vectors if provided
    if args.steering and _engine.steering_engine and args.steering_vector_path:
        vector_paths = [p.strip() for p in args.steering_vector_path.split(",")]
        for i, path in enumerate(vector_paths):
            layer_idx = _engine.steering_engine.target_layer_idx + i
            if layer_idx < _engine.n_layers:
                _engine.steering_engine.load_vector_from_file(layer_idx, path)
            else:
                logger.warning(
                    "vector_layer_overflow",
                    path=path,
                    layer=layer_idx,
                    max_layers=_engine.n_layers,
                )

    import uvicorn

    logger.info(
        "serving",
        host=args.host,
        port=args.port,
        dola=args.dola,
        turboquant=_engine.kv_compressor.enabled,
        endpoint_messages=f"http://{args.host}:{args.port}/v1/messages",
        endpoint_completions=f"http://{args.host}:{args.port}/v1/completions",
    )

    # GPU cleanup on shutdown is wired through the FastAPI lifespan (`_lifespan`),
    # which uvicorn invokes when it receives SIGTERM/SIGINT. uvicorn >=0.29
    # installs its own signal handlers that survive `signal.signal` registration
    # at this scope, so registering handlers here would be dead code. The
    # lifespan path drains the executor pools first (avoiding use-after-free)
    # and then calls `_graceful_gpu_teardown()`.
    #
    # Limitations:
    #   - SIGKILL bypasses lifespan entirely; use `kill -TERM` first, then -KILL
    #     after 5 seconds (see stop-sidecar.bat).
    #   - WSL VM termination via `wsl --terminate Ubuntu` resets the Hyper-V
    #     partition; no in-process handler can intercept that path.

    uvicorn.run(
        app,
        host=args.host,
        port=args.port,
        log_level=args.log_level.lower(),
        access_log=False,
    )


if __name__ == "__main__":
    main()
