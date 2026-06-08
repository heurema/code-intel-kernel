#!/usr/bin/env python3
"""Run code-intel-kernel Research Radar in shared-intake shadow mode."""

from __future__ import annotations

import argparse
import datetime as dt
import json
from pathlib import Path
import sys
import tempfile
from typing import Any

from check_shared_intake_dependency import check_dependency, load_lock, run_shared_cli


KNOWN_GAPS = [
    "Shadow mode writes shared runtime artifacts only; it does not replace research-radar/reports output yet.",
    "The existing run_daily.py collector remains the scheduled Research Radar path until a separate cutover.",
    "The shared github_repo collector captures repository metadata only, not release or commit enrichment parity with run_daily.py.",
]


def main() -> int:
    args = parse_args()
    repo_root = (
        Path(args.repo_root).expanduser().resolve()
        if args.repo_root
        else Path(__file__).resolve().parents[2]
    )
    lock_path = (
        Path(args.lock).expanduser().resolve()
        if args.lock
        else repo_root / "research-radar" / "shared-intake.lock.json"
    )
    lock = load_lock(lock_path)
    dependency = check_dependency(
        consumer_repo_root=repo_root,
        lock_path=lock_path,
        explicit_shared_root=Path(args.shared_repo_root).expanduser()
        if args.shared_repo_root
        else None,
    )
    profile_path = (
        Path(args.profile).expanduser().resolve()
        if args.profile
        else repo_root / lock["paths"]["profile"]
    )
    sources_dir = (
        Path(args.sources_dir).expanduser().resolve()
        if args.sources_dir
        else (repo_root / lock["paths"]["sources_glob"]).parent
    )
    runtime_root = (
        Path(args.runtime_root).expanduser().resolve()
        if args.runtime_root
        else Path.home() / ".local" / "share" / "shared-intake-governance"
    )
    run_date = parse_date(args.date)

    summary = run_shadow(
        shared_repo_root=Path(dependency["shared_repo_root"]),
        dependency=dependency,
        runtime_root=runtime_root,
        profile_path=profile_path,
        sources_dir=sources_dir,
        run_date=run_date,
        source_ids=args.source_ids or [],
        output_id=args.output_id or f"{run_date.isoformat()}-shadow",
        update_seen_state=args.update_seen_state,
    )
    json.dump(summary, sys.stdout, indent=2, sort_keys=True)
    sys.stdout.write("\n")
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run Research Radar through shared-intake shadow mode."
    )
    parser.add_argument("--date", help="Run date in YYYY-MM-DD format.")
    parser.add_argument("--repo-root", help="Consumer repository root.")
    parser.add_argument("--runtime-root", help="Shared runtime root outside git.")
    parser.add_argument("--lock", help="Path to shared-intake.lock.json.")
    parser.add_argument(
        "--shared-repo-root",
        help="Path to the shared-intake-governance repository.",
    )
    parser.add_argument("--profile", help="Consumer-owned profile path.")
    parser.add_argument("--sources-dir", help="Directory with source-config templates.")
    parser.add_argument(
        "--source-id",
        dest="source_ids",
        action="append",
        help="Limit the run to one or more source_id values.",
    )
    parser.add_argument("--output-id", help="Projection output id.")
    parser.add_argument(
        "--update-seen-state",
        action="store_true",
        help="Merge projected record ids into the profile-local seen state.",
    )
    return parser.parse_args()


def parse_date(value: str | None) -> dt.date:
    if value:
        return dt.date.fromisoformat(value)
    return dt.datetime.now(dt.timezone.utc).date()


def build_placeholders(run_date: dt.date) -> dict[str, str]:
    return {
        "${TODAY}": run_date.isoformat(),
        "${TODAY_MINUS_2D}": (run_date - dt.timedelta(days=2)).isoformat(),
        "${TODAY_MINUS_7D}": (run_date - dt.timedelta(days=7)).isoformat(),
        "${TODAY_MINUS_30D}": (run_date - dt.timedelta(days=30)).isoformat(),
    }


def replace_placeholders(value: Any, placeholders: dict[str, str]) -> Any:
    if isinstance(value, str):
        output = value
        for placeholder, replacement in placeholders.items():
            output = output.replace(placeholder, replacement)
        return output
    if isinstance(value, list):
        return [replace_placeholders(item, placeholders) for item in value]
    if isinstance(value, dict):
        return {
            key: replace_placeholders(item, placeholders)
            for key, item in value.items()
        }
    return value


def discover_source_templates(
    sources_dir: Path, source_ids: list[str]
) -> list[tuple[str, Path]]:
    templates: list[tuple[str, Path]] = []
    for path in sorted(sources_dir.glob("*.json")):
        config = json.loads(path.read_text(encoding="utf-8"))
        source_id = str(config["source_id"])
        if source_ids and source_id not in source_ids:
            continue
        templates.append((source_id, path))

    if not templates:
        raise ValueError("no source configs matched the requested source ids")
    return templates


def materialize_source_config(
    template_path: Path,
    *,
    placeholders: dict[str, str],
    destination_dir: Path,
) -> Path:
    config = json.loads(template_path.read_text(encoding="utf-8"))
    materialized = replace_placeholders(config, placeholders)
    destination = destination_dir / template_path.name
    destination.write_text(
        json.dumps(materialized, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return destination


def source_run_id(run_date: dt.date, source_id: str) -> str:
    return f"{run_date.strftime('%Y%m%d')}-shadow-{source_id}"


def run_shadow(
    *,
    shared_repo_root: Path,
    dependency: dict[str, Any],
    runtime_root: Path,
    profile_path: Path,
    sources_dir: Path,
    run_date: dt.date,
    source_ids: list[str],
    output_id: str,
    update_seen_state: bool,
) -> dict[str, Any]:
    placeholders = build_placeholders(run_date)
    source_templates = discover_source_templates(sources_dir, source_ids)
    runtime_root.mkdir(parents=True, exist_ok=True)

    source_runs: list[dict[str, Any]] = []
    with tempfile.TemporaryDirectory(prefix="shared-shadow-configs-") as tmp_dir:
        materialized_dir = Path(tmp_dir)
        for source_id, template_path in source_templates:
            materialized_path = materialize_source_config(
                template_path,
                placeholders=placeholders,
                destination_dir=materialized_dir,
            )
            run_id = source_run_id(run_date, source_id)
            result = run_shared_cli(
                shared_repo_root,
                [
                    "run-source-config",
                    "--runtime-root",
                    str(runtime_root),
                    "--profile",
                    str(profile_path),
                    "--source-config",
                    str(materialized_path),
                    "--run-id",
                    run_id,
                    "--output-id",
                    run_id,
                ],
            )
            source_runs.append(
                {
                    "source_id": source_id,
                    "template_path": str(template_path),
                    "run_summary": result,
                }
            )

    project_argv = [
        "project-profiles",
        "--runtime-root",
        str(runtime_root),
        "--profile",
        str(profile_path),
        "--output-id",
        output_id,
    ]
    if update_seen_state:
        project_argv.append("--update-seen-state")
    projection = run_shared_cli(shared_repo_root, project_argv)

    return {
        "run_date": run_date.isoformat(),
        "shared_dependency": dependency,
        "runtime_root": str(runtime_root),
        "profile_path": str(profile_path),
        "sources_dir": str(sources_dir),
        "source_count": len(source_runs),
        "source_runs": source_runs,
        "projection": projection,
        "known_gaps": KNOWN_GAPS,
    }


if __name__ == "__main__":
    raise SystemExit(main())
