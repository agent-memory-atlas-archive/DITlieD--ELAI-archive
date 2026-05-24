"""C-37 Structure Snowballing harness driver.

Replicates arXiv:2604.06066 (Zhou 2026) end-to-end against the local ELAI
sidecar (Anthropic-protocol HTTP endpoint). Pool A and Pool B are loaded
from HotpotQA distractor via dataset.py; Actor / Evaluator / Reflector
roles run with the paper-grounded uniform decoding params (temperature=0.1,
max_new_tokens=1024); the Reflector additionally requests constrained
output via the sidecar's response_format=json_schema hint.

USAGE — DEFAULT (sidecar transport, schema_hint mode):

    python -m elai_inference.c37_snowballing.run \\
        --model Qwen/Qwen3-8B \\
        --pool A \\
        --n-samples 55 \\
        --max-trials 5 \\
        --temperature 0.1 \\
        --max-new-tokens 1024 \\
        --reflector-mode schema_hint \\
        --output runs/snowballing-poolA.jsonl

USAGE — FULL PAPER REPLICATION (outlines mode, RUNPOD-PENDING):

    The paper's exact Outlines + pydantic FSM path requires the model to be
    loaded in-process with a logit-processor hook (HF transformers
    generate() path), which is NOT how the sidecar exposes generation
    (streaming HTTP). To match the paper precisely, either:

      (a) load Qwen3-8B at 4-bit in a one-shot Python session (NF4 NF4 via
          bitsandbytes; ~5 GB VRAM) and pipe Outlines through its logit
          processor; OR
      (b) load Qwen3-8B at bf16 on a 24 GB pod (~16 GB; STOP the existing
          sidecar first to avoid OOM); OR
      (c) accept the schema_hint deviation and document it.

    This module ships path (c) by default; the JSONL records carry
    `reflector_mode` so post-hoc analysis can tell the two apart.

VRAM NOTE: Qwen3-8B at bf16 needs ~16 GB; the remote 3090 has 24 GB. The
existing sidecar (Qwen2.5-Coder-7B NF4, ~1.92 GB) MUST be stopped before
loading Qwen3-8B at bf16, OR Qwen3-8B MUST be loaded at 4-bit.
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from pathlib import Path
from typing import Any

from elai_inference.c37_snowballing.dataset import (
    HotpotQAItem,
    default_dataset_path,
    load_pool_a,
    load_pool_b,
)
from elai_inference.c37_snowballing.metrics import (
    CONSTRAINED_ACCURACY_CEILING,
    REPLICATION_THRESHOLD,
    SEVERITY_THRESHOLD,
    compute_accuracy,
    compute_error_distribution,
    compute_snowballing_rate,
)
from elai_inference.c37_snowballing.roles import (
    DEFAULT_SIDECAR_URL,
    PAPER_MAX_NEW_TOKENS,
    PAPER_MODEL_DEFAULT,
    PAPER_TEMPERATURE,
    Actor,
    Evaluator,
    Reflector,
    paper_role_config,
)
from elai_inference.c37_snowballing.types import (
    QuestionResult,
    ReflectorOutput,
    TrialRecord,
)


# Paper Section 4.3 — max trials per question.
PAPER_MAX_TRIALS: int = 5


def _parse_final_answer(actor_text: str) -> str:
    """Extract the "Answer:" line from the Actor's output.

    Falls back to the trimmed full text when no `Answer:` line is present
    (this is itself a paper-grounded FORMATTING_MISMATCH signal — the
    Reflector should diagnose it on the next trial).
    """
    for line in reversed(actor_text.strip().splitlines()):
        stripped = line.strip()
        if stripped.lower().startswith("answer:"):
            return stripped.split(":", 1)[1].strip()
    return actor_text.strip()


def _run_one_question(
    item: HotpotQAItem,
    actor: Actor,
    evaluator: Evaluator,
    reflector: Reflector,
    max_trials: int,
    reflector_mode: str,
) -> QuestionResult:
    """Execute the paper-grounded multi-trial loop for one question.

    Returns a QuestionResult with full per-trial trajectory + status. No
    cross-question state: episodic memory is per-question (paper Section
    3.3 explicit).
    """
    contexts = item.gold_paragraphs + item.distractors
    episodic_memory: list[ReflectorOutput] = []
    trajectory: list[TrialRecord] = []
    final_answer: str = ""
    status: str = "failed"
    token_count = 0  # we do not have provider-side token telemetry from /v1/messages

    for trial_idx in range(1, max_trials + 1):
        actor_text = actor.generate(item.question, contexts, episodic_memory)
        candidate = _parse_final_answer(actor_text)
        final_answer = candidate
        verdict = evaluator.evaluate(item.question, item.gold_answer, candidate)

        reflector_output: ReflectorOutput | None = None
        if not verdict and trial_idx < max_trials:
            # Diagnose only on a non-final failed trial; on the last trial
            # there is no use for the diagnosis. Paper Section 3.2.
            trial_outputs = [
                tr.actor_output for tr in trajectory
            ] + [actor_text]
            verdicts = [tr.evaluator_verdict for tr in trajectory] + [verdict]
            reflector_output = reflector.diagnose(
                trial_outputs, verdicts, constrained_mode=reflector_mode
            )
            episodic_memory.append(reflector_output)

        trajectory.append(
            TrialRecord(
                trial_idx=trial_idx,
                actor_output=actor_text,
                evaluator_verdict=verdict,
                reflector_output=reflector_output,
            )
        )

        if verdict:
            status = "solved"
            break

    return QuestionResult(
        question_id=item.question_id,
        status=status,  # type: ignore[arg-type]
        n_trials=len(trajectory),
        final_answer=final_answer,
        gold_answer=item.gold_answer,
        trajectory=trajectory,
        token_count=token_count,
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="elai_inference.c37_snowballing.run",
        description="Structure Snowballing replication driver (arXiv:2604.06066).",
    )
    parser.add_argument("--model", default=PAPER_MODEL_DEFAULT)
    parser.add_argument("--dataset", default="hotpotqa")
    parser.add_argument("--pool", choices=("A", "B"), required=True)
    parser.add_argument("--n-samples", type=int, default=None)
    parser.add_argument("--max-trials", type=int, default=PAPER_MAX_TRIALS)
    parser.add_argument("--temperature", type=float, default=PAPER_TEMPERATURE)
    parser.add_argument("--max-new-tokens", type=int, default=PAPER_MAX_NEW_TOKENS)
    parser.add_argument(
        "--reflector-mode",
        choices=("schema_hint", "outlines"),
        default="schema_hint",
    )
    parser.add_argument("--sidecar-url", default=DEFAULT_SIDECAR_URL)
    parser.add_argument("--dataset-path", default=None)
    parser.add_argument(
        "--output",
        required=True,
        help="JSONL output path; one QuestionResult per line.",
    )
    args = parser.parse_args(argv)

    if args.dataset != "hotpotqa":
        parser.error(
            f"Only --dataset hotpotqa is implemented (paper Section 4.1). "
            f"Got: {args.dataset}"
        )
    if not (0.0 < args.temperature < 2.0):
        parser.error(
            f"--temperature must be in (0.0, 2.0); got {args.temperature}"
        )
    if not (1 <= args.max_trials <= 10):
        parser.error(
            f"--max-trials must be in [1, 10]; got {args.max_trials}"
        )

    dataset_path = Path(args.dataset_path) if args.dataset_path else default_dataset_path()
    if args.pool == "A":
        items = load_pool_a(dataset_path)
    else:
        items = load_pool_b(dataset_path)
    if args.n_samples is not None and args.n_samples > 0:
        items = items[: args.n_samples]

    # Build per-role configs with the user-supplied (possibly paper-default)
    # decoding params. paper_role_config emits paper defaults; we override
    # temperature + max_new_tokens explicitly per CLI args.
    actor_config = paper_role_config("actor", model=args.model)
    actor_config = type(actor_config)(  # dataclass field replacement
        role=actor_config.role,
        model=actor_config.model,
        temperature=args.temperature,
        max_new_tokens=args.max_new_tokens,
        constrained=actor_config.constrained,
    )
    evaluator_config = paper_role_config("evaluator", model=args.model)
    evaluator_config = type(evaluator_config)(
        role=evaluator_config.role,
        model=evaluator_config.model,
        temperature=args.temperature,
        max_new_tokens=args.max_new_tokens,
        constrained=evaluator_config.constrained,
    )
    reflector_config = paper_role_config("reflector", model=args.model)
    reflector_config = type(reflector_config)(
        role=reflector_config.role,
        model=reflector_config.model,
        temperature=args.temperature,
        max_new_tokens=args.max_new_tokens,
        constrained=reflector_config.constrained,
    )

    actor = Actor(actor_config, sidecar_url=args.sidecar_url)
    evaluator = Evaluator(evaluator_config, sidecar_url=args.sidecar_url)
    reflector = Reflector(reflector_config, sidecar_url=args.sidecar_url)

    output_path = Path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    results: list[QuestionResult] = []
    start = time.time()
    with open(output_path, "w", encoding="utf-8") as fh:
        for idx, item in enumerate(items):
            result = _run_one_question(
                item,
                actor,
                evaluator,
                reflector,
                max_trials=args.max_trials,
                reflector_mode=args.reflector_mode,
            )
            results.append(result)
            fh.write(result.model_dump_json() + "\n")
            fh.flush()
            elapsed = time.time() - start
            print(
                f"[{idx + 1}/{len(items)}] {result.question_id}: "
                f"{result.status} ({result.n_trials} trials, {elapsed:.1f}s elapsed)",
                file=sys.stderr,
            )

    verdicts = [
        tr.evaluator_verdict
        for r in results
        for tr in r.trajectory
        if tr.trial_idx == r.n_trials
    ]
    diagnoses = [
        tr.reflector_output
        for r in results
        for tr in r.trajectory
        if tr.reflector_output is not None
    ]
    accuracy = compute_accuracy(verdicts)
    snowballing = compute_snowballing_rate(diagnoses)
    distribution = compute_error_distribution(diagnoses)
    summary: dict[str, Any] = {
        "pool": args.pool,
        "n_questions": len(results),
        "reflector_mode": args.reflector_mode,
        "model": args.model,
        "accuracy": accuracy,
        "formatting_fraction": snowballing,
        "error_distribution": distribution,
        "replication_threshold": REPLICATION_THRESHOLD,
        "severity_threshold": SEVERITY_THRESHOLD,
        "constrained_accuracy_ceiling": CONSTRAINED_ACCURACY_CEILING,
    }
    summary_path = output_path.with_suffix(output_path.suffix + ".summary.json")
    with open(summary_path, "w", encoding="utf-8") as fh:
        json.dump(summary, fh, indent=2)
    print(f"summary written: {summary_path}", file=sys.stderr)
    print(json.dumps(summary, indent=2))
    return 0


# RUNPOD-PENDING — live invocations (require GPU; document both quantisation
# strategies):
#
#   # Path (a): Qwen3-8B at 4-bit alongside existing Qwen2.5-Coder-7B sidecar.
#   #   VRAM: ~5 GB for Qwen3-8B NF4 + 1.92 GB existing sidecar = ~7 GB total.
#   ssh -i ~/.ssh/elai_remote -p 6465 maintainer@192.0.2.10 \
#     'source ~/.venvs/elai-sidecar/bin/activate && cd ~/elai_inference && \
#      python -m elai_inference.c37_snowballing.run \
#        --model Qwen/Qwen3-8B --pool A --n-samples 55 --max-trials 5 \
#        --temperature 0.1 --output runs/snowballing-poolA.jsonl'
#
#   # Path (b): Qwen3-8B at bf16 — STOP the existing sidecar first (~16 GB).
#   ssh -i ~/.ssh/elai_remote -p 6465 maintainer@192.0.2.10 \
#     'kill $(cat ~/sidecar.pid); source ~/.venvs/elai-sidecar/bin/activate && \
#      cd ~/elai_inference && python -m elai_inference.c37_snowballing.run \
#        --model Qwen/Qwen3-8B --pool B --n-samples 45 --max-trials 5 \
#        --temperature 0.1 --output runs/snowballing-poolB.jsonl'


if __name__ == "__main__":
    raise SystemExit(main())
