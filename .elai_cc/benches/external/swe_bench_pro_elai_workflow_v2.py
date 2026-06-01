"""
ELAI full-workflow eval against REAL SWE-Bench Pro tasks, v2.

Adds two simulations on top of v1 (swe_bench_pro_elai_workflow.py):

  (A) SIMULATED-GROUNDING RETRIEVAL: shallow-clones each task's repo at the
      base_commit, indexes text files with a stdlib BM25-ish TF-IDF scan
      (or whoosh if present), extracts keywords from the problem_statement,
      and surfaces the top-N candidate file paths. This simulates what a
      real B-29 SymbolGraphCache + B-34 entry-point detection layer would
      surface to the planner. The retrieved paths are injected into the
      planning prompt as RETRIEVED CANDIDATE FILES.

  (B) UNI-CR REFUSAL GATE: after EXECUTING, inspect gg_signals
      (mean_logprob, entropy_mean) plus the patch text. If the model's
      uncertainty exceeds tau (--unicr-tau, default -0.55) OR entropy is
      high OR the patch is empty, the task is REFUSED before auditing.
      This separates "model produced wrong patch" from "model abstained
      because uncertain" — what UniCR is supposed to enable.

  NOTE on tau: D-30's calibrated tau=0.590 is for binary-verdict refusal
  rate, not raw mean_logprob. -0.55 is a loose placeholder chosen so a
  3B model's typical confident token (~-0.3 to -0.4) proceeds and a
  shaky one (~-0.7+) refuses. Treat as a knob to sweep, not a calibrated
  threshold.

Aggregate additions:
  - mean_gold_recall_at_8        — how often retrieval surfaced the gold files
  - refusal_rate                 — sum(abstained) / n
  - mean_composite_among_proceeded — composite on non-refused only

CLI args (additions over v1):
  --unicr-tau    (default -0.55)
  --skip-grounding (ablation switch)
  --repos-dir    (default /root/swe_eval_repos)
  --top-n        (default 8)

Run on pod:
    ELAI_LIVE_DISPATCH=1 python swe_bench_pro_elai_workflow_v2.py \
        --n-tasks 5 \
        --out /root/swe_bench_pro_elai_v2.jsonl \
        --aggregate /root/swe_bench_pro_elai_v2.agg.json
"""
from __future__ import annotations

import argparse
import json
import math
import os
import re
import shutil
import subprocess
import sys
import time
from collections import Counter
from pathlib import Path
from typing import Any

import httpx


SIDECAR_URL = os.environ.get("ANTHROPIC_BASE_URL", "http://localhost:8100") + "/v1/messages"
API_KEY = os.environ.get("ANTHROPIC_API_KEY", "local-dummy")
MODEL = "qwen"
DATASET_NAME = os.environ.get("SWE_BENCH_PRO_DATASET", "ScaleAI/SWE-bench_Pro")
DATASET_SPLIT = os.environ.get("SWE_BENCH_PRO_SPLIT", "test")

# Per-file read cap to avoid OOM on huge vendored blobs.
MAX_FILE_BYTES = 50 * 1024
# Cap on how many files we'll index per repo.
MAX_FILES_PER_REPO = 5000

# Skip these path fragments outright (directory components).
SKIP_DIR_FRAGMENTS = (
    ".git",
    "node_modules",
    "dist",
    "build",
    "target",
    "vendor",
    "__pycache__",
    ".venv",
    "venv",
    ".tox",
    ".mypy_cache",
    ".pytest_cache",
    ".idea",
    ".vscode",
)

# Skip these exact file suffixes / names.
SKIP_FILE_SUFFIXES = (
    ".min.js",
    ".min.css",
    ".lock",
    ".pyc",
    ".pyo",
    ".so",
    ".dll",
    ".dylib",
    ".exe",
    ".jpg",
    ".jpeg",
    ".png",
    ".gif",
    ".pdf",
    ".zip",
    ".tar",
    ".gz",
    ".bz2",
    ".xz",
    ".whl",
    ".jar",
    ".class",
    ".o",
    ".a",
    ".bin",
    ".woff",
    ".woff2",
    ".ttf",
    ".otf",
    ".ico",
    ".svg",
    ".mp4",
    ".mp3",
    ".wav",
)
SKIP_FILE_NAMES = (
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
    "poetry.lock",
    "Pipfile.lock",
    "composer.lock",
    "go.sum",
)

