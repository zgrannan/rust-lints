# collect_join

A [dylint](https://github.com/trailofbits/dylint) lint that warns when
`.collect().join(sep)` is used on an iterator.

### What it does

Checks for iterator chains that end with `.collect::<Vec<_>>().join(sep)`,
where the intermediate `Vec` is used only to call `join`.

### Why is this bad?

Collecting into a temporary `Vec` just to call `join` on it allocates
unnecessarily. The `itertools::Itertools::join` method performs the same
operation directly on the iterator without the intermediate allocation.

### Example

```rust
let s = words.iter().map(|w| w.as_str()).collect::<Vec<_>>().join(", ");
```

Use instead:

```rust
use itertools::Itertools;
let s = words.iter().map(|w| w.as_str()).join(", ");
```

### Usage

Add the lint as a dylint library in your project's `Cargo.toml`:

```toml
[workspace.metadata.dylint]
libraries = [
    { path = "<path-to>/rust-lints/collect_join" },
]
```

Then run:

```sh
cargo dylint collect_join
```
