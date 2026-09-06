#!/usr/bin/env python3
"""glm_supervisor.py - the ZCode GLM controller for the shared backlog walk.

One ZCode GLM-5.2 session IS the orchestration loop: it dispatches the same FREE
workers via the devin bridge, gates with the frozen ELAI checkers, and drives
codex as the e2e tester (per the sequential-glm-orchestrator skill). This
supervisor preserves the production Opus walk's resume chain:

  spawn ONE headless ZCode GLM turn -> it drains a bounded chunk + writes walk-state
  + a handoff entry -> exits -> if not stopped and the backlog is not drained,
  spawn a FRESH ZCode GLM session that resumes from walk-state -> repeat.

Never two controllers concurrently: this lane and the Opus lane use the exact
same walk-state file and lock.  A GLM stop cancels only its own ZCode child.

Grounded seams: zcode_adapter (installed app-server NDJSON), events.append_event
(canvas + audit trail), and opus_walk/walk_state.py (the shared resume spine).
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

_HERE = Path(__file__).resolve().parent            # ~/teikoku/conductor/opus_walk
CONDUCTOR = _HERE.parent                            # ~/teikoku/conductor
TEIKOKU_ROOT = CONDUCTOR.parent                     # ~/teikoku
SHARED_WALK = CONDUCTOR / "opus_walk"
sys.path.insert(0, str(CONDUCTOR))
sys.path.insert(0, str(SHARED_WALK))
sys.path.insert(0, str(_HERE))

import walk_state                                   # noqa: E402  (the resume spine)
import events                                       # noqa: E402  (canvas + audit emit)
import quorum_triage                                # noqa: E402  (P1.4 blind free-model cost gate)
import trust_ledger                                 # noqa: E402  (P1.6 per-class autonomy-tier read)
import plan_selector                                # noqa: E402  (free-gated auto-seed from ELAI open-gaps)
import disk_hygiene                                 # noqa: E402  (F: free-space guard + between-plan reap)
import zcode_adapter                                # noqa: E402  (headless ZCode app-server protocol)

# --- fixed wiring (no magic numbers scattered downstream) --------------------
WALK_STATE = SHARED_WALK / "walk-state.json"
SKILL_PATH = _HERE / "skills" / "sequential-glm-orchestrator" / "SKILL.md"
STOP_SENTINEL = CONDUCTOR / "GLM_WALK_STOP"
DB_PATH = CONDUCTOR / "state" / "control.db"
GLM_MODEL = "GLM-5.2"
GLM_PROVIDER = "builtin:zai-coding-plan"
GLM_CONTEXT_WINDOW = 1_000_000
# canvas / audit: the coordination plan_id. NON-backlog pattern (not B-N / phase-X),
# so the scope=mine /orchestration filter KEEPS it (it is one of the conductor's own runs).
CANVAS_PLAN_ID = "teikoku-opus-walk"
# Per-turn dispatch budget handed to the GLM controller (it stops + hands off when it trips this).
DEFAULT_DISPATCH_BUDGET = 30
# 3h wall ceiling per ZCode GLM turn (a wedged turn must not hang the chain forever).
GLM_TURN_TIMEOUT_S = 3 * 60 * 60
# Fast-fail storm guard (2026-07-07): a turn that exits NON-ZERO almost immediately did no real work
# (auth 401 at startup / bad CLI flag / MCP config error). Respawning into that is a spin/money hazard
# (observed: 10 turns in ~76s on an EXPIRED claude-elai token, each rc=1 in ~7s at 0 tokens). N
# consecutive fast-fails HALT the chain with a diagnostic instead of storming; a real working turn
# (rc=0, or a turn that ran long enough to do work) resets the counter.
FAST_FAIL_SECONDS = 45
FAST_FAIL_LIMIT = 3
# brief settle between turns; the STOP sentinel is re-checked each iteration.
INTER_TURN_SLEEP_S = 4
# Devin-bridge health endpoint. A walk with no live workers just burns GLM quota
# for nothing, so the pre-spawn gate refuses when this probe fails. 30s not 5s: the proxy
# delays /health past 5s while a gateway stream is in flight (the selector's own quorum votes
# stream right before this probe -- observed NO_SPAWN false-refusal 2026-07-17); a genuinely
# down bridge still fails instantly with connection-refused, so the longer ceiling only
# tolerates the busy case, it never slows the down case.
BRIDGE_HEALTH_URL = "http://127.0.0.1:8810/health"
BRIDGE_HEALTH_TIMEOUT_S = 30
# Relax (not bypass) the reflect_gate on the orchestrator: mutation-priority = reasoning required
# before MUTATIONS (Edit/Write/Bash -- so still before every worker dispatch + checker run, the real
# decisions) but reads/greps flow (the gate's own "lower friction for trusted runs" mode). This is
# NOT the workers' full TEIKOKU_AUTONOMOUS bypass -- scope_ack + read_full + every SAFETY gate stay
# fully ON for the GLM controller. The gate reads a per-session state file keyed by CLAUDE_CODE_SESSION_ID.
REFLECT_STATE_DIR = Path(os.path.expanduser("~/.teikoku/state"))
GLM_REFLECT_MODE = "mutation-priority"
# P1.4 blind free-model quorum cost-gate. Even after _should_spawn proves real work + a live bridge,
# poll a small free roster for a cheap second opinion; a quorum that agrees there is no actionable
# work skips the ZCode GLM boot. Cost gate (D-5), fail-open: <2 parseable votes wakes anyway.
# plan-status/ sits beside the walk-state file (same directory walk_state.is_plan_done derives).
PLAN_STATUS_DIR = SHARED_WALK / "plan-status"
QUORUM_ENV = "GLM_WALK_QUORUM"  # set to 'off' to disable the gate (default on)
# Free-gated auto-seed (2026-07-12, owner choice): instead of HALTING on a drained queue, the
# supervisor refills itself from ELAI's open-gaps deck via plan_selector (free pre-filter + free
# quorum, NO paid model). Default ON; set GLM_WALK_AUTOSEED=off to restore the plain halt-on-drain.
# The per-chain cap bounds worst-case burn if a slipped-through greenfield gap keeps getting
# SET_ASIDE (each set-aside = one paid turn; the selector skips escalated ids so the pool shrinks
# monotonically, but the cap is the hard backstop).
AUTOSEED_ENV = "GLM_WALK_AUTOSEED"
AUTOSEED_MAX_PER_CHAIN = 8
# Codex-first window (2026-07-12): the paid codex gpt-5.6-sol credits (account-a) EXPIRE 2026-07-15,
# and unused credits are wasted, so while this sentinel FILE is present the boot prompt tells the GLM controller
# to route EVERY executable EXECUTE unit to codex (not just hard/greenfield ones) -- the free swe lane
# drops to recon/fallback only. A sentinel FILE (not env) so it is checked per-turn by the running
# supervisor and the owner can toggle it (touch/rm) without a restart. Remove after the credits expire.
CODEX_FIRST_SENTINEL = CONDUCTOR / "CODEX_FIRST"
# Owner directive (2026-07-17): a free-form owner-authored mandate appended verbatim to every boot
# prompt while the file is present (same toggle shape as CODEX_FIRST: the owner edits/removes the
# FILE, no restart needed). This is the generic seam for standing owner rulings that must reach every
# headless GLM turn (e.g. the LOCAL-LANE GO). Content is capped so a runaway file cannot blow the
# prompt budget; the cap truncates with an explicit marker rather than failing the turn.
OWNER_DIRECTIVE_PATH = CONDUCTOR / "OWNER_DIRECTIVE.md"
OWNER_DIRECTIVE_MAX_CHARS = 8000
# T6-entry10 auto-restart wrapper contract: run_supervisor stamps its TERMINAL halt class + a
# made_progress flag here on every exit, so glm_walk_chain.py (the driver-launched restart loop) can
# decide whether to auto-restart the chain after a backoff. Beside the walk-state file so a test that
# patches WALK_STATE to a temp dir redirects this write there too (never the live file).
CHAIN_RESULT_NAME = "last-chain-result.json"
# Terminal halt classes stamped into the chain-result (the restart wrapper keys its decision off these).
HALT_STOPPED = "STOPPED"                # STOP sentinel -> never restart (sentinel always wins)
HALT_CHAIN_NO_WORK = "CHAIN_NO_WORK"    # queue drained / all-DONE, autoseed found nothing -> clean no-work
HALT_NO_SPAWN = "NO_SPAWN"              # pre-spawn gate refused (no actionable work) -> clean no-work
HALT_QUORUM_QUIET = "QUORUM_QUIET"      # free quorum saw no work -> clean no-work
HALT_DISK_LOW = "DISK_LOW"              # F: free below floor -> restart once, twice-consecutive halts
HALT_ERROR_WALKSTATE = "ERROR_WALKSTATE"
HALT_ERROR_TIMEOUT = "ERROR_TIMEOUT"
HALT_FAST_FAIL_STORM = "FAST_FAIL_STORM"  # auth/401/flag storm -> never restart (operator halt)


def _chain_result_path() -> Path:
    """Controller-local halt stamp; the shared walk-state is never duplicated."""
    return _HERE / CHAIN_RESULT_NAME


def _write_chain_result(halt_class: str, turns: int, made_progress: bool,
                        detail: dict | None = None) -> None:
    """Stamp THIS chain's terminal halt class + progress flag for the auto-restart wrapper. Fail-soft:
    a telemetry write must never change the supervisor's exit code or behaviour."""
    payload = {
        "halt_class": halt_class,
        "turns": turns,
        "made_progress": bool(made_progress),
        "ts": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "detail": detail or {},
    }
    try:
        p = _chain_result_path()
        tmp = p.with_name(f"{p.name}.{os.getpid()}.tmp")
        tmp.write_text(json.dumps(payload, indent=2), encoding="utf-8")
        os.replace(tmp, p)
    except OSError as exc:  # noqa: BLE001 - telemetry only, never break the chain exit
        print(f"[glm-supervisor] WARN chain-result write failed: {exc}", file=sys.stderr, flush=True)


