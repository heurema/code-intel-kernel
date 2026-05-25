# LSP Process Safety

Status: Phase 3A design draft. Not implemented.

Language servers are external processes. The kernel should treat them as read-only, bounded, and potentially unavailable.

## Process Lifecycle

Phase 3B should define a minimal lifecycle:

- discover server binary;
- report `unavailable` if missing;
- start process only for an explicit request;
- initialize with a contained workspace root;
- send read-only requests only;
- enforce request timeout;
- terminate process on completion or failure;
- return structured errors instead of panicking.

Long-running server reuse should be deferred until the basic contract is stable.

## Path Safety

Required checks:

- canonicalize workspace root;
- reject paths outside workspace root;
- reject symlink escapes;
- skip ignored/generated/cache directories;
- refuse missing or unreadable files with structured warnings;
- keep all emitted paths repository-relative.

Ignored path rules should align with RepoGraph, SymbolGraph-lite, and SourceContext.

## Request Limits

Phase 3B should define conservative defaults:

- max files opened;
- max diagnostics per report;
- max references per request;
- max definitions per request;
- max document symbols per file;
- max total JSON bytes if practical;
- timeout per request.

If a limit is exceeded, truncate deterministically and emit `result_limit_exceeded`.

## Deterministic Ordering

Sort results by:

1. repository-relative path;
2. start line;
3. start column;
4. severity/kind where relevant;
5. stable message/code string.

Do not expose server arrival order as meaningful.

## Allowed LSP Methods

Phase 3B candidate methods:

- `textDocument/publishDiagnostics` or equivalent diagnostics collection;
- `textDocument/definition`;
- `textDocument/references`;
- `textDocument/documentSymbol`.

## Forbidden LSP Methods

Do not call:

- `textDocument/formatting`;
- `textDocument/rangeFormatting`;
- `textDocument/codeAction`;
- `workspace/executeCommand`;
- `textDocument/rename`;
- any method that mutates files or asks the server to apply edits.

## Network and Environment

Do not assume network access. Do not install servers automatically. Do not download toolchains. If a server is missing or incompatible, return `unavailable`.

## Trust Model

LSP output is evidence from a tool, not ground truth. Reports should include server identity/version when available and preserve warnings for partial, stale, or unavailable data.
