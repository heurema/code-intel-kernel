# EvidenceBundle Spec

## Purpose

EvidenceBundle is the required bridge between repository facts and agent actions.

The agent should not simply say "I think this file is relevant." It should provide structured evidence.

## Shape

```json
{
  "taskId": "T-001",
  "claim": "Edit packages/web/src/auth/LoginForm.tsx.",
  "confidence": 0.87,
  "files": [],
  "symbols": [],
  "commands": [],
  "risks": [],
  "missingEvidence": []
}
```

## Required fields

- `claim`
- `confidence`
- `files`
- `symbols`
- `commands`
- `risks`
- `missingEvidence`

## Confidence rule

```text
>= 0.80 high confidence
0.60-0.79 medium confidence
0.40-0.59 low confidence
< 0.40 insufficient
```

## Strict profile requirement

In `strict`, no edit should proceed when confidence is below 0.80 unless explicitly overridden by the consumer.

## Prototype profile requirement

In `prototype`, low confidence should warn but not necessarily block unless the consumer adds stricter policy.

## Missing evidence

Always report what is missing:

```json
{
  "missingEvidence": [
    "No test file found for LoginForm.",
    "LSP references unavailable because TypeScript server was not running."
  ]
}
```
