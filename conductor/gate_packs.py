#!/usr/bin/env python3
"""TEIKOKU pack-keyed gate-set selection (master-plan sub-phases 3.1 + 3.4 / D-06).

P3-A generalizes the conductor's verifier set from the hardcoded ELAI/Rust gate list
baked into gate_runner.build_gate_commands into a PACK-KEYED selection: a project's
active packs (from projects.toml, resolved by project_resolver) decide which gates run.

Scope boundary (do NOT cross here): P3-A only ROUTES gates per pack and ships the
rust-pack set (which IS today's exact hardcoded set, just made explicit + pack-keyed) plus
a generic placeholder. The stricter-for-Opus gate CONTENT + the other packs (unity / media)
are P3-C. The frozen verifier SCRIPTS on /mnt/f are reused via the rust pack, never
re-authored (N-02 + wiring-enforcement R8) — the /mnt/f location is the rust pack's gate
home (a pack endpoint), not a path to "sweep" to the distro (RISK C).

This module was extracted OUT of gate_runner.py so adding pack routing does not push
gate_runner past the 1000-line maintainability boundary or build_gate_commands past 80
lines (maintainability-review MR-1 / MR-3). gate_runner.build_gate_commands now delegates
here.

Production hook: gate_runner.py::build_gate_commands (the sole caller; run_gates calls
build_gate_commands, and run_gates is driven from conductor.py::run_gate, conductor_loop,
integrator, and slice_driver).
"""
from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from gate_runner import GateCommand, GateContext


# Pack names (match project_resolver + projects.toml packs=[...]).
RUST_PACK = "rust"
ELAI_PACK = "elai"
GENERIC_PACK = "generic"
UNITY_PACK = "unity"

# The TEIKOKU distro root (the dir that holds conductor/ + .claude/). Resolved from this
# module's own location so the distro_gate_freeze leg targets the DISTRO gate-set
# manifest (~/teikoku/.claude/gate-manifest.txt), NOT the task project's workflow_root
# (which for the elai project is the /mnt/f frozen rust-pack gate home, RISK C).
TEIKOKU_DISTRO_ROOT = Path(__file__).resolve().parent.parent

# The packs whose presence selects the full Rust/ELAI verifier set. A project carrying
# either runs the frozen gate-freeze check, production-caller, python-wire,
# maintainability, traced-test, wiring-contract, spec-link, and cargo gates.
RUST_GATE_PACKS = frozenset({RUST_PACK, ELAI_PACK})


def select_pack(packs: tuple[str, ...] | list[str] | None) -> str:
    """Resolve a project's pack list to the gate-set key this module routes on.

    Returns RUST_PACK when the project carries the rust/elai pack (today's behaviour),
    else GENERIC_PACK. Keeping this a single explicit resolver (not scattered ``in`` checks)
    is the canonical-layering remedy from maintainability-review.
    """
    if packs and any(p in RUST_GATE_PACKS for p in packs):
        return RUST_PACK
    if packs and UNITY_PACK in packs:
        return UNITY_PACK
    return GENERIC_PACK


def build_gate_commands_for_packs(
    context: "GateContext",
    packs: tuple[str, ...] | list[str] | None,
) -> list["GateCommand"]:
    """Build the ordered gate command list for a project's active packs.

    rust pack  -> the full frozen ELAI/Rust verifier set (identical to the pre-P3-A
                  hardcoded list).
    generic    -> a test + lint placeholder that runs the project's own test/lint when a
                  manifest is present (P3-C fleshes out the real generic + unity/media packs).
    """
    pack = select_pack(packs)
    if pack == RUST_PACK:
        commands = _rust_pack_commands(context)
    elif pack == UNITY_PACK:
        commands = _unity_pack_commands(context)
    else:
        commands = _generic_pack_commands(context)
    # TEIKOKU W4 (conductor-quality steal): the slop stage runs PACK-AGNOSTICALLY on
    # every task that touched source files -- TT (tautological source-grep tests),
    # TV (trust-model violations: integrity-pragma bypass / side-writes past an owned
    # db), DUP (new-file-is-a-near-copy = the copy-instead-of-edit class, the
    # 98%-duplication root cause on the F: conductor). Per-task DONE == the
    # integration bar: catching these HERE is what keeps a per-task PASS landable.
    slop = _slop_gate_command(context)
    if slop is not None:
        commands.append(slop)
    # TEIKOKU P3-C R8: the gate-SELECTION logic (this module + gate_runner + the
    # validator) is frozen against the executing run via a SECOND, distro-side freeze
    # leg, prepended as the unconditional first gate for EVERY pack. It activates only
    # once the controller has locked the distro manifest out-of-band (see
    # _distro_gate_freeze_command); until then it is inert (WIRE-DARK[P3-C]).
    freeze = _distro_gate_freeze_command()
    if freeze is not None:
        commands = [freeze, *commands]
    return commands


