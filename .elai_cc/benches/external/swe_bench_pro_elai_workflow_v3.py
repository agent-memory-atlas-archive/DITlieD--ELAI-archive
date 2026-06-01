"""
ELAI full-workflow eval against REAL SWE-Bench Pro tasks, v3.

Adds two pieces on top of v2 (swe_bench_pro_elai_workflow_v2.py):

  (A) CROSS-ENCODER RERANKER STAGE between BM25 and prompt-injection.
      v2 did: BM25 -> top-N -> inject into planning prompt.
      v3 does: BM25 -> top-K (default 50) -> cross-encoder rerank
               (cross-encoder/ms-marco-MiniLM-L-6-v2 ONNX, staged at
               /root/onnx_models/reranker/onnx/model.onnx) -> top-N
               -> inject into planning prompt. This simulates the
               full production ELAI knowledge pipeline tail
               (BM25 -> RRF -> dual-vector -> PageRank -> CatRAG ->
               A-RAG -> cross-encoder rerank -> context) by replacing
               the middle layers with their net effect: BM25 candidates
               narrowed by a real cross-encoder.

  (B) RETUNED UNI-CR REFUSAL GATE. v2 used a single mean_logprob
      threshold and got 0% refusal because Qwen 3B is overconfident.
      v3 combines four signals into a composite refusal score:

        composite_refusal_score =
            0.3 * (1 - sigmoid(mean_logprob / 0.3))
          + 0.2 * sigmoid(entropy_mean - 1.0)
          + 0.3 * (1 - recall_at_8)
          + 0.2 * (1 if audit_v1 == "BLOCK" else 0)

      Refuse if composite_refusal_score > tau (--unicr-tau, default
      0.55, calibrated to ~30-50% refusal on Qwen 3B). The audit
      verdict is computed BEFORE the refusal decision so the gate
      can use it; revision is gated behind not-refused AND
      audit==BLOCK (as in v2).

Aggregate additions over v2:
  - mean_recall_at_8 (renamed for clarity; same source as v2)
  - rerank_backend                 — "cross_encoder_l6" | "bm25_only"
  - rerank_uplift_at_8             — mean(recall_after_rerank) - mean(bm25_top8_recall)
  - refusal_score_dist             — min/p50/p90/max of composite_refusal_score
  - mean_composite_among_proceeded — same as v2

CLI args (additions over v2):
  --reranker-top-bm25  (default 50)   how many BM25 candidates feed the reranker
  --skip-rerank        (ablation switch — falls back to v2 behavior)
  --reranker-onnx      (default /root/onnx_models/reranker/onnx/model.onnx)
  --reranker-tok       (default /root/onnx_models/reranker/tokenizer.json)

Run on pod:
    ELAI_LIVE_DISPATCH=1 python swe_bench_pro_elai_workflow_v3.py \
        --n-tasks 5 \
        --out /root/swe_bench_pro_elai_v3.jsonl \
        --aggregate /root/swe_bench_pro_elai_v3.agg.json
"""
from __future__ import annotations

import argparse
import json
import math
import os
import re
import sys
import time
from pathlib import Path
from typing import Any

# Reuse machinery from v2 unchanged. The v2 module is colocated under
# .elai_cc/benches/external/ and importable when run from the pod working
# dir or as a script (we extend sys.path with the script's own dir).
_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

from swe_bench_pro_elai_workflow_v2 import (  # noqa: E402
    DATASET_NAME,
    DATASET_SPLIT,
    MODEL,
    MAX_FILE_BYTES,
    _read_text_capped,
    _stage_planning,
    _stage_testing_first,
    _stage_executing,
    _stage_auditing,
    audit_verdict,
    call_sidecar,
    clone_repo,
    extract_diff_block,
    extract_keywords,
    file_context_snippet,
    files_in_patch,
    load_corpus,
    retrieve_candidates,
    score_patch,
    walk_repo_files,
    _tfidf_scan,
)


