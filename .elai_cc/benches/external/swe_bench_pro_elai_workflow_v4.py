"""
ELAI full-workflow eval against REAL SWE-Bench Pro tasks, v4.

Adds DUAL-VECTOR (code embedding) re-ranking stage on top of v3:

  v2 did: BM25 -> top-N -> inject into planning prompt.
  v3 did: BM25 -> top-K (default 50) -> cross-encoder (ms-marco MiniLM L6)
          -> top-N -> inject.
  v4 does: BM25 -> top-K (default 50) -> CODE-VEC re-rank
           (codebert-base ONNX, 768d, mean-pooled) blended 50/50 with
           BM25 norm -> top-15 -> CROSS-ENCODER (ms-marco MiniLM L6) ->
           top-8 -> inject.

Hypothesis (v3 finding): the web-QA-trained ms-marco cross-encoder
HURTS recall on code (rerank uplift was -0.10). A code-trained
embedding (codebert) before the cross-encoder should catch code-
specific signals BM25 alone misses (renames, idiomatic patterns,
related-function clusters), and the cross-encoder only sees the top
15 from the code-aware narrowing instead of the top 50 raw BM25.

Both query and passage are embedded with codebert (not granite-text):
the 384d granite output cannot be cosine-compared to 768d codebert
output without a learned projection. Embedding the issue text with
codebert is a small misalignment (it's not source code) but the
ranking signal carries because codebert was fine-tuned on natural-
language-to-code pairs and treats English with reasonable fidelity.

PIPELINE per task:
  1. Clone repo at base_commit (same as v2/v3).
  2. BM25 over the repo -> top-50 candidates.
  3. CODE-VEC re-rank: embed each candidate's first 2KB + the problem
     statement with codebert (mean-pool over attention_mask). Score =
     0.5 * bm25_norm + 0.5 * codebert_cos. Take top-15.
  4. CROSS-ENCODER (v3): top-15 -> ms-marco MiniLM L6 -> top-8.
  5. Inject top-8 into planning prompt (same shape as v3).
  6. Stages 1-4 (planning, testing_first, executing, auditing).
  7. 4-signal UniCR composite refusal gate.
  8. Structural scoring against gold patch.

PER-TASK RECORD ADDITIONS over v3:
  - "bm25_top50_count"
  - "codevec_rerank_top15"           list of file paths after step 3
  - "cross_encoder_top8"             list after step 4
  - "gold_files_in_top50_pct"        recall@50 (BM25 alone, head of BM25)
  - "gold_files_in_top15_pct"        recall@15 after code-vec
  - "gold_files_in_top8_pct"         recall@8 after cross-encoder
  - "codevec_lift"   = recall@15 - bm25_only_top15_recall
  - "rerank_lift"    = recall@8  - codevec_only_top8_recall

CLI args (additions over v3):
  --skip-codevec     pure BM25 + cross-encoder (v3 behavior)
  --skip-rerank      BM25 + codevec only (skip cross-encoder)
  Both          --> pure BM25 (v2 behavior)

Defensive: if onnxruntime is missing or the codebert session fails to
load, fall back to v3 behavior. If the cross-encoder fails, fall back
to BM25-only.

Run on pod:
    ELAI_LIVE_DISPATCH=1 python swe_bench_pro_elai_workflow_v4.py \
        --n-tasks 5 \
        --out /root/swe_bench_pro_elai_v4.jsonl \
        --aggregate /root/swe_bench_pro_elai_v4.agg.json
"""
from __future__ import annotations

import argparse
import json
import math
import os
import sys
import time
from pathlib import Path
from typing import Any

_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

# v2 primitives (corpus / clone / BM25 / scoring / prompt stages).
from swe_bench_pro_elai_workflow_v2 import (  # noqa: E402
    DATASET_NAME,
    DATASET_SPLIT,
    MODEL,
    _read_text_capped,
    _stage_planning,
    _stage_testing_first,
    _stage_executing,
    _stage_auditing,
    _tfidf_scan,
    audit_verdict,
    call_sidecar,
    clone_repo,
    extract_diff_block,
    extract_keywords,
    files_in_patch,
    load_corpus,
    score_patch,
    walk_repo_files,
)

