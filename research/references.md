# Research References

This file keeps the R&D source list for the Code Intelligence Kernel. Use it as a reading queue and design map.

## Core large-codebase harness context

### Anthropic — How Claude Code works in large codebases

Key idea: coding-agent performance in large codebases depends on harness design: layered context, hooks, skills, plugins, MCP, LSP/code intelligence, and subagents.

URL:
https://www.anthropic.com/engineering/how-claude-code-works-in-large-codebases

Use for:
- layered harness framing;
- repo legibility;
- LSP/code intelligence as high-value investment;
- caution against root-only context and context bloat.

## R&D papers / systems

### Repository Intelligence Graph: Deterministic Architectural Map for LLM Code Assistants

URL:
https://arxiv.org/abs/2601.10112

Use for:
- RepoGraph;
- build/test graph;
- evidence-backed architecture map;
- agent-friendly JSON view.

### Codebase-Memory: Tree-Sitter-Based Knowledge Graphs for LLM Code Exploration via MCP

URL:
https://arxiv.org/abs/2603.27277

Use for:
- Tree-sitter SymbolGraph;
- persistent knowledge graph;
- MCP tool design;
- impact analysis.

### Language Server CLI Empowers Language Agents with Process Rewards

URL:
https://arxiv.org/abs/2510.22907

Use for:
- LSP bridge;
- process reward;
- diagnostic deltas;
- safe apply envelope;
- stable selectors.

### CodeMEM: AST-Guided Adaptive Memory for Repository-Level Iterative Code Generation

URL:
https://arxiv.org/abs/2601.02868

Use for:
- session memory;
- rejected hypotheses;
- AST-guided context;
- forgetting mitigation.

### RepoHyper: Search-Expand-Refine on Semantic Graphs for Repository-Level Code Completion

URL:
https://arxiv.org/abs/2403.06095

Use for:
- graph expansion retrieval;
- seed → expand → refine strategy;
- context bundle design.

### Bridging Code Property Graphs and Language Models for Program Analysis / codebadger

URL:
https://arxiv.org/abs/2603.24837

Use for:
- optional security analyzer;
- CPG/Joern MCP integration;
- slicing, taint tracking, data-flow analysis.

### MCP-Zero: Proactive Toolchain Construction for LLM Agents from Scratch

URL:
https://arxiv.org/abs/2506.01056

Use for:
- tool router;
- avoiding tool schema overload;
- agent-specific tool subset selection.

### RepoZero: Can LLMs Generate a Code Repository from Scratch?

URL:
https://arxiv.org/abs/2605.07122

Use for:
- execution-based evaluation;
- local benchmark design;
- repository-level verification mindset.

## Official technical foundations

### Language Server Protocol

URL:
https://microsoft.github.io/language-server-protocol/

Use for:
- LSP semantics;
- definitions/references/diagnostics;
- language-server reuse.

### Tree-sitter

URL:
https://tree-sitter.github.io/tree-sitter/

Use for:
- incremental parsing;
- broad parser coverage;
- syntax tree extraction.

### Model Context Protocol

URL:
https://modelcontextprotocol.io/docs/getting-started/intro

Use for:
- optional read-only MCP surface;
- tool integration protocol;
- future agent integrations.

### SQLite FTS5

URL:
https://www.sqlite.org/fts5.html

Use for:
- lightweight full-text search over snippets/docs/comments;
- local-first storage.