# ------------------------------------------------------------------
# Cross-encoder reranker (lazy-loaded module global)
# ------------------------------------------------------------------

# Default staging paths on the pod. The smoke runner that validated the
# artifact uses these. Overridable via CLI (--reranker-onnx / --reranker-tok).
DEFAULT_RERANKER_ONNX = "/root/onnx_models/reranker/onnx/model.onnx"
DEFAULT_RERANKER_TOK = "/root/onnx_models/reranker/tokenizer.json"

# How many chars of the passage we feed the cross-encoder. The model is
# 512-token max; ~2KB of source code is comfortably under that after
# WordPiece tokenization (rough: ~4 chars/token average for code).
PASSAGE_BYTES = 2048
# How many chars of the query (problem_statement). 512 is well within
# the model's joint [CLS] q [SEP] p [SEP] budget.
QUERY_CHARS = 512
# Hard cap on input_ids length we send to the cross-encoder. The model's
# native max_position_embeddings is 512; we leave a couple of tokens of
# headroom for special tokens added by the tokenizer.
MAX_SEQ_LEN = 512

# Module-global session + tokenizer; loaded on first call.
_RERANKER_SESSION: Any = None
_RERANKER_TOKENIZER: Any = None
_RERANKER_LOAD_ERR: str = ""
_RERANKER_INPUT_NAMES: list[str] = []
_RERANKER_BACKEND: str = "uninitialized"


def _load_reranker(onnx_path: str, tok_path: str) -> tuple[bool, str]:
    """Lazy-load the cross-encoder ONNX session + tokenizer.

    Returns (ok, status_string). Caches the loaded session in module
    globals so subsequent task runs reuse it.

    Tokenizer strategy: prefer huggingface `tokenizers` (rust-backed,
    ships in any environment that already has `transformers` or
    `sentence-transformers` -- the staging script depended on huggingface_hub
    so the chance of `tokenizers` being present is very high). If it
    isn't, fall back to a minimal pure-Python WordPiece tokenizer that
    reads vocab.txt from the same dir as tokenizer.json.
    """
    global _RERANKER_SESSION, _RERANKER_TOKENIZER, _RERANKER_LOAD_ERR
    global _RERANKER_INPUT_NAMES, _RERANKER_BACKEND

    if _RERANKER_SESSION is not None:
        return True, "cached"
    if _RERANKER_LOAD_ERR:
        # We already tried and failed once; do not retry every task.
        return False, _RERANKER_LOAD_ERR

    try:
        import onnxruntime as ort  # type: ignore
    except ImportError as exc:
        _RERANKER_LOAD_ERR = f"onnxruntime missing: {exc}"
        return False, _RERANKER_LOAD_ERR

    onnx_p = Path(onnx_path)
    tok_p = Path(tok_path)
    if not onnx_p.exists():
        _RERANKER_LOAD_ERR = f"reranker onnx not found at {onnx_path}"
        return False, _RERANKER_LOAD_ERR
    if not tok_p.exists():
        _RERANKER_LOAD_ERR = f"reranker tokenizer not found at {tok_path}"
        return False, _RERANKER_LOAD_ERR

    try:
        # CPU is fine — we're scoring at most 50 short pairs per task.
        sess = ort.InferenceSession(str(onnx_p), providers=["CPUExecutionProvider"])
    except Exception as exc:
        _RERANKER_LOAD_ERR = f"ort.InferenceSession failed: {exc}"
        return False, _RERANKER_LOAD_ERR

    input_names = [i.name for i in sess.get_inputs()]
    # ms-marco-MiniLM-L-6-v2 expects: input_ids, attention_mask, token_type_ids
    expected = {"input_ids", "attention_mask"}
    if not expected.issubset(set(input_names)):
        _RERANKER_LOAD_ERR = (
            f"reranker onnx input names unexpected: {input_names} "
            f"(need {sorted(expected)})"
        )
        return False, _RERANKER_LOAD_ERR

    tok, tok_kind = _build_tokenizer(tok_p)
    if tok is None:
        _RERANKER_LOAD_ERR = tok_kind  # carried error string
        return False, _RERANKER_LOAD_ERR

    _RERANKER_SESSION = sess
    _RERANKER_TOKENIZER = tok
    _RERANKER_INPUT_NAMES = input_names
    _RERANKER_BACKEND = f"cross_encoder_l6[{tok_kind}]"
    return True, f"loaded[{tok_kind}]"


