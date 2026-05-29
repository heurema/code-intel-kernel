#!/usr/bin/env python3
"""Deterministic Research Radar collector.

This script collects public research signals and writes normalized reports.
It does not import external code, create prototypes, or modify runtime files.
"""

from __future__ import annotations

import argparse
import datetime as dt
import email.utils
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import sys
import textwrap
import time
from typing import Any
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET


CONTRACT_VERSION = "0.1"
USER_AGENT = "code-intel-kernel-research-radar/0.1"
TIMEOUT_SECONDS = 20
ARXIV_TIMEOUT_SECONDS = 75
ARXIV_MAX_ATTEMPTS = 3
MAX_TOP_ITEMS = 3
MAX_ARCHIVE_ITEMS = 10
MAX_SOURCE_ITEMS = 10
PROTOTYPE_THRESHOLD = 85
_GITHUB_TOKEN_CACHE: str | None | bool = False
_HOST_LAST_REQUEST_AT: dict[str, float] = {}


def main() -> int:
    args = parse_args()
    radar_root = args.radar_root.resolve()
    run_date = parse_date(args.date)
    context = RunContext(radar_root=radar_root, run_date=run_date)
    manifest = load_json(radar_root / "sources.automation.json")
    seen = load_seen(radar_root / "state" / "seen.jsonl")

    report = collect_report(context, manifest, seen)

    if args.dry_run:
        json.dump(report, sys.stdout, indent=2, sort_keys=True)
        sys.stdout.write("\n")
        return 0

    if args.write:
        write_outputs(context, report, seen)
        return 0

    print("Use --dry-run or --write.", file=sys.stderr)
    return 2


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run Research Radar daily collection.")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--dry-run", action="store_true", help="Collect and print report JSON.")
    mode.add_argument("--write", action="store_true", help="Collect and write report/state files.")
    parser.add_argument("--date", help="Report date in YYYY-MM-DD format.")
    parser.add_argument(
        "--radar-root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="Research Radar root; intended for tests and local dry runs.",
    )
    return parser.parse_args()


def parse_date(value: str | None) -> dt.date:
    if value:
        return dt.date.fromisoformat(value)
    return dt.datetime.now(dt.timezone.utc).date()


class RunContext:
    def __init__(self, radar_root: Path, run_date: dt.date) -> None:
        self.radar_root = radar_root
        self.run_date = run_date
        self.placeholders = {
            "${TODAY}": run_date.isoformat(),
            "${TODAY_MINUS_2D}": (run_date - dt.timedelta(days=2)).isoformat(),
            "${TODAY_MINUS_7D}": (run_date - dt.timedelta(days=7)).isoformat(),
            "${TODAY_MINUS_30D}": (run_date - dt.timedelta(days=30)).isoformat(),
        }

    def replace_placeholders(self, value: str) -> str:
        output = value
        for placeholder, replacement in self.placeholders.items():
            output = output.replace(placeholder, replacement)
        return output


