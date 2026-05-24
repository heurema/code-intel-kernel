# Punk Consumer Profile

Punk should use Code Intelligence Kernel as a fast/prototype consumer.

Recommended profile mapping:

```text
Punk -> prototype
```

Punk-specific behavior should live outside core modules. The kernel provides shared repo facts, symbol context, lightweight evidence, test-plan hints, and typed memory.

Prototype-mode defaults:

- prefer fast exploration;
- keep evidence lightweight but structured;
- avoid hard gates unless configured by the consumer;
- use the shared memory and repo graph formats;
- avoid a separate embeddings-first RAG path.
