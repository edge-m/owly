---
name: wiki-config
description: Guide a user through designing a Owly Wiki configuration interactively, then write and validate owly.yml after explicit confirmation. Use when a user wants to create, draft, or revise Wiki schema settings, directory rules, document types, frontmatter fields, or validation options.
---

# Owly Wiki Configuration Builder

Help the user turn an idea for a structured Markdown Wiki into a valid
`owly.yml`. This is an interactive design workflow: collect requirements in
small batches, show the proposed configuration in plain language, obtain an
explicit approval, write the file, and validate it.

## Workflow

### 1. Inspect before asking

- Check the current directory and look for `owly.yml`.
- If a configuration exists, read it and treat the task as an update unless the
  user explicitly asks for a separate file. Preserve existing settings that the
  user did not ask to change.
- Inspect the repository's current Wiki state when it exists. Use
  `owly schema --config <path> --format json` and
  `owly overview directories` or `owly overview types` to avoid inventing
  names or breaking existing documents.
- If the CLI is unavailable, inspect `README.md` and the existing YAML schema;
  state that validation could not be run.

### 2. Conduct a focused conversation

Ask only a few related questions at a time. Start with:

1. Where should the Wiki live relative to the configuration file?
2. Which top-level directories should exist, and what is each for?
3. Which document types should be supported?

Then ask about fields for every document and fields specific to each type.
For each field, establish its name, value type, whether it is required, a
short description, and any allowed string values. Ask about nested arrays or
objects only when the user's domain needs them. Confirm unclear terminology
instead of guessing.

Use the following supported value types exactly: `string`, `date`, `datetime`,
`boolean`, `number`, `array`, and `object`.

Do not ask the user to repeat information already present in an existing
configuration. When sensible defaults are needed, propose them and label them
as defaults; do not silently add domain-specific fields.

### 3. Summarize and confirm

Before writing anything, present a compact summary containing:

- output path and resolved `wiki_root`;
- directory tree;
- document types and their descriptions;
- required fields shared by all documents;
- the required graph entry page, `<wiki-root>/entrance.md`;
- type-specific fields, optional fields, nested shapes, and allowed values;
- `wiki_lint` and `config_lint` settings.

Ask for explicit confirmation such as “Shall I create `owly.yml` with these
settings?” Do not create or overwrite a file before confirmation. If an
existing file would be overwritten, call that out separately and require
confirmation for the overwrite.

### 4. Write the YAML

Generate valid YAML with this top-level shape:

```yaml
wiki_root: .
directories: {}
mandatory_fields: {}
types: {}
wiki_lint: {}
config_lint: {}
```

Use `mandatory_fields` for fields required on every document. These rules do
not support `optional: true`. Use `types.<type>.fields` for type-specific
fields; those fields are required unless `optional: true` is present.

Use `items` only with `type: array`, and `fields` only with `type: object`.
Use `values` only with string fields. Keep descriptions useful but concise.
Preserve YAML quoting where names or values could be interpreted as numbers,
dates, booleans, or YAML syntax. Keep user-provided ordering where possible.

### 5. Validate and report

After writing, run:

```text
owly config validate --config <path>
owly validate --config <path>
```

If the Wiki is empty or intentionally incomplete, distinguish configuration
errors from document validation diagnostics. Fix configuration errors before
reporting success. For an update, also run a focused validation when available:

```text
owly validate --config <path> --file <path>
```

Report the written path, the final choices, commands run, and any remaining
document-level warnings or errors. Never claim the configuration is valid when
validation failed.

## Safety and interaction rules

- Never invent a type, field, relationship, directory, or requiredness rule.
- Never delete, move, or rewrite Wiki documents as part of configuration work.
- Do not silently overwrite an existing `owly.yml`; propose a backup/new path
  or obtain explicit overwrite approval.
- If the user's answers conflict, show the conflict and ask one targeted
  question rather than choosing arbitrarily.
- If the user asks only for a draft, stop after showing YAML and do not write it.
- If validation reveals a schema issue, explain the issue in user terms, propose
  the smallest correction, and ask before changing the approved design.
