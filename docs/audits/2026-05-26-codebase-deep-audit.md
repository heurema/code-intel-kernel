# 2026-05-26 Codebase Deep Audit

Status: EOD-Audit-0 baseline. Runtime feature development remains paused.

## Executive Summary

The repository is in a clean, pushed state at `d5b5bed Add Research Radar report` on `main`, synced with `origin/main`.

Core checks pass:

- `cargo fmt --check`
- `cargo test` (`82 passed`)
- `cargo clippy -- -D warnings`
- `python3 research-radar/bin/validate_reports.py`
- `git diff --check`

The current implementation is internally consistent with the project boundary: read-only evidence layers exist, while `where-to-edit` still refuses with `insufficient_evidence`.

Recommendation: hygiene first. Do not start Phase 3B-B references/definitions yet. The next useful work is audit cleanup around docs/runtime drift, public metadata, test/module split points, and LSP eval coverage.

## Current Repo State

- Branch: `main`
- HEAD: `d5b5bed64e1e2b2e15a88f1ebe9eb6e1c4c63970`
- HEAD short: `d5b5bed`
- Upstream: `origin/main`
- Working tree: clean before tracked audit artifacts were created.
- Sync state at audit start: `main...origin/main`

### Recent Commits Grouped By Theme

RepoGraph:

- `77c52cb` Initialize Rust-first code intelligence kernel and stabilize inspect contract
- `46c76b7` Add RepoGraph relationships and impact analysis skeleton
- `a82c57d` Improve RepoGraph extractor quality for Cargo and command files
- `72610f1` Improve RepoGraph extraction for Python and Go projects

Impact:

- `46c76b7` Add RepoGraph relationships and impact analysis skeleton
- `fd32184` Strengthen RepoGraph impact traversal and recommendations

Eval:

- `18c6220` Add fixture-based evaluation harness for RepoGraph inspect and impact
- `18c9739` Add SymbolGraph evaluation and source evidence bundle contract
- `3476811` Add adversarial localization readiness gate

SymbolGraph:

- `744450b` Add SymbolGraph readiness gate and boundary documentation
- `e8e6723` Add Rust SymbolGraph-lite for top-level source facts
- `18c9739` Add SymbolGraph evaluation and source evidence bundle contract

SourceEvidence:

- `a605df5` Add source evidence bundle prototype
- `5c35b0a` Harden source evidence linking and refusal behavior
- `e913715` Add source context selector hints to source evidence

SourceContext:

- `016dd54` Add read-only source context slices
- `e913715` Add source context selector hints to source evidence

LSP diagnostics:

- `76d9dc4` Design LSP diagnostics and references bridge
- `a808449` Add Rust LSP diagnostics bridge

Research Radar:

- `c3e1c53` Add Research Radar scaffold
- `f40c755` Add first Research Radar dry run report
- `58397c7` Add Codex App Research Radar automation runner
- `d5b5bed` Add Research Radar report

Context Pack:

- `9306f45` docs(context-pack): add context pack draft

Docs/hygiene:

- `7d20ec9` Simplify README for public project overview
- `9306f45` docs(context-pack): add context pack draft

## Architecture Map

Current runtime modules:

- `src/core/repo_graph.rs`: RepoGraph inspection, command extraction, warning model, and RepoGraph-only impact.
- `src/core/evaluation.rs`: fixture evaluator and metrics.
- `src/core/symbol_graph.rs`: SymbolGraph-lite for Rust top-level source facts.
- `src/core/source_evidence.rs`: query evidence assembly over RepoGraph and SymbolGraph-lite.
- `src/core/source_context.rs`: explicit-selector, read-only source slices.
- `src/core/lsp_bridge.rs`: Rust diagnostics bridge; diagnostics only.
- `src/main.rs`: CLI surface and `where-to-edit` placeholder behavior.
- `research-radar/bin/run_daily.py`: bounded public-signal collector.
- `research-radar/bin/validate_reports.py`: report/state validator.

Placeholder/exported but not mature runtime areas:

