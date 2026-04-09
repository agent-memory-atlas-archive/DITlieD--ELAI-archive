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

def _log_vram(label=""):
    """Log GPU VRAM usage."""
    if torch.cuda.is_available():
        alloc = torch.cuda.memory_allocated() / 1024**3
        reserved = torch.cuda.memory_reserved() / 1024**3
        total = torch.cuda.get_device_properties(0).total_mem / 1024**3
        logger.info(
            f"VRAM [{label}]: allocated={alloc:.2f}GB, reserved={reserved:.2f}GB, "
            f"total={total:.2f}GB, free={total - reserved:.2f}GB"
        )


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
        _log_vram("after model load")

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
        gen_start = time.time()
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
        input_token_count = input_ids.shape[1]
        logger.info(
            f"[request] prompt_chars={len(prompt)}, formatted_chars={len(formatted)}, "
            f"input_tokens={input_token_count}, max_tokens={max_tokens}"
        )
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

            # Check EOS
            eos_ids = self.tokenizer.eos_token_id
            if isinstance(eos_ids, int):
                eos_ids = [eos_ids]
            elif eos_ids is None:
                eos_ids = []
            if next_token in eos_ids:
                break

            # Check for tool call stop — decode recent tokens to detect <tool_call|>
            if len(generated_tokens) >= 3:
                recent = self.tokenizer.decode(generated_tokens[-10:], skip_special_tokens=False)
                if '<tool_call|>' in recent:
                    break

        text = self.tokenizer.decode(generated_tokens, skip_special_tokens=False)
        # Clean up special tokens but preserve tool call markers for parsing
        # Remove <bos>, <eos>, turn markers, but keep tool_call markers
        for token in ['<bos>', '<eos>', '<pad>']:
            text = text.replace(token, '')

        gen_elapsed_ms = (time.time() - gen_start) * 1000
        out_tokens = len(generated_tokens)
        tps = out_tokens / (gen_elapsed_ms / 1000) if gen_elapsed_ms > 0 else 0
        logger.info(
            f"[response] input_tokens={input_token_count}, output_tokens={out_tokens}, "
            f"elapsed={gen_elapsed_ms:.0f}ms, tok/s={tps:.1f}"
        )
        _log_vram("after generate")
        return text.strip(), generated_tokens, input_token_count

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
    protocol_version = "HTTP/1.1"
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
        if self.path == "/v1/messages":
            return self._handle_messages()
        elif self.path == "/v1/completions":
            return self._handle_completions()
        elif self.path == "/shutdown":
            self._json_response(200, {"status": "shutting down"})
            import threading
            threading.Thread(target=self.server.shutdown, daemon=True).start()
        else:
            self._json_response(404, {"error": "not found"})

    def _handle_messages(self):
        """Anthropic Messages API compatible endpoint with streaming SSE."""
        content_length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(content_length)
        try:
            request = json.loads(body)
        except json.JSONDecodeError:
            self._json_response(400, {"error": "invalid JSON"})
            return

        # Extract request fields
        messages = request.get("messages", [])
        system = request.get("system", "")
        max_tokens = request.get("max_tokens", 256)
        stream = request.get("stream", False)
        tools = request.get("tools", [])

        # Convert Anthropic messages+tools to Gemma4 chat format
        prompt = self._anthropic_to_gemma_prompt(messages, system, tools)

        system_chars = len(system) if system else 0
        msg_count = len(messages)
        tool_count = len(tools) if tools else 0
        tool_names = [t.get("name", "?") for t in tools] if tools else []
        logger.info(
            f"[turn] system_chars={system_chars}, messages={msg_count}, "
            f"tools={tool_count} ({', '.join(tool_names[:10])}{'...' if tool_count > 10 else ''}), "
            f"prompt_chars={len(prompt)}"
        )
        logger.info(f"[prompt_preview] {prompt[:300]}...")

        if stream:
            self._stream_messages_response(prompt, max_tokens, request.get("model", "elai-local"), tools)
        else:
            self._non_stream_messages_response(prompt, max_tokens, request.get("model", "elai-local"), tools)

    def _anthropic_to_gemma_prompt(self, messages, system="", tools=None):
        """Convert Anthropic messages format to Gemma4 chat template input."""
        tokenizer = self.engine.tokenizer

        # Convert Anthropic tool definitions to OpenAI format (what Gemma4 chat template expects)
        # Filter to core tools only — small models can't handle 40+ tool definitions
        CORE_TOOLS = {
            "bash", "read_file", "write_file", "edit_file",
            "glob_search", "grep_search", "WebFetch", "WebSearch",
            "TodoWrite", "AskUserQuestion",
        }

        gemma_tools = None
        if tools:
            gemma_tools = []
            for tool in tools:
                name = tool.get("name", "")
                if name not in CORE_TOOLS:
                    continue
                # Truncate long descriptions to save context
                desc = tool.get("description", "")
                if len(desc) > 200:
                    desc = desc[:200] + "..."
                gemma_tool = {
                    "type": "function",
                    "function": {
                        "name": name,
                        "description": desc,
                        "parameters": tool.get("input_schema", {}),
                    },
                }
                gemma_tools.append(gemma_tool)
            if not gemma_tools:
                gemma_tools = None

        # Convert Anthropic messages to Gemma4 chat format
        gemma_messages = []

        # System message
        if system:
            # Gemma4 doesn't have a separate system role in chat template;
            # prepend to first user message or use as system instruction
            pass  # Handled below

        for msg in messages:
            role = msg.get("role", "user")
            content = msg.get("content", "")

            if role == "assistant":
                # Check for tool_use blocks
                if isinstance(content, list):
                    text_parts = []
                    for block in content:
                        if isinstance(block, dict):
                            if block.get("type") == "text":
                                text_parts.append(block.get("text", ""))
                            elif block.get("type") == "tool_use":
                                # Format as Gemma4 tool call inline
                                tool_name = block.get("name", "")
                                tool_input = block.get("input", {})
                                tool_call_str = self._format_gemma_tool_call(tool_name, tool_input)
                                text_parts.append(tool_call_str)
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
                                # Format as Gemma4 tool response
                                tool_id = block.get("tool_use_id", "")
                                result_content = block.get("content", "")
                                if isinstance(result_content, list):
                                    result_text = "\n".join(
                                        b.get("text", "") for b in result_content
                                        if isinstance(b, dict) and b.get("type") == "text"
                                    )
                                else:
                                    result_text = str(result_content)
                                text_parts.append(f"Tool result: {result_text}")
                    content = "\n".join(text_parts)
                gemma_messages.append({"role": "user", "content": content})

        # Prepend system to first user message if present
        if system and gemma_messages:
            for i, m in enumerate(gemma_messages):
                if m["role"] == "user":
                    gemma_messages[i]["content"] = system + "\n\n" + m["content"]
                    break

        # Use Gemma4's chat template
        try:
            formatted = tokenizer.apply_chat_template(
                gemma_messages,
                tools=gemma_tools if gemma_tools else None,
                tokenize=False,
                add_generation_prompt=True,
            )
            return formatted
        except Exception as e:
            logger.warning(f"Chat template failed: {e}, falling back to plain concat")
            # Fallback: plain concatenation
            parts = []
            if system:
                parts.append(system)
            for m in gemma_messages:
                parts.append(m.get("content", ""))
            return "\n".join(parts)

    def _format_gemma_tool_call(self, name, args):
        """Format a tool call in Gemma4's native format."""
        # Gemma4 format: <|tool_call>call:name{key:value,...}<tool_call|>
        parts = []
        for k, v in args.items():
            if isinstance(v, str):
                parts.append(f'{k}:<|"|>{v}<|"|>')
            else:
                parts.append(f'{k}:{json.dumps(v)}')
        args_str = ",".join(parts)
        return f"<|tool_call>call:{name}{{{args_str}}}<tool_call|>"

    def _parse_tool_calls(self, text):
        """Parse Gemma4 tool call output into Anthropic tool_use blocks.

        Gemma4 format: <|tool_call>call:tool_name{arg1:<|"|>val<|"|>,arg2:123}<tool_call|>
        Returns: list of (tool_name, args_dict, remaining_text)
        """
        import re

        tool_calls = []
        remaining = text

        # Pattern: <|tool_call>call:NAME{...}<tool_call|>
        pattern = r'<\|tool_call>call:(\w+)\{(.*?)\}<tool_call\|>'
        matches = list(re.finditer(pattern, text, re.DOTALL))

        if not matches:
            return [], text

        for match in matches:
            tool_name = match.group(1)
            args_raw = match.group(2)

            # Parse the Gemma4 arg format: key:<|"|>value<|"|>,key2:123
            args = {}
            # Split on commas that are outside <|"|>...<|"|> blocks
            current_key = ""
            current_val = ""
            in_string = False
            parsing_key = True
            i = 0

            while i < len(args_raw):
                ch = args_raw[i]

                # Check for <|"|> string delimiter
                if args_raw[i:i+5] == '<|"|>':
                    in_string = not in_string
                    i += 5
                    continue

                if in_string:
                    current_val += ch
                    i += 1
                    continue

                if ch == ':' and parsing_key:
                    parsing_key = False
                    i += 1
                    continue

                if ch == ',' and not in_string:
                    # Save current key-value
                    if current_key:
                        args[current_key.strip()] = self._parse_arg_value(current_val.strip())
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

            # Save last key-value
            if current_key:
                args[current_key.strip()] = self._parse_arg_value(current_val.strip())

            tool_calls.append((tool_name, args))

        # Remove tool call tokens from text to get remaining text
        remaining = re.sub(pattern, '', text, flags=re.DOTALL).strip()

        return tool_calls, remaining

    def _parse_arg_value(self, val):
        """Parse a tool argument value — try JSON types, fall back to string."""
        if not val:
            return ""
        # Try to parse as JSON (number, bool, null, object, array)
        try:
            return json.loads(val)
        except (json.JSONDecodeError, ValueError):
            return val

    def _stream_messages_response(self, prompt, max_tokens, model_name, tools=None):
        """Stream response in Anthropic SSE format with tool calling support."""
        msg_id = f"msg_elai_{int(time.time())}"

        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Cache-Control", "no-cache")
        self.send_header("Connection", "close")
        self.send_header("Transfer-Encoding", "chunked")
        self.end_headers()

        # Generate full response (get real token counts from tokenizer)
        text, tokens, input_token_count = self.engine.generate(prompt, max_tokens)

        # 1. message_start
        message_start = {
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
        }
        self._send_sse("message_start", message_start)

        # Check for tool calls in the output
        tool_calls, remaining_text = self._parse_tool_calls(text) if tools else ([], text)

        block_index = 0

        # Stream text content (if any)
        if remaining_text.strip():
            # content_block_start for text
            self._send_sse("content_block_start", {
                "type": "content_block_start",
                "index": block_index,
                "content_block": {"type": "text", "text": ""},
            })

            # Stream word by word
            words = remaining_text.strip().split(' ')
            for i, word in enumerate(words):
                chunk = word if i == 0 else ' ' + word
                self._send_sse("content_block_delta", {
                    "type": "content_block_delta",
                    "index": block_index,
                    "delta": {"type": "text_delta", "text": chunk},
                })

            self._send_sse("content_block_stop", {
                "type": "content_block_stop", "index": block_index,
            })
            block_index += 1

        # Stream tool_use blocks (if any)
        for tool_name, tool_args in tool_calls:
            tool_id = f"toolu_elai_{int(time.time())}_{block_index}"

            # content_block_start for tool_use
            self._send_sse("content_block_start", {
                "type": "content_block_start",
                "index": block_index,
                "content_block": {
                    "type": "tool_use",
                    "id": tool_id,
                    "name": tool_name,
                    "input": {},
                },
            })

            # Send tool input as input_json_delta
            input_json = json.dumps(tool_args)
            self._send_sse("content_block_delta", {
                "type": "content_block_delta",
                "index": block_index,
                "delta": {
                    "type": "input_json_delta",
                    "partial_json": input_json,
                },
            })

            self._send_sse("content_block_stop", {
                "type": "content_block_stop", "index": block_index,
            })
            block_index += 1

        # Determine stop reason
        stop_reason = "tool_use" if tool_calls else "end_turn"

        # message_delta
        self._send_sse("message_delta", {
            "type": "message_delta",
            "delta": {"stop_reason": stop_reason, "stop_sequence": None},
            "usage": {
                "input_tokens": input_token_count,
                "output_tokens": len(tokens),
                "cache_creation_input_tokens": 0,
                "cache_read_input_tokens": 0,
            },
        })

        # message_stop
        self._send_sse("message_stop", {"type": "message_stop"})

        # Terminate chunked transfer
        self.wfile.write(b"0\r\n\r\n")
        self.wfile.flush()

    def _non_stream_messages_response(self, prompt, max_tokens, model_name, tools=None):
        """Non-streaming Anthropic Messages response with tool calling support."""
        text, tokens, input_token_count = self.engine.generate(prompt, max_tokens)

        # Check for tool calls
        tool_calls, remaining_text = self._parse_tool_calls(text) if tools else ([], text)

        content = []
        if remaining_text.strip():
            content.append({"type": "text", "text": remaining_text.strip()})
        for tool_name, tool_args in tool_calls:
            tool_id = f"toolu_elai_{int(time.time())}_{len(content)}"
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
            "id": f"msg_elai_{int(time.time())}",
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
        self._json_response(200, response)

    def _send_sse(self, event_type, data):
        """Send a single SSE event using chunked transfer encoding."""
        payload = f"event: {event_type}\ndata: {json.dumps(data)}\n\n"
        chunk = f"{len(payload.encode()):x}\r\n{payload}\r\n"
        self.wfile.write(chunk.encode())
        self.wfile.flush()

    def _handle_completions(self):

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
        text, tokens, input_token_count = self.engine.generate(prompt, max_tokens, temperature)
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
                "prompt_tokens": input_token_count,
                "completion_tokens": len(tokens),
                "total_tokens": input_token_count + len(tokens),
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
        body = json.dumps(data).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(body)

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
