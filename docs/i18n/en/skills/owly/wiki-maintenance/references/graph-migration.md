# Graph-boundary migration

Owly treats each Wiki root and its configuration as one graph. A relative link
that escapes the current root is an external-graph link. Its target must be an
existing `entrance.md`; that existence check is the only required external
inspection.

1. Identify the source graph and the external target path without resolving the external graph's contents.
2. Check that the target file exists and is exactly the external graph's `entrance.md`.
3. Report the source link, resolved target, and whether the target exists.
4. Do not crawl, index, validate, or include nodes from the external graph.
5. Obtain approval before changing a cross-graph link or moving either entrance page.
6. After an approved change, validate the source graph and report the external existence check separately.

There is no multi-graph aggregate configuration or aggregate graph output in
this workflow. Keep graph names in their own configuration's `name` field.
