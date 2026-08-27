---
name: wiki-ingest
description: Preserve an external or local source in raw/, derive a Owly Wiki page with formal sources metadata, and validate the result without silently overwriting files.
---

# Owly Wiki Ingest

Use this workflow when a new Wiki page should be derived from a source file.
Owly validates the relationship; the Agent decides what the source means
and how it should be summarized.

## Workflow

1. Inspect the rules with `owly schema --format json`.
2. Preserve the source under the configured `raw/` directory. Do not delete or
   overwrite an existing source without explicit approval.
3. Read the source and decide whether a new page or an update is appropriate.
4. Add a `sources` record to the page frontmatter:

   ```yaml
   sources:
     - path: raw/inbox/example.md
       sha256: <sha256 of the saved source file>
       origin: https://example.com/optional-origin
   ```

5. Run `owly validate --file <page>` and then `owly validate`.
6. Run `owly validate --details --format json` when checking tracked sources later.

Do not infer missing source content. Do not silently edit, move, or delete a
page or raw source. Use `rg` for body-text searches; structured source checks
belong to `owly validate`.
