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
- References.
- Call graph, only after simpler symbol extraction is reliable.
- Symbol-level impact.
- Candidate edit locations.
- Source-level evidence bundles.

SymbolGraph facts must be evidence-backed and deterministic. Parse failures should produce structured warnings, not panics.

Phase 2A implements only SymbolGraph-lite:

- Rust source files;
- top-level Rust declarations;
- parse status;
- declaration ranges;
- source-level evidence.

It still does not own calls, references, imports/exports, LSP diagnostics, or edit localization.

Phase 2B adds evaluation coverage for SymbolGraph-lite and documents SourceEvidenceBundle as a future evidence packet. SourceEvidenceBundle does not move edit localization into SymbolGraph yet.

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

## Interaction Model

RepoGraph should be computed first. SymbolGraph can attach source facts to RepoGraph components when evidence supports it.

`where-to-edit` should remain `insufficient_evidence` until SymbolGraph produces evaluated localization evidence.

Impact should remain explicit about the layer used:

- RepoGraph impact: repository/build/test-level.
- SymbolGraph impact: source/symbol-level.
- LSP impact: diagnostics-level, deferred.
