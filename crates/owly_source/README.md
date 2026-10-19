# owly_source

`owly_source` handles the input side of Owly's indexing pipeline.

It discovers Markdown files, reads YAML frontmatter and Markdown links,
extracts typed source metadata, computes content hashes, and maintains the
directory-local file metadata cache. [`discover`](https://docs.rs/owly_source/latest/owly_source/fn.discover.html)
and [`load`](https://docs.rs/owly_source/latest/owly_source/fn.load.html) are
the main entry points.

The parsed results use the shared types from
[`owly_types`](https://crates.io/crates/owly_types). Most users should use the
[`owly`](https://crates.io/crates/owly) command-line application instead of
depending on this crate directly.

## License

Licensed under the MIT License.
