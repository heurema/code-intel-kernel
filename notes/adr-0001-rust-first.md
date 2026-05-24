# ADR 0001: Rust-first implementation

## Status

Accepted.

## Context

The kernel should be reusable, local-first, structural-first, and conservative about dependencies. The first scaffold was created in TypeScript because the bootstrap prompt suggested it, but the product direction is a standalone kernel rather than a TypeScript application module.

## Decision

Implement the Code Intelligence Kernel as a Rust CLI/library first.

The kernel can still analyze TypeScript, TSX, JavaScript, Python, and other languages. Those are target repository languages, not the kernel implementation language.

## Consequences

- Phase 0 uses `Cargo.toml`, `src/lib.rs`, `src/main.rs`, Rust modules, and `tests/smoke.rs`.
- The CLI binary remains `code-intel`.
- Early implementation should stay stdlib-first where possible.
- Future Tree-sitter, SQLite, LSP, and MCP dependencies must be added only when the matching milestone needs them.
- Consumer-specific policy remains outside core modules.