- `src/storage/sqlite.rs`: `open_kernel_database` placeholder; no SQLite dependency.
- `src/core/process_reward.rs`: placeholder scoring.
- `src/core/memory.rs`: in-memory event constructor only.
- `src/core/evidence.rs`: placeholder EvidenceBundle used by `where-to-edit` refusal path.

## Contract Versions

- `inspect`: `0.2`
- `impact`: `0.2`
- `eval`: `0.4`
- `symbols`: `0.1`
- `source_evidence`: `0.3`
- `source_context`: `0.1`
- `lsp_diagnostics`: `0.1`
- `Context Pack`: `draft-0.1`, docs-only
- `Research Radar report`: `0.1`
- `where-to-edit`: no explicit contract version; behavior is `insufficient_evidence`

Captured snapshots:

- `target/audit/eod-2026-05-26/inspect.json`
- `target/audit/eod-2026-05-26/impact.json`
- `target/audit/eod-2026-05-26/eval.json`
- `target/audit/eod-2026-05-26/symbols.json`
- `target/audit/eod-2026-05-26/source-evidence.json`
- `target/audit/eod-2026-05-26/source-context.json`
- `target/audit/eod-2026-05-26/lsp-diagnostics.json`
- `target/audit/eod-2026-05-26/where-to-edit.json`

## Layer-By-Layer Review

### RepoGraph

Current repo inspection finds:

- package manager: `cargo`
- components: `rust_bin_target`, `rust_lib_target`, `rust_crate`
- commands: `cargo build`, `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo test`
- warning: ignored `target`

Assessment: strong for local Rust crate facts and command discovery. Main hotspot is size: `src/core/repo_graph.rs` is 2921 LOC and now combines many extraction concerns.

### Impact

`impact src/main.rs Cargo.toml --json` returns:

- `contract_version`: `0.2`
- `status`: `partial`
- `confidence`: `medium`
- impacted components: `3`
- command recommendations: `0`
- warnings: RepoGraph-only impact and no dependency edges.

Assessment: conservative behavior is correct. The absence of command recommendations for the current local query should be reviewed before claiming impact is useful as a verification planner.

### Eval

`eval-fixtures` summary:

- total cases: `35`
- passed: `35`
- failed: `0`
- inspect cases: `11`
- impact cases: `6`
- symbol cases: `3`
- source evidence cases: `9`
- source context cases: `6`
- deterministic output pass rate: `1.0`
- evidence coverage pass rate: `1.0`
- expected fact recall: `1.0`
- false broad count: `0`
- false narrow count: `0`

Assessment: good fixture coverage for current layers. Gap: LSP diagnostics has smoke tests but no eval case count in `eval-fixtures`.

### SymbolGraph-lite

`symbols . --json` returns:

- `contract_version`: `0.1`
- source files: `28`
- symbols: `473`
- warnings: `2`

Assessment: works as Rust top-level symbol evidence. It is not a references/imports/call graph layer and should not be treated as semantic localization.

### SourceEvidence

`source-evidence "parse repo graph" --json` returns:

- `contract_version`: `0.3`
- `status`: `partial`
- `confidence`: `low`
- candidate files: `8`
- candidate symbols: `12`
- selector hints: `12`
- evidence records: `506`
- warnings: `5`
- missing evidence: `8`

Assessment: useful context assembly, but broad query output is truncated and explicitly low-confidence. Missing evidence correctly includes no call graph, no LSP diagnostics, no symbol reference layer, query relevance, and localization not supported.

### SourceContext

`source-context --file src/lib.rs --json` returns:

- `contract_version`: `0.1`
- `status`: `ok`
- selectors: `1`
- slices: `1`
- warning: `source_context_not_localization`
- limitations: explicit selectors only, Rust source only, no references/call graph/LSP/patch planning.

Assessment: boundary is clean. SourceContext is a safe explicit-selector slice layer, not natural-language localization.

### LSP Diagnostics

`lsp-diagnostics --file src/lib.rs --json` returns:

- `contract_version`: `0.1`
- `status`: `unavailable`
- diagnostics: `0`
- warnings: `lsp_not_localization`, `lsp_diagnostics_unavailable`, `rust_analyzer_unavailable`
- missing evidence: `lsp_diagnostics_unavailable`, `no_lsp_diagnostics`, `rust_analyzer_unavailable`

