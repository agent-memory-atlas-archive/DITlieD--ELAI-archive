"""FITM RAG Eval Harness (E-32 Task 8).

Measures whether Found-in-the-Middle (FITM) calibration gives a recall uplift
on ELAI's actual code-context retrieval workload. Paper baseline: +15pp Recall@3
on NaturalQuestions (arxiv:2406.16008 ACL 2024 Hsieh et al.). ELAI's workload
(code context, multi-file synthesis) may have a different positional distribution
-- this harness measures it empirically before FITM is turned on by default.

Gate: FITM becomes default-on ONLY if this harness measures >= +5pp Recall@3
over the baseline (no-FITM) run. See E-32-decisions.md DECISION 2.

Usage:
    python run_fitm_eval.py [--sidecar http://127.0.0.1:8100] [--fixtures DIRPATH]
                            [--recall-k 3] [--min-uplift-pp 5]
                            [--output results.json]

The sidecar must be running. Both FITM-off and FITM-on passes are run against
the same fixture set; the uplift is the difference in Recall@K.

Exit code:
    0 -- uplift >= min_uplift_pp (gate passes; FITM is safe to enable)
    1 -- uplift < min_uplift_pp or error
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.request
import urllib.error
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Fixtures
# ---------------------------------------------------------------------------

# Each fixture is a multi-chunk RAG context. The "relevant" chunk is placed at
# a known position (beginning / middle / end) so we can measure recall
# separately by position.
#
# Format: list of {"question": str, "chunks": [str], "relevant_chunk_idx": int,
#                  "expected_answer_fragment": str}
# relevant_chunk_idx: 0 = first, len-1 = last, middle = 1..(len-2).

DEFAULT_FIXTURES: list[dict] = [
    # -- Position: beginning --
    {
        "question": "What does the `ContextBudget::compute` function return?",
        "chunks": [
            "fn compute(model_max_context: u32, slot_estimates: impl IntoIterator<Item=u32>, "
            "output_reserve: u32) -> Result<ContextBudget, BudgetError> { ... "
            "returns an allocation plan respecting the per-slot weights. }",
            "fn detect_model_capability_profile(model_name: &str) -> ModelCapabilityProfile "
            "{ match model_name { \"qwen\" => ModelCapabilityProfile::micro_12gb(), _ => ... } }",
            "struct ModelCapabilityProfile { in_vram_ceiling: u32, effective_ceiling: u32, ... }",
            "pub fn emit_context_budget_computed(ssb: &Ssb, payload: ContextBudgetComputedPayload)"
            " -> impl Future { ssb.write_payload(...) }",
        ],
        "relevant_chunk_idx": 0,
        "expected_answer_fragment": "allocation plan",
    },
    # -- Position: middle --
    {
        "question": "How does `FoundInTheMiddleCalibrator` subtract the positional prior?",
        "chunks": [
            "class SidecarBootError(RuntimeError): pass",
            "def refuse_yarn_at_boot(model_name: str) -> None: "
            "if os.getenv('ELAI_USE_YARN') == '1': raise SidecarBootError(...)",
            "class FoundInTheMiddleCalibrator(LogitsProcessor): "
            "def _compute_prior(self, input_ids, dummy_text): "
            "with torch.inference_mode(): prior_logits = self.model(input_ids_with_dummy).logits[0, -1]; "
            "return prior_logits  # subtracted from actual logits in __call__",
            "def activate_deep_mode(default_model: str) -> tuple[str, int]: "
            "if os.getenv('ELAI_DEEP_MODE') != '1': return default_model, 32000",
        ],
        "relevant_chunk_idx": 2,
        "expected_answer_fragment": "prior_logits",
    },
    # -- Position: middle --
    {
        "question": "What is the HANDOFF_THRESHOLD_FRACTION in escalation_policy.rs?",
        "chunks": [
            "pub const FALLBACK_EFFECTIVE_MAX_CONTEXT: u32 = 25_000;",
            "pub const DEEP_MODE_CTX_CAP: u32 = 64_000;",
            "pub const HANDOFF_THRESHOLD_FRACTION: f64 = 0.85;",
            "pub const DEEP_MODE_MIN_FILES_REFERENCED: u32 = 3;",
            "pub const DEFAULT_MAX_TOKENS: u32 = 512;",
        ],
        "relevant_chunk_idx": 2,
        "expected_answer_fragment": "0.85",
    },
    # -- Position: end --
    {
        "question": "What is the DEEP_MODE_MIN_VRAM_GB constant value?",
        "chunks": [
            "FITM_PROMPT_LEN_THRESHOLD = 8192",
            "FITM_DUMMY_MAX_TOKENS = 100",
            "DEEP_MODE_CTX_CAP = 64000",
            "DEEP_MODE_MIN_VRAM_GB = 24.0",
        ],
        "relevant_chunk_idx": 3,
        "expected_answer_fragment": "24.0",
    },
    # -- Position: beginning --
    {
        "question": "What schema version does E-32 set in ssb.rs?",
        "chunks": [
            "// SCHEMA_VERSION 95: E-32 context window extension. "
            "Adds 4 new SSB action types: FitmCalibrationApplied, DeepModeActivated, "
            "DeepModeEscalationChosen, YarnRefused. Previous: 94.",
            "pub const SCHEMA_VERSION: u32 = 95;",
            "SsbActionType::FitmCalibrationApplied => 'fitm_calibration_applied',",
            "SsbActionType::YarnRefused => 'yarn_refused',",
        ],
        "relevant_chunk_idx": 0,
        "expected_answer_fragment": "95",
    },
    # -- Position: middle --
    {
        "question": "What happens if ELAI_USE_YARN=1 is set at sidecar boot?",
        "chunks": [
            "# FITM gate: only activate when prompt is long enough",
            "if prompt_len <= FITM_PROMPT_LEN_THRESHOLD: return logits",
            "# YaRN rejection: raises SidecarBootError at boot for Qwen family",
            "if os.getenv('ELAI_USE_YARN') == '1' and 'qwen' in model_name.lower(): "
            "raise SidecarBootError('YaRN rejected: 15-20% quality drop on short prompts')",
            "# deep mode: swap model to 1M variant at boot",
        ],
        "relevant_chunk_idx": 3,
        "expected_answer_fragment": "SidecarBootError",
    },
]


# ---------------------------------------------------------------------------
# Types
# ---------------------------------------------------------------------------

@dataclass
class EvalResult:
    """Recall@K result for one pass (FITM on or off)."""
    fitm_enabled: bool
    recall_at_k: float
    total_queries: int
    hits: int
    position_breakdown: dict = field(default_factory=dict)  # "beginning"/"middle"/"end" -> recall
    latency_ms_mean: float = 0.0


@dataclass
class EvalReport:
    baseline: EvalResult
    fitm: EvalResult

    @property
    def uplift_pp(self) -> float:
        return (self.fitm.recall_at_k - self.baseline.recall_at_k) * 100.0

    def passes_gate(self, min_uplift_pp: float) -> bool:
        return self.uplift_pp >= min_uplift_pp

    def as_dict(self) -> dict:
        return {
            "baseline_recall_at_k": self.baseline.recall_at_k,
            "fitm_recall_at_k": self.fitm.recall_at_k,
            "uplift_pp": round(self.uplift_pp, 2),
            "baseline_hits": self.baseline.hits,
            "fitm_hits": self.fitm.hits,
            "total_queries": self.baseline.total_queries,
            "baseline_latency_ms_mean": round(self.baseline.latency_ms_mean, 1),
            "fitm_latency_ms_mean": round(self.fitm.latency_ms_mean, 1),
            "baseline_position_breakdown": self.baseline.position_breakdown,
            "fitm_position_breakdown": self.fitm.position_breakdown,
        }


# ---------------------------------------------------------------------------
# Sidecar client
# ---------------------------------------------------------------------------

def _post_messages(
    sidecar_url: str,
    context_text: str,
    question: str,
    fitm_enabled: bool,
    max_tokens: int = 64,
    timeout_secs: int = 60,
) -> tuple[str, float]:
    """POST to /v1/messages, return (response_text, elapsed_ms)."""
    payload = {
        "model": "qwen",
        "max_tokens": max_tokens,
        "messages": [
            {
                "role": "user",
                "content": (
                    f"Context:\n{context_text}\n\n"
                    f"Question: {question}\n\n"
                    "Answer in one sentence using only information from the context above."
                ),
            }
        ],
        "fitm": fitm_enabled,
    }
    body = json.dumps(payload).encode()
    req = urllib.request.Request(
        f"{sidecar_url}/v1/messages",
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    t0 = time.monotonic()
    try:
        with urllib.request.urlopen(req, timeout=timeout_secs) as resp:
            data = json.loads(resp.read())
            elapsed_ms = (time.monotonic() - t0) * 1000.0
            text_blocks = [
                b.get("text", "")
                for b in data.get("content", [])
                if b.get("type") == "text"
            ]
            return "".join(text_blocks), elapsed_ms
    except urllib.error.URLError as exc:
        elapsed_ms = (time.monotonic() - t0) * 1000.0
        return f"[sidecar_error: {exc}]", elapsed_ms


def _sidecar_health(sidecar_url: str) -> bool:
    try:
        with urllib.request.urlopen(f"{sidecar_url}/health", timeout=5) as resp:
            return resp.status == 200
    except Exception:
        return False


# ---------------------------------------------------------------------------
# Eval runner
# ---------------------------------------------------------------------------

def _position_label(idx: int, total: int) -> str:
    if idx == 0:
        return "beginning"
    if idx == total - 1:
        return "end"
    return "middle"


def run_eval_pass(
    sidecar_url: str,
    fixtures: list[dict],
    fitm_enabled: bool,
    recall_k: int,
) -> EvalResult:
    hits = 0
    latencies: list[float] = []
    position_hits: dict[str, int] = {"beginning": 0, "middle": 0, "end": 0}
    position_total: dict[str, int] = {"beginning": 0, "middle": 0, "end": 0}

    for fix in fixtures:
        question: str = fix["question"]
        chunks: list[str] = fix["chunks"]
        relevant_idx: int = fix["relevant_chunk_idx"]
        expected: str = fix["expected_answer_fragment"]
        pos = _position_label(relevant_idx, len(chunks))

        # Build context: relevant chunk at the specified position.
        # For Recall@K we present all chunks; the model should surface
        # the relevant one regardless of position.
        context_text = "\n\n".join(
            f"[Chunk {i+1}]\n{c}" for i, c in enumerate(chunks)
        )

        response, elapsed_ms = _post_messages(
            sidecar_url, context_text, question, fitm_enabled
        )
        latencies.append(elapsed_ms)
        position_total[pos] = position_total.get(pos, 0) + 1

        hit = expected.lower() in response.lower()
        if hit:
            hits += 1
            position_hits[pos] = position_hits.get(pos, 0) + 1

    total = len(fixtures)
    recall = hits / total if total > 0 else 0.0
    mean_latency = sum(latencies) / len(latencies) if latencies else 0.0

    pos_recall = {
        pos: (position_hits.get(pos, 0) / max(position_total.get(pos, 1), 1))
        for pos in ("beginning", "middle", "end")
    }

    return EvalResult(
        fitm_enabled=fitm_enabled,
        recall_at_k=recall,
        total_queries=total,
        hits=hits,
        position_breakdown=pos_recall,
        latency_ms_mean=mean_latency,
    )


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(description="FITM RAG Eval Harness (E-32)")
    parser.add_argument(
        "--sidecar", default="http://127.0.0.1:8100", help="Sidecar base URL"
    )
    parser.add_argument(
        "--fixtures",
        default=None,
        help="Path to JSON fixtures file (default: use built-in fixtures)",
    )
    parser.add_argument(
        "--recall-k", type=int, default=3, help="K for Recall@K (default: 3)"
    )
    parser.add_argument(
        "--min-uplift-pp",
        type=float,
        default=5.0,
        help="Minimum FITM uplift in percentage points to pass gate (default: 5.0)",
    )
    parser.add_argument(
        "--output", default=None, help="Write JSON report to this path"
    )
    args = parser.parse_args()

    # Health check
    print(f"Checking sidecar at {args.sidecar} ...", flush=True)
    if not _sidecar_health(args.sidecar):
        print(
            f"ERROR: sidecar not reachable at {args.sidecar}. "
            "Boot the sidecar before running this harness.",
            file=sys.stderr,
        )
        return 1

    # Load fixtures
    if args.fixtures:
        with open(args.fixtures, encoding="utf-8") as f:
            fixtures = json.load(f)
    else:
        fixtures = DEFAULT_FIXTURES

    total = len(fixtures)
    print(f"Running eval: {total} fixtures, Recall@{args.recall_k}", flush=True)

    # Baseline pass (FITM off)
    print("Pass 1/2: FITM=off (baseline) ...", flush=True)
    baseline = run_eval_pass(args.sidecar, fixtures, fitm_enabled=False, recall_k=args.recall_k)
    print(
        f"  Recall@{args.recall_k}={baseline.recall_at_k:.3f} "
        f"({baseline.hits}/{baseline.total_queries}) "
        f"latency={baseline.latency_ms_mean:.0f}ms",
        flush=True,
    )

    # FITM pass (FITM on)
    print("Pass 2/2: FITM=on ...", flush=True)
    fitm_result = run_eval_pass(args.sidecar, fixtures, fitm_enabled=True, recall_k=args.recall_k)
    print(
        f"  Recall@{args.recall_k}={fitm_result.recall_at_k:.3f} "
        f"({fitm_result.hits}/{fitm_result.total_queries}) "
        f"latency={fitm_result.latency_ms_mean:.0f}ms",
        flush=True,
    )

    report = EvalReport(baseline=baseline, fitm=fitm_result)
    uplift = report.uplift_pp

    print(
        f"\nUplift: {uplift:+.2f}pp (gate threshold: +{args.min_uplift_pp}pp)",
        flush=True,
    )
    print(f"Position breakdown (FITM on): {fitm_result.position_breakdown}", flush=True)

    passed = report.passes_gate(args.min_uplift_pp)
    status = "PASS -- FITM uplift meets gate; safe to set ELAI_FITM_ENABLED=1" if passed else (
        f"FAIL -- uplift {uplift:+.2f}pp < {args.min_uplift_pp}pp gate; keep FITM default-off"
    )
    print(status, flush=True)

    if args.output:
        out_path = Path(args.output)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        with out_path.open("w", encoding="utf-8") as f:
            json.dump(report.as_dict(), f, indent=2)
        print(f"Report written to {out_path}", flush=True)

    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
