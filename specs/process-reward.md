# ProcessReward Spec

## Purpose

ProcessReward gives agents and strict consumers a machine-checkable signal about patch quality.

It is not a full correctness proof.

## Inputs

- diagnostics before/after;
- changed files;
- intended scope;
- impacted tests;
- risk flags;
- symbol confidence;
- test plan confidence.

## Output

```json
{
  "score": 0.72,
  "diagnosticsDelta": {
    "before": 12,
    "after": 9,
    "newErrors": 1,
    "fixedErrors": 4
  },
  "editScopeOk": true,
  "impactedTestsKnown": true,
  "affectedSymbolsConfidence": 0.81,
  "testPlanConfidence": 0.74,
  "riskFlags": []
}
```

## Initial scoring heuristic

```text
base = 0.5
+0.2 if diagnostics improved
-0.2 if diagnostics worsened
+0.1 if edit scope matches evidence
-0.2 if edit scope violates evidence
+0.1 if tests are known
-0.1 if no tests are known
-0.1..0.4 based on risk flags
```

Clamp to `[0.0, 1.0]`.

## Interpretation

```text
>= 0.80 strong positive signal
0.60-0.79 acceptable but review
0.40-0.59 weak / needs human attention
< 0.40 block in strict profile
```

## Important caveat

No diagnostic errors does not mean the patch is correct. ProcessReward must be combined with tests, review, and domain constraints.