# Mirrors SOURCE_EXTS in .claude/tools/slop_semantic_check.py (the checker ignores
# non-source files anyway; this pre-filter just avoids spawning it for nothing).
_SLOP_SOURCE_EXTS = frozenset({".rs", ".py", ".ts", ".tsx", ".js", ".jsx", ".svelte"})


def _slop_gate_command(context: "GateContext") -> "GateCommand | None":
    """The W4 slop stage: distro-frozen slop_semantic_check over the task's touched
    source files. Repo-RELATIVE paths + cwd=worktree so the checker's git legs
    (diff / cat-file / ls-tree for the DUP pass) resolve inside the task worktree.
    Returns None when the task touched no source files (nothing to scan; the
    substantive-gate invariant in run_gates still holds via the pack gates)."""
    from gate_runner import GateCommand

    touched_src = [
        f for f in context.touched_files
        if Path(f).suffix.lower() in _SLOP_SOURCE_EXTS
    ]
    if not touched_src:
        return None
    return GateCommand(
        "slop_semantic_check",
        [
            sys.executable,
            str(TEIKOKU_DISTRO_ROOT / ".claude" / "tools" / "slop_semantic_check.py"),
            "--files", ",".join(touched_src),
            "--base", context.base_commit,
            "--json",
        ],
        context.worktree,
        120,
    )


