---
name: using-wiki
description: Establish the Owly Wiki workflow before any Wiki task, inspect the current schema and state, and select the appropriate operation skill.
---

# Using Owly Wiki

Use this skill once at the beginning of a task involving a Owly Wiki,
before loading an operation-specific skill.

## Workflow

1. Identify the Wiki root and locate `owly.yml`.
2. Read the required `<wiki-root>/entrance.md` as the entry point for this
   graph. If it is missing, report the invalid Wiki state.
3. Run `owly schema --format json` to inspect the current configuration.
4. Run `owly inspect --format json` to obtain the current Wiki state. Use
   `owly overview directories` or `owly overview types` for a focused view
   when the task depends on one specific part of the schema.
5. Select the operation skill that matches the user's request.
6. After any file change, run `owly validate` and report remaining diagnostics.

## Operation selection

- Use `wiki-config` to design or revise `owly.yml`.
- Use `wiki-query` to find structured metadata, links, body text, or basic structure.
- Use `wiki-ingest` when preserving a source in `raw/` and deriving a page.
- Use `wiki-maintenance` to assess broken links, orphans, and stale content.

## Safety rules

- Do not invent types, fields, relationships, or placement rules.
- Do not overwrite, move, or delete existing files without explicit approval.
- Treat `owly` as the deterministic validator; the Agent makes semantic
  decisions and edits content.