def _finish(halt_class: str, turns: int, made_progress: bool, rc: int,
            detail: dict | None = None) -> int:
    """Stamp the chain result and return the supervisor exit code. Single choke point so every
    run_supervisor exit records its halt class for the restart wrapper."""
    _write_chain_result(halt_class, turns, made_progress, detail)
    return rc


def _set_reflect_mode(session_id: str) -> None:
    """Write the per-session reflect_gate mode file so the GLM turn runs at mutation-priority
    (relaxed reads, reasoning still gated before every mutation). Best-effort: a write failure just
    leaves the gate at its every-action default, never blocks the spawn."""
    try:
        REFLECT_STATE_DIR.mkdir(parents=True, exist_ok=True)
        (REFLECT_STATE_DIR / f"{session_id}.reflect_mode").write_text(
            GLM_REFLECT_MODE, encoding="utf-8")
    except OSError as exc:  # noqa: BLE001 - relaxation is best-effort, never fatal
        print(f"[glm-supervisor] WARN could not set reflect_mode for {session_id}: {exc}",
              file=sys.stderr, flush=True)


def _emit(event_type: str, payload: dict) -> None:
    """Append a lifecycle event to control.db (the canvas wake/event surface +
    the durable audit trail). Fail-soft: a telemetry write must never kill the
    chain, but the failure is surfaced on stderr, never swallowed silently."""
    try:
        events.append_event(
            str(DB_PATH), event_type, task_id=CANVAS_PLAN_ID,
            payload=payload, project_id="elai", session_id=CANVAS_PLAN_ID,
        )
    except Exception as exc:  # noqa: BLE001 - telemetry is best-effort by design
        print(f"[glm-supervisor] WARN emit {event_type} failed: "
              f"{type(exc).__name__}: {exc}", file=sys.stderr, flush=True)


def _backlog_drained() -> bool:
    """True when the walk has nothing left to do: the queue is empty AND the
    cursor is not pointing at an in-flight plan. Read-only; a malformed state is
    NOT treated as drained (that would silently end the chain) - it re-raises so
    the supervisor stops loudly."""
    state = walk_state.load(str(WALK_STATE))
    queue = state.get("queue") or []
    cursor = state.get("cursor") or {}
    cursor_plan = cursor.get("plan_id")
    return not queue and not cursor_plan


