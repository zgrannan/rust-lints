# ok_is_some

A [dylint](https://github.com/trailofbits/dylint) lint that warns when
`.ok().is_some()` is used on a `Result` and suggests `.is_ok()` instead.

### What it does

Checks for `Result` expressions where `.ok()` is immediately followed by
`.is_some()`.

### Why is this bad?

`.ok()` converts a `Result<T, E>` into an `Option<T>`, discarding the error.
Calling `.is_some()` on the result immediately discards the value too. The
direct equivalent — `.is_ok()` — expresses the intent more clearly and avoids
the intermediate `Option`.

### Example

```rust
let ok: bool = some_result.ok().is_some();
```

Use instead:

```rust
let ok: bool = some_result.is_ok();
```

### Usage

Add the lint as a dylint library in your project's `Cargo.toml`:

```toml
[workspace.metadata.dylint]
libraries = [
    { path = "<path-to>/rust-lints/ok_is_some" },
]
```

Then run:

```sh
cargo dylint ok_is_some
```
