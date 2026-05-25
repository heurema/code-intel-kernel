#!/usr/bin/env python3
"""Validate Research Radar reports and state files."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re
import subprocess
import sys


SECRET_PATTERNS = [
    re.compile(pattern, re.IGNORECASE)
    for pattern in [
        r"ghp_",
        r"github_pat_",
        r"token",
        r"authorization",
        r"bearer",
        r"cookie",
        r"api_key",
        r"access_token",
        r"private",
        r"password",
    ]
]
MAX_FILE_BYTES = 500_000
ALLOWED_CHANGED_PREFIXES = ("research-radar/reports/", "research-radar/state/")


def main() -> int:
    args = parse_args()
    radar_root = args.radar_root.resolve()
    errors: list[str] = []

    validate_json_files(radar_root, errors)
    validate_seen_jsonl(radar_root / "state" / "seen.jsonl", errors)
    validate_secret_patterns(radar_root, errors)
    validate_file_sizes(radar_root, errors)

    if args.allowlist_changes:
        validate_changed_paths(errors)

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("Research Radar reports validated.")
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Validate Research Radar reports/state.")
    parser.add_argument(
        "--radar-root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="Research Radar root.",
    )
    parser.add_argument(
        "--allowlist-changes",
        action="store_true",
        help="Fail if git changed paths are outside reports/state allowlist.",
    )
    return parser.parse_args()


def validate_json_files(radar_root: Path, errors: list[str]) -> None:
    for path in sorted((radar_root / "reports").glob("*.json")):
        parse_json(path, errors)
    for path in [radar_root / "state" / "source_health.json", radar_root / "state" / "last_run.json"]:
        if path.exists():
            parse_json(path, errors)


def parse_json(path: Path, errors: list[str]) -> None:
    try:
        with path.open("r", encoding="utf-8") as handle:
            json.load(handle)
    except Exception as error:  # noqa: BLE001 - validator reports all parse failures.
        errors.append(f"{path} is not valid JSON: {error}")


def validate_seen_jsonl(path: Path, errors: list[str]) -> None:
    if not path.exists():
        return
    with path.open("r", encoding="utf-8") as handle:
        for index, line in enumerate(handle, start=1):
            line = line.strip()
            if not line:
                continue
            try:
                json.loads(line)
            except Exception as error:  # noqa: BLE001
                errors.append(f"{path}:{index} is not valid JSONL: {error}")


def validate_secret_patterns(radar_root: Path, errors: list[str]) -> None:
    for path in list((radar_root / "reports").glob("*")) + list((radar_root / "state").glob("*")):
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for pattern in SECRET_PATTERNS:
            if pattern.search(text):
                errors.append(f"{path} contains possible secret pattern `{pattern.pattern}`")


def validate_file_sizes(radar_root: Path, errors: list[str]) -> None:
    for path in list((radar_root / "reports").glob("*")) + list((radar_root / "state").glob("*")):
        if path.is_file() and path.stat().st_size > MAX_FILE_BYTES:
            errors.append(f"{path} is too large; possible raw payload dump")


def validate_changed_paths(errors: list[str]) -> None:
    try:
        output = subprocess.check_output(
            ["git", "status", "--porcelain=v1"],
            text=True,
            stderr=subprocess.STDOUT,
        )
    except subprocess.CalledProcessError as error:
        errors.append(f"git status failed: {error.output}")
        return
    for line in output.splitlines():
        if not line:
            continue
        path = line[3:]
        if " -> " in path:
            path = path.split(" -> ", 1)[1]
        if not path.startswith(ALLOWED_CHANGED_PREFIXES):
            errors.append(f"changed path outside Research Radar report/state allowlist: {path}")


if __name__ == "__main__":
    raise SystemExit(main())
