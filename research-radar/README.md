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

The v0.1 scaffold is config and docs only. It does not fetch sources, schedule runs, run scraping automation, or modify runtime code.

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

For v0.1, run manual dry runs only. Do not add a scheduler or scraper until several dry runs show that source noise and scoring are acceptable.

The first next step is a manual dry run over 3-5 P0 sources.

## Output Rule

Daily output is candidate evidence. It cannot trigger code changes automatically.