Assessment: unavailable path is structured and expected in this environment. Before Phase 3B-B, add eval coverage for LSP diagnostics behavior and decide whether local rust-analyzer availability should be part of a manual smoke profile.

### Context Pack Draft

Context Pack is documented only:

- `docs/context-pack-design.md`
- `docs/context-pack-json-contract-draft.md`
- `notes/context-pack-idea.md`

Assessment: boundary is currently correct. It is a future read-only context assembly layer with `decision_semantics: "not_supported"`, not localization or planning.

### Research Radar

Checks:

- `python3 research-radar/bin/validate_reports.py` passed.
- `python3 research-radar/bin/run_daily.py --dry-run` exited successfully.
- `python3 -m py_compile research-radar/bin/run_daily.py research-radar/bin/validate_reports.py` passed.
- dry-run left no git changes.
- secret search found no token-like strings.
- raw payload search found only guardrail text stating reports are normalized summaries.
- forbidden implementation language search found only guardrail text: `Do not import external code.`

Dry-run source status:

- GitHub sources succeeded.
- arXiv code agents timed out.
- arXiv code graphs returned HTTP 429.

Assessment: automation remains report/state-only and external scheduling is documented as Codex App Automation, not GitHub Actions. Source reliability needs monitoring before source expansion.

### where-to-edit Refusal

`where-to-edit "change login validation copy" --profile=strict --json` returns:

- `ok`: `false`
- `status`: `insufficient_evidence`
- `confidence`: `0`
- files: `[]`
- symbols: `[]`
- missing evidence:
  - `SymbolGraph-lite is not evaluated for edit localization`
  - `No file/symbol relevance model yet`
- warning: placeholder until evaluated localization evidence exists.

Assessment: correct. This must remain unchanged until a dedicated localization gate exists.

## Strengths

- Strong refusal posture: missing evidence is surfaced instead of converted into guesses.
- CLI outputs are structured and deterministic for current eval/symbols probes.
- Current contracts are small and explicit.
- Research Radar has a clear report/state-only boundary and validator.
- Core dependency footprint is small: `serde`, `serde_json`, `toml`, `tree-sitter`, `tree-sitter-rust`.
- No runtime mutation features are exposed.

## Weaknesses

- `src/core/repo_graph.rs` is too large and mixes many extractor concerns.
- `tests/smoke.rs` is too large and mixes CLI, eval, source, LSP, and safety assertions.
- LSP diagnostics has runtime smoke coverage but no fixture eval count.
- Public exports include placeholder areas (`sqlite`, `process_reward`, `memory`, `evidence`) that look more mature than they are.
- `docs/05-agent-tools.md` still describes future tools as if `where_to_edit` can return ranked files/symbols and `symbol_context` can return definitions/references/imports/exports.
- No `LICENSE`, `SECURITY.md`, CI workflow, or Cargo.toml license metadata.

## Hidden Coupling Risks

- SourceEvidence depends on RepoGraph and SymbolGraph-lite; broad query truncation can make downstream consumers overinterpret capped output.
- `src/main.rs` owns CLI parsing and user-visible placeholder language; CLI growth will increase coupling unless split.
- LSP diagnostics process handling is isolated in `lsp_bridge.rs`, but Phase 3B-B could accidentally turn locations into edit targets.
- Placeholder public exports make it easy for consumers to rely on unstable APIs.
- Research Radar reports use `suggested_action` labels; guardrails currently prevent implementation, but future agents could overread them without the report-state boundary.

## Docs/Runtime Drift

Confirmed drift:

- `docs/05-agent-tools.md` says `where_to_edit(task)` returns ranked files and symbols; runtime refuses.
- `docs/05-agent-tools.md` says `symbol_context` returns definitions, references, imports, exports, tests; runtime has no reference/import/export semantic layer.
- `docs/04-data-model.md` describes imports, exports, calls, references, events, SQLite-style memory concepts, EvidenceBundle, and ProcessReward as core model ideas; runtime is still mostly placeholders for these.
- `src/lib.rs` exports `open_kernel_database`, `ProcessReward`, and memory/evidence placeholders. That is acceptable for a private prototype but public API maturity is overstated.

