#!/usr/bin/env python3
"""Validate the Research Radar shared-intake dependency lock."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from typing import Any


LOCK_SCHEMA_VERSION = "shared-intake-consumer.v1"
SHARED_CLI_MARKER = Path("src/shared_intake_governance/cli/__main__.py")


class DependencyError(RuntimeError):
    """Raised when the shared-intake dependency cannot be trusted."""


def repo_root_from_script() -> Path:
    return Path(__file__).resolve().parents[2]


def default_lock_path(consumer_repo_root: Path) -> Path:
    return consumer_repo_root / "research-radar" / "shared-intake.lock.json"


def load_lock(lock_path: Path) -> dict[str, Any]:
    try:
        payload = json.loads(lock_path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise DependencyError(f"lock file not found: {lock_path}") from exc
    except json.JSONDecodeError as exc:
        raise DependencyError(f"invalid lock JSON: {lock_path}: {exc}") from exc

    if not isinstance(payload, dict):
        raise DependencyError("lock file must contain a JSON object")
    if payload.get("schema_version") != LOCK_SCHEMA_VERSION:
        raise DependencyError(
            f"unsupported lock schema_version: {payload.get('schema_version')!r}"
        )

    upstream = _required_object(payload, "upstream")
    _required_string(upstream, "repository")
    _required_string(upstream, "pinned_commit")
    _required_string(upstream, "default_relative_path")

    paths = _required_object(payload, "paths")
    _required_string(paths, "profile")
    _required_string(paths, "sources_glob")
    return payload


def _required_object(payload: dict[str, Any], key: str) -> dict[str, Any]:
    value = payload.get(key)
    if not isinstance(value, dict):
        raise DependencyError(f"lock field {key!r} must be an object")
    return value


def _required_string(payload: dict[str, Any], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value.strip():
        raise DependencyError(f"lock field {key!r} must be a non-empty string")
    return value


def resolve_shared_repo_root(
    lock: dict[str, Any],
    *,
    consumer_repo_root: Path,
    explicit: Path | None = None,
    env: dict[str, str] | None = None,
) -> Path:
    current_env = os.environ if env is None else env
    candidates: list[Path] = []
    if explicit is not None:
        candidates.append(explicit)
    for env_name in ("SIG_SHARED_REPO_ROOT", "SHARED_INTAKE_ROOT"):
        env_root = current_env.get(env_name)
        if env_root:
            candidates.append(Path(env_root).expanduser())
    candidates.append(
        consumer_repo_root / lock["upstream"]["default_relative_path"]
    )

    for candidate in candidates:
        resolved = candidate.expanduser().resolve()
        if (resolved / SHARED_CLI_MARKER).exists():
            return resolved

    candidate_text = ", ".join(str(candidate) for candidate in candidates)
    raise DependencyError(
        "shared-intake-governance repo not found; pass --shared-repo-root "
        f"or set SIG_SHARED_REPO_ROOT. Checked: {candidate_text}"
    )


def actual_git_commit(shared_repo_root: Path) -> str:
    completed = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=shared_repo_root,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise DependencyError(
            "could not read shared-intake git commit: "
            + completed.stderr.strip()
        )
    return completed.stdout.strip()


def verify_pinned_commit(*, expected_commit: str, actual_commit: str) -> None:
    if expected_commit != actual_commit:
        raise DependencyError(
            "shared-intake pinned commit mismatch: "
            f"expected {expected_commit}, got {actual_commit}"
        )


def cli_env(shared_repo_root: Path) -> dict[str, str]:
    env = dict(os.environ)
    source_path = str(shared_repo_root / "src")
    existing = env.get("PYTHONPATH")
    env["PYTHONPATH"] = (
        source_path if not existing else source_path + os.pathsep + existing
    )
    return env


def run_shared_cli(shared_repo_root: Path, argv: list[str]) -> dict[str, Any]:
    completed = subprocess.run(
        [sys.executable, "-m", "shared_intake_governance.cli", *argv],
        cwd=shared_repo_root,
        capture_output=True,
        text=True,
        check=False,
        env=cli_env(shared_repo_root),
    )
    if completed.returncode != 0:
        raise DependencyError(
            "shared-intake CLI failed for "
            + " ".join(argv)
            + f"\nstdout:\n{completed.stdout}\nstderr:\n{completed.stderr}"
        )
    try:
        payload = json.loads(completed.stdout)
    except json.JSONDecodeError as exc:
        raise DependencyError(
            "shared-intake CLI did not return JSON for " + " ".join(argv)
        ) from exc
    if not isinstance(payload, dict):
        raise DependencyError("shared-intake CLI returned non-object JSON")
    return payload


def consumer_preflight_commands(
    lock: dict[str, Any], consumer_repo_root: Path
) -> list[list[str]]:
    profile_path = consumer_repo_root / lock["paths"]["profile"]
    source_paths = sorted(consumer_repo_root.glob(lock["paths"]["sources_glob"]))
    if not profile_path.exists():
        raise DependencyError(f"profile path not found: {profile_path}")
    if not source_paths:
        raise DependencyError(
            f"no source configs matched: {lock['paths']['sources_glob']}"
        )

    commands = [["inspect-profile", "--profile", str(profile_path)]]
    for source_path in source_paths:
        commands.append(
            ["inspect-source-config", "--source-config", str(source_path)]
        )
    return commands


def check_dependency(
    *,
    consumer_repo_root: Path,
    lock_path: Path | None = None,
    explicit_shared_root: Path | None = None,
) -> dict[str, Any]:
    resolved_consumer_root = consumer_repo_root.resolve()
    resolved_lock_path = (lock_path or default_lock_path(resolved_consumer_root)).resolve()
    lock = load_lock(resolved_lock_path)
    shared_repo_root = resolve_shared_repo_root(
        lock,
        consumer_repo_root=resolved_consumer_root,
        explicit=explicit_shared_root,
    )
    expected_commit = lock["upstream"]["pinned_commit"]
    actual_commit = actual_git_commit(shared_repo_root)
    verify_pinned_commit(
        expected_commit=expected_commit,
        actual_commit=actual_commit,
    )

    commands = consumer_preflight_commands(lock, resolved_consumer_root)
    checked_commands: list[list[str]] = []
    for command in commands:
        run_shared_cli(shared_repo_root, command)
        checked_commands.append(command)

    return {
        "status": "ok",
        "lock_path": str(resolved_lock_path),
        "shared_repo_root": str(shared_repo_root),
        "pinned_commit": expected_commit,
        "actual_commit": actual_commit,
        "profile_path": str(resolved_consumer_root / lock["paths"]["profile"]),
        "source_config_count": len(commands) - 1,
        "checked_commands": checked_commands,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate Research Radar's pinned shared-intake dependency."
    )
    parser.add_argument(
        "--repo-root",
        default=str(repo_root_from_script()),
        help="Consumer repository root. Defaults to this checkout.",
    )
    parser.add_argument("--lock", help="Path to shared-intake.lock.json.")
    parser.add_argument(
        "--shared-repo-root",
        help="Path to the shared-intake-governance checkout.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        summary = check_dependency(
            consumer_repo_root=Path(args.repo_root),
            lock_path=Path(args.lock) if args.lock else None,
            explicit_shared_root=Path(args.shared_repo_root)
            if args.shared_repo_root
            else None,
        )
    except DependencyError as exc:
        json.dump({"status": "error", "error": str(exc)}, sys.stderr)
        sys.stderr.write("\n")
        return 1

    json.dump(summary, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
