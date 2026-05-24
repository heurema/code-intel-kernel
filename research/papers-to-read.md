# Papers to Read First

## Reading order

### 1. Repository Intelligence Graph

Read first because it maps most directly to RepoGraph MVP.

Questions:

- What node/edge types are essential?
- How much build/test information can be extracted deterministically?
- How should the graph be serialized for agents?

### 2. Lanser-CLI

Read second because Goalrail needs process reward.

Questions:

- How do they define stable selectors?
- How do they compute diagnostic deltas?
- What safety envelope is realistic for early MVP?

### 3. Codebase-Memory

Read third because it maps to SymbolGraph + MCP.

Questions:

- Which graph queries provide highest value?
- How do they handle Tree-sitter language coverage?
- Which MCP tools are useful without mutation?

### 4. CodeMEM

Read fourth because memory can easily become vague summaries.

Questions:

- How do they prevent forgetting?
- Which AST-guided memory operations are MVP-worthy?
- How should rejected hypotheses be represented?

### 5. RepoZero

Read when designing the benchmark.

Questions:

- What can be execution-verified locally?
- How to avoid LLM-judge dependence?
- What task set reflects Goalrail/Punk workflows?

### 6. codebadger / CPG

Read after MVP.

Questions:

- Which security flows need CPG?
- Can CPG be optional and on-demand?
- Which languages matter most?