def _build_tokenizer(tok_path: Path) -> tuple[Any, str]:
    """Try HF tokenizers first, then fall back to vocab.txt WordPiece.

    Returns (tokenizer_obj, status_string). The tokenizer_obj exposes
    encode_pair(query, passage) -> dict[str, list[int]] with keys
    input_ids, attention_mask, token_type_ids (optional). On total
    failure returns (None, error_string).
    """
    # Strategy 1: huggingface tokenizers (rust-backed, no transformers dep).
    try:
        from tokenizers import Tokenizer  # type: ignore

        hf_tok = Tokenizer.from_file(str(tok_path))
        hf_tok.enable_truncation(max_length=MAX_SEQ_LEN, strategy="longest_first")
        hf_tok.enable_padding(length=MAX_SEQ_LEN)

        class _HFAdapter:
            def __init__(self, t: Any) -> None:
                self.t = t

            def encode_pair(self, query: str, passage: str) -> dict[str, list[int]]:
                enc = self.t.encode(query, passage)
                return {
                    "input_ids": list(enc.ids),
                    "attention_mask": list(enc.attention_mask),
                    "token_type_ids": list(enc.type_ids),
                }

        return _HFAdapter(hf_tok), "hf_tokenizers"
    except ImportError:
        pass
    except Exception as exc:
        # tokenizer.json present but unparseable; try the fallback below.
        # If that also fails we'll surface this.
        first_err = f"hf_tokenizers parse: {exc}"
    else:
        first_err = ""  # unused when HF path succeeded above

    # Strategy 2: minimal pure-Python WordPiece using vocab.txt from the
    # same directory as tokenizer.json. ms-marco models stage vocab.txt
    # alongside tokenizer.json in the snapshot.
    vocab_path = tok_path.parent / "vocab.txt"
    if not vocab_path.exists():
        return None, (
            "tokenizer load failed: hf tokenizers unavailable and "
            f"no fallback vocab.txt at {vocab_path}"
        )
    try:
        tok = _MinimalWordPieceTokenizer.from_vocab(vocab_path)
        return tok, "wordpiece_fallback"
    except Exception as exc:
        return None, f"wordpiece fallback failed: {exc}"