def _rust_pack_commands(context: "GateContext") -> list["GateCommand"]:
    """Today's exact ELAI/Rust gate set, made explicit and pack-keyed.

    The frozen verifier scripts live under workflow_root/.claude/tools (the /mnt/f frozen
    gate home for the rust pack); they are reused, never re-authored.
    """
    from gate_runner import (
        GateCommand,
        _abs_csv,
        affected_crate,
        resolve_plan_file,
    )

    py = sys.executable
    tools = context.workflow_root / ".claude" / "tools"
    commands: list[GateCommand] = [
        GateCommand(
            "gate_freeze_check",
            [
                py,
                str(tools / "gate_freeze_check.py"),
                "--manifest",
                str(context.workflow_root / ".claude" / "gate-manifest.txt"),
                "--lock-file",
                str(context.workflow_root / ".claude" / "gate-manifest.lock"),
                "--root",
                str(context.workflow_root),
                "--json",
            ],
            context.workflow_root,
            60,
        )
    ]
    if context.touched_rs and context.new_rust_pub_symbols:
        commands.append(
            GateCommand(
                "production_caller_check",
                [
                    py,
                    str(tools / "production_caller_check.py"),
                    "--symbols",
                    ",".join(context.new_rust_pub_symbols),
                    "--root",
                    str(context.worktree),
                    "--json",
                ],
                context.worktree,
                120,
            )
        )
    if context.touched_py and context.new_py_pub_symbols:
        # Narrow the caller-scan root. Post-merge runs with worktree=repo_root
        # (canonical elai). Scanning the full tree or even all of .elai_cc hits
        # docs/.source and .elai_cc/.venv-staging and dies with RecursionError
        # (I-25 Slice2 POST_GATE_FAILED 2026-08-08). Frozen python_wire_check.py
        # cannot widen SKIP_DIR without re-lock, so root at the common parent of
        # the touched .py files (package-local) instead.
        abs_py = []
        for rel in context.touched_py:
            p = (context.worktree / rel).resolve()
            if p.suffix == ".py" and p.is_file():
                abs_py.append(p)
        if abs_py:
            try:
                py_root = Path(os.path.commonpath([str(p.parent) for p in abs_py]))
            except ValueError:
                py_root = abs_py[0].parent
        else:
            py_root = context.worktree / ".elai_cc"
            if not py_root.is_dir():
                py_root = context.worktree
        commands.append(
            GateCommand(
                "python_wire_check",
                [
                    py,
                    str(tools / "python_wire_check.py"),
                    "--symbols",
                    ",".join(context.new_py_pub_symbols),
                    "--root",
                    str(py_root),
                    "--json",
                ],
                context.worktree,
                120,
            )
        )
    if context.touched_files:
        commands.append(
            GateCommand(
                "maintainability_check",
                [
                    py,
                    str(tools / "maintainability_check.py"),
                    "--files",
                    _abs_csv(context.worktree, context.touched_files),
                    "--base",
                    context.base_commit,
                    "--json",
                ],
                context.worktree,
                120,
            )
        )
    if context.touched_test_rs:
        # Stricter for Opus-1M (D-06 / tracing-test-assert T-2, R2): --expect-traced
        # makes a rust test changeset that touches *.rs tests but carries ZERO
        # #[traced_test] a hard FAIL (the dead-evidence path screams) instead of a
        # silent pass. Rust-only; N/A for the python/generic/unity packs.
        commands.append(
            GateCommand(
                "traced_test_assert_check",
                [
                    py,
                    str(tools / "traced_test_assert_check.py"),
                    "--files",
                    _abs_csv(context.worktree, context.touched_test_rs),
                    "--expect-traced",
                    "--json",
                ],
                context.worktree,
                120,
            )
        )
    plan_file = resolve_plan_file(context)
    if plan_file is not None:
        commands.append(
            GateCommand(
                "wiring_contract_check",
                [
                    py,
                    str(tools / "wiring_contract_check.py"),
                    "--bn-file",
                    str(plan_file),
                    "--map",
                    str(context.workflow_root / ".claude" / "tools" / "elai.map"),
                    "--project-root",
                    str(context.worktree),
                    "--json",
                ],
                context.worktree,
                90,
            )
        )
        commands.append(
            GateCommand(
                "spec_link_check",
                [
                    py,
                    str(tools / "spec_link_check.py"),
                    "--file",
                    str(plan_file),
                    "--json",
                ],
                context.workflow_root,
                60,
            )
        )
    if (context.worktree / "Cargo.toml").exists():
        cargo_cwd = context.worktree
        commands.append(GateCommand("cargo_check", ["cargo", "check", "--workspace"], cargo_cwd, 300))
        crate = affected_crate(context, cargo_cwd)
        if crate:
            commands.append(GateCommand("cargo_test", ["cargo", "test", "-p", crate], cargo_cwd, 300))
    return commands


def _generic_pack_commands(context: "GateContext") -> list["GateCommand"]:
    """Generic-pack gate set (master-plan 3.3 'test+lint'): run the project's own
    test (npm test / pytest) when a recognized manifest is present.

    P3-C fail-closed correction: a project with NO recognized manifest yields an empty
    command list here, but that empty case is NO LONGER treated as a pass. run_gates
    (the verdict authority) turns a task that ran ZERO SUBSTANTIVE gates into a hard FAIL
    via the no_substantive_gates invariant (fix-b + W1 / wiring-enforcement R1), uniformly
    across packs. Counting SUBSTANTIVE (non-meta) gates rather than total commands also
    closes the post-lock fail-OPEN: once the controller locks the distro manifest,
    distro_gate_freeze (a META integrity guard) is prepended for every pack, so an empty
    generic set would otherwise become [distro_gate_freeze] (non-empty) and slip past an
    empty-list-only guard. The earlier P3-A rationalization that an empty list was 'a safe
    placeholder, NOT a fail-open done-gate' was WRONG for D-06: an empty list let the loop
    never run and overall stayed PASS. A lint command is added only when a linter is
    verified present in the environment; no linter is shipped, so the no-manifest FAIL is
    the lint-absent posture (no new dependency is introduced, anti-slopsquat C-05).
    """
    from gate_runner import GateCommand

    commands: list[GateCommand] = []
    if (context.worktree / "package.json").exists():
        commands.append(GateCommand("npm_test", ["npm", "test"], context.worktree, 300))
    if (context.worktree / "pyproject.toml").exists():
        commands.append(GateCommand("py_test", [sys.executable, "-m", "pytest", "-q"], context.worktree, 300))
    return commands