# v3 primitives (cross-encoder reranker + composite refusal score).
from swe_bench_pro_elai_workflow_v3 import (  # noqa: E402
    DEFAULT_RERANKER_ONNX,
    DEFAULT_RERANKER_TOK,
    composite_refusal_score,
    rerank_candidates as cross_encoder_rerank,
    PASSAGE_BYTES,
)


# ------------------------------------------------------------------
# Code-vec (codebert) embedder — lazy-loaded module global
# ------------------------------------------------------------------

DEFAULT_CODEVEC_DIR = "/root/onnx_models/code_vec"
DEFAULT_CODEVEC_ONNX = "/root/onnx_models/code_vec/onnx/model.onnx"

# Codebert max position is 512. We use 256 to keep CPU embed time near
# the smoke-measured ~140 ms / snippet on the pod; with 256 it stays
# well under that budget per file.
CODEVEC_MAX_LEN = 256
# Query and passage byte caps (passage matches v3's 2KB convention).
CODEVEC_QUERY_CHARS = 2048
CODEVEC_PASSAGE_BYTES = 2048

_CODEVEC_SESSION: Any = None
_CODEVEC_TOKENIZER: Any = None
_CODEVEC_INPUT_NAMES: list[str] = []
_CODEVEC_LOAD_ERR: str = ""
_CODEVEC_BACKEND: str = "uninitialized"


def _load_codevec(model_dir: str, onnx_path: str) -> tuple[bool, str]:
    """Lazy-load codebert ONNX + HF tokenizer. Cached in module globals.

    Returns (ok, status). The tokenizer is loaded via transformers
    AutoTokenizer.from_pretrained(model_dir) the same way the staging
    smoke validates it; this gives correct merges.txt + special tokens
    without re-implementing BPE.
    """
    global _CODEVEC_SESSION, _CODEVEC_TOKENIZER, _CODEVEC_INPUT_NAMES
    global _CODEVEC_LOAD_ERR, _CODEVEC_BACKEND

    if _CODEVEC_SESSION is not None:
        return True, "cached"
    if _CODEVEC_LOAD_ERR:
        return False, _CODEVEC_LOAD_ERR

    try:
        import onnxruntime as ort  # type: ignore
    except ImportError as exc:
        _CODEVEC_LOAD_ERR = f"onnxruntime missing: {exc}"
        return False, _CODEVEC_LOAD_ERR

    onnx_p = Path(onnx_path)
    md_p = Path(model_dir)
    if not onnx_p.exists():
        _CODEVEC_LOAD_ERR = f"codevec onnx not found at {onnx_path}"
        return False, _CODEVEC_LOAD_ERR
    if not md_p.exists():
        _CODEVEC_LOAD_ERR = f"codevec model dir not found at {model_dir}"
        return False, _CODEVEC_LOAD_ERR

    try:
        from transformers import AutoTokenizer  # type: ignore

        tok = AutoTokenizer.from_pretrained(model_dir)
    except Exception as exc:
        _CODEVEC_LOAD_ERR = f"AutoTokenizer.from_pretrained failed: {exc}"
        return False, _CODEVEC_LOAD_ERR

    try:
        sess = ort.InferenceSession(str(onnx_p), providers=["CPUExecutionProvider"])
    except Exception as exc:
        _CODEVEC_LOAD_ERR = f"ort.InferenceSession failed: {exc}"
        return False, _CODEVEC_LOAD_ERR

    in_names = [i.name for i in sess.get_inputs()]
    if "input_ids" not in in_names or "attention_mask" not in in_names:
        _CODEVEC_LOAD_ERR = (
            f"codevec onnx unexpected inputs {in_names}, "
            "need input_ids + attention_mask"
        )
        return False, _CODEVEC_LOAD_ERR

    _CODEVEC_SESSION = sess
    _CODEVEC_TOKENIZER = tok
    _CODEVEC_INPUT_NAMES = in_names
    _CODEVEC_BACKEND = "codebert_base_onnx"
    return True, "loaded"


