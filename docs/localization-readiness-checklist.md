# Localization Readiness Checklist

Status: Phase 2G adversarial gate. Current conclusion: `not_ready_for_confident_localization`.

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

- [x] Duplicate source symbol names are tested.
  Evidence: eval case `adversarial_duplicate_symbol_source_evidence`.

- [x] Reference/call-graph-style queries refuse missing evidence.
  Evidence: eval case `adversarial_reference_query_source_evidence`.

- [x] Unsupported-language source does not become SymbolGraph evidence.
  Evidence: eval case `adversarial_unsupported_language_source_evidence`.

- [x] RepoGraph/component text without source-symbol evidence does not create source candidates.
  Evidence: eval case `adversarial_component_without_symbol_source_evidence`.

- [x] SourceContext path traversal refusal is tested.
  Evidence: eval case `adversarial_source_context_path_outside`.

- [x] Malformed source can be sliced only through explicit safe selectors.
  Evidence: eval case `adversarial_source_context_malformed_slice`.

- [x] Runtime outputs avoid edit-target language.
  Evidence: source-evidence/source-context eval `output_not_contains`; smoke tests `source_evidence_output_has_no_edit_target_language`, `source_context_output_is_deterministic_and_has_no_edit_target_language`, and `where_to_edit_still_refuses_after_selector_hints_and_source_context`.

- [ ] Candidate files and symbols are tied to queries by semantic/evaluated relevance evidence, not only string matching.
- [ ] SymbolGraph-to-RepoGraph linking is defined and tested.
- [ ] Reference/import/call graph decision is made.
- [ ] LSP diagnostics bridge decision is made.
- [x] Negative localization fixtures prove no false confident candidate behavior.
  Evidence: Phase 2G adversarial eval cases plus `where-to-edit` refusal tests.
- [x] Source context/snippet policy is defined for explicit selectors.
  Evidence: `docs/source-context-json-contract.md`.
- [ ] False broad versus false narrow localization metrics exist.

## Conclusion

`not_ready_for_confident_localization`

Phase 2G verifies top-level Rust source facts, evidence assembly, source-to-repo context roles, limits, refusal behavior, bounded read-only source slices, explicit selector hints, and adversarial refusal cases. It still uses deterministic string/token matching and explicit selectors. That is not enough to decide edit locations. `where-to-edit` must remain `insufficient_evidence`.
