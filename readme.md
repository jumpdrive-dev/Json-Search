# JSON Search

Library for searching a JSON structure for certain paths.

> **Warning**
> This library is work in progress and should not be used in production yet.

## Usage

Because this library is still work in progress, it's not available on crates.io yet. But if you want to play around with
this unstable version, you can add this as a git dependency like so:

```toml
json_search = { git = "https://github.com/jumpdrive-dev/Json-Search", rev = "<commit to use>" }
```

## Features

This is a list of current and future features that this library supports:

- [x] Resolving paths for a [Serde JSON](https://github.com/serde-rs/json) based on a search path.
- [x] Support for additional resolution modes like optional and wildcard resolution.
- [ ] Ability to perform operations on a per-path basis.
- [ ] Ability to perform bulk operation based on a json search.
