# Owly

![Owly logo](../../img/wide.webp)

Owly is a configuration-driven linter for structured Markdown wikis.

It is designed for wikis maintained with the help of AI Agents. Owly gives
those Agents a deterministic way to inspect wiki structure and validate pages.

The linter understands YAML frontmatter, document types, required fields,
field types, allowed values, links, orphan pages, and tracked source files.
Markdown body text is treated as opaque content; Owly does not judge its
meaning or writing quality.

The repository contains:

- `owly`: the command-line application
- `owly_core`: the reusable validation and wiki-model library
- Agent Skills for configuring, exploring, searching, and maintaining wikis
- End-to-end CLI tests and release automation

Owly is written in Rust and exposes stable exit codes for scripts and CI.
The project is currently version `0.1.0` and licensed under MIT.

For development commands and repository conventions, see [AGENTS.md](../../../AGENTS.md).