def _embed_codevec(text: str) -> Any:
    """Embed one chunk of text with codebert. Returns a (768,) numpy
    array (mean-pooled over attention mask), or None on failure."""
    import numpy as np  # type: ignore

    sess = _CODEVEC_SESSION
    tok = _CODEVEC_TOKENIZER
    if sess is None or tok is None:
        return None

    try:
        enc = tok(
            text,
            return_tensors="np",
            truncation=True,
            max_length=CODEVEC_MAX_LEN,
            padding="max_length",
        )
    except Exception:
        return None

    feed: dict[str, Any] = {}
    feed["input_ids"] = enc["input_ids"].astype(np.int64)
    feed["attention_mask"] = enc["attention_mask"].astype(np.int64)
    if "token_type_ids" in _CODEVEC_INPUT_NAMES and "token_type_ids" in enc:
        feed["token_type_ids"] = enc["token_type_ids"].astype(np.int64)

    try:
        out = sess.run(None, feed)
    except Exception:
        return None

    last_hidden = out[0]  # (1, seq, 768)
    mask = enc["attention_mask"][..., None].astype(last_hidden.dtype)
    summed = (last_hidden * mask).sum(axis=1)
    counts = mask.sum(axis=1).clip(min=1e-9)
    pooled = summed / counts
    return pooled[0]  # (768,)


def _cosine(a: Any, b: Any) -> float:
    import numpy as np  # type: ignore

    na = float(np.linalg.norm(a))
    nb = float(np.linalg.norm(b))
    if na == 0.0 or nb == 0.0:
        return 0.0
    return float(np.dot(a, b) / (na * nb))


def _bm25_norm(scores: list[float]) -> list[float]:
    """Min-max normalize BM25 scores into [0, 1]. Empty/degenerate
    inputs return all-zeros so the blend falls back to pure code-vec."""
    if not scores:
        return []
    lo = min(scores)
    hi = max(scores)
    if hi <= lo:
        return [0.0 for _ in scores]
    return [(s - lo) / (hi - lo) for s in scores]


def _passage_for(repo_root: Path, rel_path: str) -> str:
    """First ~CODEVEC_PASSAGE_BYTES of a file, with rel path prefix so
    the embedder sees the filename signal too. Mirrors v3's helper."""
    full = _read_text_capped(repo_root / rel_path)
    snippet = full[:CODEVEC_PASSAGE_BYTES]
    return f"FILE: {rel_path}\n\n{snippet}"


def codevec_rerank(
    repo_root: Path,
    query: str,
    candidates: list[tuple[str, float]],
    top_n: int,
    model_dir: str,
    onnx_path: str,
    bm25_weight: float = 0.5,
    code_weight: float = 0.5,
) -> tuple[list[tuple[str, float]], dict[str, Any]]:
    """Re-rank BM25 candidates by 0.5*bm25_norm + 0.5*codebert_cos.

    Returns (top_n_list, info). On any failure to load codebert, falls
    back to BM25 order (info.backend = "bm25_only_fallback")."""
    info: dict[str, Any] = {
        "backend": "uninitialized",
        "load_status": "",
        "n_pairs_scored": 0,
        "bm25_weight": bm25_weight,
        "code_weight": code_weight,
    }
    ok, status = _load_codevec(model_dir, onnx_path)
    info["load_status"] = status
    if not ok:
        info["backend"] = "bm25_only_fallback"
        return candidates[:top_n], info
    info["backend"] = _CODEVEC_BACKEND

    try:
        import numpy as np  # type: ignore
    except ImportError as exc:
        info["backend"] = "bm25_only_fallback"
        info["load_status"] = f"numpy missing: {exc}"
        return candidates[:top_n], info

    # Embed the query (issue text) once.
    q_text = query[:CODEVEC_QUERY_CHARS]
    q_vec = _embed_codevec(q_text)
    if q_vec is None:
        info["backend"] = "bm25_only_fallback"
        info["load_status"] = "query embed failed"
        return candidates[:top_n], info

    bm25_scores = [s for (_, s) in candidates]
    norms = _bm25_norm(bm25_scores)

    blended: list[tuple[str, float, float, float]] = []
    # (path, blend, bm25_norm, cos)
    for (rel, bm25_raw), bm25_n in zip(candidates, norms):
        passage = _passage_for(repo_root, rel)
        if not passage:
            blended.append((rel, -1e9, bm25_n, 0.0))
            continue
        p_vec = _embed_codevec(passage)
        if p_vec is None:
            # Code-vec failure on this file: fall back to BM25-only signal.
            blended.append((rel, bm25_n, bm25_n, 0.0))
            info.setdefault("embed_errors", []).append(rel)
            continue
        cos = _cosine(q_vec, p_vec)
        score = bm25_weight * bm25_n + code_weight * cos
        blended.append((rel, score, bm25_n, cos))
        info["n_pairs_scored"] += 1

    blended.sort(key=lambda x: x[1], reverse=True)
    top = [(rel, score) for (rel, score, _bm, _cs) in blended[:top_n]]
    info["top_breakdown"] = [
        {"path": rel, "blend": round(score, 4), "bm25_norm": round(bm, 4), "cos": round(cs, 4)}
        for (rel, score, bm, cs) in blended[:top_n]
    ]
    return top, info


