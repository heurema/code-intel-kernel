# 06 — Goalrail Consumer Profile

## Role

Goalrail is an example strict/control-plane consumer of Code Intelligence Kernel.

Goalrail does not need to parse code itself. It should ask the kernel for evidence, impact, diagnostics, risk, and memory.

## Strict rails

### Rail 1 — No edit without evidence

Before an agent edits code, it must obtain an EvidenceBundle.

### Rail 2 — No patch without impact analysis

Before applying or approving a diff, Goalrail asks for impacted files, tests, packages, and risk flags.

### Rail 3 — No final answer without verification status

The agent must report one of:

```text
verified
partially_verified
not_verified
blocked
```

### Rail 4 — No repeated failed hypothesis

If a hypothesis was previously rejected, the agent must provide new evidence before retrying.

### Rail 5 — No tool expansion without policy

Goalrail controls which tools are exposed.

### Rail 6 — Source/provenance required

Context injected into the agent must have provenance:

```text
file
symbol
config
diagnostic
event
decision
external reference
```

## Goalrail flow

```text
TASK_INTENT
  ↓
repo_overview
  ↓
where_to_edit
  ↓
EvidenceBundle approval
  ↓
plan edit
  ↓
impact_analysis
  ↓
patch_preflight
  ↓
apply / simulate
  ↓
diagnostic_delta / test result
  ↓
record_event / record_decision
```

## Policy modes

### strict

Use for production code, security-sensitive code, auth, payments, infrastructure.

### standard

Use for typical feature work.

### prototype

Use for Punk-like exploratory work.

## Consumer contract

Goalrail should be able to call:

```ts
const evidence = await kernel.whereToEdit(task)
const impact = await kernel.analyzeImpact(diff)
const reward = await kernel.preflightPatch(diff)
await kernel.recordEvent(event)
```

## Goalrail-specific metrics

- unsafe edit prevented;
- low-confidence edit blocked;
- repeated hypothesis prevented;
- tests selected correctly;
- diagnostic regressions caught before final;
- verification status accuracy.
