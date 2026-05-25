# RepoGraph vs SymbolGraph Boundary

Status: Phase 1G boundary document.

RepoGraph and SymbolGraph are complementary layers. RepoGraph describes how a repository is organized, built, and tested. SymbolGraph should describe source-code structure inside those components.

## RepoGraph Owns

- Manifests and lockfiles.
- Package manager detection.
- Workspaces.
- Components.
- Build, check, lint, format, and test commands.
- Command and test scopes.
- Dependency edges extracted from manifests.
- Build/test-level impact analysis.
- Structured warnings for unsupported, malformed, ambiguous, or missing build/test facts.

RepoGraph output must remain evidence-backed and deterministic. It should prefer warnings and `insufficient_evidence` over guesses.

## SymbolGraph Should Own

- Source files as code units.
- Top-level symbols.
- Functions, classes, types, modules, and methods when supported.
- Imports and exports.
- References only after an evaluated reference layer exists.
- Call graph only after simpler symbol/reference extraction is reliable.
- Symbol-level impact only after source-level evidence is evaluated.
- Source-level evidence that can feed a future localization layer.

SymbolGraph facts must be evidence-backed and deterministic. Parse failures should produce structured warnings, not panics.

Phase 2A implements only SymbolGraph-lite:

- Rust source files;
- top-level Rust declarations;
- parse status;
- declaration ranges;
- source-level evidence.

It still does not own calls, references, imports/exports, LSP diagnostics, or edit localization.

Phase 2B adds evaluation coverage for SymbolGraph-lite. Phase 2C adds SourceEvidenceBundle as read-only evidence assembly. Phase 2D hardens source-to-repo context roles and refusal behavior. Phase 2E adds SourceContext as explicit-selector, read-only source slices. Phase 2F adds selector hints from SourceEvidence to SourceContext. Phase 2G adds adversarial refusal cases for duplicate symbols, broad/reference-style queries, unsupported language input, and path safety. None of these move edit localization into SymbolGraph.

## Explicit Non-Goals

RepoGraph must not:

- infer source symbols;
- choose edit locations;
- claim call/reference impact;
- localize `where-to-edit`.

SymbolGraph must not:

- infer package managers;
- replace build/test command extraction;
- own workspace/component discovery;
- own manifest dependency extraction;
- encode Goalrail, Punk, or other consumer-specific behavior.

SourceContext owns bounded source snippets for explicit file or symbol selectors. SourceEvidence may suggest selector hints for manual SourceContext retrieval. Neither layer may accept natural-language localization queries or turn evidence candidates into edit targets.

LSP should own diagnostics, definitions, references, and document-symbol facts only after Phase 3B implements a read-only bridge. LSP locations are evidence, not edit targets.

## Interaction Model

RepoGraph should be computed first. SymbolGraph can attach source facts to RepoGraph components when evidence supports it.

`where-to-edit` should remain `insufficient_evidence` until a separate localization layer has evaluated reference, call graph, diagnostics, ambiguity, and negative-case evidence.

Impact should remain explicit about the layer used:

- RepoGraph impact: repository/build/test-level.
- SymbolGraph impact: source/symbol-level.
- LSP impact: diagnostics-level, deferred.
