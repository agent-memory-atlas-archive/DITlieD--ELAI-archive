from __future__ import annotations

import re
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


WINDOWS_DRIVE_RE = re.compile(r"^([A-Za-z]):[\\/](.*)$")


def normalize_path(value: str | Path) -> Path:
    text = str(value)
    match = WINDOWS_DRIVE_RE.match(text)
    if match:
        drive = match.group(1).lower()
        rest = match.group(2).replace("\\", "/")
        mounted = Path("/mnt") / drive / rest
        if mounted.exists() or Path("/mnt").exists():
            return mounted
    return Path(text).expanduser()


@dataclass(frozen=True)
class WorkerSlotConfig:
    slot_id: str
    harness: str
    model: str
    model_tier: str
    account_id: str | None = None
    max_concurrency: int = 1
    codex_home: str | None = None
    wsl_distro: str | None = None
    # TEIKOKU P4-B (D1/D2): the base_url a base-cc backend points at (the devin bridge
    # :8810 for devin-free, the litellm proxy :4000 for free-api). None -> backend default.
    proxy_base_url: str | None = None
    # TEIKOKU P3-A (master-plan 3.4): the rate-limit pool this slot belongs to
    # (devin | free_api.groq | ...), keyed to ~/teikoku/workers.toml. Drives the
    # per-pool concurrency envelope; None means "ungrouped" (its slot_id is its own
    # pool key, preserving the legacy one-slot-per-pool behaviour).
    pool: str | None = None
    # TEIKOKU: role->slot affinity. When set, this slot only serves tasks whose role is
    # in the set (EXECUTE/WIRE/INTEGRATE/AUDIT/POLISH). None = serves all roles. Used to
    # map an Opus/Max account: opus slots for exec/wiring/audit, a sonnet slot for polish.
    roles: tuple[str, ...] | None = None


@dataclass(frozen=True)
class LimitConfig:
    worker_cap: int
    lease_ttl_seconds: int
    heartbeat_stale_seconds: int
    subprocess_timeout_seconds: int
    free_retry_cap: int
    paid_retry_cap: int
    daily_free_attempt_cap: int
    daily_paid_attempt_cap: int
    default_token_budget: int
    model_cooldown_seconds: int
    # TEIKOKU P4-A (master-plan 4.4 / D-07, FORK F3): the recursion/fan-out fork-bomb guard
    # for session-spawn-via-queue. DISTINCT from worker_cap / the workers.toml per-pool RATE
    # cap (which bound actual parallelism); these bound the structural spawn TREE so a session
    # cannot recursively spawn unboundedly. A spawn at depth > max_spawn_depth, or whose parent
    # already has >= max_spawn_fanout live children, is REFUSED (SPAWN_REFUSED), not run.
    max_spawn_depth: int = 2
    max_spawn_fanout: int = 4
    # TEIKOKU P-BE2 (interactive-orchestration ENGINE): the Conservative per-top-level-run hard
    # COST CEILING in whole US cents (owner decision: ~$10 default = 1000 cents). DISTINCT from
    # the spawn TREE caps above (which bound structure) and the per-pool RATE cap (which bounds
    # parallelism): this bounds total spend on one top-level run lineage. budgets.RunCostLedger
    # accumulates against it; a spawn whose projected spend would exceed it is REFUSED
    # (SPAWN_REFUSED + COST_CEILING_EXCEEDED). 0 = disabled (no ceiling enforced).
    run_cost_ceiling_cents: int = 1000


@dataclass(frozen=True)
class SafetyConfig:
    allow_local_commits: bool
    allow_push: bool
    allow_gate_lock_refresh: bool
    block_dirty_integration_checkout: bool
    run_devin_sandbox: bool
    # When True, a task with an EMPTY/undeclared write-set no longer conflicts with every other
    # task (the conservative default that serializes an undeclared-scope backlog to 1 worker).
    # Declared write-sets still conflict-check ("parallel where it can", the glm-orchestrator
    # model). Safe because each task runs in an isolated worktree + integration is serialized
    # (a real collision surfaces as a rejected ff-merge -> NEEDS_REVIEW, never corruption).
    # Default False keeps the safe serialize for every project that does not opt in.
    parallel_undeclared_scopes: bool = False


@dataclass(frozen=True)
class TemporalConfig:
    host: str
    port: int
    namespace: str
    task_queue: str
    identity: str | None


