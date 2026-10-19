# owly_cache

`owly_cache` provides Owly's SQLite-backed derived index.

The crate stores parsed document metadata, flattened frontmatter values,
resolved links, and tracked source references. [`CacheDb`](https://docs.rs/owly_cache/latest/owly_cache/struct.CacheDb.html)
provides database initialization, CRUD operations, and queries over the
derived data.

The cache is rebuildable and is not the source of truth for a wiki. Most users
should use the [`owly`](https://crates.io/crates/owly) command-line application
instead of depending on this crate directly.

## License

Licensed under the MIT License.