def _no_actionable_work(path: str = str(WALK_STATE)) -> bool:
    """True when the chain has nothing to DO: every queued plan is already DONE (plan-status row) or
    the queue is empty, AND the cursor is unset (drained), DONE, or points at a plan already in the
    escalation_queue (SET_ASIDE). Respawning a GLM controller into THIS state is pure waste -- it burned 6
    no-work paid turns on 2026-07-07 (each fresh controller booted, saw the empty queue + escalated I-33, and
    stopped for nothing) and re-boots forever on a DONE plan still left queued/cursored (the H-28
    respawn-burn). This driver-side guard halts the chain instead; only an owner seed (new plan into
    queue) or an escalation-clear re-arms the walk. A malformed state re-raises (never silently
    'no work')."""
    state = walk_state.load(path)
    if any(not walk_state.is_plan_done(p, path) for p in (state.get("queue") or [])):
        return False  # at least one queued plan is not yet DONE -> real work
    cursor = state.get("cursor") or {}
    cursor_plan = cursor.get("plan_id")
    if not cursor_plan:
        return True  # genuinely drained
    if walk_state.is_plan_done(cursor_plan, path):
        return True  # cursor points only at an already-DONE plan -> nothing to run
    stage = (cursor.get("stage") or "").lower()
    if stage in ("done", "drained", "set_aside", "escalated", "complete", "completed"):
        return True  # settled-stage cursor is not work (see _cursor_actionable overnight fix)
    escalated = {e.get("plan_id") for e in (state.get("escalation_queue") or [])}
    return cursor_plan in escalated  # cursor points only at a set-aside plan -> nothing to run


def _reconcile_done_plans(path: str = str(WALK_STATE)) -> list:
    """Self-heal the live walk before the no-work test: any queued or cursored plan whose plan-status
    row is DONE is dequeued via walk_state.complete_plan (a GLM turn may have gated+committed a plan
    but exited before its own bookkeeping ran, leaving a DONE plan still queued/cursored -> the H-28
    respawn-burn). Returns the plan_ids reconciled. A malformed state re-raises through walk_state.load
    so the supervisor halts loudly rather than silently treating the walk as drained."""
    state = walk_state.load(path)
    candidates = list(state.get("queue") or [])
    cursor_plan = (state.get("cursor") or {}).get("plan_id")
    if cursor_plan and cursor_plan not in candidates:
        candidates.append(cursor_plan)
    reconciled = []
    for plan_id in candidates:
        if walk_state.is_plan_done(plan_id, path):
            walk_state.complete_plan(plan_id, path)
            reconciled.append(plan_id)
    return reconciled


def _bridge_healthy(url: str = BRIDGE_HEALTH_URL, timeout_s: float = BRIDGE_HEALTH_TIMEOUT_S) -> bool:
    """True when the devin bridge answers /health with a 2xx inside timeout_s. Any connection error,
    timeout, or non-2xx status is a dead bridge -> False. Injected as the default probe into
    _should_spawn so unit tests stub it and never touch the network."""
    import urllib.request
    import urllib.error
    try:
        with urllib.request.urlopen(url, timeout=timeout_s) as resp:
            status = getattr(resp, "status", None) or resp.getcode()
            return 200 <= status < 300
    except (urllib.error.URLError, OSError):
        return False


def _cursor_actionable(state: dict) -> bool:
    """True when the cursor points at a plan that is still runnable: set, its stage is not a terminal
    settled stage (done/drained/set_aside/escalated), and it is not in the escalation_queue.

    2026-07-13 overnight fix: heads write stage=drained/set_aside on recon closeouts; those are
    settled (not work). Treating only stage==done as terminal left a drained cursor as actionable,
    which blocked autoseed refill and re-spawned GLM turns on the same already-dispositioned id
    (observed: E-51 re-PICK after drained disposition because cursor.stage=drained still passed)."""
    cursor = state.get("cursor") or {}
    plan_id = cursor.get("plan_id")
    if not plan_id:
        return False
    stage = (cursor.get("stage") or "").lower()
    if stage in ("done", "drained", "set_aside", "escalated", "complete", "completed"):
        return False
    escalated = {e.get("plan_id") for e in (state.get("escalation_queue") or [])}
    return plan_id not in escalated


def _should_spawn(state: dict, probe=_bridge_healthy, reconcile=None,
                  path: str = str(WALK_STATE)) -> tuple:
    """Final pre-spawn gate (returns (ok, reason)). Refuses -- consuming NO budget and minting NO
    session -- unless ALL hold: (a) the post-reconcile queue is non-empty OR the cursor is actionable;
    (b) the GLM_WALK_STOP sentinel is absent; (c) the devin bridge health probe passes (a walk with no
    live workers burns GLM quota for nothing). probe is injectable so tests never hit the network.

    F-6 (reconcile-before-work race): when `reconcile` is supplied (run_supervisor passes
    _reconcile_done_plans), the gate re-runs the DONE-plan reconcile and re-reads state FIRST, so a
    plan that LANDED (its plan-status row flipped to DONE) but was not yet dequeued cannot slip past
    this gate's plain queue check and wake a GLM turn -- this gate is now independently DONE-aware
    rather than trusting an earlier reconcile in the loop. Left None by the in-memory unit tests (they
    pass an explicit state and want no file I/O)."""
    if STOP_SENTINEL.is_file():
        return False, "GLM_WALK_STOP sentinel present"
    if reconcile is not None:
        if reconcile(path):            # dequeued a landed-but-queued DONE plan -> re-read the truth
            state = walk_state.load(path)
    if not (bool(state.get("queue")) or _cursor_actionable(state)):
        return False, "no actionable work (empty queue; cursor unset/done/escalated)"
    if not probe():
        return False, f"devin bridge health probe failed ({BRIDGE_HEALTH_URL})"
    return True, "actionable work + bridge healthy"


def _quorum_enabled() -> bool:
    """The P1.4 quorum gate is ON unless env GLM_WALK_QUORUM=off (the injectable/skippable escape)."""
    return (os.environ.get(QUORUM_ENV) or "on").strip().lower() != "off"


