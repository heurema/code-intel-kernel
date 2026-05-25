# Research Radar Automation

Research Radar automation is a bounded collector. It may collect, score, report, and persist Research Radar state. It must not implement ideas.

## Schedule

Scheduling is managed outside the repository by Codex App Automation.

- Weekday target: about 08:17 Europe/Moscow.
- Automation type: Codex App scheduled workspace run.
- Workspace: this repository checkout.

The repository contains the deterministic collector and validator only. It does not contain a GitHub Actions workflow, cron script, or platform scheduler.

## Local Run

```bash
python3 research-radar/bin/run_daily.py --dry-run
python3 research-radar/bin/run_daily.py --write
python3 research-radar/bin/run_daily.py --write --date YYYY-MM-DD
python3 research-radar/bin/validate_reports.py
```

## Sources

Automation reads:

- `research-radar/sources.automation.json`
- `research-radar/state/seen.jsonl`

Supported v0.1 source types:

- `github_repo`
- `github_search`
- `arxiv_query`

Unsupported sources must be added explicitly. There is no recursive crawling, browser automation, or login-gated scraping.

## Token Handling

`GITHUB_TOKEN` is optional.

- If present, it is used only for GitHub REST API requests.
- If GitHub REST returns an auth/rate-limit error and `gh` is available locally, the collector may fall back to `gh api` for public GitHub metadata.
- It is never printed.
- It is never written to reports or state files.
- Reports store normalized summaries only, not raw API payloads.

## Files Written

Codex App Automation may write only:

- `research-radar/reports/YYYY-MM-DD.md`
- `research-radar/reports/YYYY-MM-DD.json`
- `research-radar/state/seen.jsonl`
- `research-radar/state/source_health.json`
- `research-radar/state/last_run.json`

The automation prompt must validate changed paths before finishing.

Allowed changed paths:

- `research-radar/reports/**`
- `research-radar/state/**`

The scheduled job must not commit by default. A human can review and commit generated reports later.

## Failure Behavior

- Source errors are recorded in `source_health`.
- A source failure does not fail the whole run unless report validation fails.
- Validation fails on invalid JSON, invalid JSONL, secret-like strings, oversized raw payloads, or changed paths outside the allowlist.
- If validation finds runtime/config changes, the automation should report failure and leave changes uncommitted.

## Guardrails

- No runtime code changes.
- No external code import.
- No prototype generation.
- No patch generation.
- No `where-to-edit` integration.
- No MCP/SQLite/LSP feature changes.
- No implementation from external repositories.
- Any experiment candidate is a report item only and requires human approval.
