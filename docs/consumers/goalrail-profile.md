# Goalrail Consumer Profile

Goalrail should use Code Intelligence Kernel as a strict/control-plane consumer.

Recommended profile mapping:

```text
Goalrail -> strict
```

Goalrail-specific policy should live outside core modules. The kernel provides generic evidence, impact, diagnostics, process-reward, and typed memory outputs.

Strict rails:

- require EvidenceBundle before edits;
- require impact analysis before patch approval;
- require verification status before final answers;
- require new evidence before retrying rejected hypotheses;
- keep tool expansion policy-gated;
- preserve provenance for injected context.