def _quorum_gate(state: dict) -> tuple:
    """Build the walk signals and run the blind free-model quorum. Returns (wake, detail). A thin
    seam over quorum_triage so tests monkeypatch it (or quorum_triage.vote) to avoid the network."""
    signals = quorum_triage.build_signals(state, str(PLAN_STATUS_DIR))
    return quorum_triage.vote(signals)


def _autoseed_enabled() -> bool:
    """The free-gated auto-seed is ON unless env GLM_WALK_AUTOSEED=off (the skippable escape)."""
    return (os.environ.get(AUTOSEED_ENV) or "on").strip().lower() != "off"


def _codex_first() -> bool:
    """Codex-first window: while the CODEX_FIRST sentinel file is present, the boot prompt routes EVERY
    executable EXECUTE unit to the paid codex gpt-5.6-sol lane (not just hard/greenfield ones) so the
    account-a credits are spent before they expire 2026-07-15. Checked per turn (owner toggles the file)."""
    return CODEX_FIRST_SENTINEL.is_file()


def _owner_directive() -> str:
    """The verbatim owner-authored mandate for this turn, or '' when absent/empty/unreadable.
    Capped at OWNER_DIRECTIVE_MAX_CHARS with an explicit truncation marker (never a failed turn)."""
    try:
        text = OWNER_DIRECTIVE_PATH.read_text(encoding="utf-8").strip()
    except OSError:
        return ""
    if not text:
        return ""
    if len(text) > OWNER_DIRECTIVE_MAX_CHARS:
        text = text[:OWNER_DIRECTIVE_MAX_CHARS] + "\n[... truncated at prompt cap; read " \
               f"{OWNER_DIRECTIVE_PATH} in full for the rest]"
    return text


def _drain_superseded(turn: int) -> int:
    """Free-skip pass (deliverable 4): auto-disposition every SKIP_SUPERSEDED unit corpus_drain
    classifies -- a dark unit whose superseder is PROVEN SHIPPED (the engine's fail-closed guard). The
    residual is owned by the superseder, so the unit is dispositioned 'drained' (reason names the
    confirmed superseders) and drops out of every future pool WITHOUT spending a GLM turn.
    Fail-closed: ONLY the SKIP_SUPERSEDED action is auto-skipped (the engine already proved the
    superseder shipped); any corpus/ledger failure skips nothing. Returns the count dispositioned."""
    try:
        import corpus_drain
        index = corpus_drain.load_index()
        actions = corpus_drain.classify_all(index)
    except Exception as exc:  # noqa: BLE001 - additive free path; never break the walk
        print(f"[glm-supervisor] superseded-drain skipped (corpus unavailable: {exc})", flush=True)
        return 0
    try:
        already = walk_state.dispositioned_ids()
    except Exception:  # noqa: BLE001 - fail-safe: an unreadable ledger just means 'skip nothing yet'
        already = set()
    by_id = {u.get("id"): u for u in index.get("units", [])}
    count = 0
    for uid, action in actions.items():
        if action != corpus_drain.SKIP_SUPERSEDED or uid in already:
            continue
        supers = (by_id.get(uid) or {}).get("superseded_by") or []
        reason = f"superseded by {', '.join(supers) or '(unknown)'}, confirmed shipped"
        try:
            walk_state._emit_disposition(uid, "drained", reason)
            count += 1
        except Exception as exc:  # noqa: BLE001
            print(f"[glm-supervisor] WARN superseded-drain emit failed for {uid}: {exc}",
                  file=sys.stderr, flush=True)
    if count:
        print(f"[glm-supervisor] SUPERSEDED-DRAIN: auto-dispositioned {count} superseded unit(s) "
              f"free (no paid turn)", flush=True)
        _emit("GLM_WALK_SUPERSEDED_DRAIN", {"turns": turn, "count": count})
    return count


def _try_autoseed(turn: int) -> bool:
    """On a drained queue, try to refill from ELAI's open-gaps via plan_selector BEFORE halting. NO
    paid model is called (free pre-filter + free quorum). Returns True iff a plan was seeded (caller
    re-loops instead of halting). Fully guarded: any selector failure, a stale/unreadable status
    snapshot, or no safe pick leaves the queue empty and returns False (the caller halts exactly as
    before). Emits a canvas/audit event for every outcome so the auto-seed is observable."""
    if not _autoseed_enabled():
        return False
    # Free-skip pass FIRST: auto-disposition SKIP_SUPERSEDED units (their residual is owned by an
    # already-shipped superseder) so they leave the frontier without a paid turn, BEFORE selecting a
    # build. Dispositioning them is real progress, so even if select_next then finds no build pick we
    # re-loop once (bounded by AUTOSEED_MAX_PER_CHAIN).
    freed = _drain_superseded(turn)
    try:
        state = walk_state.load(str(WALK_STATE))
        pick = plan_selector.select_next(state, str(PLAN_STATUS_DIR),
                                         corpus_reader=plan_selector.load_corpus_order)
    except (walk_state.WalkStateError, OSError, ValueError) as exc:
        print(f"[glm-supervisor] autoseed error ({exc}); halting normally", flush=True)
        _emit("GLM_WALK_AUTOSEED_ERROR", {"turns": turn, "error": str(exc)})
        return False
    if not pick:
        if freed:  # dispositioned superseded units (progress) -> re-loop to re-evaluate the frontier
            _emit("GLM_WALK_AUTOSEED_NONE", {"turns": turn, "superseded_drained": freed})
            return True
        _emit("GLM_WALK_AUTOSEED_NONE", {"turns": turn})
        return False
    try:
        walk_state.seed_plan(pick["plan_id"], pick["pickup"], path=str(WALK_STATE))
    except (walk_state.WalkStateError, OSError, ValueError) as exc:
        print(f"[glm-supervisor] autoseed seed_plan failed ({exc}); halting normally", flush=True)
        _emit("GLM_WALK_AUTOSEED_ERROR", {"turns": turn, "plan_id": pick["plan_id"],
                                           "error": str(exc)})
        return False
    # T6-entry10: bump the seed-attempt ledger so plan_selector's reseed filter can bound how many
    # times a not-yet-dispositioned plan is re-selected (the paid-turn-burning re-seed loop). Fail-soft:
    # a ledger-write failure never breaks the seed that already landed in walk-state.
    try:
        seeds = plan_selector.record_seed_attempt(pick["plan_id"])
    except (OSError, ValueError) as exc:
        print(f"[glm-supervisor] WARN seed-attempt ledger bump failed for {pick['plan_id']}: {exc}",
              file=sys.stderr, flush=True)
        seeds = None
    print(f"[glm-supervisor] AUTOSEED: seeded {pick['plan_id']} from ELAI open-gaps "
          f"(free-gated; eligible={pick['detail'].get('eligible')}; seed_attempt={seeds})", flush=True)
    _emit("GLM_WALK_AUTOSEED", {"turns": turn, "plan_id": pick["plan_id"],
                                 "seed_attempt": seeds, "detail": pick.get("detail")})
    return True


