---
name: using-wiki
description: Establish the Owly Wiki workflow before any Wiki task, inspect the current schema and state, and select the appropriate operation skill.
---

# Using Owly Wiki

Use this skill once at the beginning of a task involving a Owly Wiki,
before loading an operation-specific skill.

## Workflow

1. Identify the Wiki root and locate `owly.yml`.
2. Run `owly schema --format json` to inspect the current configuration.
3. Run `owly overview directories` and `owly overview types` when the task
   depends on Wiki structure or document types.
4. Select the operation skill that matches the user's request.
5. After any file change, run `owly validate` and report remaining diagnostics.

## Operation selection

- Use `wiki-config` to design or revise `owly.yml`.
- Use `wiki-overview` to understand structure, counts, and health.
- Use `wiki-search` to find structured metadata, links, or body text.
- Use `wiki-ingest` when preserving a source in `raw/` and deriving a page.
- Use `wiki-maintenance` to assess broken links, orphans, and stale content.

## Safety rules

- Do not invent types, fields, relationships, or placement rules.
- Do not overwrite, move, or delete existing files without explicit approval.
- Treat `owly` as the deterministic validator; the Agent makes semantic
  decisions and edits content.