No problematic drift:

- README now says no IDE/agent/MCP/vector database/edit planner.
- Context Pack docs are explicitly draft-only.
- Research Radar automation docs correctly state Codex App external scheduling and report/state-only writes.

## Test/Eval Robustness

Current robustness is good for:

- evidence ID coverage;
- deterministic output;
- malformed manifest warnings;
- ignored path behavior;
- source context path safety;
- no edit-target-language checks for SourceEvidence, SourceContext, LSP diagnostics, and `where-to-edit`.

Gaps:

- LSP diagnostics is not represented as an eval-fixture count.
- `tests/smoke.rs` is large enough that failures may be hard to localize.
- Eval does not yet include performance thresholds or report-size/token-budget checks.
- Context Pack draft has no tests yet, which is correct because it has no runtime.

## Security/Dependency/License Posture

Security scan:

- `cargo audit --json`: `0` vulnerabilities.
- `cargo deny`: unavailable.
- `cargo geiger`: unavailable.
- `cargo outdated`: unavailable.

Metadata:

- `LICENSE`: missing.
- `SECURITY.md`: missing.
- Cargo.toml `license`: missing.
- CI workflow: none found under `.github`.

Dependency posture:

- Small Rust dependency set.
- `cargo tree --duplicates` output is limited to repeated serde/serde_json paths, not a large version-skew problem.
- Research Radar uses network access in local Python scripts, but reports normalized summaries and validator checks token/raw-payload guardrails.

Public posture is acceptable for private/local development, but incomplete for a public project.

## Performance/Determinism Baseline

Determinism:

- `eval-fixtures` run 1 and run 5: no diff.
- `symbols .` run 1 and run 5: no diff.

Timing baseline on macOS:

| Command | Real | User | Sys | Max RSS |
| --- | ---: | ---: | ---: | ---: |
| `cargo test --test smoke -- --test-threads=1` | 2.50s | 1.86s | 0.18s | 31,326,208 bytes |
| `cargo run --quiet -- eval-fixtures --json >/dev/null` | 0.71s | 0.26s | 0.04s | 12,173,312 bytes |
| `cargo run --quiet -- symbols . --json >/dev/null` | 0.59s | 0.14s | 0.03s | 10,092,544 bytes |
| `cargo run --quiet -- source-evidence "parse repo graph" --json >/dev/null` | 0.57s | 0.15s | 0.04s | 11,485,184 bytes |

## What Not To Do Next

- Do not start Phase 3B-B references/definitions immediately.
- Do not wire LSP facts into `where-to-edit`.
- Do not add MCP, SQLite, embeddings, call graph, or localization.
- Do not expand Research Radar schedule or sources before reviewing arXiv reliability.
- Do not treat Context Pack as implemented.
- Do not add runtime features to solve docs drift.

## Prioritized Next Actions

1. Hygiene pass: label or rewrite stale bootstrap docs (`docs/05-agent-tools.md`, `docs/04-data-model.md`) so future ideas do not read as implemented behavior.
2. Public metadata pass: decide whether to add `LICENSE`, `SECURITY.md`, Cargo.toml license metadata, and minimal CI.
3. Test split pass: split `tests/smoke.rs` by layer or scenario without changing behavior.
4. Module split pass: split `repo_graph.rs` into extractor-focused modules after tests are stable.
5. LSP eval pass: add fixture or mocked-response eval cases for LSP diagnostics before considering references/definitions.
6. Research Radar reliability pass: record arXiv timeout/429 behavior and decide whether to back off, cache, or keep as best-effort.

## Decision Recommendation

Recommended next phase: hygiene first.

Phase 3B-B should remain deferred until:

- docs/runtime drift is reduced;
- LSP diagnostics has eval/adversarial coverage;
- placeholder public exports are either documented as placeholders or hidden from public API expectations;
- license/security/CI posture is intentionally accepted or fixed.

