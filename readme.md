# JSON Search

Library for searching a JSON structure for certain paths.

## Usage

To use this library, add the following git dependency to your `Cargo.toml` file:

```toml
json_search = { git = "https://github.com/jumpdrive-dev/Json-Search", tag = "1.0.1" }
```

## Features

This is a list of current and future features that this library supports:

- [x] Resolving paths for a [Serde JSON](https://github.com/serde-rs/json) based on a search path.
- [x] Support for additional resolution modes like optional and wildcard resolution.
- [x] Ability to perform operations on a per-path basis.
- [ ] Ability to perform bulk operation based on a json search.
