# Codex Experiment Proposal Prompt

Create an experiment proposal markdown file only.

Do not implement code.
Do not import external code.
Do not change runtime contracts.
Do not change `where-to-edit`.

Required fields:

- title
- source_url
- source_type
- hypothesis
- affected_modules
- minimal_reversible_change
- expected_signal
- evaluation_plan
- agent_bench_lab_fit
- agent_bench_lab_eval_handoff
- agent_bench_lab_blockers
- fixtures_or_benchmarks_needed
- contract_risk
- licensing_attribution_notes
- security_notes
- stop_condition
- reason_not_to_implement_immediately

The proposal must explain why the experiment should remain separate from mainline feature work until approved.
The proposal must not assume Agent Bench Lab is complete. If the benchmark layer cannot evaluate the idea yet, record the blocker instead of converting the idea into implementation work.
