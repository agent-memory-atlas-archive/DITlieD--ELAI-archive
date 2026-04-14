"""ELAI Inference Sidecar -- HF Transformers with DoLa + TurboQuant + Steering.

Serves Anthropic-compatible /v1/messages and OpenAI-compatible /v1/completions
endpoints via FastAPI + uvicorn (async ASGI).

DoLa contrastive decoding, TurboQuant KV compression, and ActAdd steering
vectors are integrated via model forward hooks + a custom LogitsProcessor,
allowing model.generate() to drive the decode loop in optimized C++/CUDA.

Usage:
    python -m elai_inference.server --model google/gemma-4-E2B-it --port 8100
"""

import argparse
import asyncio
import re
import signal
import time
import uuid
from concurrent.futures import ThreadPoolExecutor
from contextlib import asynccontextmanager
from threading import Lock
from typing import AsyncIterator

import numpy as np
import orjson
import structlog
import torch
import torch.nn.functional as F
from fastapi import FastAPI, Request
from fastapi.responses import Response, StreamingResponse
from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    BitsAndBytesConfig,
    LogitsProcessor,
    TextIteratorStreamer,
)

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

# ETC (Entropy-Trend Constraint) constants
ETC_THRESHOLD_DISABLED = float("inf")  # BLOCK-7: disabled until calibration
ETC_RECENT_WINDOW = 20  # tokens to decode for import-context check
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
    ):
        # Auto-detect FA2 availability
        try:
            import flash_attn  # noqa: F401
            _attn_impl = "flash_attention_2"
            logger.info("flash_attention_2_available", version=flash_attn.__version__)
        except ImportError:
            _attn_impl = "sdpa"

        logger.info("loading_model", model=model_name, load_in_4bit=load_in_4bit, attn=_attn_impl)
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)

        if load_in_4bit:
            bnb_config = BitsAndBytesConfig(
                load_in_4bit=True,
                bnb_4bit_compute_dtype=torch.bfloat16,
                bnb_4bit_quant_type="nf4",
                bnb_4bit_use_double_quant=True,
            )
            self.model = AutoModelForCausalLM.from_pretrained(
                model_name,
                quantization_config=bnb_config,
                device_map=device_map,
                attn_implementation=_attn_impl,
            )
        else:
            self.model = AutoModelForCausalLM.from_pretrained(
                model_name,
                dtype=torch.bfloat16,
                device_map=device_map,
                attn_implementation=_attn_impl,
            )
        self.model.eval()
        self.lock = Lock()
        _log_vram("after_model_load")

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

    def generate(
        self,
        prompt: str,
        max_tokens: int | None = None,
        temperature: float = 0.0,
    ) -> tuple[str, list[int], int, dict | None, dict[str, float | None]]:
        """Generate text synchronously.

        Returns (text, token_ids, input_token_count, search_trigger, gg_signals).
        search_trigger is None unless ETC detected a high-entropy import context.
        gg_signals contains mean_logprob, min_logprob, entropy_mean, entropy_max.
        """
        max_tokens = max_tokens or self.max_new_tokens

        with self.lock:
            return self._generate_impl(prompt, max_tokens, temperature)

    def generate_streaming(
        self,
        prompt: str,
        max_tokens: int | None = None,
        temperature: float = 0.0,
    ) -> tuple[TextIteratorStreamer, int]:
        """Start streaming generation in a background thread.

        Returns (streamer, input_token_count). The caller iterates the
        streamer to get token chunks as they are produced.
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

        generation_kwargs = {
            "input_ids": inputs.input_ids,
            "attention_mask": inputs.attention_mask,
            "max_new_tokens": max_tokens,
            "do_sample": temperature > 0,
            "streamer": streamer,
            "output_hidden_states": self.dola_enabled,
            "return_dict_in_generate": True,
        }
        if temperature > 0:
            generation_kwargs["temperature"] = temperature

        logits_processors = []
        if self.dola_enabled and self._dola_processor is not None:
            self._dola_processor.register_hooks()
            logits_processors.append(self._dola_processor)
            generation_kwargs["logits_processor"] = logits_processors

        def _run():
            try:
                with self.lock, torch.inference_mode():
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
        )
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

    def _generate_impl(
        self, prompt: str, max_tokens: int, temperature: float
    ) -> tuple[str, list[int], int, dict | None, dict[str, float | None]]:
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
            gen_input_ids = inputs.input_ids
            gen_attention_mask = inputs.attention_mask
            # Use quantized cache even for short prompts if TQ enabled
            if self.kv_compressor.enabled:
                past_kv = self._make_cache()

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
        if logits_processors:
            generation_kwargs["logits_processor"] = logits_processors

        try:
            with torch.inference_mode():
                outputs = self.model.generate(**generation_kwargs)
        finally:
            if self._dola_processor is not None:
                self._dola_processor.remove_hooks()

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
        )
        _log_vram("after_generate")

        gg_signals = self._gg_collector.signals()
        return text.strip(), generated_ids, input_token_count, search_trigger, gg_signals


# ---------------------------------------------------------------------------
# Anthropic message format helpers
# ---------------------------------------------------------------------------

_TOOL_CALL_RE = re.compile(
    r"<\|tool_call>call:(\w+)\{(.*?)\}<tool_call\|>", re.DOTALL
)


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
# Separate pools: _thread_pool for GPU-bound model.generate() calls,
# _drain_pool for lightweight streamer-drain threads. Prevents deadlock
# when concurrent streaming requests exhaust a single shared pool.
_thread_pool: ThreadPoolExecutor = ThreadPoolExecutor(
    max_workers=THREAD_POOL_SIZE, thread_name_prefix="elai-gpu"
)
_drain_pool: ThreadPoolExecutor = ThreadPoolExecutor(
    max_workers=THREAD_POOL_SIZE * 2, thread_name_prefix="elai-drain"
)


@asynccontextmanager
async def _lifespan(app: FastAPI):
    yield
    # Shutdown: cleanup
    _thread_pool.shutdown(wait=False)
    _drain_pool.shutdown(wait=False)
    if _engine is not None and _engine._dola_processor is not None:
        _engine._dola_processor.remove_hooks()
    logger.info("server_shutdown")


app = FastAPI(title="ELAI Inference Sidecar", lifespan=_lifespan)


@app.get("/health")
async def health():
    return OrjsonResponse({"status": "healthy"})


@app.get("/v1/models")
async def list_models():
    return OrjsonResponse({
        "data": [{"id": "elai-composed", "object": "model"}]
    })


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


@app.post("/v1/messages")
async def handle_messages(request: Request):
    if _engine is None:
        return OrjsonResponse({"error": "engine not initialized"}, status_code=503)
    body = await request.body()
    try:
        data = orjson.loads(body)
    except orjson.JSONDecodeError:
        return OrjsonResponse({"error": "invalid JSON"}, status_code=400)

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

    prompt = _anthropic_to_gemma_prompt(_engine.tokenizer, messages, system, tools)

    tool_names = [t.get("name", "?") for t in tools] if tools else []
    logger.info(
        "turn",
        system_chars=len(system) if system else 0,
        messages=len(messages),
        tools=len(tools),
        tool_names=tool_names[:10],
        prompt_chars=len(prompt),
    )

    if stream:
        return _stream_messages_response(prompt, max_tokens, model_name, tools, temperature)
    return await _non_stream_messages_response(prompt, max_tokens, model_name, tools, temperature)


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

    t0 = time.time()
    loop = asyncio.get_running_loop()
    text, tokens, input_token_count, search_trigger, gg_signals = await loop.run_in_executor(
        _thread_pool,
        lambda: _engine.generate(prompt, max_tokens, temperature),
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
# Streaming helpers
# ---------------------------------------------------------------------------


def _stream_messages_response(
    prompt: str, max_tokens: int, model_name: str, tools: list | None,
    temperature: float = 0.0,
) -> StreamingResponse:
    """Real streaming: tokens are yielded as model.generate() produces them."""

    async def event_generator() -> AsyncIterator[bytes]:
        msg_id = f"msg_elai_{uuid.uuid4().hex[:12]}"
        streamer, input_token_count = _engine.generate_streaming(
            prompt, max_tokens, temperature
        )

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
                        "cache_creation_input_tokens": 0,
                        "cache_read_input_tokens": 0,
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
                "usage": {"output_tokens": actual_output_tokens},
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
) -> OrjsonResponse:
    loop = asyncio.get_running_loop()
    text, tokens, input_token_count, _search_trigger, _gg_signals = await loop.run_in_executor(
        _thread_pool,
        lambda: _engine.generate(prompt, max_tokens, temperature),
    )

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
        "usage": {
            "input_tokens": input_token_count,
            "output_tokens": len(tokens),
            "cache_creation_input_tokens": 0,
            "cache_read_input_tokens": 0,
        },
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
    parser.add_argument("--model", default="google/gemma-4-E2B-it")
    parser.add_argument("--port", type=int, default=SIDECAR_PORT_DEFAULT)
    parser.add_argument("--host", default=SIDECAR_HOST_DEFAULT)
    parser.add_argument(
        "--dola", action="store_true", default=True, help="Enable DoLa decoding"
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
    args = parser.parse_args()

    # Configure structlog level filter
    level_map = {
        "DEBUG": 10, "INFO": 20, "WARNING": 30, "ERROR": 40, "CRITICAL": 50,
    }
    min_level = level_map.get(args.log_level.upper(), 20)
    structlog.configure(
        wrapper_class=structlog.make_filtering_bound_logger(min_level),
    )

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

    uvicorn.run(
        app,
        host=args.host,
        port=args.port,
        log_level=args.log_level.lower(),
        access_log=False,
    )


if __name__ == "__main__":
    main()
