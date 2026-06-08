# Research Radar

Research Radar is a documentation/config-first intake loop for external code-intelligence research.

It is not a crawler, scheduler, scraper, implementation bot, or feature backlog. Its job is to produce a small daily digest that a human can review before any experiment proposal is written.

## Flow

```text
core runtime paused
  -> research-radar/
  -> daily digest
  -> human approval
  -> experiment proposal with Agent Bench Lab evaluation handoff
  -> only then Codex prototype
  -> Agent Bench Lab run/compare when the benchmark layer is ready
```

## Current Scope

Research Radar v0.1 tracks public sources that may affect `code-intel-kernel`:

- structural retrieval and repo intelligence;
- LSP diagnostics, references, and disambiguation;
- Tree-sitter and parser infrastructure;
- code intelligence benchmarks and Agent Bench Lab evaluation handoff;
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
8. If an experiment candidate is proposed, state whether Agent Bench Lab can evaluate it, which suite or task family would be needed, and what benchmark-layer blockers remain.

For local manual runs, use dry-run first:

```bash
python3 research-radar/bin/run_daily.py --dry-run
python3 research-radar/bin/run_daily.py --write
python3 research-radar/bin/validate_reports.py
```

## Shared Intake Shadow

`research-radar/bin/run_daily.py` is still the scheduled daily collector. The
shared-intake path is a manual shadow path for checking whether this project can
consume the shared collector/governance repo without changing daily report
output yet.

The shared-intake consumer contract is repo-owned here:

- `research-radar/shared-intake.lock.json` pins the exact
  `heurema/shared-intake-governance` commit this project accepts.
- `research-radar/shared-intake/profile.json` defines this project's intake
  profile.
- `research-radar/shared-intake/sources/*.json` defines the source configs this
  project asks shared-intake to validate or run.

Check the pinned dependency and local configs:

```bash
python3 research-radar/bin/check_shared_intake_dependency.py \
  --shared-repo-root ../shared-intake-governance
```

Run a shadow pass with shared-intake runtime artifacts outside git:

```bash
python3 research-radar/bin/run_shared_shadow.py \
  --shared-repo-root ../shared-intake-governance \
  --runtime-root /tmp/code-intel-shared-intake-shadow
```

To use a new shared-intake version, update the shared checkout, run
`python3 scripts/check_repo.py` in `shared-intake-governance`, replace
`upstream.pinned_commit` in `research-radar/shared-intake.lock.json`, then run
the dependency check and shadow command above. A moving upstream `main` does not
silently change this project while the lock is enforced.

## Codex App Automation

The bounded weekday automation is configured in Codex App, not as a repository workflow. Details are documented in `research-radar/automation.md`.

It may write only:

- `research-radar/reports/**`
- `research-radar/state/**`

It must never modify runtime code, import external code, create prototypes, commit automatically, or turn report items into implementation tasks.

## Output Rule

Daily output is candidate evidence. It cannot trigger code changes automatically.
Experiment candidates may define an Agent Bench Lab evaluation handoff, but they still require human approval before prototype work or benchmark repo changes.
