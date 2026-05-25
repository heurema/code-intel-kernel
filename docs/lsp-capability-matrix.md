# LSP Capability Matrix

Status: Phase 3A design draft. Not implemented.

The LSP bridge should start read-only and diagnostic/reference-focused.

| Capability | Phase | Reason |
| --- | --- | --- |
| Diagnostics | Phase 3B candidate | Core missing evidence for compile/type errors and validation. |
| Diagnostic delta | Phase 3B candidate | Useful process signal without mutation. Keep in memory first. |
| Go to definition | Phase 3B candidate | Helps disambiguate same-name symbols through explicit selectors. |
| Find references | Phase 3B candidate | Directly addresses Phase 2G reference/call-graph refusal cases. |
| Document symbols | Phase 3B candidate | Useful cross-check against SymbolGraph-lite; read-only and bounded. |
| Hover/type info | Defer | Useful later, but often verbose and server/version dependent. |
| Workspace symbols | Defer | Can return broad noisy results; needs strict limits and eval first. |
| Implementation lookup | Defer | Useful after basic definitions/references are stable. |
| Call hierarchy | Defer | More semantic and expensive; avoid before references are evaluated. |
| Semantic tokens | Defer | Mostly presentation/context; not required for first evidence bridge. |
| Formatting | Avoid for now | Mutation-adjacent and not evidence collection. |
| Code actions | Avoid for now | Mutation-capable and easy to misuse as edit planning. |
| Rename | Avoid for now | Mutation-capable. Requires much stronger safety and review gates. |

## Phase 3B Candidate Set

Phase 3B should implement only:

- diagnostics;
- diagnostic delta if cheap;
- go to definition;
- find references;
- document symbols.

All requests should be explicit, bounded, read-only, and evidence-backed.

## Deferred Capabilities

Defer hover/type info, workspace symbols, implementation lookup, call hierarchy, and semantic tokens until:

- Phase 3B is stable;
- LSP eval cases exist;
- result limits and ordering are validated;
- SourceEvidence integration remains non-localizing.

## Avoided Capabilities

Avoid formatting, code actions, rename, and any mutation method until a separate mutation safety design exists. These are not needed for diagnostics/reference evidence and would increase accidental edit-planning risk.