# Text file extensions we'll actually index. Anything else is skipped to
# avoid spending IDF mass on binary noise.
TEXT_EXTS = (
    ".py", ".pyi", ".rs", ".js", ".jsx", ".ts", ".tsx", ".go", ".java",
    ".c", ".cc", ".cpp", ".cxx", ".h", ".hpp", ".rb", ".php", ".cs",
    ".swift", ".kt", ".scala", ".clj", ".ex", ".exs", ".erl", ".lua",
    ".sh", ".bash", ".zsh", ".fish", ".ps1",
    ".md", ".rst", ".txt", ".cfg", ".ini", ".toml", ".yaml", ".yml",
    ".json", ".xml", ".html", ".htm", ".css", ".scss", ".sass", ".less",
    ".sql", ".proto", ".graphql", ".vue", ".svelte",
)

STOPWORDS = frozenset({
    "the", "and", "for", "with", "that", "this", "from", "have", "has", "had",
    "are", "was", "were", "been", "being", "but", "not", "you", "your", "our",
    "their", "they", "them", "his", "her", "she", "him", "what", "which", "when",
    "where", "who", "why", "how", "all", "any", "some", "each", "other", "into",
    "out", "over", "under", "than", "then", "there", "here", "also", "only",
    "would", "could", "should", "will", "shall", "may", "might", "must", "can",
    "could", "did", "does", "doing", "done", "get", "got", "gets", "getting",
    "make", "makes", "made", "making", "use", "uses", "used", "using",
    "issue", "bug", "fix", "fixed", "error", "errors", "fail", "failed",
    "fails", "failing", "test", "tests", "testing", "tested", "code", "function",
    "class", "method", "methods", "file", "files", "line", "lines", "case",
    "cases", "value", "values", "type", "types", "data", "string", "strings",
    "number", "numbers", "list", "dict", "set", "tuple", "true", "false",
    "none", "null", "nil", "self", "cls", "args", "kwargs", "return", "returns",
    "returned", "returning", "raise", "raises", "raised", "raising", "throw",
    "throws", "thrown", "throwing", "if", "else", "elif", "while", "for",
    "in", "on", "at", "by", "to", "of", "as", "is", "be", "or", "an", "a",
    "no", "yes", "do", "we", "i", "it", "its", "us",
})

TOKEN_RE = re.compile(r"[a-z_][a-z0-9_]{3,}")


def load_corpus(n_tasks: int) -> list[dict]:
    """Load N tasks from the SWE-Bench Pro public set via HF datasets."""
    try:
        from datasets import load_dataset
    except ImportError:
        sys.stderr.write("huggingface datasets library required: pip install datasets\n")
        raise

    ds = load_dataset(DATASET_NAME, split=DATASET_SPLIT, streaming=True)
    tasks = []
    seen_repos: set[str] = set()
    for row in ds:
        repo = row.get("repo", "")
        if repo in seen_repos:
            continue
        seen_repos.add(repo)
        patch = row.get("patch", "")
        if not patch or "diff --git" not in patch:
            continue
        tasks.append({
            "instance_id": row.get("instance_id", ""),
            "repo": repo,
            "base_commit": row.get("base_commit", ""),
            "problem_statement": row.get("problem_statement", "")[:4000],
            "gold_patch": patch,
            "FAIL_TO_PASS": row.get("FAIL_TO_PASS", []),
            "PASS_TO_PASS": row.get("PASS_TO_PASS", [])[:5],
        })
        if len(tasks) >= n_tasks:
            break
    return tasks


def call_sidecar(prompt: str, max_tokens: int = 1500, timeout_s: float = 240.0) -> dict:
    body = {
        "model": MODEL,
        "max_tokens": max_tokens,
        "messages": [{"role": "user", "content": prompt}],
    }
    headers = {"Content-Type": "application/json", "x-api-key": API_KEY}
    t0 = time.time()
    with httpx.Client(timeout=timeout_s) as c:
        r = c.post(SIDECAR_URL, json=body, headers=headers)
    latency = time.time() - t0
    r.raise_for_status()
    data = r.json()
    text = "".join(b.get("text", "") for b in data.get("content", []) if b.get("type") == "text")
    usage = data.get("usage", {})
    meta = data.get("meta", {})
    gg = meta.get("gg_signals", {})
    return {
        "text": text,
        "latency_s": round(latency, 3),
        "tokens_in": int(usage.get("input_tokens", 0)),
        "tokens_out": int(usage.get("output_tokens", 0)),
        "gg_mean_logprob": gg.get("mean_logprob"),
        "gg_min_logprob": gg.get("min_logprob"),
        "gg_entropy_mean": gg.get("entropy_mean"),
        "gg_novelty": gg.get("generation_novelty"),
    }


