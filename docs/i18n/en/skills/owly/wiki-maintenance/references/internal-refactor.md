# Internal graph refactor

Use this reference after an audit has identified a change inside one Wiki
root. Keep the first pass read-only.

## Classify before editing

- Broken link: confirm the target path and whether the link is intended to be relative to the current file.
- Orphan or unreachable page: confirm that it is not intentionally a leaf or an archive before proposing a link or move.
- Duplicate concept: compare titles, frontmatter identity, inbound links, and content. Do not merge based on title similarity alone.
- Missing relationship: add a link only when the source text provides evidence; do not create a dense speculative link graph.

## Change protocol

1. Show the candidate set and the proposed link, move, merge, or archive.
2. Obtain explicit approval for writes. A merge or deletion also needs the exact source and destination paths.
3. Preserve the configured frontmatter and required `entrance.md` path.
4. Run `owly validate --details --format json` and `owly graph` after edits.
5. Report unresolved references and any pages whose reachability changed.

Never rewrite all pages merely to normalize formatting. If a proposed change
would alter meaning, stop and ask for a semantic decision.
