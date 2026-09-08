# Source File Ingest Reference

Read this reference when applying `wiki-ingest` to a source file.

## Source preservation

- If the source is already under the configured raw directory, keep its path.
- Otherwise copy it into the raw area before deriving Wiki knowledge.
- Choose a stable, collision-free path. Ask before replacing an existing file.
- Record the preserved path in the page's `sources` metadata.
- Compute the SHA-256 of the preserved source file for source tracking; this is
  provenance metadata, not a substitute for reading the source.

## Distillation

Read enough of the source to identify its topic, claims, entities, and useful
relationships. Do not transcribe the source or create one page per paragraph.

- Markdown and text: read directly; preserve important headings and claims.
- PDF or other binary documents: use an available document reader; if the
  contents cannot be read, report the limitation instead of guessing.
- Large files: read in bounded portions and retain only the information needed
  for the page and its links.
- If the source contradicts an existing page, preserve both claims with clear
  provenance and report the contradiction; do not silently choose a winner.

## Page choice

Before writing, search for an existing page covering the same subject. Update
that page when the source adds evidence or changes its understanding. Create a
new page only when no existing page is an appropriate home.

Every created or updated page should have valid frontmatter for the configured
type and a source record such as:

```yaml
sources:
  - path: raw/inbox/example.md
    sha256: <sha256 of the preserved source file>
    origin: https://example.com/optional-origin
```

Use the page's configured type and fields; never invent a schema when
`owly schema` provides one.

## Graph integration

Link the page from `entrance.md` when it is a top-level entry, otherwise link
it from the most relevant existing page. Use links to connect related concepts,
entities, and source-derived pages. Do not add speculative links merely to
avoid an orphan diagnostic.