def _unity_pack_commands(context: "GateContext") -> list["GateCommand"]:
    """Unity-pack gate set (master-plan 3.3 / D-13 + AOR U-10 / AOR-D10).

    TWO classes of gate, so the pack is substantively useful WITHOUT a live Editor:

    STATIC legs (AOR U-10, always run on the changeset's touched .cs, NO bridge needed):
      - toban001          : the banned-Unity-API analyzer (a BLOCK is a real fail)
      - unity_symbol_census : verify-before-write deny-tier symbol-without-receipt = BLOCK
      - mono_wire_census  : a NEW MonoBehaviour attached to nothing + referenced by nothing
                            is WIRE-DARK (the Unity producer-without-consumer BLOCK, R6/R7)
      (the .cs 600-line prod-cap is enforced inside toban001 as TOBAN-FILESIZE, because the
       frozen maintainability_check.py does NOT scan .cs -- it would be a no-op fail-open here)
    These are the SUBSTANTIVE gates that make a Unity task pass/fail off the static analysis,
    exactly the way the rust pack's wire/maintainability legs do, so a Unity changeset is
    judged even when no Editor is reachable from the distro.

    RUNTIME legs (AOR-D10, the off-agent artifact the agent did not author):
      - unity_compile / unity_console_clean / unity_playmode_smoke run only when a Unity
        project + a live unityMCP bridge are configured (resolved from the threaded env the
        orchestrator sets from the project descriptor). The PlayMode run_tests NUnit XML is
        the load-bearing wiring-enforcement R4 proof; it is DRIVEN by the auditor (AOR-D10),
        so day-1 / bridge-down these are reported as a WIRE-DARK runtime leg, not fabricated.

    Fail-closed posture (wiring-enforcement R1 + fix-b): if NEITHER a static substantive gate
    NOR a configured runtime leg applies (no .cs touched AND no bridge), the pack returns a
    SINGLE synthetic-FAIL gate, never an empty (==PASS) list. An inactive pack must scream.
    """
    from gate_runner import GateCommand

    py = sys.executable
    tools = TEIKOKU_DISTRO_ROOT / ".claude" / "tools"
    touched_cs = [f for f in context.touched_files if f.endswith(".cs")]
    touched_yaml = [f for f in context.touched_files
                    if f.endswith((".unity", ".prefab", ".asset"))]
    new_cs = _unity_new_cs(context)
    project = _unity_project_dir(context)
    bridge = _unity_bridge_endpoint()

    commands: list[GateCommand] = []
    if touched_yaml:
        # AOR-D9: the Unity executor NEVER hand-writes scene/prefab/asset YAML (fileID/GUID
        # hallucination corrupts the project); it mutates via unityMCP / a builder. A
        # changeset that touched .unity/.prefab/.asset directly is a BLOCK -- the deterministic
        # enforcement of the executor profile's hard rule (not just prose). A legitimate
        # bridge-driven mutation does not show as a raw text edit in the diff; if a YAML file
        # MUST be authored directly it carries an in-file // YAML-ALLOW marker (rare).
        yaml_csv = ", ".join(touched_yaml)
        msg = (
            "raw Unity YAML hand-edit BLOCKED (AOR-D9): the changeset touched scene/prefab/"
            "asset YAML directly (%s). Mutate via unityMCP (manage_prefabs / manage_gameobject"
            " / manage_components / manage_scriptable_object) or a Procedural Builder, never by"
            " hand (fileID/GUID hallucination corrupts the project)." % yaml_csv
        )
        commands.append(
            GateCommand(
                "unity_yaml_handedit",
                [py, "-c", "import sys; sys.stderr.write(%r); sys.exit(1)" % msg],
                context.worktree,
                30,
            )
        )
    if touched_cs:
        cs_csv = ",".join(str(context.worktree / f) for f in touched_cs)
        commands.append(
            GateCommand("toban001", [py, str(tools / "toban001_check.py"), "--files", cs_csv, "--json"],
                        context.worktree, 90)
        )
        policy = TEIKOKU_DISTRO_ROOT / ".claude" / "unity-symbol-policy.json"
        commands.append(
            GateCommand("unity_symbol_census",
                        [py, str(tools / "unity_symbol_census.py"), "--files", cs_csv,
                         "--policy", str(policy), "--json"],
                        context.worktree, 90)
        )
    if new_cs:
        # The MonoBehaviour-attachment wire census judges only the NEW .cs (producers); a
        # NEW component attached to nothing + referenced by nothing is wire-dark (R6/R7).
        scan_root = project if project is not None else context.worktree
        new_csv = ",".join(str(context.worktree / f) for f in new_cs)
        commands.append(
            GateCommand("mono_wire_census",
                        [py, str(tools / "mono_wire_census.py"), "--new-files", new_csv,
                         "--project", str(scan_root), "--json"],
                        context.worktree, 120)
        )
    if project is not None and bridge is not None:
        # Configured runtime legs (the auditor drives run_tests as the off-agent NUnit XML,
        # AOR-D10). WIRE-DARK[U-10] day-1: no Unity project + live bridge is reachable from
        # the distro, so these are unreachable until the live Editor is up; the static legs
        # above carry the substance. Backing row: workflow/handoff/teikoku-build.md.
        commands.append(GateCommand("unity_compile", _unity_compile_argv(project, bridge), context.worktree, 600))
        commands.append(GateCommand("unity_console_clean", _unity_console_argv(project, bridge), context.worktree, 120))
        commands.append(GateCommand("unity_playmode_smoke", _unity_playmode_argv(project, bridge), context.worktree, 600))

    if not commands:
        # Fail-closed: a Unity task that touched no .cs AND has no live bridge verified
        # nothing -- scream, do not pass (R1 / fix-b). run_gates' no_substantive_gates
        # invariant would also catch an all-meta list, but a self-describing FAIL here is
        # clearer than an empty list.
        msg = (
            "unity gate not configured + no C# touched: nothing to verify "
            "(static analyzers need touched .cs; the runtime legs need a native-Windows "
            "Unity Editor + a live unityMCP bridge -- see AOR-D7/D10)"
        )
        return [
            GateCommand(
                "unity_gate_not_configured",
                [py, "-c", "import sys; sys.stderr.write(%r); sys.exit(1)" % msg],
                context.worktree,
                30,
            )
        ]
    return commands


