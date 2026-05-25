# Phase 3B-A: Rust LSP Diagnostics

Phase 3B-A implements the first narrow LSP runtime slice: read-only Rust diagnostics.

## Scope

- Rust only.
- `rust-analyzer` only.
- Diagnostics only.
- Read-only.
- In-memory.
- No definitions.
- No references.
- No hover/type info.
- No formatting, code actions, rename, or mutation methods.
- No SQLite.
- No MCP.
- No `where-to-edit` integration.

## Dependency Choice

No new dependencies were added.

The bridge uses:

- `std::process` for the rust-analyzer process;
- `std::io` for JSON-RPC framing;
- existing `serde`/`serde_json` for messages and output.

This keeps Phase 3B-A small and avoids async runtimes, server frameworks, or heavy LSP client libraries.

## Runtime Behavior

`lsp-diagnostics` validates requested files before starting rust-analyzer.

It returns:

- `unavailable` when rust-analyzer is not installed or cannot run;
- `error` for invalid requests such as paths outside the repo, ignored paths, or missing files;
- `ok` when diagnostics collection completes without partial warnings;
- `partial` when diagnostics are collected with limits or partial warnings.

Unavailable rust-analyzer is an expected, tested path.

## Safety

Implemented or enforced:

- repository root canonicalization;
- repository-relative file selectors;
- parent/root path component refusal;
- ignored/generated path refusal;
- symlink refusal;
- missing file warnings;
- Rust-only file extension check;
- bounded timeout;
- bounded diagnostics count;
- deterministic sorting;
- no mutation methods.

## Localization

Diagnostics are evidence only. They are not fixes, root-cause claims, edit targets, or `where-to-edit` inputs.

Current localization readiness remains:

`not_ready_for_confident_localization`

## Phase 3B-B Recommendation

Next slice: definitions/references design-to-implementation, still read-only and explicit-selector only.

Keep tests independent from local rust-analyzer timing by requiring mocked/unavailable paths by default and gating real rust-analyzer integration behind an env var.
