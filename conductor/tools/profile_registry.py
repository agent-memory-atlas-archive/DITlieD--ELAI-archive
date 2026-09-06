#!/usr/bin/env python3
"""Validate and resolve project-local Depot offload profiles."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any, Optional

SCHEMA_VERSION = 1
STATUS_VALUES = frozenset({"ready", "setup-required"})
ADAPTER_VALUES = frozenset(
    {"ougi-depot-offload", "depot-general", "depot-container-builds", "depot-ci"}
)
MODE_VALUES = frozenset({"remote"})
NEEDS = frozenset({"verify", "artifact"})
REQUIRED_VERIFY_EVIDENCE = (
    "/home/researcher/teikoku/Ougi/tools/depot_offload.py",
    "/home/researcher/teikoku/Ougi/docs/CONDUCTOR-DEPOT-USAGE.md",
)
BUNDLED_PROFILE_PATH = (
    Path(__file__).resolve().parents[1] / "references" / "project-profiles.json"
)
BUNDLED_PROFILE_IDS = frozenset({"grok-build-main", "zenov", "elai"})
SECRET_KEY_PARTS = frozenset(
    {
        "apikey",
        "token",
        "tokens",
        "secret",
        "secrets",
        "password",
        "passwords",
        "credential",
        "credentials",
    }
)
DEPOT_TOKEN_VALUE_RE = re.compile(r"\bDEPOT_TOKEN\s*=", re.IGNORECASE)
BEARER_VALUE_RE = re.compile(
    r"\bBearer\s+[A-Za-z0-9][A-Za-z0-9._~+/=-]{7,}", re.IGNORECASE
)
PRIVATE_KEY_VALUE_RE = re.compile(
    r"-----BEGIN [^-\r\n]*PRIVATE KEY-----", re.IGNORECASE
)
DEPOT_PROVENANCE_RE = re.compile(
    r"^DEPOT_PROVENANCE path=(?P<path>build|ci) "
    r"head=(?P<head>[0-9a-fA-F]{40}) "
    r"dirty=(?P<dirty>[01]) "
    r"worktree=(?P<worktree>/\S+)$"
)


class ProfileError(ValueError):
    """Raised when a profile document violates the local schema or policy."""


def _path_label(path: Sequence[str]) -> str:
    return ".".join(path) or "document"


def _normal_path(value: str | os.PathLike[str]) -> Path:
    return Path(value).expanduser().resolve()


def _secret_key(key: str) -> bool:
    normalized = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", key)
    normalized = re.sub(r"[^a-zA-Z0-9]+", "_", normalized).strip("_").lower()
    parts = set(normalized.split("_"))
    compact = re.sub(r"[^a-z0-9]", "", key.lower())
    return bool(parts & SECRET_KEY_PARTS) or "apikey" in compact or "api_key" in normalized


def _scan_secret_values(value: Any, path: Sequence[str] = ()) -> None:
    if isinstance(value, Mapping):
        for key, child in value.items():
            if not isinstance(key, str):
                raise ProfileError(f"non-string key at {_path_label(path)}")
            child_path = (*path, key)
            if _secret_key(key):
                raise ProfileError(f"secret-bearing key at {_path_label(child_path)}")
            _scan_secret_values(child, child_path)
        return
    if isinstance(value, list):
        for index, child in enumerate(value):
            _scan_secret_values(child, (*path, str(index)))
        return
    if isinstance(value, str):
        if DEPOT_TOKEN_VALUE_RE.search(value):
            raise ProfileError(f"secret-looking Depot token at {_path_label(path)}")
        if BEARER_VALUE_RE.search(value):
            raise ProfileError(f"secret-looking bearer token at {_path_label(path)}")
        if PRIVATE_KEY_VALUE_RE.search(value):
            raise ProfileError(f"private key material at {_path_label(path)}")


def _require_mapping(value: Any, path: Sequence[str]) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ProfileError(f"{_path_label(path)} must be an object")
    return value


def _parse_provenance(value: str, path: Sequence[str]) -> dict[str, Any]:
    lines = [line for line in value.splitlines() if "DEPOT_PROVENANCE" in line]
    if len(lines) != 1:
        raise ProfileError(f"smoke must contain one DEPOT_PROVENANCE line at {_path_label(path)}")
    match = DEPOT_PROVENANCE_RE.fullmatch(lines[0])
    if match is None:
        raise ProfileError(f"malformed DEPOT_PROVENANCE line at {_path_label(path)}")
    fields = match.groupdict()
    worktree = fields["worktree"]
    if not os.path.isabs(worktree):
        raise ProfileError(f"DEPOT_PROVENANCE worktree must be absolute at {_path_label(path)}")
    return {
        "path": fields["path"],
        "head": fields["head"].lower(),
        "dirty": int(fields["dirty"]),
        "worktree": worktree,
    }


def _validate_smoke(route: Mapping[str, Any], path: Sequence[str]) -> None:
    smoke = route.get("smoke")
    smoke_path = (*path, "smoke")
    if not isinstance(smoke, Mapping):
        raise ProfileError(f"ready route lacks smoke at {_path_label(path)}")
    for field in ("run_or_build_id", "observed_at", "provenance"):
        value = smoke.get(field)
        if not isinstance(value, str) or not value.strip():
            raise ProfileError(f"smoke lacks non-empty {field} at {_path_label(smoke_path)}")
    _parse_provenance(smoke["provenance"], (*smoke_path, "provenance"))


def _validate_route(
    value: Any,
    path: Sequence[str],
    *,
    require_smoke: bool,
    artifact_route: bool,
) -> None:
    route = _require_mapping(value, path)
    status = route.get("status")
    if not isinstance(status, str) or status not in STATUS_VALUES:
        raise ProfileError(f"unknown route status at {_path_label((*path, 'status'))}")
    for field in ("adapter", "mode"):
        if field not in route:
            continue
        item = route[field]
        if not isinstance(item, str):
            raise ProfileError(f"{_path_label((*path, field))} must be a string")
        allowed = ADAPTER_VALUES if field == "adapter" else MODE_VALUES
        if item not in allowed:
            raise ProfileError(f"unknown {field} at {_path_label((*path, field))}: {item}")
    if status == "ready":
        for field in ("adapter", "mode"):
            if field not in route:
                raise ProfileError(f"ready route lacks {field} at {_path_label(path)}")
        evidence = route.get("evidence")
        if not isinstance(evidence, list) or not evidence:
            raise ProfileError(f"ready route lacks evidence at {_path_label(path)}")
        if any(not isinstance(item, str) or not item.strip() for item in evidence):
            raise ProfileError(f"route evidence must contain non-empty strings at {_path_label(path)}")
        if require_smoke:
            _validate_smoke(route, path)
        if artifact_route:
            artifact_name = route.get("artifact_name")
            if not isinstance(artifact_name, str) or not artifact_name.strip():
                raise ProfileError(f"ready artifact route lacks artifact_name at {_path_label(path)}")
    elif "evidence" in route:
        evidence = route["evidence"]
        if not isinstance(evidence, list):
            raise ProfileError(f"route evidence must be a list at {_path_label(path)}")
        if any(not isinstance(item, str) or not item.strip() for item in evidence):
            raise ProfileError(f"route evidence must contain non-empty strings at {_path_label(path)}")


def _validate_profile(
    value: Any,
    index: int,
    *,
    require_local_proof: bool,
    bundled: bool,
) -> dict[str, Any]:
    path = ("profiles", str(index))
    profile = _require_mapping(value, path)
    profile_id = profile.get("id")
    if not isinstance(profile_id, str) or not profile_id.strip():
        raise ProfileError(f"profile id must be a non-empty string at {_path_label(path)}")
    roots = profile.get("roots")
    if not isinstance(roots, list) or not roots:
        raise ProfileError(f"profile roots must be a non-empty list at {_path_label(path)}")
    for root_index, root in enumerate(roots):
        if not isinstance(root, str) or not root.strip() or not os.path.isabs(root):
            raise ProfileError(
                f"profile root must be absolute at {_path_label((*path, 'roots', str(root_index)))}"
            )
    junk_paths = profile.get("junk_paths")
    if junk_paths is not None:
        if not isinstance(junk_paths, list) or any(
            not isinstance(item, str) or not item.strip() for item in junk_paths
        ):
            raise ProfileError(f"junk_paths must contain non-empty strings at {_path_label(path)}")
    for route_name in ("verify_route", "artifact_route"):
        if route_name not in profile:
            raise ProfileError(f"profile lacks {route_name} at {_path_label(path)}")
        _validate_route(
            profile[route_name],
            (*path, route_name),
            require_smoke=require_local_proof,
            artifact_route=route_name == "artifact_route",
        )
    if bundled and profile_id in BUNDLED_PROFILE_IDS:
        verify_route = _require_mapping(profile["verify_route"], (*path, "verify_route"))
        evidence = verify_route.get("evidence")
        if not isinstance(evidence, list) or not set(REQUIRED_VERIFY_EVIDENCE).issubset(evidence):
            raise ProfileError(f"bundled verify evidence is incomplete at {_path_label(path)}")
    return dict(profile)


def validate_document(
    document: Any,
    *,
    require_one: bool = False,
    require_local_proof: bool = False,
    bundled: bool = False,
) -> list[dict[str, Any]]:
    """Validate a schema-versioned profile document and return its profiles."""
    _scan_secret_values(document)
    root = _require_mapping(document, ())
    if root.get("schema_version") != SCHEMA_VERSION:
        raise ProfileError(f"schema_version must be {SCHEMA_VERSION}")
    profiles = root.get("profiles")
    if not isinstance(profiles, list) or not profiles:
        raise ProfileError("profiles must be a non-empty list")
    if require_one and len(profiles) != 1:
        raise ProfileError("a local profile document must contain exactly one profile")
    result = [
        _validate_profile(
            item,
            index,
            require_local_proof=require_local_proof,
            bundled=bundled,
        )
        for index, item in enumerate(profiles)
    ]
    ids = [profile["id"] for profile in result]
    if len(ids) != len(set(ids)):
        raise ProfileError("duplicate profile id")
    return result


def _read_document(
    path: Path,
    *,
    require_one: bool,
    require_local_proof: bool = False,
    bundled: bool = False,
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise ProfileError(f"profile file does not exist: {path}") from exc
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ProfileError(f"invalid profile JSON: {path}") from exc
    profiles = validate_document(
        document,
        require_one=require_one,
        require_local_proof=require_local_proof,
        bundled=bundled,
    )
    return document, profiles


def validate_file(path: str | os.PathLike[str], *, require_one: bool = True) -> list[dict[str, Any]]:
    """Validate a profile file without contacting Depot."""
    resolved = _normal_path(path)
    is_bundled = resolved == BUNDLED_PROFILE_PATH.resolve()
    _, profiles = _read_document(
        resolved,
        require_one=require_one,
        require_local_proof=not is_bundled,
        bundled=is_bundled,
    )
    return profiles


def _contains(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
        return True
    except ValueError:
        return False


def _best_match(
    profiles: Sequence[dict[str, Any]], cwd: Path
) -> Optional[tuple[dict[str, Any], Path]]:
    candidates: list[tuple[int, dict[str, Any], Path]] = []
    for profile in profiles:
        for raw_root in profile["roots"]:
            root = _normal_path(raw_root)
            if _contains(cwd, root):
                candidates.append((len(root.parts), profile, root))
    if not candidates:
        return None
    _, profile, root = max(candidates, key=lambda item: item[0])
    return profile, root


def _find_local_profile(cwd: Path) -> Optional[Path]:
    for candidate in (cwd, *cwd.parents):
        profile_path = candidate / ".depot" / "offload-profile.json"
        if profile_path.is_file():
            return profile_path
    return None


def _unknown_profile(cwd: Path) -> dict[str, Any]:
    route = {"status": "setup-required"}
    return {
        "id": "unknown",
        "roots": [str(cwd)],
        "junk_paths": [],
        "verify_route": dict(route),
        "artifact_route": dict(route),
        "notes": ["No local or bundled profile matches this project root."],
    }


def resolve_profile(
    cwd: str | os.PathLike[str], need: str
) -> dict[str, Any]:
    """Resolve a verification or artifact route for a project directory."""
    if need not in NEEDS:
        raise ProfileError(f"need must be one of: {', '.join(sorted(NEEDS))}")
    cwd_path = _normal_path(cwd)
    local_path = _find_local_profile(cwd_path)
    if local_path is not None:
        _, profiles = _read_document(local_path, require_one=True, require_local_proof=True)
        match = _best_match(profiles, cwd_path)
        if match is None:
            raise ProfileError(f"local profile does not cover cwd: {cwd_path}")
        profile, resolved_root = match
        _bind_ready_routes(profile, resolved_root)
        source = "local"
        source_path: Optional[str] = str(local_path)
    else:
        _, profiles = _read_document(
            BUNDLED_PROFILE_PATH,
            require_one=False,
            bundled=True,
        )
        match = _best_match(profiles, cwd_path)
        if match is None:
            profile = _unknown_profile(cwd_path)
            resolved_root = cwd_path
            source = "unknown"
            source_path = None
        else:
            profile, resolved_root = match
            source = "bundled"
            source_path = str(BUNDLED_PROFILE_PATH)
    route = profile[f"{need}_route"]
    return {
        "schema_version": SCHEMA_VERSION,
        "source": source,
        "source_path": source_path,
        "cwd": str(cwd_path),
        "resolved_root": str(resolved_root),
        "need": need,
        "profile": profile,
        "route": route,
    }


def _read_git_state(root: Path, *, need_main: bool) -> dict[str, Any]:
    def run_git(arguments: Sequence[str], *, allow_empty: bool = False) -> str:
        try:
            result = subprocess.run(
                ["git", "-C", str(root), *arguments],
                capture_output=True,
                text=True,
                errors="replace",
                timeout=30,
            )
        except (OSError, subprocess.TimeoutExpired) as exc:
            raise ProfileError(f"cannot read git state for {root}: {exc}") from exc
        if result.returncode != 0:
            raise ProfileError(f"cannot read git state for {root}: git {' '.join(arguments)}")
        value = (result.stdout or "").strip()
        if not value and not allow_empty:
            raise ProfileError(f"cannot read git state for {root}: git {' '.join(arguments)}")
        return value

    head = run_git(("rev-parse", "HEAD"))
    if re.fullmatch(r"[0-9a-fA-F]{40}", head) is None:
        raise ProfileError(f"git HEAD is not a 40-hex commit for {root}")
    status = run_git(("status", "--porcelain"), allow_empty=True)
    state: dict[str, Any] = {
        "head": head.lower(),
        "dirty": 1 if status else 0,
        "main_heads": {},
    }
    if need_main:
        for ref in ("origin/main", "refs/remotes/origin/main", "main", "refs/heads/main"):
            try:
                value = run_git(("rev-parse", "--verify", ref))
            except ProfileError:
                continue
            if re.fullmatch(r"[0-9a-fA-F]{40}", value):
                state["main_heads"][ref] = value.lower()
        if not state["main_heads"]:
            raise ProfileError(f"no resolvable local main ref for {root}")
    return state


def _bind_ready_routes(profile: Mapping[str, Any], root: Path) -> None:
    ready_routes: list[tuple[str, Mapping[str, Any], dict[str, Any]]] = []
    needs_main = False
    for route_name in ("verify_route", "artifact_route"):
        route = _require_mapping(profile[route_name], ("profile", route_name))
        if route.get("status") != "ready":
            continue
        smoke = _require_mapping(route.get("smoke"), ("profile", route_name, "smoke"))
        provenance = smoke.get("provenance")
        if not isinstance(provenance, str):
            raise ProfileError(f"ready route provenance must be a string at profile.{route_name}")
        parsed = _parse_provenance(
            provenance, ("profile", route_name, "smoke", "provenance")
        )
        if parsed["worktree"] != str(root):
            raise ProfileError(
                f"DEPOT_PROVENANCE worktree must equal resolved root at profile.{route_name}"
            )
        if parsed["path"] == "ci":
            needs_main = True
        ready_routes.append((route_name, route, parsed))
    if not ready_routes:
        return
    state = _read_git_state(root, need_main=needs_main)
    for route_name, _route, parsed in ready_routes:
        if parsed["head"] != state["head"]:
            raise ProfileError(f"DEPOT_PROVENANCE HEAD does not match root at profile.{route_name}")
        if parsed["dirty"] != state["dirty"]:
            raise ProfileError(f"DEPOT_PROVENANCE dirty state does not match root at profile.{route_name}")
        if parsed["path"] == "ci":
            if state["dirty"] != 0:
                raise ProfileError(f"CI provenance requires a clean root at profile.{route_name}")
            if state["head"] not in set(state["main_heads"].values()):
                raise ProfileError(
                    f"CI provenance HEAD is not a resolvable local main ref at profile.{route_name}"
                )


def learn_profile(
    root: str | os.PathLike[str], profile_json: str | os.PathLike[str], *, replace: bool = False
) -> Path:
    """Validate and atomically install one project-local profile without network access."""
    root_path = _normal_path(root)
    if not root_path.exists() or not root_path.is_dir():
        raise ProfileError(f"root must be an existing directory: {root_path}")
    source_path = _normal_path(profile_json)
    document, profiles = _read_document(
        source_path,
        require_one=True,
        require_local_proof=True,
    )
    profile_roots = {_normal_path(item) for item in profiles[0]["roots"]}
    if root_path not in profile_roots:
        raise ProfileError(f"profile roots must contain the exact resolved root: {root_path}")
    _bind_ready_routes(profiles[0], root_path)
    depot_dir = root_path / ".depot"
    depot_dir.mkdir(parents=True, exist_ok=True)
    destination = depot_dir / "offload-profile.json"
    if (destination.exists() or destination.is_symlink()) and not replace:
        raise ProfileError(f"local profile already exists: {destination}")
    payload = json.dumps(document, indent=2, ensure_ascii=False) + "\n"
    temporary: Optional[Path] = None
    try:
        descriptor, temporary_name = tempfile.mkstemp(
            prefix=".offload-profile.", suffix=".tmp", dir=str(depot_dir)
        )
        temporary = Path(temporary_name)
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, destination)
        temporary = None
    finally:
        if temporary is not None and temporary.exists():
            temporary.unlink()
    return destination


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Manage project-scoped Depot offload profiles")
    commands = parser.add_subparsers(dest="command", required=True)
    resolve = commands.add_parser("resolve", help="resolve a project route")
    resolve.add_argument("--cwd", required=True)
    resolve.add_argument("--need", choices=sorted(NEEDS), required=True)
    resolve.add_argument("--json", action="store_true")
    validate = commands.add_parser("validate", help="validate a profile document")
    validate.add_argument("path", nargs="?")
    learn = commands.add_parser("learn", help="install a verified local profile")
    learn.add_argument("--root", required=True)
    learn.add_argument("--profile-json", required=True)
    learn.add_argument("--replace", action="store_true")
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = _build_parser().parse_args(argv)
    try:
        if args.command == "resolve":
            result = resolve_profile(args.cwd, args.need)
            if args.json:
                print(json.dumps(result, indent=2, ensure_ascii=False))
            else:
                print(
                    f"{result['source']} {result['profile']['id']} "
                    f"{result['route']['status']} {result['resolved_root']}"
                )
            return 0
        if args.command == "validate":
            path = BUNDLED_PROFILE_PATH if args.path is None else _normal_path(args.path)
            profiles = validate_file(path, require_one=args.path is not None)
            print(f"valid {path} profiles={len(profiles)}")
            return 0
        destination = learn_profile(args.root, args.profile_json, replace=args.replace)
        print(f"learned {destination}")
        return 0
    except (OSError, ProfileError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