# ------------------------------------------------------------------
# Task runner (v4)
# ------------------------------------------------------------------


def run_one_task(
    task: dict,
    repos_dir: Path,
    do_revision: bool,
    skip_grounding: bool,
    skip_codevec: bool,
    skip_rerank: bool,
    unicr_tau: float,
    top_n: int,
    bm25_top: int,
    codevec_top: int,
    codevec_dir: str,
    codevec_onnx: str,
    reranker_onnx: str,
    reranker_tok: str,
) -> dict:
    record: dict[str, Any] = {
        "instance_id": task["instance_id"],
        "repo": task["repo"],
        "base_commit": task.get("base_commit", ""),
        "ts_unix": int(time.time()),
        "version": "v4",
        "stages": {},
        "abstained": False,
        "unicr_verdict": "PROCEEDED",
    }

    repo_root: Path | None = None
    bm25_candidates: list[tuple[str, float]] = []
    codevec_top_list: list[tuple[str, float]] = []
    final_retrieved: list[tuple[str, float]] = []
    keywords: list[str] = extract_keywords(task["problem_statement"], top_n=20)

    grounding: dict[str, Any] = {
        "skipped": skip_grounding,
        "keywords": keywords,
        "bm25_top": bm25_top,
        "codevec_top": codevec_top,
        "rerank_top_n": top_n,
        "bm25_files": [],
        "bm25_scores": [],
        "codevec_top_files": [],
        "codevec_top_scores": [],
        "retrieved_files": [],
        "retrieved_scores": [],
        "bm25_top50_count": 0,
        "gold_files_in_top50_pct": 0.0,
        "gold_files_in_top15_pct": 0.0,
        "gold_files_in_top8_pct": 0.0,
        "codevec_lift": 0.0,
        "rerank_lift": 0.0,
        "codevec_info": {"backend": "skipped" if skip_codevec else "uninitialized"},
        "rerank_info": {"backend": "skipped" if skip_rerank else "uninitialized"},
        "clone_status": "not_attempted",
    }

    if not skip_grounding:
        repo_root, status = clone_repo(task["repo"], task["base_commit"], repos_dir)
        grounding["clone_status"] = status
        if repo_root is not None and repo_root.exists():
            try:
                # Step 2: wide BM25 net.
                files = walk_repo_files(repo_root)
                bm25_candidates = _tfidf_scan(repo_root, files, keywords, bm25_top)
                grounding["bm25_top50_count"] = len(bm25_candidates)

                # Step 3: code-vec rerank to top-codevec_top (default 15).
                if skip_codevec or not bm25_candidates:
                    codevec_top_list = bm25_candidates[:codevec_top]
                    grounding["codevec_info"] = {"backend": "skipped"}
                else:
                    codevec_top_list, cv_info = codevec_rerank(
                        repo_root,
                        query=task["problem_statement"],
                        candidates=bm25_candidates,
                        top_n=codevec_top,
                        model_dir=codevec_dir,
                        onnx_path=codevec_onnx,
                    )
                    grounding["codevec_info"] = cv_info

                # Step 4: cross-encoder rerank top-codevec_top -> top_n.
                if skip_rerank or not codevec_top_list:
                    final_retrieved = codevec_top_list[:top_n]
                    grounding["rerank_info"] = {"backend": "skipped"}
                else:
                    final_retrieved, rk_info = cross_encoder_rerank(
                        repo_root,
                        query=task["problem_statement"],
                        candidates=codevec_top_list,
                        top_n=top_n,
                        onnx_path=reranker_onnx,
                        tok_path=reranker_tok,
                    )
                    grounding["rerank_info"] = rk_info
            except Exception as exc:
                grounding["retrieval_error"] = str(exc)[:300]
                bm25_candidates = []
                codevec_top_list = []
                final_retrieved = []

            grounding["bm25_files"] = [p for (p, _) in bm25_candidates]
            grounding["bm25_scores"] = [round(s, 3) for (_, s) in bm25_candidates]
            grounding["codevec_top_files"] = [p for (p, _) in codevec_top_list]
            grounding["codevec_top_scores"] = [round(s, 4) for (_, s) in codevec_top_list]
            grounding["retrieved_files"] = [p for (p, _) in final_retrieved]
            grounding["retrieved_scores"] = [round(s, 4) for (_, s) in final_retrieved]

            # Recall@K bookkeeping.
            gold = files_in_patch(task["gold_patch"])
            n_gold = max(len(gold), 1)
            bm25_50 = set([p for (p, _) in bm25_candidates[:bm25_top]])
            cv_15 = set([p for (p, _) in codevec_top_list[:codevec_top]])
            ce_8 = set([p for (p, _) in final_retrieved[:top_n]])
            # Also need "BM25 head truncated to codevec_top" and "codevec
            # head truncated to top_n" to attribute lift.
            bm25_15 = set([p for (p, _) in bm25_candidates[:codevec_top]])
            cv_8 = set([p for (p, _) in codevec_top_list[:top_n]])

            r50 = len(gold & bm25_50) / n_gold
            r15 = len(gold & cv_15) / n_gold
            r8 = len(gold & ce_8) / n_gold
            r_bm25_15 = len(gold & bm25_15) / n_gold
            r_cv_8 = len(gold & cv_8) / n_gold

            grounding["gold_files_in_top50_pct"] = round(r50, 3)
            grounding["gold_files_in_top15_pct"] = round(r15, 3)
            grounding["gold_files_in_top8_pct"] = round(r8, 3)
            grounding["codevec_lift"] = round(r15 - r_bm25_15, 3)
            grounding["rerank_lift"] = round(r8 - r_cv_8, 3)
            # Keep v3-shape "retrieved_pct" as recall@8 for downstream
            # composite_refusal_score wiring.
            grounding["gold_files_in_retrieved_pct"] = round(r8, 3)
            grounding["gold_files_in_retrieved"] = len(gold & ce_8)

    record["grounding"] = grounding

    # Stage 1 PLANNING (top-8 injected).
    s1 = call_sidecar(
        _stage_planning(task, final_retrieved, repo_root, keywords),
        max_tokens=700,
    )
    record["stages"]["planning"] = s1

    s2 = call_sidecar(_stage_testing_first(task, s1["text"]), max_tokens=500)
    record["stages"]["testing_first"] = s2

    s3 = call_sidecar(_stage_executing(task, s1["text"]), max_tokens=2500)
    patch_v1 = extract_diff_block(s3["text"])
    record["stages"]["executing"] = s3
    record["patch_v1"] = patch_v1

    s4 = call_sidecar(_stage_auditing(task, patch_v1), max_tokens=600)
    v1 = audit_verdict(s4["text"])
    record["stages"]["auditing_v1"] = {**s4, **v1}

    # 4-signal UniCR composite refusal (same as v3).
    recall = grounding.get("gold_files_in_retrieved_pct")
    refusal_score, breakdown = composite_refusal_score(
        mean_logprob=s3.get("gg_mean_logprob"),
        entropy_mean=s3.get("gg_entropy_mean"),
        recall_at_8=recall,
        audit_v1_verdict=v1["verdict"],
    )
    should_refuse = refusal_score > unicr_tau
    refuse_reason = ""
    if not patch_v1.strip():
        should_refuse = True
        refuse_reason = "empty_patch"
    elif should_refuse:
        refuse_reason = f"composite={refusal_score:.3f} > tau={unicr_tau:.3f}"
    else:
        refuse_reason = "proceeded"

    record["unicr"] = {
        "version": "v4",
        "tau": unicr_tau,
        "composite_refusal_score": round(refusal_score, 4),
        "breakdown": breakdown,
        "mean_logprob": s3.get("gg_mean_logprob"),
        "entropy_mean": s3.get("gg_entropy_mean"),
        "recall_at_8": recall,
        "audit_v1_verdict": v1["verdict"],
        "should_refuse": should_refuse,
        "reason": refuse_reason,
    }

    if should_refuse:
        record["unicr_verdict"] = "UNICR_REFUSED_V4"
        record["abstained"] = True
        record["final_patch"] = ""
        record["score"] = {
            "files_overlap": 0.0,
            "hunks_overlap": 0.0,
            "funcs_overlap": 0.0,
            "composite": 0.0,
            "patch_present": False,
            "refused": True,
        }
        record["gold_patch_preview"] = task["gold_patch"][:600]
        total_wall = sum(s.get("latency_s", 0.0) for s in record["stages"].values())
        record["totals"] = {
            "wall_s": round(total_wall, 2),
            "tokens_in": sum(s.get("tokens_in", 0) for s in record["stages"].values()),
            "tokens_out": sum(s.get("tokens_out", 0) for s in record["stages"].values()),
            "n_stages": len(record["stages"]),
        }
        return record

    final_patch = patch_v1
    if do_revision and v1["verdict"] == "BLOCK":
        rev_prompt = (
            "Your reviewer flagged BLOCK findings. Rewrite the diff incorporating "
            "the feedback. Output ONLY the diff fence.\n\n"
            f"Audit findings:\n{s4['text'][:2000]}\n\n"
            f"Issue:\n{task['problem_statement'][:1500]}"
        )
        s3b = call_sidecar(rev_prompt, max_tokens=2500)
        patch_v2 = extract_diff_block(s3b["text"])
        record["stages"]["executing_v2"] = s3b
        record["patch_v2"] = patch_v2
        s4b = call_sidecar(_stage_auditing(task, patch_v2), max_tokens=600)
        v2 = audit_verdict(s4b["text"])
        record["stages"]["auditing_v2"] = {**s4b, **v2}
        final_patch = patch_v2 if patch_v2 else patch_v1

    score = score_patch(final_patch, task["gold_patch"])
    record["score"] = score
    record["final_patch"] = final_patch[:3000]
    record["gold_patch_preview"] = task["gold_patch"][:600]

    total_wall = sum(s.get("latency_s", 0.0) for s in record["stages"].values())
    total_tokens_in = sum(s.get("tokens_in", 0) for s in record["stages"].values())
    total_tokens_out = sum(s.get("tokens_out", 0) for s in record["stages"].values())
    record["totals"] = {
        "wall_s": round(total_wall, 2),
        "tokens_in": total_tokens_in,
        "tokens_out": total_tokens_out,
        "n_stages": len(record["stages"]),
    }
    return record


