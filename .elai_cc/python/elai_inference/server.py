"""ELAI Inference Sidecar — HF Transformers with DoLa + TurboQuant.

Serves an OpenAI-compatible /v1/completions endpoint that internally
applies DoLa layer-contrast decoding and TurboQuant KV cache compression.

The Rust orchestrator talks to this server the same way it talks to vLLM.
All layer-level optimizations happen in-process on the Python side.

Usage:
    python -m elai_inference.server --model google/gemma-4-E2B-it --port 8100
"""

import argparse
import json
import logging
import sys
import time
from http.server import HTTPServer, BaseHTTPRequestHandler
from threading import Lock

import numpy as np
import torch
import torch.nn.functional as F

logger = logging.getLogger("elai_inference")

# ---------------------------------------------------------------------------
# DoLa constants
# ---------------------------------------------------------------------------

ALPHA_DEFAULT = 0.1
ALPHA_AMPLIFIED = 0.05
JSD_MAX_NATS = 0.6931471805599453
EPSILON = 1e-12

# ---------------------------------------------------------------------------
# DoLa implementation
# ---------------------------------------------------------------------------

def compute_jsd(p, q):
    m = 0.5 * (p + q)
    m_safe = m.clamp(min=EPSILON)
    p_safe = p.clamp(min=EPSILON)
    q_safe = q.clamp(min=EPSILON)
    kl_pm = (p * (p_safe / m_safe).log()).sum()
    kl_qm = (q * (q_safe / m_safe).log()).sum()
    return (0.5 * kl_pm + 0.5 * kl_qm).clamp(0.0, JSD_MAX_NATS).item()


def get_candidate_layers(n_layers):
    if n_layers <= 40:
        mid = n_layers // 2
        buckets = [range(0, mid), range(mid, n_layers)]
    else:
        buckets = []
        start = 0
        while start < n_layers:
            buckets.append(range(start, min(start + 20, n_layers)))
            start += 20
    return [l for bucket in buckets for l in bucket if l % 2 == 0]


def dola_decode_token(final_logits, final_probs, candidate_raw, candidate_probs, alpha):
    """Select next token using DoLa contrastive decoding."""
    # Select premature layer via JSD maximization
    best_layer = None
    best_jsd = -1.0
    for layer_idx, q_j in candidate_probs.items():
        jsd = compute_jsd(final_probs, q_j)
        if jsd > best_jsd:
            best_jsd = jsd
            best_layer = layer_idx

    if best_layer is None:
        return final_logits.argmax().item(), None

    # Contrastive logits with V_head constraint
    max_prob = final_probs.max().item()
    threshold = alpha * max_prob
    v_head_mask = final_probs >= threshold

    contrastive = torch.full_like(final_logits, float('-inf'))
    contrastive[v_head_mask] = (
        final_logits[v_head_mask] - candidate_raw[best_layer][v_head_mask]
    )
    return contrastive.argmax().item(), best_layer


# ---------------------------------------------------------------------------
# TurboQuant KV compression
# ---------------------------------------------------------------------------

class KVCompressor:
    """Compress KV cache using TurboQuant. Falls back to no-op if unavailable."""

    def __init__(self, head_dim, bit_width=4):
        self.head_dim = head_dim
        self.bit_width = bit_width
        self.tq = None
        self.enabled = False

        try:
            from turboquant import TurboQuantMSE
            self.tq = TurboQuantMSE(d=head_dim, bit_width=bit_width)
            self.enabled = True
            logger.info(f"TurboQuant enabled: {bit_width}-bit, d={head_dim}")
        except ImportError:
            logger.warning("TurboQuant not available, KV cache compression disabled")

    def compress_cache(self, past_key_values):
        """Compress KV cache in-place (quantize then dequantize)."""
        if not self.enabled or past_key_values is None:
            return

        if not hasattr(past_key_values, 'layers'):
            return

        for layer in past_key_values.layers:
            if not hasattr(layer, 'keys') or not hasattr(layer, 'values'):
                continue

            keys = layer.keys
            values = layer.values
            batch, n_heads, seq_len, head_dim = keys.shape

            if head_dim != self.head_dim:
                continue

            device = keys.device
            dtype = keys.dtype

            k_np = keys.float().cpu().numpy()
            v_np = values.float().cpu().numpy()

            for b in range(batch):
                for h in range(n_heads):
                    for s in range(seq_len):
                        ki, kn = self.tq.quantize(k_np[b, h, s, :])
                        k_np[b, h, s, :] = self.tq.dequantize(ki, kn)

                        vi, vn = self.tq.quantize(v_np[b, h, s, :])
                        v_np[b, h, s, :] = self.tq.dequantize(vi, vn)

            layer.keys = torch.from_numpy(k_np).to(dtype=dtype, device=device)
            layer.values = torch.from_numpy(v_np).to(dtype=dtype, device=device)