# ------------------------------------------------------------------
# (A) SIMULATED GROUNDING RETRIEVAL
# ------------------------------------------------------------------


def _safe_slug(repo: str, base_commit: str) -> str:
    safe_repo = re.sub(r"[^A-Za-z0-9_.-]", "_", repo)
    short = (base_commit or "nocommit")[:12]
    return f"{safe_repo}__{short}"


def _run(cmd: list[str], cwd: str | None = None, timeout_s: int = 600) -> tuple[int, str, str]:
    try:
        p = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
            timeout=timeout_s,
        )
        return p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired as exc:
        return 124, exc.stdout or "", exc.stderr or "timeout"
    except FileNotFoundError as exc:
        return 127, "", str(exc)


def clone_repo(repo: str, base_commit: str, repos_dir: Path) -> tuple[Path | None, str]:
    """Shallow-clone repo at base_commit. Returns (path, status_message)."""
    if not repo or "/" not in repo:
        return None, f"invalid repo: {repo!r}"
    slug = _safe_slug(repo, base_commit)
    target = repos_dir / slug
    if target.exists() and any(target.iterdir()):
        # idempotent: trust existing checkout
        return target, "cached"
    repos_dir.mkdir(parents=True, exist_ok=True)
    url = f"https://github.com/{repo}.git"

    # Primary path: filter clone + fetch the exact commit, then check it out.
    rc, _, err = _run(
        ["git", "clone", "--filter=blob:none", "--no-checkout", url, str(target)],
        timeout_s=600,
    )
    if rc != 0:
        if target.exists():
            shutil.rmtree(target, ignore_errors=True)
        return None, f"clone failed: {err[:300]}"

    if base_commit:
        rc, _, err = _run(
            ["git", "fetch", "origin", base_commit, "--depth", "1"],
            cwd=str(target),
            timeout_s=600,
        )
        if rc != 0:
            # try a broader fetch as fallback
            rc2, _, err2 = _run(
                ["git", "fetch", "origin", "--depth", "1"],
                cwd=str(target),
                timeout_s=600,
            )
            if rc2 != 0:
                return target, f"fetch warn: {err[:200]} / {err2[:200]}"

        rc, _, err = _run(
            ["git", "checkout", base_commit],
            cwd=str(target),
            timeout_s=300,
        )
        if rc != 0:
            # Fall back to checking out HEAD so we still have something to index.
            rc2, _, err2 = _run(
                ["git", "checkout", "FETCH_HEAD"],
                cwd=str(target),
                timeout_s=300,
            )
            if rc2 != 0:
                return target, f"checkout warn: {err[:200]} / {err2[:200]}"
    else:
        # No base_commit: just check out default branch.
        _run(["git", "checkout", "HEAD"], cwd=str(target), timeout_s=300)

    return target, "cloned"


def _is_indexable(path: Path) -> bool:
    name = path.name
    if name in SKIP_FILE_NAMES:
        return False
    lower = name.lower()
    for suf in SKIP_FILE_SUFFIXES:
        if lower.endswith(suf):
            return False
    # require a recognized text extension
    for ext in TEXT_EXTS:
        if lower.endswith(ext):
            return True
    return False


def _path_skipped(rel_parts: tuple[str, ...]) -> bool:
    for part in rel_parts:
        if part in SKIP_DIR_FRAGMENTS:
            return True
        if part.endswith(".egg-info"):
            return True
    return False


def _read_text_capped(path: Path) -> str:
    try:
        with path.open("rb") as f:
            blob = f.read(MAX_FILE_BYTES)
    except OSError:
        return ""
    # Heuristic: skip if there are too many NULs (binary).
    if blob.count(b"\x00") > 4:
        return ""
    try:
        return blob.decode("utf-8", errors="ignore")
    except Exception:
        return ""


