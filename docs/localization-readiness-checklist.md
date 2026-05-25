# Localization Readiness Checklist

Status: Phase 2C gate. Current conclusion: `not_ready_for_confident_localization`.

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

- [ ] Candidate files and symbols are tied to queries by evaluated evidence, not string guessing.
- [ ] SymbolGraph-to-RepoGraph linking is defined and tested.
- [ ] Reference/import/call graph decision is made.
- [ ] LSP diagnostics bridge decision is made.
- [ ] Negative localization fixtures prove no false confident candidate behavior.
- [ ] Source context/snippet policy is defined.
- [ ] False broad versus false narrow localization metrics exist.

## Conclusion

`not_ready_for_confident_localization`

Phase 2C verifies top-level Rust source facts and adds evidence assembly. It still uses only deterministic string/token matching. That is not enough to decide edit locations. `where-to-edit` must remain `insufficient_evidence`.
