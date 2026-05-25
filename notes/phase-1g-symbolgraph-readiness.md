# Phase 1G SymbolGraph Readiness

## Decision

`ready_with_constraints`

RepoGraph is stable enough to plan Phase 2A SymbolGraph-lite, but not enough to claim broad source-level readiness. The current eval set is fixture-based and limited. Phase 2A must be narrow and must not make `where-to-edit` appear confident before symbol evidence is validated.

## Current Contract Versions

- `inspect`: `0.2`
- `impact`: `0.2`
- `eval`: `0.1`

## Current Eval Summary

Latest fixture evaluation:

- total cases: 17
- passed cases: 17
- failed cases: 0
- evidence coverage pass rate: 1.0
- expected fact recall: 1.0
- false broad count: 0
- false narrow count: 0
- deterministic output pass rate: 1.0

The eval set now includes positive fixtures and refusal-oriented negative cases. This is useful as a gate, but it is still fixture-sized and semantic, not a broad benchmark or large-repository validation pass.

## Current RepoGraph Capabilities

RepoGraph can answer:

- which supported manifests and build files are present;
- which package managers are evidenced;
- which workspaces and components are evidenced;
- which build, lint, format, and test commands are safe to recommend;
- which commands/tests are scoped to repo or component-level facts;
- which explicit manifest dependency edges exist where supported;
- which changed files have conservative build/test-level impact;
- when impact is broad, targeted, mixed, unknown, or insufficient;
- which facts and recommendations are backed by evidence.

## Current RepoGraph Refusals

RepoGraph must refuse to answer:

- which source symbol should be edited;
- whether a specific function/class/type is impacted;
- import/reference/call graph questions;
- exact edit locations;
- semantic correctness of code changes;
- LSP diagnostic state;
- source-level equivalence or dead-code claims.

`where-to-edit` remains `insufficient_evidence`.

## Remaining RepoGraph Risks

- Fixture coverage is still small.
- Manifest extraction is conservative and incomplete.
- Dependency edge coverage exists only for a few manifest patterns.
- Impact can still be broader than ideal in real repositories.
- Missing source-level context means recommended tests are build/test-level only.
- Eval does not execute recommended commands.
- Cyclic dependency edges are not explicitly covered by eval fixtures.
- Symlink escapes and path containment edge cases are not explicitly covered.
- Unsupported-language repositories need more refusal fixtures.
- Mixed root ecosystems can still expose scope-boundary bugs.
- Generated code and build artifacts may need more ignore-policy tests.
- Dynamic or custom build scripts are only weakly represented.
- Source files with parse errors are deferred to SymbolGraph eval.
- Large repositories may not behave like fixture-sized repositories.

## What SymbolGraph Should Add

SymbolGraph should add evidence-backed source-level facts:

- source files as code units;
- top-level symbols;
- modules;
- imports/exports when supported;
- references only after extraction is reliable;
- source-level warnings for parse failures;
- candidate symbol context for future localization.

## What SymbolGraph Must Not Own

SymbolGraph must not own:

- package manager detection;
- build/test command inference;
- workspace/component detection;
- manifest dependency extraction;
- RepoGraph impact and command ranking;
- consumer-specific policy such as Goalrail or Punk.

RepoGraph remains the build/test control plane. SymbolGraph should consume RepoGraph context, not replace it.

## Phase 2A Constraint

Phase 2A should be SymbolGraph-lite:

- narrow source discovery and top-level symbol extraction;
- deterministic IDs;
- evidence-backed facts;
- parse failures as structured warnings;
- internal API first;
- no public SymbolGraph placeholder API; the old stub was removed from library exports;
- no call graph;
- no LSP;
- no SQLite;
- no MCP;
- no embeddings;
- no confident `where-to-edit` localization yet.
