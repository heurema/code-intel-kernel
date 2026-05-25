# Phase 2G: Adversarial Localization Gate

Phase 2G adds refusal-oriented eval coverage and readiness documentation. It is a gate, not a localization implementation.

## What Changed

- Added adversarial source-evidence eval cases for duplicate symbol names, reference-style queries, unsupported-language input, and component/query matches without source symbols.
- Added adversarial source-context eval cases for path escape refusal and malformed-source slicing.
- Extended eval expectations with max candidate counts and runtime output forbidden-string checks.
- Added smoke coverage proving selector hints and SourceContext slices do not make `where-to-edit` confident.
- Added readiness documentation for adversarial localization risks.

## Contracts

- `inspect`: `0.2`
- `impact`: `0.2`
- `symbols`: `0.1`
- `source_evidence`: `0.3`
- `source_context`: `0.1`
- `eval`: `0.4`

The eval report shape did not change. New expectations are case schema additions only, so `eval_contract_version` remains `0.4`.

## Adversarial Coverage

- Duplicate same-name functions return evidence candidates with ambiguity, not edit locations.
- Reference-style queries expose missing reference/call-graph evidence.
- Unsupported-language files do not become SymbolGraph evidence.
- Component or command text does not create source candidates without source evidence.
- SourceContext rejects path traversal and still allows explicit safe text slicing for malformed local files.
- Runtime JSON is checked for edit-target language.

## Readiness Conclusion

`not_ready_for_confident_localization`

The system has evidence-backed source facts, selector hints, and bounded snippets, but it still lacks reference resolution, call graph, LSP diagnostics, method-level extraction, semantic type information, and evaluated localization metrics.

## Recommended Next Phase

Phase 3A should be LSP diagnostics/reference bridge design, not implementation. The design should clarify how references and diagnostics will be captured without weakening current refusal behavior or turning selector hints into edit targets.
