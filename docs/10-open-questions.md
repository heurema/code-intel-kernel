# 10 — Open Questions

## Product

1. Rust is the current implementation language; when should bindings/wrappers be added, if ever?
2. Should the CLI be the primary interface or should SDK come first?
3. Which real repo should be used as the first benchmark target?
4. What are the first Goalrail rails that must call the kernel?
5. What Punk workflows need fast mode first?

## Technical

1. Which Tree-sitter packages are stable enough for TS/Python?
2. Is TypeScript project diagnostics best accessed through `typescript-language-server`, `tsserver`, or a Rust wrapper/process bridge?
3. Should diagnostics be stored per snapshot or per run?
4. How to define stable selectors in a simple MVP?
5. How to avoid indexing secrets by default?

## Architecture

1. Should MCP be implemented as a package in the same repo or separate repo?
2. How should tool profiles be represented?
3. Should Goalrail policy live inside the kernel or outside it?
4. How much of session memory belongs to the kernel vs Goalrail?
5. When does a graph DB become necessary?

## Evaluation

1. What is the first 20-task benchmark?
2. What baseline should be used?
3. How to measure "agent did not repeat failed hypothesis"?
4. How to report confidence calibration?
5. Which metrics matter most for Goalrail vs Punk?

## Research

1. Which RIG concepts should be copied into RepoGraph?
2. Which Codebase-Memory ideas should be adapted?
3. How much Lanser-CLI-style process reward is needed in MVP?
4. Is codebadger/CPG useful for a security-only extension?
5. Do embeddings add value after structural retrieval works?