def _boot_prompt(session_id: str, dispatch_budget: int, trust_tiers: dict | None = None) -> str:
    """The stdin prompt for one headless ZCode GLM turn. Explicit-read of the skill
    (headless workers do not reliably auto-surface skills by description the way
    the interactive REPL does), a bounded per-turn dispatch budget, and the hard
    contract: stop at a unit boundary + hand off, DO NOT run forever - the
    supervisor respawns a fresh ZCode GLM session to continue the chain. trust_tiers (P1.6) is
    surfaced as one compact advisory line; the routing decision stays skill prose."""
    trust_line = ""
    if trust_tiers:
        trust_line = (
            f"\n\nTRUST TIERS (per task-class autonomy signal from the trust ledger; "
            f"watch<queue<auto -- ROUTE autonomy off these per the skill, NOT a gate): "
            f"{json.dumps(trust_tiers, sort_keys=True)}")
    directive = _owner_directive()
    directive_block = (
        f"\n\nOWNER DIRECTIVE (BINDING, owner-authored, from {OWNER_DIRECTIVE_PATH.name}; overrides "
        f"conflicting standing mandates below EXCEPT the frozen gates + opus_land which always "
        f"bind):\n{directive}" if directive else "")
    if _codex_first():
        routing = (
            "You ORCHESTRATE ONLY. CODEX-FIRST WINDOW: the paid codex gpt-5.6-sol credits (account-a) "
            "EXPIRE 2026-07-15 and unused credits are WASTED, so the objective this window is to BURN "
            "THEM productively. Route the EXECUTOR to codex for EVERY executable EXECUTE unit -- trivial "
            "OR hard (a doc-comment fix, a one-line wiring, an orphan mount ALL go to codex now, not the "
            "free swe lane) -- via `python3 opus_walk/opus_dispatch.py --lane codex --role EXECUTE "
            "--elai-plan <id> --allowed-writes <vertical-slice csv> --prompt-file <task>` (flag set this "
            "turn; a non-empty --allowed-writes is REQUIRED). Use the FREE swe-1-x lane ONLY for "
            "context/explore/recon, or as a fallback when codex is unavailable or refuses (codex still "
            "refuses greenfield/undesigned-seam units -- those SET_ASIDE, they do not fall back to free). "
            "codex writes a SCOPED worktree it cannot escape; gate its diff OFF-AGENT with "
            "opus_land.py (pass --codex-reap-task <task_id>) -- opus_land REFUSES any changed path not in "
            "--files and runs the frozen checkers over the real diff. NEVER author product code yourself; "
            "NEVER dictate the exact diff (escalation ladder only)."
        )
    else:
        routing = (
            "You ORCHESTRATE ONLY. CODEX IS DEAD this window: both codex accounts return 403 "
            "INSUFFICIENT_BALANCE (bmapi prepaid pool exhausted, credits expired 2026-07-15), so the paid "
            "codex lane is UNAVAILABLE. Route the EXECUTOR to the FREE swe-1-x lane (devin bridge) for "
            "EVERY execution unit -- context/explore/easy AND hard/greenfield/big alike -- via `python3 "
            "opus_walk/opus_dispatch.py --lane free --role EXECUTE --elai-plan <id> --allowed-writes "
            "<vertical-slice csv> --prompt-file <task>` (a non-empty --allowed-writes is REQUIRED). The "
            "free-lane fallback for hard/greenfield units is EXPLICITLY GRANTED (owner GO 2026-07-14 for "
            "the K backlog incl K-5); do NOT pause to ask for free-lane authorization and do NOT route to "
            "or retry codex (guaranteed 403, a wasted respawn). opus_land.py gates every free-lane diff "
            "OFF-AGENT (REFUSES any changed path not in --files, runs the frozen checkers over the real "
            "diff), so a free-lane botch cannot dirty main -- the only downside is a wasted free dispatch. "
            "If a free EXECUTE returns a 0-byte/empty diff or fails to produce a changeset, retry ONCE "
            "then SET_ASIDE that unit and hand off (do not spin). Re-arm the paid codex lane by touching "
            "conductor/CODEX_FIRST once an account is funded. NEVER author product code yourself; NEVER "
            "dictate the exact diff (escalation ladder only)."
        )
    return (
        f"You are the sequential-glm-orchestrator controller running HEADLESS through the official "
        f"ZCode app-server with builtin:zai-coding-plan / GLM-5.2 (1,000,000 context), "
        f"session_id={session_id}. This is ONE turn of a supervised resume chain.\n\n"
        f"FIRST: Read {SKILL_PATH} in full and follow it as your binding operating protocol. "
        f"The walk-state spine is {WALK_STATE} (load/claim_lock/update_cursor/record_dispatch/"
        f"mark_plan_done/session_rollover/release_lock via opus_walk/walk_state.py).\n\n"
        f"TOOLS (MCP): teikoku-memory -> memory_search(query,project) / memory_log / memory_recent "
        f"(use memory_search to retrieve rule depth per the CLAUDE.md router AND to recall prior walk "
        f"decisions; memory_log to persist ONE terse walk fact when a plan closes); codegraph-elai -> "
        f"codegraph_explore / codegraph_search / codegraph_callers over .elai_cc IF it connects, else "
        f"navigate via /workspace/ELAI/.claude/tools/elai.map (grep-optimal) + Grep; gpt-bridge -> "
        f"gpt_chat / grok_chat for a cross-family second opinion on a HARD fork (occasional, not "
        f"routine). Plus the full base-cc tool set. There is NO mcp_think -- reason in visible text "
        f"before each tool action.\n"
        f"RULES: before orchestrating, read in full the binding rules the SKILL points at that exist in "
        f"this distro: .claude/rules/{{orchestrate-protocol,wiring-enforcement,plan-standard,agent-"
        f"dispatch-preamble,token-discipline,commit-cadence,handoff-protocol,main-llm-role}}.md (the "
        f"headless router may lack the corpus, so read the files directly).\n\n"
        f"EXECUTE the skill's RESUME PROTOCOL as session {session_id}: claim the walk lock "
        f"(LockRefused = another Opus or GLM controller holds it = STOP immediately and exit), "
        f"run the preflight, "
        f"reconcile the cursor against git + the frozen checkers, then continue the per-plan loop "
        f"from the cursor. {routing}\n\n"
        f"CODEX BURN PRIORITY (owner 2026-07-14, credits expire ~2026-07-15): while CODEX_FIRST "
        f"is present, PREFER letter J/K/L/M/N/O/P/Q for EXECUTE via --lane codex (gpt-5.6-sol "
        f"PAID). For already-delivered C/B plans: drain disposition FAST, do not burn long recon. "
        f"Then execute a J+ unit on codex. Undesigned seams still SET_ASIDE (K-27). Spending "
        f"codex credits is the goal; recon-only loops are a credit miss.\n\n"
        f"CONFIRMED-STALE PLANS (MANDATORY, the paid-turn-burner): when your S1/S2/S3 recon confirms a "
        f"queued/cursored plan is ALREADY DELIVERED or stale (nothing left to build, not even polish), "
        f"you MUST, before ending the turn, call walk_state._emit_disposition(plan_id, 'drained', "
        f"reason=<one line + the evidence ref that proved it delivered>) AND walk_state.complete_plan("
        f"plan_id) to dequeue it. A confirmed-stale plan LEFT QUEUED is a protocol violation -- it "
        f"re-seeds and burns another paid turn (the reseed loop the seed-attempt cap only bounds, never "
        f"cures). Disposition first, THEN move to the next plan or run the STOP PROTOCOL.\n\n"
        f"DISPATCH BUDGET FOR THIS TURN = {dispatch_budget}. When record_dispatch() trips the budget "
        f"OR you reach a clean unit boundary: run the STOP PROTOCOL (update_cursor with the exact "
        f"next action, session_rollover, append a handoff entry, commit+push any drained plan, "
        f"release_lock) and EXIT. Do NOT keep going past the budget - the TEIKOKU supervisor will "
        f"spawn a FRESH ZCode GLM session that resumes from the walk-state + handoff you leave. "
        f"Never call session/resume or reuse a provider session. All product "
        f"output lands in .elai_cc and is committed+pushed per drained plan; the orchestrator "
        f"tooling is local-only."
    ) + trust_line + directive_block


