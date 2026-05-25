# Research Radar

Research Radar is a documentation/config-first intake loop for external code-intelligence research.

It is not a crawler, scheduler, scraper, implementation bot, or feature backlog. Its job is to produce a small daily digest that a human can review before any experiment proposal is written.

## Flow

```text
core runtime paused
  -> research-radar/
  -> daily digest
  -> human approval
  -> experiment proposal
  -> only then Codex prototype
```

## Current Scope

Research Radar v0.1 tracks public sources that may affect `code-intel-kernel`:

- structural retrieval and repo intelligence;
- LSP diagnostics, references, and disambiguation;
- Tree-sitter and parser infrastructure;
- code intelligence benchmarks;
- Codebase-Memory, RIG/SPADE, SWE-bench, and adjacent systems.

The v0.1 scaffold is config and docs only. R2-A adds a bounded collector for reports/state only; it still does not modify runtime code or implement ideas.

## Manual Daily Run

1. Read `research-radar/sources.yaml`.
2. Read `research-radar/scoring.md`.
3. Read `research-radar/guardrails.md`.
4. Use `research-radar/codex-daily-research-prompt.md`.
5. Write:
   - `research-radar/reports/YYYY-MM-DD.md`
   - `research-radar/reports/YYYY-MM-DD.json`
6. Do not modify source code.
7. Do not propose an implementation unless the item scores at least 85 and has an available artifact.

For local manual runs, use dry-run first:

```bash
python3 research-radar/bin/run_daily.py --dry-run
python3 research-radar/bin/run_daily.py --write
python3 research-radar/bin/validate_reports.py
```

## Codex App Automation

The bounded weekday automation is configured in Codex App, not as a repository workflow. Details are documented in `research-radar/automation.md`.

It may write only:

- `research-radar/reports/**`
- `research-radar/state/**`

It must never modify runtime code, import external code, create prototypes, commit automatically, or turn report items into implementation tasks.

## Output Rule

Daily output is candidate evidence. It cannot trigger code changes automatically.
