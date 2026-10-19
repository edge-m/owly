# owly_types

`owly_types` defines the domain data exchanged by Owly's source and cache
libraries.

It contains `DocumentIndex`, indexed document, frontmatter, link, and source
reference types, together with the typed `FrontmatterValue` representation.
The crate does not perform file I/O or database access.

These types are primarily used internally by the other Owly crates. Most users
should install or depend on the [`owly`](https://crates.io/crates/owly)
command-line application instead.

## License

Licensed under the MIT License.
