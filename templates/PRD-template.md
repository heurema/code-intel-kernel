# PRD: Code Intelligence Kernel

## Problem

AI coding agents need structured repo understanding and process signals for Goalrail and Punk.

## Users

- Goalrail orchestrator / policy runtime.
- Punk prototype agent.
- Human developer using CLI.

## MVP

- RepoGraph.
- SymbolGraph for TS/Python.
- LSP diagnostics bridge.
- EvidenceBundle.
- ProcessReward.
- Typed session memory.

## Exclusions

- Full RAG.
- Full MCP mutation tools.
- Full CPG analyzer.
- UI.

## Success metrics

- correct_file@3
- correct_symbol@5
- test_plan_precision
- diagnostic_delta_accuracy
- human_rescue_rate
- repeated_mistake_rate