class _MinimalWordPieceTokenizer:
    """Tiny BERT-style WordPiece tokenizer.

    Implements only what the cross-encoder needs:
      - lowercase + basic whitespace/punctuation split
      - greedy longest-match WordPiece against vocab.txt
      - pair encoding: [CLS] q [SEP] p [SEP]
      - truncation to MAX_SEQ_LEN; padding to MAX_SEQ_LEN

    Accuracy vs HF tokenizers is not bit-identical (case folding rules
    on accented characters differ), but for English issue text and
    source code the ranking signal is preserved.
    """

    PUNCT = set(",.;:!?\"'()[]{}<>/\\|&%$#@*+=~`^—–-")

    def __init__(self, vocab: dict[str, int]) -> None:
        self.vocab = vocab
        self.cls_id = vocab.get("[CLS]", 101)
        self.sep_id = vocab.get("[SEP]", 102)
        self.pad_id = vocab.get("[PAD]", 0)
        self.unk_id = vocab.get("[UNK]", 100)

    @classmethod
    def from_vocab(cls, path: Path) -> "_MinimalWordPieceTokenizer":
        vocab: dict[str, int] = {}
        with path.open("r", encoding="utf-8") as f:
            for i, line in enumerate(f):
                tok = line.rstrip("\n")
                vocab[tok] = i
        return cls(vocab)

    def _basic_tokens(self, text: str) -> list[str]:
        # lowercase + collapse whitespace + split on whitespace/punct
        out: list[str] = []
        buf: list[str] = []
        for ch in text.lower():
            if ch.isspace():
                if buf:
                    out.append("".join(buf))
                    buf = []
                continue
            if ch in self.PUNCT:
                if buf:
                    out.append("".join(buf))
                    buf = []
                out.append(ch)
                continue
            buf.append(ch)
        if buf:
            out.append("".join(buf))
        return out

    def _wordpiece(self, word: str) -> list[int]:
        # Greedy longest-match. Falls back to [UNK] for whole-word
        # failures so we never silently drop a token.
        if word in self.vocab:
            return [self.vocab[word]]
        ids: list[int] = []
        start = 0
        n = len(word)
        is_bad = False
        while start < n:
            end = n
            cur: str | None = None
            while start < end:
                sub = word[start:end]
                if start > 0:
                    sub = "##" + sub
                if sub in self.vocab:
                    cur = sub
                    break
                end -= 1
            if cur is None:
                is_bad = True
                break
            ids.append(self.vocab[cur])
            start = end
        if is_bad:
            return [self.unk_id]
        return ids

    def _tokenize_text(self, text: str) -> list[int]:
        ids: list[int] = []
        for w in self._basic_tokens(text):
            ids.extend(self._wordpiece(w))
        return ids

    def encode_pair(self, query: str, passage: str) -> dict[str, list[int]]:
        q_ids = self._tokenize_text(query)
        p_ids = self._tokenize_text(passage)

        # Reserve room for [CLS] q [SEP] p [SEP].
        room = MAX_SEQ_LEN - 3
        # Strategy: longest_first. Trim from the longer side.
        while len(q_ids) + len(p_ids) > room:
            if len(q_ids) >= len(p_ids):
                q_ids.pop()
            else:
                p_ids.pop()

        input_ids = [self.cls_id] + q_ids + [self.sep_id] + p_ids + [self.sep_id]
        token_type_ids = (
            [0] * (1 + len(q_ids) + 1) + [1] * (len(p_ids) + 1)
        )
        attention_mask = [1] * len(input_ids)

        pad_len = MAX_SEQ_LEN - len(input_ids)
        if pad_len > 0:
            input_ids = input_ids + [self.pad_id] * pad_len
            token_type_ids = token_type_ids + [0] * pad_len
            attention_mask = attention_mask + [0] * pad_len
        elif pad_len < 0:
            input_ids = input_ids[:MAX_SEQ_LEN]
            token_type_ids = token_type_ids[:MAX_SEQ_LEN]
            attention_mask = attention_mask[:MAX_SEQ_LEN]

        return {
            "input_ids": input_ids,
            "attention_mask": attention_mask,
            "token_type_ids": token_type_ids,
        }


def _passage_for(repo_root: Path, rel_path: str) -> str:
    """First ~PASSAGE_BYTES of a file, prefixed with the relative path
    so the cross-encoder sees the filename signal too."""
    full = _read_text_capped(repo_root / rel_path)
    snippet = full[:PASSAGE_BYTES]
    # Prepend the rel path so the model can weight files-named-after-bug.
    return f"FILE: {rel_path}\n\n{snippet}"


