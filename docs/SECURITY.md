# Code Intelligence Kernel Security

## Trust Boundaries

- **Local deterministic boundary**: Rust CLI/library code under `src/`, fixture evaluation under `tests/`, and JSON contracts under `docs/`.
- **Repository input boundary**: `inspect`, `impact`, `symbols`, `source-evidence`, and `source-context` read local repository files and must treat malformed or unsupported input as structured warnings.
- **LSP process boundary**: `lsp-diagnostics` may start `rust-analyzer` and must keep requests read-only, bounded, path-contained, and unavailable-safe.
- **Research Radar boundary**: `research-radar/bin/` may collect public-source metadata and must write only normalized reports/state, not raw payload dumps or runtime code.

## Sensitive Surfaces

| Surface | Why sensitive |
| --- | --- |
| `src/core/source_context.rs` | Returns source slices and enforces path containment, ignored paths, symlink, UTF-8, and size handling. |
| `src/core/lsp_bridge.rs` | Starts and communicates with an external language-server process. |
| `src/core/repo_graph.rs` | Reads manifests and workflow files from arbitrary repositories. |
| `src/core/source_evidence.rs` | Assembles evidence candidates that downstream users could overinterpret as localization. |
| `research-radar/bin/run_daily.py` | Performs public-source collection and records license/terms notes. |
| `research-radar/bin/validate_reports.py` | Guards generated reports/state against raw payloads, oversized files, and obvious secret patterns. |
| `.github/workflows/` | Controls automated validation on repository changes. |

## Existing Controls

- Runtime commands are read-only and must not mutate inspected repositories.
- Missing evidence is represented explicitly instead of guessed.
- `where-to-edit` remains `insufficient_evidence` until a dedicated localization gate passes.
- SourceContext refuses path traversal, ignored/generated paths, symlinks, missing files, non-UTF8 files, and oversized slices with structured warnings.
- LSP diagnostics has deterministic unavailable/path-safety eval cases and does not expose mutation-capable LSP methods.
- Research Radar validation checks changed-path allowlists, JSON/JSONL validity, file size, and obvious secret patterns.

## Secrets and Data Handling

- Do not add real credentials, tokens, cookies, private keys, or sensitive PII to fixtures, reports, docs, prompts, or examples.
- Do not commit raw external API payloads from Research Radar runs.
- Do not widen external-service or provider data exposure without updating this document and the relevant contract docs.
- Keep reports and diagnostics as evidence, not implementation triggers or edit instructions.

## Security Review Triggers

- Any change to path handling, ignored-path logic, symlink handling, or source slicing.
- Any change to `src/core/lsp_bridge.rs` process lifecycle, timeout, request, or parsing behavior.
- Any change that makes `where-to-edit`, SourceEvidence, SourceContext, or LSP output more localization-like.
- Any change to Research Radar collection, validation, source config, or generated report/state boundaries.
- Any CI workflow, dependency, install script, or public API change.

## Minimum Security Evidence for Sensitive Changes

- Add or update deterministic tests for security-relevant behavior changes.
- Run the full deterministic test script before merge.
- Update public contract docs when externally visible behavior changes.
- Keep LSP, SourceContext, SourceEvidence, and Research Radar outputs explicit about limitations and missing evidence.
