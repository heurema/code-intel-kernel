# Localization Readiness Checklist

Status: Phase 2F gate. Current conclusion: `not_ready_for_confident_localization`.

This checklist defines what must be true before `where-to-edit` can return confident file or symbol candidates.

## Current Checks

- [x] RepoGraph inspect eval cases pass.
  Evidence: `cargo run --quiet -- eval-fixtures --json`, `inspect_cases > 0`, `failed_cases = 0`.

- [x] RepoGraph impact eval cases pass.
  Evidence: `cargo run --quiet -- eval-fixtures --json`, `impact_cases > 0`, `failed_cases = 0`.

- [x] SymbolGraph-lite eval cases pass.
  Evidence: `cargo run --quiet -- eval-fixtures --json`, `symbol_cases > 0`, `failed_cases = 0`.

- [x] Source and symbol facts are evidence-backed.
  Evidence: eval metric `evidence_coverage_pass_rate = 1.0`; smoke test `every_symbol_graph_source_file_and_symbol_has_valid_evidence`.

- [x] SymbolGraph-lite output is deterministic on fixtures.
  Evidence: eval metric `deterministic_output_pass_rate = 1.0`; smoke test `symbol_graph_ids_and_order_are_deterministic`.

- [x] Parse errors produce structured warnings, not panics.
  Evidence: eval case `rust_symbols_malformed_warning`; smoke test `malformed_rust_source_produces_symbol_warning_without_panic`.

- [x] Ignored paths do not produce source symbols.
  Evidence: eval case `rust_symbols_ignored_paths`; smoke test `symbol_graph_ignores_generated_and_dependency_directories`.

- [x] `where-to-edit` still refuses to guess.
  Evidence: smoke test `where_to_edit_remains_insufficient_evidence_placeholder`.

## Missing Before Confident Localization

- [x] SourceEvidenceBundle runtime prototype exists.
  Evidence: `cargo run --quiet -- source-evidence "parse repo graph" --json`; smoke test `source_evidence_cli_output_is_valid_json`.

- [x] SourceEvidenceBundle eval cases pass.
  Evidence: `cargo run --quiet -- eval-fixtures --json`, `source_evidence_cases > 0`, `failed_cases = 0`.

- [x] Candidate files and symbols include source-to-repo context when path evidence supports it.
  Evidence: source-evidence eval cases for component/command/test context roles.

- [x] Candidate limits prevent unbounded evidence bundle output.
  Evidence: eval case `source_evidence_broad_query_limit`; smoke test `source_evidence_broad_query_truncates_candidates`.

- [x] SourceContext returns bounded read-only slices for explicit selectors.
  Evidence: eval cases `source_context_file_slice` and `source_context_symbol_slice`; smoke tests `source_context_file_selector_returns_bounded_slice_with_evidence` and `source_context_symbol_id_selector_returns_symbol_slice`.

- [x] SourceContext path safety is checked.
  Evidence: eval cases `source_context_missing_file` and `source_context_ignored_path`; smoke tests for missing, ignored, outside, symlink, and non-UTF8 paths.

- [x] SourceEvidence emits explicit SourceContext selector hints.
  Evidence: source-evidence eval cases for selector hints; smoke test `source_evidence_selector_hint_can_feed_source_context_manually`.

- [ ] Candidate files and symbols are tied to queries by semantic/evaluated relevance evidence, not only string matching.
- [ ] SymbolGraph-to-RepoGraph linking is defined and tested.
- [ ] Reference/import/call graph decision is made.
- [ ] LSP diagnostics bridge decision is made.
- [ ] Negative localization fixtures prove no false confident candidate behavior.
- [x] Source context/snippet policy is defined for explicit selectors.
  Evidence: `docs/source-context-json-contract.md`.
- [ ] False broad versus false narrow localization metrics exist.

## Conclusion

`not_ready_for_confident_localization`

Phase 2F verifies top-level Rust source facts, evidence assembly, source-to-repo context roles, limits, refusal behavior, bounded read-only source slices, and explicit selector hints. It still uses deterministic string/token matching and explicit selectors. That is not enough to decide edit locations. `where-to-edit` must remain `insufficient_evidence`.