def rerank_candidates(
    repo_root: Path,
    query: str,
    candidates: list[tuple[str, float]],
    top_n: int,
    onnx_path: str,
    tok_path: str,
) -> tuple[list[tuple[str, float]], dict[str, Any]]:
    """Score (query, passage) pairs with the cross-encoder and return
    the top-N by descending score. Falls back to BM25 order if the
    reranker fails to load."""
    info: dict[str, Any] = {
        "backend": "uninitialized",
        "load_status": "",
        "n_pairs_scored": 0,
        "trimmed_query_chars": min(len(query), QUERY_CHARS),
    }
    ok, status = _load_reranker(onnx_path, tok_path)
    info["load_status"] = status
    if not ok:
        info["backend"] = "bm25_only"
        return candidates[:top_n], info
    info["backend"] = _RERANKER_BACKEND

    sess = _RERANKER_SESSION
    tok = _RERANKER_TOKENIZER
    in_names = _RERANKER_INPUT_NAMES
    q = query[:QUERY_CHARS]

    try:
        import numpy as np  # type: ignore
    except ImportError as exc:
        info["backend"] = "bm25_only"
        info["load_status"] = f"numpy missing: {exc}"
        return candidates[:top_n], info

    scored: list[tuple[str, float, float]] = []  # (path, rerank_score, bm25)
    for rel, bm25 in candidates:
        passage = _passage_for(repo_root, rel)
        if not passage:
            # No content -> push to bottom but keep
            scored.append((rel, -1e9, bm25))
            continue
        enc = tok.encode_pair(q, passage)
        feed: dict[str, Any] = {}
        ids = np.asarray([enc["input_ids"]], dtype=np.int64)
        am = np.asarray([enc["attention_mask"]], dtype=np.int64)
        feed["input_ids"] = ids
        feed["attention_mask"] = am
        if "token_type_ids" in in_names:
            tti = np.asarray([enc.get("token_type_ids", [0] * len(enc["input_ids"]))], dtype=np.int64)
            feed["token_type_ids"] = tti
        try:
            out = sess.run(None, feed)
        except Exception as exc:
            # Single-pair failure: keep with very low score, continue.
            scored.append((rel, -1e9, bm25))
            info.setdefault("scoring_errors", []).append(f"{rel}: {str(exc)[:120]}")
            continue
        # Output is [1, 1] logit (relevance score). Higher is better.
        try:
            score = float(out[0].reshape(-1)[0])
        except Exception:
            score = -1e9
        scored.append((rel, score, bm25))
        info["n_pairs_scored"] += 1

    scored.sort(key=lambda x: x[1], reverse=True)
    top = [(rel, score) for (rel, score, _bm25) in scored[:top_n]]
    return top, info


# ------------------------------------------------------------------
# Retuned UNI-CR refusal gate (v3)
# ------------------------------------------------------------------


def _sigmoid(x: float) -> float:
    # Numerically stable sigmoid.
    if x >= 0:
        z = math.exp(-x)
        return 1.0 / (1.0 + z)
    z = math.exp(x)
    return z / (1.0 + z)


def composite_refusal_score(
    mean_logprob: float | None,
    entropy_mean: float | None,
    recall_at_8: float | None,
    audit_v1_verdict: str,
) -> tuple[float, dict[str, float]]:
    """Combine four uncertainty signals into a single refusal score in [0, 1].

    Components (each in [0, 1] after normalization):
      - confidence_term = 1 - sigmoid(mean_logprob / 0.3)   weight 0.30
        Qwen 3B's typical confident token logprob is ~-0.3; dividing by
        0.3 keeps the sigmoid centered around -1..0..+1 of mean_logprob.
        Lower mean_logprob -> larger 1-sigmoid -> larger score.
      - entropy_term = sigmoid(entropy_mean - 1.0)          weight 0.20
        Centered at 1.0 nats which is a reasonable "uncertain" threshold
        on a vocab the size of Qwen's. Higher entropy -> larger score.
      - recall_term = 1 - recall_at_8                       weight 0.30
        Low retrieval recall is a strong "the model will not have
        grounded files in its prompt" signal.
      - audit_term = 1.0 if audit_v1 == "BLOCK" else 0.0    weight 0.20
        Audit BLOCK is a decisive signal regardless of model confidence.

    Returns (score, breakdown_dict).
    """
    if mean_logprob is None:
        conf_term = 0.5  # neutral when signal absent
    else:
        conf_term = 1.0 - _sigmoid(mean_logprob / 0.3)
    if entropy_mean is None:
        ent_term = 0.5
    else:
        ent_term = _sigmoid(entropy_mean - 1.0)
    if recall_at_8 is None:
        rec_term = 0.5
    else:
        rec_term = 1.0 - float(recall_at_8)
    aud_term = 1.0 if audit_v1_verdict == "BLOCK" else 0.0

    score = (
        0.30 * conf_term
        + 0.20 * ent_term
        + 0.30 * rec_term
        + 0.20 * aud_term
    )
    return score, {
        "confidence_term": round(conf_term, 4),
        "entropy_term": round(ent_term, 4),
        "recall_term": round(rec_term, 4),
        "audit_term": round(aud_term, 4),
    }