def _spawn_glm_turn(session_id: str, dispatch_budget: int) -> dict:
    """Run one fresh headless ZCode GLM-5.2 controller session."""
    try:
        trust_tiers = trust_ledger.tier_map()
    except Exception as exc:  # noqa: BLE001 - advisory read, never blocks the controller
        print(f"[glm-supervisor] WARN trust tier_map read failed: {exc}", file=sys.stderr, flush=True)
        trust_tiers = {}
    prompt = _boot_prompt(session_id, dispatch_budget, trust_tiers)
    controller_env = {
        "CLAUDE_CODE_SESSION_ID": session_id,
        "TEIKOKU_AUTONOMOUS": "1",
        "ELAI_WALK_DISPATCH_CAP": str(dispatch_budget),
        "ELAI_WALK_PLAN_CAP": "3",
        "ELAI_WALK_CONTROLLER_PID": str(os.getpid()),
        # DEPOT LAW for the BOSS shell (2026-08-07, after the glm boss ran an ABSOLUTE-PATH
        # local `cargo test` and tripped the breaker): the official-grok adapter injects these
        # via build_grok_env, but this spawn predates that and got NEITHER the offload law nor
        # the shim PATH. Plain `cargo` now resolves to the depot shim; absolute-path bypass
        # stays a tripwire break (proven live).
        "TEIKOKU_DEPOT_OFFLOAD": os.environ.get("TEIKOKU_DEPOT_OFFLOAD", "remote"),
        "TEIKOKU_DEPOT_OFFLOAD_CONFIG": os.environ.get(
            "TEIKOKU_DEPOT_OFFLOAD_CONFIG",
            "/home/researcher/teikoku/conductor/state/depot_offload_config.json",
        ),
        "DEPOT_ORG": os.environ.get("DEPOT_ORG", "example-org"),
        "PATH": (
            str(Path(__file__).resolve().parents[1] / "opus_walk" / "depot-shim")
            + os.pathsep + os.environ.get("PATH", "")
        ),
    }
    if _codex_first():
        # This is the exact flag consumed by shared opus_walk/opus_dispatch.py.
        controller_env["ELAI_OPUS_CODEX_EXECUTOR"] = "1"
    _set_reflect_mode(session_id)
    return zcode_adapter.run_zcode_turn(
        prompt, session_id, timeout_s=GLM_TURN_TIMEOUT_S, stop_path=STOP_SENTINEL,
        extra_env=controller_env)


