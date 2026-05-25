# arXiv Queries

Use date placeholders from `research-radar/sources.yaml`.

## P0 Daily: Code Agents

```text
(agent OR agents OR "coding agent" OR "software engineering agent")
AND (code OR repository OR "program repair")
AND submittedDate >= ${TODAY_MINUS_2D}
```

## P0 Daily: Code Graphs

```text
("code graph" OR "repository graph" OR "call graph" OR "Tree-sitter" OR "code intelligence")
AND submittedDate >= ${TODAY_MINUS_2D}
```

## Handling

- Record canonical arXiv URL.
- Record paper title, authors, date, abstract, and artifact URL if present.
- Do not treat benchmark claims as validated without artifact review.