def extract_keywords(text: str, top_n: int = 20) -> list[str]:
    """Lowercase, tokenize, strip stopwords, keep tokens >=4 chars, top-N by freq."""
    lowered = text.lower()
    toks = TOKEN_RE.findall(lowered)
    counts: Counter[str] = Counter()
    for t in toks:
        if t in STOPWORDS:
            continue
        if len(t) < 4:
            continue
        counts[t] += 1
    return [w for (w, _) in counts.most_common(top_n)]


def walk_repo_files(repo_root: Path, cap: int = MAX_FILES_PER_REPO) -> list[Path]:
    out: list[Path] = []
    for root, dirs, files in os.walk(repo_root):
        # mutate dirs in-place to prune walk
        dirs[:] = [d for d in dirs if d not in SKIP_DIR_FRAGMENTS and not d.endswith(".egg-info")]
        for fn in files:
            p = Path(root) / fn
            try:
                rel = p.relative_to(repo_root)
            except ValueError:
                continue
            if _path_skipped(rel.parts[:-1]):
                continue
            if not _is_indexable(p):
                continue
            out.append(p)
            if len(out) >= cap:
                return out
    return out


def _try_whoosh_index(repo_root: Path, files: list[Path], keywords: list[str], top_n: int) -> list[tuple[str, float]] | None:
    try:
        from whoosh.fields import Schema, TEXT, ID  # type: ignore
        from whoosh.analysis import StemmingAnalyzer  # type: ignore
        from whoosh import index as whoosh_index  # type: ignore
        from whoosh.qparser import OrGroup, MultifieldParser  # type: ignore
    except ImportError:
        return None
    try:
        idx_dir = repo_root.parent / (repo_root.name + ".whoosh")
        if idx_dir.exists():
            shutil.rmtree(idx_dir, ignore_errors=True)
        idx_dir.mkdir(parents=True, exist_ok=True)
        schema = Schema(
            path=ID(stored=True, unique=True),
            content=TEXT(analyzer=StemmingAnalyzer(), stored=False),
        )
        ix = whoosh_index.create_in(str(idx_dir), schema)
        writer = ix.writer()
        for p in files:
            txt = _read_text_capped(p)
            if not txt:
                continue
            try:
                rel = str(p.relative_to(repo_root)).replace("\\", "/")
            except ValueError:
                continue
            writer.add_document(path=rel, content=txt)
        writer.commit()

        with ix.searcher() as searcher:
            parser = MultifieldParser(["content", "path"], schema=schema, group=OrGroup)
            q = parser.parse(" ".join(keywords))
            hits = searcher.search(q, limit=top_n)
            return [(h["path"], float(h.score)) for h in hits]
    except Exception:
        return None


def _tfidf_scan(
    repo_root: Path,
    files: list[Path],
    keywords: list[str],
    top_n: int,
) -> list[tuple[str, float]]:
    """Pure-Python TF-IDF over keywords only. Cheap; we only care about K terms."""
    if not keywords:
        return []
    kw_set = set(keywords)
    # df[term] = # docs containing term
    df: Counter[str] = Counter()
    # per-doc tf restricted to kw_set
    doc_tfs: list[tuple[str, dict[str, int], int]] = []  # (rel_path, tf, doc_len)
    for p in files:
        txt = _read_text_capped(p)
        if not txt:
            continue
        try:
            rel = str(p.relative_to(repo_root)).replace("\\", "/")
        except ValueError:
            continue
        toks = TOKEN_RE.findall(txt.lower())
        if not toks:
            continue
        tf: dict[str, int] = {}
        for t in toks:
            if t in kw_set:
                tf[t] = tf.get(t, 0) + 1
        # Also bias by path-name keyword hits (helps surface files named after the bug).
        path_lower = rel.lower()
        for kw in kw_set:
            if kw in path_lower:
                tf[kw] = tf.get(kw, 0) + 5  # path hits get a strong boost
        if not tf:
            continue
        for term in tf:
            df[term] += 1
        doc_tfs.append((rel, tf, len(toks) or 1))
    n_docs = max(len(doc_tfs), 1)
    # idf with BM25-ish smoothing
    idf = {term: math.log((n_docs - dfv + 0.5) / (dfv + 0.5) + 1.0) for term, dfv in df.items()}
    # BM25 params
    k1 = 1.5
    b = 0.75
    avgdl = sum(dl for (_, _, dl) in doc_tfs) / n_docs if doc_tfs else 1.0
    scores: list[tuple[str, float]] = []
    for rel, tf, dl in doc_tfs:
        s = 0.0
        for term, freq in tf.items():
            idf_t = idf.get(term, 0.0)
            denom = freq + k1 * (1 - b + b * (dl / max(avgdl, 1.0)))
            s += idf_t * (freq * (k1 + 1)) / max(denom, 1e-9)
        scores.append((rel, s))
    scores.sort(key=lambda x: x[1], reverse=True)
    return scores[:top_n]


