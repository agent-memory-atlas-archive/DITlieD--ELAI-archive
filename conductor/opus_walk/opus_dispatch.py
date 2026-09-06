#!/usr/bin/env python3
"""opus_dispatch.py - canvas-rich single-worker dispatch primitive for the opus-orchestrator walk.

The teikoku-resident Opus (an opus_supervisor turn) calls THIS to dispatch ONE free bridge worker
(glm-5-2 for implement/fix, kimi-k2-7 for read/explore) and have it show up as a LIVE NODE on the
SIGNAL process canvas. It wraps the shipped WorkerAdapter.dispatch (worktree + guards + telemetry +
diff capture -- no scheduler reimplementation) and adds the canvas node: a tasks row (plan_id
teikoku-opus-walk, so the scope=mine /orchestration filter KEEPS it) upserted RUNNING before the run
and closed DONE/FAILED after, plus OPUS_WORKER_* lifecycle events on control.db.

Why this exists: the ported F: skill's dispatch primitive (workflow/glm-orchestrator/glm_dispatch.py
+ claudedevin.ps1 + a Windows-side multi-port 8811/8814 scheme) is Windows-native and unreachable
from the teikoku WSL distro. Grounded 2026-07-06: the teikoku bridge serves BOTH glm-5-2 and
kimi-k2-7 on the SINGLE :8810 lane, so a worker = a devin-free walk slot (conductor.elai-walk.toml,
proxy_base_url http://127.0.0.1:8810) with a pinned_model override to the chosen model
(lane_serves_model(devin-free, glm-5-2|kimi-k2-7) == True, verified).

RUNNING (not READY) is deliberate: a concurrent conductor scheduler only leases READY tasks, so this
coordination row can never be double-dispatched to another worker.

STATUS-line contract: the LAST stdout line is `STATUS: <hint>` where hint is the WorkerAdapter
worker_status_hint (APPLIED | NO_CHANGE | FAILED | RATE_LIMITED | BLOCKED | NEEDS_REVIEW), preceded
by a JSON blob {status, task_id, model, role, touched, diff, notes, artifacts}. The Opus greps the
final line for the outcome class and reads the blob for the evidence refs. This dispatch NEVER
commits or pushes -- the Opus's S10 committer owns that downstream.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import time
import uuid
from pathlib import Path

_HERE = Path(__file__).resolve().parent            # ~/teikoku/conductor/opus_walk
CONDUCTOR = _HERE.parent                            # ~/teikoku/conductor
sys.path.insert(0, str(CONDUCTOR))
sys.path.insert(0, str(_HERE))

import events                                       # noqa: E402  (canvas + audit emit)
from conductor_config import ConductorConfig        # noqa: E402
from conductor_db import ControlDB                  # noqa: E402
from models import TaskSpec, stable_idem_key, ROLES  # noqa: E402
from worker_adapter import WorkerAdapter            # noqa: E402
import disk_hygiene                                  # noqa: E402
import guard                                          # noqa: E402

# The FREE devin-free walk slots (:8810 bridge) the worker rides, with pinned_model override.
WALK_CONFIG = CONDUCTOR / "config" / "conductor.elai-walk.toml"
DB_PATH = CONDUCTOR / "state" / "control.db"
# Non-backlog plan_id so the scope=mine /orchestration filter keeps the node on the canvas.
CANVAS_PLAN_ID = "teikoku-opus-walk"
# FREE on Devin bridge ONLY (owner 2026-08-04, free_roster.json live):
#   glm-5-2, swe-1-7, swe-1-7-medium, swe-check
# NOT free / NEVER treat as free-bridge workers:
#   grok-4-5-high / any Grok 4.5 (paid on Devin AND on official Grok)
#   posthog-gpt56-* (PostHog credits via zenov, not the free bridge)
# kimi-k2-7 stays DISABLED: still ACCEPTED on CLI then coerced to glm-5-2.
FREE_BRIDGE_MODELS = ("glm-5-2", "swe-1-7", "swe-1-7-medium", "swe-check")
# Back-compat name used by preflight callers — free bridge only.
BRIDGE_MODELS = FREE_BRIDGE_MODELS
# Paid-on-bridge ids that must never be selected as FREE workers.
PAID_BRIDGE_MODELS = ("grok-4-5-high",)
# Ordinary worker route for every boss-dispatched worker lane. Specialized design,
# codex, and independent Terra verifier paths keep their own safety contracts.
ORDINARY_WORKER_MODEL = "cline-glm-53-flash"
ORDINARY_WORKER_EFFORT = "max"
WORKER_CONCURRENCY_CAP = 6
RATE_LIMITED_WORKER_CONCURRENCY_CAP = 2

# PostHog-credit workers via Grok CLI / zenov (account pool, not free Devin).
POSTHOG_WORKER_MODELS = (
    ORDINARY_WORKER_MODEL,
    "posthog-gpt56-luna",
    "posthog-gpt56-terra",
    "posthog-gpt56-sol",
    # Legacy ordinary-worker ids remain accepted at the boundary and normalize
    # to cline-glm-53-flash.
    "cline-deepseek",
)
# SEAM-DESIGN / escalation-decision authors (owner 2026-08-06). The walk-state escalation_queue
# (93 units, mostly "no designed seam") needs DESIGN turns from a top model, not worker retries.
# These are dispatchable on the posthog lane ONLY with a doc-only write set (every allowed_write
# ends in .md): a K-27 seam proposal / disposition doc, never product code. EXECUTE work stays
# on luna/glm per the worker model law.
POSTHOG_DESIGN_MODELS = (
    "posthog-fable-5",
    "posthog-opus-5",
)
_DISABLED_BRIDGE_MODELS = ("kimi-k2-7",)
DEFAULT_FREE_MODEL = ORDINARY_WORKER_MODEL
MAIN_FREE_MODEL = ORDINARY_WORKER_MODEL
DEFAULT_POSTHOG_WORKER_MODEL = ORDINARY_WORKER_MODEL


def effective_worker_model(_requested: str | None = None) -> str:
    """Every boss-dispatched worker uses the owner-pinned Cline GLM-5.3 Flash model."""
    return ORDINARY_WORKER_MODEL


def worker_concurrency_cap(rate_limited: bool = False) -> int:
    return RATE_LIMITED_WORKER_CONCURRENCY_CAP if rate_limited else WORKER_CONCURRENCY_CAP

PRIME_RLM_MODEL_ALIASES = {
    "cline-glm-53-flash": "cline/z-ai/glm-5.3-flash",
    "cline-deepseek": "cline/z-ai/glm-5.3-flash",
    "posthog-gpt56-luna-fast": "posthog-bridge/gpt-5.6-luna-fast",
    "posthog-gpt56-luna": "posthog-bridge/gpt-5.6-luna-fast",
}
PRIME_RLM_DEFAULT_MODEL = ORDINARY_WORKER_MODEL
PRIME_RLM_PROVIDER = "zenov"
PRIME_RLM_BASE_URL = "http://127.0.0.1:9020/v1"
PRIME_RLM_CLI = _HERE / "prime-agent-dist" / "cli.js"
PRIME_RLM_KERNEL_PYTHON = _HERE / "prime-runtime-venv" / "bin" / "python"
PRIME_RLM_AUTONOMOUS_GATE = "git diff --check"
PRIME_RLM_READ_ONLY_HARNESS_ROOT = Path("/usr/share")
PRIME_RLM_ZENOV_ENV_FILE = Path("/home/researcher/teikoku/Ougi/zenov/.env")

# worker_status_hint values that count as a clean finish (else the node closes FAILED).
DONE_HINTS = ("APPLIED", "NO_CHANGE")
DEPOT_SHIM_DIR = _HERE / "depot-shim"
# Default hard per-dispatch ceiling (2026-07-07 worker-hang fix). A worker slice that legitimately
# needs longer than this is a DECOMPOSITION failure (SKILL CONTEXT DISCIPLINE), not a reason to wait.
# Kept SHORTER than worker_adapter.subprocess_timeout (1800s) AND shorter than any outer shell
# `timeout` the Opus might improvise, so WorkerAdapter's OWN group-killing timeout fires first and
# NEVER orphans the base-cc child. Recon can pass a lower --timeout; implement rarely needs more.
DEFAULT_DISPATCH_TIMEOUT_S = 600

# The ELAI repo (mounted) + its CANONICAL agent roster. The `--role` above is the conductor's
# coarse worker-role/wave tier (wave ordering, slot affinity, permissions, thinking budget) -- it is
# NOT the agent roster. The 39 real ELAI agents (auditor, context-compiler, integration-explorer-elai,
# wiring-contract-validator, impact-mapper, spec-grounder, fixer, e2e-tester, ...) live here, and a
# worker EMBODIES one via --agent: the dispatch prepends the canonical ELAI agent-dispatch pointer so
# the worker reads that agent's .md + the binding preamble. Absolute paths (not worktree-relative):
# the worker reads them straight off the mount, and the F: repo gitignores .claude so a fresh worktree
# checkout would NOT contain them.
ELAI_REPO = Path("/workspace/ELAI")
ELAI_AGENTS_DIR = ELAI_REPO / ".claude" / "agents"
ELAI_PREAMBLE = ELAI_REPO / ".claude" / "rules" / "agent-dispatch-preamble.md"

# F-2 L1: the ELAI design overview, inlined into the codex stdin prompt. codex runs in a scoped mount
# (redteam B2/B3) that cannot read /mnt/f/docs, so a pointer to this path would be unreachable -- the
# CONDUCTOR (full /mnt/f visible) reads it at compose time and inlines the text. gpt-5.6-sol has a 1M
# window, so the cap is a safety valve against a runaway overview, not a tight budget.
ELAI_OVERVIEW = ELAI_REPO / "docs" / "thefinalglance" / "ELAI-SPEC-OVERVIEW.md"
CODEX_OVERVIEW_MAX_CHARS = 60_000
_CODEX_OVERVIEW_TRUNC_MARK = ("\n\n[... ELAI-SPEC-OVERVIEW.md truncated at {n} chars for the codex "
                              "prompt; read the per-letter SUMMARY digest for the rest ...]\n")

# A `Read <path> FIRST and obey it as binding` / `... follow it strictly` pointer references a
# /mnt/f/.claude path codex's scoped mount cannot open (observed CODEX-DEMO-1 2026-07-11: codex BLOCKs
# "preamble unavailable" on it, wasting a paid call). Such lines are STRIPPED from any incoming codex
# objective and the binding content is INLINED instead (F-4).
_UNREACHABLE_POINTER_RE = re.compile(
    r"^[ \t]*Read[ \t]+.+?[ \t]+FIRST[ \t]+and[ \t]+(?:obey[ \t]+it[ \t]+as[ \t]+binding|"
    r"follow[ \t]+it[ \t]+strictly).*$",
    re.IGNORECASE | re.MULTILINE,
)
# Collapse 3+ blank lines a stripped pointer leaves behind back to a single blank line.
_MULTI_BLANK_RE = re.compile(r"\n{3,}")


def _strip_unreachable_pointers(objective: str) -> str:
    """Drop every `Read <path> FIRST and obey/follow ...` pointer line from a codex-bound objective.
    codex's scoped mount cannot open /mnt/f/.claude, so such a pointer makes it BLOCK on a missing
    binding file; the binding content is inlined by _compose_codex_objective instead (F-4)."""
    cleaned = _UNREACHABLE_POINTER_RE.sub("", objective)
    return _MULTI_BLANK_RE.sub("\n\n", cleaned).lstrip("\n")


def _cap_overview(text: str, max_chars: int = CODEX_OVERVIEW_MAX_CHARS) -> str:
    """Cap the inlined overview at max_chars with an explicit truncation marker (F-2 L1)."""
    if len(text) <= max_chars:
        return text
    return text[:max_chars] + _CODEX_OVERVIEW_TRUNC_MARK.format(n=max_chars)


def _load_codex_overview() -> str:
    """Read + cap the ELAI design overview teikoku-side for the codex L1 inline. Fail-soft: an
    unreadable overview logs a WARN and returns '' so the dispatch proceeds without L1 rather than
    crashing (F-2 L1 read-at-compose-time contract)."""
    try:
        text = ELAI_OVERVIEW.read_text(encoding="utf-8")
    except OSError as exc:  # noqa: BLE001 handled below -- fail-soft, never crash the dispatch
        print(f"[opus-dispatch] WARN L1 overview unreadable ({ELAI_OVERVIEW}): "
              f"{type(exc).__name__}: {exc}; proceeding without it", file=sys.stderr, flush=True)
        return ""
    return _cap_overview(text)


def _compose_agent_objective(agent: str, objective: str, evidence_file: str | None = None) -> str:
    """Prepend the canonical ELAI agent-dispatch pointer so the worker embodies `agent`. Fail-closed:
    an agent whose .md does not exist on disk is a hard error, never a silent generic-role fallback
    (anti-slopsquat: do not dispatch a worker to read a non-existent agent).

    When `evidence_file` is given, also fold in the AGENT-TO-AGENT FILE-HANDOFF directive (owner
    leverage 2026-07-06): the orchestrator's context is the scarce resource, so the worker writes its
    FULL product to that file for the NEXT agent to read directly, and returns to the orchestrator
    only a short STATUS + the path -- never its full output dumped into the reply."""
    agent_md = ELAI_AGENTS_DIR / f"{agent}.md"
    if not agent_md.is_file():
        raise ValueError(f"unknown ELAI agent {agent!r}: {agent_md} not found (roster is "
                         f"{ELAI_AGENTS_DIR})")
    handoff = ""
    if evidence_file:
        handoff = (
            f"CONTEXT DISCIPLINE (binding): the orchestrator context is scarce and must not hold your "
            f"full output. Write your COMPLETE work product (context package / findings / review / "
            f"the diff rationale -- whatever your agent role produces) to {evidence_file} for the NEXT "
            f"agent to read directly. Return to the orchestrator ONLY a 2-4 line STATUS summary + the "
            f"path {evidence_file} + any one BLOCKED question. Never paste your full output into the "
            f"reply.\n\n"
            f"CODEX-PREP (F-2 L2/L3, when this evidence file feeds a codex EXECUTE unit whose scoped "
            f"mount CANNOT read the ELAI repo root): you run on the FREE lane with full repo access, "
            f"so PRE-DIGEST into {evidence_file} the design context codex cannot open -- the relevant "
            f"per-letter SUMMARY_INDEX slice (docs/thefinalglance/{{B..Q}}-plan/*SUMMARY*.md for the "
            f"plan's letter) AND the original plan file named in your task -- distilled to the sections "
            f"that bind THIS unit (the seam, the production hook, the contract, the acceptance), so "
            f"codex reads L2/L3 from inside its worktree instead of a path it cannot reach. Skip this "
            f"clause when the consumer is a FREE-lane agent that can read the repo itself.\n\n"
        )
    pointer = (
        f"Read {ELAI_PREAMBLE} FIRST and obey it as binding.\n"
        f"Read {agent_md} FIRST and follow it strictly as your binding role for this task.\n\n"
        f"BUILD LAW (owner 2026-08-06, TEIKOKU_DEPOT_OFFLOAD=remote): all cargo runs on Depot, "
        f"never on this PC. Invoke cargo ONLY as plain `cargo ...` so PATH resolves the depot shim. "
        f"BANNED: absolute-path cargo/rustc (~/.rustup/toolchains/**, ~/.cargo/bin/**), `rustup run`, "
        f"setting CARGO_TARGET_DIR, or any workaround after a failed check. A watchdog KILLS every "
        f"absolute-path cargo/rustc ON SIGHT -- if your build printed `Killed`, that was the watchdog, "
        f"not memory: the SAME command retried will be killed again, every time, so NEVER retry it "
        f"(observed 2026-08-10/11: workers burned 3 kill-retry cycles each on this exact mistake). "
        f"You do not need a local check at all: opus_land's gate runs the same checks and its verdict "
        f"is the only one that counts. If the plain `cargo` shim fails, STOP and report BLOCKED with "
        f"the shim output.\n\n"
    )
    return pointer + handoff + objective


def _compose_codex_objective(agent: str | None, objective: str, evidence_file: str | None = None,
                             overview_text: str | None = None) -> str:
    """Codex EXECUTOR variant of _compose_agent_objective. Codex runs inside a SCOPED codex-vm mount
    (redteam B2/B3) that can ONLY see its own worktree, so /mnt/f/.ELAI_workflow/.claude (where the
    agent-dispatch-preamble + the agent .md live) AND /mnt/f/docs are UNREACHABLE -- a `Read <that
    path> FIRST` pointer makes codex BLOCK on a missing binding file (observed CODEX-DEMO-1
    2026-07-11), wasting a paid call. So:

      - F-4: the binding preamble is ALWAYS inlined (read teikoku-side, where /mnt/f IS reachable),
        regardless of whether an --agent is named -- a codex dispatch with agent=None still gets the
        binding contract inline instead of a pointer to a path it cannot open. The agent-role .md block
        stays conditional on `agent`. Any `Read <path> FIRST and obey/follow ...` pointer already in the
        incoming objective is STRIPPED, so no unreachable path survives in what codex sees.
      - F-2 L1: when overview_text is given, the ELAI design overview is inlined too (codex cannot read
        /mnt/f/docs), so the executor has the design context without a mount widening.

    Fail-closed on an unknown agent, same as _compose_agent_objective (anti-slopsquat)."""
    objective = _strip_unreachable_pointers(objective)
    preamble_text = ELAI_PREAMBLE.read_text(encoding="utf-8") if ELAI_PREAMBLE.is_file() else ""
    agent_block = ""
    if agent:
        agent_md = ELAI_AGENTS_DIR / f"{agent}.md"
        if not agent_md.is_file():
            raise ValueError(f"unknown ELAI agent {agent!r}: {agent_md} not found (roster is "
                             f"{ELAI_AGENTS_DIR})")
        agent_block = (f"----- your agent role: {agent} -----\n"
                       + agent_md.read_text(encoding="utf-8") + "\n")
    blocks: list[str] = [
        "BINDING RULES (INLINED): you run in a sandbox that can read ONLY your working directory, "
        "so these ARE your binding rules -- do NOT try to open any path outside the worktree "
        "(there is no .claude rules dir here); the rules below are the whole contract.\n\n"
        "----- agent-dispatch-preamble (binding) -----\n" + preamble_text + "\n"
        + agent_block +
        "----- end binding rules -----\n\n"
    ]
    if overview_text:
        blocks.append(
            "----- ELAI design overview (inlined; codex cannot read the repo docs/) -----\n"
            + overview_text +
            "\n----- end ELAI design overview -----\n\n"
        )
    if evidence_file:
        blocks.append(
            f"CONTEXT DISCIPLINE (binding): write your COMPLETE work product to {evidence_file} for the "
            f"next agent; return to the orchestrator ONLY a 2-4 line STATUS summary + that path.\n\n"
        )
    blocks.append(objective)
    return "".join(blocks)


def _make_taskspec(model, role, elai_plan, objective, allowed_writes, base_commit,
                   token_budget, ttl, model_tier="FREE") -> TaskSpec:
    """Build the canvas/worker TaskSpec with its effective model identity."""
    effective_model = effective_worker_model(model)
    safe = f"{elai_plan}-{role}-{int(time.time())}-{uuid.uuid4().hex[:6]}".replace("/", "_")
    task_id = f"opuswalk-{safe}"
    # WORKTREE SCOPE BANNER (2026-07-18, 3rd canonical-leak occurrence): free-lane workers keep
    # editing the canonical repo because plan digests cite absolute canonical paths. Make the
    # write boundary explicit in the task text itself; the leak detector in _run_dispatch is the
    # off-agent backstop.
    if model_tier == "FREE":
        objective = (
            "WRITE BOUNDARY (binding): make ALL file edits inside YOUR ASSIGNED WORKTREE (your "
            "working directory), never in the canonical repo. Any absolute path under "
            "/workspace/ELAI/ in the task below is a READ-ONLY reference; resolve it to "
            "the same relative path inside your worktree before editing. An edit to the canonical "
            "tree cannot be landed and voids the dispatch.\n\n" + objective
        )
    return TaskSpec(
        task_id=task_id,
        plan_id=CANVAS_PLAN_ID,
        wave=role,
        role=role,
        objective=objective,
        base_commit=base_commit,
        allowed_write_set=list(allowed_writes),
        context_refs=[],
        production_hook=None,
        acceptance=("The off-agent verifier + the opus controller certify DONE; this worker never "
                    "self-certifies (execution/evaluation separation)."),
        deterministic_check=None,
        forbidden=["no gate edits", "no destructive git", "no commit", "no push",
                   "no credential reads"],
        verifier=[],
        lease_ttl_seconds=ttl,
        idem_key=stable_idem_key([task_id, base_commit]),
        token_budget=token_budget,
        model_tier=model_tier,
        pinned_model=effective_model,
    )


# K-27 codex EXECUTOR lane constants.
CODEX_EXECUTOR_FLAG = "ELAI_OPUS_CODEX_EXECUTOR"   # default-OFF opt-in; the walk boot sets =1
CODEX_MODEL = "gpt-5-6-sol"                        # roster id; account-a config.toml pins the real model
CODEX_VM_DISTRO = "codex-vm"
# The codex EXECUTOR worktree lives under this base: opus_land reads/merges it conductor-side, while
# the codex sandbox gets a SCOPED mount of ONLY this dir (codex_provider._ensure_scoped_codex_mount),
# so the canonical repo + every other file is unreachable to codex (redteam B2/B3).
# "elai-codex-worktrees" is the marker codex_provider keys the scoped-mount + network-off hardening
# on. Outside the repo so the git worktree never nests inside it.
CODEX_WORKTREE_BASE = Path("/home/researcher/teikoku/conductor/elai-codex-worktrees")
try:
    from ssb_write_set import expand_allowed_writes, elai_cc_path_preamble
except ImportError:
    from opus_walk.ssb_write_set import expand_allowed_writes, elai_cc_path_preamble  # type: ignore
try:
    from pending_lands import add as pending_land_add
except ImportError:
    try:
        from opus_walk.pending_lands import add as pending_land_add  # type: ignore
    except ImportError:
        def pending_land_add(*_a, **_k):
            return None

def _maybe_keep_codex_worktree(elai_plan, worktree_path, out, task_id) -> None:
    from pathlib import Path as _P
    import subprocess as _sp
    wt = _P(worktree_path) if worktree_path else None
    if wt is None or not wt.is_dir():
        return
    dirty = False
    art = Path(__file__).resolve().parents[1] / 'artifacts' / str(task_id)
    patch = art / 'patch.diff'
    try:
        if patch.is_file() and patch.stat().st_size > 0:
            dirty = True
    except OSError:
        pass
    try:
        st = _sp.run(['git', '-C', str(wt), 'status', '--porcelain'],
                     capture_output=True, text=True, timeout=30, check=False)
        if st.returncode == 0 and (st.stdout or '').strip():
            dirty = True
    except (OSError, _sp.SubprocessError):
        pass
    if not dirty:
        return
    try:
        (wt / '.KEEP_FOR_LAND').write_text(
            f'plan={elai_plan}\ntask={task_id}\n', encoding='utf-8')
    except OSError:
        pass
    pending_land_add(elai_plan, str(wt), str(art) if art.is_dir() else '')
    print(f'[opus-dispatch] KEEP_FOR_LAND set on {wt} plan={elai_plan}', flush=True)


maybe_keep_codex_worktree = _maybe_keep_codex_worktree  # public alias (P0 land handoff)

# gpt-5.6-sol has a 1M window (OpenAI top-line): default the codex executor a large budget so the
# controller can feed it whole plans + rich context, per the owner's routing directive.
CODEX_DEFAULT_TOKEN_BUDGET = 800_000


def _ensure_user_bus_env() -> None:
    """systemd-run --user needs a session bus; headless orchestrator shells often omit it."""
    uid = os.getuid()
    runtime = os.environ.get("XDG_RUNTIME_DIR") or f"/run/user/{uid}"
    bus_path = f"{runtime}/bus"
    if not os.environ.get("XDG_RUNTIME_DIR") and os.path.isdir(runtime):
        os.environ["XDG_RUNTIME_DIR"] = runtime
    if not os.environ.get("DBUS_SESSION_BUS_ADDRESS") and os.path.exists(bus_path):
        os.environ["DBUS_SESSION_BUS_ADDRESS"] = f"unix:path={bus_path}"


def _scope_wrap(
    argv: list[str],
    tasks_max: int = 512,
    unit: str | None = None,
    env: dict[str, str] | None = None,
) -> list[str]:
    memmax = (os.environ.get("TEIKOKU_WORKER_MEMMAX") or "10G").strip()
    if memmax.lower() == "off":
        return list(argv)
    if not memmax:
        raise ValueError("worker dispatch refused: TEIKOKU_WORKER_MEMMAX is empty")
    if shutil.which("systemd-run") is None:
        raise RuntimeError(
            "worker dispatch refused: systemd-run is not on PATH; refusing an unscoped worker"
        )
    _ensure_user_bus_env()
    service_args = [
        "systemd-run",
        "--user",
        "--wait",
        "--collect",
        "--pipe",
        "--quiet",
        "--same-dir",
    ]
    if unit:
        service_args.extend(["--unit", unit])
    service_args.extend(
        [
            "-p",
            f"MemoryMax={memmax}",
            "-p",
            "MemorySwapMax=0",
            "-p",
            f"TasksMax={tasks_max}",
            *_cpu_quota_props(),
        ]
    )
    # A bare --setenv=NAME asks systemd-run to copy NAME from its caller. The
    # values stay in the Popen environment and never enter this argv list.
    service_args.extend(f"--setenv={name}" for name in sorted(env or {}))
    return service_args + list(argv)


def _cpu_quota_props() -> list[str]:
    quota = (os.environ.get("TEIKOKU_WORKER_CPUQUOTA") or "200%").strip()
    if quota.lower() == "off":
        return []
    return ["-p", f"CPUQuota={quota}"]


class _ScopedWorkerAdapter(WorkerAdapter):
    def _service_unit(self, task_id: str) -> str:
        return f"worker-{task_id}"

    def _wrap_worker_command(self, command, task, *, tasks_max: int = 512):
        self._scope_original_command = list(command)
        self._scope_requested_unit = self._service_unit(task.task_id)
        self._scope_tasks_max = tasks_max
        self._scope_command = _scope_wrap(
            self._scope_original_command,
            tasks_max=tasks_max,
            unit=self._scope_requested_unit,
        )
        self._scope_unit = (
            self._scope_requested_unit
            if self._scope_command != self._scope_original_command
            else None
        )
        return self._scope_command

    def _refresh_scope_command(self, env):
        command = getattr(self, "_scope_command", None)
        original = getattr(self, "_scope_original_command", None)
        if command is None or original is None:
            return
        command[:] = _scope_wrap(
            original,
            tasks_max=getattr(self, "_scope_tasks_max", 512),
            unit=getattr(self, "_scope_requested_unit", None),
            env=env,
        )

    def _build_worker_command(self, slot, task, taskspec_file, role_config, atif_file,
                              worktree_path):
        command, input_text = super()._build_worker_command(
            slot, task, taskspec_file, role_config, atif_file, worktree_path
        )
        self._scope_tasks_max = 512
        return self._wrap_worker_command(command, task), input_text

    def _worker_env(self, slot):
        env = super()._worker_env(slot)
        # WorkerAdapter builds the command before its final provider environment. Rebuild the
        # same list after that environment exists so every forwarded name is copied by systemd.
        self._refresh_scope_command(env)
        return env

    def _stop_service(self, task_id: str) -> None:
        unit = getattr(self, "_scope_unit", None)
        if not unit:
            return
        unit = f"{unit}.service"
        try:
            result = subprocess.run(
                ["systemctl", "--user", "stop", unit],
                capture_output=True,
                text=True,
                timeout=30,
                check=False,
            )
            if result.returncode != 0:
                raise RuntimeError((result.stderr or result.stdout or "unknown systemd stop failure").strip())
        except (OSError, subprocess.SubprocessError, RuntimeError) as exc:
            print(
                f"[opus-dispatch] WARN worker service stop failed for {unit}: "
                f"{type(exc).__name__}: {exc}",
                file=sys.stderr,
                flush=True,
            )

    def dispatch(self, taskspec, *args, **kwargs):
        try:
            return super().dispatch(taskspec, *args, **kwargs)
        finally:
            self._stop_service(taskspec.task_id)


def _prime_guard_projection(text: str) -> str:
    fragments: list[str] = []
    prime_event_prefixes = (
        '{"type":"session',
        '{"type":"agent_start',
        '{"type":"turn_start',
        '{"type":"message_start',
        '{"type":"message_end',
        '{"type":"message_update',
        '{"type":"tool_execution_start',
        '{"type":"tool_execution_update',
        '{"type":"tool_execution_end',
        '{"type":"turn_end',
        '{"type":"agent_end',
    )

    def add_assistant_message(value: object) -> None:
        if not isinstance(value, dict) or value.get("role") != "assistant":
            return
        fragments.append(json.dumps(value, ensure_ascii=False, sort_keys=True))

    for line in (text or "").splitlines():
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            if not line.lstrip().startswith(prime_event_prefixes):
                fragments.append(line)
            continue
        if not isinstance(event, dict):
            fragments.append(line)
            continue
        event_type = event.get("type")
        if event_type not in {
            "message_start",
            "message_end",
            "message_update",
            "agent_end",
            "tool_execution_start",
            "session",
            "agent_start",
            "turn_start",
            "tool_execution_update",
            "tool_execution_end",
            "turn_end",
        }:
            fragments.append(line)
            continue
        if event_type in {"message_start", "message_end"}:
            add_assistant_message(event.get("message"))
        elif event_type == "message_update":
            assistant_event = event.get("assistantMessageEvent")
            if isinstance(assistant_event, dict):
                add_assistant_message(assistant_event.get("partial"))
                add_assistant_message(assistant_event.get("message"))
        elif event_type == "agent_end":
            messages = event.get("messages")
            if isinstance(messages, list):
                for message in messages:
                    add_assistant_message(message)
        elif event_type == "tool_execution_start":
            args = event.get("args")
            if args is not None:
                fragments.append(json.dumps(args, ensure_ascii=False, sort_keys=True))
    return "\n".join(fragments)


class _PrimeRlmOutputGuard:
    def __init__(self):
        self._guard = guard.OutputGuard()

    def check(self, text: str):
        return self._guard.check(_prime_guard_projection(text))


class _PrimeRlmWorkerAdapter(_ScopedWorkerAdapter):
    def _service_unit(self, task_id: str) -> str:
        return f"prime-{task_id}"

    @classmethod
    def from_conductor_config(cls, config, provider=None, prime_model=PRIME_RLM_DEFAULT_MODEL):
        return cls(
            {
                "worktree_root": str(config.worktree_root),
                "artifact_root": str(config.artifact_root),
                "repo_root": str(config.repo_root),
                "agent_root": str(config.workflow_root / ".claude" / "agents"),
                "subprocess_timeout_seconds": config.limits.subprocess_timeout_seconds,
                "workers": list(config.workers),
                "escalation_workers": list(config.escalation_workers),
            },
            provider=provider,
            prime_model=prime_model,
        )

    def __init__(self, config=None, provider=None, prime_model=PRIME_RLM_DEFAULT_MODEL):
        super().__init__(config, provider=provider)
        self._output_guard = _PrimeRlmOutputGuard()
        self.prime_model = prime_model

    def _prime_agent_dir(self, worktree_path: Path) -> Path:
        return worktree_path / ".task" / "prime-agent"

    def _prime_session_dir(self, worktree_path: Path) -> Path:
        return worktree_path / ".task" / "prime-session"

    def _prime_harness_dir(self, worktree_path: Path) -> Path:
        return worktree_path / ".task" / "prime-harness"

    def _read_zenov_api_key(self) -> str:
        value = os.environ.get("ZENOV_API_KEY")
        if value:
            return value
        try:
            for line in PRIME_RLM_ZENOV_ENV_FILE.read_text(encoding="utf-8").splitlines():
                if line.startswith("ZENOV_MASTER_KEY="):
                    value = line.split("=", 1)[1].strip().strip('"').strip("'")
                    if value:
                        return value
        except OSError:
            pass
        raise RuntimeError("prime-rlm requires ZENOV_API_KEY or the local Zenov .env")

    def _read_only_harness_link(self, link_path: Path) -> None:
        link_path.parent.mkdir(parents=True, exist_ok=True)
        try:
            link_path.unlink()
        except FileNotFoundError:
            pass
        link_path.symlink_to(PRIME_RLM_READ_ONLY_HARNESS_ROOT, target_is_directory=True)

    def _write_prime_models(self, agent_dir: Path, harness_dir: Path) -> None:
        api_key = self._read_zenov_api_key()
        agent_dir.mkdir(parents=True, exist_ok=True)
        self._read_only_harness_link(harness_dir)
        self._read_only_harness_link(agent_dir / "harness")
        session_dir = getattr(self, "_current_session_dir", agent_dir.parent / "prime-session")
        self._read_only_harness_link(session_dir / "harness")

        (agent_dir / "settings.json").write_text(
            json.dumps(
                {
                    "autoRefine": {
                        "enabled": False,
                        "compact": False,
                        "turnInterval": 1000000,
                        "cooldownMs": 2147483647,
                    },
                    "quietStartup": True,
                },
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
        (agent_dir / "models.json").write_text(
            json.dumps(
                {
                    "providers": {
                        PRIME_RLM_PROVIDER: {
                            "api": "openai-completions",
                            "baseUrl": PRIME_RLM_BASE_URL,
                            "apiKey": api_key,
                            "models": [
                                {
                                    "id": PRIME_RLM_MODEL_ALIASES[self.prime_model],
                                    "name": "Zenov GPT-5.6 Luna FAST",
                                    "api": "openai-completions",
                                    "baseUrl": PRIME_RLM_BASE_URL,
                                    "reasoning": True,
                                    "input": ["text"],
                                    "contextWindow": 1000000,
                                    "maxTokens": 16384,
                                }
                            ],
                        }
                    }
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )

    def _build_worker_command(self, slot, task, taskspec_file, role_config, atif_file,
                              worktree_path):
        if self.prime_model not in PRIME_RLM_MODEL_ALIASES:
            raise ValueError(f"prime-rlm model is not configured: {self.prime_model!r}")
        agent_dir = self._prime_agent_dir(worktree_path)
        session_dir = self._prime_session_dir(worktree_path)
        harness_dir = self._prime_harness_dir(worktree_path)
        self._current_agent_dir = agent_dir
        self._current_session_dir = session_dir
        self._current_harness_dir = harness_dir
        self._write_prime_models(agent_dir, harness_dir)
        command = [
            "node",
            str(PRIME_RLM_CLI),
            "-p",
            "--mode",
            "json",
            "--cwd",
            str(worktree_path),
            "--session-dir",
            str(session_dir),
            "--daemon-socket",
            str(session_dir / "prime-daemon.sock"),
            "--no-session",
            "--model",
            f"{PRIME_RLM_PROVIDER}/{PRIME_RLM_MODEL_ALIASES[self.prime_model]}",
            "--thinking",
            "max",
            "--no-skills",
            "--no-extensions",
            "--no-prompt-templates",
            "--no-themes",
            "--no-context-files",
            "--autonomous",
            "--autonomous-gate",
            PRIME_RLM_AUTONOMOUS_GATE,
            "--autonomous-gate-retries",
            "1",
            "--autonomous-gate-timeout-ms",
            "60000",
        ]
        self._scope_tasks_max = 256
        return self._wrap_worker_command(command, task, tasks_max=256), taskspec_file.read_text(encoding="utf-8")

    def _worker_env(self, slot):
        env = super()._worker_env(slot)
        env.update(
            {
                "PRIME_AGENT_CODING_AGENT_DIR": str(self._current_agent_dir),
                "PRIME_AGENT_SESSION_DIR": str(self._current_session_dir),
                "RLM_HARNESS_STATE_DIR": str(self._current_harness_dir),
                "RLM_GLOBAL_HARNESS_STATE_DIR": str(self._current_harness_dir),
                "PRIME_AGENT_KERNEL_PYTHON": str(PRIME_RLM_KERNEL_PYTHON),
                "PRIME_AGENT_KERNEL_FORKSERVER": "0",
                "PRIME_AGENT_INTERNAL_LEGACY_OWNED_WORKER_FRONTEND": "1",
                "PI_OFFLINE": "1",
                "PI_SKIP_VERSION_CHECK": "1",
            }
        )
        self._refresh_scope_command(env)
        return env



    def _extract_worker_notes(self, harness, atif_path, stdout, stderr):
        final_text = None
        events_seen = []
        for line in stdout.splitlines():
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            if not isinstance(event, dict):
                continue
            events_seen.append(event.get("type"))
            if event.get("type") not in {"message_end", "agent_end"}:
                continue
            candidates = [event.get("message")]
            if event.get("type") == "agent_end":
                candidates = event.get("messages") or []
            for message in reversed(candidates):
                if not isinstance(message, dict) or message.get("role") != "assistant":
                    continue
                content = message.get("content")
                if isinstance(content, str) and content.strip():
                    final_text = content
                    break
                if isinstance(content, list):
                    text = "".join(
                        item.get("text", "")
                        for item in content
                        if isinstance(item, dict) and item.get("type") == "text"
                    )
                    if text.strip():
                        final_text = text
                        break
            if final_text:
                break
        atif_path.parent.mkdir(parents=True, exist_ok=True)
        atif_path.write_text(
            json.dumps(
                {
                    "harness": "prime-rlm",
                    "final_message": final_text or "",
                    "event_types": events_seen[-100:],
                    "stderr_tail": stderr[-2000:] if stderr else "",
                },
                indent=2,
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )
        return final_text


def _plan_worktree_prefix(elai_plan: str) -> str:
    safe_plan = re.sub(r"[^A-Za-z0-9_.-]+", "_", str(elai_plan).replace("/", "_"))
    return f"opuswalk-{safe_plan}-"


def _dispatch_worktree_bases() -> tuple[Path, ...]:
    bases = list(disk_hygiene.WORKTREE_DIRS)
    if CODEX_WORKTREE_BASE not in bases:
        bases.append(CODEX_WORKTREE_BASE)
    return tuple(bases)


def _plan_worktree_count(elai_plan: str) -> int:
    prefix = _plan_worktree_prefix(elai_plan)
    count = 0
    for base in _dispatch_worktree_bases():
        try:
            children = base.iterdir() if base.is_dir() else ()
            count += sum(1 for child in children if child.is_dir() and child.name.startswith(prefix))
        except OSError:
            continue
    return count


def _check_attempt_cap(elai_plan: str) -> None:
    try:
        cap = int(os.environ.get("TEIKOKU_MAX_PLAN_WORKTREES") or "6")
    except ValueError as exc:
        raise ValueError("TEIKOKU_MAX_PLAN_WORKTREES must be an integer") from exc
    if cap < 1:
        raise ValueError("TEIKOKU_MAX_PLAN_WORKTREES must be at least 1")
    count = _plan_worktree_count(elai_plan)
    if count >= cap:
        raise RuntimeError(
            f"attempt cap: {count} live worktrees for {elai_plan}; reap or stop, do not pile"
        )


def _is_existing_worktree(path: Path | str | None) -> bool:
    if not path:
        return False
    p = Path(path)
    return p.is_dir() and (p / ".git").exists()


def _worktree_identity(path: Path | str | None) -> tuple | None:
    if not path:
        return None
    try:
        worktree = Path(path).resolve(strict=True)
        worktree_stat = worktree.stat()
        git_marker = worktree / ".git"
        git_identity = None
        if git_marker.is_file():
            git_stat = git_marker.stat()
            git_identity = (
                "file",
                git_stat.st_dev,
                git_stat.st_ino,
                git_marker.read_text(encoding="utf-8", errors="replace"),
            )
        elif git_marker.is_dir():
            git_stat = git_marker.stat()
            git_identity = ("dir", git_stat.st_dev, git_stat.st_ino)
        return (
            worktree,
            worktree_stat.st_dev,
            worktree_stat.st_ino,
            git_identity,
        ) if git_identity is not None else None
    except (OSError, RuntimeError):
        return None


def _cleanup_target_is_dispatch_owned(
    owner: str,
    requested_path: Path | None,
    result_path: Path | str | None,
    expected_identity: tuple | None,
) -> bool:
    if owner != "dispatch-owned-fresh" or not result_path or expected_identity is None:
        return False
    current_identity = _worktree_identity(result_path)
    if current_identity != expected_identity:
        return False
    if requested_path is None:
        return True
    return requested_path == current_identity[0]


def _head_tail_notes(text: str, limit: int) -> str:
    """Bound notes to ~limit chars keeping BOTH ends (the typed verdict is at the tail)."""
    if len(text) <= limit:
        return text
    half = max(1, (limit - 20) // 2)
    return text[:half] + "\n...[middle cut]...\n" + text[-half:]


def _reap_failed_worktree(path: str | Path | None) -> None:
    if not path:
        return
    worktree = Path(path)
    if not worktree.is_dir() or disk_hygiene._has_keep_for_land(worktree):
        return
    try:
        parent = worktree.parent.resolve()
        allowed = {base.resolve() for base in _dispatch_worktree_bases()}
    except OSError:
        return
    if parent not in allowed:
        return
    disk_hygiene._safe_rmtree(worktree, parent, disk_hygiene._log, shutil.rmtree)
    disk_hygiene._run_soft(
        ["git", "-C", str(disk_hygiene.ELAI_REPO), "worktree", "prune"],
        subprocess.run,
        disk_hygiene._log,
    )


def _canonical_dirty_set() -> set:
    """Porcelain dirty paths of the CANONICAL elai repo, for the leak detector. Empty set on any
    git error (fail-open: detection must never block a dispatch)."""
    try:
        proc = subprocess.run(["git", "-C", str(ELAI_REPO), "status", "--porcelain"],
                              capture_output=True, text=True, timeout=30)
        if proc.returncode != 0:
            return set()
        return {line[3:].strip() for line in proc.stdout.splitlines() if len(line) > 3}
    except Exception:  # noqa: BLE001 - detector is advisory, never fatal
        return set()


def _prime_process_records() -> list[tuple[int, str, str]]:
    records = []
    try:
        entries = list(Path("/proc").iterdir())
    except OSError:
        return records
    for entry in entries:
        if not entry.name.isdigit():
            continue
        try:
            cmdline = " ".join(
                part.decode("utf-8", errors="replace")
                for part in (entry / "cmdline").read_bytes().split(b"\0")
                if part
            )
            if "prime-agent-dist/cli.js" not in cmdline:
                continue
            cgroup = (entry / "cgroup").read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        records.append((int(entry.name), cmdline, cgroup))
    return records


def _prime_stale_daemon_notes() -> list[str]:
    notes = []
    for pid, _cmdline, cgroup in _prime_process_records():
        if re.search(r"(?:^|/)(?:run-[^/]*\.scope|run-[^/]*\.service|prime-[^/]*\.service)(?:$|/)", cgroup):
            continue
        note = (
            f"WARNING: prime-agent-dist/cli.js pid={pid} is outside a managed service; "
            "not killed"
        )
        print(f"[opus-dispatch] {note}", file=sys.stderr, flush=True)
        notes.append(note)
    return notes


def _run_dispatch(task, elai_plan, agent, dry_run, run_timeout, worktree_path=None,
                  reap_failed=True, adapter_factory=None, pre_dispatch_notes=None) -> dict:
    """Shared canvas-node + reaping + adapter.dispatch core for BOTH the FREE bridge lane and the
    PAID codex lane. The caller builds the tier-specific TaskSpec; this runs it as a live canvas node
    and returns the structured result. worktree_path (codex only) forces the worktree onto /mnt/f."""
    model = task.pinned_model
    role = task.role
    dispatch_notes = list(pre_dispatch_notes or [])
    config = ConductorConfig.from_file(WALK_CONFIG)
    db = ControlDB(DB_PATH)

    worktree_owner = (
        "caller-owned-reused"
        if _is_existing_worktree(worktree_path)
        else "dispatch-owned-fresh"
    )
    requested_worktree_path = None
    if worktree_path is not None:
        try:
            requested_worktree_path = Path(worktree_path).resolve(strict=False)
        except (OSError, RuntimeError):
            pass
    if not dry_run:
        if worktree_owner == "dispatch-owned-fresh":
            _check_attempt_cap(elai_plan)
        _scope_wrap(["worker"])

    # canvas node up: RUNNING (never leased by a concurrent scheduler) so the SIGNAL frontier shows
    # this worker live for the duration of the dispatch.
    db.upsert_task(task, status="RUNNING")
    events.append_event(
        str(DB_PATH), "OPUS_WORKER_DISPATCHED", task_id=task.task_id,
        payload={"model": model, "role": role, "agent": agent, "elai_plan": elai_plan,
                 "tier": task.model_tier, "objective": task.objective[:300], "notes": dispatch_notes},
        project_id="elai", session_id=CANVAS_PLAN_ID,
    )

    adapter = adapter_factory(config) if adapter_factory else _ScopedWorkerAdapter.from_conductor_config(config)

    # A SIGTERM/SIGINT to this dispatch process (outer `timeout`, supervisor reap, Ctrl-C) must reap
    # the live worker's process group BEFORE we exit -- otherwise the base-cc child (start_new_session)
    # is orphaned and keeps mutating the worktree. Belt-and-suspenders with the run_timeout ceiling
    # below; best-effort, only installs in the main thread.
    prev_handlers = {}

    def _reap_and_exit(signum, _frame):
        try:
            adapter.cancel()
        except Exception:  # noqa: BLE001 - cleanup path, never mask the shutdown
            pass
        raise SystemExit(128 + signum)

    for _sig in (signal.SIGTERM, signal.SIGINT):
        try:
            prev_handlers[_sig] = signal.signal(_sig, _reap_and_exit)
        except (ValueError, OSError):
            pass  # not the main thread / unsupported: rely on the run_timeout ceiling
    pre_dirty = _canonical_dirty_set()
    try:
        # WorkerAdapter.dispatch has no run_timeout_override kwarg; the provider
        # uses adapter.subprocess_timeout (from conductor config). Pin it to the
        # per-dispatch ceiling so --timeout is honored.
        if run_timeout is not None:
            adapter.subprocess_timeout = int(run_timeout)
        result = adapter.dispatch(task, dry_run=dry_run, db_path=DB_PATH,
                                  worktree_path=worktree_path)
    finally:
        for _sig, _prev in prev_handlers.items():
            try:
                signal.signal(_sig, _prev)
            except (ValueError, OSError):
                pass
    # CANONICAL-LEAK DETECTOR (2026-07-18, after the 3rd worker bypassed worktree isolation and
    # edited the canonical repo directly, stranding un-landable product edits in a contested tree):
    # any file newly dirty in the CANONICAL repo across this dispatch is a leak. Detection only,
    # never a mutation -- the controller decides (migrate into the worktree / land scoped / redo).
    canonical_leak = sorted(_canonical_dirty_set() - pre_dirty)
    if canonical_leak:
        print(f"[opus-dispatch] CANONICAL_LEAK: worker wrote the canonical repo, not its worktree: "
              f"{canonical_leak} -- these edits are NOT in the worktree and opus_land cannot land "
              f"them; controller must migrate or re-dispatch", file=sys.stderr, flush=True)

    hint = result.worker_status_hint or "FAILED"
    failed = result.exit_code != 0 or hint not in DONE_HINTS
    final_status = "FAILED" if failed else "DONE"
    result_worktree = getattr(result, "worktree_path", None) or (
        str(worktree_path) if worktree_path else None
    )
    result_worktree_identity = _worktree_identity(result_worktree)

    # canvas node down: close the frontier row to the worker's terminal status.
    try:
        db.update_task_status(task.task_id, final_status, reason=f"opus-worker {hint}")
    except Exception as exc:  # noqa: BLE001 - a status-close failure must not lose the worker result
        print(f"[opus-dispatch] WARN status update failed for {task.task_id}: "
              f"{type(exc).__name__}: {exc}", file=sys.stderr, flush=True)
    events.append_event(
        str(DB_PATH), "OPUS_WORKER_DONE", task_id=task.task_id,
        payload={"model": result.model_used, "role": role, "hint": hint,
                 "tier": task.model_tier,
                 "touched": result.touched_files[:20], "exit_code": result.exit_code,
                 "notes": dispatch_notes},
        project_id="elai", session_id=CANVAS_PLAN_ID,
    )

    if (
        final_status == "FAILED"
        and reap_failed
        and not dry_run
        and _cleanup_target_is_dispatch_owned(
            worktree_owner,
            requested_worktree_path,
            result_worktree,
            result_worktree_identity,
        )
    ):
        _reap_failed_worktree(result_worktree)

    return {
        "status": hint,
        "final_status": final_status,
        "task_id": task.task_id,
        "model": result.model_used,
        "role": role,
        "tier": task.model_tier,
        "agent": agent,
        "elai_plan": elai_plan,
        "worktree": result_worktree,
        "canonical_leak": canonical_leak,
        "touched": result.touched_files,
        "diff": result.diff_artifact,
        # Head+tail, not head-only: the worker's typed verdict (recon law) is in its FINAL
        # message, so a [:1200] head cut amputated every verdict (N-2/N-14/O-1, 2026-08-12).
        "notes": _head_tail_notes(
            "\n".join(dispatch_notes + ([result.sanitized_notes] if result.sanitized_notes else [])),
            1200,
        ),
        "artifacts": result.artifact_refs,
        "exit_code": result.exit_code,
    }


def dispatch_worker(model, role, elai_plan, objective, allowed_writes, base_commit,
                    token_budget, ttl, dry_run=False, agent=None,
                    run_timeout=DEFAULT_DISPATCH_TIMEOUT_S, worktree_path=None) -> dict:
    """Dispatch one ordinary worker through the Luna-fast PostHog route.

    The legacy ``free`` CLI lane remains accepted for compatibility. It no longer
    selects a Devin model. It normalizes to Luna-fast/max and the PostHog slot.
    """
    model = effective_worker_model(model)
    if role not in ROLES:
        raise ValueError(f"role must be one of {sorted(ROLES)}, got {role!r}")
    if not objective or not objective.strip():
        raise ValueError("objective must be a non-empty worker task text")

    # FREE lane must expand SSB cascade the same way codex does; otherwise a
    # vertical slice that adds an SsbActionType variant lands NEEDS_REVIEW on
    # schema.rs / ssb.rs / action_type_lock.* even when those writes are
    # required for the enum exhaustiveness.
    allowed_writes = expand_allowed_writes(allowed_writes)

    task = _make_taskspec(model, role, elai_plan, objective, allowed_writes, base_commit,
                          token_budget, ttl, model_tier="PAID")
    return _run_dispatch(task, elai_plan, agent, dry_run, run_timeout, worktree_path=worktree_path)


def dispatch_posthog_worker(
    model,
    role,
    elai_plan,
    objective,
    allowed_writes,
    base_commit,
    token_budget,
    ttl,
    dry_run=False,
    agent=None,
    run_timeout=DEFAULT_DISPATCH_TIMEOUT_S,
    worktree_path=None,
) -> dict:
    """Dispatch ONE PostHog-credit worker via Grok CLI (zenov → posthog-bridge).

    Not free. Uses live PostHog account pool (e.g. posthog-gpt56-luna). Fail-closed
    if model is not in POSTHOG_WORKER_MODELS or POSTHOG_DESIGN_MODELS. Does not use
    the Devin free bridge.

    POSTHOG_DESIGN_MODELS (fable-5 / opus-5) are the seam-design authors for the
    escalation_queue: they REQUIRE a non-empty, doc-only write set (every entry ends
    in .md). A design model asked to write product code is a caller bug, fail-closed.
    """
    requested_model = model
    if requested_model not in POSTHOG_WORKER_MODELS and requested_model not in POSTHOG_DESIGN_MODELS:
        raise ValueError(
            f"posthog worker request must be one of {POSTHOG_WORKER_MODELS} "
            f"or legacy design shapes {POSTHOG_DESIGN_MODELS}, got {requested_model!r}"
        )
    is_design = requested_model in POSTHOG_DESIGN_MODELS
    if is_design:
        raw_writes = [p for p in (allowed_writes or []) if p and p.strip()]
        if not raw_writes:
            raise ValueError(
                f"design model {model} requires a non-empty doc write set "
                f"(the seam-proposal .md path); an empty set is a caller bug"
            )
        non_md = [p for p in raw_writes if not p.strip().endswith(".md")]
        if non_md:
            raise ValueError(
                f"design model {model} is doc-only: every allowed_write must end in .md, "
                f"got {non_md}. Product code stays on luna/glm workers."
            )
        # J-80 regression (2026-08-07, 4 burned design dispatches): the boss passed the
        # CONDUCTOR EVIDENCE path ("conductor/opus_walk/evidence/<ID>/seam-design-proposal.md")
        # as the write set. That path does not exist inside the ELAI worktree, so whatever the
        # worker writes violates scope -> BLOCKED, on every model, forever. Fail-closed BEFORE
        # spawning (zero model cost) with the exact correction: allowed_writes must be
        # WORKTREE-RELATIVE; the evidence copy is the BOSS's job after the dispatch returns.
        bad_roots = []
        for p in raw_writes:
            q = p.strip()
            while q.startswith("./"):
                q = q[2:]
            if q.startswith("/") or q.startswith("conductor/"):
                bad_roots.append(p)
        if bad_roots:
            raise ValueError(
                f"design write set must be WORKTREE-RELATIVE, got {bad_roots}. Use "
                f"'workflow/reasoning/seam-proposals/<ID>-seam-proposal.md'; you (the boss) copy "
                f"the result to conductor/opus_walk/evidence/<ID>/seam-design-proposal.md yourself."
            )
        # Design floor (L-1 regression 2026-08-07): an 80-turn grounded design run cannot fit
        # the 600s EXECUTE-slice wall — both live L-1 tries died exit -1 on it. Measured opus-5
        # pace on I-33 was ~34s/turn, so 80 turns needs ~2700s: floor 3600. Auto-raised here so
        # the lane works regardless of whether the caller remembered --timeout.
        if run_timeout is None or run_timeout < 3600:
            run_timeout = 3600
    model = effective_worker_model(requested_model)
    if role not in ROLES:
        raise ValueError(f"role must be one of {sorted(ROLES)}, got {role!r}")
    if not objective or not objective.strip():
        raise ValueError("objective must be a non-empty worker task text")
    if dry_run:
        return {
            "ok": True,
            "dry_run": True,
            "model": model,
            "role": role,
            "lane": "posthog",
            "objective": objective[:200],
        }
    # Reuse canvas dispatch with PAID tier TaskSpec; WorkerAdapter selects by tier.
    # pinned_model is posthog-gpt56-luna so the grok harness (if present) or adapter
    # receives the PostHog model string — never a free-bridge id.
    allowed_writes = expand_allowed_writes(allowed_writes)
    task = _make_taskspec(
        model,
        role,
        elai_plan,
        objective,
        allowed_writes,
        base_commit,
        token_budget,
        ttl,
        model_tier="PAID",
    )
    return _run_dispatch(
        task, elai_plan, agent, dry_run, run_timeout, worktree_path=worktree_path
    )


def dispatch_codex_executor(elai_plan, objective, allowed_writes, base_commit="HEAD",
                            token_budget=CODEX_DEFAULT_TOKEN_BUDGET, ttl=3600, dry_run=False,
                            agent=None, run_timeout=1800, role="EXECUTE") -> dict:
    """Run a legacy codex-shaped request on the owner-pinned Luna-fast worker.

    The old lane name remains compatible, but no codex process, account, VM, or
    network route is used. The non-empty write-set, role, prompt, worktree,
    off-agent gate, and no-self-certification safety contracts remain binding.
    """
    if role not in ROLES:
        raise ValueError(f"role must be one of {sorted(ROLES)}, got {role!r}")
    if not objective or not objective.strip():
        raise ValueError("objective must be a non-empty worker task text")
    allowed = [p for p in (allowed_writes or []) if p and p.strip()]
    if not allowed:
        raise ValueError(
            "codex executor requires a non-empty allowed_writes (vertical-slice files): an empty set "
            "makes codex read-only (codex_provider sandbox rule), which is never a valid executor run")
    allowed = expand_allowed_writes(allowed)
    preamble = elai_cc_path_preamble()
    if preamble not in (objective or ""):
        objective = preamble + "\n\n" + (objective or "")

    task = _make_taskspec(
        ORDINARY_WORKER_MODEL,
        role,
        elai_plan,
        objective,
        allowed,
        base_commit,
        token_budget,
        ttl,
        model_tier="PAID",
    )
    return _run_dispatch(
        task,
        elai_plan,
        agent,
        dry_run,
        run_timeout,
    )


def dispatch_prime_rlm(
    model,
    role,
    elai_plan,
    objective,
    allowed_writes,
    base_commit,
    token_budget,
    ttl,
    dry_run=False,
    agent=None,
    run_timeout=DEFAULT_DISPATCH_TIMEOUT_S,
    worktree_path=None,
) -> dict:
    """Dispatch one opt-in Prime Agent RLM worker through the shared worker contract."""
    if model not in PRIME_RLM_MODEL_ALIASES:
        raise ValueError(f"prime-rlm model must be one of {tuple(PRIME_RLM_MODEL_ALIASES)}, got {model!r}")
    model = effective_worker_model(model)
    if role not in ROLES:
        raise ValueError(f"role must be one of {sorted(ROLES)}, got {role!r}")
    if not objective or not objective.strip():
        raise ValueError("objective must be a non-empty worker task text")
    if dry_run:
        return {
            "ok": True,
            "dry_run": True,
            "model": model,
            "role": role,
            "lane": "prime-rlm",
            "objective": objective[:200],
        }
    allowed_writes = expand_allowed_writes(allowed_writes)
    task = _make_taskspec(
        model,
        role,
        elai_plan,
        objective,
        allowed_writes,
        base_commit,
        token_budget,
        ttl,
        model_tier="PAID",
    )
    return _run_dispatch(
        task,
        elai_plan,
        agent,
        dry_run,
        run_timeout,
        worktree_path=worktree_path,
        adapter_factory=lambda config: _PrimeRlmWorkerAdapter.from_conductor_config(
            config, prime_model=model
        ),
        pre_dispatch_notes=_prime_stale_daemon_notes(),
    )


def _configure_worker_cargo_environment() -> None:
    mode = (os.environ.get("TEIKOKU_DEPOT_OFFLOAD") or "remote").strip().lower()
    if mode not in {"remote", "prefer", "local"}:
        mode = "remote"
    if mode == "remote":
        os.environ["TEIKOKU_DEPOT_OFFLOAD"] = "remote"
        os.environ["PATH"] = f"{DEPOT_SHIM_DIR}:{os.environ.get('PATH', '')}"
        os.environ.pop("CARGO_TARGET_DIR", None)
        return
    if mode == "prefer":
        _cargo_bin = str(Path.home() / ".cargo" / "bin")
        if _cargo_bin not in os.environ.get("PATH", ""):
            os.environ["PATH"] = f"{_cargo_bin}:{os.environ.get('PATH', '')}"
        os.environ.setdefault("CARGO_TARGET_DIR", str(Path.home() / ".cache" / "cargo-target-elai"))
        return
    _cargo_bin = str(Path.home() / ".cargo" / "bin")
    if _cargo_bin not in os.environ.get("PATH", ""):
        os.environ["PATH"] = f"{_cargo_bin}:{os.environ.get('PATH', '')}"
    os.environ.setdefault("CARGO_TARGET_DIR", str(Path.home() / ".cache" / "cargo-target-elai"))


def main() -> int:
    ap = argparse.ArgumentParser(
        description=(
            "Worker dispatch: free Devin (glm-5-2/swe-1-7*) OR PostHog Luna OR paid codex. "
            "Grok 4.5 is never free."
        )
    )
    ap.add_argument("--lane", default="free", choices=["free", "codex", "posthog", "prime-rlm"],
                    help="free = free Devin bridge only (glm-5-2 / swe-1-7*); "
                         "posthog = PostHog-credit workers via grok CLI (posthog-gpt56-luna); "
                         "prime-rlm = opt-in Prime Agent RLM worker via Zenov; "
                         "codex = legacy codex-shaped request with non-empty write-set safety; "
                         "execution still uses Luna-fast/max.")
    ap.add_argument(
        "--model",
        required=False,
        choices=list(FREE_BRIDGE_MODELS)
        + list(POSTHOG_WORKER_MODELS)
        + list(POSTHOG_DESIGN_MODELS)
        + list(PRIME_RLM_MODEL_ALIASES)
        + list(_DISABLED_BRIDGE_MODELS),
        help=(
            "free: glm-5-2 (default) / swe-1-7*. posthog: posthog-gpt56-luna (default); "
            "posthog-fable-5/posthog-opus-5 are DESIGN-ONLY (doc-only write set). "
            "Never grok-4-5-high as free. kimi-k2-7 coerced to glm-5-2."
        ),
    )
    ap.add_argument("--role", required=True, choices=sorted(ROLES),
                    help="the conductor worker-role/wave TIER (EXECUTE/FIX/WIRE/AUDIT/POLISH/"
                         "INTEGRATE/PLAN) -- the mechanical tier, NOT the agent identity")
    ap.add_argument("--agent", default=None,
                    help="the ELAI agent the worker EMBODIES (e.g. context-compiler, "
                         "integration-explorer-elai, wiring-contract-validator, auditor, "
                         "impact-mapper, spec-grounder, fixer, e2e-tester). Prepends the canonical "
                         "agent-dispatch pointer so the worker reads that agent's .md; resolved off "
                         f"{ELAI_AGENTS_DIR}, fail-closed on an unknown name.")
    ap.add_argument("--elai-plan", required=True,
                    help="the ELAI plan id being worked (canvas label + event tag only)")
    ap.add_argument("--objective", help="the worker's task text (or use --prompt-file)")
    ap.add_argument("--prompt-file", help="file whose contents are the worker objective")
    ap.add_argument("--allowed-writes", default="",
                    help="comma-separated repo-relative write set (empty for a read/explore worker)")
    ap.add_argument("--evidence-file",
                    help="path the worker writes its FULL product to for the next agent (agent-to-"
                         "agent file handoff). When set, the worker returns only STATUS + this path "
                         "to keep the orchestrator context clean. Requires --agent.")
    ap.add_argument("--base-commit", default="HEAD")
    ap.add_argument("--token-budget", type=int, default=100000)
    ap.add_argument("--lease-ttl", type=int, default=3600)
    ap.add_argument("--timeout", type=int, default=DEFAULT_DISPATCH_TIMEOUT_S,
                    help=f"hard per-dispatch ceiling in seconds (default {DEFAULT_DISPATCH_TIMEOUT_S}). "
                         "WorkerAdapter kills the worker's process group on expiry (no orphan) and the "
                         "call returns a FAILED result. Do NOT wrap this command in a shorter shell "
                         "`timeout` -- that orphans the base-cc child; use this flag instead. Lower it "
                         "for a quick recon (e.g. 300); a slice needing more is a decomposition failure.")
    ap.add_argument("--dry-run", action="store_true",
                    help="render + create the node but do not launch the worker (WorkerAdapter dry_run)")
    ap.add_argument("--json", action="store_true", help="pretty-print the result blob")
    ap.add_argument("--reuse-worktree", default=None,
                    help="FREE lane only: an EXISTING free worktree path (from a prior dispatch's "
                         "result \"worktree\" field) to REUSE instead of creating a fresh one. The "
                         "continuation worker inherits the prior worker's uncommitted partial (e.g. "
                         "MOVES 1-4b of a coupled slice too big for one 150k window) so the slice is "
                         "finished across dispatches WITHOUT a worker git-apply (banned by R-1). "
                         "Ignored for --lane codex (codex forces its own /mnt/f worktree).")
    args = ap.parse_args()

    _configure_worker_cargo_environment()

    # Every ordinary CLI lane resolves to Luna-fast/max. Codex remains a
    # specialized executor with its own isolated worktree and safety gates.
    if args.lane == "posthog":
        model = args.model if args.model in POSTHOG_DESIGN_MODELS else DEFAULT_POSTHOG_WORKER_MODEL
    elif args.lane == "prime-rlm":
        model = PRIME_RLM_DEFAULT_MODEL
    elif args.lane == "codex":
        model = CODEX_MODEL
    else:
        model = DEFAULT_FREE_MODEL

    objective = args.objective
    if args.prompt_file:
        objective = Path(args.prompt_file).read_text(encoding="utf-8")
    if not objective or not objective.strip():
        print("ERROR: one of --objective or a non-empty --prompt-file is required", file=sys.stderr)
        return 2

    # --agent (when set) folds the canonical ELAI agent-dispatch pointer into the objective so the
    # worker EMBODIES that agent (context-compiler / wiring-contract-validator / auditor / ...), not
    # just the coarse --role tier. Fail-closed on an unknown agent name.
    if args.evidence_file and not args.agent:
        print("ERROR: --evidence-file requires --agent", file=sys.stderr)
        return 2
    try:
        if args.lane == "codex":
            # codex's scoped mount cannot read /mnt/f/.claude OR /mnt/f/docs -> ALWAYS inline the
            # binding preamble (F-4) + the L1 design overview (F-2 L1) and strip any unreachable
            # "Read <path> FIRST" pointer, regardless of --agent (codex dispatches pass agent=None).
            objective = _compose_codex_objective(
                args.agent, objective, evidence_file=args.evidence_file,
                overview_text=_load_codex_overview(),
            )
        elif args.agent or args.evidence_file:
            objective = _compose_agent_objective(args.agent, objective, evidence_file=args.evidence_file)
    except ValueError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2
    allowed = [p.strip() for p in args.allowed_writes.split(",") if p.strip()]
    if args.lane == "prime-rlm":
        try:
            out = dispatch_prime_rlm(
                model,
                args.role,
                args.elai_plan,
                objective,
                allowed,
                args.base_commit,
                args.token_budget,
                args.lease_ttl,
                dry_run=args.dry_run,
                agent=args.agent,
                run_timeout=args.timeout,
                worktree_path=args.reuse_worktree,
            )
        except (RuntimeError, ValueError) as exc:
            print(f"ERROR: {exc}", file=sys.stderr)
            return 2
    elif args.lane == "codex":
        # PAID gpt-5.6-sol executor. Larger defaults than the FREE lane (1M ctx, hard units run
        # longer): a bare --token-budget/--timeout at their FREE defaults is bumped to the codex
        # defaults; an explicit larger value is respected.
        token_budget = args.token_budget if args.token_budget > 100000 else CODEX_DEFAULT_TOKEN_BUDGET
        run_timeout = args.timeout if args.timeout != DEFAULT_DISPATCH_TIMEOUT_S else 1800
        try:
            out = dispatch_codex_executor(
                args.elai_plan, objective, allowed, base_commit=args.base_commit,
                token_budget=token_budget, ttl=args.lease_ttl, dry_run=args.dry_run,
                agent=args.agent, run_timeout=run_timeout, role=args.role,
            )
        except (RuntimeError, ValueError) as exc:
            print(f"ERROR: {exc}", file=sys.stderr)
            return 2
    elif args.lane == "posthog":
        try:
            out = dispatch_posthog_worker(
                model,
                args.role,
                args.elai_plan,
                objective,
                allowed,
                args.base_commit,
                args.token_budget,
                args.lease_ttl,
                dry_run=args.dry_run,
                agent=args.agent,
                run_timeout=args.timeout,
                worktree_path=args.reuse_worktree,
            )
        except (RuntimeError, ValueError) as exc:
            print(f"ERROR: {exc}", file=sys.stderr)
            return 2
    else:
        try:
            out = dispatch_worker(
                model, args.role, args.elai_plan, objective, allowed,
                args.base_commit, args.token_budget, args.lease_ttl, dry_run=args.dry_run,
                agent=args.agent, run_timeout=args.timeout, worktree_path=args.reuse_worktree,
            )
        except (RuntimeError, ValueError) as exc:
            print(f"ERROR: {exc}", file=sys.stderr)
            return 2
    print(json.dumps(out, indent=2, sort_keys=True) if args.json else json.dumps(out, sort_keys=True))
    # STATUS line LAST so the Opus greps the final line deterministically for the outcome class.
    # .get: the posthog/prime dry-run early-return dicts carry no "status" (pre-existing; found
    # smoking the seam-design lane 2026-08-06) — a probe must not crash after printing its JSON.
    print(f"STATUS: {out.get('status', 'DRY_RUN' if out.get('dry_run') else 'UNKNOWN')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
