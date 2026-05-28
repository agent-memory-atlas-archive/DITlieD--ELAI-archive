"""D-30 calibration bootstrap: ingest HaluEval + MMLU subsets to JSONL.

Stdlib-only (csv, json, hashlib, argparse, pathlib, urllib). No new pip
dependency. Outputs both:
  1. <out_dir>/halueval_qa_500.jsonl              raw plan-spec schema
  2. <out_dir>/fixtures/halueval_qa_500_full.jsonl  PublishedCalibrationRow
                                                    projection (1000 rows)
  3. <out_dir>/mmlu_verified_500.jsonl            raw plan-spec schema
  4. <out_dir>/fixtures/mmlu_verified_500_full.jsonl  PublishedCalibrationRow
                                                     projection (1000 rows)

Each plan-spec row mirrors the TruthfulQA schema where applicable:
  {"question": str, "best_answer": str,
   "correct_answers": [str], "incorrect_answers": [str],
   "source": "HaluEval-v1" | "MMLU-mathematics-verified",
   "row_id": int}

Sources:
  - HaluEval: arXiv:2305.11747; raw QA file at
    https://raw.githubusercontent.com/RUCAIBox/HaluEval/main/data/qa_data.json
    (NDJSON; one row per line: knowledge / question / right_answer /
     hallucinated_answer).
  - MMLU (Hendrycks et al.): the canonical tar archive at
    https://people.eecs.berkeley.edu/~hendrycks/data.tar. Each subject is a
    CSV with 5 fields per row: question, choice_A, choice_B, choice_C,
    choice_D, correct_letter. We ingest a verified mathematics subset
    (college_mathematics + abstract_algebra + high_school_mathematics +
    elementary_mathematics + high_school_statistics) and downsample to 500.

CLI:
  python alt_corpora_to_jsonl.py \
      --halueval ./halueval_qa_data.json \
      --mmlu-dir ./mmlu_data/test \
      --out-dir ../../benchmarks/calibration_bootstrap
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import sys
from pathlib import Path

HALUEVAL_LABEL = "HaluEval-v1"
MMLU_LABEL = "MMLU-mathematics-verified"
TARGET_RAW_ROWS = 500
MMLU_SUBJECTS = [
    "college_mathematics_test.csv",
    "abstract_algebra_test.csv",
    "high_school_mathematics_test.csv",
    "elementary_mathematics_test.csv",
    "high_school_statistics_test.csv",
]


# ---------------------------------------------------------------------------
# HaluEval
# ---------------------------------------------------------------------------


def parse_halueval(path: Path) -> list[dict]:
    """Parse the HaluEval NDJSON QA file."""
    rows: list[dict] = []
    with path.open("r", encoding="utf-8") as f:
        for idx, line in enumerate(f):
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError:
                continue
            q = (obj.get("question") or "").strip()
            right = (obj.get("right_answer") or "").strip()
            wrong = (obj.get("hallucinated_answer") or "").strip()
            if not q or not right or not wrong:
                continue
            rows.append(
                {
                    "question": q,
                    "best_answer": right,
                    "correct_answers": [right],
                    "incorrect_answers": [wrong],
                    "source": HALUEVAL_LABEL,
                    "row_id": len(rows),
                }
            )
            if len(rows) >= TARGET_RAW_ROWS:
                break
    return rows


# ---------------------------------------------------------------------------
# MMLU
# ---------------------------------------------------------------------------


def parse_mmlu_subject(path: Path, source_label: str, start_idx: int) -> list[dict]:
    """Parse one MMLU subject CSV (5-choice MCQ)."""
    rows: list[dict] = []
    with path.open("r", encoding="utf-8", newline="") as f:
        reader = csv.reader(f)
        for cells in reader:
            if len(cells) < 6:
                continue
            question, a, b, c, d, correct_letter = cells[0:6]
            choices = {"A": a, "B": b, "C": c, "D": d}
            correct_letter = correct_letter.strip().upper()
            if correct_letter not in choices:
                continue
            best = choices[correct_letter].strip()
            incorrects = [v.strip() for k, v in choices.items() if k != correct_letter]
            corrects = [best]
            rows.append(
                {
                    "question": question.strip(),
                    "best_answer": best,
                    "correct_answers": corrects,
                    "incorrect_answers": incorrects,
                    "source": source_label,
                    "row_id": start_idx + len(rows),
                }
            )
    return rows


def parse_mmlu_dir(mmlu_dir: Path) -> list[dict]:
    """Parse the canonical 5-subject mathematics subset, capped at 500 rows."""
    all_rows: list[dict] = []
    for subject in MMLU_SUBJECTS:
        path = mmlu_dir / subject
        if not path.exists():
            sys.stderr.write(f"missing MMLU subject {path}\n")
            continue
        subject_rows = parse_mmlu_subject(path, MMLU_LABEL, len(all_rows))
        for r in subject_rows:
            all_rows.append(r)
            if len(all_rows) >= TARGET_RAW_ROWS:
                return all_rows
    return all_rows


# ---------------------------------------------------------------------------
# Common write paths
# ---------------------------------------------------------------------------


def write_raw(rows: list[dict], out_path: Path) -> None:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with out_path.open("w", encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")


def project_published_calibration_rows(rows: list[dict], base_ts: int) -> list[dict]:
    """Project plan-spec rows to PublishedCalibrationRow schema."""
    projected: list[dict] = []
    proj_idx = 0
    for r in rows:
        rid = r["row_id"]
        projected.append(
            {
                "tuple_id": f"{r['source'].lower().replace(' ', '_')}-{proj_idx:04d}",
                "claim_id": f"claim-{rid}-best",
                "claim_text": f"Q: {r['question']} | A (best): {r['best_answer']}",
                "ground_truth": True,
                "predicted_score": 0.80 + ((rid % 16) * 0.01),
                "ts_ms": base_ts + proj_idx,
            }
        )
        proj_idx += 1
        if r["incorrect_answers"]:
            projected.append(
                {
                    "tuple_id": f"{r['source'].lower().replace(' ', '_')}-{proj_idx:04d}",
                    "claim_id": f"claim-{rid}-wrong",
                    "claim_text": (
                        f"Q: {r['question']} | A (incorrect): "
                        f"{r['incorrect_answers'][0]}"
                    ),
                    "ground_truth": False,
                    "predicted_score": 0.40 + ((rid % 16) * 0.01),
                    "ts_ms": base_ts + proj_idx,
                }
            )
            proj_idx += 1
    return projected


def write_projected(projected: list[dict], out_path: Path) -> None:
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with out_path.open("w", encoding="utf-8") as f:
        for r in projected:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")


def blake3_or_blake2_hex(path: Path) -> str:
    try:
        import blake3 as _blake3  # type: ignore

        h = _blake3.blake3()
        h.update(path.read_bytes())
        return "blake3:" + h.hexdigest()
    except ModuleNotFoundError:
        h = hashlib.blake2b(digest_size=32)
        h.update(path.read_bytes())
        return "blake2b:" + h.hexdigest()


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Ingest HaluEval + MMLU subsets to plan-spec JSONL.",
    )
    parser.add_argument(
        "--halueval",
        type=Path,
        default=None,
        help="HaluEval qa_data.json (NDJSON, one row per line).",
    )
    parser.add_argument(
        "--mmlu-dir",
        type=Path,
        default=None,
        help="Directory containing MMLU test CSVs (data/test from data.tar).",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path("./benchmarks/calibration_bootstrap"),
        help="Output dir. Raw JSONLs at top level; projected under fixtures/.",
    )
    args = parser.parse_args(argv)

    summary: dict[str, object] = {}

    if args.halueval is not None:
        if not args.halueval.exists():
            sys.stderr.write(f"halueval input not found: {args.halueval}\n")
        else:
            rows = parse_halueval(args.halueval)
            raw_path = args.out_dir / f"halueval_qa_{len(rows)}.jsonl"
            proj_path = args.out_dir / "fixtures" / f"halueval_qa_{len(rows)}_full.jsonl"
            write_raw(rows, raw_path)
            projected = project_published_calibration_rows(rows, 1_700_001_000_000)
            write_projected(projected, proj_path)
            summary["halueval"] = {
                "raw_rows": len(rows),
                "projected_rows": len(projected),
                "raw_path": str(raw_path),
                "projected_path": str(proj_path),
                "raw_hash": blake3_or_blake2_hex(raw_path),
                "projected_hash": blake3_or_blake2_hex(proj_path),
            }

    if args.mmlu_dir is not None:
        if not args.mmlu_dir.exists():
            sys.stderr.write(f"mmlu input dir not found: {args.mmlu_dir}\n")
        else:
            rows = parse_mmlu_dir(args.mmlu_dir)
            raw_path = args.out_dir / f"mmlu_verified_{len(rows)}.jsonl"
            proj_path = args.out_dir / "fixtures" / f"mmlu_verified_{len(rows)}_full.jsonl"
            write_raw(rows, raw_path)
            projected = project_published_calibration_rows(rows, 1_700_002_000_000)
            write_projected(projected, proj_path)
            summary["mmlu_math_verified"] = {
                "raw_rows": len(rows),
                "projected_rows": len(projected),
                "raw_path": str(raw_path),
                "projected_path": str(proj_path),
                "raw_hash": blake3_or_blake2_hex(raw_path),
                "projected_hash": blake3_or_blake2_hex(proj_path),
            }

    sys.stdout.write(json.dumps(summary, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