def retrieve_candidates(
    repo_root: Path,
    keywords: list[str],
    top_n: int,
) -> list[tuple[str, float]]:
    files = walk_repo_files(repo_root)
    # Try whoosh first; fall back to pure-Python BM25-ish scan.
    via_whoosh = _try_whoosh_index(repo_root, files, keywords, top_n)
    if via_whoosh:
        return via_whoosh
    return _tfidf_scan(repo_root, files, keywords, top_n)


def file_context_snippet(repo_root: Path, rel_path: str, keywords: list[str]) -> str:
    """Return up to 4 numbered lines from the file matching any keyword."""
    p = repo_root / rel_path
    txt = _read_text_capped(p)
    if not txt:
        return ""
    kw_set = set(keywords)
    lines = txt.splitlines()
    picks: list[str] = []
    for i, line in enumerate(lines, start=1):
        low = line.lower()
        if any(kw in low for kw in kw_set):
            trimmed = line.strip()[:160]
            picks.append(f"  L{i}: {trimmed}")
            if len(picks) >= 4:
                break
    if not picks:
        return ""
    return "\n".join(picks)


# ------------------------------------------------------------------
# (B) STAGE PROMPTS  (planning gets the retrieved-files injection)
# ------------------------------------------------------------------


def _stage_planning(task: dict, retrieved: list[tuple[str, float]], repo_root: Path | None, keywords: list[str]) -> str:
    if retrieved and repo_root is not None:
        lines = ["", "RETRIEVED CANDIDATE FILES (top-{} from repo grounding; PREFER these):".format(len(retrieved))]
        for rel, score in retrieved:
            snippet = file_context_snippet(repo_root, rel, keywords)
            lines.append(f"- {rel}  (score={score:.2f})")
            if snippet:
                lines.append(snippet)
        retrieved_block = "\n".join(lines) + "\n\n"
    else:
        retrieved_block = ""
    return (
        "You are an experienced software engineer. Read this GitHub issue and "
        "produce a STRUCTURED PLAN (no patch yet).\n\n"
        f"Repo: {task['repo']}\n"
        f"Instance: {task['instance_id']}\n\n"
        f"Issue:\n{task['problem_statement']}\n"
        f"{retrieved_block}"
        "Output sections (markdown):\n"
        "## Files to edit\n(list relative paths, one per line, only files you "
        "would modify; PREFER files from the RETRIEVED CANDIDATE FILES block "
        "above when they are relevant)\n\n"
        "## Root cause hypothesis\n(2-3 sentences)\n\n"
        "## Risk rating\n(LOW / MEDIUM / HIGH and one-sentence justification)"
    )


def _stage_testing_first(task: dict, plan_text: str) -> str:
    return (
        "Same issue as before. Now list the tests that SHOULD fail before your "
        "fix and pass after. Don't run them — just name them.\n\n"
        f"Plan you produced:\n{plan_text[:1500]}\n\n"
        "## Tests that should currently FAIL\n(one test path per line; "
        "format: tests/path/test_x.py::TestClass::test_method)\n\n"
        "## Tests that should currently PASS (regression guard)\n"
        "(list any tests you DO NOT want to break)"
    )


def _stage_executing(task: dict, plan_text: str) -> str:
    return (
        "Now produce the actual unified diff patch fixing the issue. Use git "
        "format: ```diff\\ndiff --git a/<path> b/<path>\\n--- a/<path>\\n+++ "
        "b/<path>\\n@@ -<l>,<n> +<l>,<n> @@\\n<context>\\n-<old>\\n+<new>\\n"
        "<context>\\n``` -- multiple files OK. Edit ONLY the files you listed "
        "in the plan unless absolutely necessary.\n\n"
        f"Issue:\n{task['problem_statement']}\n\n"
        f"Plan:\n{plan_text[:1500]}\n\n"
        "Output ONLY the diff fence, no surrounding prose."
    )


