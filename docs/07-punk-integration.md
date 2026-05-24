# 07 — Punk Consumer Profile

## Role

Punk is an example prototype-mode consumer of Code Intelligence Kernel.

Same kernel, different policy.

## Punk mode

Punk values:

- fast exploration;
- lower context cost;
- lightweight evidence;
- fewer hard gates;
- high iteration speed.

## Differences from Goalrail

| Area | Goalrail | Punk |
|---|---|---|
| Strictness | high | medium/low |
| Evidence | required before edits | recommended |
| Process reward | required | recommended |
| Memory | governance + safety | iteration + continuity |
| MCP | policy-gated | read-only preferred |
| Mode | control plane | creative/prototype |

## Punk tools

Recommended default tools:

```text
repo_overview
where_to_edit
symbol_context
test_plan
memory_lookup
record_event
```

Optional:

```text
impact_analysis
patch_preflight
```

## What Punk should avoid

- forking its own code intelligence;
- project-specific repo parser;
- independent memory format;
- embeddings-first RAG separate from the kernel.

## Punk-specific success metrics

- time to useful repo context;
- number of tool calls before correct file;
- prototype iteration speed;
- broken refactor rate;
- repeated mistake rate;
- context tokens per task.