def run_supervisor() -> int:
    """Run fresh ZCode GLM turns until a non-count stop gate ends the chain."""
    print(f"[glm-supervisor] chain start {time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime())}",
          flush=True)
    _emit("GLM_WALK_CHAIN_START", {
        "model": GLM_MODEL, "provider": GLM_PROVIDER,
        "context_window": GLM_CONTEXT_WINDOW, "dispatch_budget": DEFAULT_DISPATCH_BUDGET})
    turn = 0
    consecutive_fast_fail = 0
    autoseeded = 0  # free-gated auto-seeds this chain (bounded by AUTOSEED_MAX_PER_CHAIN)
    # T6-entry10: did THIS chain do real work (a successful autoseed OR a turn that ran non-trivially)?
    # The auto-restart wrapper resets its backoff when a chain made progress and escalates when it did
    # not. Accumulated across the whole chain, so a chain that ran real turns then drained still counts.
    made_progress = False
    while True:
        if STOP_SENTINEL.is_file():
            print("[glm-supervisor] STOP sentinel present before turn; graceful exit", flush=True)
            _emit("GLM_WALK_CHAIN_STOPPED", {"reason": "stop_sentinel", "turns": turn})
            return _finish(HALT_STOPPED, turn, made_progress, 0, {"reason": "stop_sentinel"})
        try:
            # Self-heal first: dequeue any plan whose plan-status row is DONE (a GLM turn may have
            # committed a plan but exited before its own bookkeeping ran). Then test for no-work on the
            # reconciled state so a DONE-but-still-queued plan can never respawn a paid turn (H-28 burn).
            reconciled = _reconcile_done_plans()
            if reconciled:
                print(f"[glm-supervisor] reconciled DONE plans (dequeued): {reconciled}", flush=True)
                _emit("GLM_WALK_RECONCILED", {"turns": turn, "dequeued": reconciled})
                # After each drained plan: reap the regenerable build scratch so the worktree/cargo pile
                # cannot grow unbounded and fill host F: (incident 2026-07-12, the vhdx-fills-F: EIO wedge).
                # Runs BETWEEN turns (no turn active) so it is structurally safe; the in-flight plan-ids are
                # passed as a defensive skip-set. Fail-soft (reclaim never raises); skippable via the guard env.
                if disk_hygiene.guard_enabled():
                    try:
                        _protected = disk_hygiene.in_flight_plan_ids(walk_state.load(str(WALK_STATE)))
                    except walk_state.WalkStateError:
                        _protected = None  # unknown in-flight set -> reclaim skips worktrees, cache+trim only
                    _rc = disk_hygiene.reclaim(_protected)
                    print(f"[glm-supervisor] between-plan reclaim: freed~{_rc['freed_gb']}GB on F: "
                          f"({_rc['f_free_before']}->{_rc['f_free_after']}GB); "
                          f"reaped {len(_rc['reaped'])} dir(s)", flush=True)
                    _emit("GLM_WALK_DISK_RECLAIM", _rc)
            if _no_actionable_work():
                # Free-gated auto-seed: before halting on a drained queue, try to refill from ELAI's
                # open-gaps deck (plan_selector: free pre-filter + free quorum, NO paid model). A seed
                # re-loops so the reconcile/no-work/spawn path picks it up; we halt only when nothing
                # safe is selectable. Bounded by AUTOSEED_MAX_PER_CHAIN so a slipped greenfield gap that
                # keeps getting SET_ASIDE cannot burn unbounded paid turns.
                if autoseeded < AUTOSEED_MAX_PER_CHAIN and _try_autoseed(turn):
                    autoseeded += 1
                    made_progress = True  # a fresh seed = real work; the wrapper resets its backoff
                    continue
                print("[glm-supervisor] no actionable work (queue empty/all-DONE; cursor unset, done, "
                      "or escalated); halting -- NOT respawning into an empty queue (empty-queue guard)",
                      flush=True)
                _emit("GLM_WALK_CHAIN_NO_WORK",
                      {"turns": turn, "autoseeded": autoseeded,
                       "reason": "queue empty/all-done and cursor drained-done-or-escalated"})
                return _finish(HALT_CHAIN_NO_WORK, turn, made_progress, 0, {"autoseeded": autoseeded})
            # Final pre-spawn gate: refuse (no budget consumed, no session minted) unless there is real
            # work AND the bridge is live AND no stop sentinel. Halt exactly as the no-work path does.
            # F-6: pass the reconcile so this gate re-dequeues any landed-but-queued DONE plan and
            # re-checks on the reconciled truth BEFORE the quorum gate below -- a plan whose plan-status
            # row flips between the loop-top reconcile and here can never wake a paid turn.
            ok, reason = _should_spawn(walk_state.load(str(WALK_STATE)),
                                       reconcile=_reconcile_done_plans, path=str(WALK_STATE))
            if not ok:
                print(f"[glm-supervisor] refusing spawn: {reason}; halting "
                      "(no budget consumed, no session minted)", flush=True)
                _emit("GLM_WALK_CHAIN_NO_SPAWN", {"turns": turn, "reason": reason})
                return _finish(HALT_NO_SPAWN, turn, made_progress, 0, {"reason": reason})
            # P1.4 blind quorum cost-gate: real work + a live bridge is not enough -- poll the free
            # roster for a cheap second opinion. A quorum that is QUIET (majority says no work) skips
            # the ZCode GLM boot; a DEGRADED quorum (<2 votes) fails open and wakes. Skippable via
            # GLM_WALK_QUORUM=off; _quorum_gate is injectable so tests never hit the bridge.
            if _quorum_enabled():
                wake, qdetail = _quorum_gate(walk_state.load(str(WALK_STATE)))
                if not wake:
                    print(f"[glm-supervisor] quorum quiet ({qdetail.get('reason')}: "
                          f"yes={qdetail.get('yes')} no={qdetail.get('no')}); halting -- free voters "
                          "see no actionable work, skipping the paid boot", flush=True)
                    _emit("GLM_WALK_QUORUM_QUIET", {"turns": turn, "detail": qdetail})
                    return _finish(HALT_QUORUM_QUIET, turn, made_progress, 0, {"detail": qdetail})
                _emit("GLM_WALK_QUORUM_WAKE", {"turns": turn, "detail": qdetail})
        except walk_state.WalkStateError as exc:
            print(f"[glm-supervisor] walk-state unreadable, halting: {exc}", flush=True)
            _emit("GLM_WALK_CHAIN_ERROR", {"reason": f"walk_state: {exc}", "turns": turn})
            return _finish(HALT_ERROR_WALKSTATE, turn, made_progress, 1, {"error": str(exc)})

        # Pre-spawn disk guard (BEFORE minting a paid turn): refuse when root-fs free is dangerously
        # low, reclaiming first in the soft band. Runs between turns so its reclaim is structurally
        # safe. Emits DISK_RECLAIM for any reclaim it ran and DISK_LOW + halts (return 0, like the
        # empty-queue guard) when free stays below the soft threshold. Skippable via
        # OPUS_WALK_DISK_GUARD=off (intentional shared disk_hygiene contract).
        if disk_hygiene.guard_enabled():
            decision = disk_hygiene.disk_guard(state_loader=lambda: walk_state.load(str(WALK_STATE)))
            if decision.get("reclaim"):
                _emit("GLM_WALK_DISK_RECLAIM", decision["reclaim"])
            if decision["action"] == "halt":
                print(f"[glm-supervisor] disk guard HALT: {decision['reason']}; NOT minting a paid turn",
                      flush=True)
                _emit("GLM_WALK_DISK_LOW",
                      {"turns": turn, "f_free_gb": decision["f_free_gb"],
                       "floor_gb": disk_hygiene.HARD_FLOOR_GB, "soft_gb": disk_hygiene.SOFT_THRESHOLD_GB,
                       "below_hard": decision["below_hard"], "reason": decision["reason"]})
                return _finish(HALT_DISK_LOW, turn, made_progress, 0,
                               {"f_free_gb": decision["f_free_gb"], "reason": decision["reason"]})

        turn += 1
        # D1 hardening (2026-07-07): stamp the supervisor PID into the session_id so two supervisors
        # can NEVER mint the same id (the epoch+turn form collided when a second driver was fired in
        # the same second, and walk_state.claim_lock admits a same-session_id re-claim as a heartbeat
        # refresh -- so a colliding id let two controllers race the lock). With a unique id per
        # supervisor, claim_lock's different-session_id fresh-heartbeat refusal correctly rejects the
        # second controller; opus_walk_start's single-instance pgrep guard is the belt to this braces.
        session_id = f"s-glm-walk-{int(time.time())}-{os.getpid()}-{turn}"
        print(f"[glm-supervisor] turn {turn} spawn ZCode GLM session={session_id}", flush=True)
        _emit("GLM_TURN_SPAWNED", {"turn": turn, "session_id": session_id,
                                    "dispatch_budget": DEFAULT_DISPATCH_BUDGET})
        _t0 = time.time()
        result = _spawn_glm_turn(session_id, DEFAULT_DISPATCH_BUDGET)
        _dt = time.time() - _t0
        print(f"[glm-supervisor] turn {turn} exit rc={result['rc']} "
              f"timed_out={result['timed_out']} dur={_dt:.0f}s", flush=True)
        if result["stderr_tail"]:
            print(f"[glm-supervisor] turn {turn} stderr: {result['stderr_tail']}", flush=True)
        _emit("GLM_TURN_DONE", {"turn": turn, "session_id": session_id,
                                 "rc": result["rc"], "timed_out": result["timed_out"],
                                 "stderr_tail": result["stderr_tail"]})

        if result.get("cancelled"):
            print("[glm-supervisor] active ZCode turn cancelled by GLM stop sentinel", flush=True)
            _emit("GLM_WALK_CHAIN_STOPPED", {"reason": "stop_sentinel", "turns": turn})
            return _finish(HALT_STOPPED, turn, made_progress, 0, {"reason": "stop_sentinel"})

        # A hung turn that TIMED OUT is an infra fault, not "drained" - stop loudly
        # rather than respawn straight into the same wedge (anti-spiral).
        if result["timed_out"]:
            print("[glm-supervisor] turn timed out; halting chain for operator", flush=True)
            _emit("GLM_WALK_CHAIN_ERROR", {"reason": "turn_timeout", "turns": turn})
            return _finish(HALT_ERROR_TIMEOUT, turn, made_progress, 1, {"reason": "turn_timeout"})

        # Fast-fail storm guard: a non-zero exit in < FAST_FAIL_SECONDS did no real work (auth 401 /
        # bad flag / MCP config). Do NOT respawn into it -- count consecutive fast-fails and halt.
        if result["rc"] != 0 and _dt < FAST_FAIL_SECONDS:
            consecutive_fast_fail += 1
            print(f"[glm-supervisor] turn {turn} FAST-FAIL rc={result['rc']} in {_dt:.0f}s "
                  f"({consecutive_fast_fail}/{FAST_FAIL_LIMIT}); turn errored almost immediately "
                  f"(ZCode startup/protocol/auth, NOT real work).", flush=True)
            if consecutive_fast_fail >= FAST_FAIL_LIMIT:
                print("[glm-supervisor] fast-fail storm; permanent halt", flush=True)
                _emit("GLM_WALK_CHAIN_ERROR",
                      {"reason": "fast_fail_storm", "turns": turn, "last_rc": result["rc"],
                       "hint": "ZCode turns exit non-zero in <45s; inspect redacted adapter diagnostics"})
                return _finish(HALT_FAST_FAIL_STORM, turn, made_progress, 1,
                               {"last_rc": result["rc"]})
        else:
            consecutive_fast_fail = 0
            made_progress = True  # a real turn ran (rc=0 or ran long enough) -> the wrapper resets backoff

        for _ in range(INTER_TURN_SLEEP_S):
            if STOP_SENTINEL.is_file():
                break
            time.sleep(1)


def main() -> int:
    ap = argparse.ArgumentParser(description="ZCode GLM-orchestrator resume-chain supervisor")
    ap.add_argument("--check", action="store_true",
                    help="dry check: print resolved wiring + drained status, spawn nothing")
    args = ap.parse_args()
    if args.check:
        print(f"TEIKOKU_ROOT   = {TEIKOKU_ROOT}")
        print(f"SKILL_PATH     = {SKILL_PATH} (exists={SKILL_PATH.is_file()})")
        print(f"WALK_STATE     = {WALK_STATE} (exists={WALK_STATE.is_file()})")
        print(f"DB_PATH        = {DB_PATH} (exists={DB_PATH.is_file()})")
        print(f"STOP_SENTINEL  = {STOP_SENTINEL} (present={STOP_SENTINEL.is_file()})")
        print(f"GLM_MODEL      = {GLM_PROVIDER}/{GLM_MODEL} context={GLM_CONTEXT_WINDOW}")
        try:
            print(f"backlog_drained= {_backlog_drained()}")
        except walk_state.WalkStateError as exc:
            print(f"backlog_drained= <walk-state error: {exc}>")
        return 0
    return run_supervisor()


if __name__ == "__main__":
    raise SystemExit(main())
