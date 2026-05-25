# Localization Adversarial Readiness

Status: Phase 2G adversarial gate.

Current conclusion: `not_ready_for_confident_localization`.

Phase 2G stress-tests the evidence stack against ambiguity and refusal cases. It does not add localization, reference resolution, call graph, LSP, SQLite, MCP, embeddings, or patch planning.

## Current Stack

- RepoGraph answers repository/build/test questions.
- SymbolGraph-lite answers Rust top-level source fact questions.
- SourceEvidence assembles query-linked evidence candidates and explicit SourceContext selector hints.
- SourceContext returns bounded read-only source slices for explicit selectors.
- `where-to-edit` remains an `insufficient_evidence` placeholder.

## Safe Answers

The current stack can safely say:

- a query matched one or more evidence-backed source files or top-level symbols;
- a candidate symbol belongs to a source file;
- a source file may belong to a RepoGraph component when path evidence supports it;
- a SourceContext selector can be used to retrieve a bounded read-only snippet;
- evidence is ambiguous, partial, missing, or unsupported.

It must not say:

- edit this file;
- edit here;
- this is the edit location;
- apply this patch;
- this is the root cause;
- this function is referenced by that caller;
- this source slice proves the correct change.

## Adversarial Cases

Phase 2G adds cases for:

- duplicate top-level Rust symbol names in different files;
- broad queries that match multiple source facts;
- ignored/generated paths;
- malformed Rust source;
- unsupported non-Rust source;
- reference/call-graph queries such as "who calls X";
- RepoGraph/component matches without source-symbol matches;
- SourceContext path safety failures.

These cases verify refusal behavior and runtime output wording. Selector hints remain context handles, not edit locations.

## Readiness Decision

Decision: `not_ready_for_confident_localization`.

Reason:

- top-level symbols are not references;
- selector hints are not edit targets;
- source snippets are not proof of root cause;
- duplicate symbol names require disambiguation evidence;
- reference and call-graph queries still require a reference layer or LSP bridge;
- broad queries still require human or higher-layer disambiguation;
- unsupported-language source facts are intentionally absent.

## Recommended Next Phase

Recommended next phase: Phase 3A LSP diagnostics/reference bridge design.

Reason: Phase 2G's main blocker is not another context packaging layer. The highest-risk gaps are references, duplicate-symbol disambiguation, diagnostics, and semantic navigation. Those should be designed before implementation.

Do not start LSP implementation until the design preserves the current contracts:

- RepoGraph remains build/test/repo-level.
- SymbolGraph-lite remains Rust top-level source facts.
- SourceEvidence remains evidence assembly.
- SourceContext remains explicit-selector source slicing.
- `where-to-edit` remains refusal-only until localization evidence is evaluated.
