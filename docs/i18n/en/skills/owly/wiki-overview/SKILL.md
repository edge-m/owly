---
name: wiki-overview
description: Inspect a Owly Wiki's structure, document types, page counts, relationships, and current validation state.
---

# Owly Wiki Overview Skill

Use this workflow to understand the wiki structure, document types, page
counts, relationships, and current validation state.

## Workflow

1. Read `<wiki-root>/entrance.md` as the graph entry, then run
   `owly schema --format json` to obtain the wiki root, configured types,
   fields, directory roles, and configuration diagnostics.
2. Run `owly inspect --format json` to collect the current page counts,
   configured directories and types, Graph state, validation state, and
   SQLite index state.
3. Use `owly overview directories --statistics` or
   `owly overview types --statistics` for a focused view, and `owly graph`
   when the individual links and edges are needed.
4. Run `owly validate` and report the current validation state.

Do not infer missing configuration from directory names or page content.
