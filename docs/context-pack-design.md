# Context Pack Design

Status: future phase design draft. No runtime implementation, CLI command, or contract change exists yet.

Contract identity: `context_pack = "draft-0.1"`.

## Verdict

DEFER implementation. Document the concept now so future work can evaluate it without turning `code-intel-kernel` into an agent, IDE, planner, PR generator, or edit-localization tool.

## Purpose

Context Pack is a read-only, evidence-backed, token-efficient context assembly layer.

It exists to help other agents and humans understand a repository faster by composing already available read-only facts:

- RepoGraph structure, components, commands, and repository context;
- Impact output at the repository/build/test layer;
- SymbolGraph-lite summaries;
- SourceEvidence facts and selector hints;
- SourceContext selectors and bounded slices;
- LSP diagnostics when a read-only diagnostics layer is available;
- known facts, unknowns, ambiguities, missing evidence, and limitations.

Context Pack does not decide what to change. It packages context so a downstream human or agent can decide outside the kernel.

## Non-Goals

Context Pack must not provide:

- edit location;
- edit target;
- patch;
- plan;
- PR;
- root cause;
- recommended change;
- `use this function` recommendation;
- `modify this symbol` recommendation.

`where-to-edit` remains `insufficient_evidence`. Context Pack must not be wired into `where-to-edit` as a decision layer.

## Relationship To Existing Layers

Context Pack is not localization. It is also not a replacement for SourceEvidence.

It composes existing read-only layers:

| Layer | Context Pack use |
| --- | --- |
| RepoGraph | Repository structure, components, commands, workspaces, warnings. |
| Impact | Build/test-level context and affected repository areas. |
| SymbolGraph-lite | Observed source files and top-level symbols. |
| SourceEvidence | Evidence candidates, missing evidence, selector hints. |
| SourceContext | Explicit selectors and bounded source slices. |
| LSP diagnostics | Diagnostics excerpts when available, without fix semantics. |

The owning layer remains the source of truth for each fact. Context Pack should reference evidence IDs and source layer IDs rather than inventing new facts.

## Field Language

Context Pack field names must avoid decision semantics.

Prefer:

- `context_files` over `relevant_files`;
- `observed_symbols` over `target_symbols`;
- `existing_capabilities` over `reuse_recommendations`;
- `coverage` over `confidence`;
- `evidence_channels` over `reasoning_paths`.

The contract should expose `decision_semantics: "not_supported"` at the top level.

## Budget Modes

`fast`:

- RepoGraph plus Impact plus SymbolGraph-lite summary only.
- No source text by default.
- No LSP startup.
- Designed for a compact first-pass context map.

`deep`:

- Includes `fast` output.
- Adds SourceEvidence.
- Adds SourceContext selectors and bounded slices when evidence supports them.
- Adds LSP diagnostics if the read-only diagnostics layer is available.

`very_deep`:

- Includes `deep` output.
- Runs multiple independent evidence channels.
- Reports convergence and disagreement between channels.
- Still does not convert agreement into edit targets or recommended changes.

## Output Format Modes

`compact`:

- IDs, short reasons, selector hints, warnings, and missing evidence.
- No source text by default.

`standard`:

- Bounded snippets and diagnostic excerpts.
- Enough context for a downstream agent or human to inspect without raw file exploration first.

`full`:

- Complete evidence details within deterministic limits.
- Still bounded by safety, ignore, path, and token-budget policies.

## Evidence Channels

An evidence channel is an independently produced read-only observation. Examples:

- RepoGraph component and command evidence.
- Impact traversal evidence.
- SymbolGraph-lite symbol evidence.
- SourceEvidence query match evidence.
- SourceContext slice evidence.
- LSP diagnostic evidence.

`convergence` means two or more channels point to the same contextual fact, such as the same file appearing in both RepoGraph component context and SourceEvidence output.

`disagreements` means channels conflict, are incomplete, or expose ambiguity. Disagreement is reported as evidence, not resolved into a decision.

## Draft CLI

The following syntax is reserved as a design sketch only. Do not implement it until the phase is explicitly opened.

```bash
code-intel context-pack "<query>" --budget fast --format compact --json
code-intel context-pack "<query>" --budget deep --format standard --json
code-intel context-pack "<query>" --budget very-deep --format compact --json
```

The CLI flag value `very-deep` should map to contract value `very_deep`.

## Evaluation Ideas

- Token budget measurement.
- Context Pack compactness.
- Evidence coverage.
- No edit-target-language tests.
- Ambiguity and refusal tests.
- Convergence/disagreement correctness.
- Comparison against raw file exploration.

## Deferral Rule

Implementation stays deferred until the existing read-only layers are stable enough to compose and the evaluation harness can prove that Context Pack output stays contextual. The first implementation phase, if approved later, should be CLI-only, read-only, and contract-gated. It must not add MCP, SQLite, embeddings, call graph, planner behavior, PR generation, or edit localization.