def _unity_new_cs(context: "GateContext") -> list[str]:
    """The NEW .cs files in the changeset (added, not merely modified) -- the producers the
    MonoBehaviour wire census judges. Derived from `git diff --diff-filter=A` against the
    base; falls back to all touched .cs if the diff is unavailable (fail toward judging)."""
    try:
        proc = subprocess.run(
            ["git", "-C", str(context.worktree), "diff", "--name-only", "--diff-filter=A",
             context.base_commit],
            capture_output=True, text=True, timeout=60,
        )
        if proc.returncode == 0:
            added = [ln.strip() for ln in proc.stdout.splitlines() if ln.strip().endswith(".cs")]
            # Untracked-but-new .cs are added to the index as -N by run_gates before this,
            # so they appear in the diff; if the diff is empty, judge all touched .cs.
            if added:
                return added
    except (subprocess.SubprocessError, OSError):
        pass
    return [f for f in context.touched_files if f.endswith(".cs")]


def _unity_project_dir(context: "GateContext") -> Path | None:
    """Resolve the configured Unity project dir, or None when unconfigured.

    The orchestrator threads the descriptor's Windows-native game path (mapped to the
    distro's /mnt view) into the environment as TEIKOKU_UNITY_PROJECT_DIR before running the
    gates (AOR-D7 topology: the game lives on its Windows path, reached over /mnt). None when
    the env is unset OR the path does not resolve to a real Unity project (an Assets/ dir),
    so a mis-set path fails closed rather than scanning a bogus tree."""
    raw = os.environ.get("TEIKOKU_UNITY_PROJECT_DIR", "").strip()
    if not raw:
        return None
    path = Path(raw)
    return path if (path / "Assets").is_dir() else None


def _unity_bridge_endpoint() -> str | None:
    """Resolve the live unityMCP bridge endpoint, or None when no live Editor bridge is up.

    The orchestrator threads the forwarded-localhost unityMCP host:port from the descriptor
    into TEIKOKU_UNITY_BRIDGE only AFTER a health probe confirms the bridge answers (the
    orient brief's EDITOR/BRIDGE STATE field, AOR §9.4). None when unset, so the runtime legs
    fail toward 'not configured' and the static legs carry the verdict (AOR-D7 fallback ladder)."""
    raw = os.environ.get("TEIKOKU_UNITY_BRIDGE", "").strip()
    return raw or None


