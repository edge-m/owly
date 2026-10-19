# Owly

![Owly logo](https://raw.githubusercontent.com/edge-m/owly/develop/docs/img/wide.webp)

Owly is a configuration-driven linter for structured Markdown wikis.

It is designed for wikis maintained with the help of AI Agents. Owly gives
those Agents a deterministic way to inspect wiki structure and validate pages.

The linter understands YAML frontmatter, document types, required fields,
field types, allowed values, links, orphan pages, and tracked source files.
Markdown body text is treated as opaque content; Owly does not judge its
meaning or writing quality.

The repository contains:

- `owly`: the command-line application and the recommended entry point
- `owly_core`: validation, diagnostics, configuration, and wiki-model logic
- `owly_source`: Markdown discovery, loading, and source metadata extraction
- `owly_cache`: SQLite-backed derived index and query support
- `owly_types`: shared domain types exchanged by the library crates
- Agent Skills for configuring, exploring, searching, and maintaining wikis
- End-to-end CLI tests and release automation

Owly is written in Rust and exposes stable exit codes for scripts and CI.
The project is currently version `0.1.1` and licensed under MIT.

## Published crates

The crates are maintained in one Cargo workspace and published individually.
The library crates are implementation-oriented building blocks; most users
should install or depend on `owly` rather than using them directly.

- [`owly`](https://crates.io/crates/owly): CLI application
- [`owly_core`](https://crates.io/crates/owly_core): validation and wiki model
- [`owly_source`](https://crates.io/crates/owly_source): source discovery and parsing
- [`owly_cache`](https://crates.io/crates/owly_cache): SQLite derived index
- [`owly_types`](https://crates.io/crates/owly_types): shared data types

## Agent Skills

The English Skills are the files distributed by `owly skill <directory>`.

- [using-wiki](skills/owly/using-wiki/SKILL.md): resolve the Wiki and choose an operation
- [wiki-config](skills/owly/wiki-config/SKILL.md): design and validate `owly.yml`
- [wiki-ingest](skills/owly/wiki-ingest/SKILL.md): ingest local source files and Git repositories
- [wiki-overview](skills/owly/wiki-overview/SKILL.md): inspect Wiki structure and current state
- [wiki-query](skills/owly/wiki-query/SKILL.md): answer questions using Wiki evidence
- [wiki-search](skills/owly/wiki-search/SKILL.md): search structured metadata and relationships
- [wiki-maintenance](skills/owly/wiki-maintenance/SKILL.md): audit and maintain existing Wiki structure

The [reference files](skills/owly/) include the supporting ingest and maintenance procedures. Japanese translations are available in the corresponding `docs/i18n/ja/skills/owly/` paths for readers; they are not written by the Skill distribution command.

For development commands and repository conventions, see [AGENTS.md](../../../AGENTS.md).