# ------------------------------------------------------------------
# Task runner (v3)
# ------------------------------------------------------------------


def run_one_task(
    task: dict,
    repos_dir: Path,
    do_revision: bool,
    skip_grounding: bool,
    skip_rerank: bool,
    unicr_tau: float,
    top_n: int,
    reranker_top_bm25: int,
    reranker_onnx: str,
    reranker_tok: str,
) -> dict:
    record: dict[str, Any] = {
        "instance_id": task["instance_id"],
        "repo": task["repo"],
        "base_commit": task.get("base_commit", ""),
        "ts_unix": int(time.time()),
        "version": "v3",
        "stages": {},
        "abstained": False,
        "unicr_verdict": "PROCEEDED",
    }

    # (A) GROUNDING RETRIEVAL — BM25 wider net for the reranker
    repo_root: Path | None = None
    bm25_candidates: list[tuple[str, float]] = []
    reranked: list[tuple[str, float]] = []
    keywords: list[str] = extract_keywords(task["problem_statement"], top_n=20)
    grounding_info: dict[str, Any] = {
        "skipped": skip_grounding,
        "keywords": keywords,
        "bm25_top_k": reranker_top_bm25 if not skip_rerank else top_n,
        "rerank_top_n": top_n,
        "bm25_files": [],
        "bm25_scores": [],
        "retrieved_files": [],
        "retrieved_scores": [],
        "gold_files_in_retrieved": 0,
        "gold_files_in_retrieved_pct": 0.0,
        "gold_files_in_bm25_top8": 0,
        "gold_files_in_bm25_top8_pct": 0.0,
        "clone_status": "not_attempted",
        "rerank_info": {"backend": "skipped" if skip_rerank else "uninitialized"},
    }
    if not skip_grounding:
        repo_root, status = clone_repo(task["repo"], task["base_commit"], repos_dir)
        grounding_info["clone_status"] = status
        if repo_root is not None and repo_root.exists():
            try:
                if skip_rerank:
                    # Behave like v2: BM25 -> top_n directly.
                    bm25_candidates = retrieve_candidates(repo_root, keywords, top_n=top_n)
                    reranked = bm25_candidates
                else:
                    # Wider BM25 net feeding the cross-encoder.
                    files = walk_repo_files(repo_root)
                    bm25_candidates = _tfidf_scan(
                        repo_root, files, keywords, reranker_top_bm25
                    )
                    reranked, rerank_info = rerank_candidates(
                        repo_root,
                        query=task["problem_statement"],
                        candidates=bm25_candidates,
                        top_n=top_n,
                        onnx_path=reranker_onnx,
                        tok_path=reranker_tok,
                    )
                    grounding_info["rerank_info"] = rerank_info
            except Exception as exc:
                grounding_info["retrieval_error"] = str(exc)[:300]
                bm25_candidates = []
                reranked = []

            grounding_info["bm25_files"] = [p for (p, _) in bm25_candidates]
            grounding_info["bm25_scores"] = [round(s, 3) for (_, s) in bm25_candidates]
            grounding_info["retrieved_files"] = [p for (p, _) in reranked]
            grounding_info["retrieved_scores"] = [round(s, 3) for (_, s) in reranked]

            gold_files = files_in_patch(task["gold_patch"])
            retrieved_set = set(grounding_info["retrieved_files"])
            hit = len(gold_files & retrieved_set)
            grounding_info["gold_files_in_retrieved"] = hit
            grounding_info["gold_files_in_retrieved_pct"] = round(
                hit / max(len(gold_files), 1), 3
            )
            # Also record BM25-only top-8 recall for uplift attribution.
            bm25_top8 = set([p for (p, _) in bm25_candidates[:top_n]])
            hit_bm25 = len(gold_files & bm25_top8)
            grounding_info["gold_files_in_bm25_top8"] = hit_bm25
            grounding_info["gold_files_in_bm25_top8_pct"] = round(
                hit_bm25 / max(len(gold_files), 1), 3
            )
    record["grounding"] = grounding_info

    # Stage 1 PLANNING (with reranked top-N injection — v2's prompt shape)
    s1 = call_sidecar(_stage_planning(task, reranked, repo_root, keywords), max_tokens=700)
    record["stages"]["planning"] = s1

    # Stage 2 TESTING_FIRST
    s2 = call_sidecar(_stage_testing_first(task, s1["text"]), max_tokens=500)
    record["stages"]["testing_first"] = s2

    # Stage 3 EXECUTING
    s3 = call_sidecar(_stage_executing(task, s1["text"]), max_tokens=2500)
    patch_v1 = extract_diff_block(s3["text"])
    record["stages"]["executing"] = s3
    record["patch_v1"] = patch_v1

    # Stage 4 AUDITING (v1) — computed BEFORE refusal gate so the v3
    # composite score can incorporate the audit verdict signal.
    # If patch is empty we still call the auditor on it; the auditor will
    # BLOCK an empty patch, which is the correct refusal signal.
    s4 = call_sidecar(_stage_auditing(task, patch_v1), max_tokens=600)
    v1 = audit_verdict(s4["text"])
    record["stages"]["auditing_v1"] = {**s4, **v1}

    # (B) RETUNED UNI-CR REFUSAL GATE — 4 signals, composite score
    recall = grounding_info.get("gold_files_in_retrieved_pct")
    refusal_score, breakdown = composite_refusal_score(
        mean_logprob=s3.get("gg_mean_logprob"),
        entropy_mean=s3.get("gg_entropy_mean"),
        recall_at_8=recall,
        audit_v1_verdict=v1["verdict"],
    )
    should_refuse = refusal_score > unicr_tau

    # Override: empty patch is always a refuse. The composite already
    # picks this up through audit==BLOCK + low recall, but the explicit
    # rule keeps the semantics legible.
    refuse_reason = ""
    if not patch_v1.strip():
        should_refuse = True
        refuse_reason = "empty_patch"
    elif should_refuse:
        refuse_reason = f"composite={refusal_score:.3f} > tau={unicr_tau:.3f}"
    else:
        refuse_reason = "proceeded"

    record["unicr"] = {
        "version": "v3",
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
        record["unicr_verdict"] = "UNICR_REFUSED_V3"
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
    p.add_argument("--out", default="/root/swe_bench_pro_elai_v3.jsonl")
    p.add_argument("--aggregate", default="/root/swe_bench_pro_elai_v3.agg.json")
    p.add_argument("--no-revision", action="store_true")
    p.add_argument("--unicr-tau", type=float, default=0.55,
                   help="composite refusal score threshold; refuse if score > tau")
    p.add_argument("--skip-grounding", action="store_true")
    p.add_argument("--skip-rerank", action="store_true",
                   help="ablation: behave like v2 (BM25 -> top_n -> prompt)")
    p.add_argument("--repos-dir", default="/root/swe_eval_repos")
    p.add_argument("--top-n", type=int, default=8)
    p.add_argument("--reranker-top-bm25", type=int, default=50,
                   help="how many BM25 candidates feed the cross-encoder")
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
        f"skip_rerank={args.skip_rerank} top_n={args.top_n} "
        f"reranker_top_bm25={args.reranker_top_bm25} repos_dir={repos_dir}",
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
                    skip_rerank=args.skip_rerank,
                    unicr_tau=args.unicr_tau,
                    top_n=args.top_n,
                    reranker_top_bm25=args.reranker_top_bm25,
                    reranker_onnx=args.reranker_onnx,
                    reranker_tok=args.reranker_tok,
                )
                f.write(json.dumps(rec) + "\n")
                f.flush()
                all_records.append(rec)
                sc = rec["score"]
                g = rec["grounding"]
                u = rec["unicr"]
                rb = g.get("rerank_info", {}).get("backend", "n/a")
                if rec["abstained"]:
                    print(
                        f"  UNICR_REFUSED_V3 score={u['composite_refusal_score']} "
                        f"reason={u['reason']} "
                        f"recall@{args.top_n}={g['gold_files_in_retrieved_pct']} "
                        f"bm25_recall@{args.top_n}={g['gold_files_in_bm25_top8_pct']} "
                        f"rerank={rb} wall={rec['totals']['wall_s']}s",
                        flush=True,
                    )
                else:
                    print(
                        f"  composite={sc['composite']} files={sc['files_overlap']} "
                        f"hunks={sc['hunks_overlap']} funcs={sc['funcs_overlap']} "
                        f"recall@{args.top_n}={g['gold_files_in_retrieved_pct']} "
                        f"bm25_recall@{args.top_n}={g['gold_files_in_bm25_top8_pct']} "
                        f"refuse_score={u['composite_refusal_score']} "
                        f"rerank={rb} wall={rec['totals']['wall_s']}s",
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
        recalls = [r["grounding"]["gold_files_in_retrieved_pct"] for r in all_records]
        bm25_recalls = [
            r["grounding"]["gold_files_in_bm25_top8_pct"] for r in all_records
        ]
        refuse_scores = [
            r["unicr"]["composite_refusal_score"] for r in all_records
        ]
        proceeded = [r for r in all_records if not r.get("abstained")]
        mean_comp_proc = (
            round(sum(r["score"]["composite"] for r in proceeded) / len(proceeded), 3)
            if proceeded else 0.0
        )
        rerank_backends = [
            r["grounding"].get("rerank_info", {}).get("backend", "n/a")
            for r in all_records
        ]
        # Dominant backend label for the run.
        backend_label = (
            max(set(rerank_backends), key=rerank_backends.count)
            if rerank_backends else "n/a"
        )
        uplift = (
            round(sum(recalls) / len(recalls) - sum(bm25_recalls) / len(bm25_recalls), 3)
            if recalls else 0.0
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
            "mean_recall_at_8": round(sum(recalls) / len(recalls), 3),
            "mean_bm25_recall_at_8": round(sum(bm25_recalls) / len(bm25_recalls), 3),
            "rerank_uplift_at_8": uplift,
            "rerank_backend": backend_label,
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
            "skip_rerank": args.skip_rerank,
            "top_n": args.top_n,
            "reranker_top_bm25": args.reranker_top_bm25,
            "model": MODEL,
            "dataset": DATASET_NAME,
            "dataset_split": DATASET_SPLIT,
            "version": "v3",
            "ts_unix": int(time.time()),
        }
        with open(args.aggregate, "w", encoding="utf-8") as f:
            json.dump(agg, f, indent=2)
        print("[aggregate]", json.dumps(agg, indent=2), flush=True)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