def _stage_auditing(task: dict, patch_text: str) -> str:
    return (
        "You are a SENIOR CODE REVIEWER. Review the proposed patch below. "
        "For each finding emit either BLOCK / WARN / INFO with a single-line "
        "reason. After listing findings, emit a final verdict: PASS or BLOCK.\n\n"
        f"Issue summary:\n{task['problem_statement'][:1000]}\n\n"
        f"Proposed patch:\n{patch_text[:6000]}\n\n"
        "Output format:\n"
        "## Findings\n"
        "- BLOCK: <reason>\n"
        "- WARN: <reason>\n"
        "- INFO: <reason>\n\n"
        "## Verdict\nPASS or BLOCK"
    )


# ------------------------------------------------------------------
# Patch parsing + scoring (shared with v1)
# ------------------------------------------------------------------

PATCH_FENCE_RE = re.compile(r"```(?:diff|patch)?\s*\n(.*?)```", re.DOTALL)
DIFF_FILE_RE = re.compile(r"^diff --git a/(.+?) b/.+$", re.MULTILINE)
HUNK_HEADER_RE = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@", re.MULTILINE)
FUNC_NAME_RE = re.compile(r"^[+-]\s*(?:def|fn|function|class)\s+([A-Za-z_][A-Za-z0-9_]*)", re.MULTILINE)


def extract_diff_block(text: str) -> str:
    m = PATCH_FENCE_RE.search(text)
    if m:
        return m.group(1).strip()
    if "diff --git" in text:
        idx = text.find("diff --git")
        return text[idx:].strip()
    return ""


def files_in_patch(patch: str) -> set[str]:
    return set(DIFF_FILE_RE.findall(patch))


