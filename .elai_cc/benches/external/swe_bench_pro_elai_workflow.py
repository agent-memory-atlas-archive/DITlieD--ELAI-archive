"""
ELAI full-workflow eval against REAL SWE-Bench Pro tasks (B-PLAN-EVAL).

Goal (per HANDOFF-2026-05-30-ELAI-VS-CLAUDE-CODE-EVAL.md section 6.2):
  Empirically compare ELAI-with-full-workflow against Claude-Code on the SAME
  SWE-Bench Pro tasks. This script runs the ELAI side; the user runs Claude
  Code separately on the same task_ids for direct comparison.

Pipeline per task (5 stages, simulating ELAI's PLANNING -> TESTING_FIRST ->
EXECUTING -> AUDITING -> MEMORIZING FSM without booting the full Rust harness):

  Stage 1 PLANNING       model produces a structured plan + identifies files
                         to edit + risk-rates the fix
  Stage 2 TESTING_FIRST  model lists the tests that should fail before and
                         pass after (it does not run them; this is signal,
                         not gate)
  Stage 3 EXECUTING      model produces a unified diff patch
  Stage 4 AUDITING       model self-critiques the patch; tags BLOCK/WARN/PASS
                         findings; one revision cycle if BLOCK
  Stage 5 SCORING        structural similarity to gold patch (no test exec)
                          - files_touched_overlap        = |M ∩ G| / |G|
                          - hunks_overlap_min            = min line-range
                                                            overlap across
                                                            paired hunks
                          - functions_touched_overlap    = lexical match of
                                                            function names
                                                            in the diff
                          - composite_score              = simple weighted mean

CLAIM: structural scoring is NOT semantic test verdict. It tells us "is the
model touching the right places with the right shape" but NOT "does it pass."
The user wants full test-execution; that's out of scope for this session
(needs per-repo Python envs + pytest runs).

Cost capture per task:
  - wall_clock_seconds per stage
  - sidecar_tokens_in / sidecar_tokens_out per stage
  - gg_signals per stage (mean_logprob + entropy_mean + min_logprob)
  - audit_verdict (BLOCK / WARN / PASS) per stage

Output: JSONL one line per task to --out, plus aggregate JSON to --aggregate.

Run on pod (preferred, avoids tunnel latency):
    ELAI_LIVE_DISPATCH=1 python swe_bench_pro_elai_workflow.py \
        --n-tasks 5 \
        --out /root/swe_bench_pro_elai_full_workflow.jsonl \
        --aggregate /root/swe_bench_pro_elai_full_workflow.aggregate.json
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time
from typing import Any

import httpx


SIDECAR_URL = os.environ.get("ANTHROPIC_BASE_URL", "http://localhost:8100") + "/v1/messages"
API_KEY = os.environ.get("ANTHROPIC_API_KEY", "local-dummy")
MODEL = "qwen"
DATASET_NAME = os.environ.get("SWE_BENCH_PRO_DATASET", "ScaleAI/SWE-bench_Pro")
DATASET_SPLIT = os.environ.get("SWE_BENCH_PRO_SPLIT", "test")


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


def _stage_planning(task: dict) -> str:
    return (
        "You are an experienced software engineer. Read this GitHub issue and "
        "produce a STRUCTURED PLAN (no patch yet).\n\n"
        f"Repo: {task['repo']}\n"
        f"Instance: {task['instance_id']}\n\n"
        f"Issue:\n{task['problem_statement']}\n\n"
        "Output sections (markdown):\n"
        "## Files to edit\n(list relative paths, one per line, only files you "
        "would modify)\n\n"
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
    """List of (file, line_start, line_count) for + side."""
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
        return {"files_overlap": 0.0, "hunks_overlap": 0.0, "funcs_overlap": 0.0, "composite": 0.0, "patch_present": False}
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


def run_one_task(task: dict, do_revision: bool = True) -> dict:
    """Run the 5-stage ELAI workflow on one SWE-Bench Pro task."""
    record: dict[str, Any] = {
        "instance_id": task["instance_id"],
        "repo": task["repo"],
        "ts_unix": int(time.time()),
        "stages": {},
    }

    # Stage 1 PLANNING
    s1 = call_sidecar(_stage_planning(task), max_tokens=600)
    record["stages"]["planning"] = s1

    # Stage 2 TESTING_FIRST
    s2 = call_sidecar(_stage_testing_first(task, s1["text"]), max_tokens=500)
    record["stages"]["testing_first"] = s2

    # Stage 3 EXECUTING
    s3 = call_sidecar(_stage_executing(task, s1["text"]), max_tokens=2500)
    patch_v1 = extract_diff_block(s3["text"])
    record["stages"]["executing"] = s3
    record["patch_v1"] = patch_v1

    # Stage 4 AUDITING (v1)
    s4 = call_sidecar(_stage_auditing(task, patch_v1), max_tokens=600)
    v1 = audit_verdict(s4["text"])
    record["stages"]["auditing_v1"] = {**s4, **v1}

    final_patch = patch_v1
    if do_revision and v1["verdict"] == "BLOCK":
        # one revision cycle: model rewrites patch given audit
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

    # Stage 5 SCORING
    score = score_patch(final_patch, task["gold_patch"])
    record["score"] = score
    record["final_patch"] = final_patch[:3000]
    record["gold_patch_preview"] = task["gold_patch"][:600]

    # Aggregates
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
    p.add_argument("--out", default="/root/swe_bench_pro_elai_full_workflow.jsonl")
    p.add_argument("--aggregate", default="/root/swe_bench_pro_elai_full_workflow.aggregate.json")
    p.add_argument("--no-revision", action="store_true")
    args = p.parse_args()

    if os.environ.get("ELAI_LIVE_DISPATCH", "") != "1":
        sys.stderr.write("ELAI_LIVE_DISPATCH=1 required.\n")
        return 2

    print(f"[corpus] loading {args.n_tasks} tasks from {DATASET_NAME} split={DATASET_SPLIT}...", flush=True)
    tasks = load_corpus(args.n_tasks)
    print(f"[corpus] loaded {len(tasks)} unique-repo tasks", flush=True)

    all_records = []
    with open(args.out, "w", encoding="utf-8") as f:
        for i, task in enumerate(tasks):
            print(f"[task {i+1}/{len(tasks)}] {task['instance_id']} ({task['repo']})", flush=True)
            try:
                rec = run_one_task(task, do_revision=(not args.no_revision))
                f.write(json.dumps(rec) + "\n")
                f.flush()
                all_records.append(rec)
                sc = rec["score"]
                print(f"  composite={sc['composite']} files={sc['files_overlap']} hunks={sc['hunks_overlap']} funcs={sc['funcs_overlap']} wall={rec['totals']['wall_s']}s", flush=True)
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
        block_v1 = sum(1 for r in all_records if r["stages"].get("auditing_v1", {}).get("verdict") == "BLOCK")
        revised = sum(1 for r in all_records if "patch_v2" in r)
        agg = {
            "n_tasks": len(all_records),
            "n_completed": len(all_records),
            "mean_composite": round(sum(comps)/len(comps), 3),
            "max_composite": max(comps),
            "mean_files_overlap": round(sum(r["score"]["files_overlap"] for r in all_records)/len(all_records), 3),
            "mean_funcs_overlap": round(sum(r["score"]["funcs_overlap"] for r in all_records)/len(all_records), 3),
            "mean_wall_s": round(sum(walls)/len(walls), 2),
            "mean_tokens_in": int(sum(tok_in)/len(tok_in)),
            "mean_tokens_out": int(sum(tok_out)/len(tok_out)),
            "audit_block_rate_v1": round(block_v1/len(all_records), 3),
            "revision_rate": round(revised/len(all_records), 3),
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
