---
name: wiki-maintenance
description: Use when auditing or changing an Owly Wiki's existing structure, links, duplicates, or graph boundaries. Audit first; obtain explicit approval before writes.
---

# Owly Wiki Maintenance Skill

Use this workflow to separate read-only diagnosis from approved maintenance.

## Workflow

1. Read `<wiki-root>/entrance.md` and the graph's `owly.yml` before making a plan.
2. Run `owly validate --details --format json` to collect diagnostics. Add
   `--stale-before YYYY-MM-DD` when an age threshold is requested.
3. Run `owly graph` to verify the evidence for broken references, orphan
   pages, and unreachable pages. Check that `entrance.md` exists and that
   links escaping the Wiki root point only to an existing external
   `entrance.md`; do not inspect the external graph's contents.
4. Run `owly validate` to verify frontmatter, types, values, and
   duplicate identifiers.
5. Classify the request as an internal refactor or a graph-boundary migration
   and read the matching reference before proposing edits.
6. Report each candidate, its reason, path, likely impact, and proposed edits.
7. Wait for explicit approval before changing, moving, archiving, or deleting
   any file. Validate again after approved edits.

## References

- `references/internal-refactor.md` covers links, orphans, duplicates, and
  other changes within one graph.
- `references/graph-migration.md` covers links that escape one Wiki root and
  target another graph's `entrance.md`.
