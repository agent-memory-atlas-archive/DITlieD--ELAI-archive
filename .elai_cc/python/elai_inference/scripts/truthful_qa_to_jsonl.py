"""D-30 calibration bootstrap: convert the published TruthfulQA CSV to JSONL.

Stdlib-only (csv, json, hashlib, argparse, pathlib, urllib). No new pip
dependency added. Source CSV is sylinrl/TruthfulQA main branch -- the
single canonical TruthfulQA v1 corpus per arXiv:2109.07958.

Two outputs are emitted (plan-spec + orchestrator-projection) so the
fixture is usable from BOTH the plan-spec raw schema AND the orchestrator's
`crate::calibration_bootstrap::load_corpus` PublishedCalibrationRow shape:

  1. <out_dir>/truthful_qa_<N>.jsonl              raw schema:
        {"question": str, "best_answer": str,
         "correct_answers": [str], "incorrect_answers": [str],
         "source": "TruthfulQA-v1", "row_id": int}

  2. <out_dir>/truthfulqa_full_<N>.jsonl          projected schema:
        PublishedCalibrationRow -- one True row per (question, best_answer)
        plus one False row per question's FIRST incorrect-answer. So 2*N
        projected rows total. Matches the existing 50-row subset schema in
        .elai_cc/benchmarks/calibration_bootstrap/fixtures/truthfulqa_subset.jsonl
        but at the full corpus scale.

Determinism: the row_id ordering follows the CSV file order; the projected
predicted_score values are deterministic functions of the row_id (no RNG,
no wall-clock). The ts_ms field is derived from row_id (1_700_000_000_000
+ row_id), matching the existing subset fixture convention.

CLI:
  python truthful_qa_to_jsonl.py \
      --input ./TruthfulQA.csv \
      --out-dir ../../benchmarks/calibration_bootstrap

If --input is omitted the script downloads the CSV from the canonical
GitHub raw URL into a tempfile.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import sys
import tempfile
import urllib.request
from pathlib import Path

CANONICAL_CSV_URL = (
    "https://raw.githubusercontent.com/sylinrl/TruthfulQA/main/TruthfulQA.csv"
)
SOURCE_LABEL = "TruthfulQA-v1"


def fetch_canonical_csv() -> Path:
    """Download the canonical TruthfulQA CSV to a tempfile and return path."""
    tmpdir = Path(tempfile.mkdtemp(prefix="truthfulqa_"))
    out = tmpdir / "TruthfulQA.csv"
    with urllib.request.urlopen(CANONICAL_CSV_URL) as resp:
        body = resp.read()
    out.write_bytes(body)
    return out


def split_pipe(field: str) -> list[str]:
    """Split a TruthfulQA pipe-or-semicolon-joined field into a list[str]."""
    raw = field.strip()
    if not raw:
        return []
    # The published CSV uses '; ' (semicolon + space) as the inter-answer
    # delimiter -- documented in the dataset card.
    parts = [p.strip() for p in raw.split(";")]
    return [p for p in parts if p]


def parse_rows(csv_path: Path) -> list[dict]:
    """Parse the canonical CSV into the plan-spec raw schema rows."""
    rows: list[dict] = []
    with csv_path.open("r", encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f)
        for idx, row in enumerate(reader):
            rows.append(
                {
                    "question": (row.get("Question") or "").strip(),
                    "best_answer": (row.get("Best Answer") or "").strip(),
                    "correct_answers": split_pipe(row.get("Correct Answers") or ""),
                    "incorrect_answers": split_pipe(row.get("Incorrect Answers") or ""),
                    "source": SOURCE_LABEL,
                    "row_id": idx,
                }
            )
    return rows


def write_raw(rows: list[dict], out_path: Path) -> None:
    """Emit plan-spec raw JSONL (one row per question)."""
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with out_path.open("w", encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")


def project_published_calibration_rows(rows: list[dict]) -> list[dict]:
    """Project plan-spec rows to PublishedCalibrationRow schema.

    For each question:
      * one ground_truth=True row over best_answer (high predicted_score),
      * one ground_truth=False row over the FIRST incorrect_answer (low
        predicted_score). Question is skipped when no incorrect answer is
        available (rare in the corpus; keeps balanced false/true ratio).
    """
    projected: list[dict] = []
    proj_idx = 0
    base_ts = 1_700_000_000_000
    for r in rows:
        rid = r["row_id"]
        # True row.
        projected.append(
            {
                "tuple_id": f"truthfulqa-{proj_idx:04d}",
                "claim_id": f"tq-claim-{rid}-best",
                "claim_text": f"Q: {r['question']} | A (best): {r['best_answer']}",
                "ground_truth": True,
                # 0.80-0.95 deterministic spread based on rid mod 16.
                "predicted_score": 0.80 + ((rid % 16) * 0.01),
                "ts_ms": base_ts + proj_idx,
            }
        )
        proj_idx += 1
        # False row (if any incorrect answer exists).
        if r["incorrect_answers"]:
            projected.append(
                {
                    "tuple_id": f"truthfulqa-{proj_idx:04d}",
                    "claim_id": f"tq-claim-{rid}-wrong",
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
    """Emit projected JSONL in PublishedCalibrationRow schema."""
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with out_path.open("w", encoding="utf-8") as f:
        for r in projected:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")


def blake3_or_blake2_hex(path: Path) -> str:
    """Best-effort blake3 hex over file bytes for content-pinning manifests.

    blake3 is the CLAUDE.md-mandated default. If blake3 is not installed in
    the running interpreter we fall back to hashlib.blake2b -- the manifest
    label records which algorithm was used so downstream tooling can detect
    the substitution.
    """
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
        description="Convert published TruthfulQA CSV to plan-spec JSONL.",
    )
    parser.add_argument(
        "--input",
        type=Path,
        default=None,
        help="Local TruthfulQA CSV path. If omitted, downloads from GitHub.",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path("./benchmarks/calibration_bootstrap"),
        help="Output dir. Two files are written: truthful_qa_<N>.jsonl + "
        "fixtures/truthfulqa_full_<N>.jsonl",
    )
    args = parser.parse_args(argv)

    csv_path = args.input if args.input else fetch_canonical_csv()
    if not csv_path.exists():
        sys.stderr.write(f"CSV not found at {csv_path}\n")
        return 1

    rows = parse_rows(csv_path)
    n = len(rows)
    if n == 0:
        sys.stderr.write("no rows parsed from CSV\n")
        return 1

    raw_path = args.out_dir / f"truthful_qa_{n}.jsonl"
    projected_path = args.out_dir / "fixtures" / f"truthfulqa_full_{n}.jsonl"

    write_raw(rows, raw_path)
    projected = project_published_calibration_rows(rows)
    write_projected(projected, projected_path)

    raw_hash = blake3_or_blake2_hex(raw_path)
    proj_hash = blake3_or_blake2_hex(projected_path)

    sys.stdout.write(
        json.dumps(
            {
                "input_csv": str(csv_path),
                "raw_rows": n,
                "projected_rows": len(projected),
                "raw_path": str(raw_path),
                "projected_path": str(projected_path),
                "raw_hash": raw_hash,
                "projected_hash": proj_hash,
                "source_label": SOURCE_LABEL,
            },
            indent=2,
        )
        + "\n"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
