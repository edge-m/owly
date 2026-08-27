# Development instructions

## Verification before finishing

After making changes, run the checks appropriate to the files and behavior that
changed. Do not skip verification when the change affects Rust code, the CLI,
configuration parsing, or tests.

- Rust source or tests: run `make pre-commit` (format check, lint, and tests).
- Formatting changes are needed: run `make format`, then `make format-check`.
- CLI behavior changes: run the relevant command manually in addition to the
  test suite.
- Makefile or documentation-only changes: run the relevant target or command
  when practical; at minimum, verify the changed examples and file paths.

If a check cannot be run, report which check was skipped and why. In the final
summary, report the checks that were run and their results.

## Available project commands

Use the Makefile targets from the repository root:

```sh
make format
make lint
make test
make pre-commit
make check
make build
```

Keep changes within the requested scope and avoid overwriting existing user
files without explicit permission.

## Documentation synchronization

`README.md` is the primary project README. Keep every language version under
`docs/i18n/` synchronized with it when changing project identity, usage,
commands, configuration examples, development instructions, or other user-
facing information. Update the relevant translations in the same change, and
add a note when a translation is intentionally incomplete.
