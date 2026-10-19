---
name: wiki-query
description: Use when answering questions from an Owly Wiki or inspecting its structure, metadata, links, or full-text content. Keep this skill read-only; use wiki-ingest or wiki-maintenance for changes.
---

# Query an Owly Wiki

Use this skill to retrieve evidence from one graph without changing Wiki
source files. `owly search` may refresh the derived SQLite index automatically.

## Workflow

1. Identify the Wiki root and read `<wiki-root>/entrance.md`.
2. Read the relevant `owly.yml` with `owly schema --config <config> --format json`.
3. For structured metadata, use `owly search --config <config> --query '<field>:<value>' --format json`.
4. For relationships, use `owly graph --config <config>` and follow the relevant paths.
5. For body text or an exact phrase, use `rg` within the Wiki root after narrowing the candidate paths.
6. Cite the source path and distinguish what the Wiki states from an inference.

Do not edit, normalize, or silently repair Wiki source files while answering a
query. Search may create or refresh `.owly/index.sqlite` and its local metadata
cache; these are derived and reproducible. Route requests to save knowledge to
`wiki-ingest`, and requests to repair or reorganize existing knowledge to
`wiki-maintenance`.

## Safety rules

- Treat `entrance.md` as the entry point of the selected graph.
- Do not treat a broken or external link as evidence that its target exists.
- Do not inspect an external graph when answering a single-graph query unless
  the user explicitly asks for that separate graph.
- Never invent fields, types, or relationships to fill a gap in the evidence.