def load_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def load_seen(path: Path) -> dict[str, dict]:
    seen: dict[str, dict] = {}
    if not path.exists():
        return seen
    with path.open("r", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if not line:
                continue
            record = json.loads(line)
            key = record.get("id") or stable_id(record.get("canonical_url", ""))
            seen[key] = record
            if record.get("canonical_url"):
                seen[record["canonical_url"]] = record
    return seen


def collect_report(context: RunContext, manifest: dict, seen: dict[str, dict]) -> dict:
    all_items: list[dict] = []
    source_health: list[dict] = []
    errors: list[dict] = []

    for source in manifest.get("sources", []):
        items, health = collect_source(context, source)
        source_health.append(health)
        if health.get("status") == "error":
            errors.append({"source": source["id"], "message": health.get("error", "unknown error")})
        all_items.extend(items)

    normalized_items = sorted(
        all_items,
        key=lambda item: (-item["score"], item["source_id"], item["title"], item["canonical_url"]),
    )
    new_items = [
        item
        for item in normalized_items
        if item["id"] not in seen and item["canonical_url"] not in seen
    ]
    top_items = [item for item in new_items if item["score"] >= 70][:MAX_TOP_ITEMS]
    archive_items = [item for item in new_items if item["score"] < 70][:MAX_ARCHIVE_ITEMS]
    experiment_candidate = next((item for item in top_items if item["score"] >= PROTOTYPE_THRESHOLD), None)

    if experiment_candidate is not None:
        experiment_candidate = build_experiment_candidate(experiment_candidate)

    return {
        "contract_version": CONTRACT_VERSION,
        "date": context.run_date.isoformat(),
        "sources_checked": [
            {
                "id": health["id"],
                "name": health["name"],
                "status": health["status"],
                "errors": [health["error"]] if health.get("error") else [],
            }
            for health in source_health
        ],
        "source_health": source_health,
        "new_items": public_items(new_items),
        "top_items": public_items(top_items),
        "archive_items": public_archive_items(archive_items),
        "experiment_candidate": experiment_candidate,
        "errors": errors,
        "next_actions": build_next_actions(source_health, top_items, experiment_candidate),
        "guardrails": {
            "runtime_code_modified": False,
            "automation_created_code_changes": False,
            "external_code_imported": False,
            "implementation_triggered": False,
            "prototype_threshold": PROTOTYPE_THRESHOLD,
            "experiment_candidate_is_code": False,
        },
        "automation_notes": [
            "Collector uses configured public sources only.",
            "Reports are normalized summaries, not raw API payloads.",
            "No item may trigger implementation automatically.",
        ],
    }


def collect_source(context: RunContext, source: dict) -> tuple[list[dict], dict]:
    source_type = source.get("type")
    try:
        if source_type == "github_repo":
            return collect_github_repo(source)
        if source_type == "github_search":
            return collect_github_search(context, source)
        if source_type == "arxiv_query":
            return collect_arxiv_query(context, source)
        if source_type == "arxiv_rss_keywords":
            return collect_arxiv_rss_keywords(context, source)
        return [], source_health(source, "error", error=f"Unsupported source type: {source_type}")
    except Exception as error:  # noqa: BLE001 - command-line collector must keep other sources running.
        return [], source_health(source, "error", error=str(error))


def collect_github_repo(source: dict) -> tuple[list[dict], dict]:
    try:
        return collect_github_repo_api(source)
    except RuntimeError as error:
        if not is_github_rate_limit_error(error):
            raise
        return collect_github_repo_public_fallback(source)


def collect_github_repo_api(source: dict) -> tuple[list[dict], dict]:
    repo = source["repo"]
    repo_meta = fetch_json(f"https://api.github.com/repos/{repo}")
    items: list[dict] = []
    license_id = (repo_meta.get("license") or {}).get("spdx_id") or source.get("license_hint") or "unknown"

    items.append(
        normalize_item(
            source=source,
            item_type="github_repo",
            title=f"{repo} repository activity",
            canonical_url=repo_meta.get("html_url") or source["url"],
            summary=repo_meta.get("description") or "",
            published_at=repo_meta.get("updated_at"),
            license_note=f"{license_id} reported by GitHub API.",
            score_delta=0,
        )
    )

    for release in fetch_json_list(f"https://api.github.com/repos/{repo}/releases?per_page={source.get('max_releases', 1)}"):
        items.append(
            normalize_item(
                source=source,
                item_type="github_release",
                title=f"{repo} {release.get('tag_name') or release.get('name') or 'release'}",
                canonical_url=release.get("html_url") or source["url"],
                summary=release.get("name") or "",
                published_at=release.get("published_at"),
                license_note=f"{license_id} reported by GitHub API.",
                score_delta=4,
            )
        )

    max_commits = min(int(source.get("max_commits", 3)), MAX_SOURCE_ITEMS)
    for commit in fetch_json_list(f"https://api.github.com/repos/{repo}/commits?per_page={max_commits}"):
        commit_info = commit.get("commit") or {}
        commit_message = (commit_info.get("message") or "").splitlines()[0]
        items.append(
            normalize_item(
                source=source,
                item_type="github_commit",
                title=f"{repo} commit {commit.get('sha', '')[:7]}",
                canonical_url=commit.get("html_url") or source["url"],
                summary=commit_message,
                published_at=(commit_info.get("author") or {}).get("date"),
                license_note=f"{license_id} reported by GitHub API.",
                score_delta=-6,
            )
        )

    health = source_health(
        source,
        "ok",
        canonical_url=repo_meta.get("html_url") or source["url"],
        observed_at=repo_meta.get("updated_at"),
        license_note=license_id,
        checked_via="github_rest_api",
    )
    return bounded(items), health


def collect_github_repo_public_fallback(source: dict) -> tuple[list[dict], dict]:
    repo = source["repo"]
    repo_url = source["url"]
    license_id = source.get("license_hint") or "unknown"
    summary = fetch_github_repo_summary(repo_url)
    commits = fetch_github_atom_feed(f"https://github.com/{repo}/commits.atom")
    releases = fetch_github_atom_feed(
        f"https://github.com/{repo}/releases.atom",
        allow_not_found=True,
    )
    observed_at = commits[0]["published_at"] if commits else None
    items: list[dict] = [
        normalize_item(
            source=source,
            item_type="github_repo",
            title=f"{repo} repository activity",
            canonical_url=repo_url,
            summary=summary,
            published_at=observed_at,
            license_note=f"{license_id} from source manifest; GitHub API rate-limited.",
            score_delta=0,
        )
    ]

    for release in releases[: min(int(source.get("max_releases", 1)), MAX_SOURCE_ITEMS)]:
        items.append(
            normalize_item(
                source=source,
                item_type="github_release",
                title=release["title"],
                canonical_url=release["canonical_url"],
                summary=release["summary"],
                published_at=release["published_at"],
                license_note=f"{license_id} from source manifest; GitHub API rate-limited.",
                score_delta=4,
            )
        )

    for commit in commits[: min(int(source.get("max_commits", 3)), MAX_SOURCE_ITEMS)]:
        items.append(
            normalize_item(
                source=source,
                item_type="github_commit",
                title=commit["title"],
                canonical_url=commit["canonical_url"],
                summary=commit["summary"],
                published_at=commit["published_at"],
                license_note=f"{license_id} from source manifest; GitHub API rate-limited.",
                score_delta=-6,
            )
        )

    return bounded(items), source_health(
        source,
        "ok",
        canonical_url=repo_url,
        observed_at=observed_at,
        license_note=license_id,
        checked_via="github_public_feeds",
    )


def collect_github_search(context: RunContext, source: dict) -> tuple[list[dict], dict]:
    query = context.replace_placeholders(source["query"])
    max_items = min(int(source.get("max_items", 5)), MAX_SOURCE_ITEMS)
    encoded = urllib.parse.urlencode({"q": query, "sort": "updated", "order": "desc", "per_page": str(max_items)})
    data = fetch_json(f"https://api.github.com/search/repositories?{encoded}")
    items = []
    for repo in data.get("items", [])[:max_items]:
        license_id = (repo.get("license") or {}).get("spdx_id") or "needs_license_review"
        items.append(
            normalize_item(
                source=source,
                item_type="github_search_result",
                title=repo.get("full_name") or repo.get("name") or "GitHub repository",
                canonical_url=repo.get("html_url"),
                summary=repo.get("description") or "",
                published_at=repo.get("updated_at"),
                license_note=f"{license_id} reported by GitHub API.",
                score_delta=-2,
            )
        )
    return bounded(items), source_health(
        source,
        "ok",
        canonical_url="https://github.com/search",
        observed_at=context.run_date.isoformat(),
        checked_via="github_search_api",
    )


def collect_arxiv_query(context: RunContext, source: dict) -> tuple[list[dict], dict]:
    max_items = min(int(source.get("max_items", 5)), MAX_SOURCE_ITEMS)
    search_query = context.replace_placeholders(source["search_query"])
    submitted_after = parse_date(context.replace_placeholders(source.get("submitted_after", "1970-01-01")))
    params = urllib.parse.urlencode(
        {
            "search_query": search_query,
            "start": "0",
            "max_results": str(max_items),
            "sortBy": "submittedDate",
            "sortOrder": "descending",
        }
    )
    body = fetch_text(
        f"https://export.arxiv.org/api/query?{params}",
        accept="application/atom+xml",
        timeout=ARXIV_TIMEOUT_SECONDS,
        max_attempts=ARXIV_MAX_ATTEMPTS,
        retry_statuses={429, 503},
    )
    root = ET.fromstring(body)
    ns = {"atom": "http://www.w3.org/2005/Atom"}
    items = []
    for entry in root.findall("atom:entry", ns):
        published = text_of(entry, "atom:published", ns)
        published_date = parse_date(published[:10]) if published else dt.date.min
        if published_date < submitted_after:
            continue
        title = " ".join(text_of(entry, "atom:title", ns).split())
        arxiv_id = text_of(entry, "atom:id", ns)
        summary = " ".join(text_of(entry, "atom:summary", ns).split())[:500]
        items.append(
            normalize_item(
                source=source,
                item_type="arxiv_paper",
                title=title,
                canonical_url=arxiv_id,
                summary=summary,
                published_at=published,
                license_note="Paper terms need review before reuse.",
                score_delta=2,
            )
        )
    return bounded(items), source_health(
        source,
        "ok",
        canonical_url="https://export.arxiv.org/api/query",
        observed_at=context.run_date.isoformat(),
        checked_via="arxiv_atom_api",
    )


def collect_arxiv_rss_keywords(context: RunContext, source: dict) -> tuple[list[dict], dict]:
    submitted_after = parse_date(context.replace_placeholders(source.get("submitted_after", "1970-01-01")))
    categories = source.get("rss_categories", [])
    keywords = [keyword.lower() for keyword in source.get("keywords", [])]
    per_feed_items = min(int(source.get("per_feed_items", 25)), 100)
    filtered: dict[str, dict] = {}
    last_observed_at = None
    for category in categories:
        feed_items = fetch_arxiv_rss_feed(category, limit=per_feed_items)
        for entry in feed_items:
            published = entry.get("published_at")
            published_date = parse_date(published[:10]) if published else dt.date.min
            if published_date < submitted_after:
                continue
            haystack = f"{entry['title']} {entry['summary']}".lower()
            if keywords and not any(keyword in haystack for keyword in keywords):
                continue
            item = normalize_item(
                source=source,
                item_type="arxiv_paper",
                title=entry["title"],
                canonical_url=entry["canonical_url"],
                summary=entry["summary"],
                published_at=published,
                license_note="Paper terms need review before reuse.",
                score_delta=2,
            )
            existing = filtered.get(item["canonical_url"])
            if existing is None or item["score"] > existing["score"]:
                filtered[item["canonical_url"]] = item
            if last_observed_at is None or (published and published > last_observed_at):
                last_observed_at = published
    return bounded(list(filtered.values())), source_health(
        source,
        "ok",
        canonical_url="https://rss.arxiv.org/",
        observed_at=last_observed_at or context.run_date.isoformat(),
        checked_via="arxiv_rss_keyword_filter",
    )


def fetch_json_list(url: str) -> list[dict]:
    data = fetch_json(url)
    return data if isinstance(data, list) else []


def fetch_arxiv_rss_feed(category: str, *, limit: int) -> list[dict]:
    body = fetch_text(
        f"https://rss.arxiv.org/rss/{category}",
        accept="application/rss+xml",
        timeout=ARXIV_TIMEOUT_SECONDS,
        max_attempts=ARXIV_MAX_ATTEMPTS,
        retry_statuses={429, 503},
    )
    root = ET.fromstring(body)
    channel = root.find("channel")
    if channel is None:
        return []
    items = []
    for item in channel.findall("item")[:limit]:
        title = " ".join((item.findtext("title", "") or "").split())
        canonical_url = (item.findtext("link", "") or "").strip()
        summary = normalize_rss_description(item.findtext("description", "") or "")
        pub_date = parse_rss_pub_date(item.findtext("pubDate", "") or "")
        items.append(
            {
                "title": title,
                "canonical_url": canonical_url,
                "summary": summary,
                "published_at": pub_date,
            }
        )
    return items


def fetch_json(url: str) -> dict | list:
    body = fetch_text(url)
    return json.loads(body)


def fetch_text(
    url: str,
    *,
    accept: str = "application/json",
    timeout: int = TIMEOUT_SECONDS,
    max_attempts: int = 1,
    retry_statuses: set[int] | None = None,
) -> str:
    retry_statuses = retry_statuses or set()
    token = resolve_github_token() if url.startswith("https://api.github.com/") else None
    for attempt in range(1, max_attempts + 1):
        maybe_throttle_request(url)
        headers = {"User-Agent": USER_AGENT, "Accept": accept}
        if token:
            headers["Authorization"] = f"Bearer {token}"
        request = urllib.request.Request(url, headers=headers)
        try:
            with urllib.request.urlopen(request, timeout=timeout) as response:
                mark_request_complete(url)
                return response.read().decode("utf-8", errors="replace")
        except urllib.error.HTTPError as error:
            if url.startswith("https://api.github.com/") and error.code in {401, 403}:
                fallback = fetch_with_gh_api(url)
                if fallback is not None:
                    return fallback
            if error.code in retry_statuses and attempt < max_attempts:
                delay = compute_retry_delay(url, attempt, error.headers)
                time.sleep(delay)
                continue
            raise RuntimeError(f"HTTP {error.code} for {redact_url(url)}") from error
        except urllib.error.URLError as error:
            if is_timeout_error(error.reason) and attempt < max_attempts:
                delay = compute_retry_delay(url, attempt, None)
                time.sleep(delay)
                continue
            raise RuntimeError(f"URL error for {redact_url(url)}: {error.reason}") from error
        except TimeoutError as error:
            if attempt < max_attempts:
                delay = compute_retry_delay(url, attempt, None)
                time.sleep(delay)
                continue
            raise RuntimeError("The read operation timed out") from error
        except socket.timeout as error:
            if attempt < max_attempts:
                delay = compute_retry_delay(url, attempt, None)
                time.sleep(delay)
                continue
            raise RuntimeError("The read operation timed out") from error
    raise RuntimeError(f"Request retries exhausted for {redact_url(url)}")


def fetch_with_gh_api(url: str) -> str | None:
    parsed = urllib.parse.urlsplit(url)
    path = parsed.path.lstrip("/")
    if parsed.query:
        path = f"{path}?{parsed.query}"
    try:
        return subprocess.check_output(
            ["gh", "api", path],
            text=True,
            stderr=subprocess.DEVNULL,
            timeout=TIMEOUT_SECONDS,
        )
    except (FileNotFoundError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        return None


def resolve_github_token() -> str | None:
    global _GITHUB_TOKEN_CACHE
    if _GITHUB_TOKEN_CACHE is not False:
        return _GITHUB_TOKEN_CACHE or None
    token = os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")
    if token:
        _GITHUB_TOKEN_CACHE = token
        return token
    try:
        token = subprocess.check_output(
            ["gh", "auth", "token"],
            text=True,
            stderr=subprocess.DEVNULL,
            timeout=TIMEOUT_SECONDS,
        ).strip()
    except (FileNotFoundError, subprocess.CalledProcessError, subprocess.TimeoutExpired):
        token = ""
    _GITHUB_TOKEN_CACHE = token or None
    return _GITHUB_TOKEN_CACHE


def maybe_throttle_request(url: str) -> None:
    parsed = urllib.parse.urlsplit(url)
    if parsed.netloc != "export.arxiv.org":
        return
    now = time.monotonic()
    last_request_at = _HOST_LAST_REQUEST_AT.get(parsed.netloc)
    if last_request_at is None:
        return
    elapsed = now - last_request_at
    min_delay = 3.0
    if elapsed < min_delay:
        time.sleep(min_delay - elapsed)


def mark_request_complete(url: str) -> None:
    parsed = urllib.parse.urlsplit(url)
    _HOST_LAST_REQUEST_AT[parsed.netloc] = time.monotonic()


def compute_retry_delay(url: str, attempt: int, headers: Any | None) -> float:
    parsed = urllib.parse.urlsplit(url)
    retry_after = parse_retry_after(headers)
    if retry_after is not None:
        return max(retry_after, 1.0)
    if parsed.netloc == "export.arxiv.org":
        return min(15.0 * attempt, 45.0)
    return float(min(attempt, 3))


def parse_retry_after(headers: Any | None) -> float | None:
    if headers is None:
        return None
    value = headers.get("Retry-After")
    if not value:
        return None
    try:
        return float(value)
    except ValueError:
        return None


def fetch_github_repo_summary(repo_url: str) -> str:
    body = fetch_text(repo_url, accept="text/html")
    for pattern in [
        r'<meta[^>]+property="og:description"[^>]+content="([^"]*)"',
        r'<meta[^>]+name="description"[^>]+content="([^"]*)"',
    ]:
        match = re.search(pattern, body, flags=re.IGNORECASE)
        if match:
            return html_unescape(match.group(1)).strip()
    return ""


def fetch_github_atom_feed(url: str, *, allow_not_found: bool = False) -> list[dict]:
    try:
        body = fetch_text(url, accept="application/atom+xml")
    except RuntimeError as error:
        if allow_not_found and str(error).startswith("HTTP 404 "):
            return []
        raise
    root = ET.fromstring(body)
    ns = {"atom": "http://www.w3.org/2005/Atom"}
    items = []
    for entry in root.findall("atom:entry", ns):
        title = " ".join(text_of(entry, "atom:title", ns).split())
        canonical_url = ""
        summary = " ".join(text_of(entry, "atom:content", ns).split())
        published_at = text_of(entry, "atom:updated", ns) or text_of(entry, "atom:published", ns)
        link = entry.find("atom:link", ns)
        if link is not None:
            canonical_url = link.attrib.get("href", "")
        items.append(
            {
                "title": title,
                "canonical_url": canonical_url,
                "summary": summary,
                "published_at": published_at,
            }
        )
    return items


def is_github_rate_limit_error(error: RuntimeError) -> bool:
    message = str(error)
    return message.startswith("HTTP 403 for https://api.github.com/")


def is_timeout_error(reason: object) -> bool:
    if isinstance(reason, TimeoutError | socket.timeout):
        return True
    return "timed out" in str(reason).lower()


def html_unescape(value: str) -> str:
    return (
        value.replace("&amp;", "&")
        .replace("&quot;", '"')
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
    )


def normalize_rss_description(value: str) -> str:
    text = re.sub(r"<[^>]+>", " ", value)
    text = html_unescape(text)
    return " ".join(text.split())[:500]


def parse_rss_pub_date(value: str) -> str | None:
    if not value.strip():
        return None
    return email.utils.parsedate_to_datetime(value).astimezone(dt.timezone.utc).isoformat().replace("+00:00", "Z")


def normalize_item(
    *,
    source: dict,
    item_type: str,
    title: str,
    canonical_url: str | None,
    summary: str,
    published_at: str | None,
    license_note: str,
    score_delta: int,
) -> dict:
    canonical = canonical_url or source.get("url") or stable_url(title)
    score = score_item(source, item_type, title, summary, score_delta)
    action = suggested_action(score, item_type, source.get("id", ""))
    return {
        "id": stable_id(f"{source['id']}|{item_type}|{canonical}|{title}"),
        "title": title.strip() or source["name"],
        "source": source["name"],
        "source_id": source["id"],
        "item_type": item_type,
        "canonical_url": canonical,
        "summary": summary.strip(),
        "published_at": published_at,
        "score": score,
        "affected_module": source.get("affected_module", "research only"),
        "suggested_action": action,
        "confidence": confidence_for(score),
        "estimated_cost": estimated_cost(action),
        "expected_upside": expected_upside(source, item_type),
        "reason_not_now": reason_not_now(score, source),
        "license_or_terms_note": license_note or "needs_license_review",
    }


def score_item(source: dict, item_type: str, title: str, summary: str, score_delta: int) -> int:
    score = int(source.get("priority", 60)) + score_delta
    text = f"{title} {summary}".lower()
    if any(term in text for term in ["mcp", "sqlite", "call graph", "knowledge graph"]):
        score += 2
    if any(term in text for term in ["diagnostic", "reference", "definition", "lsp"]):
        score += 2
    if item_type == "arxiv_paper":
        score += 2
    if "github_search" in source.get("id", ""):
        score -= 3
    return max(0, min(score, 84))


def suggested_action(score: int, item_type: str, source_id: str) -> str:
    if score >= PROTOTYPE_THRESHOLD:
        return "prototype"
    if "codebase-memory" in source_id:
        return "create design note"
    if item_type in {"github_release", "github_repo"} and score >= 75:
        return "add eval case"
    if item_type in {"arxiv_paper", "github_search_result"}:
        return "compare"
    if score >= 70:
        return "read"
    if score >= 55:
        return "archive"
    return "ignore"


def confidence_for(score: int) -> str:
    if score >= 80:
        return "high"
    if score >= 70:
        return "medium"
    return "low"


def estimated_cost(action: str) -> str:
    return {
        "create design note": "low for design note, high for implementation",
        "add eval case": "medium",
        "compare": "low to medium",
        "read": "low",
        "archive": "low",
        "ignore": "low",
        "prototype": "requires human approval",
    }.get(action, "unknown")


def expected_upside(source: dict, item_type: str) -> str:
    if source["id"] == "codebase-memory":
        return "Clarify graph/source/MCP boundaries without importing architecture."
    if source["id"] == "rust-analyzer":
        return "Reduce LSP bridge brittleness before references or definitions."
    if source["id"] == "tree-sitter":
        return "Avoid stale parser assumptions before SymbolGraph hardening."
    if item_type == "arxiv_paper":
        return "Surface research direction before implementation planning."
    return "Identify relevant research input with bounded follow-up."


def reason_not_now(score: int, source: dict) -> str:
    if score < PROTOTYPE_THRESHOLD:
        return f"Score {score} is below the {PROTOTYPE_THRESHOLD} prototype threshold."
    return "Even high-scoring items require human approval before any prototype."


def build_experiment_candidate(item: dict) -> dict:
    return {
        "title": item["title"],
        "source_url": item["canonical_url"],
        "source_type": item["item_type"],
        "hypothesis": "Needs human review before any implementation.",
        "minimal_reversible_change": "Experiment proposal only; no runtime code changes.",
        "evaluation_plan": "Define fixtures or metrics before prototype.",
        "stop_condition": "Stop if evidence is unclear, unsafe, or too broad.",
        "reason_not_to_implement_immediately": item["reason_not_now"],
    }


def public_items(items: list[dict]) -> list[dict]:
    keys = [
        "title",
        "source",
        "canonical_url",
        "score",
        "affected_module",
        "suggested_action",
        "confidence",
        "estimated_cost",
        "expected_upside",
        "reason_not_now",
        "license_or_terms_note",
    ]
    return [{key: item.get(key) for key in keys} for item in items]


def public_archive_items(items: list[dict]) -> list[dict]:
    return [
        {
            "title": item["title"],
            "source": item["source"],
            "canonical_url": item["canonical_url"],
            "score": item["score"],
            "reason": item["reason_not_now"],
        }
        for item in items
    ]


def build_next_actions(source_health: list[dict], top_items: list[dict], experiment_candidate: dict | None) -> list[str]:
    actions = []
    if any(health["status"] == "error" for health in source_health):
        actions.append("Review source_health errors before expanding source coverage.")
    if top_items:
        actions.append("Review top items manually; do not implement from report output.")
    if experiment_candidate is None:
        actions.append("No experiment candidate crossed the prototype threshold.")
    return actions or ["No action required."]


def write_outputs(context: RunContext, report: dict, seen: dict[str, dict]) -> None:
    reports_dir = context.radar_root / "reports"
    state_dir = context.radar_root / "state"
    reports_dir.mkdir(parents=True, exist_ok=True)
    state_dir.mkdir(parents=True, exist_ok=True)
    date = context.run_date.isoformat()

    write_json(reports_dir / f"{date}.json", report)
    write_markdown_report(reports_dir / f"{date}.md", report)
    write_json(state_dir / "source_health.json", {"date": date, "sources": report["source_health"]})
    write_json(
        state_dir / "last_run.json",
        {
            "date": date,
            "kind": "research_radar_automation",
            "report_md": f"research-radar/reports/{date}.md",
            "report_json": f"research-radar/reports/{date}.json",
            "runtime_code_modified": False,
            "automation_created": True,
            "implementation_triggered": False,
            "experiment_candidate": report["experiment_candidate"],
        },
    )
    append_seen(state_dir / "seen.jsonl", report["new_items"], seen, date)


def write_json(path: Path, value: dict) -> None:
    with path.open("w", encoding="utf-8") as handle:
        json.dump(value, handle, indent=2, sort_keys=False)
        handle.write("\n")


def write_markdown_report(path: Path, report: dict) -> None:
    lines = [
        "# Research Radar Daily Digest",
        "",
        f"Date: `{report['date']}`",
        "",
        "## Sources Checked",
        "",
        "| Source | Status | Notes |",
        "| --- | --- | --- |",
    ]
    for health in report["source_health"]:
        note = health.get("error") or health.get("canonical_url") or ""
        lines.append(f"| {health['name']} | {health['status']} | {note} |")
    lines.extend(["", "## New Items", ""])
    if report["new_items"]:
        for item in report["new_items"]:
            lines.extend(markdown_item(item))
    else:
        lines.append("No new items after dedupe.")
    lines.extend(["", "## Top 3 R&D Ideas", ""])
    if report["top_items"]:
        for index, item in enumerate(report["top_items"], start=1):
            lines.append(f"{index}. {item['title']}")
            lines.append(f"   - Canonical URL: {item['canonical_url']}")
            lines.append(f"   - Source: {item['source']}")
            lines.append(f"   - Score: {item['score']}")
            lines.append(f"   - Affected module: {item['affected_module']}")
            lines.append(f"   - Suggested action: {item['suggested_action']}")
            lines.append(f"   - Reason not now: {item['reason_not_now']}")
    else:
        lines.append("No top items.")
    lines.extend(["", "## Top 1 Experiment Candidate", ""])
    if report["experiment_candidate"]:
        lines.append(json.dumps(report["experiment_candidate"], indent=2))
    else:
        lines.append("None. No item crossed the prototype threshold.")
    lines.extend(["", "## Archive or Ignore Items", ""])
    if report["archive_items"]:
        for item in report["archive_items"]:
            lines.append(f"- {item['title']}: {item['reason']}")
    else:
        lines.append("None.")
    lines.extend(
        [
            "",
            "## Risks and Security Concerns",
            "",
            "- External code remains research input only.",
            "- License and terms notes are preserved where available.",
            "- No report item may trigger implementation automatically.",
            "",
            "## Do Not Act Yet",
            "",
            "- Do not import external code.",
            "- Do not create patches from this report.",
            "- Do not modify runtime code from Research Radar output.",
            "",
            "## Follow-Up Queue",
            "",
        ]
    )
    for action in report["next_actions"]:
        lines.append(f"- {action}")
    lines.extend(
        [
            "",
            "## Automation Notes",
            "",
        ]
    )
    for note in report["automation_notes"]:
        lines.append(f"- {note}")
    lines.extend(["", "## Guardrail Status", ""])
    for key, value in report["guardrails"].items():
        lines.append(f"- {key}: `{json.dumps(value)}`")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def markdown_item(item: dict) -> list[str]:
    return [
        f"- {item['title']}",
        f"  - Source: {item['source']}",
        f"  - URL: {item['canonical_url']}",
        f"  - Score: {item['score']}",
        f"  - Suggested action: {item['suggested_action']}",
        f"  - Reason not now: {item['reason_not_now']}",
        "",
    ]


def append_seen(path: Path, items: list[dict], seen: dict[str, dict], run_date: str) -> None:
    with path.open("a", encoding="utf-8") as handle:
        for item in items:
            item_id = stable_id(f"{item['source']}|{item['canonical_url']}|{item['title']}")
            if item_id in seen:
                continue
            record = {
                "id": item_id,
                "canonical_url": item["canonical_url"],
                "title": item["title"],
                "source_id": item["source"],
                "first_seen_date": run_date,
                "last_seen_date": run_date,
                "item_type": "research_item",
                "score": item["score"],
            }
            handle.write(json.dumps(record, sort_keys=True) + "\n")


def source_health(
    source: dict,
    status: str,
    *,
    canonical_url: str | None = None,
    observed_at: str | None = None,
    license_note: str | None = None,
    checked_via: str | None = None,
    error: str | None = None,
) -> dict:
    output = {
        "id": source["id"],
        "name": source["name"],
        "status": status,
        "canonical_url": canonical_url or source.get("url"),
    }
    if observed_at:
        output["observed_at"] = observed_at
    if license_note:
        output["license_or_terms_note"] = license_note
    if checked_via:
        output["checked_via"] = checked_via
    if error:
        output["error"] = error
    return output


def bounded(items: list[dict]) -> list[dict]:
    return sorted(items, key=lambda item: (-item["score"], item["title"], item["canonical_url"]))[:MAX_SOURCE_ITEMS]


def text_of(entry: ET.Element, path: str, ns: dict) -> str:
    value = entry.find(path, ns)
    return value.text if value is not None and value.text else ""


def stable_id(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()[:24]


def stable_url(title: str) -> str:
    return f"urn:research-radar:{stable_id(title)}"


def redact_url(url: str) -> str:
    parsed = urllib.parse.urlsplit(url)
    return urllib.parse.urlunsplit((parsed.scheme, parsed.netloc, parsed.path, "", ""))


if __name__ == "__main__":
    raise SystemExit(main())