def _percentile(values: list[float], pct: float) -> float:
    if not values:
        return 0.0
    s = sorted(values)
    k = (len(s) - 1) * (pct / 100.0)
    f = math.floor(k)
    c = math.ceil(k)
    if f == c:
        return float(s[int(k)])
    return float(s[f] + (s[c] - s[f]) * (k - f))


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--n-tasks", type=int, default=5)
    p.add_argument("--out", default="/root/swe_bench_pro_elai_v4.jsonl")
    p.add_argument("--aggregate", default="/root/swe_bench_pro_elai_v4.agg.json")
    p.add_argument("--no-revision", action="store_true")
    p.add_argument("--unicr-tau", type=float, default=0.55)
    p.add_argument("--skip-grounding", action="store_true")
    p.add_argument("--skip-codevec", action="store_true",
                   help="ablation: BM25 + cross-encoder (v3 behavior)")
    p.add_argument("--skip-rerank", action="store_true",
                   help="ablation: BM25 + code-vec only (no cross-encoder)")
    p.add_argument("--repos-dir", default="/root/swe_eval_repos")
    p.add_argument("--top-n", type=int, default=8,
                   help="final top-N after cross-encoder")
    p.add_argument("--bm25-top", type=int, default=50,
                   help="how many BM25 candidates feed code-vec")
    p.add_argument("--codevec-top", type=int, default=15,
                   help="how many code-vec survivors feed the cross-encoder")
    p.add_argument("--codevec-dir", default=DEFAULT_CODEVEC_DIR)
    p.add_argument("--codevec-onnx", default=DEFAULT_CODEVEC_ONNX)
    p.add_argument("--reranker-onnx", default=DEFAULT_RERANKER_ONNX)
    p.add_argument("--reranker-tok", default=DEFAULT_RERANKER_TOK)
    args = p.parse_args()

    if os.environ.get("ELAI_LIVE_DISPATCH", "") != "1":
        sys.stderr.write("ELAI_LIVE_DISPATCH=1 required.\n")
        return 2

    repos_dir = Path(args.repos_dir)

    print(
        f"[corpus] loading {args.n_tasks} tasks from {DATASET_NAME} "
        f"split={DATASET_SPLIT}...",
        flush=True,
    )
    tasks = load_corpus(args.n_tasks)
    print(f"[corpus] loaded {len(tasks)} unique-repo tasks", flush=True)
    print(
        f"[config] unicr_tau={args.unicr_tau} skip_grounding={args.skip_grounding} "
        f"skip_codevec={args.skip_codevec} skip_rerank={args.skip_rerank} "
        f"top_n={args.top_n} bm25_top={args.bm25_top} "
        f"codevec_top={args.codevec_top} repos_dir={repos_dir}",
        flush=True,
    )
    print(
        f"[codevec] dir={args.codevec_dir} onnx={args.codevec_onnx}",
        flush=True,
    )
    print(
        f"[reranker] onnx={args.reranker_onnx} tok={args.reranker_tok}",
        flush=True,
    )

    all_records: list[dict] = []
    with open(args.out, "w", encoding="utf-8") as f:
        for i, task in enumerate(tasks):
            print(
                f"[task {i+1}/{len(tasks)}] {task['instance_id']} ({task['repo']})",
                flush=True,
            )
            try:
                rec = run_one_task(
                    task,
                    repos_dir=repos_dir,
                    do_revision=(not args.no_revision),
                    skip_grounding=args.skip_grounding,
                    skip_codevec=args.skip_codevec,
                    skip_rerank=args.skip_rerank,
                    unicr_tau=args.unicr_tau,
                    top_n=args.top_n,
                    bm25_top=args.bm25_top,
                    codevec_top=args.codevec_top,
                    codevec_dir=args.codevec_dir,
                    codevec_onnx=args.codevec_onnx,
                    reranker_onnx=args.reranker_onnx,
                    reranker_tok=args.reranker_tok,
                )
                f.write(json.dumps(rec) + "\n")
                f.flush()
                all_records.append(rec)
                sc = rec["score"]
                g = rec["grounding"]
                u = rec["unicr"]
                cv_b = g.get("codevec_info", {}).get("backend", "n/a")
                rk_b = g.get("rerank_info", {}).get("backend", "n/a")
                if rec["abstained"]:
                    print(
                        f"  UNICR_REFUSED_V4 score={u['composite_refusal_score']} "
                        f"reason={u['reason']} "
                        f"r@50={g.get('gold_files_in_top50_pct')} "
                        f"r@15={g.get('gold_files_in_top15_pct')} "
                        f"r@8={g.get('gold_files_in_top8_pct')} "
                        f"cv_lift={g.get('codevec_lift')} "
                        f"rk_lift={g.get('rerank_lift')} "
                        f"cv={cv_b} rk={rk_b} wall={rec['totals']['wall_s']}s",
                        flush=True,
                    )
                else:
                    print(
                        f"  composite={sc['composite']} files={sc['files_overlap']} "
                        f"hunks={sc['hunks_overlap']} funcs={sc['funcs_overlap']} "
                        f"r@50={g.get('gold_files_in_top50_pct')} "
                        f"r@15={g.get('gold_files_in_top15_pct')} "
                        f"r@8={g.get('gold_files_in_top8_pct')} "
                        f"cv_lift={g.get('codevec_lift')} "
                        f"rk_lift={g.get('rerank_lift')} "
                        f"refuse_score={u['composite_refusal_score']} "
                        f"cv={cv_b} rk={rk_b} wall={rec['totals']['wall_s']}s",
                        flush=True,
                    )
            except Exception as exc:
                err = {"instance_id": task["instance_id"], "error": str(exc)}
                f.write(json.dumps(err) + "\n")
                f.flush()
                print(f"  ERROR: {exc}", flush=True)

    if all_records:
        comps = [r["score"]["composite"] for r in all_records]
        walls = [r["totals"]["wall_s"] for r in all_records]
        tok_in = [r["totals"]["tokens_in"] for r in all_records]
        tok_out = [r["totals"]["tokens_out"] for r in all_records]
        block_v1 = sum(
            1 for r in all_records
            if r["stages"].get("auditing_v1", {}).get("verdict") == "BLOCK"
        )
        revised = sum(1 for r in all_records if "patch_v2" in r)
        abstained = sum(1 for r in all_records if r.get("abstained"))

        r50 = [r["grounding"].get("gold_files_in_top50_pct", 0.0) for r in all_records]
        r15 = [r["grounding"].get("gold_files_in_top15_pct", 0.0) for r in all_records]
        r8 = [r["grounding"].get("gold_files_in_top8_pct", 0.0) for r in all_records]
        cv_lift = [r["grounding"].get("codevec_lift", 0.0) for r in all_records]
        rk_lift = [r["grounding"].get("rerank_lift", 0.0) for r in all_records]
        refuse_scores = [
            r["unicr"]["composite_refusal_score"] for r in all_records
        ]
        proceeded = [r for r in all_records if not r.get("abstained")]
        mean_comp_proc = (
            round(sum(r["score"]["composite"] for r in proceeded) / len(proceeded), 3)
            if proceeded else 0.0
        )
        cv_backends = [
            r["grounding"].get("codevec_info", {}).get("backend", "n/a")
            for r in all_records
        ]
        rk_backends = [
            r["grounding"].get("rerank_info", {}).get("backend", "n/a")
            for r in all_records
        ]
        cv_backend_label = (
            max(set(cv_backends), key=cv_backends.count) if cv_backends else "n/a"
        )
        rk_backend_label = (
            max(set(rk_backends), key=rk_backends.count) if rk_backends else "n/a"
        )

        agg = {
            "n_tasks": len(all_records),
            "n_completed": len(all_records),
            "mean_composite": round(sum(comps) / len(comps), 3),
            "max_composite": max(comps),
            "mean_files_overlap": round(
                sum(r["score"]["files_overlap"] for r in all_records) / len(all_records), 3
            ),
            "mean_funcs_overlap": round(
                sum(r["score"]["funcs_overlap"] for r in all_records) / len(all_records), 3
            ),
            "mean_wall_s": round(sum(walls) / len(walls), 2),
            "mean_tokens_in": int(sum(tok_in) / len(tok_in)),
            "mean_tokens_out": int(sum(tok_out) / len(tok_out)),
            "audit_block_rate_v1": round(block_v1 / len(all_records), 3),
            "revision_rate": round(revised / len(all_records), 3),
            "mean_recall_at_50": round(sum(r50) / len(r50), 3),
            "mean_recall_at_15": round(sum(r15) / len(r15), 3),
            "mean_recall_at_8": round(sum(r8) / len(r8), 3),
            "mean_codevec_lift": round(sum(cv_lift) / len(cv_lift), 3),
            "mean_rerank_lift": round(sum(rk_lift) / len(rk_lift), 3),
            "codevec_backend": cv_backend_label,
            "rerank_backend": rk_backend_label,
            "refusal_rate": round(abstained / len(all_records), 3),
            "n_proceeded": len(proceeded),
            "mean_composite_among_proceeded": mean_comp_proc,
            "refusal_score_dist": {
                "min": round(min(refuse_scores), 4) if refuse_scores else 0.0,
                "p50": round(_percentile(refuse_scores, 50), 4),
                "p90": round(_percentile(refuse_scores, 90), 4),
                "max": round(max(refuse_scores), 4) if refuse_scores else 0.0,
            },
            "unicr_tau": args.unicr_tau,
            "skip_grounding": args.skip_grounding,
            "skip_codevec": args.skip_codevec,
            "skip_rerank": args.skip_rerank,
            "top_n": args.top_n,
            "bm25_top": args.bm25_top,
            "codevec_top": args.codevec_top,
            "model": MODEL,
            "dataset": DATASET_NAME,
            "dataset_split": DATASET_SPLIT,
            "version": "v4",
            "ts_unix": int(time.time()),
        }
        with open(args.aggregate, "w", encoding="utf-8") as f:
            json.dump(agg, f, indent=2)
        print("[aggregate]", json.dumps(agg, indent=2), flush=True)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