# ---------------------------------------------------------------------------
# Inference Engine
# ---------------------------------------------------------------------------

class InferenceEngine:
    """Composed inference: HF Transformers + DoLa + TurboQuant."""

    def __init__(self, model_name, dola_enabled=True, dola_alpha=ALPHA_DEFAULT,
                 turboquant_bits=4, max_new_tokens=256, device_map="auto"):
        from transformers import AutoModelForCausalLM, AutoTokenizer

        logger.info(f"Loading model: {model_name}")
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
        self.model = AutoModelForCausalLM.from_pretrained(
            model_name, dtype=torch.float16, device_map=device_map,
        )
        self.model.eval()
        self.lm_head = self.model.lm_head
        self.lock = Lock()

        # Model config
        if hasattr(self.model.config, 'text_config'):
            tc = self.model.config.text_config
            self.n_layers = tc.num_hidden_layers
            self.head_dim = getattr(tc, 'head_dim', 256)
        else:
            self.n_layers = self.model.config.num_hidden_layers
            self.head_dim = getattr(self.model.config, 'head_dim', 256)

        self.candidate_layers = get_candidate_layers(self.n_layers)
        self.dola_enabled = dola_enabled
        self.dola_alpha = dola_alpha
        self.max_new_tokens = max_new_tokens

        # TurboQuant
        self.kv_compressor = KVCompressor(self.head_dim, turboquant_bits)

        logger.info(
            f"Engine ready: layers={self.n_layers}, head_dim={self.head_dim}, "
            f"dola={'ON' if dola_enabled else 'OFF'} (alpha={dola_alpha}), "
            f"turboquant={'ON' if self.kv_compressor.enabled else 'OFF'} "
            f"({turboquant_bits}-bit)"
        )

    def generate(self, prompt, max_tokens=None, temperature=0.0, stream=False):
        """Generate text with DoLa + TurboQuant."""
        max_tokens = max_tokens or self.max_new_tokens

        with self.lock:
            return self._generate_impl(prompt, max_tokens, temperature)

    def _generate_impl(self, prompt, max_tokens, temperature):
        # Format with chat template if available
        if hasattr(self.tokenizer, 'apply_chat_template'):
            try:
                messages = [{"role": "user", "content": prompt}]
                formatted = self.tokenizer.apply_chat_template(
                    messages, tokenize=False, add_generation_prompt=True
                )
            except Exception:
                formatted = prompt
        else:
            formatted = prompt

        inputs = self.tokenizer(formatted, return_tensors="pt").to(self.model.device)
        input_ids = inputs.input_ids.clone()
        generated_tokens = []
        past = None

        for step in range(max_tokens):
            with torch.no_grad():
                if self.dola_enabled and past is None:
                    # First token: need hidden states for DoLa
                    outputs = self.model(
                        input_ids=input_ids,
                        output_hidden_states=True,
                        use_cache=True,
                        return_dict=True,
                    )
                    next_token = self._dola_select(outputs)
                    past = outputs.past_key_values
                elif self.dola_enabled:
                    # Subsequent tokens with KV cache
                    next_input = torch.tensor(
                        [[generated_tokens[-1]]], device=self.model.device
                    )
                    outputs = self.model(
                        input_ids=next_input,
                        past_key_values=past,
                        output_hidden_states=True,
                        use_cache=True,
                        return_dict=True,
                    )
                    next_token = self._dola_select(outputs)
                    past = outputs.past_key_values
                else:
                    # No DoLa: standard greedy
                    if past is None:
                        outputs = self.model(
                            input_ids=input_ids,
                            use_cache=True,
                            return_dict=True,
                        )
                    else:
                        next_input = torch.tensor(
                            [[generated_tokens[-1]]], device=self.model.device
                        )
                        outputs = self.model(
                            input_ids=next_input,
                            past_key_values=past,
                            use_cache=True,
                            return_dict=True,
                        )
                    next_token = outputs.logits[:, -1, :].argmax(dim=-1).item()
                    past = outputs.past_key_values

            # TurboQuant: compress KV cache after each step
            if self.kv_compressor.enabled:
                self.kv_compressor.compress_cache(past)

            generated_tokens.append(next_token)

            if next_token == self.tokenizer.eos_token_id:
                break
            if isinstance(self.tokenizer.eos_token_id, list):
                if next_token in self.tokenizer.eos_token_id:
                    break

        text = self.tokenizer.decode(generated_tokens, skip_special_tokens=True)
        return text, generated_tokens

    def _dola_select(self, outputs):
        """Apply DoLa contrastive decoding to select next token."""
        hidden_states = outputs.hidden_states
        final_hidden = hidden_states[-1][:, -1, :].to(self.lm_head.weight.device)
        final_logits = self.lm_head(final_hidden).squeeze(0).float()
        final_probs = F.softmax(final_logits, dim=-1)

        candidate_probs = {}
        candidate_raw = {}
        n_hidden = len(hidden_states)

        for layer_idx in self.candidate_layers:
            hs_idx = layer_idx + 1
            if hs_idx < n_hidden:
                h = hidden_states[hs_idx][:, -1, :].to(self.lm_head.weight.device)
                raw = self.lm_head(h).squeeze(0).float()
                candidate_probs[layer_idx] = F.softmax(raw, dim=-1)
                candidate_raw[layer_idx] = raw

        if not candidate_probs:
            return final_logits.argmax().item()

        token, _ = dola_decode_token(
            final_logits, final_probs, candidate_raw, candidate_probs,
            self.dola_alpha
        )
        return token