def _unity_compile_argv(project: Path, bridge: str) -> list[str]:
    """The cross-boundary Unity compile endpoint (P4). WIRE-DARK[P3-C]: never reached
    day-1 (project/bridge are None); shape only, real -batchmode argv authored in P4."""
    return ["unity_bridge", "compile", "--project", str(project), "--bridge", bridge]


def _unity_console_argv(project: Path, bridge: str) -> list[str]:
    """The cross-boundary Unity console-clean endpoint (P4). WIRE-DARK[P3-C]: shape only."""
    return ["unity_bridge", "console-clean", "--project", str(project), "--bridge", bridge]


def _unity_playmode_argv(project: Path, bridge: str) -> list[str]:
    """The cross-boundary Unity PlayMode-smoke endpoint (P4). WIRE-DARK[P3-C]: shape only."""
    return ["unity_bridge", "playmode-smoke", "--project", str(project), "--bridge", bridge]


def _distro_gate_freeze_command():
    """The distro-side R8 freeze leg for the gate-SELECTION logic (gate_packs / gate_runner /
    the validator), built only when the distro gate-manifest is LOCKED.

    D-06 says 'gates are frozen', but the /mnt/f gate-manifest freezes only the checker
    SCRIPTS, not this distro's gate-selection code. This second leg runs the FROZEN
    /mnt/f gate_freeze_check.py against the DISTRO manifest at
    ~/teikoku/.claude/gate-manifest.txt so a run that weakens its own selection logic is
    caught.

    WIRE-DARK[P3-C] until the controller locks: gate_freeze_check with no lock-file yet
    returns NO_BASELINE (exit 2 -> ERROR -> overall FAIL), and per R8 a run CANNOT lock its
    own gate. So this gate is INERT (returns None, absent from the command list) until the
    controller runs `gate_freeze_check.py --lock` against the distro manifest out-of-band
    AFTER the audit. Once the .lock exists it activates as the unconditional first gate.
    Backing row: workflow/handoff/teikoku-build.md (P3-C WIRE-DARK rows). The controller
    runs --lock; --lock is never run by the conductor (execute_gate_command forbids it).
    """
    from gate_runner import GateCommand

    claude_dir = TEIKOKU_DISTRO_ROOT / ".claude"
    manifest = claude_dir / "gate-manifest.txt"
    lock_file = claude_dir / "gate-manifest.lock"
    # Inert until the controller has locked the baseline out-of-band (R8: the run cannot
    # lock its own gate). A missing manifest or a missing lock = WIRE-DARK / not-yet-active.
    if not manifest.is_file() or not lock_file.is_file():
        return None
    # The frozen checker lives in the distro .claude/tools copy (byte-identical to /mnt/f).
    tool = claude_dir / "tools" / "gate_freeze_check.py"
    return GateCommand(
        "distro_gate_freeze",
        [
            sys.executable,
            str(tool),
            "--manifest",
            str(manifest),
            "--lock-file",
            str(lock_file),
            "--root",
            str(TEIKOKU_DISTRO_ROOT),
            "--json",
        ],
        TEIKOKU_DISTRO_ROOT,
        60,
    )

def verifier_descriptor_for_packs(
    packs,
    plan_path,
    deterministic_check=None,
    wiring_contract=None,
    is_phase=False,
):
    """Build the STATIC verifier descriptor list a TaskSpec carries for a project's packs.

    This is the IMPORT-TIME, descriptive twin of build_gate_commands_for_packs (which is
    the EXECUTION-TIME authority that runs in gate_runner with a live GateContext). A
    TaskSpec.verifier is informational (rendered into the worker prompt by
    context_builder); the actual gate selection at run time is pack-routed by
    build_gate_commands_for_packs(context, project.packs). Keeping BOTH keyed off the
    SAME select_pack(packs) is the canonical-layering remedy: the import-time descriptor
    and the run-time command set never disagree about which pack a project is.

    rust/elai pack -> the frozen ELAI verifier names (gate_freeze_check + wiring +
                      deterministic + phase acceptance). The frozen scripts live under the
                      rust pack's gate home on /mnt/f; they are referenced by NAME here,
                      never re-authored (N-02 / wiring-enforcement R8).
    generic pack   -> a project test+lint descriptor with NO F:/.ELAI_workflow reference,
                      so a non-ELAI project's tasks never carry an ELAI gate path.
    """
    pack = select_pack(packs)
    if pack == RUST_PACK:
        return _rust_pack_verifier_descriptor(
            plan_path, deterministic_check, wiring_contract, is_phase
        )
    if pack == UNITY_PACK:
        return _unity_pack_verifier_descriptor()
    return _generic_pack_verifier_descriptor(deterministic_check)


