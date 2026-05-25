# GitHub Queries

Use date placeholders from `research-radar/sources.yaml`.

## P0 Daily: Research Repositories

```text
("code intelligence" OR "coding agent" OR "repository graph" OR "Tree-sitter")
pushed:>=${TODAY_MINUS_7D}
```

## P0 Watchlist

- Tree-sitter GitHub repository: URL needs verification.
- rust-analyzer GitHub repository: URL needs verification.
- Codebase-Memory repository: URL needs verification.
- RIG/SPADE repository: URL needs verification.
- Aider RepoMap repository/docs: URL needs verification.

## Handling

- Record canonical repository URL.
- Record latest release, issue, PR, or commit only when relevant.
- Record license.
- Do not import or copy code.
- Do not run watched repositories locally during intake.
