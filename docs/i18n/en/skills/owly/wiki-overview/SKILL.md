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
2. Run `owly overview directories --statistics` and
   `owly overview types --statistics` to collect actual page counts.
3. Run `owly graph` to inspect links, broken references, orphan pages, and
   unreachable pages.
4. Run `owly validate` and report the current validation state.

Do not infer missing configuration from directory names or page content.