# ---------------------------------------------------------------------------
# HTTP Server (OpenAI-compatible /v1/completions)
# ---------------------------------------------------------------------------

class InferenceHandler(BaseHTTPRequestHandler):
    engine: InferenceEngine = None  # Set by server setup

    def do_GET(self):
        if self.path == "/health":
            self._json_response(200, {"status": "healthy"})
        elif self.path == "/v1/models":
            self._json_response(200, {
                "data": [{"id": "elai-composed", "object": "model"}]
            })
        else:
            self._json_response(404, {"error": "not found"})

    def do_POST(self):
        if self.path != "/v1/completions":
            self._json_response(404, {"error": "not found"})
            return

        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length)
        try:
            request = json.loads(body)
        except json.JSONDecodeError:
            self._json_response(400, {"error": "invalid JSON"})
            return

        prompt = request.get("prompt", "")
        max_tokens = request.get("max_tokens", 256)
        temperature = request.get("temperature", 0.0)

        t0 = time.time()
        text, tokens = self.engine.generate(prompt, max_tokens, temperature)
        elapsed = time.time() - t0

        # OpenAI-compatible response (non-streaming)
        response = {
            "id": f"cmpl-elai-{int(time.time())}",
            "object": "text_completion",
            "created": int(time.time()),
            "model": "elai-composed",
            "choices": [{
                "text": text,
                "index": 0,
                "finish_reason": "stop" if (tokens and tokens[-1] in
                    ([self.engine.tokenizer.eos_token_id] if isinstance(
                        self.engine.tokenizer.eos_token_id, int)
                     else self.engine.tokenizer.eos_token_id or []))
                    else "length",
            }],
            "usage": {
                "prompt_tokens": 0,  # Not tracked
                "completion_tokens": len(tokens),
                "total_tokens": len(tokens),
            },
            "elai_metadata": {
                "dola_enabled": self.engine.dola_enabled,
                "turboquant_enabled": self.engine.kv_compressor.enabled,
                "turboquant_bits": self.engine.kv_compressor.bit_width,
                "elapsed_ms": int(elapsed * 1000),
            },
        }
        self._json_response(200, response)

    def _json_response(self, status, data):
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(data).encode())

    def log_message(self, format, *args):
        logger.debug(format, *args)


def main():
    parser = argparse.ArgumentParser(description="ELAI Inference Sidecar")
    parser.add_argument("--model", default="google/gemma-4-E2B-it")
    parser.add_argument("--port", type=int, default=8100)
    parser.add_argument("--host", default="0.0.0.0")
    parser.add_argument("--dola", action="store_true", default=True,
                        help="Enable DoLa decoding")
    parser.add_argument("--no-dola", dest="dola", action="store_false")
    parser.add_argument("--dola-alpha", type=float, default=ALPHA_DEFAULT)
    parser.add_argument("--turboquant-bits", type=int, default=4,
                        choices=[0, 2, 3, 4],
                        help="TurboQuant bit-width (0=disabled)")
    parser.add_argument("--max-tokens", type=int, default=256)
    parser.add_argument("--log-level", default="INFO")
    args = parser.parse_args()

    logging.basicConfig(
        level=getattr(logging, args.log_level.upper()),
        format="%(asctime)s [%(name)s] %(levelname)s: %(message)s",
    )

    engine = InferenceEngine(
        model_name=args.model,
        dola_enabled=args.dola,
        dola_alpha=args.dola_alpha,
        turboquant_bits=args.turboquant_bits if args.turboquant_bits > 0 else 4,
        max_new_tokens=args.max_tokens,
    )
    if args.turboquant_bits == 0:
        engine.kv_compressor.enabled = False

    InferenceHandler.engine = engine

    server = HTTPServer((args.host, args.port), InferenceHandler)
    logger.info(f"Serving on {args.host}:{args.port}")
    logger.info(f"  DoLa: {'ON' if args.dola else 'OFF'}")
    logger.info(f"  TurboQuant: {'ON' if engine.kv_compressor.enabled else 'OFF'}")
    logger.info(f"  Endpoint: http://{args.host}:{args.port}/v1/completions")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        logger.info("Shutting down")
        server.shutdown()


if __name__ == "__main__":
    main()
