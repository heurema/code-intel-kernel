# First prompt for Codex

You are in an empty project directory named `code-intel-kernel`.

A zip archive named `code-intel-kernel-bootstrap.zip` is present in the current directory. Your first job is to unpack it, read the documentation, and initialize the repository as a documentation-first project. Do not implement the full system yet.

## Step 1: Unpack

Run:

```bash
unzip -o code-intel-kernel-bootstrap.zip -d .
```

If `unzip` is unavailable, use Python:

```bash
python - <<'PY'
import zipfile
with zipfile.ZipFile("code-intel-kernel-bootstrap.zip") as z:
    z.extractall(".")
PY
```

## Step 2: Read the docs

Read in this order:

1. `README.md`
2. `docs/00-product-brief.md`
3. `docs/01-architecture.md`
4. `docs/03-mvp-roadmap.md`
5. `specs/domain-model.types.ts`
6. `specs/sqlite-schema.sql`
7. `specs/cli-contract.md`
8. `docs/09-risks-and-guardrails.md`

## Step 3: Summarize before coding

Before creating or changing implementation files, write a short summary in `notes/codex-initial-read.md` with:

- Your understanding of the product.
- The minimum viable implementation.
- Main risks.
- Files you plan to create.
- Any assumptions you are making.

## Step 4: Initialize repository skeleton only

Create a minimal repo skeleton suitable for future implementation. Use a Rust-first CLI/library shape.

Suggested skeleton:

```text
Cargo.toml
src/
  lib.rs
  main.rs
  core/
    mod.rs
    repo_graph.rs
    symbol_graph.rs
    evidence.rs
    process_reward.rs
    memory.rs
  storage/
    mod.rs
    sqlite.rs
  adapters/
    mod.rs
    tree_sitter.rs
    lsp.rs
tests/
  smoke.rs
notes/
  codex-initial-read.md
```

Do not build full functionality. Implement only placeholders/stubs where useful.

## Step 5: Produce a next-step plan

Create `notes/next-implementation-plan.md` with:

- Phase 0: repo skeleton.
- Phase 1: RepoGraph MVP.
- Phase 2: SymbolGraph MVP.
- Phase 3: LSP diagnostics bridge.
- Phase 4: EvidenceBundle and ProcessReward.
- Phase 5: optional MCP read-only server.

## Constraints

- Keep everything reversible.
- Prefer local-first, no external service dependency.
- Do not introduce Neo4j, vector DB, or MCP mutation tools in the first commit.
- No embeddings-first retrieval in Milestone 1.
- Start read-only.
- Treat Goalrail as strict/control-plane consumer.
- Treat Punk as fast/prototype consumer.
- Keep docs and decisions explicit.
