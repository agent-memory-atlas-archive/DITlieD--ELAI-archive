#!/usr/bin/env python3
"""J-26 golden-pack evaluator (deterministic, no LLM).

Usage: evaluate.py --gen-dir <dir>
Writes <dir>/results.json with a known scalar {"fitness": 0.87} and exits 0.
The contract test asserts run_evaluator reads this scalar and that
results_to_execution_fitness.fitness == 0.87.
"""

from __future__ import annotations

import argparse
import json
import os
import sys


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--gen-dir", required=True)
    args = parser.parse_args()

    gen_dir = args.gen_dir
    if not gen_dir:
        print("evaluate.py: --gen-dir is required", file=sys.stderr)
        return 2

    os.makedirs(gen_dir, exist_ok=True)
    payload = {
        "fitness": 0.87,
        "pass_rate": 0.9,
        "tests_total": 3,
        "tests_passed": 3,
    }
    with open(os.path.join(gen_dir, "results.json"), "w") as f:
        json.dump(payload, f)
    return 0


if __name__ == "__main__":
    sys.exit(main())
