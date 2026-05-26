# Localization Readiness Checklist

Status: Phase 3B-A LSP diagnostics gate. Current conclusion: `not_ready_for_confident_localization`.

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
- [x] Reference/import/call graph decision is made for the next step.
  Evidence: `docs/lsp-capability-matrix.md`; references are Phase 3B candidate, call hierarchy and import/export semantics remain deferred.
- [x] LSP diagnostics bridge decision is made.
  Evidence: `docs/lsp-bridge-design.md`; Phase 3A is design-only and Phase 3B is a narrow read-only Rust candidate.

- [x] Rust LSP diagnostics unavailable path is implemented and tested.
  Evidence: `docs/lsp-diagnostics-json-contract.md`; smoke test `lsp_diagnostics_unavailable_when_rust_analyzer_is_missing`.
- [x] Negative localization fixtures prove no false confident candidate behavior.
  Evidence: Phase 2G adversarial eval cases plus `where-to-edit` refusal tests.
- [x] Source context/snippet policy is defined for explicit selectors.
  Evidence: `docs/source-context-json-contract.md`.
- [ ] False broad versus false narrow localization metrics exist.

## LSP Blockers Before Localization

- [x] LSP diagnostics layer has a first read-only runtime slice.
  Evidence: `lsp-diagnostics` CLI and `docs/lsp-diagnostics-json-contract.md`.
- [ ] LSP diagnostics layer has fixture/adversarial eval coverage.
- [ ] LSP reference layer is implemented and evaluated.
- [ ] Definition disambiguation is implemented and evaluated.
- [ ] Semantic type/hover evidence is evaluated or explicitly deferred.
- [ ] Call hierarchy is evaluated or explicitly deferred.
- [ ] Symbol-to-reference resolution is verified against adversarial fixtures.
- [ ] LSP facts are integrated into SourceEvidence as evidence, not edit targets.
- [ ] A localization-specific adversarial gate passes after LSP evidence exists.

## Context Pack Boundary

Context Pack is a future context assembly idea, not a localization milestone.

Before any Context Pack implementation exists, it must have eval coverage proving that output remains contextual:

- [ ] Token budget measurement exists.
- [ ] Context Pack compactness is measured.
- [ ] Evidence coverage is measured by layer.
- [ ] No edit-target-language tests cover Context Pack output.
- [ ] Ambiguity and refusal tests cover Context Pack output.
- [ ] Convergence and disagreement reporting is tested.
- [ ] Comparison against raw file exploration exists.

Even if Context Pack output is available, it must not satisfy localization readiness by itself. `where-to-edit` remains `insufficient_evidence` until the dedicated localization gate passes.

## Conclusion

`not_ready_for_confident_localization`

Phase 3B-A adds read-only Rust diagnostics, but diagnostics alone are not enough to decide edit locations. The kernel still lacks references, definitions, semantic disambiguation, and localization-specific eval. `where-to-edit` must remain `insufficient_evidence`.