def hunks_in_patch(patch: str) -> list[tuple[str, int, int]]:
    files = []
    current_file = None
    out = []
    for line in patch.splitlines():
        m_file = re.match(r"^diff --git a/(.+?) b/.+$", line)
        if m_file:
            current_file = m_file.group(1)
            files.append(current_file)
            continue
        m_hunk = re.match(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@", line)
        if m_hunk and current_file:
            start = int(m_hunk.group(3))
            count = int(m_hunk.group(4) or 1)
            out.append((current_file, start, count))
    return out


def funcs_in_patch(patch: str) -> set[str]:
    return set(FUNC_NAME_RE.findall(patch))


def score_patch(model_patch: str, gold_patch: str) -> dict:
    if not model_patch:
        return {
            "files_overlap": 0.0,
            "hunks_overlap": 0.0,
            "funcs_overlap": 0.0,
            "composite": 0.0,
            "patch_present": False,
        }
    m_files = files_in_patch(model_patch)
    g_files = files_in_patch(gold_patch)
    files_overlap = (len(m_files & g_files) / max(len(g_files), 1)) if g_files else 0.0

    m_funcs = funcs_in_patch(model_patch)
    g_funcs = funcs_in_patch(gold_patch)
    funcs_overlap = (len(m_funcs & g_funcs) / max(len(g_funcs), 1)) if g_funcs else 0.0

    m_hunks = hunks_in_patch(model_patch)
    g_hunks = hunks_in_patch(gold_patch)
    hunk_score = 0.0
    if g_hunks:
        matched = 0
        for gf, gs, gc in g_hunks:
            ge = gs + gc
            for mf, ms, mc in m_hunks:
                if mf != gf:
                    continue
                me = ms + mc
                if max(gs, ms) < min(ge, me):
                    matched += 1
                    break
        hunk_score = matched / max(len(g_hunks), 1)

    composite = 0.5 * files_overlap + 0.2 * hunk_score + 0.3 * funcs_overlap
    return {
        "files_overlap": round(files_overlap, 3),
        "hunks_overlap": round(hunk_score, 3),
        "funcs_overlap": round(funcs_overlap, 3),
        "composite": round(composite, 3),
        "patch_present": True,
        "model_files_count": len(m_files),
        "gold_files_count": len(g_files),
        "model_hunks_count": len(m_hunks),
        "gold_hunks_count": len(g_hunks),
    }


def audit_verdict(audit_text: str) -> dict:
    block_lines = re.findall(r"^\s*[-*]?\s*BLOCK:\s*(.+)$", audit_text, re.MULTILINE | re.IGNORECASE)
    warn_lines = re.findall(r"^\s*[-*]?\s*WARN:\s*(.+)$", audit_text, re.MULTILINE | re.IGNORECASE)
    info_lines = re.findall(r"^\s*[-*]?\s*INFO:\s*(.+)$", audit_text, re.MULTILINE | re.IGNORECASE)
    verdict = "BLOCK" if re.search(r"^\s*##?\s*Verdict\s*\n\s*BLOCK", audit_text, re.MULTILINE | re.IGNORECASE) else "PASS"
    if not re.search(r"^\s*##?\s*Verdict", audit_text, re.MULTILINE | re.IGNORECASE):
        verdict = "BLOCK" if block_lines else "PASS"
    return {
        "verdict": verdict,
        "blocks": len(block_lines),
        "warns": len(warn_lines),
        "infos": len(info_lines),
    }


# ------------------------------------------------------------------
# (C) UNI-CR REFUSAL GATE
# ------------------------------------------------------------------


def unicr_check(stage_executing: dict, patch: str, tau: float, entropy_max: float = 1.5) -> tuple[bool, str]:
    """Return (should_refuse, reason)."""
    if not patch.strip():
        return True, "empty_patch"
    mean_lp = stage_executing.get("gg_mean_logprob")
    ent = stage_executing.get("gg_entropy_mean")
    if mean_lp is not None and mean_lp < tau:
        return True, f"mean_logprob={mean_lp:.3f} < tau={tau:.3f}"
    if ent is not None and ent > entropy_max:
        return True, f"entropy_mean={ent:.3f} > {entropy_max:.3f}"
    return False, "proceeded"


# ------------------------------------------------------------------
# Task runner
# ------------------------------------------------------------------


def run_one_task(
    task: dict,
    repos_dir: Path,
    do_revision: bool,
    skip_grounding: bool,
    unicr_tau: float,
    top_n: int,
) -> dict:
    record: dict[str, Any] = {
        "instance_id": task["instance_id"],
        "repo": task["repo"],
        "base_commit": task.get("base_commit", ""),
        "ts_unix": int(time.time()),
        "stages": {},
        "abstained": False,
        "unicr_verdict": "PROCEEDED",
    }

    # (A) GROUNDING RETRIEVAL
    repo_root: Path | None = None
    retrieved: list[tuple[str, float]] = []
    keywords: list[str] = extract_keywords(task["problem_statement"], top_n=20)
    grounding_info: dict[str, Any] = {
        "skipped": skip_grounding,
        "keywords": keywords,
        "retrieved_files": [],
        "retrieved_scores": [],
        "gold_files_in_retrieved": 0,
        "gold_files_in_retrieved_pct": 0.0,
        "clone_status": "not_attempted",
    }
    if not skip_grounding:
        repo_root, status = clone_repo(task["repo"], task["base_commit"], repos_dir)
        grounding_info["clone_status"] = status
        if repo_root is not None and repo_root.exists():
            try:
                retrieved = retrieve_candidates(repo_root, keywords, top_n=top_n)
            except Exception as exc:
                grounding_info["retrieval_error"] = str(exc)[:300]
                retrieved = []
            grounding_info["retrieved_files"] = [p for (p, _) in retrieved]
            grounding_info["retrieved_scores"] = [round(s, 3) for (_, s) in retrieved]
            gold_files = files_in_patch(task["gold_patch"])
            retrieved_set = set(grounding_info["retrieved_files"])
            hit = len(gold_files & retrieved_set)
            grounding_info["gold_files_in_retrieved"] = hit
            grounding_info["gold_files_in_retrieved_pct"] = round(
                hit / max(len(gold_files), 1), 3
            )
    record["grounding"] = grounding_info

    # Stage 1 PLANNING (with retrieved-files injection)
    s1 = call_sidecar(_stage_planning(task, retrieved, repo_root, keywords), max_tokens=700)
    record["stages"]["planning"] = s1

    # Stage 2 TESTING_FIRST
    s2 = call_sidecar(_stage_testing_first(task, s1["text"]), max_tokens=500)
    record["stages"]["testing_first"] = s2

    # Stage 3 EXECUTING
    s3 = call_sidecar(_stage_executing(task, s1["text"]), max_tokens=2500)
    patch_v1 = extract_diff_block(s3["text"])
    record["stages"]["executing"] = s3
    record["patch_v1"] = patch_v1

    # (B) UNI-CR REFUSAL GATE — before auditing
    should_refuse, reason = unicr_check(s3, patch_v1, tau=unicr_tau)
    record["unicr"] = {
        "tau": unicr_tau,
        "mean_logprob": s3.get("gg_mean_logprob"),
        "entropy_mean": s3.get("gg_entropy_mean"),
        "should_refuse": should_refuse,
        "reason": reason,
    }
    if should_refuse:
        record["unicr_verdict"] = "UNICR_REFUSED"
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

    # Stage 4 AUDITING (v1)
    s4 = call_sidecar(_stage_auditing(task, patch_v1), max_tokens=600)
    v1 = audit_verdict(s4["text"])
    record["stages"]["auditing_v1"] = {**s4, **v1}

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


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--n-tasks", type=int, default=5)
    p.add_argument("--out", default="/root/swe_bench_pro_elai_v2.jsonl")
    p.add_argument("--aggregate", default="/root/swe_bench_pro_elai_v2.agg.json")
    p.add_argument("--no-revision", action="store_true")
    p.add_argument("--unicr-tau", type=float, default=-0.55)
    p.add_argument("--skip-grounding", action="store_true")
    p.add_argument("--repos-dir", default="/root/swe_eval_repos")
    p.add_argument("--top-n", type=int, default=8)
    args = p.parse_args()

    if os.environ.get("ELAI_LIVE_DISPATCH", "") != "1":
        sys.stderr.write("ELAI_LIVE_DISPATCH=1 required.\n")
        return 2

    repos_dir = Path(args.repos_dir)

    print(f"[corpus] loading {args.n_tasks} tasks from {DATASET_NAME} split={DATASET_SPLIT}...", flush=True)
    tasks = load_corpus(args.n_tasks)
    print(f"[corpus] loaded {len(tasks)} unique-repo tasks", flush=True)
    print(
        f"[config] unicr_tau={args.unicr_tau} skip_grounding={args.skip_grounding} "
        f"top_n={args.top_n} repos_dir={repos_dir}",
        flush=True,
    )

    all_records: list[dict] = []
    with open(args.out, "w", encoding="utf-8") as f:
        for i, task in enumerate(tasks):
            print(f"[task {i+1}/{len(tasks)}] {task['instance_id']} ({task['repo']})", flush=True)
            try:
                rec = run_one_task(
                    task,
                    repos_dir=repos_dir,
                    do_revision=(not args.no_revision),
                    skip_grounding=args.skip_grounding,
                    unicr_tau=args.unicr_tau,
                    top_n=args.top_n,
                )
                f.write(json.dumps(rec) + "\n")
                f.flush()
                all_records.append(rec)
                sc = rec["score"]
                g = rec["grounding"]
                if rec["abstained"]:
                    print(
                        f"  UNICR_REFUSED reason={rec['unicr']['reason']} "
                        f"recall@{args.top_n}={g['gold_files_in_retrieved_pct']} "
                        f"wall={rec['totals']['wall_s']}s",
                        flush=True,
                    )
                else:
                    print(
                        f"  composite={sc['composite']} files={sc['files_overlap']} "
                        f"hunks={sc['hunks_overlap']} funcs={sc['funcs_overlap']} "
                        f"recall@{args.top_n}={g['gold_files_in_retrieved_pct']} "
                        f"wall={rec['totals']['wall_s']}s",
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
        proceeded = [r for r in all_records if not r.get("abstained")]
        mean_comp_proc = (
            round(sum(r["score"]["composite"] for r in proceeded) / len(proceeded), 3)
            if proceeded else 0.0
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
            "mean_gold_recall_at_8": round(sum(recalls) / len(recalls), 3),
            "refusal_rate": round(abstained / len(all_records), 3),
            "n_proceeded": len(proceeded),
            "mean_composite_among_proceeded": mean_comp_proc,
            "unicr_tau": args.unicr_tau,
            "skip_grounding": args.skip_grounding,
            "top_n": args.top_n,
            "model": MODEL,
            "dataset": DATASET_NAME,
            "dataset_split": DATASET_SPLIT,
            "ts_unix": int(time.time()),
        }
        with open(args.aggregate, "w", encoding="utf-8") as f:
            json.dump(agg, f, indent=2)
        print("[aggregate]", json.dumps(agg, indent=2), flush=True)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
