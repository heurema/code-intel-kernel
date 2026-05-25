# Codex Daily Research Prompt

You are running the `code-intel-kernel` Research Radar daily intake.

Read:

- `research-radar/sources.yaml`
- `research-radar/scoring.md`
- `research-radar/guardrails.md`
- `research-radar/state/seen.jsonl` if it exists

Task:

1. Fetch only configured public sources.
2. Use date placeholders from `sources.yaml`.
3. Normalize, dedupe, and score items using `scoring.md`.
4. Produce:
   - `research-radar/reports/YYYY-MM-DD.md`
   - `research-radar/reports/YYYY-MM-DD.json`
5. Return at most:
   - 3 top ideas;
   - 1 experiment candidate.

Rules:

- Do not modify runtime code.
- Do not import external code.
- Do not copy code from external repositories.
- Do not run external repository code.
- Do not schedule automation.
- Do not propose implementation unless score is at least 85 and code/artifact exists.
- Preserve canonical URLs, attribution, and license/terms notes.
- Put uncertain source URLs as `null` and mark `status: needs_verification`.
- Add all failures to `source_health` and `errors`.
- Keep the daily digest short enough for human review.

Output must include a `do-not-act-yet` section.
