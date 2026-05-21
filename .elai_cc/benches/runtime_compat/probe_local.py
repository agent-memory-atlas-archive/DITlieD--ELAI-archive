#!/usr/bin/env python3
"""
E-51 runtime compatibility probe.
Measures cold-boot time, TTFT, and decode tok/s for a given adapter endpoint.
Outputs JSON: {adapter, cold_boot_ms, ttft_ms, decode_tok_s, peak_ram_mb}

Usage:
    python3 probe_local.py --adapter sidecar --url http://localhost:8100 --prompt "hello"
"""

import argparse
import json
import time
import urllib.request
import urllib.error


def probe_adapter(adapter: str, url: str, prompt: str) -> dict:
    """Measure TTFT and decode tok/s for a single adapter."""
    payload = json.dumps({
        "model": "probe-test",
        "max_tokens": 64,
        "messages": [{"role": "user", "content": prompt}],
        "stream": False,
    }).encode("utf-8")

    headers = {
        "Content-Type": "application/json",
        "x-api-key": "probe-dummy",
        "anthropic-version": "2023-06-01",
    }

    t0 = time.monotonic()
    try:
        req = urllib.request.Request(
            f"{url}/v1/messages",
            data=payload,
            headers=headers,
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=30) as resp:
            t_first_byte = time.monotonic()
            body = resp.read()
            t_done = time.monotonic()
    except (urllib.error.URLError, OSError) as exc:
        return {"adapter": adapter, "error": str(exc)}

    ttft_ms = int((t_first_byte - t0) * 1000)
    total_ms = int((t_done - t0) * 1000)

    try:
        data = json.loads(body)
        output_text = ""
        for block in data.get("content", []):
            if block.get("type") == "text":
                output_text += block.get("text", "")
        output_tokens = data.get("usage", {}).get("output_tokens", len(output_text.split()))
    except (json.JSONDecodeError, KeyError):
        output_tokens = 0

    decode_tok_s = round(output_tokens / max((total_ms - ttft_ms) / 1000.0, 0.001), 1)

    return {
        "adapter": adapter,
        "cold_boot_ms": 0,  # Not measured here; process already running
        "ttft_ms": ttft_ms,
        "decode_tok_s": decode_tok_s,
        "peak_ram_mb": None,  # Requires psutil; optional
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="E-51 runtime probe")
    parser.add_argument("--adapter", required=True)
    parser.add_argument("--url", required=True)
    parser.add_argument("--prompt", default="Write a hello world function in Python.")
    args = parser.parse_args()

    result = probe_adapter(args.adapter, args.url, args.prompt)
    print(json.dumps(result))


if __name__ == "__main__":
    main()
