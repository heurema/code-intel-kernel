# Example Custom Consumer

Custom integrations should map their local policy needs onto generic kernel profiles instead of adding project names to core contracts.

```ts
const consumerProfiles = {
  docsAgent: "standard",
  researchAgent: "research",
  localPrototype: "prototype",
} as const;
```

Core modules should only receive:

```ts
profile: "strict" | "standard" | "prototype" | "research" | "custom"
```

Keep project-specific routing, policy, and naming in the consumer layer.
