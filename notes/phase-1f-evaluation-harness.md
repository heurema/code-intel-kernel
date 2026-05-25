# Phase 1F Evaluation Harness

## Scope

This phase adds a lightweight fixture-based evaluation harness for existing RepoGraph `inspect` and `impact` behavior.

It does not add SymbolGraph, Tree-sitter, LSP, SQLite, MCP, embeddings, workspace split, or xtask.

## What Changed

- Added an in-process evaluator with `eval_contract_version = "0.1"`.
- Added `code-intel eval-fixtures --json`.
- Added JSON eval cases under `tests/eval/cases/`.
- Added semantic evaluator tests for loading cases, report JSON, deliberate false narrow, deliberate false broad, evidence coverage, and deterministic output.

## Eval Case Format

Each case defines:

- fixture path;
- kind: `inspect` or `impact`;
- changed files for impact cases;
- expected components, commands, tests, warnings, status, confidence, and scope;
- forbidden facts or maximum impacted component count where useful.

The harness avoids full JSON snapshots. It checks semantic expectations.

## Metrics

- evidence coverage pass rate;
- expected fact recall;
- unexpected warning count;
- missing expected warning count;
- false broad count;
- false narrow count;
- deterministic output pass rate.

## Current Limitations

- Evaluation is fixture-sized and hand-authored.
- No external tooling is executed.
- No recommended command execution is measured.
- No source-level localization is evaluated.
- No performance or trend storage is included.

## Phase 1G Recommendation

Use Phase 1G as a decision gate:

- If eval failures show RepoGraph extraction or impact gaps, fix RepoGraph first.
- If eval remains stable, open Phase 2A SymbolGraph-lite.

SymbolGraph readiness checklist:

- inspect eval cases pass;
- impact eval cases pass;
- false narrow count is zero on core fixtures;
- false broad cases are documented and acceptable;
- evidence coverage is 100%;
- warnings are structured and expected;
- `where-to-edit` still refuses to guess.
