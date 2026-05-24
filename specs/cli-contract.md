# CLI Contract

## Binary name

```bash
code-intel
```

## Commands

### inspect

```bash
code-intel inspect <repo-path> [--json]
```

Scans repository and writes local cache.

### repo-map

```bash
code-intel repo-map [--scope <path>] [--json]
```

Returns compact repository map.

### where-to-edit

```bash
code-intel where-to-edit "<task>" [--profile strict|standard|prototype|research|custom] [--json]
```

Returns EvidenceBundle.

Consumer-specific mappings live outside the kernel. For example, Goalrail can map to `strict`, and Punk can map to `prototype`, but the CLI contract should not encode those project names.

### symbol-context

```bash
code-intel symbol-context "<symbol-or-query>" [--json]
```

Returns symbol context.

### impact

```bash
code-intel impact --changed-files <paths> [--json]
code-intel impact --diff patch.diff [--json]
```

Returns impacted packages, symbols, tests, commands, risks.

### test-plan

```bash
code-intel test-plan --changed-files <paths> [--json]
```

Returns minimal and fallback commands.

### diagnostics

```bash
code-intel diagnostics <repo-path> [--run-id <id>] [--json]
```

Collects diagnostics snapshot.

### diagnostic-delta

```bash
code-intel diagnostic-delta --before <run-id> --after <run-id> [--json]
```

Returns delta.

### preflight

```bash
code-intel preflight patch.diff [--json]
```

Returns ProcessReward.

### memory

```bash
code-intel memory record --event event.json
code-intel memory lookup "<query>" [--json]
```

## Exit codes

```text
0 success
1 general failure
2 low confidence
3 risky operation blocked
4 diagnostics worsened
5 invalid input
```

## Output principle

Every JSON command returns:

```json
{
  "ok": true,
  "data": {},
  "evidence": [],
  "confidence": 0.0,
  "warnings": []
}
```
