"""ETC threshold calibration script (A-9).

Runs N code-completion prompts against the sidecar with ETC in calibration
mode (threshold=0), collects d2H values at every token, then computes the
p95 of non-import d2H as the production threshold.

Usage:
    # Start sidecar with ETC calibration mode:
    python -m elai_inference.server --model <path> --etc-threshold 0

    # Then run calibration:
    python metrics/calibrate_etc_threshold.py --url http://localhost:8100 \
        --samples 100 --output metrics/etc_calibration.json

The calibrated threshold should be set in elai_cc.toml [inference.etc_threshold]
or passed via --etc-threshold when starting the sidecar.
"""

import argparse
import sys
from pathlib import Path

import numpy as np

try:
    import orjson as json

    def _dumps(obj):
        return json.dumps(obj, option=json.OPT_INDENT_2).decode()
except ImportError:
    import json

    def _dumps(obj):
        return json.dumps(obj, indent=2)

try:
    import httpx
except ImportError:
    sys.exit("httpx required: pip install httpx")

# Calibration prompts — short code-completion tasks that exercise import lines
CALIBRATION_PROMPTS = [
    "Write a Python function that reads a CSV file and returns a list of dictionaries.",
    "Implement a Rust function that hashes a string using a modern algorithm.",
    "Write a Go HTTP handler that accepts JSON and returns a processed response.",
    "Create a TypeScript function that fetches data from an API with error handling.",
    "Write a Python script that connects to PostgreSQL and runs a query.",
    "Implement a Rust CLI that reads TOML config and validates it.",
    "Write a Python FastAPI endpoint that accepts file uploads.",
    "Create a Go function that reads environment variables with defaults.",
    "Write a TypeScript React component that fetches and displays a list.",
    "Implement a Rust async function that makes HTTP requests with retries.",
    "Write a Python function using numpy to normalize a matrix.",
    "Create a Go gRPC server with a simple health check endpoint.",
    "Write a Python function that serializes data to JSON with orjson.",
    "Implement a Rust function that parses command-line arguments with clap.",
    "Write a TypeScript function that validates an email address with zod.",
    "Create a Python async function that processes items from a queue.",
    "Write a Rust function that reads a file line by line and filters matches.",
    "Implement a Go function that sorts a slice of structs by a field.",
    "Write a Python function that computes SHA-256 of a file in chunks.",
    "Create a TypeScript Express middleware for JWT authentication.",
]

SIDECAR_MAX_TOKENS = 256
SIDECAR_TEMPERATURE = 0.0


def run_calibration(url: str, n_samples: int) -> dict:
    """Send n_samples prompts and collect d2H data from ETC calibration logs."""
    completions_url = f"{url}/v1/completions"

    all_d2h_import: list[float] = []
    all_d2h_non_import: list[float] = []
    total_tokens = 0
    errors = 0

    for i in range(n_samples):
        prompt = CALIBRATION_PROMPTS[i % len(CALIBRATION_PROMPTS)]
        try:
            resp = httpx.post(
                completions_url,
                json={
                    "prompt": prompt,
                    "max_tokens": SIDECAR_MAX_TOKENS,
                    "temperature": SIDECAR_TEMPERATURE,
                },
                timeout=120.0,
            )
            resp.raise_for_status()
        except (httpx.HTTPError, httpx.TimeoutException) as exc:
            print(f"  [{i+1}/{n_samples}] ERROR: {exc}")
            errors += 1
            continue

        data = resp.json()
        # The sidecar returns etc_calibration_log in elai_metadata when threshold=0
        metadata = data.get("elai_metadata", {})
        d2h_log = metadata.get("etc_d2h_log", [])

        for entry in d2h_log:
            # Each entry: [token_pos, d2h_value, is_import_context]
            if len(entry) >= 3:
                d2h_val = float(entry[1])
                is_import = bool(entry[2])
                if is_import:
                    all_d2h_import.append(d2h_val)
                else:
                    all_d2h_non_import.append(d2h_val)
                total_tokens += 1

        completion_tokens = data.get("usage", {}).get("completion_tokens", 0)
        triggered = metadata.get("etc_triggered", False)
        print(
            f"  [{i+1}/{n_samples}] tokens={completion_tokens} "
            f"d2h_entries={len(d2h_log)} triggered={triggered}"
        )

    if not all_d2h_non_import:
        print("ERROR: no non-import d2H data collected. Is the sidecar running with --etc-threshold 0?")
        return {"error": "no_data", "errors": errors}

    # Compute statistics
    non_import_arr = np.array(all_d2h_non_import)
    import_arr = np.array(all_d2h_import) if all_d2h_import else np.array([0.0])

    p95_non_import = float(np.percentile(non_import_arr, 95))
    p99_non_import = float(np.percentile(non_import_arr, 99))
    mean_non_import = float(np.mean(non_import_arr))
    std_non_import = float(np.std(non_import_arr))

    mean_import = float(np.mean(import_arr))
    p50_import = float(np.percentile(import_arr, 50))

    # Threshold = p95 of non-import d2H (spec: section 2.1 step 4)
    threshold = p95_non_import

    result = {
        "threshold": threshold,
        "p95_non_import": p95_non_import,
        "p99_non_import": p99_non_import,
        "mean_non_import": mean_non_import,
        "std_non_import": std_non_import,
        "mean_import": mean_import,
        "p50_import": p50_import,
        "sample_count": n_samples,
        "total_tokens": total_tokens,
        "import_tokens": len(all_d2h_import),
        "non_import_tokens": len(all_d2h_non_import),
        "errors": errors,
    }

    print(f"\n--- ETC Calibration Results ---")
    print(f"Samples:           {n_samples} ({errors} errors)")
    print(f"Total tokens:      {total_tokens}")
    print(f"Import tokens:     {len(all_d2h_import)}")
    print(f"Non-import tokens: {len(all_d2h_non_import)}")
    print(f"Threshold (p95):   {threshold:.6f}")
    print(f"p99 non-import:    {p99_non_import:.6f}")
    print(f"Mean import d2H:   {mean_import:.6f}")
    print(f"Mean non-import:   {mean_non_import:.6f}")
    print(f"\nSet in elai_cc.toml: [inference] etc_threshold = {threshold:.6f}")

    return result


def main():
    parser = argparse.ArgumentParser(description="ETC threshold calibration")
    parser.add_argument("--url", default="http://localhost:8100", help="Sidecar URL")
    parser.add_argument("--samples", type=int, default=100, help="Number of calibration samples")
    parser.add_argument("--output", default="metrics/etc_calibration.json", help="Output JSON path")
    args = parser.parse_args()

    print(f"ETC calibration: {args.samples} samples against {args.url}")
    print(f"Sidecar must be running with --etc-threshold 0 (calibration mode)\n")

    result = run_calibration(args.url, args.samples)

    out_path = Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(_dumps(result), encoding="utf-8")
    print(f"\nResults written to {out_path}")


if __name__ == "__main__":
    main()
