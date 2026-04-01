# long_path

A [dylint](https://github.com/trailofbits/dylint) lint that warns on qualified
paths with more than three segments used outside of `use` statements.

### What it does

Checks for long qualified paths (e.g. `a::b::c::d::e`) in expressions, types,
and patterns — but not inside `use` items.

### Why is this bad?

Long qualified paths reduce readability. Prefer importing a prefix with `use`
and referring to the item with a shorter path.

### Example

```rust
fn make_map() -> std::collections::hash_map::HashMap<i32, i32> {
    Default::default()
}
```

Use instead:

```rust
use std::collections;

fn make_map() -> collections::hash_map::HashMap<i32, i32> {
    Default::default()
}
```

### Usage

Add the lint as a dylint library in your project's `Cargo.toml`:

```toml
[workspace.metadata.dylint]
libraries = [
    { path = "<path-to>/rust-lints/long_path" },
]
```

Then run:

```sh
cargo dylint long_path
```
