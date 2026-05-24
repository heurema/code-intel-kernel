# 04 — Data Model

## Core entities

```text
Repo
Package
File
Symbol
Import
Command
Diagnostic
Event
Decision
EvidenceBundle
ProcessReward
```

## Node types

```text
repo
package
file
module
function
class
method
interface
type_alias
import
export
test
config
command
decision
event
```

## Edge types

```text
contains
imports
exports
defines
calls
references
tests
depends_on
belongs_to_package
has_command
modified_in_session
mentioned_in_decision
risk_related_to
```

## Confidence

Every inferred fact should carry a confidence:

```text
1.0  directly parsed from config or source
0.8  derived from conventional naming
0.6  heuristic inference
0.4  weak textual match
```

## Evidence

Every ranked result should include human-readable evidence:

```json
{
  "path": "packages/web/src/auth/LoginForm.tsx",
  "reason": "Contains symbol LoginForm and imports useAuth; route /login references it.",
  "score": 0.89
}
```

## Stable IDs

Avoid brittle line-only IDs. Prefer:

```text
file_hash + symbol_name + kind + range
```

For future selector design, consider:

```text
symbolic selector
AST path selector
content-anchored selector
file path + range fallback
```

## Event model

Events should be append-only.

Example:

```json
{
  "event": "hypothesis_rejected",
  "task_id": "T-123",
  "reason": "packages/server/src/auth.ts handles token validation, but user requested UI copy.",
  "evidence": [
    "packages/server/src/auth.ts",
    "packages/web/src/auth/LoginForm.tsx"
  ],
  "created_at": "2026-05-24T12:00:00Z"
}
```

## ProcessReward inputs

```text
diagnostic_delta
edit_scope_ok
affected_symbols_confidence
test_plan_confidence
risk_flags
new_errors
fixed_errors
```

## ProcessReward output

```json
{
  "score": 0.72,
  "diagnostics_delta": {
    "before": 12,
    "after": 9,
    "new_errors": 1,
    "fixed_errors": 4
  },
  "edit_scope_ok": true,
  "impacted_tests_known": true,
  "risk_flags": []
}
```