def _rust_pack_verifier_descriptor(plan_path, deterministic_check, wiring_contract, is_phase):
    descriptor = [{"name": "gate_freeze_check", "pack": RUST_PACK}]
    if deterministic_check:
        descriptor.append({"name": "deterministic_check", "command_text": deterministic_check})
    if wiring_contract:
        descriptor.append(
            {"name": "wiring_contract_check", "bn_file": str(plan_path), "pack": RUST_PACK}
        )
    if is_phase:
        descriptor.append({"name": "phase_acceptance", "source": str(plan_path)})
    # P3-C R10: the cross-seam integration smoke is a DECLARED member of the rust set,
    # but it is DRIVEN at phase-end by the auditor against the real entrypoint on a HARD
    # input, NOT run per-task here (no fabricated per-task smoke). WIRE-DARK[P3-C]:
    # consumer = the phase-end auditor. Backing row: workflow/handoff/teikoku-build.md.
    descriptor.append({"name": "integration_smoke", "pack": RUST_PACK, "driver": "auditor_phase_end"})
    return descriptor


def _generic_pack_verifier_descriptor(deterministic_check):
    descriptor = [
        {"name": "project_test", "pack": GENERIC_PACK},
        {"name": "project_lint", "pack": GENERIC_PACK},
    ]
    if deterministic_check:
        descriptor.append({"name": "deterministic_check", "command_text": deterministic_check})
    return descriptor


def _unity_pack_verifier_descriptor():
    """The static verifier-descriptor twin for the unity pack (AOR U-10). Mirrors the run-time
    _unity_pack_commands gate names so the import-time descriptor and the run-time command set
    never disagree about which pack a project is (canonical-layering remedy). It names BOTH
    the STATIC legs (always-run on touched .cs: toban001 [incl the .cs 600-line cap as
    TOBAN-FILESIZE], the verify-before-write symbol census, the MonoBehaviour-attachment wire
    census) AND the RUNTIME legs (auditor-driven via the live bridge, AOR-D10): compile /
    console-clean / PlayMode smoke. PlayMode run_tests carries driver=auditor_phase_end (the
    off-agent NUnit XML, wiring-enforcement R4)."""
    return [
        {"name": "unity_yaml_handedit", "pack": UNITY_PACK},
        {"name": "toban001", "pack": UNITY_PACK},
        {"name": "unity_symbol_census", "pack": UNITY_PACK},
        {"name": "mono_wire_census", "pack": UNITY_PACK},
        {"name": "unity_compile", "pack": UNITY_PACK, "driver": "auditor_phase_end"},
        {"name": "unity_console_clean", "pack": UNITY_PACK, "driver": "auditor_phase_end"},
        {"name": "unity_playmode_smoke", "pack": UNITY_PACK, "driver": "auditor_phase_end"},
    ]


def followup_verifier_descriptor_for_packs(packs, plan_path, wave, deterministic_check=None):
    """The static verifier descriptor for a wave-follow-up task (INTEGRATE/AUDIT/POLISH).

    Same pack routing as verifier_descriptor_for_packs; the rust pack carries the frozen
    gate-freeze name + a per-wave gate; the generic pack carries the project test/lint.
    """
    if select_pack(packs) == RUST_PACK:
        descriptor = [
            {"name": "gate_freeze_check", "pack": RUST_PACK},
            {"name": wave.lower() + "_gate", "source": str(plan_path)},
        ]
    else:
        descriptor = [
            {"name": "project_test", "pack": GENERIC_PACK},
            {"name": wave.lower() + "_gate", "source": str(plan_path)},
        ]
    if deterministic_check:
        descriptor.append({"name": "deterministic_check", "command_text": deterministic_check})
    return descriptor