---
name: wiki-maintenance
description: Find and assess Owly Wiki maintenance candidates such as broken links, orphan pages, unreachable pages, and stale content without changing files.
---

# Owly Wiki Maintenance Skill

Use this workflow to find and assess maintenance candidates without changing
the wiki.

## Workflow

1. Run `owly validate --details --format json` to collect diagnostics. Add
   `--stale-before YYYY-MM-DD` when an age threshold is requested.
2. Run `owly graph` to verify the evidence for broken references, orphan
   pages, and unreachable pages.
3. Run `owly validate` to verify frontmatter, types, values, and
   duplicate identifiers.
4. Report each candidate, its reason, path, and likely impact.
5. Wait for explicit approval before changing, moving, archiving, or deleting
   any file. The current CLI provides detection only.