@dataclass(frozen=True)
class ConductorConfig:
    workflow_root: Path
    conductor_root: Path
    repo_root: Path
    state_db: Path
    worktree_root: Path
    artifact_root: Path
    devin_bin: Path
    plan_corpus_root: Path
    finish_dir: Path
    host: str
    port: int
    backbone: str
    temporal: TemporalConfig
    limits: LimitConfig
    safety: SafetyConfig
    workers: tuple[WorkerSlotConfig, ...]
    escalation_workers: tuple[WorkerSlotConfig, ...]

    @classmethod
    def from_file(cls, path: str | Path) -> "ConductorConfig":
        config_path = Path(path)
        data = tomllib.loads(config_path.read_text(encoding="utf-8"))
        return cls.from_dict(data, base_dir=config_path.parent)

    @classmethod
    def from_dict(cls, data: dict[str, Any], base_dir: Path | None = None) -> "ConductorConfig":
        paths = data.get("paths", {})
        server = data.get("server", {})
        temporal = data.get("temporal", {})
        limits = data.get("limits", {})
        safety = data.get("safety", {})

        def path_value(key: str, default: str) -> Path:
            raw = paths.get(key, default)
            path = normalize_path(raw)
            if not path.is_absolute() and base_dir is not None:
                path = base_dir / path
            return path

        limit_config = LimitConfig(
            worker_cap=int(limits.get("worker_cap", 1)),
            lease_ttl_seconds=int(limits.get("lease_ttl_seconds", 3600)),
            heartbeat_stale_seconds=int(limits.get("heartbeat_stale_seconds", 600)),
            subprocess_timeout_seconds=int(limits.get("subprocess_timeout_seconds", 1800)),
            free_retry_cap=int(limits.get("free_retry_cap", 3)),
            paid_retry_cap=int(limits.get("paid_retry_cap", 1)),
            daily_free_attempt_cap=int(limits.get("daily_free_attempt_cap", 200)),
            daily_paid_attempt_cap=int(limits.get("daily_paid_attempt_cap", 0)),
            default_token_budget=int(limits.get("default_token_budget", 100000)),
            model_cooldown_seconds=int(limits.get("model_cooldown_seconds", 300)),
            max_spawn_depth=int(limits.get("max_spawn_depth", 2)),
            max_spawn_fanout=int(limits.get("max_spawn_fanout", 4)),
            run_cost_ceiling_cents=int(limits.get("run_cost_ceiling_cents", 1000)),
        )
        safety_config = SafetyConfig(
            allow_local_commits=bool(safety.get("allow_local_commits", False)),
            allow_push=bool(safety.get("allow_push", False)),
            allow_gate_lock_refresh=bool(safety.get("allow_gate_lock_refresh", False)),
            block_dirty_integration_checkout=bool(safety.get("block_dirty_integration_checkout", True)),
            run_devin_sandbox=bool(safety.get("run_devin_sandbox", False)),
            parallel_undeclared_scopes=bool(safety.get("parallel_undeclared_scopes", False)),
        )

        backbone = str(data.get("backbone", "sqlite")).lower()
        if backbone not in ("sqlite", "temporal"):
            raise ValueError(f"invalid backbone: {backbone}")
        temporal_config = TemporalConfig(
            host=str(temporal.get("host", "127.0.0.1")),
            port=int(temporal.get("port", 7233)),
            namespace=str(temporal.get("namespace", "default")),
            task_queue=str(temporal.get("task_queue", "conductor-task-queue")),
            identity=temporal.get("identity"),
        )
        return cls(
            workflow_root=path_value("workflow_root", "F:\\.ELAI_workflow"),
            conductor_root=path_value("conductor_root", "F:\\.ELAI_workflow\\conductor"),
            repo_root=path_value("repo_root", "F:\\.ELAI_workflow\\.elai_cc"),
            state_db=path_value("state_db", "state/control.db"),
            worktree_root=path_value("worktree_root", "worktrees"),
            artifact_root=path_value("artifact_root", "artifacts"),
            devin_bin=path_value("devin_bin", "devin"),
            plan_corpus_root=path_value("plan_corpus_root", "F:\\.ELAI_workflow\\docs"),
            finish_dir=path_value(
                "finish_dir",
                "F:\\.ELAI_workflow\\workflow\\sessions\\s-plan-audit-2026-06-04\\FINISH",
            ),
            host=str(server.get("host", "127.0.0.1")),
            port=int(server.get("port", 8766)),
            backbone=backbone,
            temporal=temporal_config,
            limits=limit_config,
            safety=safety_config,
            workers=tuple(_worker_slots(data.get("workers", []))),
            escalation_workers=tuple(_worker_slots(data.get("escalation_workers", []))),
        )


def _worker_slots(items: list[dict[str, Any]]) -> list[WorkerSlotConfig]:
    slots = []
    for item in items:
        slots.append(
            WorkerSlotConfig(
                slot_id=str(item["slot_id"]),
                harness=str(item.get("harness", "devin-free")),
                model=str(item["model"]),
                model_tier=str(item.get("model_tier", "FREE")).upper(),
                account_id=item.get("account_id"),
                max_concurrency=int(item.get("max_concurrency", 1)),
                codex_home=item.get("codex_home"),
                wsl_distro=item.get("wsl_distro"),
                proxy_base_url=item.get("proxy_base_url"),
                pool=item.get("pool"),
                roles=(tuple(str(x).strip().upper() for x in item.get("roles", [])) or None),
            )
        )
    return slots
