# Owly Philosophy

Owly is the structural foundation for Markdown wikis written by AI agents and maintained by teams.

Owly does not pretend to understand the meaning of knowledge, and it does not rebuild tools that already search text well. It makes the human-defined structure explicit and checks, quickly and reliably, whether an agent has followed it.

## Principles

### 1. Scale from the start

The wiki should be able to grow from a personal collection of notes into a large knowledge base maintained by multiple people and agents.

- Use Git for history, diffs, reviews, recovery, and conflict resolution.
- Keep each Graph bounded and self-contained instead of growing one huge Graph forever.
- Connect Graphs through entry points, not direct links to individual articles.
- Allow recursive nesting: a Graph can reference a Graph of Graphs, and so on.
- Use a SQLite index for frontmatter queries as the number of files grows.

Each Graph can be searched and validated locally, while the whole system grows by adding Graphs. SQLite is derived search data; Markdown and `owly.yml` remain the source of truth.

### 2. Stay understandable and interoperable

Owly should not create a world of its own. Use established tools first, and add Owly-specific behavior only where they are insufficient.

- Use `rg`, `fd`, `grep`, or an editor for body-text search.
- Use Owly for structured frontmatter queries and wiki validation.
- Define the wiki structure freely in `owly.yml` instead of imposing one fixed layout.
- Never infer rules from filenames or the accidental shape of existing pages.
- Do not check the meaning, truth, style, or writing quality of body text.

Users should be able to inspect the files directly and work with ordinary commands at any time.

### 3. Let agents write; do not let them define correctness

Agents should handle reading sources, summarizing, classifying, writing articles, and adding links. Their output should still be checked rather than trusted blindly.

1. A human defines the wiki contract in `owly.yml`.
2. An agent writes articles according to that contract.
3. Owly validates frontmatter, types, fields, links, orphan pages, and sources.
4. The agent or a human fixes the reported issues.

Owly is deterministic: the same configuration and files produce the same result. It also never edits files during validation. Validation and modification are separate responsibilities.

## The boundary Owly protects

| Area | Owner |
|---|---|
| Body-text search | `rg` / `fd` / `grep` / editor |
| Wiki structure | Human and `owly.yml` |
| Article creation and updates | AI agent and human |
| Structured frontmatter search | Owly and SQLite index |
| Structural, link, and source validation | Owly |
| History, review, and conflict resolution | Git |

Owly does not try to do everything. It works with existing tools to keep the structure of an agent-written wiki reliable over time.
