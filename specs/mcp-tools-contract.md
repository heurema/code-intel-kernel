# MCP Tools Contract

## Status

Deferred until CLI and SDK work.

## Principle

Start read-only. No arbitrary shell, no arbitrary file writes.

## Tools

### repo_overview

Input:

```json
{ "scope": "optional/path" }
```

Output:

```json
{
  "packages": [],
  "commands": [],
  "risk_notes": []
}
```

### where_to_edit

Input:

```json
{
  "task": "change login validation copy",
  "profile": "strict"
}
```

Output: EvidenceBundle.

### symbol_context

Input:

```json
{ "query": "LoginForm" }
```

Output: definitions, references, imports, exports, tests.

### impact_analysis

Input:

```json
{ "changed_files": ["src/a.ts"] }
```

Output: impacted packages, tests, commands, risks.

### test_plan

Input:

```json
{ "changed_files": ["src/a.ts"] }
```

Output: minimal commands and fallback commands.

### patch_preflight

Input:

```json
{ "diff": "..." }
```

Output: ProcessReward.

### memory_lookup

Input:

```json
{ "query": "login validation" }
```

Output: related events and decisions.

## Forbidden in early MCP

- shell execution;
- git commit/push;
- arbitrary file write;
- network fetch;
- secrets access;
- mutation/refactor operations.

## Tool profiles

The strict profile should expose fewer tools and require stronger confidence than the prototype profile.
