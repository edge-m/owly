---
name: owly
description: Use Owly to inspect, search, and validate a structured Markdown wiki.
---

# Owly Wiki Skills

Owly is the deterministic validator for a structured Markdown wiki. Use the
specialized Skills in this directory for configuration, overview, ingestion,
maintenance, and search workflows.

## Workflow

1. Locate `owly.yml` and inspect the schema with `owly schema --format json`.
2. Use `owly overview directories` and `owly overview types` to understand the
   wiki structure.
3. Make the requested wiki changes.
4. Run `owly validate --details` and fix reported errors.
5. Use `owly graph` and `owly search` when investigating relationships or
   structured frontmatter.

Treat `owly` as the source of truth for validation. Do not silently overwrite
an existing configuration file.
