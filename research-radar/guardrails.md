# Research Radar Guardrails

Research Radar is intake, not implementation.

## Hard Rules

- No automatic implementation from external sources.
- No code copy without license review.
- No login-required scraping unless explicitly configured.
- No uncontrolled crawling.
- No source without attribution.
- No daily report item can trigger code changes automatically.
- Every prototype requires human approval.
- Preserve canonical URLs.
- Preserve license and terms notes.
- Treat external repositories as research input, not dependencies, unless separately approved.
- Generated experiment proposals must include a stop condition.
- Generated experiment proposals must include a reason not to implement immediately.
- Codex App scheduled automation may write only `research-radar/reports/**` and `research-radar/state/**`.
- Codex App scheduled automation must fail instead of continuing if runtime or configuration files change unexpectedly.
- Codex App scheduled automation must not commit automatically.

## Safety Boundaries

- Do not run external repository code during intake.
- Do not install dependencies from watched repositories.
- Do not create patches from watched repositories.
- Do not use generated code from papers or repos without attribution and license review.
- Do not turn Research Radar into `where-to-edit`, roadmap automation, or a repo-owned scheduler.
- Do not use Research Radar automation to create patches, code-intelligence features, or runtime changes.

## Human Approval Gate

A daily digest may propose one experiment candidate only when:

- score is at least 85;
- source is public;
- artifact or evidence exists;
- licensing status is recorded;
- security concerns are recorded;
- minimal reversible change is clear;
- stop condition is clear.

Even then, implementation requires explicit human approval.
