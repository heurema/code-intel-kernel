# Research Radar Scoring

Score each item from 0 to 100. No item can trigger implementation automatically.

## Dimensions

Use these dimensions as a checklist. Weights are intentionally rough for v0.1.

- Project relevance: Does it affect RepoGraph, SymbolGraph-lite, SourceEvidence, SourceContext, LSP, benchmarks, or future MCP?
- Novelty: Does it add a new technique, evaluation signal, or failure mode?
- Implementation availability: Is there a paper, repo, benchmark, release, or reproducible artifact?
- Evidence quality: Are claims backed by code, data, benchmarks, or clear methodology?
- Source credibility: Is the source a known lab, maintained repo, benchmark, or peer-reviewed venue?
- Reproducibility: Can the result be checked locally without private access or fragile services?
- Agent Bench Lab fit: Can the expected signal be evaluated through an existing or clearly proposed Agent Bench Lab suite, task family, scorer, or compare protocol?
- Local-first fit: Can the idea work without hosted dependencies or login-gated APIs?
- Rust/Rust-compatible feasibility: Does it fit a Rust-first kernel or expose a clean protocol/data boundary?
- Safety/security risk: Does it avoid unsafe scraping, untrusted code execution, or unclear licensing?
- Scope creep risk: Can it be tested as a tiny reversible experiment?

## Thresholds

- `>=85`: human review and experiment card.
- `70-84`: daily digest.
- `55-69`: weekly backlog.
- `<55`: archive.

## Hard Stops

Score does not override guardrails.

- Do not import external code automatically.
- Do not create implementation tasks without human approval.
- Do not copy code without license review.
- Do not scrape login-gated or restricted sources unless explicitly configured.
- Do not treat benchmark claims as validated until locally reviewed.
- Do not raise an item to experiment proposal if the Agent Bench Lab evaluation path is unknown and no explicit benchmark-layer blocker is recorded.
