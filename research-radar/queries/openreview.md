# OpenReview Queries

Use weekly cadence for v0.1.

```text
("code agent" OR "software engineering agent" OR "repository")
updated >= ${TODAY_MINUS_30D}
```

## Handling

- Record canonical OpenReview URL.
- Record venue, status, authors, and artifact link if present.
- Treat under-review claims as unvalidated.
- Do not scrape login-gated content.
