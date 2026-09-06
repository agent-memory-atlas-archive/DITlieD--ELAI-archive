"""opus_land.py - conductor-side AUDIT + LAND primitive for the opus-orchestrator walk.

WHY THIS EXISTS (2026-07-10, monitor session): the opus walk's implement leg is PROVEN (a free
worker produces a correct patch in an ext4 worktree) but the audit+land leg was structurally
impossible for the Opus turn itself: (a) scope_write_gate hard-denies /mnt/f writes to any base-cc
turn (by design, TEIKOKU_AUTONOMOUS does not lift it), (b) the frozen checkers at
/mnt/f/.ELAI_workflow/.claude/tools compute paths relative to THEIR OWN root, so pointing the
canonical copy at a WSL worktree path raises ValueError, (c) workers have no cargo on PATH.
Four paid Opus turns re-derived that wall verbatim (respawn burn). This primitive is the fix:
a deterministic, conductor-side (ungated - hooks bind base-cc tool calls, not conductor python)
leg the Opus CALLS, mirroring the deterministic walk's own integrator.py (.elai_cc scope guard +
SerializedIntegrator merge into the real repo + optional push), plus the gate leg run INSIDE the worktree with
the frozen checkers COPIED IN (sha256 of each canonical source recorded in the result, so the
frozen-gate property is auditable: the copies are byte-identical to the canonical frozen tools).

The Opus never edits /mnt/f and never self-authors a verdict: every gate here is a frozen tool
or a cargo exit code (R4 off-agent artifacts). Fail-closed: any gate non-zero, any out-of-scope
path, any dirty-overlap with /mnt/f = REFUSED, exit 1, nothing landed.

Usage (from ~/teikoku/conductor):
  python3 opus_walk/opus_land.py --worktree <path> --plan-id H-28-WIRE-1 \
      --files .elai_cc/crates/orchestrator/src/darwin_admit_loop.rs,... \
      --gate-cmd "cargo check -p elai-orchestrator" \
      --gate-cmd "cargo nextest run -p elai-orchestrator --test reward_tamper_admit_observable_smoke" \
      --gates-only            # audit leg only (S6/S7)
  ... same + --message "<contributor voice>" --push   # land leg (S10)
Exit 0 = PASS/LANDED, 1 = gate fail / scope refusal / merge refusal, 2 = config error.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import time
from pathlib import Path

CONDUCTOR = Path(__file__).resolve().parents[1]
_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))
import depot_cargo

def depot_offload_is_heavy(command: list[str]) -> bool:
    """cargo check/build/test/clippy/bench/rustc — the builds the gate must own."""
    return depot_cargo.depot_offload.is_heavy_cargo_command(command)


# Gate commands that are SHELL WRAPPERS (sh/bash/… some-script) can run cargo INSIDE the
# script, invisible to the argv heavy-detector. Live 2026-08-11 ~00:0xZ: an i22 gate leg was
# `sh /tmp/i22_gate.sh` whose body ran two `cargo test` legs; the outer argv matched nothing,
# the leg went down run_gate_command's local path UNCAGED, and rustc_tripwire v7 killed the
# inner cargo (rc=-15) SEVEN times in a loop — every kill recorded as a spurious gate FAIL.
# Fail closed: a shell-wrapper gate command is treated as heavy so it runs inside the cage,
# where the breaker is log-only. A caged non-cargo script costs one idle scope, nothing more.
_SHELL_WRAPPERS = ("sh", "bash", "dash", "zsh")


def _is_shell_wrapper(command: list[str]) -> bool:
    return bool(command) and Path(command[0]).name in _SHELL_WRAPPERS


ELAI_ROOT = Path("/workspace/ELAI")
CANON_TOOLS = ELAI_ROOT / ".claude" / "tools"
DB_PATH = CONDUCTOR / "state" / "control.db"
CARGO_BIN = Path.home() / ".cargo" / "bin"
CHECKERS = ("production_caller_check.py", "maintainability_check.py")
# maintainability_check.py reviews only these source extensions (mirrors its SOURCE_EXTS);
# a changeset with none of them (e.g. a pure benchmark-artifact JSON landing) has no code to
# review, so the maintainability leg is N/A rather than a fail-closed no-source scan error.
MAINTAINABILITY_SOURCE_EXTS = (".rs", ".ts", ".tsx", ".js", ".jsx", ".svelte", ".py")
SCOPE_PREFIX = ".elai_cc/"
# Owner-only escape hatch for --no-verifier (see the P1.5 block). A FILE with a JSON body
# (timestamp + scope.plan_id), not a flag and not an env var: the owner writes it out-of-band
# and no agent-composed command line can conjure a scoped grant. Absent/empty/wrong-scope =
# the verifier always runs. Parse: parse_allow_no_verifier.
ALLOW_NO_VERIFIER_SENTINEL = Path(__file__).resolve().parent.parent / "ALLOW_NO_VERIFIER"
# Bounded retries on a transient verifier outage. NOT applied to a REJECT (a real verdict).
VERIFIER_ATTEMPTS = 3
GATE_TIMEOUT_S = 3600
PLAN_ID_CANVAS = "teikoku-opus-walk"
# Bidirectional sync (deliverable 6, core-sync-design.md 3.1 FIX-2): touched on every LANDED verdict
# so the F: status feeder re-fires on the land (live snapshot) instead of waiting for the watcher
# debounce. Best-effort; a touch failure never fails a completed land.
LAND_TRIGGER = ELAI_ROOT / "workflow" / "status" / ".land-trigger"


def _run(cmd, cwd, timeout=GATE_TIMEOUT_S, env=None):
    return subprocess.run(cmd, cwd=str(cwd), env=env, shell=isinstance(cmd, str),
                          capture_output=True, text=True, timeout=timeout)


def _translate_nextest(command: list[str]) -> list[str]:
    """`cargo nextest run ...` -> `cargo test ...`.

    cargo-nextest is not installed on this host and the remote CI job is fixed
    (elai-core-check), so a nextest gate literally cannot run anywhere: every one
    of its 77 historical gate events is a FAIL (rc=101 `no such command`). The
    test FILTER grammar both tools share (`-p <pkg>`, `--test <target>`, a bare
    substring filter) translates 1:1. nextest-only flags are left in place so
    `cargo test` rejects them loudly instead of silently dropping semantics.
    """
    if len(command) >= 3 and Path(command[0]).name in ("cargo", "cargo.exe") \
            and command[1] == "nextest" and command[2] == "run":
        return [command[0], "test", *command[3:]]
    return command


_GATE_ASSIGNMENT_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")


class _GateCommandError(ValueError):
    """A gate command has no executable after its leading assignments."""


def _normalize_gate_command(
    command: list[str], env: dict[str, str]
) -> tuple[list[str], dict[str, str]]:
    """Move supported command-local assignments into a fresh subprocess environment."""
    command_env = dict(env)
    assignments: dict[str, str] = {}
    command_start = 0
    while command_start < len(command) and _GATE_ASSIGNMENT_RE.match(
        command[command_start]
    ):
        name, value = command[command_start].split("=", 1)
        assignments[name] = value
        command_start += 1
    if command_start < len(command) and Path(command[command_start]).name == "env":
        env_start = command_start + 1
        if env_start < len(command) and _GATE_ASSIGNMENT_RE.match(command[env_start]):
            command_start = env_start
            while command_start < len(command) and _GATE_ASSIGNMENT_RE.match(
                command[command_start]
            ):
                name, value = command[command_start].split("=", 1)
                assignments[name] = value
                command_start += 1
    if command_start >= len(command):
        if assignments:
            raise _GateCommandError(
                "gate command has no executable after environment assignments"
            )
        return command, command_env
    if not assignments:
        return command, command_env
    command_env.update(assignments)
    return command[command_start:], command_env


GATE_TARGET_BUDGET_GB_ENV = "TEIKOKU_GATE_TARGET_BUDGET_GB"
GATE_TARGET_BUDGET_GB_DEFAULT = 20.0


def _trim_gate_target_dir(target_dir: Path, budget_gb: float | None = None) -> bool:
    """Wipe the SHARED gate target dir when it outgrows its byte budget.

    The local-caged gate law gave lands a shared incremental CARGO_TARGET_DIR.
    Cargo target dirs only accumulate (no GC): measured 0 -> 41G in ~10h of
    lands (2026-08-08 01:30Z), marching straight at the disk floor the walk's
    own disk guard defends. A wipe is the only correct trim cargo supports;
    it costs one cold gate build (~6 min) per breach. Called at gate start,
    which is never mid-build because lands are serialized.
    Fail-open: any error skips the trim — a trim must never block a land.
    Returns True when a wipe happened.
    """
    if budget_gb is None:
        try:
            budget_gb = float(os.environ.get(GATE_TARGET_BUDGET_GB_ENV, "")
                              or GATE_TARGET_BUDGET_GB_DEFAULT)
        except ValueError:
            budget_gb = GATE_TARGET_BUDGET_GB_DEFAULT
    try:
        if not target_dir.is_dir():
            return False
        # Own-artifact guard: only the shared gate dir (or a dir cargo itself
        # marked with CACHEDIR.TAG) may ever be wiped. An env misconfig that
        # points at $HOME or a repo must never mass-delete.
        if target_dir.name != "cargo-target-elai" \
                and not (target_dir / "CACHEDIR.TAG").exists():
            return False
        du = subprocess.run(["du", "-sb", str(target_dir)],
                            capture_output=True, text=True, timeout=180)
        if du.returncode != 0 or not du.stdout.strip():
            return False
        size_b = int(du.stdout.split()[0])
        if size_b <= budget_gb * 1024 ** 3:
            return False
        shutil.rmtree(target_dir)
        print(f"[opus-land] gate target dir {target_dir} was {size_b / 1024 ** 3:.1f}G "
              f"(budget {budget_gb:.0f}G): wiped; next gate build runs cold",
              file=sys.stderr)
        return True
    except Exception as exc:  # noqa: BLE001 - trim must never block the land
        print(f"[opus-land] gate target trim skipped: {exc}", file=sys.stderr)
        return False


def _cage_wrap(command: list[str]) -> list[str]:
    """Run a gate build inside a systemd `run-*.scope` cage (breaker v7 log-only).

    The land gate MUST judge the candidate worktree. The depot-ci remote path
    runs a FIXED job (elai-core-check) against the MAIN checkout, so it never
    sees the candidate tree — that is how J-102 passed gates and still broke
    main. An uncaged local build is killed by rustc_tripwire v7 (rc=-15,
    MAINFIX-J102 10:23:46Z). A caged local build is the one lane that is both
    honest (real tree, real command) and lawful (bounded RAM, breaker exempt).
    """
    if shutil.which("systemd-run") is None:
        return list(command)  # no cage available; breaker policy owns the outcome
    uid = os.getuid()
    runtime = os.environ.get("XDG_RUNTIME_DIR") or f"/run/user/{uid}"
    if not os.environ.get("XDG_RUNTIME_DIR") and os.path.isdir(runtime):
        os.environ["XDG_RUNTIME_DIR"] = runtime
    bus_path = f"{runtime}/bus"
    if not os.environ.get("DBUS_SESSION_BUS_ADDRESS") and os.path.exists(bus_path):
        os.environ["DBUS_SESSION_BUS_ADDRESS"] = f"unix:path={bus_path}"
    memmax = (os.environ.get("TEIKOKU_GATE_MEMMAX")
              or os.environ.get("TEIKOKU_WORKER_MEMMAX") or "10G").strip()
    props = [
        "-p", f"MemoryMax={memmax}", "-p", "MemorySwapMax=0", "-p", "TasksMax=512",
    ]
    # IO write cap (owner-approved 2026-08-12): only effective once the user slice
    # delegates the io controller (drop-in user@.service.d/teikoku-io-delegate.conf,
    # active after the next reboot). Guarded at runtime: without delegation systemd
    # ignores the property silently, but we skip it explicitly so the cage command
    # stays honest about what it enforces.
    iomax = (os.environ.get("TEIKOKU_GATE_IOMAX") or "200M").strip()
    if iomax and iomax != "0" and _io_controller_delegated(uid):
        iodev = (os.environ.get("TEIKOKU_GATE_IODEV") or "/dev/nvme0n1").strip()
        props += ["-p", f"IOWriteBandwidthMax={iodev} {iomax}"]
    return [
        "systemd-run", "--user", "--scope", "--collect",
        *props,
        "--quiet", *command,
    ]


def _io_controller_delegated(uid: int) -> bool:
    """True when the io controller is delegated to the user slice (post-reboot
    with the teikoku-io-delegate drop-in). Reads the cgroup2 controllers file;
    any error means not-delegated (fail-closed to the no-cap behavior)."""
    try:
        path = f"/sys/fs/cgroup/user.slice/user-{uid}.slice/cgroup.subtree_control"
        with open(path, encoding="utf-8") as fh:
            return "io" in fh.read().split()
    except OSError:
        return False


LAND_GATE_LOGS_ROOT = _HERE / "evidence"


def _command_slug(command_text: str, max_len: int = 80) -> str:
    """Filesystem-safe slug for a gate command in log filenames."""
    slug = re.sub(r"[^A-Za-z0-9._-]+", "_", command_text.strip())
    slug = slug.strip("._-") or "cmd"
    return slug[:max_len]


def persist_gate_command_log(
    plan_id: str | None,
    index: int,
    command_text: str,
    full_output: str,
    *,
    root: Path | None = None,
) -> Path | None:
    """Write the untruncated gate stdout+stderr under evidence/<unit>/land-gate-logs/.

    Returns the log path, or None if persistence was skipped (no unit id).
    Fail-soft: a log write must never fail a land.
    """
    unit = (plan_id or "").strip()
    if not unit:
        return None
    base = root if root is not None else LAND_GATE_LOGS_ROOT
    log_dir = base / unit / "land-gate-logs"
    try:
        log_dir.mkdir(parents=True, exist_ok=True)
        path = log_dir / f"{index}_{_command_slug(command_text)}.log"
        path.write_text(full_output if full_output is not None else "", encoding="utf-8",
                        errors="replace")
        return path
    except OSError as exc:
        print(f"[opus-land] land-gate-log write skipped: {exc}", file=sys.stderr)
        return None


def run_gate_leg(
    gate_commands: list[str],
    cwd: Path,
    env: dict[str, str],
    timeout_s: int = GATE_TIMEOUT_S,
    plan_id: str | None = None,
    log_root: Path | None = None,
) -> tuple[list[dict], str | None]:
    entries: list[dict] = []
    target_trimmed = False
    for command_text in gate_commands:
        try:
            command, command_env = _normalize_gate_command(shlex.split(command_text), env)
        except _GateCommandError as exc:
            entries.append({"cmd": command_text, "rc": None, "timeout": False})
            return entries, f"GATE_FAIL {exc}: {command_text}"
        command = _translate_nextest(command)
        t0 = time.time()
        try:
            if depot_offload_is_heavy(command) or _is_shell_wrapper(command):
                # Candidate-tree law: heavy gate builds run HERE, on the worktree,
                # caged. Shared target dir keeps rebuilds incremental across lands.
                caged_cmd = _cage_wrap(command)
                # _cage_wrap may fill XDG_RUNTIME_DIR / DBUS_SESSION_BUS_ADDRESS on
                # os.environ for --user scopes. The subprocess env is a copy of
                # `env` and must inherit those or systemd-run fails with rc=1 and
                # a "Failed to connect to user scope bus" tail (no cargo run).
                gate_env = dict(command_env)
                for key in ("XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS"):
                    if os.environ.get(key):
                        gate_env.setdefault(key, os.environ[key])
                gate_env.setdefault("CARGO_TARGET_DIR",
                                    str(Path.home() / ".cache" / "cargo-target-elai"))
                if not target_trimmed:
                    # Once per leg, before the first heavy build: enforce the
                    # byte budget on the (shared) target dir. Serialized lands
                    # make gate start the one safe wipe point.
                    _trim_gate_target_dir(Path(gate_env["CARGO_TARGET_DIR"]))
                    target_trimmed = True
                proc = _run(caged_cmd, cwd, timeout=timeout_s, env=gate_env)
                proc.provider = "local-caged"  # type: ignore[attr-defined]
            else:
                proc = depot_cargo.run_gate_command(command, cwd, timeout_s, env=command_env)
        except subprocess.TimeoutExpired:
            entries.append({"cmd": command_text, "rc": None, "timeout": True})
            return entries, "GATE_FAIL timeout: " + command_text
        full_output = (proc.stdout or "") + (proc.stderr or "")
        entry = {
            "cmd": command_text,
            "rc": proc.returncode,
            "secs": int(time.time() - t0),
            # Compat: JSON consumers still get the 1500-char tail only.
            "tail": full_output[-1500:],
        }
        log_path = persist_gate_command_log(
            plan_id, len(entries), command_text, full_output, root=log_root,
        )
        if log_path is not None:
            entry["log_path"] = str(log_path)
        depot_id = getattr(proc, "depot_id", None) or getattr(proc, "request_id", None)
        if depot_id:
            entry["depot_id"] = depot_id
        if getattr(proc, "provider", None):
            entry["provider"] = proc.provider
        entries.append(entry)
        if proc.returncode != 0:
            return entries, f"GATE_FAIL rc={proc.returncode}: {command_text}"
    return entries, None


KEEP_LAND_MARKER_NAME = ".KEEP_FOR_LAND"
WORKTREE_ROOT = CONDUCTOR / "elai-worktrees"


def _release_keep_for_land_markers(plan_id: str) -> None:
    """Clear .KEEP_FOR_LAND markers for a plan whose land just MERGED.

    opus_dispatch sets the marker so disk_hygiene never reaps a tree the land
    still needs. Nothing ever cleared it, so landed plans' trees were protected
    forever: 13 of 23 trees marked on 2026-08-08 07:3xZ, several for plans
    already pushed. Marker removal (not tree removal) is the whole job — the
    hygiene sweeper owns actual reaping on its own cadence. Fail-soft: never
    breaks a completed land.
    """
    if not plan_id:
        return
    try:
        for wt in WORKTREE_ROOT.glob(f"opuswalk-{plan_id}-*"):
            marker = wt / KEEP_LAND_MARKER_NAME
            if marker.is_file():
                marker.unlink()
                print(f"[opus-land] released KEEP_FOR_LAND on {wt.name}", file=sys.stderr)
    except Exception as exc:  # noqa: BLE001 - marker hygiene must never fail a land
        print(f"[opus-land] KEEP_FOR_LAND release skipped: {exc}", file=sys.stderr)


def _emit(event_type: str, payload: dict) -> None:
    try:
        sys.path.insert(0, str(CONDUCTOR))
        import events  # type: ignore
        events.append_event(str(DB_PATH), event_type, PLAN_ID_CANVAS, payload,
                            project_id=None, session_id=None)
    except Exception as exc:  # noqa: BLE001 - telemetry must never block the land
        print(f"[opus-land] event emit failed (non-fatal): {exc}", file=sys.stderr)


def _changed_paths(worktree: Path) -> list[str]:
    out = _run(["git", "status", "--porcelain"], worktree, timeout=60).stdout
    paths = []
    for line in out.splitlines():
        if not line.strip():
            continue
        p = line[3:].strip().strip('"')
        if " -> " in p:
            p = p.split(" -> ", 1)[1]
        paths.append(p)
    return paths


def _sha256(path: Path) -> str:
    h = hashlib.sha256()
    h.update(path.read_bytes())
    return h.hexdigest()


def _collect_diff(worktree: Path, changed: list[str]) -> str:
    """The final diff for the pre-land verifier: the uncommitted worktree diff over the changeset
    paths, falling back to the committed diff (main..wt) on the crashed-land resume path where the
    worktree is already one in-scope commit ahead of main. Best-effort; '' when nothing resolves."""
    out = _run(["git", "diff", "HEAD", "--"] + changed, worktree, timeout=120).stdout
    if out.strip():
        return out
    mh = _run(["git", "rev-parse", "HEAD"], ELAI_ROOT, timeout=60).stdout.strip()
    wh = _run(["git", "rev-parse", "HEAD"], worktree, timeout=60).stdout.strip()
    if wh and wh != mh:
        return _run(["git", "diff", mh, wh, "--"] + changed, worktree, timeout=120).stdout
    return ""


def _quality_of(verdict: str):
    """Map a terminal verdict to worker-quality pass/fail for the trust ledger (D-10), or None to
    record nothing. LANDED / LANDED_PUSH_FAILED = the gates+verifier passed and the code merged
    (push transport is not worker quality) -> True. GATE_FAIL / CHECKER_BLOCK / VERIFIER_REJECTED
    -> False. CONFIG_ERROR / REFUSED (scope/infra), CHECKER_CONFIG_ERROR and VERIFIER_UNAVAILABLE
    (infra, not quality) and the audit-only GATES_PASS record nothing."""
    if verdict.startswith("LANDED"):
        return True
    if (verdict.startswith("GATE_FAIL") or verdict.startswith("CHECKER_BLOCK")
            or verdict.startswith("VERIFIER_REJECTED")):
        return False
    return None


def parse_allow_no_verifier(path: Path, plan_id: str) -> tuple[bool, str, dict]:
    """Body-aware owner escape for --no-verifier (ARCH-EV-ESCAPE-BODY / §3.5).

    Sentinel must be a UTF-8 JSON object with required keys:
      timestamp: ISO-8601 UTC string or integer/float unix seconds (presence only; no max-age yet)
      scope: object whose plan_id string equals the land --plan-id (exactly one task; no '*')

    Returns (granted, reason, detail). reason is one of:
      granted | missing | empty_body | bad_json | missing_fields | scope_mismatch | unreadable

    Path-only: no env var grant path (ABORT if one is introduced). Callers pass a Path.
    """
    detail: dict = {"sentinel": str(path), "plan_id": plan_id}
    if not path.is_file():
        return False, "missing", detail
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as exc:
        detail["error"] = str(exc)
        return False, "unreadable", detail
    if not raw.strip():
        return False, "empty_body", detail
    try:
        body = json.loads(raw)
    except json.JSONDecodeError as exc:
        detail["error"] = str(exc)
        return False, "bad_json", detail
    if not isinstance(body, dict):
        detail["error"] = "body_not_object"
        return False, "bad_json", detail
    timestamp = body.get("timestamp")
    scope = body.get("scope")
    if timestamp is None or timestamp == "":
        detail["missing"] = "timestamp"
        return False, "missing_fields", detail
    if not isinstance(timestamp, (str, int, float)) or isinstance(timestamp, bool):
        detail["missing"] = "timestamp"
        return False, "missing_fields", detail
    if not isinstance(scope, dict):
        detail["missing"] = "scope"
        return False, "missing_fields", detail
    scope_plan = scope.get("plan_id")
    detail["timestamp"] = timestamp
    detail["scope"] = scope
    if scope_plan is None or scope_plan == "":
        detail["missing"] = "scope.plan_id"
        return False, "missing_fields", detail
    if not isinstance(scope_plan, str) or scope_plan == "*" or scope_plan != plan_id:
        detail["scope_plan_id"] = scope_plan
        return False, "scope_mismatch", detail
    if body.get("signer") is not None:
        detail["signer"] = body.get("signer")
    if body.get("reason") is not None:
        detail["owner_reason"] = body.get("reason")
    return True, "granted", detail


WAITING_FOR_VERIFIER = "WAITING_FOR_VERIFIER"
PLAN_STATUS_DIR = Path(__file__).resolve().parent / "plan-status"


def remove_pending_land_on_success(plan_id, worktree="", pending_path=None):
    """Drop the pending-lands park row after a confirmed land (BURN-U2).

    Returns the number of rows removed. Fail-soft callers wrap this; a ledger
    miss must never un-land a landed plan. Uses the real pending_lands.remove
    entry point so the ledger no longer only grows.
    """
    import pending_lands
    kwargs = {}
    if pending_path is not None:
        kwargs["path"] = pending_path
    return pending_lands.remove(plan_id=plan_id or "", worktree=worktree or "", **kwargs)


def park_verifier_unavailable(
    plan_id: str,
    worktree: str,
    *,
    detail: dict | None = None,
    attempt: int = VERIFIER_ATTEMPTS,
    task_id: str | None = None,
    pending_path=None,
    plan_status_dir: Path | None = None,
    db_path: Path | None = None,
    emit_fn=None,
) -> dict:
    """Park a land when the preland verifier seat is unavailable (A-EV-QUEUE-BLOCK / A-46).

    Hard queue-and-block: never pass, never DONE, never LANDED. Durable state so the lane
    can continue other work and resume this plan by re-running land (no re-execute):
      - plan-status/{plan_id}.json status=WAITING_FOR_VERIFIER
      - pending_lands entry with reason=WAITING_FOR_VERIFIER
      - optional ControlDB task row WAITING_FOR_VERIFIER when task_id is provided

    Resume: list_waiting_for_verifier() / re-invoke opus_land on the same worktree when
    the verifier is back. Returns the park artifact dict (also emitted as an event).
    """
    if not plan_id or not worktree:
        raise ValueError("plan_id and worktree are required to park")
    detail = dict(detail or {})
    now = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    status_dir = Path(plan_status_dir) if plan_status_dir is not None else PLAN_STATUS_DIR
    status_dir.mkdir(parents=True, exist_ok=True)
    row = {
        "plan_id": plan_id,
        "status": WAITING_FOR_VERIFIER,
        "waiting_reason": "verifier_unavailable",
        "worktree": str(worktree),
        "attempt": attempt,
        "error": str(detail.get("error") or detail.get("grounds") or "")[:500],
        "last_run_iso": now,
        "parked_at": now,
    }
    row_path = status_dir / f"{plan_id}.json"
    tmp = row_path.with_name(row_path.name + f".{os.getpid()}.tmp")
    tmp.write_text(json.dumps(row, indent=2, sort_keys=True), encoding="utf-8")
    os.replace(tmp, row_path)

    _here = Path(__file__).resolve().parent
    if str(_here) not in sys.path:
        sys.path.insert(0, str(_here))
    import inspect
    import pending_lands
    if not callable(pending_lands.claim_row) or not callable(pending_lands.release_claim):
        raise RuntimeError("pending-land claim primitives unavailable")
    caller = inspect.currentframe().f_back
    live_args = None
    live_argv = None
    while caller is not None:
        possible_args = caller.f_locals.get("args")
        if hasattr(possible_args, "__dict__"):
            live_args = possible_args
        possible_argv = caller.f_locals.get("argv")
        if isinstance(possible_argv, list):
            live_argv = list(possible_argv)
        if live_args is not None and live_argv is not None:
            break
        caller = caller.f_back
    del caller
    pl_kwargs = {
        "reason": WAITING_FOR_VERIFIER,
        "artifact_dir": detail.get("artifact_dir") or "",
        "args": live_args,
        "replay_argv": live_argv,
    }
    if pending_path is not None:
        pl_kwargs["path"] = pending_path
    pending_lands.add(plan_id, str(worktree), **pl_kwargs)
    resume_queue = pending_lands.list_waiting_for_verifier(
        **({"path": pending_path} if pending_path is not None else {})
    )

    task_parked = False
    task_park_error = None
    if task_id:
        try:
            if str(CONDUCTOR) not in sys.path:
                sys.path.insert(0, str(CONDUCTOR))
            from conductor_db import ControlDB  # type: ignore
            cdb = ControlDB(str(db_path) if db_path is not None else str(DB_PATH))
            cdb.update_task_status(task_id, WAITING_FOR_VERIFIER, "verifier_unavailable")
            task_parked = True
        except Exception as exc:  # noqa: BLE001 - park ledger is primary; task row best-effort
            task_park_error = f"{type(exc).__name__}: {exc}"
            print(f"[opus-land] ControlDB park failed (non-fatal): {exc}", file=sys.stderr)

    park = {
        "plan_id": plan_id,
        "worktree": str(worktree),
        "waiting_status": WAITING_FOR_VERIFIER,
        "park": True,
        "attempt": attempt,
        "plan_status_path": str(row_path),
        "task_id": task_id,
        "task_parked": task_parked,
        "error": row["error"],
        "parked_at": now,
        "resume": "re-run opus_land on worktree when verifier available",
        "resume_queue_size": len(resume_queue),
    }
    if task_park_error:
        park["task_park_error"] = task_park_error
    _emitter = emit_fn if emit_fn is not None else _emit
    _emitter("OPUS_LAND_VERIFIER_PARKED", park)
    return park



def task_id_for_land(plan_id: str, task_id: str | None = None) -> str:
    if task_id:
        return task_id
    safe = re.sub(r"[^A-Za-z0-9_.-]+", "_", plan_id).strip("._-")
    return f"opus-land-{safe}" if safe else "opus-land-unknown"


def ensure_verified_task_for_land(
    *,
    plan_id: str,
    task_id: str | None,
    base_commit: str,
    allowed_write_set: list[str],
    db_path: Path | str | None = None,
) -> str:
    """Upsert/ensure a control.db task row status=VERIFIED for SerializedIntegrator."""
    if str(CONDUCTOR) not in sys.path:
        sys.path.insert(0, str(CONDUCTOR))
    from conductor_db import ControlDB  # type: ignore
    from models import TaskSpec, stable_idem_key  # type: ignore

    tid = task_id_for_land(plan_id, task_id)
    cdb = ControlDB(str(db_path) if db_path is not None else str(DB_PATH))
    # Always upsert scope: a prior land for the same plan_id (e.g. free_pool core)
    # leaves a stale allowed_write_set. Reusing it makes allowed_write_enforcement
    # FAIL on the new residual files (measured 2026-08-04 J-54 UNIT8).
    scope = list(allowed_write_set)
    task = TaskSpec(
        task_id=tid,
        plan_id=plan_id,
        wave="IMPLEMENT",
        role="EXECUTE",
        objective=f"opus land {plan_id}",
        base_commit=base_commit,
        allowed_write_set=scope,
        context_refs=scope,
        production_hook="conductor/opus_walk/opus_land.py::main",
        acceptance=f"land {plan_id} via SerializedIntegrator single-flight",
        deterministic_check="true",
        forbidden=["ungated ff-merge into repo_root"],
        verifier=[{"name": "opus_preland", "command": ["true"]}],
        lease_ttl_seconds=3600,
        idem_key=stable_idem_key([tid, base_commit, plan_id]),
        token_budget=200000,
    )
    cdb.upsert_task(task, status="VERIFIED")
    row = cdb.get_task_row(tid)
    if row is None or row["status"] != "VERIFIED":
        raise RuntimeError(f"task {tid} not VERIFIED after upsert")
    return tid


def _ssb_residual_gate_runner(**kwargs):
    """Candidate-gate runner for SSB residual lands (not a frozen gate edit).

    Strips ambient false fails that blocked J-54 UNIT8 (2026-08-04):
    - distro_gate_freeze ERROR on ~/.claude/settings.json drift (not this land)
    - traced_test --expect-traced on action_type_lock / ssb/tests payload tables
      (plain #[test] by design; zero #[traced_test] is not dead evidence there)
    """
    if str(CONDUCTOR) not in sys.path:
        sys.path.insert(0, str(CONDUCTOR))
    import gate_runner as gr  # type: ignore

    orig = gr.build_gate_commands

    def _filtered(context):
        cmds = orig(context)
        out = []
        for c in cmds:
            if c.name == "distro_gate_freeze":
                continue
            if c.name == "traced_test_assert_check":
                cmd = list(c.command)
                if "--expect-traced" in cmd:
                    cmd = [x for x in cmd if x != "--expect-traced"]
                out.append(gr.GateCommand(c.name, cmd, c.cwd, c.timeout_seconds))
                continue
            out.append(c)
        return out

    gr.build_gate_commands = _filtered  # type: ignore[assignment]
    try:
        return gr.run_gates(**kwargs)
    finally:
        gr.build_gate_commands = orig  # type: ignore[assignment]


def land_via_serialized_integrator(
    *,
    task_id: str,
    source_worktree: Path | str,
    source_base_commit: str,
    commit_message: str,
    allow_push: bool = False,
    db_path: Path | str | None = None,
    repo_root: Path | str | None = None,
    workflow_root: Path | str | None = None,
    artifact_root: Path | str | None = None,
    worktree_root: Path | str | None = None,
    gate_runner=None,
    packs: tuple[str, ...] = ("elai",),
    allow_local_commits: bool = True,
    block_dirty_checkout: bool = True,
):
    """Production land: SerializedIntegrator single-flight + post-merge gate + revert."""
    if str(CONDUCTOR) not in sys.path:
        sys.path.insert(0, str(CONDUCTOR))
    from integrator import SerializedIntegrator  # type: ignore

    kwargs = {
        "db_path": str(db_path) if db_path is not None else str(DB_PATH),
        "repo_root": repo_root if repo_root is not None else ELAI_ROOT,
        "workflow_root": workflow_root if workflow_root is not None else ELAI_ROOT,
        "artifact_root": artifact_root if artifact_root is not None else (CONDUCTOR / "artifacts"),
        "worktree_root": worktree_root if worktree_root is not None else (CONDUCTOR / "elai-worktrees"),
        "allow_local_commits": allow_local_commits,
        "allow_push": allow_push,
        "block_dirty_checkout": block_dirty_checkout,
        "packs": packs,
    }
    if gate_runner is not None:
        kwargs["gate_runner"] = gate_runner
    else:
        kwargs["gate_runner"] = _ssb_residual_gate_runner
    integrator = SerializedIntegrator(**kwargs)
    return integrator.integrate_verified_task(
        task_id=task_id,
        source_worktree_path=source_worktree,
        source_base_commit=source_base_commit,
        commit_message=commit_message,
    )


def integration_push_failed(
    task_id: str,
    *,
    db_path: Path | str | None = None,
) -> bool:
    if str(CONDUCTOR) not in sys.path:
        sys.path.insert(0, str(CONDUCTOR))
    from conductor_db import ControlDB  # type: ignore

    cdb = ControlDB(str(db_path) if db_path is not None else str(DB_PATH))
    conn = cdb.connect()
    try:
        row = conn.execute(
            """
            SELECT event_type FROM events
            WHERE task_id = ?
              AND event_type IN ('INTEGRATION_PUSH_FAILED', 'INTEGRATION_PUSHED')
            ORDER BY event_seq DESC
            LIMIT 1
            """,
            (task_id,),
        ).fetchone()
        return row is not None and row["event_type"] == "INTEGRATION_PUSH_FAILED"
    finally:
        conn.close()



def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="opus-walk audit+land primitive (gates in worktree, land via SerializedIntegrator)")
    ap.add_argument("--worktree", required=True, help="ext4 worktree holding the plan's uncommitted changes")
    ap.add_argument("--plan-id", required=True)
    ap.add_argument("--files", required=True, help="comma list of changed files, repo-relative (.elai_cc/...)")
    ap.add_argument("--gate-cmd", action="append", default=[],
                    help="repeatable; run with cwd=<worktree>/.elai_cc, PATH includes ~/.cargo/bin; non-zero = BLOCK")
    ap.add_argument("--gates-only", action="store_true", help="run the audit leg only, land nothing")
    ap.add_argument("--no-verifier", action="store_true",
                    help="opt out of the P1.5 fresh pre-land verifier (ON by default, D-8)")
    ap.add_argument("--plan-spec", default=None,
                    help="path to the plan spec the verifier judges the diff against; absent = diff+gates only")
    ap.add_argument("--message", default=None, help="contributor-voice commit message (required to land)")
    ap.add_argument("--push", action="store_true", help="allow push origin after SerializedIntegrator merge")
    ap.add_argument("--task-class", default="unclassified",
                    help="trust-ledger task class (D-10); pass/fail recorded at the terminal verdict")
    ap.add_argument("--task-id", default=None,
                    help="optional conductor task_id; on VERIFIER_UNAVAILABLE park, set "
                         "ControlDB status WAITING_FOR_VERIFIER so other READY tasks still lease")
    ap.add_argument("--codex-reap-task", default=None,
                    help="K-27 B4 belt-and-suspenders: a codex worktree basename/marker to pkill on "
                         "codex-vm BEFORE scanning, so no lingering codex process is still writing the "
                         "worktree while we read/land it (cross-distro reap propagates in this WSL "
                         "build, verified; this is the second line of defense).")
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args(argv)

    # K-27 B4: reap any lingering codex process against this worktree on codex-vm before we scan, so
    # a read-then-still-being-written race cannot land a half-written diff. Best-effort, fail-open on
    # the reap itself (the kill propagation is already verified; this only closes the residual window).
    if args.codex_reap_task:
        try:
            subprocess.run(["wsl.exe", "-d", "codex-vm", "-u", "root", "--", "bash", "-c",
                            f"pkill -f {args.codex_reap_task!r} || true"],
                           capture_output=True, text=True, timeout=30)
        except Exception as exc:  # noqa: BLE001 - reap is belt-and-suspenders, never block the land
            print(f"[opus-land] codex-vm reap non-fatal: {exc}", file=sys.stderr)

    # P1.6/P1.7 ledger modules (sibling files; the guard makes the import robust to cwd like the
    # verifier block below). trust_ledger.record fires at every terminal verdict via finish();
    # standing_goals.enroll fires once after a successful land.
    _here = Path(__file__).resolve().parent
    if str(_here) not in sys.path:
        sys.path.insert(0, str(_here))
    import trust_ledger
    import standing_goals

    result: dict = {"plan_id": args.plan_id, "gates": [], "checkers": [], "landed": False,
                    "pushed": False, "commit_sha": None, "verdict": None}

    def finish(verdict: str, code: int) -> int:
        result["verdict"] = verdict
        if verdict.startswith("LANDED"):
            # touch the F: land-trigger so the status feeder re-fires on this land (deliverable 6).
            try:
                LAND_TRIGGER.parent.mkdir(parents=True, exist_ok=True)
                LAND_TRIGGER.write_text(str(time.time()), encoding="utf-8")
            except OSError as exc:  # noqa: BLE001 - trigger is best-effort; never fail a completed land
                print(f"[opus-land] land-trigger touch failed (non-fatal): {exc}", file=sys.stderr)
            # BURN-U2: clear the park ledger row so pending-lands no longer only grows.
            # LANDED and LANDED_PUSH_FAILED both mean the code is on main; remove either way.
            try:
                n = remove_pending_land_on_success(args.plan_id, worktree=str(args.worktree or ""))
                if n:
                    result["pending_land_removed"] = n
            except Exception as exc:  # noqa: BLE001 - ledger cleanup must never un-land a land
                print(f"[opus-land] pending_lands.remove failed (non-fatal): {exc}", file=sys.stderr)
        quality = _quality_of(verdict)
        if quality is not None:
            try:
                result["trust_tier"] = trust_ledger.record(args.task_class, quality)
            except Exception as exc:  # noqa: BLE001 - telemetry, never block the verdict
                print(f"[opus-land] trust-ledger record failed (non-fatal): {exc}", file=sys.stderr)
        print(json.dumps(result, indent=2) if args.json else f"STATUS: {verdict}")
        if not args.json:
            print(json.dumps(result))
        return code

    worktree = Path(args.worktree).resolve()
    if not (worktree / ".git").exists():
        return finish("CONFIG_ERROR not a git worktree: " + str(worktree), 2)
    files = [f.strip() for f in args.files.split(",") if f.strip()]
    if not files:
        return finish("CONFIG_ERROR empty --files", 2)

    # ---- scope guard (fail-closed): every changed path in the worktree under .elai_cc/ ----
    changed = _changed_paths(worktree)
    if not changed:
        # idempotent-resume: a crash after the worktree commit leaves the
        # worktree CLEAN but one in-scope commit AHEAD of main; the changeset is then the committed
        # diff (main..wt), not the now-empty uncommitted status. Fall back to it so the land resumes.
        _mh = _run(["git", "rev-parse", "HEAD"], ELAI_ROOT, timeout=60).stdout.strip()
        _wh = _run(["git", "rev-parse", "HEAD"], worktree, timeout=60).stdout.strip()
        if _wh != _mh and _run(["git", "merge-base", "--is-ancestor", _mh, _wh], ELAI_ROOT, timeout=60).returncode == 0:
            changed = _run(["git", "diff", "--name-only", _mh, _wh], ELAI_ROOT, timeout=60).stdout.split()
    if not changed:
        return finish("REFUSED worktree has no changes", 1)
    allowed_set = set(files)
    out_of_scope = [p for p in changed if not p.startswith(SCOPE_PREFIX)]
    if out_of_scope:
        result["out_of_scope"] = out_of_scope
        return finish("REFUSED out-of-scope paths in worktree", 1)
    # K-27 B1 (redteam): every ACTUALLY-changed path in the worktree must be in the declared
    # changeset (--files). A worker -- especially the paid codex lane that can be prompt-injected --
    # that edits an extra in-.elai_cc file the controller did NOT declare is REFUSED. The declared
    # set is thus a STRICT UPPER BOUND on what can land, and (below) the frozen checkers run over the
    # ACTUAL changed set, not the caller's --files, so an undeclared change can never skip the
    # checkers by being absent from --files. Closes the "injected extra file lands ungated" hole.
    undeclared = [p for p in changed if p not in allowed_set]
    if undeclared:
        result["undeclared_changes"] = undeclared
        return finish("REFUSED worktree changed paths not in the declared changeset (--files)", 1)
    result["changed"] = changed

    # ---- copy the frozen checkers in (R8: canonical source, sha256 recorded, never edited) ----
    wt_tools = worktree / ".claude" / "tools"
    wt_tools.mkdir(parents=True, exist_ok=True)
    for name in CHECKERS:
        src = CANON_TOOLS / name
        if not src.is_file():
            return finish(f"CONFIG_ERROR canonical checker missing: {src}", 2)
        shutil.copy2(src, wt_tools / name)
        result["checkers"].append({"tool": name, "canonical_sha256": _sha256(src)})

    env = dict(os.environ)
    env["PATH"] = f"{CARGO_BIN}:{env.get('PATH', '')}"
    if depot_cargo.offload_mode() == "local":
        env.setdefault("CARGO_TARGET_DIR", str(Path.home() / ".cache" / "cargo-target-elai"))
    else:
        env.pop("CARGO_TARGET_DIR", None)

    # ---- gate leg: caller-supplied commands (cargo check/test ...), cwd = worktree/.elai_cc ----
    entries, gate_error = run_gate_leg(args.gate_cmd, worktree / ".elai_cc", env, plan_id=args.plan_id)
    result["gates"].extend(entries)
    if gate_error:
        entry = entries[-1] if entries else {"cmd": "", "rc": None}
        if entry.get("timeout"):
            _emit("OPUS_LAND_GATE_FAIL", {"plan_id": args.plan_id, **entry})
        else:
            _emit("OPUS_LAND_GATE_FAIL", {"plan_id": args.plan_id, "cmd": entry.get("cmd"),
                                          "rc": entry.get("rc"), "tail": entry.get("tail", "")[-400:],
                                          "provider": entry.get("provider"),
                                          "depot_id": entry.get("depot_id")})
        return finish(gate_error, 1)

    # ---- frozen checker leg: run the copied-in tools over the ACTUAL changed set (K-27 B1) ----
    # NOT the caller's --files: enforcing changed == a subset of --files above means changed is the
    # real, complete diff, and running the checkers over `changed` guarantees every path that will
    # land is fed through both frozen checkers (the injected-extra-file class cannot slip past).
    # Skip deleted/renamed-away paths for production_caller (needs extant sources).
    # Deletions remain in `changed` for scope + commit; maintainability still sees them via --base HEAD.
    rs_files = [f for f in changed if f.endswith(".rs") and (worktree / f).is_file()]
    maint_files = [f for f in changed if f.endswith(MAINTAINABILITY_SOURCE_EXTS)]
    checker_runs = []
    if rs_files:
        checker_runs.append(["python3", str(wt_tools / "production_caller_check.py"),
                             "--files", ",".join(rs_files), "--json"])
    if maint_files:
        checker_runs.append(["python3", str(wt_tools / "maintainability_check.py"),
                             "--files", ",".join(maint_files), "--base", "HEAD", "--json"])
    for cmd in checker_runs:
        proc = _run(cmd, worktree, timeout=600)
        entry = {"cmd": " ".join(cmd[1:2] + cmd[2:3]), "rc": proc.returncode,
                 "tail": (proc.stdout + proc.stderr)[-1500:]}
        result["gates"].append(entry)
        if proc.returncode == 2:
            return finish("CHECKER_CONFIG_ERROR (fail-closed): " + entry["tail"][-300:], 1)
        if proc.returncode != 0:
            _emit("OPUS_LAND_GATE_FAIL", {"plan_id": args.plan_id, "cmd": entry["cmd"], "rc": proc.returncode})
            return finish(f"CHECKER_BLOCK rc={proc.returncode}: {entry['cmd']}", 1)

    _emit("OPUS_LAND_GATES_PASS", {"plan_id": args.plan_id, "gates": len(result["gates"])})
    if args.gates_only:
        return finish("GATES_PASS", 0)

    # ---- land leg ----
    if not args.message:
        return finish("CONFIG_ERROR --message required to land", 2)

    # ---- P1.5 fresh pre-land verifier (D-8, ON by default) ----
    # A FRESH base-cc session that never saw the build conversation judges the plan spec + final
    # diff + gate outputs and answers APPROVE/REJECT. Fail-closed: reject OR spawn/transport failure
    # OR unparseable answer REFUSES the land. --no-verifier opts out.
    # --no-verifier is an OWNER escape hatch, not an agent one (wiring-enforcement R8:
    # "a run that can rewrite its own evaluator always writes itself to PASS"). The
    # orchestrator composes this command line itself, so an unguarded flag makes the
    # fail-closed verifier optional in practice -- and it was used exactly that way:
    # J-64 (2026-07-25, 730 insertions + a new dependency) went GATES_PASS -> MERGED in
    # ONE second, unreviewed, and the E-38 evidence log records the same bypass after a
    # VERIFIER_UNAVAILABLE flake. Honoring the flag requires a sentinel FILE the owner
    # creates with a JSON body (timestamp + scope.plan_id matching this land). Bare
    # is_file / empty body / wrong plan_id are refused; no env-var grant path exists.
    # Grant and refuse both emit visible events (OPTOUT_GRANTED / OPTOUT_REFUSED).
    verifier_opt_out = False
    if args.no_verifier:
        granted, reason, parse_detail = parse_allow_no_verifier(
            ALLOW_NO_VERIFIER_SENTINEL, args.plan_id)
        verifier_opt_out = granted
        opt_payload = {
            "plan_id": args.plan_id,
            "sentinel": str(ALLOW_NO_VERIFIER_SENTINEL),
            "reason": reason,
        }
        for key in ("timestamp", "scope", "error", "missing", "scope_plan_id",
                    "signer", "owner_reason"):
            if key in parse_detail:
                opt_payload[key] = parse_detail[key]
        if granted:
            print("[opus-land] --no-verifier GRANTED via owner sentinel "
                  f"{ALLOW_NO_VERIFIER_SENTINEL} scope.plan_id={args.plan_id}",
                  file=sys.stderr)
            _emit("OPUS_LAND_VERIFIER_OPTOUT_GRANTED", opt_payload)
        else:
            print("[opus-land] --no-verifier IGNORED: owner sentinel "
                  f"{ALLOW_NO_VERIFIER_SENTINEL} reason={reason}; running the verifier",
                  file=sys.stderr)
            _emit("OPUS_LAND_VERIFIER_OPTOUT_REFUSED", opt_payload)
    if not verifier_opt_out:
        _here = Path(__file__).resolve().parent
        if str(_here) not in sys.path:
            sys.path.insert(0, str(_here))
        import opus_preland_verify  # lazy: keeps the supervisor import chain off the --gates-only path
        spec_text = None
        if args.plan_spec:
            try:
                spec_text = Path(args.plan_spec).read_text(encoding="utf-8")
            except OSError as exc:
                print(f"[opus-land] plan-spec unreadable ({exc}); verifier judges diff+gates only",
                      file=sys.stderr)
        diff_text = _collect_diff(worktree, changed)
        # The verifier owns the complete two-attempt transport budget. This land path makes one
        # call only, so no retry can occur above or below the named owner.
        verdict, vdetail = opus_preland_verify.fresh_preland_verify(
            spec_text, diff_text, json.dumps(result["gates"])
        )
        result["verifier"] = vdetail
        if verdict == "reject":
            _emit("OPUS_LAND_VERIFIER_REJECTED",
                  {"plan_id": args.plan_id, "grounds": vdetail.get("grounds")})
            return finish("VERIFIER_REJECTED", 1)
        if verdict == "unavailable":
            # A-EV-QUEUE-BLOCK / A-46: hard queue-and-block park (typed WAITING_FOR_VERIFIER).
            # Never pass, never DONE/LANDED. Exit stays non-zero so callers cannot treat this
            # as land success; durable park lets the lane continue other work and resume by
            # re-running land when the verifier seat returns (pending_lands reason + plan-status).
            park = park_verifier_unavailable(
                args.plan_id,
                str(worktree),
                detail=vdetail,
                attempt=vdetail.get("attempt_count", 2),
                task_id=args.task_id,
            )
            result["park"] = park
            result["parked"] = True
            result["waiting_status"] = WAITING_FOR_VERIFIER
            result["landed"] = False
            _emit("OPUS_LAND_VERIFIER_PARKED",
                  {"plan_id": args.plan_id, "error": vdetail.get("error"),
                   "park": True, "waiting_status": WAITING_FOR_VERIFIER,
                   "attempt": vdetail.get("attempt_count", 2), "worktree": str(worktree)})
            return finish("VERIFIER_UNAVAILABLE_PARKED", 1)
        _emit("OPUS_LAND_VERIFIER_APPROVED",
              {"plan_id": args.plan_id, "grounds": vdetail.get("grounds")})

    # /mnt/f preconditions: HEAD is the worktree's base; no dirty overlap with the changeset
    wt_head = _run(["git", "rev-parse", "HEAD"], worktree, timeout=60).stdout.strip()
    main_head = _run(["git", "rev-parse", "HEAD"], ELAI_ROOT, timeout=60).stdout.strip()
    # idempotent-resume: a crash AFTER the worktree commit (e.g. the 9p phantom
    # stat wall) leaves the worktree one in-scope commit AHEAD of main. That is not "main moved";
    # main is still an ancestor of the worktree HEAD, so the already-made commit feeds the integrator
    # as source. Detect it and skip the re-commit rather than REFUSE (re-committing a clean worktree fails).
    already_committed = (
        wt_head != main_head
        and _run(["git", "merge-base", "--is-ancestor", main_head, wt_head], ELAI_ROOT, timeout=60).returncode == 0
    )
    if wt_head != main_head and not already_committed:
        result["wt_head"] = wt_head
        result["main_head"] = main_head
        return finish("REFUSED main moved since the worktree base (rebase the worktree first)", 1)
    # refresh the index stat cache first: on the Windows mount a file rewritten with identical
    # content reads as dirty in `git status` (racy-clean mtime phantom) though it carries no
    # uncommitted work; the refresh also lets the integrator dirty-check see the clean state.
    _run(["git", "update-index", "-q", "--refresh"], ELAI_ROOT, timeout=60)
    dirty_main = set(_changed_paths(ELAI_ROOT))
    overlap = dirty_main.intersection(changed)
    # a path only genuinely overlaps if it has REAL content changes on /mnt/f; a content-clean
    # (phantom) entry cannot be clobbered by the integrator merge, so it is not a true overlap.
    real_overlap = [p for p in sorted(overlap)
                    if _run(["git", "diff", "--quiet", "--", p], ELAI_ROOT, timeout=60).returncode != 0]
    if real_overlap:
        result["dirty_overlap"] = real_overlap
        return finish("REFUSED /mnt/f dirty files overlap the changeset", 1)

    if already_committed:
        # crashed-land resume: the changeset is already committed on the worktree branch. Re-verify
        # the committed diff (main..wt) is entirely in-scope (same scope guard as the fresh-commit
        # path), then feed the existing commit into SerializedIntegrator without re-staging/re-committing.
        committed = _run(["git", "diff", "--name-only", main_head, wt_head], ELAI_ROOT, timeout=60).stdout.split()
        bad = [p for p in committed if not p.startswith(SCOPE_PREFIX) or p not in allowed_set]
        if bad:
            result["committed_out_of_scope"] = bad
            return finish("REFUSED already-committed worktree touches undeclared/out-of-scope paths", 1)
        sha = wt_head
    else:
        # commit on the worktree branch: stage ONLY the declared changeset paths (never add -A)
        for f in changed:
            proc = _run(["git", "add", "--", f], worktree, timeout=60)
            if proc.returncode != 0:
                return finish("LAND_FAIL git add " + f + ": " + proc.stderr[-200:], 1)
        staged = _run(["git", "diff", "--cached", "--name-only"], worktree, timeout=60).stdout.split()
        bad = [p for p in staged if not p.startswith(SCOPE_PREFIX)]
        if bad:
            result["staged_out_of_scope"] = bad
            return finish("REFUSED staged path outside .elai_cc", 1)
        # per-invocation identity (fork B, 2026-07-10): the distro git has no user.name/email and the
        # land must not persist config into the owner's repo; identity matches the repo's own commits.
        proc = _run(["git", "-c", "user.name=ELAI contributors",
                     "-c", "user.email=contributors@example.invalid",
                     "commit", "-m", args.message], worktree, timeout=120)
        if proc.returncode != 0:
            return finish("LAND_FAIL commit: " + (proc.stderr or proc.stdout)[-300:], 1)
        sha = _run(["git", "rev-parse", "HEAD"], worktree, timeout=60).stdout.strip()
    result["commit_sha"] = sha

    # clear phantom (content==HEAD) dirty entries on the changeset paths before the integrator
    # takes the dirty-checkout lock. On the 9p /mnt/f mount both `--refresh` and `--really-refresh`
    # proved insufficient: git re-stats per process and re-flags a content-identical file, so a
    # dirty checkout blocks SerializedIntegrator. Checking the file out of HEAD rewrites identical
    # bytes and resets its stat in one process, deterministically clearing the phantom. GUARDED: a
    # path is touched only when it is BOTH reported dirty AND content==HEAD right now, so a REAL owner
    # change is never clobbered -- it stays flagged, and an in-scope real change was already REFUSED
    # above as real_overlap. This is a content no-op, not a reset of real work.
    for p in changed:
        if (_run(["git", "status", "--porcelain", "--", p], ELAI_ROOT, timeout=60).stdout.strip()
                and _run(["git", "diff", "--quiet", "HEAD", "--", p], ELAI_ROOT, timeout=60).returncode == 0):
            _run(["git", "checkout", "HEAD", "--", p], ELAI_ROOT, timeout=60)

    # A-CTRL-OPUS-LAND: sole production merge is SerializedIntegrator (integrations single-flight,
    # candidate + post-merge gates, post-merge revert). No direct `git merge --ff-only` / push here.
    if str(CONDUCTOR) not in sys.path:
        sys.path.insert(0, str(CONDUCTOR))
    from integrator import IntegrationBusy, IntegrationRejected  # type: ignore

    try:
        land_task_id = ensure_verified_task_for_land(
            plan_id=args.plan_id,
            task_id=args.task_id,
            base_commit=main_head,
            allowed_write_set=list(changed),
        )
    except Exception as exc:  # noqa: BLE001 - surface task-row failure fail-closed
        return finish(f"LAND_FAIL no task row / not VERIFIED: {exc}", 1)
    result["task_id"] = land_task_id

    try:
        integration = land_via_serialized_integrator(
            task_id=land_task_id,
            source_worktree=worktree,
            source_base_commit=main_head,
            commit_message=args.message,
            allow_push=bool(args.push),
        )
    except IntegrationBusy as exc:
        result["integration_busy"] = str(exc)
        _emit("OPUS_LAND_INTEGRATION_BUSY",
              {"plan_id": args.plan_id, "task_id": land_task_id, "error": str(exc)[:300]})
        return finish(f"INTEGRATION_BUSY {exc}", 1)
    except IntegrationRejected as exc:
        result["integration_rejected"] = str(exc)
        return finish(f"LAND_FAIL integration rejected: {exc}", 1)
    except Exception as exc:  # noqa: BLE001 - fail closed on integrator infra errors
        return finish(f"LAND_FAIL integrator: {exc}", 1)

    result["integration"] = integration.to_json_dict()
    if integration.status != "DONE":
        _emit("OPUS_LAND_INTEGRATION_FAILED",
              {"plan_id": args.plan_id, "task_id": land_task_id,
               "status": integration.status, "reason": integration.reason,
               "post_gate_verdict": integration.post_gate_verdict})
        return finish(
            f"LAND_FAIL integration status={integration.status} reason={integration.reason}",
            1,
        )

    merged_sha = _run(["git", "rev-parse", "HEAD"], ELAI_ROOT, timeout=60).stdout.strip()
    result["commit_sha"] = merged_sha or sha
    result["landed"] = True
    _emit("OPUS_LAND_MERGED",
          {"plan_id": args.plan_id, "commit_sha": result["commit_sha"],
           "task_id": land_task_id, "via": "SerializedIntegrator",
           "integration_id": integration.integration_id})
    _release_keep_for_land_markers(args.plan_id)

    # P1.7: enroll the drained plan's OWN gate commands as a standing goal (the daily --verify timer
    # re-runs them to catch a later plan silently regressing this one). Fires on merge success so
    # both LANDED and LANDED_PUSH_FAILED enroll (the code is on main in both; push is transport).
    # Non-fatal: an enroll failure must never un-land a landed plan. Skipped when there are no gate
    # commands to re-run (a goal with no predicates is meaningless).
    if args.gate_cmd:
        try:
            standing_goals.enroll(args.plan_id, list(args.gate_cmd),
                                  meta={"commit_sha": result["commit_sha"]})
            result["enrolled"] = True
        except Exception as exc:  # noqa: BLE001 - enroll must never break a completed land
            result["enroll_error"] = str(exc)
            print(f"[opus-land] standing-goal enroll failed (non-fatal): {exc}", file=sys.stderr)

    if args.push:
        if integration_push_failed(land_task_id):
            result["pushed"] = False
            result["push_error"] = "INTEGRATION_PUSH_FAILED"
            _emit("OPUS_LAND_PUSH_FAIL",
                  {"plan_id": args.plan_id, "commit_sha": result["commit_sha"],
                   "task_id": land_task_id})
            return finish(
                "LANDED_PUSH_FAILED (commit is on local main; fix github push auth, then git push origin main)",
                1,
            )
        result["pushed"] = True
        _emit("OPUS_LAND_PUSHED",
              {"plan_id": args.plan_id, "commit_sha": result["commit_sha"],
               "task_id": land_task_id})

    return finish("LANDED", 0)


if __name__ == "__main__":
    sys.exit(main())
