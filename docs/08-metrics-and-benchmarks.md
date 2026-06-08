# 08 — Metrics and Benchmarks

## Why metrics matter

Do not evaluate this module by "the agent feels smarter." Evaluate by concrete agent workflow improvements.

## Core metrics

### Retrieval and localization

```text
correct_file@1
correct_file@3
correct_symbol@1
correct_symbol@5
tool_calls_to_correct_context
tokens_to_correct_context
```

### Test planning

```text
test_plan_precision
test_plan_recall
minimal_command_accuracy
fallback_command_accuracy
```

### Patch preflight

```text
diagnostic_delta_accuracy
new_error_detection
fixed_error_detection
scope_violation_detection
risk_flag_precision
```

### Agent workflow

```text
patch_success_rate
human_rescue_rate
repeated_mistake_rate
stale_context_rate
low_confidence_action_rate
```

### Goalrail-specific

```text
unsafe_edits_prevented
hypothesis_retries_blocked
unverified_final_answers_blocked
```

### Punk-specific

```text
time_to_repo_understanding
prototype_loop_time
context_cost_per_task
```

## Mini-benchmark design

Create 20-50 local tasks from real repos.

Task categories:

1. find correct file;
2. find correct symbol;
3. explain module boundary;
4. propose minimal edit plan;
5. predict impacted tests;
6. detect risky edit;
7. patch simple bug;
8. patch cross-file bug;
9. avoid rejected hypothesis;
10. verify diagnostic improvement.

## Agent Bench Lab handoff

`code-intel-kernel` owns the hypothesis, evidence contract, and expected workflow signal.
Agent Bench Lab owns benchmark task families, scorers, run records, and compare protocol.

Every Research Radar experiment proposal should map the idea to:

```text
hypothesis
expected_signal
candidate Agent Bench Lab suite or task family
public smoke check vs private holdout need
run-validity or harness blocker
baseline setup
candidate setup
comparison metric
```

If Agent Bench Lab cannot evaluate the idea yet, keep that as a benchmark-layer blocker. Do not implement the idea in `code-intel-kernel` merely because it is interesting.

## Execution-based evaluation

Prefer execution-based checks where possible:

```text
tests pass/fail
diagnostics improve/worsen
expected file selected
expected symbol selected
expected command selected
```

Avoid relying only on LLM-as-judge.

## Baselines

Compare against:

```text
grep-only agent
file-exploration agent
agent + RepoGraph
agent + RepoGraph + SymbolGraph
agent + RepoGraph + SymbolGraph + LSP
```

## Reporting format

For every benchmark run:

```json
{
  "task_id": "T-001",
  "baseline": "grep-only",
  "variant": "repo-symbol-lsp",
  "correct_file_at_3": true,
  "correct_symbol_at_5": true,
  "tool_calls": 7,
  "context_tokens": 8420,
  "diagnostic_delta": {
    "before": 4,
    "after": 2
  }
}
```
