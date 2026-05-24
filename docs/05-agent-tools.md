# 05 — Agent Tools

## Tool design principle

Expose high-level tools, not arbitrary graph queries at first.

Agents should ask:

```text
where should I edit?
what context do I need?
what is impacted?
what tests should run?
did this patch improve things?
what did I already try?
```

Not:

```text
run arbitrary SQL
execute arbitrary shell
read any file without reason
write to files
```

## Proposed tools

### repo_overview(scope?)

Returns compact repository map.

```json
{
  "root": ".",
  "packages": [],
  "commands": [],
  "risk_notes": []
}
```

### where_to_edit(task)

Returns ranked files and symbols.

```json
{
  "task": "change login validation copy",
  "candidates": [],
  "missing_evidence": []
}
```

### symbol_context(symbol_or_query)

Returns definition, references, imports, exports, tests.

### impact_analysis(files_or_diff)

Returns affected packages, commands, tests, and risk boundaries.

### test_plan(diff_or_files)

Returns minimal commands plus fallback commands.

### diagnostics(scope)

Returns diagnostics snapshot.

### diagnostic_delta(before, after)

Returns improvements/regressions.

### patch_preflight(diff)

Returns process reward and risk flags.

### memory_lookup(task)

Returns related previous decisions and rejected hypotheses.

### record_event(event)

Records typed event.

## Tool profiles

### Strict profile

Allowed tools:

```text
repo_overview
where_to_edit
symbol_context
impact_analysis
test_plan
diagnostics
diagnostic_delta
patch_preflight
memory_lookup
record_event
```

Hard requirements:

```text
evidence before edit
impact before patch
verification before final answer
```

### Prototype profile

Allowed tools:

```text
repo_overview
where_to_edit
symbol_context
test_plan
memory_lookup
record_event
```

Soft requirements:

```text
prefer evidence
warn on low confidence
allow prototype mode
```

## MCP read-only mode

When adding MCP, start with read-only tools only.

Do not expose:

- arbitrary shell execution;
- arbitrary file writes;
- git commit;
- refactor/mutation tools;
- network access.

## Tool response shape

Every tool should return:

```json
{
  "ok": true,
  "data": {},
  "evidence": [],
  "confidence": 0.0,
  "warnings": []
}
```
